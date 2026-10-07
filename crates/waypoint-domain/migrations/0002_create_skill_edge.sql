-- A skill edge: one Prerequisite or Recommendation between two nodes.
-- "Write a for loop is a prerequisite of Write a function" is one row.
-- The conventions behind every line are in docs/adr/0002-table-conventions.md.
-- As in 0001, the constraints are a backstop. The Rust domain layer is the
-- primary guard and refuses a bad edge with a clear message first.
CREATE TABLE skill_edge (
    -- UUIDv7 text (ADR 0002 rule 1). Node state changes will point at it
    -- (node_state_event.cause_ref, the next migration).
    id           TEXT PRIMARY KEY NOT NULL,

    -- Direction: for "A is a prerequisite of B", from_node_id is A and
    -- to_node_id is B. The arrow points the way you learn, from what comes
    -- first to what it unlocks (issue #41, "How I5 checks for a cycle").
    --
    -- REFERENCES makes each a foreign key: it must hold the id of a node that
    -- really exists (docs/learning/concepts/foreign-keys.md). SQLite ignores
    -- REFERENCES unless the connection turns foreign keys on, so `prepare` in
    -- src/db.rs runs `conn.pragma_update(None, "foreign_keys", "ON")` on every
    -- connection. There's no ON DELETE (what to do with an edge when its node
    -- is deleted), because nodes are never deleted: Retired is a mark
    -- (skill_node.retired_at), not a deletion.
    from_node_id TEXT NOT NULL REFERENCES skill_node (id), -- the prerequisite: learned first
    to_node_id   TEXT NOT NULL REFERENCES skill_node (id), -- the dependent: waits for from_node_id

    -- A Prerequisite or a Recommendation (CONTEXT.md).
    edge_type    TEXT NOT NULL CHECK (edge_type IN ('hard_prerequisite', 'soft_recommendation')),

    -- Why the edge exists, in plain words. NULL for edges made in the app;
    -- the curriculum import (milestone 6) fills it.
    reason       TEXT,

    -- No duplicate edges. edge_type is part of the rule, so the same two
    -- nodes may have one Prerequisite and one Recommendation, but not two of
    -- either.
    UNIQUE (from_node_id, to_node_id, edge_type),

    -- No edge from a node to itself. The domain refuses these first, with a
    -- clear message (milestone 2, step 6); this catches a bug that slips past.
    CHECK (from_node_id <> to_node_id)
) STRICT;
