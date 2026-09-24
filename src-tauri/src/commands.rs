//! The Tauri commands: the Rust functions the React UI can call.
//!
//! Each one is thin (ADR 0001 §2, architecture doc §1). It locks the
//! connection, calls one function in `waypoint_domain` or `waypoint_read`, and
//! turns any error into its `Display` text, because the UI can only receive
//! values that turn into JSON, and a plain string is what it shows the user.
//! Rules such as "a node needs a title" live in the domain, not here.
//!
//! See docs/learning/concepts/tauri-command.md for how a TypeScript
//! `invoke("create_node", …)` reaches `create_node` below, and
//! tauri-managed-state.md for `State<'_, Db>`.

use tauri::State;
use waypoint_read::SkillNode;

use crate::Db;

/// The message `.expect` panics with if the lock is poisoned.
///
/// A lock is poisoned only when some earlier code panicked while it held the
/// connection, possibly halfway through a write. Nothing sensible can happen
/// after that, so the command panics too rather than carry on with a
/// connection in an unknown state.
const POISONED: &str = "database lock poisoned: an earlier panic happened mid-write";

/// Adds a skill node and returns it as stored.
///
/// Called from the UI as `invoke("create_node", { title, description })`.
#[tauri::command]
pub fn create_node(
    db: State<'_, Db>,
    title: String,
    description: String,
) -> Result<SkillNode, String> {
    let conn = db.lock().expect(POISONED);
    // `map_err` changes only the error side of the `Result`: a `DomainError`
    // becomes its message, such as "A node needs a title.". A success passes
    // through untouched.
    waypoint_domain::create_node(&conn, &title, &description).map_err(|e| e.to_string())
}

/// Every skill node, oldest first.
///
/// Called from the UI as `invoke("list_nodes")`.
#[tauri::command]
pub fn list_nodes(db: State<'_, Db>) -> Result<Vec<SkillNode>, String> {
    let conn = db.lock().expect(POISONED);
    waypoint_read::list_nodes(&conn).map_err(|e| e.to_string())
}
