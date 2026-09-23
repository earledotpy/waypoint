//! Opening the database and bringing its schema up to date.
//!
//! Every connection Waypoint uses is made by `open` (or `open_in_memory` in
//! tests), so every connection gets the same settings and the same schema
//! (architecture doc §2 and §6). Nothing else should call rusqlite's
//! `Connection::open` directly.
//! See docs/learning/concepts/sqlite-migrations.md, and borrowing.md for
//! the `&Path` / `&mut conn` / `mut conn` in the signatures below.

use std::path::Path;

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

use crate::error::DomainError;

/// Every migration, oldest first. Migration N is the Nth entry, and after it
/// runs SQLite's `PRAGMA user_version` is N. Only ever append to this list:
/// editing or reordering an entry that has already run on someone's database
/// would leave their schema different from a fresh one.
///
/// `include_str!` pastes the `.sql` file into the program when it's compiled,
/// so the app never has to find the file on disk at runtime.
fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!(
        "../migrations/0001_create_skill_node.sql"
    ))])
}

/// Opens (or creates) the database file at `path`, ready to use.
pub fn open(path: &Path) -> Result<Connection, DomainError> {
    let conn = Connection::open(path)?;
    prepare(conn)
}

/// Opens a fresh database that lives only in memory and disappears when the
/// connection is dropped. For tests: same settings, same schema, no file.
pub fn open_in_memory() -> Result<Connection, DomainError> {
    let conn = Connection::open_in_memory()?;
    prepare(conn)
}

/// The settings and migrations every connection gets, whichever way it was
/// opened.
fn prepare(mut conn: Connection) -> Result<Connection, DomainError> {
    // SQLite ignores foreign keys unless each connection turns them on. Later
    // tables (edges, evidence records) rely on them.
    conn.pragma_update(None, "foreign_keys", "ON")?;
    // Write-ahead logging: readers don't block the writer, so a read (for
    // example, the UI refreshing a list) never waits for a write to finish.
    // An in-memory database has no file to log beside, so SQLite quietly
    // keeps its "memory" mode there.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    // Runs every migration newer than the database's `user_version`. It runs
    // before `open` returns, so no command ever sees an old schema.
    migrations().to_latest(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user_version(conn: &Connection) -> i64 {
        conn.query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap()
    }

    fn table_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type = 'table' AND name = ?1",
            [name],
            |row| row.get::<_, i64>(0),
        )
        .unwrap()
            == 1
    }

    #[test]
    fn open_in_memory_creates_skill_node_table() {
        let conn = open_in_memory().unwrap();

        assert_eq!(user_version(&conn), 1);
        assert!(table_exists(&conn, "skill_node"));
    }

    #[test]
    fn open_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("waypoint.db");

        let first = open(&path).unwrap();
        assert_eq!(user_version(&first), 1);
        drop(first);

        let second = open(&path).unwrap();
        assert_eq!(user_version(&second), 1);
        assert!(table_exists(&second, "skill_node"));
    }

    #[test]
    fn open_turns_on_foreign_keys_and_wal() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open(&dir.path().join("waypoint.db")).unwrap();

        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1);
        assert_eq!(journal_mode, "wal");
    }

    #[test]
    fn migrations_are_valid() {
        migrations().validate().unwrap();
    }

    #[test]
    fn skill_node_rejects_unknown_state() {
        let conn = open_in_memory().unwrap();

        let result = conn.execute(
            "INSERT INTO skill_node (id, title, state, created_at)
             VALUES ('0199a1b2-0000-7000-8000-000000000001', 'Solve linear equations',
                     'stale', '2026-09-23T10:00:00.000Z')",
            [],
        );

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("CHECK constraint failed"),
            "expected a CHECK failure, got: {error}"
        );
    }
}
