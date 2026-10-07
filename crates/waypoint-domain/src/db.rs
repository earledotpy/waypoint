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
    Migrations::new(vec![
        M::up(include_str!("../migrations/0001_create_skill_node.sql")),
        M::up(include_str!("../migrations/0002_create_skill_edge.sql")),
    ])
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
    // SQLite ignores foreign keys unless each connection turns them on.
    // skill_edge relies on them, and later tables (evidence records) will too.
    // See docs/learning/concepts/foreign-keys.md.
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

    /// Inserts a node with the given id, so an edge has something to point at.
    fn insert_node(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO skill_node (id, title, state, created_at)
             VALUES (?1, 'A skill', 'available', '2026-10-07T10:00:00.000Z')",
            [id],
        )
        .unwrap();
    }

    /// Tries to insert an edge and hands back SQLite's answer, so each test
    /// can check whether the database accepted or refused it.
    fn insert_edge(
        conn: &Connection,
        id: &str,
        from_node_id: &str,
        to_node_id: &str,
        edge_type: &str,
    ) -> rusqlite::Result<usize> {
        conn.execute(
            "INSERT INTO skill_edge (id, from_node_id, to_node_id, edge_type)
             VALUES (?1, ?2, ?3, ?4)",
            [id, from_node_id, to_node_id, edge_type],
        )
    }

    // Ids for the skill_edge tests. Real ids are UUIDv7s made by the domain
    // layer; these only need to be different from each other.
    const FOR_LOOP: &str = "0199a1b2-0000-7000-8000-000000000001";
    const FUNCTION: &str = "0199a1b2-0000-7000-8000-000000000002";
    const EDGE_1: &str = "0199a1b2-0000-7000-8000-0000000000e1";
    const EDGE_2: &str = "0199a1b2-0000-7000-8000-0000000000e2";

    #[test]
    fn open_in_memory_creates_every_table() {
        let conn = open_in_memory().unwrap();

        assert_eq!(user_version(&conn), 2);
        assert!(table_exists(&conn, "skill_node"));
        assert!(table_exists(&conn, "skill_edge"));
    }

    #[test]
    fn open_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("waypoint.db");

        let first = open(&path).unwrap();
        assert_eq!(user_version(&first), 2);
        drop(first);

        let second = open(&path).unwrap();
        assert_eq!(user_version(&second), 2);
        assert!(table_exists(&second, "skill_node"));
        assert!(table_exists(&second, "skill_edge"));
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

    #[test]
    fn skill_edge_rejects_an_unknown_node() {
        let conn = open_in_memory().unwrap();
        insert_node(&conn, FUNCTION);

        // FOR_LOOP was never inserted, so no node has that id.
        let result = insert_edge(&conn, EDGE_1, FOR_LOOP, FUNCTION, "hard_prerequisite");

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("FOREIGN KEY constraint failed"),
            "expected a foreign key failure, got: {error}"
        );
    }

    #[test]
    fn skill_edge_rejects_a_duplicate() {
        let conn = open_in_memory().unwrap();
        insert_node(&conn, FOR_LOOP);
        insert_node(&conn, FUNCTION);
        insert_edge(&conn, EDGE_1, FOR_LOOP, FUNCTION, "hard_prerequisite").unwrap();

        // Same from, to and type as EDGE_1, under a new id.
        let result = insert_edge(&conn, EDGE_2, FOR_LOOP, FUNCTION, "hard_prerequisite");

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("UNIQUE constraint failed"),
            "expected a UNIQUE failure, got: {error}"
        );
    }

    #[test]
    fn skill_edge_allows_a_prerequisite_and_a_recommendation_between_the_same_nodes() {
        let conn = open_in_memory().unwrap();
        insert_node(&conn, FOR_LOOP);
        insert_node(&conn, FUNCTION);

        insert_edge(&conn, EDGE_1, FOR_LOOP, FUNCTION, "hard_prerequisite").unwrap();
        insert_edge(&conn, EDGE_2, FOR_LOOP, FUNCTION, "soft_recommendation").unwrap();
    }

    #[test]
    fn skill_edge_rejects_a_self_edge() {
        let conn = open_in_memory().unwrap();
        insert_node(&conn, FOR_LOOP);

        let result = insert_edge(&conn, EDGE_1, FOR_LOOP, FOR_LOOP, "hard_prerequisite");

        let error = result.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("CHECK constraint failed: from_node_id <> to_node_id"),
            "expected the self-edge CHECK to fail, got: {error}"
        );
    }

    #[test]
    fn skill_edge_rejects_unknown_edge_type() {
        let conn = open_in_memory().unwrap();
        insert_node(&conn, FOR_LOOP);
        insert_node(&conn, FUNCTION);

        let result = insert_edge(&conn, EDGE_1, FOR_LOOP, FUNCTION, "dependency");

        let error = result.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("CHECK constraint failed: edge_type"),
            "expected the edge_type CHECK to fail, got: {error}"
        );
    }
}
