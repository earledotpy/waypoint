//! Creating skill nodes.
//! See docs/learning/concepts/borrowing-in-signatures.md for the `&Connection`
//! and `&str` in `create_node`'s signature.

use rusqlite::Connection;
use uuid::Uuid;
use waypoint_read::SkillNode;

use crate::error::DomainError;

/// Adds a new skill node and returns it exactly as it was stored.
pub fn create_node(
    conn: &Connection,
    title: &str,
    description: &str,
) -> Result<SkillNode, DomainError> {
    let title = title.trim();
    let description = description.trim();
    // Once trimmed, a title of spaces is empty too. Stop here, before SQLite:
    // the table's CHECK would also refuse it, but with a database error the
    // learner can't act on.
    if title.is_empty() {
        return Err(DomainError::EmptyTitle);
    }

    // A UUIDv7 starts with the current time, so ids sort in creation order
    // (ADR 0002).
    let id = Uuid::now_v7().to_string();

    // A new node starts `available`. The transition table (architecture doc
    // §2) says a new node with no unmet prerequisites starts Available, and
    // edges (so prerequisites) don't exist until milestone 2. Milestone 2
    // will work out Locked or Available here instead.
    //
    // TODO(milestone 2): I7. Every state change, creation included, must also
    // write one `node_state_event` row. That table arrives in milestone 2,
    // whose migration backfills a creation event (`from_state` NULL, `actor`
    // `human`) for every node made before it.
    //
    // SQLite supplies `created_at`, so every timestamp comes from one clock
    // in one format. `RETURNING *` hands back the row as stored, so the
    // caller sees exactly what's in the database, not what we meant to write.
    let node = conn.query_row(
        "INSERT INTO skill_node (id, external_id, title, description, state, created_at)
         VALUES (?1, NULL, ?2, ?3, 'available', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         RETURNING *",
        (&id, title, description),
        SkillNode::from_row,
    )?;
    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;
    use waypoint_read::list_nodes;

    #[test]
    fn create_node_rejects_a_blank_title() {
        let conn = open_in_memory().unwrap();

        for blank in ["", "   "] {
            let result = create_node(&conn, blank, "Some description");
            assert!(
                matches!(result, Err(DomainError::EmptyTitle)),
                "expected EmptyTitle for {blank:?}, got {result:?}"
            );
        }
        assert_eq!(list_nodes(&conn).unwrap(), vec![]);
    }

    #[test]
    fn create_node_gives_each_node_a_new_id() {
        let conn = open_in_memory().unwrap();

        // Titles aren't unique: the id is a node's identity.
        let first = create_node(&conn, "Read a bar chart", "").unwrap();
        let second = create_node(&conn, "Read a bar chart", "").unwrap();

        assert_ne!(first.id, second.id);
        assert_eq!(list_nodes(&conn).unwrap().len(), 2);
    }

    #[test]
    fn list_nodes_on_an_empty_database_is_empty() {
        let conn = open_in_memory().unwrap();

        assert_eq!(list_nodes(&conn).unwrap(), vec![]);
    }

    #[test]
    fn list_nodes_returns_nodes_oldest_first() {
        let conn = open_in_memory().unwrap();
        let first = create_node(&conn, "Count to ten", "").unwrap();
        let second = create_node(&conn, "Add single digits", "").unwrap();
        let third = create_node(&conn, "Subtract single digits", "").unwrap();

        // Made in quick succession, so some may share a `created_at`
        // millisecond: this also checks the tie-break on `id`.
        assert_eq!(list_nodes(&conn).unwrap(), vec![first, second, third]);
    }

    #[test]
    fn create_node_returns_an_available_node() {
        let conn = open_in_memory().unwrap();

        let node = create_node(&conn, "  Solve linear equations  ", "One variable").unwrap();

        assert_eq!(node.id.len(), 36, "expected a UUID, got {:?}", node.id);
        assert_eq!(node.title, "Solve linear equations");
        assert_eq!(node.description, "One variable");
        assert_eq!(node.state, "available");
        assert_eq!(node.external_id, None);
        assert_eq!(node.retired_at, None);
        assert!(
            node.created_at.ends_with('Z'),
            "expected UTC, got {:?}",
            node.created_at
        );
    }
}
