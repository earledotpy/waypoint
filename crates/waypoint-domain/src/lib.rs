//! Waypoint's domain: the one writer to SQLite.
//!
//! Owns every write to SQLite: `open` + migrations, the state machine, I1–I7,
//! the artifact store, and inserting advisory annotations.
//!
//! Invariants I1–I7 are enforced here and nowhere else (`AGENTS.md`). See
//! ADR 0001 §2 and §3.
