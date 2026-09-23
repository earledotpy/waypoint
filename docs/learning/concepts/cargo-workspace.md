# Cargo workspace

## In one line

A Cargo workspace is one repository holding several Rust crates that build together, share one `Cargo.lock` and one `target/` folder, and may import each other only when a crate's `Cargo.toml` says so.

## Where it appears here

- `Cargo.toml` (repo root): the workspace itself. It lists the members and the settings they share.
- `src-tauri/Cargo.toml`: the `waypoint-app` package, the Tauri shell.
- `crates/waypoint-domain/Cargo.toml`: the `waypoint-domain` package, the one writer to SQLite.
- `crates/waypoint-read/Cargo.toml`: the `waypoint-read` package, shared types and read-only queries.

## What it does

Three words matter:

- A **crate** is one unit the compiler builds: a library (`src/lib.rs`) or a program (`src/main.rs`). `waypoint-app` has both: `main.rs` only calls `run()` in `lib.rs`.
- A **package** is a folder with a `Cargo.toml`. It describes one or more crates: their name, version and dependencies. In everyday talk, and in this repo, "crate" usually means the package.
- A **workspace** is a root `Cargo.toml` with a `[workspace]` section. Waypoint's root is a *virtual* workspace: it has no `[package]` of its own, so there's no code at the root, only the list of members:

  ```toml
  [workspace]
  members = ["src-tauri", "crates/*"]
  ```

The root also holds what the members share:

- `[workspace.package]`: `edition`, `version`, `rust-version`. A member copies each with `edition.workspace = true`.
- `[workspace.dependencies]`: one version for each shared dependency. A member writes `tauri = { workspace = true }` and gets it.
- `[profile.release]`: compiler settings for release builds. Cargo reads profiles *only* from the workspace root and ignores them in members, which is why the template's profile was moved up from `src-tauri/`.

A member may use another member's code only if it lists it under `[dependencies]`. `waypoint-domain` lists `waypoint-read`, so domain code can write `use waypoint_read::…`. `waypoint-read` lists nothing, so `use waypoint_domain::…` inside it is a compile error. (The package name `waypoint-read` becomes `waypoint_read` in code, because `-` isn't allowed in a Rust name.)

Try it: `cargo metadata --no-deps --format-version 1` prints every member and what it depends on.

## Python comparison

The nearest Python idea is a monorepo with several packages, each in its own folder with its own `pyproject.toml`, installed together into one virtual environment (for example a `uv` workspace).

Where it breaks: Python doesn't *enforce* the dependency lists. Once everything is installed into the same environment, any package can `import` any other, whether or not its `pyproject.toml` lists it. The list is documentation plus an install instruction. In Cargo the list is the rule: the compiler only lets a crate see the crates it declared. An undeclared import fails the build, every time, on every machine.

A second difference: Cargo refuses dependency cycles. If `waypoint-read` declared `waypoint-domain` while the domain declares `waypoint-read`, Cargo rejects the workspace. In Python, circular imports are allowed and fail (or not) at runtime depending on import order.

## Why this code uses it

Waypoint needs rules like "only the domain writes to SQLite" and, from Tier 1, "the advisory crate can read but never write" to hold without anyone having to remember them. If everything were one crate, those rules would be conventions. With separate crates they're dependency lists, and the compiler checks them. ADR 0001 explains the choice of three crates now (`waypoint-app`, `waypoint-domain`, `waypoint-read`) and a fourth (`waypoint-advisory`) later, and why other ways of hiding writes were rejected.

The one shared `Cargo.lock` also means all crates agree on every dependency's version, and the one `target/` folder means a dependency such as Tauri compiles once for the whole workspace, not once per crate.

## See also

- [ADR 0001: crate layout, write hiding and frontend tooling](../../adr/0001-crate-layout-write-hiding-and-frontend-tooling.md)
- [Continuous integration](continuous-integration.md): `cargo clippy --workspace` and `cargo test --workspace` check every member at once.
- The Cargo Book, [Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
