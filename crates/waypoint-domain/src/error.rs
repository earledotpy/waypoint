//! The one error type every `waypoint-domain` function returns.
//!
//! Each variant wraps the error from one library that can fail. The
//! `impl From<…>` blocks at the bottom are what let a domain function write
//! `?` after a rusqlite or rusqlite_migration call: `?` sees the library's
//! error, finds the matching `From`, and converts it into a `DomainError`.
//! They're written out by hand rather than generated with `thiserror`
//! (`AGENTS.md`, "Plain construction").
//! See docs/learning/concepts/result-and-question-mark.md.

use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    /// SQLite refused something: the file couldn't be opened, a statement
    /// failed, or a CHECK constraint rejected a row.
    Database(rusqlite::Error),
    /// Bringing the schema up to date failed. The database is left exactly
    /// as it was, because the pending migrations share one transaction.
    Migration(rusqlite_migration::Error),
}

// `Display` is the human-readable message, what `print(str(e))` shows in
// Python. The app will pass it on to the UI.
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Database(e) => write!(f, "database error: {e}"),
            DomainError::Migration(e) => write!(f, "migration error: {e}"),
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
