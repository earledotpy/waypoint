-- A skill node: one thing to learn, and where the learner stands with it.
-- The conventions behind every line are in docs/adr/0002-table-conventions.md.
-- The CHECK constraints are a backstop. The Rust domain layer is the primary
-- guard (architecture doc §2 preamble); these catch a bug that slips past it.
CREATE TABLE skill_node (
    id          TEXT PRIMARY KEY NOT NULL,     -- UUIDv7 text; never reused (I4)
    external_id TEXT UNIQUE,                   -- NULL for nodes authored in-app
    title       TEXT NOT NULL CHECK (length(trim(title)) > 0),
    description TEXT NOT NULL DEFAULT '',
    state       TEXT NOT NULL CHECK (state IN ('locked', 'available', 'in_progress', 'evidenced')),
    created_at  TEXT NOT NULL,                 -- ISO 8601 UTC, e.g. 2026-09-22T20:53:40.123Z
    retired_at  TEXT                           -- NULL unless retired
) STRICT;
