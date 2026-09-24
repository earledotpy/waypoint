//! The Tauri shell. Each command here will stay thin: parse the input, call one
//! function in `waypoint_domain` or `waypoint_read`, return the result
//! (ADR 0001 §2). There are no commands yet.
//!
//! At startup, `run` opens the database and hands the connection to Tauri
//! to hold. See docs/learning/concepts/tauri-managed-state.md.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
// Provides `app.path()` and `app.manage(…)` inside `setup`.
use tauri::Manager;

/// The one database connection, as Tauri holds it. A command will reach it
/// by asking for `tauri::State<'_, Db>`.
///
/// Tauri finds managed state by its exact type, and only when the command
/// runs, not when the code compiles. A command that asked for a different
/// type, such as a bare `Connection` or Tokio's `Mutex`, would compile and
/// then fail at runtime with "state not managed". Every command names this
/// alias, so they all ask for exactly the type `run` registers.
pub type Db = Mutex<Connection>;

/// The database file's name inside the app data folder.
const DATABASE_FILE_NAME: &str = "waypoint.sqlite3";

/// Creates `dir` if it's missing, then opens (or creates) the database in it
/// and brings its schema up to date.
///
/// The `Mutex` is there because Tauri only holds values that are safe to
/// share between threads, and a `Connection` on its own isn't.
///
/// Two kinds of error can happen here, an `io::Error` from making the folder
/// and a `DomainError` from `open`, so the error type is the one that holds
/// either. It's also the type `setup` returns, so `setup` passes it on as is.
fn open_database(dir: &Path) -> Result<Db, Box<dyn std::error::Error>> {
    // Like Python's `os.makedirs(dir, exist_ok=True)`: on a first launch the
    // folder doesn't exist yet, and on every later one it already does.
    std::fs::create_dir_all(dir)?;
    // `open` runs the migrations before it returns (sqlite-migrations.md), so
    // the connection Tauri will hold always has the current schema.
    let conn = waypoint_domain::open(&dir.join(DATABASE_FILE_NAME))?;
    Ok(Mutex::new(conn))
}

// `mobile_entry_point` lets the same `run` start the app on iOS and Android.
// On desktop it does nothing, and `main.rs` calls `run` directly.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // `setup` runs once at startup, before any command can be handled,
        // so it's where the database has to be opened.
        .setup(|app| {
            // The per-user app data folder, named after `identifier` in
            // tauri.conf.json. On Windows that's
            // %APPDATA%\com.earledotpy.waypoint.
            let dir = app.path().app_data_dir()?;
            // If the folder can't be made, or opening or migrating fails, `?`
            // returns the error from `setup`. Tauri then stops the app with
            // "Failed to setup app: …" rather than let it run on a database
            // it couldn't bring up to date (architecture doc §6: migrations
            // run before any command is accepted).
            let db = open_database(&dir)?;
            // Hands the connection to Tauri, which keeps it until the app
            // quits and gives it to any command that asks for `Db`.
            app.manage(db);
            Ok(())
        })
        // `generate_context!` reads tauri.conf.json and embeds the built
        // frontend (`dist/`) into the binary at compile time.
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_database_creates_missing_folder_and_file() {
        let root = tempfile::tempdir().unwrap();
        // A folder that doesn't exist yet, like app data on a first launch.
        let dir = root.path().join("app-data");

        open_database(&dir).unwrap();

        assert!(dir.join(DATABASE_FILE_NAME).is_file());
    }

    #[test]
    fn open_database_twice_keeps_existing_data() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("app-data");

        let first = open_database(&dir).unwrap();
        let conn = first.lock().unwrap();
        waypoint_domain::create_node(&conn, "Solve linear equations", "").unwrap();
        drop(conn);
        drop(first);

        // A second launch: the folder and file already exist, and the
        // migrations have already run on them.
        let second = open_database(&dir).unwrap();
        let conn = second.lock().unwrap();
        let nodes = waypoint_read::list_nodes(&conn).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].title, "Solve linear equations");
    }
}
