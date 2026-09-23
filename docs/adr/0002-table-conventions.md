# ADR 0002 — Table conventions

Status: Proposed
Date: 2026-09-23

## In plain words

Every Waypoint table follows five rules. Each `id` is a UUIDv7 stored as text. Each timestamp is ISO 8601 UTC text with milliseconds, like `2026-09-22T20:53:40.123Z`. Every table is `STRICT`, so SQLite refuses a value of the wrong type instead of quietly storing it. Every enum column (such as `skill_node.state`) gets a `CHECK` constraint listing its allowed values, as a backstop behind the Rust domain layer, which is the real guard. And every migration is its own `.sql` file in `crates/waypoint-domain/migrations/`. This rules out auto-increment integer IDs, random (v4) UUIDs, ULIDs, Unix-number timestamps, SQLite's default loose typing, and SQL written inside Rust strings.

## Context

`skill_node` is the first table (issue #16), and every later table will copy it. So the choices made here are really choices for the whole schema, and it's cheaper to write them down once than to argue them again for each table.

The facts that shape the decision:

- **Evidence must outlive edits (I4).** A node's `id` is never reused, even after retirement, so old evidence records still point at the right node. IDs also leave the database: curriculum import (`external_id`), the portable backup archive (architecture doc §6) and, in Tier 2, merging. An ID that is only unique *inside one file* doesn't survive that.
- **Append-only tables should read back in order.** `node_state_event` (I7) and `review_log` only ever grow, and they're mostly read oldest-first or newest-first.
- **SQLite is loosely typed by default.** A column declared `INTEGER` will happily store the text `'abc'`. SQLite also turns some constraints off unless asked (foreign keys need a per-connection `PRAGMA`, see `db.rs`).
- **The Rust domain layer is the primary guard** (architecture doc §2 preamble). Database constraints exist to catch a bug in that layer, not to replace it.
- **The reader is learning** (`AGENTS.md`). Someone opening the database in a browser tool, or reading a migration, should understand what they see without a decoder.
- `rusqlite` is built with `bundled`, so Waypoint always runs a known, recent SQLite (well past 3.37, the version that added `STRICT`).

## Decision

### 1. IDs are UUIDv7, stored as TEXT

`id TEXT PRIMARY KEY NOT NULL`, holding the standard 36-character form, e.g. `0199a1b2-6c3e-7a51-9f2d-3b8e4c1d0a77`. The Rust layer generates it. SQLite never invents IDs.

A UUIDv7 (RFC 9562) starts with the creation time in milliseconds and ends with random bits. So two IDs made anywhere never collide in practice, *and* sorting by `id` sorts by creation time.

### 2. Timestamps are ISO 8601 UTC text, with milliseconds

Always exactly this shape: `YYYY-MM-DDTHH:MM:SS.sssZ`. Always UTC (the `Z`), always three digits of milliseconds. Because every value has the same length and layout, sorting the text sorts the times correctly, and SQLite's own date functions (`datetime()`, `julianday()`) understand it. Converting to the learner's local time is the UI's job.

### 3. Every table is `STRICT`

`CREATE TABLE … ( … ) STRICT;`. A `TEXT` column then only accepts text (or `NULL` where allowed), and an `INTEGER` column only integers. A wrong-typed insert fails loudly.

### 4. Enums get a `CHECK` constraint, as a backstop

`state TEXT NOT NULL CHECK (state IN ('locked', 'available', 'in_progress', 'evidenced'))`. The Rust layer only ever writes values from a Rust `enum`, so in correct code the `CHECK` never fires. It's there for incorrect code: a typo in a hand-written `UPDATE`, or a future migration that forgets a value. The same goes for simple row rules such as "title isn't blank".

The `CHECK` constraints don't enforce the invariants I1–I7. Those depend on other rows and on *who* is writing, which a `CHECK` can't see. They live only in `waypoint-domain` (`AGENTS.md`).

### 5. One `.sql` file per migration

`crates/waypoint-domain/migrations/NNNN_<what_it_does>.sql`, numbered from `0001`. `db.rs` lists them in order and pastes each into the program with `include_str!`, so nothing is looked up on disk at runtime. A migration that has shipped is never edited: a fix is a new migration. See [SQLite migrations](../learning/concepts/sqlite-migrations.md).

### How this reads in `0001_create_skill_node.sql`

| Line | Why it's there |
|---|---|
| `id TEXT PRIMARY KEY NOT NULL` | Rule 1. In an ordinary SQLite table a text primary key may, for historical reasons, be `NULL`. `STRICT` already forbids that, but `NOT NULL` is written out anyway so the rule is visible without knowing the quirk. |
| `external_id TEXT UNIQUE` | Matches a node to its entry in an imported curriculum file. `UNIQUE` means two nodes can't claim the same curriculum entry. It's nullable because in-app nodes have none, and `UNIQUE` allows many `NULL`s. |
| `title TEXT NOT NULL CHECK (length(trim(title)) > 0)` | Rule 4. A node must have a real title. `trim` makes `'   '` count as blank too. |
| `description TEXT NOT NULL DEFAULT ''` | Markdown. An empty description is fine, so the default is `''` rather than `NULL`: code reading it never has to handle "missing" separately from "empty". |
| `state TEXT NOT NULL CHECK (state IN (…))` | Rule 4. The four states from `CONTEXT.md`. `stale` isn't one of them (removed in v0.2), and the test `skill_node_rejects_unknown_state` shows the `CHECK` refusing it. |
| `created_at TEXT NOT NULL` | Rule 2. |
| `retired_at TEXT` | Rule 2. `NULL` means "not retired". Retirement is a mark on the node, not a fifth state, and it's never cleared (I4). |
| `) STRICT;` | Rule 3. |

## Alternatives rejected

### `INTEGER PRIMARY KEY AUTOINCREMENT`

The simplest choice, and the one most tutorials use. With `AUTOINCREMENT`, SQLite never reuses a number within one file, which would satisfy I4. It lost because the numbers are only unique inside one file: node `42` in a backup, a curriculum export and a second install are three different nodes. Import and Tier 2 merging would need a translation table for every foreign key. Small integers are also easy to mistake for each other in logs and tests.

### UUIDv4 (random)

Globally unique like v7, but with no time order. Append-only tables would come back in a random order unless every query sorts by `created_at`, and inserts land at random places in the primary-key index instead of at its end. UUIDv7 costs nothing extra and fixes both.

### ULID

A ULID is the same idea as UUIDv7 (time first, then randomness) written in a shorter 26-character alphabet. It lost because UUIDv7 is an RFC standard, the `uuid` crate and most database tools already understand it, and a UUID is easier to recognise. The shorter form isn't worth a second format to learn.

### Unix timestamps as integers

`created_at INTEGER` holding seconds (or milliseconds) since 1970 is smaller and a little faster to compare. It lost on readability: `1790110420123` means nothing when you open the database, and it's easy to mix up seconds and milliseconds. Waypoint's tables are small enough that the size and speed don't matter.

### SQLite's default (non-`STRICT`) typing

The default is "type affinity": SQLite tries to convert a value to the column's type and stores it anyway if it can't. A bug that writes a number into a text column, or text into a number column, goes unnoticed until something reads it back. `STRICT` turns that into an immediate error, at no cost.

### No `CHECK` constraints ("the Rust layer already guards it")

The Rust layer *is* the guard, but a `CHECK` is one line and catches the bug the guard missed, at the moment it happens rather than weeks later when a bad row is read. It lost because it's the cheaper side of that trade. The opposite extreme, putting the domain rules in the database with triggers, lost too: the rules would then live in two places, and `AGENTS.md` keeps invariants in `waypoint-domain` only.

### SQL inside Rust strings, or loaded from a directory at runtime

Writing each migration as a Rust string literal keeps everything in one file, but long SQL inside Rust quotes is hard to read and gets no SQL highlighting in review. `rusqlite_migration`'s `from-directory` feature finds the `.sql` files by itself, but it adds a dependency (`include_dir`) and hides the order of the list. One file per migration plus an explicit list in `db.rs` is the most readable of the three.

## Consequences

- **Easier:** IDs and timestamps can be copied between databases, archives and import files without translation. Reading the raw database is possible without a decoder. A wrong-typed or unknown-enum write fails at once, with a clear error.
- **Harder:** TEXT IDs and timestamps take more space than integers (about 36 and 24 bytes a value), which is irrelevant at Waypoint's size. Every insert needs the Rust layer to supply an `id` and a `created_at`: the next build issue ("Add create_node and list_nodes to the domain crate") brings in UUID generation and the clock.
- **Constrains:** every new table uses the same `id`, timestamp, `STRICT` and `CHECK` pattern, and a table that breaks it needs a new ADR. `node_state_event.id` and `review_log.id` are UUIDv7 text too (architecture doc v0.4). A migration is never edited once merged.
