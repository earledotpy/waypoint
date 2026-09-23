//! The skill node type, and the query that reads nodes back out.
//! See docs/learning/concepts/struct-and-derive.md for `struct`, `Option`
//! and `#[derive(…)]`.

use rusqlite::{Connection, Row};
use serde::Serialize;

/// One row of the `skill_node` table (migration 0001), as Rust sees it.
///
/// The field names are the column names, kept in snake_case with no serde
/// renaming, so one word (say `created_at`) can be searched for across SQL,
/// Rust and, later, TypeScript.
///
/// `Serialize` is what lets the app send a node to the frontend as JSON.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillNode {
    /// UUIDv7 as text (ADR 0002).
    pub id: String,
    /// `None` for a node made in the app; set only by a curriculum import.
    pub external_id: Option<String>,
    pub title: String,
    pub description: String,
    /// One of `locked`, `available`, `in_progress`, `evidenced`. A plain
    /// `String` for now: milestone 2 (the state machine) replaces it with an
    /// enum, so the compiler rejects any other value.
    pub state: String,
    /// ISO 8601 UTC with milliseconds, e.g. `2026-09-22T20:53:40.123Z`.
    pub created_at: String,
    /// `None` unless the node has been retired.
    pub retired_at: Option<String>,
}

impl SkillNode {
    /// Builds a `SkillNode` from one result row that has every `skill_node`
    /// column. Both the domain's `INSERT … RETURNING` and `list_nodes` use
    /// it, so a row becomes a node the same way wherever it was read.
    ///
    /// Columns are looked up by name, not position, so a query may list them
    /// in any order. A `NULL` column becomes `None` in an `Option` field; in
    /// a plain `String` field it's an error, which the `NOT NULL` columns
    /// rule out.
    pub fn from_row(row: &Row) -> rusqlite::Result<SkillNode> {
        Ok(SkillNode {
            id: row.get("id")?,
            external_id: row.get("external_id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            state: row.get("state")?,
            created_at: row.get("created_at")?,
            retired_at: row.get("retired_at")?,
        })
    }
}

/// Every skill node, oldest first, retired ones included.
///
/// Ties on `created_at` (two nodes in the same millisecond) fall back to
/// `id`, which is a UUIDv7 and so also sorts by creation time. That keeps the
/// order the same on every call.
///
/// Returns plain `rusqlite::Result`, not `DomainError`: this crate can't see
/// the domain (ADR 0001 §2, "Read errors"), and domain code that calls this
/// with `?` still gets a `DomainError`.
///
/// Its tests are in `crates/waypoint-domain/src/node.rs`, not here. Only the
/// domain can create the schema and the rows a test needs, and this crate may
/// not depend on the domain, even for tests (ADR 0001 §2 and §4).
pub fn list_nodes(conn: &Connection) -> rusqlite::Result<Vec<SkillNode>> {
    let mut statement = conn.prepare(
        "SELECT id, external_id, title, description, state, created_at, retired_at
         FROM skill_node
         ORDER BY created_at, id",
    )?;
    // `query_map` runs `from_row` on each row as it's read. Each result is
    // itself a `Result`, because any one row could fail to convert, and
    // `collect` stops at the first failure.
    let nodes = statement
        .query_map([], SkillNode::from_row)?
        .collect::<rusqlite::Result<Vec<SkillNode>>>()?;
    Ok(nodes)
}
