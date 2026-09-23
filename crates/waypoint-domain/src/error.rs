//! The one error type every `waypoint-domain` function returns.
//!
//! Each variant wraps the error from one library that can fail, so the app
//! has one error type to handle whichever library failed underneath. The
//! `impl From<…>` blocks at the bottom exist so domain code can use `?` on
//! library calls. They're written out by hand rather than generated with
//! `thiserror` (`AGENTS.md`, "Plain construction").
//! See docs/learning/concepts/result-and-question-mark.md,
//! traits-and-impl.md and enums-and-match.md.

use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    /// SQLite refused something: the file couldn't be opened, a statement
    /// failed, or a CHECK constraint rejected a row.
    Database(rusqlite::Error),
    /// Bringing the schema up to date failed. The database is left exactly
    /// as it was, because the pending migrations share one transaction.
    Migration(rusqlite_migration::Error),
    /// A skill node's title was empty or only spaces. Checked before SQLite
    /// is touched, so the user gets this message rather than a CHECK
    /// constraint failure. It wraps no library error.
    EmptyTitle,
}

// `Display` is the human-readable message, what `print(str(e))` shows in
// Python. The app will pass it on to the UI.
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Database(e) => write!(f, "database error: {e}"),
            DomainError::Migration(e) => write!(f, "migration error: {e}"),
            DomainError::EmptyTitle => write!(f, "a skill node needs a title"),
        }
    }
}

// Marks `DomainError` as a standard Rust error. `source` hands back the
// library error it wraps, like Python's `__cause__` after `raise … from e`,
// so the full chain of causes is never lost.
impl std::error::Error for DomainError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DomainError::Database(e) => Some(e),
            DomainError::Migration(e) => Some(e),
            // Waypoint's own rule failed, not a library, so there's no
            // underlying cause to hand back.
            DomainError::EmptyTitle => None,
        }
    }
}

impl From<rusqlite::Error> for DomainError {
    fn from(e: rusqlite::Error) -> Self {
        DomainError::Database(e)
    }
}

impl From<rusqlite_migration::Error> for DomainError {
    fn from(e: rusqlite_migration::Error) -> Self {
        DomainError::Migration(e)
    }
}
