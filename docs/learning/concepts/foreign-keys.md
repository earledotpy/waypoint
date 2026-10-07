# Foreign keys

## In plain words

Some columns hold a pointer to a row in another table, like a cross-reference in a book: "see page 42". A foreign key is a rule that the page must exist. Waypoint's edges point at two nodes, and the database refuses to store an edge that points at a node that isn't there.

## In one line

A foreign key (`REFERENCES other_table (column)`) is a [constraint](../glossary.md#constraint) saying a column may only hold a value that exists in that column of another table, and SQLite checks it on every write once `PRAGMA foreign_keys` is on.

## Where it appears here

- `crates/waypoint-domain/migrations/0002_create_skill_edge.sql`: `from_node_id` and `to_node_id` both say `REFERENCES skill_node (id)`.
- `crates/waypoint-domain/src/db.rs`, `prepare`: `conn.pragma_update(None, "foreign_keys", "ON")` turns the checks on for every [connection](../glossary.md#connection).
- `crates/waypoint-domain/src/db.rs`, tests `skill_edge_rejects_an_unknown_node` (the check refuses a bad id) and `open_turns_on_foreign_keys_and_wal` (the setting is on).

## What it does

Every node has an `id`, its [primary key](../glossary.md#primary-key): the one value that picks out that row and no other. An edge doesn't copy the nodes it joins. It stores their ids:

```sql
from_node_id TEXT NOT NULL REFERENCES skill_node (id),
```

`REFERENCES skill_node (id)` reads as "whatever goes in `from_node_id` must already be the `id` of some row in `skill_node`". When anything inserts or changes an edge, SQLite looks the id up. If no node has it, the write fails with `FOREIGN KEY constraint failed` and nothing is saved.

The check also runs the other way. Deleting a node that an edge still points at would leave the edge pointing at nothing, so SQLite refuses that too. A foreign key can say what to do instead, with `ON DELETE CASCADE` (delete the edges too) or `ON DELETE SET NULL` (blank the pointer). Waypoint's don't, because nodes are never deleted: retiring a node is a mark (`retired_at`), not a deletion.

There's one catch. For historical reasons, SQLite ignores every `REFERENCES` unless the connection has run `PRAGMA foreign_keys = ON`, and the setting lasts only as long as that connection. Forget it, and the table still accepts edges to nodes that don't exist, with no error. That's why `prepare` turns it on for every connection, and why `open` is the only way Waypoint opens one (see [SQLite migrations](sqlite-migrations.md), "Why this code uses it").

## Python comparison

Picture the nodes as a dictionary from id to node:

```python
nodes = {"0199…01": for_loop, "0199…02": function}
```

A foreign key is like insisting that every edge passes `nodes[edge.from_node_id]` before it's stored: if the id isn't a key in the dictionary, that line raises `KeyError` and the edge is never saved.

Where the comparison breaks:

- **Who checks.** In Python, that lookup happens only where someone wrote it. A foreign key is checked by the database on every write, whoever writes: the Rust code, a test, or someone typing SQL by hand.
- **It can be switched off.** A dictionary lookup always raises. A SQLite foreign key only checks while `PRAGMA foreign_keys` is on for that connection.
- **It guards deletes too.** Deleting a key from a Python dictionary never checks what else points at it. SQLite refuses to delete a node while an edge points at it.

## Why this code uses it

The Rust domain layer will check that both nodes exist before it adds an edge (milestone 2, step 6), and that check gives the clear message. The foreign key is a backstop, like the `CHECK`s in migration 0001: if a bug ever slips past the Rust check, the database still refuses an edge that points at nothing. An edge like that would quietly break prerequisite arithmetic and the I5 cycle check, which both follow edges from node to node.

## See also

- [SQLite migrations](sqlite-migrations.md): how `0002_create_skill_edge.sql` gets run, and its section on rules across several columns.
- [ADR 0002: table conventions](../../adr/0002-table-conventions.md): why ids are UUIDv7 text.
- SQLite documentation, [Foreign Key Support](https://www.sqlite.org/foreignkeys.html), especially §2 "Enabling Foreign Key Support".
