//! Waypoint's shared types and read-only queries.
//!
//! Owns the shared types (`SkillNode`, …) and the public read queries. From
//! Tier 1, also a read-only `open` (`SQLITE_OPEN_READ_ONLY`) for advisory.
//!
//! This crate has no workspace dependencies, so any crate can depend on it
//! without gaining a way to write. See ADR 0001 §2.
