# Research: Tauri v2 scaffold defaults and current versions

Ticket: [Tauri v2 scaffold defaults and current versions](https://github.com/earledotpy/waypoint/issues/3), part of wayfinder map #1.
**All versions checked 2026-09-22** against crates.io, the npm registry, nodejs.org, static.rust-lang.org, GitHub, and the official docs. Nothing was installed or scaffolded. This is a decision aid; the tooling decision itself belongs to ADR 0001.

## TL;DR

- Start from **Tauri 2.11.6** (latest stable) with `create-tauri-app` 4.7.4's `react-ts` template: Vite 8, React 19, TypeScript. **Tauri 3.0 is in alpha** (3.0.0-alpha.0/.1/.2, Sept 13 to 21, 2026). Don't build on it yet.
- Rust side: `rusqlite` 0.40.2 (`bundled`), `rusqlite_migration` 2.6.0 and `fsrs` 6.6.2 all work together. No version conflicts. The Rust floor is **1.95**, set by `rusqlite_migration`; the current stable is 1.98.1.
- Frontend add-ons: Tailwind **v4** (4.3.3) through `@tailwindcss/vite`, with no config file. Vitest **5.0.1**. Plain `cargo test`, plus Tauri's `test` feature (mock runtime) for command tests.
- Node: use **24.21.0 LTS (Krypton)**. The binding constraint is `>=24.15.0`, which comes from jsdom 30 and npm 12, not from Vite.
- Package manager: recommend **npm** for learner readability (reasoning below).
- DB in state: `app.manage(Mutex::new(conn))` in `setup`, with sync commands taking `State<'_, Mutex<Connection>>`. This is Tauri's documented pattern.
- **Correction for design-document §9:** `fsrs` is **BSD-3-Clause**, not MIT.

## 1. What `create-tauri-app` generates (React + TS)

Source: `tauri-apps/create-tauri-app` at tag `create-tauri-app-v4.7.4` (released 2026-09-04; npm `create-tauri-app@4.7.4`). Commands are from <https://v2.tauri.app/start/create-project/>: `npm create tauri-app@latest`, `pnpm create tauri-app`, `yarn create tauri-app`, `bun create tauri-app`, `deno run -A npm:create-tauri-app`, `cargo create-tauri-app`, or `irm https://create.tauri.app/ps | iex`.

**Package manager options** (`src/package_manager.rs`): `cargo`, `pnpm`, `yarn`, `npm`, `deno`, `bun`, `dotnet`. The Node-based ones are pnpm, yarn, npm, deno and bun. The template's `.manifest` writes the chosen manager into `tauri.conf.json`, as `beforeDevCommand = <pm> run dev` and `beforeBuildCommand = <pm> run build`.

**File layout** (from `templates/template-react-ts` + `templates/_base_`):

```
index.html
package.json
tsconfig.json, tsconfig.node.json
vite.config.ts
public/            vite.svg, tauri.svg
src/               main.tsx, App.tsx, App.css, vite-env.d.ts, assets/react.svg
src-tauri/
  Cargo.toml
  build.rs
  tauri.conf.json
  capabilities/default.json
  icons/…
  src/main.rs      (thin: calls <lib>::run())
  src/lib.rs       (Builder, plugin-opener, a `greet` command)
```

**npm scripts** (from `package.json.lte`): `dev: vite`, `build: tsc && vite build`, `preview: vite preview`, `tauri: tauri`. You run the app with `npm run tauri dev`.

**Other defaults worth knowing:**
- `vite.config.ts` pins the dev server to port **1420** with `strictPort: true`, uses HMR port 1421 when `TAURI_DEV_HOST` is set, sets `clearScreen: false`, and ignores `src-tauri/**` in the watcher.
- `tauri.conf.json`: `devUrl: http://localhost:1420`, `frontendDist: ../dist`, one 800×600 window, `csp: null`.
- `Cargo.toml`: `edition = "2021"`, `tauri = "2"`, `tauri-plugin-opener = "2"`, `serde`, `serde_json`, and a size-optimised `[profile.release]` (`lto`, `codegen-units = 1`, `panic = "abort"`, `strip`). The `[lib]` has `crate-type = ["staticlib", "cdylib", "rlib"]`. For a learner, this lib/main split is *why* plain `cargo test` works on the app's Rust code: the logic lives in a library crate, not only in a binary.

### Template pins vs. current latest

| Package | Template pin | npm `latest` (2026-09-22) | Resolves to latest? |
|---|---|---|---|
| `react`, `react-dom` | `^19.1.0` | 19.3.0 | yes |
| `@types/react` / `@types/react-dom` | `^19.1.8` / `^19.1.6` | n/a | yes (caret) |
| `vite` | `^8.0.16` | 8.3.0 | yes |
| `@vitejs/plugin-react` | `^6.0.2` | 6.1.1 | yes |
| `typescript` | `~6.0.3` | **7.0.2** | **no.** Tilde stays on 6.0.x. |
| `@tauri-apps/api` | `^2` | 2.11.1 | yes |
| `@tauri-apps/cli` | `^2` | 2.11.5 | yes |
| `@tauri-apps/plugin-opener` | `^2` | 2.5.5 | yes |

The TypeScript pin is deliberate on the template's side. Keep `~6.0.3` until something needs TS 7; moving to 7 is a separate decision.

## 2. Add-ons: Tailwind, Vitest, Rust tests

**Tailwind CSS v4** (`tailwindcss` 4.3.3, `@tailwindcss/vite` 4.3.3, peer `vite ^5.2 || ^6 || ^7 || ^8`). From <https://tailwindcss.com/docs/installation/using-vite>:
1. `npm install tailwindcss @tailwindcss/vite`
2. Add `tailwindcss()` to `plugins` in `vite.config.ts`.
3. Put `@import "tailwindcss";` in the CSS file (the template's `src/App.css`, or a new `index.css`).

v4 needs no `tailwind.config.js` and no `postcss.config.js`. Tutorials that show those files are v3-era.

**Vitest 5.0.1** (<https://vitest.dev/guide/>): requires "Vite >=v6.4.0 and Node >=v22.12.0", installs with `npm install -D vitest`, and reads the existing `vite.config.*`. For component tests add `jsdom` (30.1.1) and `@testing-library/react` (16.3.3, peer `react ^18 || ^19`).

**Tauri's testing guidance** (<https://v2.tauri.app/develop/tests/>):
- **Rust:** a mock runtime through the `tauri` crate's `test` feature (it exists in 2.11.6). "Under the mock runtime, native webview libraries are not executed." Use this to test commands. Pure domain code needs only plain `cargo test`.
- **Frontend:** `mockIPC` / `clearMocks` from `@tauri-apps/api/mocks`. The docs' examples use Vitest, with a jsdom `window.crypto` polyfill because "jsdom doesn't come with a WebCrypto implementation" (<https://v2.tauri.app/develop/tests/mocking/>).
- **End-to-end:** WebDriver (WebdriverIO or Selenium), supported on Windows and Linux. Probably not needed for v1.

## 3. Rust crates: versions and compatibility

| Crate | Latest stable | Published | MSRV (`rust_version`) | Notes |
|---|---|---|---|---|
| `tauri` | **2.11.6** | 2026-09-19 | 1.77.2 | 3.0.0-alpha.2 was published 2026-09-21. crates.io `max_stable_version` is 2.11.6. |
| `tauri-build` | 2.6.3 | 2026-06-17 | 1.77.2 | |
| `tauri-cli` / `@tauri-apps/cli` | 2.11.5 | 2026-09-19 | 1.77.2 | |
| `rusqlite` | **0.40.2** | 2026-08-08 | not declared | `bundled` = `libsqlite3-sys?/bundled` + `modern_sqlite`. Depends on `libsqlite3-sys ^0.38.2`. |
| `rusqlite_migration` | **2.6.0** | 2026-05-28 | **1.95** | Requires `rusqlite ^0.40.0`, so 0.40.2 satisfies it. |
| `fsrs` (repo `fsrs-rs`) | **6.6.2** | 2026-08-29 | not declared | License **BSD-3-Clause** (crates.io and GitHub agree). No SQLite or Tauri dependencies. |
| `tauri-plugin-sql` (not chosen) | 2.4.1 | 2026-08-31 | 1.77.2 | Listed for reference only (§9 rejects it). |

Rust stable on 2026-09-22: **1.98.1** (2026-09-01), from `channel-rust-stable.toml`.

**Conflict check:**
- **No crate conflicts.** `rusqlite_migration` 2.6.0 → `rusqlite ^0.40.0`, and 0.40.2 matches. `fsrs` shares no native or SQLite dependencies with the others. `tauri` 2.11.6 doesn't touch SQLite.
- **Effective MSRV is 1.95** (from `rusqlite_migration`), not Tauri's 1.77.2. Any current stable toolchain is fine. Just don't pin an old toolchain.
- **Latent trap:** do not also add `tauri-plugin-sql`. It pulls in `sqlx`, which links its own `libsqlite3-sys`. Two crates can't both link the native `sqlite3` library, so mixing the two stacks tends to cause a Cargo `links` conflict. The §9 decision already avoids this.
- **Tauri 3.0 alphas** (3.0.0-alpha.0/.1/.2, published 2026-09-13/15/21, MSRV 1.95) exist. Start on 2.x. Revisit when 3.0 is stable and has a migration guide.
- **`bundled` on Windows:** it compiles SQLite from C source. The MSVC "Desktop development with C++" workload you already need for Rust covers this. No vcpkg and no system SQLite needed.

## 4. Holding the SQLite connection in managed state

The official pattern is on <https://v2.tauri.app/develop/state-management/>. That page shows `app.manage(Mutex::new(AppState::default()))` inside `.setup(...)`, and commands that take `state: State<'_, Mutex<AppState>>` and call `state.lock().unwrap()`. It also says "You don't need to use `Arc` for things stored in `State` because Tauri will do this for you."

For Waypoint that becomes:

```rust
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::{Manager, State};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // resolve the DB path via Tauri's path resolver (app.path().app_data_dir())
            let conn = Connection::open(db_path)?;   // run migrations here too
            app.manage(Mutex::new(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![create_node])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn create_node(db: State<'_, Mutex<Connection>>, title: String) -> Result<i64, String> {
    let conn = db.lock().unwrap();
    // ... call into the domain module with &conn ...
}
```

Why this shape:
- `rusqlite::Connection` is `Send` but **not `Sync`** (docs.rs, rusqlite 0.40.2). It can move between threads, but two threads can't use it at once. A `Mutex` makes it shareable, and that is exactly what `State` needs.
- **Use `std::sync::Mutex`, not `tokio::sync::Mutex`.** The docs quote Tokio saying the async mutex is for "IO resources such as a database connection". That advice is about async drivers like sqlx. rusqlite is blocking, and you only need an async mutex if a guard is held across an `.await`. Waypoint's design has no such case, so keep commands synchronous (`fn`, not `async fn`).
- The state type must match exactly. Registering `Mutex<Connection>` and asking for `State<Connection>` fails at runtime, not at compile time. A type alias (`type Db = Mutex<Connection>;`) helps.
- Learner readability: one `Mutex`, one `lock()`, no generics and no `Arc`. This fits §12.2 "prefer the plain construction".
- `app.path().app_data_dir()` exists on `tauri::path::PathResolver` in 2.11.6 (docs.rs). It needs `use tauri::Manager`.

## 5. Windows development prerequisites

From <https://v2.tauri.app/start/prerequisites/>, plus current versions:

1. **Microsoft C++ Build Tools** (Visual Studio Build Tools) with the **"Desktop development with C++"** workload.
2. **WebView2**: "already installed on Windows 10 (from version 1803 onward) and later versions of Windows". Nothing to do on Windows 11.
3. **Rust** via rustup, MSVC toolchain: `rustup default stable-msvc` (host triple `x86_64-pc-windows-msvc`). Current stable is 1.98.1, and the project needs at least 1.95.
4. **Node.js LTS**: the Tauri page only says "LTS", and its example shows v20. Use **24.21.0 (Krypton, active LTS, 2026-09-07)**. v20 is too old: Vitest 5 needs ≥22.12, and jsdom 30.1.1 and npm 12.0.2 need `^22.22.2 || ^24.15.0 || >=26`. Node 26.x is "Current", not LTS.

## 6. npm vs. pnpm vs. bun (learner readability first)

| | npm | pnpm (12.5.1) | bun (1.4.2) |
|---|---|---|---|
| Extra install | none (ships with Node) | yes (Corepack or standalone) | yes (separate runtime) |
| Matches docs/tutorials | Tailwind, Vitest, Tauri and React docs show npm first | usually shown as an alternative tab | shown sometimes |
| Mental model | flat `node_modules`, like pip + venv | symlinked store, strict hoisting (can surprise) | also a JS runtime and test runner (blurs the boundaries) |
| Speed / disk | slowest | fast, dedupes | fastest |
| Written into `tauri.conf.json` | `npm run dev` | `pnpm dev` | `bun run dev` |

**Recommendation: npm.** For a reader who knows early Python:
- It needs nothing beyond Node.
- Every official page this project will lean on shows npm first. The `.manifest` also writes the manager into `beforeDevCommand` / `beforeBuildCommand`, so choosing npm keeps the repo's commands identical to the docs.
- `npm install` / `npm run X` maps simply onto `pip install` / running a script.

pnpm's speed and strictness pay off in big monorepos, which Waypoint is not. bun adds a second JS runtime to learn. Either could be revisited later at low cost, since swapping is mostly a lockfile change plus editing the two `tauri.conf.json` commands. **This is a recommendation for ADR 0001, not the decision.**

## Sources (all checked 2026-09-22)

- crates.io API: `/api/v1/crates/{tauri,tauri-build,tauri-cli,rusqlite,rusqlite_migration,fsrs,tauri-plugin-sql}` and `/dependencies` for pinned versions
- npm registry: `/<pkg>/latest` for create-tauri-app, @tauri-apps/{cli,api,plugin-opener}, vite, react, react-dom, typescript, @vitejs/plugin-react, tailwindcss, @tailwindcss/vite, vitest, @testing-library/react, jsdom, npm, pnpm, bun
- <https://github.com/tauri-apps/create-tauri-app/tree/create-tauri-app-v4.7.4/templates> (`template-react-ts`, `_base_`, `src/package_manager.rs`)
- <https://github.com/tauri-apps/tauri/releases> (tauri-v2.11.6, tauri-v3.0.0-alpha.0–2)
- <https://github.com/open-spaced-repetition/fsrs-rs> (license)
- <https://v2.tauri.app/start/create-project/>, <https://v2.tauri.app/start/prerequisites/>, <https://v2.tauri.app/develop/state-management/>, <https://v2.tauri.app/develop/tests/>, <https://v2.tauri.app/develop/tests/mocking/>
- <https://tailwindcss.com/docs/installation/using-vite>, <https://vitest.dev/guide/>
- <https://nodejs.org/dist/index.json>, <https://static.rust-lang.org/dist/channel-rust-stable.toml>
- docs.rs: `rusqlite` 0.40.2 `Connection` (Send, !Sync); `tauri` 2.11.6 `path::PathResolver::app_data_dir`
