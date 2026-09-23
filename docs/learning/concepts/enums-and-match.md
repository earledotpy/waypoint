# Enums and `match`

## In one line

A Rust `enum` is a type whose value is exactly one of a fixed list of variants, each of which can carry its own data, and `match` handles a value by listing what to do for every variant.

## Where it appears here

- `crates/waypoint-domain/src/error.rs`, `DomainError`: an enum with two variants, `Database(rusqlite::Error)` and `Migration(rusqlite_migration::Error)`.
- The same file, `Display::fmt` and `Error::source`: each is one `match self { … }` over those two variants.
- `Result` itself, used everywhere in `crates/waypoint-domain/src/db.rs`, is an enum from the standard library: `Ok(value)` or `Err(error)`.

## What it does

```rust
pub enum DomainError {
    Database(rusqlite::Error),
    Migration(rusqlite_migration::Error),
}
```

A `DomainError` is *either* a `Database` holding a `rusqlite::Error`, *or* a `Migration` holding a `rusqlite_migration::Error`. It's never both, never neither, and never a third thing. You make one by naming the variant: `DomainError::Database(e)`.

To use one, you `match` on it:

```rust
match self {
    DomainError::Database(e) => write!(f, "database error: {e}"),
    DomainError::Migration(e) => write!(f, "migration error: {e}"),
}
```

Each line is an *arm*: a pattern, then `=>`, then what to do. The pattern `DomainError::Database(e)` checks the variant *and* unpacks the data inside it into `e`, for that arm only. The whole `match` is an expression. Whichever arm runs, its value is the result, which is why `fmt` doesn't need `return`.

The important rule: **a `match` must cover every variant.** If a later PR adds a third variant, say `DomainError::Invariant(…)`, every `match` on `DomainError` that doesn't handle it stops compiling, and the compiler lists each one. Nothing can silently fall through.

## Python comparison

Python's `enum.Enum` is only half of this. Its members are fixed constants (`Color.RED`) and can't each carry a different value at runtime. The closer match is a union of small dataclasses handled with Python 3.10's `match`:

```python
@dataclass
class Database: error: sqlite3.Error
@dataclass
class Migration: error: Exception

DomainError = Database | Migration

match err:
    case Database(error=e): msg = f"database error: {e}"
    case Migration(error=e): msg = f"migration error: {e}"
```

Where it breaks: **exhaustiveness.** If Python's `match` meets a case it doesn't list, it quietly does nothing, and `msg` is never set. `mypy` can warn about the gap, but only if you set it up to. Rust makes it a compile error on every build. Also, in Python anything can be passed where a `DomainError` is expected, while in Rust the value can only ever be one of the listed variants.

## Why this code uses it

The domain can fail in a small, known number of ways, and the app should handle each one on purpose. An enum states that list in one place. `match` then makes every consumer of the error handle all of it, now and whenever the list grows. The state machine will lean on the same guarantee once node states are a Rust type.

## See also

- [Traits and `impl` blocks](traits-and-impl.md): the `impl` blocks these `match`es live in.
- [`Result` and `?`](result-and-question-mark.md): `Result` is an enum too, and `?` is a short `match` on it.
- The Rust Book, [Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
