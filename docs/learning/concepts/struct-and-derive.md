# `struct`, `Option` and `#[derive(…)]`

## In one line

A `struct` is a named bundle of typed fields, `Option<T>` is a field that may hold nothing (`None`) or a `T` (`Some(value)`), and `#[derive(…)]` asks the compiler to write standard code for the struct from its fields.

## Where it appears here

- `crates/waypoint-read/src/node.rs`, `SkillNode`: one skill node row, with `#[derive(Debug, Clone, PartialEq, Serialize)]`.
- The same file, `SkillNode::from_row`: builds a `SkillNode` from one SQLite row. `list_nodes` and the domain's `create_node` both pass it to rusqlite, which calls it once per row.
- `crates/waypoint-domain/src/node.rs`, the tests: `assert_eq!(node.external_id, None)`, and `assert_eq!(list_nodes(&conn).unwrap(), vec![first, second, third])`, which compares whole nodes.

## What it does

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillNode {
    pub id: String,
    pub external_id: Option<String>,
    ...
}
```

- **`struct`** declares the type and its fields. Every field has a type, and a value can't be built without every field: `SkillNode { id: …, title: … }` with a field missing doesn't compile. `pub` makes a field readable outside the crate.
- **`Option<String>`** is an enum with two variants, `None` and `Some(String)` (see [enums and `match`](enums-and-match.md)). It maps directly onto a nullable column: a `NULL` `external_id` reads back as `None`. A field that's plain `String` can never be missing, so code that reads `node.title` never has to check.
- **`#[derive(…)]`** makes the compiler write an `impl` block for each listed trait (see [traits and `impl`](traits-and-impl.md)):
  - `Debug`: the developer-facing print, `{:?}`. A failing `assert_eq!` uses it to show both sides.
  - `Clone`: `.clone()`, a full copy of the node.
  - `PartialEq`: `==` compares the two nodes field by field. That's what lets a test compare a whole `Vec<SkillNode>` in one line.
  - `Serialize` (from the `serde` crate): code that turns a `SkillNode` into JSON, `{"id": "…", "external_id": null, …}`, using the field names as keys. The Tauri app will use it to send nodes to the frontend.

`impl SkillNode { pub fn from_row(…) }` adds a function *to the type* without a trait. `SkillNode::from_row` is called on the type, like a `@classmethod` or `@staticmethod` in Python, not on one node.

## Python comparison

The nearest Python idea is a `@dataclass` with `Optional[...]` fields:

```python
from dataclasses import dataclass
from typing import Optional

@dataclass
class SkillNode:
    id: str
    external_id: Optional[str]
    title: str
    ...
```

`@dataclass` writes `__init__`, `__repr__` (≈ `Debug`) and `__eq__` (≈ `PartialEq`) for you, much as `derive` does.

Where it breaks:

- **Compile time, not runtime.** `@dataclass` is a function that runs when Python imports the module and builds the methods then. `derive` runs inside the compiler: the generated code is ordinary Rust, type-checked like the code you write, and there's nothing left to do while the app runs. If a field's type can't be serialized, the build fails; Python's `json.dumps` would fail only when that value is actually dumped.
- **The types are enforced.** In Python, `Optional[str]` is a hint, and nothing stops `title=None`. In Rust, `title: String` can't hold "nothing" at all, and to read a value out of an `Option` you have to handle the `None` case (with `match`, `if let`, or `.unwrap()`), so a forgotten `None` check doesn't compile.
- **Nothing is derived unless you ask.** A Python class always has `==` (identity, by default) and a `repr`. A Rust struct without `derive(PartialEq)` can't be compared with `==` at all.

## Why this code uses it

`SkillNode` is the one shape a node has everywhere: the domain returns it, `list_nodes` returns it, and later the app sends it to the frontend. A struct makes that shape a type the compiler checks. `Option` makes the two nullable columns (`external_id`, `retired_at`) visibly nullable, and no others. The four derives are the standard code a data type needs, and writing them by hand would be many lines of boilerplate with nothing to decide, which is why `derive` isn't the kind of "clever" code `AGENTS.md` asks an ADR for.

The field names are the SQL column names, left as snake_case (no serde `rename_all`), so one word finds a field in SQL, Rust and, later, TypeScript.

## See also

- [Traits and `impl` blocks](traits-and-impl.md): what a derive writes.
- [Enums and `match`](enums-and-match.md): `Option` is an enum.
- serde's guide, [Using derive](https://serde.rs/derive.html)
