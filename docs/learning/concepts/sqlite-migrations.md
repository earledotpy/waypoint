# SQLite migrations

## In one line

A migration is one numbered, never-edited step that changes the database's schema, and SQLite's `PRAGMA user_version` records how many steps a given database file has already had.

## Where it appears here

- `crates/waypoint-domain/migrations/0001_create_skill_node.sql`: migration 1, which creates the `skill_node` table.
- `crates/waypoint-domain/src/db.rs`, `migrations`: the ordered list of every migration.
- `crates/waypoint-domain/src/db.rs`, `prepare`: calls `to_latest`, which runs the missing migrations every time a connection is opened.
- The tests in `db.rs`: `migrations_are_valid`, `open_in_memory_creates_skill_node_table` and `open_is_idempotent`.

## What it does

The schema (the tables and their columns) changes as Waypoint grows, but the learner's database file already exists with the old schema and real data in it. Deleting it and starting again isn't an option. So each change is written as a small script that goes from version N to version N+1, and the scripts are run in order.

SQLite has a built-in slot for "which version is this file at": `PRAGMA user_version`, a single integer stored in the file's header. It starts at 0. The `rusqlite_migration` library uses it like this:

1. Read `user_version`. Say it's 0.
2. Run every migration after that one, in order. For a brand-new file that's `0001_create_skill_node.sql`. They all run inside one transaction, so either all of them happen or none do.
3. Set `user_version` to the number of the last migration run (here, 1).

Opening the same file again finds `user_version = 1`, sees that nothing is newer, and does nothing. That's what `open_is_idempotent` checks. When a later issue adds `0002_….sql` to the list in `migrations`, a file at version 1 gets only migration 2 the next time it's opened.

Two rules follow:

- **Only append.** A migration that has run on someone's file is never edited or reordered. Their `user_version` says "1 is done", so an edited migration 1 would never run for them, and their schema would silently differ from a fresh install. A fix is always a new migration.
- **`validate()` in a test.** `migrations_are_valid` runs every migration on a throwaway database, so broken SQL fails a test instead of the app's start-up.

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
- SQLite documentation, [`PRAGMA user_version`](https://www.sqlite.org/pragma.html#pragma_user_version)
