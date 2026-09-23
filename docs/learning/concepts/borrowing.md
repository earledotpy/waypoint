# Ownership and borrowing (`&`, `&mut`)

## In one line

Every value in Rust has exactly one owner. Other code can *borrow* the value, with `&` to read it or `&mut` to change it, without taking it away from its owner.

## Where it appears here

- `crates/waypoint-domain/src/db.rs`:
  - `open(path: &Path)` borrows the path to read it.
  - `prepare(mut conn: Connection)` takes ownership of the connection, then gives it back in `Ok(conn)`.
  - `migrations().to_latest(&mut conn)` lends the connection out, able to change it.
- The tests in `db.rs`: `drop(first)` ends ownership, which closes a connection.
- `crates/waypoint-domain/src/error.rs`: `fn fmt(&self, …)` and `fn source(&self)` borrow the error to read it.

## What it does

There are three ways to pass a value, and the function signature says which one it uses:

| Written | Name | The function may… | Afterwards the caller… |
|---|---|---|---|
| `x: T` | move (take ownership) | do anything, including destroy it | can't use `x` any more |
| `x: &T` | shared borrow | read it | still owns it, unchanged |
| `x: &mut T` | mutable borrow | read and change it | still owns it, maybe changed |

In `db.rs`:

- **`path: &Path`**: `open` only needs to read the path, so it borrows it. The caller keeps its path and can use it again. That's why `open_is_idempotent` can call `open(&path)` twice.
- **`mut conn: Connection`**: `open` hands its connection to `prepare` and never touches it again. `prepare` becomes the owner (`mut` lets it change what it owns), sets it up, and moves it back out with `Ok(conn)`. If `open` tried to use `conn` after `prepare(conn)`, it wouldn't compile: "use of moved value".
- **`&mut conn`**: `to_latest` needs to start a transaction on the connection, which changes it. So `prepare` lends it out mutably for the length of that call, and gets it back when the call returns.
- **`drop(first)`**: when an owner ends, its value is cleaned up at once. For a `Connection` that means closing the database file. The test drops the first connection so the second `open` is a genuine reopen.

Two rules, checked by the compiler, make this safe:

1. **Many readers or one writer, never both.** At any moment a value can have any number of `&` borrows *or* one `&mut` borrow.
2. **A borrow can't outlive its owner.** You can't keep a `&` to something after it's been dropped.

(`'static` in `Migrations<'static>` and `'_` in `Formatter<'_>` are *lifetimes*: labels for how long a borrow is valid. `'static` means "for the whole program", which is true of the SQL that `include_str!` builds into the app.)

## Python comparison

There's no honest Python equivalent. In Python every variable is a reference to a shared object, anyone holding a reference can change it, and the garbage collector frees it once nobody refers to it. Nothing says who owns what, and nothing stops two parts of the code changing one object at once.

What will feel strange:

- **Passing a value can use it up.** After `prepare(conn)`, `conn` is gone from `open`, even though Python would let you keep using it.
- **The compiler refuses code that would run fine in Python.** For example, it won't let you hold a `&` to something while also changing it through a `&mut`. The error messages ("cannot borrow as mutable because it is also borrowed as immutable") are the compiler enforcing rule 1.
- **Cleanup happens at a known point.** A value is dropped when its owner ends, not "some time later" as with Python's garbage collector. This is why a Rust connection closes exactly when it goes out of scope, without a `with` block.

## Why this code uses it

Rust has no garbage collector, so ownership is how it knows when to free memory and close files. It's not optional. What the code chooses is *which* of the three to use. Each function asks for the least it needs: `open` only reads the path, so it borrows it, and `to_latest` has to change the connection, so it borrows it mutably. `prepare` takes the connection outright because its whole job is to finish it and hand it back, so the caller can't accidentally use a half-prepared one.

## See also

- [Traits and `impl` blocks](traits-and-impl.md): `&self` in a method is a shared borrow of the value the method is called on.
- [`Result` and `?`](result-and-question-mark.md): `Ok(conn)` is where `prepare` hands ownership back.
- The Rust Book, [Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
