# Continuous integration

## In one line

Continuous integration (CI) means a server runs the repo's checks on every proposed change, on a clean machine, and shows a pass or fail on the pull request before anyone merges.

## Where it appears here

- `.github/workflows/ci.yml`: the one CI workflow. GitHub Actions runs it on every pull request and every push to `main`.
- `rust-toolchain.toml` and `package.json` (`engines`): pin the Rust and Node versions, so CI and the laptop use the same tools.

## What it does

A GitHub Actions *workflow* is a YAML file in `.github/workflows/`. `on:` says when it runs. `jobs:` says what runs. Each job gets a fresh virtual machine (`runs-on: windows-latest`) and runs its `steps` in order. If a step fails, the job stops and the pull request shows a red ✗. In the PR's **Checks** tab, each step's `name:` appears as its own line, with that step's log.

Waypoint's job first sets things up: it checks out the code, installs Node 24, and restores the Rust build cache. Then it runs five checks:

| Step | Command | What it catches |
|---|---|---|
| Install frontend dependencies | `npm ci` | `package.json` and `package-lock.json` disagree (someone edited one without the other). It installs exactly the locked versions. |
| Build frontend | `npm run build` | TypeScript type errors (`tsc`) and a frontend that doesn't bundle. It also produces `dist/`, which the Rust build embeds, so it runs before the Cargo steps. |
| Check Rust formatting | `cargo fmt --all --check` | Rust code not formatted the standard way. Nothing is changed. It only reports. |
| Lint Rust | `cargo clippy --workspace --all-targets -- -D warnings` | Code that compiles but is suspicious or unidiomatic, plus every compiler warning. `-D warnings` makes warnings fail the build. |
| Test Rust | `cargo test --workspace` | Behaviour that's wrong: every `#[test]` in every crate, such as the database tests in `crates/waypoint-domain/src/db.rs`. |

## Python comparison

The same idea exists in Python projects. A typical Python CI runs `pip install -r requirements.txt` (or `uv sync --locked`), `ruff format --check`, `ruff check`, `mypy` and `pytest`. Mapped across: `npm ci` ≈ locked install, `cargo fmt --check` ≈ `ruff format --check` / `black --check`, `cargo clippy` ≈ `ruff check`, `tsc` ≈ `mypy`, `cargo test` ≈ `pytest`.

Where it differs: in Python, CI is often the *first* place type errors show up, because running the code doesn't check types. In Rust the compiler checks types on every build, so a lot of what `mypy` catches in CI already stops `cargo build` on the laptop. Clippy and tests still find what the compiler lets through.

## Why this code uses it

`AGENTS.md` workflow step 4 says: before opening a PR, run every check the repo defines and make sure they pass locally. `ci.yml` *is* that list of checks, written down once. The agent runs the same five commands locally before pushing, and CI runs them again on a clean Windows machine. That catches what "works on my machine" hides: a file never committed, a lock file out of date, a tool version that differs.

For the author, a green check on a PR means the change builds and passes from scratch. It doesn't replace reading the diff, but it means reviewing can focus on design, not on whether the code compiles.

The Rust cache (`Swatinem/rust-cache`) is there because a cold build of Tauri takes about 10 minutes. With the cache, later runs only recompile Waypoint's own crates.

## See also

- [Cargo workspace](cargo-workspace.md): why `--workspace` is on the clippy and test commands.
- [ADR 0001 §6, Pinning](../../adr/0001-crate-layout-write-hiding-and-frontend-tooling.md#6-pinning)
- GitHub Docs, [Understanding GitHub Actions](https://docs.github.com/en/actions/get-started/understand-github-actions)
