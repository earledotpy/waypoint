# Tauri managed state

## In one line

`app.manage(value)` hands a value to Tauri once, at startup, and Tauri then gives it to every command that asks for that value's exact type.

## Where it appears here

- `src-tauri/src/lib.rs`, `run`: inside `setup`, finds the app data folder, calls `open_database`, and passes the result to `app.manage`.
- `src-tauri/src/lib.rs`, `open_database`: makes the folder, calls `waypoint_domain::open`, and wraps the connection in a `Mutex`.
- `src-tauri/src/lib.rs`, `Db`: the name for `Mutex<Connection>`, the one type that is managed.
- The tests in `lib.rs`: `open_database_creates_missing_folder_and_file` and `open_database_twice_keeps_existing_data`. The second one also shows `.lock()` in use.

## What it does

A Tauri command is an ordinary Rust function that the UI calls. It can't have the database connection passed in by the UI, because the UI only sends JSON. So Tauri keeps a store of values, one per type, and fills in a command's arguments from it.

`app.manage(db)` puts `db` in that store. It stays there until the app quits. The next issue's commands will ask for it by adding an argument like `db: tauri::State<'_, Db>`, and Tauri looks up the value whose type is `Db` and passes it in.

The lookup is by type, and it happens when the command is called, not when the code compiles. If a command asks for a type that was never managed (say, a bare `Connection`), it still compiles, and fails at runtime with a "state not managed" error. That's why `lib.rs` names the type once, as `Db`, and every command uses that name.

### Why a `Mutex`

`manage` only accepts values that are safe to share between threads: its signature requires `T: Send + Sync + 'static`. Tauri insists on that because commands don't all run on one thread (an `async` command, for example, runs on a pool of background threads), and any of them may reach for the same state. Rust describes "safe to share" with two marker traits:

- **`Send`**: the value can be *moved* to another thread.
- **`Sync`**: the value can be *used* from several threads at the same time.

A rusqlite `Connection` is `Send` but not `Sync`. It can move to another thread, but it isn't built to be used from two threads at the same time: it keeps internal bookkeeping (such as a cache of prepared statements) that two threads could corrupt by changing it at once. So the compiler rejects `app.manage(conn)` outright.

`Mutex<Connection>` is both `Send` and `Sync`. A `Mutex` (mutual exclusion) lets one thread in at a time. To reach the connection, code calls `.lock()`, which waits until no one else holds it and returns a guard. The guard behaves like the connection, and the lock is released when the guard is dropped, at the end of its block or by an explicit `drop(guard)`, as the tests do.

`.lock()` returns a `Result` (see [`Result` and `?`](result-and-question-mark.md)). It's an `Err` only if another thread panicked while holding the lock, which Rust calls a *poisoned* lock. The tests `.unwrap()` it.

### Why `std::sync::Mutex`, not Tokio's

Tokio, the async runtime Tauri uses, has its own `tokio::sync::Mutex`, whose `lock()` must be `.await`ed. Its one advantage is that it can be held across an `.await`, while the code waits for something else. Waypoint never needs that: rusqlite's calls are blocking, not `async`, so a command locks, runs its SQL, and lets go without ever awaiting while it holds the lock. For that case [Tokio's own docs](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html#which-kind-of-mutex-should-you-use) recommend the standard library's `Mutex`, which is simpler and faster.

### The other Rust in `run`

Four pieces of syntax appear in `lib.rs` for the first time. None needs a note of its own yet:

- **`|app| { … }`** is a *closure*: a function with no name, written where it's used. It's like a Python `lambda`, except that it can hold several statements. `setup` takes one and calls it with the app.
- **`Box<dyn std::error::Error>`** means "an error of any type". `dyn std::error::Error` is the same "some type that implements `Error`" as in `DomainError`'s `source` (see [Traits and `impl` blocks](traits-and-impl.md)), and the `Box` puts it on the heap, because different error types are different sizes. `?` converts any error into it, so `open_database` can fail with an `io::Error` or a `DomainError`. It's closest to annotating a Python function's failure as plain `Exception`.
- **`pub type Db = Mutex<Connection>;`** is a *type alias*: a second name for an existing type, not a new type. Python has the same thing in type hints: `Rows = list[tuple[str, int]]`.
- **`use tauri::Manager;`** brings a trait into scope. A trait's methods can only be called where the trait is imported, so without this line `app.path()` and `app.manage(…)` don't exist.

## Python comparison

The nearest Python idea is a module-level connection guarded by a `threading.Lock`:

```python
import sqlite3, threading

_conn = sqlite3.connect(path, check_same_thread=False)
_lock = threading.Lock()

def list_nodes():
    with _lock:
        return _conn.execute("SELECT ...").fetchall()
```

`app.manage` plays the role of the module-level `_conn`, and `with _lock:` is `.lock()` plus the guard being dropped.

Where the analogy breaks:

- **The lock owns the data.** In Python, `_lock` and `_conn` are two separate names, and nothing stops a function from using `_conn` without taking `_lock`. In Rust, the connection is *inside* the `Mutex`. There's no way to reach it except through `.lock()`.
- **Thread safety is checked when compiling.** Python finds out that a connection was shared unsafely by crashing or corrupting data at runtime (`check_same_thread=False` switches off its one guard). Rust refuses to compile `app.manage(conn)` without the `Mutex`, because `Connection` isn't `Sync`.
- **Found by type, not by name.** A Python function imports `_conn` by its name. A Tauri command gets its state by declaring its type, which is why a type mismatch shows up only at runtime.

## Why this code uses it

Every future command needs the same connection, and architecture doc §6 says migrations must finish "before any command is accepted". `setup` runs once at startup, and Tauri doesn't handle any command until it has returned, so opening the database there and managing the result makes both true. If `open_database` fails, `?` returns the error from `setup`, and Tauri stops the app with a panic message beginning `Failed to setup app:`, rather than let it run on a database it couldn't open or bring up to date.

Holding one connection for the app's whole life, rather than opening one per command, means the settings and migrations in `waypoint_domain::open` run once per launch.

## See also

- [SQLite migrations](sqlite-migrations.md): what `waypoint_domain::open` does before it returns.
- [Traits and `impl` blocks](traits-and-impl.md): `Send`, `Sync` and `tauri::Manager` are traits.
- Tauri documentation, [State Management](https://v2.tauri.app/develop/state-management/)
