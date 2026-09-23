# Borrowing in signatures (`&Connection`, `&str`)

## In one line

A `&` in a parameter's type means the function *borrows* that argument for the length of the call and hands it straight back, so the caller keeps it; `&str` is the borrowed form of text, and `String` is the owned form.

This note builds on [ownership and borrowing](borrowing.md), which explains `&`, `&mut` and moves in general. This one covers the two borrowed types in `create_node`'s signature.

## Where it appears here

- `crates/waypoint-domain/src/node.rs`, `create_node(conn: &Connection, title: &str, description: &str) -> Result<SkillNode, DomainError>`.
- `crates/waypoint-read/src/node.rs`, `list_nodes(conn: &Connection)`.
- The tests in `crates/waypoint-domain/src/node.rs`: `create_node(&conn, "Count to ten", "")`, called several times with the same `conn`.

## What it does

**`conn: &Connection`**. The test owns the connection (`let conn = open_in_memory()…`). `create_node(&conn, …)` lends it for one call. When the call returns, the test still has `conn` and uses it again for the next `create_node` and for `list_nodes`. If the signature were `conn: Connection` (no `&`), the first call would take the connection away and the second call wouldn't compile. A shared `&` borrow is enough for an `INSERT`, even though the database changes: rusqlite's `execute` and `query_row` are written to take `&Connection`. The connection does update some private bookkeeping of its own as a statement runs (a cache of prepared statements), but rusqlite handles that carefully inside, so callers only need to lend it for reading. Rust calls this *interior mutability*. Starting a transaction does need `&mut` (as in `db.rs`'s `prepare`), so the compiler can stop a second transaction starting on the same connection while one is open.

**`title: &str`**. Rust has two text types:

| Type | Owns its text? | Typical source |
|---|---|---|
| `String` | yes, and can grow | built at runtime, e.g. `Uuid::now_v7().to_string()`, or read from the database |
| `&str` | no, it's a view into text someone else owns | a literal like `"Count to ten"`, or a borrow of a `String` |

Taking `&str` means a caller can pass either a literal or a borrowed `String`, and nothing is copied. Inside, `title.trim()` doesn't make new text: it returns a smaller `&str` that points into the same characters, just without the spaces at each end. For `"   "` that view is empty, so `title.is_empty()` is true and `create_node` returns `EmptyTitle` before it reaches SQLite. The text is copied only when rusqlite hands it to SQLite for the `INSERT`. The `SkillNode` that comes back owns its own `String`s, read from the stored row.

## Python comparison

There's no honest Python equivalent. Every Python argument is a reference to a shared object, so "does the function keep it?" never comes up, and Python has one string type, `str`, which is immutable and shared freely.

What will feel strange:

- **Two string types.** `String` and `&str` both look like text, and a function signature picks one. Passing a `String` where `&str` is expected needs a `&` (`&my_string`). Going the other way needs a copy (`.to_string()`). Python never asks.
- **`trim()` doesn't copy.** Python's `" a ".strip()` builds a new string. Rust's `trim()` returns a view into the old one, which can't outlive it (rule 2 in [borrowing.md](borrowing.md)).
- **A write through a shared `&`.** In `borrowing.md`, "change it" meant `&mut`. `create_node` changes the *database* through a plain `&Connection`. The `&` is about the Rust value `conn`, which the call doesn't change. What happens inside SQLite is SQLite's business.

## Why this code uses it

Each parameter asks for the least it needs. `create_node` reads the title and description and never keeps them, so it borrows them as `&str`. It uses the connection and gives it back, so it borrows that too. This lets one test (and, later, one Tauri command) make many calls on the same connection, and callers pass literals or `String`s alike. It's also the standard Rust habit for text parameters: take `&str` unless the function needs to own the text.

## See also

- [Ownership and borrowing](borrowing.md): the general rules this note relies on.
- [`struct`, `Option` and `#[derive(…)]`](struct-and-derive.md): the `SkillNode` that comes back.
- The Rust Book, [The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html) (string slices, `&str`)
