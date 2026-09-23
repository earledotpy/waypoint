# Rust unit tests

## In one line

A Rust unit test is a function marked `#[test]`, kept in a `mod tests` block at the bottom of the file it tests, and `cargo test` finds and runs every one.

## Where it appears here

- `crates/waypoint-domain/src/db.rs`, `mod tests`: the database tests (`migrations_are_valid`, `open_in_memory_creates_skill_node_table`, `open_is_idempotent`, `open_turns_on_foreign_keys_and_wal`, `skill_node_rejects_unknown_state`).
- `crates/waypoint-domain/Cargo.toml`, `[dev-dependencies]`: `tempfile`, a crate only the tests use.

## What it does

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_is_idempotent() { … }
}
```

- `#[test]` marks a function as a test. It takes no arguments and passes if it returns without panicking.
- `assert_eq!(a, b)` and `assert!(condition)` panic with a message when they fail, which fails the test. `.unwrap()` on an `Err` panics too, so a test stops at the first unexpected error.
- `#[cfg(test)]` means "compile this module only when testing". The tests and their helpers (`user_version`, `table_exists`) are left out of the app entirely.
- `use super::*;` imports everything from the file above, *including private items*. That's how `migrations_are_valid` can call the private `migrations()` function.
- `[dev-dependencies]` are crates that only tests and examples can use. `tempfile` makes a temporary folder that's deleted when the test ends, so `open_is_idempotent` gets a real file without leaving it behind.

Run them with `cargo test --workspace`, or one crate with `cargo test -p waypoint-domain`, or only tests whose name contains a word with `cargo test -p waypoint-domain idempotent`.

## Python comparison

The nearest Python idea is pytest: `#[test] fn open_is_idempotent()` ≈ `def test_open_is_idempotent():`, `assert_eq!(a, b)` ≈ `assert a == b`, and `tempfile::tempdir()` ≈ pytest's `tmp_path` fixture.

Where it differs: pytest tests usually live in a separate `tests/` folder and can only reach what the module exports (unless you import `_private` names by convention-breaking). Rust unit tests live *inside* the file they test and can see its private functions, because they're a child module. Rust also has pytest-style separate tests in a crate's `tests/` folder, called integration tests, which see only the public API. Waypoint has none yet.

## Why this code uses it

`AGENTS.md` asks for every invariant to have a test named after it, so "where is I6 enforced?" is a search for `i6_`. Keeping the tests next to the code means that search lands in the file that does the enforcing. For `db.rs`, the tests also check the one thing that's hard to see by reading: that opening a database twice doesn't run its migrations twice.

## See also

- [Continuous integration](continuous-integration.md): CI runs `cargo test --workspace` on every pull request.
- [SQLite migrations](sqlite-migrations.md): what the `db.rs` tests are checking.
- The Rust Book, [How to Write Tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
