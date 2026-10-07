# SQLite migrations

## In one line

A migration is one numbered, never-edited step that changes the database's schema, and SQLite's `PRAGMA user_version` records how many steps a given database file has already had.

## Where it appears here

- `crates/waypoint-domain/migrations/0001_create_skill_node.sql`: migration 1, which creates the `skill_node` table.
- `crates/waypoint-domain/migrations/0002_create_skill_edge.sql`: migration 2, which creates the `skill_edge` table.
- `crates/waypoint-domain/src/db.rs`, `migrations`: the ordered list of every migration.
- `crates/waypoint-domain/src/db.rs`, `prepare`: calls `to_latest`, which runs the missing migrations every time a connection is opened.
- The tests in `db.rs`: `migrations_are_valid`, `open_in_memory_creates_every_table` and `open_is_idempotent`.

## What it does

The schema (the tables and their columns) changes as Waypoint grows, but the learner's database file already exists with the old schema and real data in it. Deleting it and starting again isn't an option. So each change is written as a small script that goes from version N to version N+1, and the scripts are run in order.

SQLite has a built-in slot for "which version is this file at": `PRAGMA user_version`, a single integer stored in the file's header. It starts at 0. The `rusqlite_migration` library uses it like this:

1. Read `user_version`. Say it's 0.
2. Run every migration after that one, in order. For a brand-new file that's `0001_create_skill_node.sql`, then `0002_create_skill_edge.sql`. They all run inside one transaction, so either all of them happen or none do.
3. Set `user_version` to the number of the last migration run (here, 2).

Opening the same file again finds `user_version = 2`, sees that nothing is newer, and does nothing. That's what `open_is_idempotent` checks. A file made in milestone 1 is at version 1, because only migration 1 existed then. The next time it's opened, it gets only migration 2 and ends at version 2, just like a brand-new file.

Two rules follow:

- **Only append.** A migration that has run on someone's file is never edited or reordered. Their `user_version` says "1 is done", so an edited migration 1 would never run for them, and their schema would silently differ from a fresh install. A fix is always a new migration.
- **`validate()` in a test.** `migrations_are_valid` runs every migration on a throwaway database, so broken SQL fails a test instead of the app's start-up.

### Rules across several columns

Migration 0001's [constraints](../glossary.md#constraint) each look at one column: `title` is not blank, `state` is one of four words. Migration 0002 adds two that look at several columns of the same row at once. They're written on their own lines after the columns, not beside one column.

- `UNIQUE (from_node_id, to_node_id, edge_type)`: no two rows may have the same **combination** of these three values. Each value on its own may repeat. So "for loop → function, Prerequisite" and "for loop → function, Recommendation" can both exist, because their `edge_type` differs, but a second "for loop → function, Prerequisite" fails with `UNIQUE constraint failed`. In Python terms, it's like keeping a set of `(from_node_id, to_node_id, edge_type)` tuples and refusing a tuple that's already in it.
- `CHECK (from_node_id <> to_node_id)`: `<>` is SQL for "not equal" (Python's `!=`). The check compares two columns of the same row, so it refuses an edge from a node to itself, and the error names the rule: `CHECK constraint failed: from_node_id <> to_node_id`.

The tests `skill_edge_rejects_a_duplicate`, `skill_edge_allows_a_prerequisite_and_a_recommendation_between_the_same_nodes` and `skill_edge_rejects_a_self_edge` in `db.rs` show each one at work. The two `REFERENCES` in the same migration are a different kind of rule, one that looks at another table: see [Foreign keys](foreign-keys.md).

## Python comparison

This is the same idea as Alembic (with SQLAlchemy) or Django migrations: numbered scripts, applied in order, with the database remembering which have run.

Where it differs:

- **Where the version lives.** Alembic and Django keep a table (`alembic_version`, `django_migrations`). SQLite already has `user_version` in its file header, so `rusqlite_migration` needs no extra table.
- **Nothing is generated.** Django's `makemigrations` writes the migration by comparing your model classes with the database. Here there are no model classes to compare. Each migration is hand-written SQL, and the reviewer reads exactly what will run.
- **No `downgrade`.** Alembic migrations usually have an `upgrade()` and a `downgrade()`. Waypoint only moves forward (`M::up`). Undoing a change is a new migration.
- **Who runs them.** Django migrations are a command you run (`manage.py migrate`) before starting the server. Waypoint is a desktop app with no separate deploy step, so `open` runs them itself, every time.

## Why this code uses it

Architecture doc §6 says migrations run "at Rust-layer startup before any command is accepted". Putting the call inside `open` makes that true by construction: Waypoint gets every writable `Connection` from `open` (and nothing else calls rusqlite's `Connection::open`), and `open` doesn't return until the schema is current. So no command, now or later, can ever run against an older schema, and nobody has to remember to call a "migrate" step first.

If a migration fails, `open` returns `DomainError::Migration` and the app doesn't start working on a half-updated database. The pending migrations share one transaction, so the file is left exactly as it was before `open` was called, never halfway.

## See also

- [ADR 0002: table conventions](../../adr/0002-table-conventions.md), rule 5: one `.sql` file per migration.
- [`Result` and `?`](result-and-question-mark.md): how a failed migration becomes a `DomainError`.
- [Foreign keys](foreign-keys.md): the `REFERENCES` in migration 0002.
- SQLite documentation, [`PRAGMA user_version`](https://www.sqlite.org/pragma.html#pragma_user_version)
