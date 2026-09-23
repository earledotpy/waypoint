# ADR 0001 — Crate layout, write hiding and frontend tooling

Status: Proposed
Date: 2026-09-22

## In plain words

The Rust code is one Cargo workspace rooted at the repo root. It has three crates now and a fourth later. `waypoint-read` holds the shared types and the read-only queries. `waypoint-domain` is the only code that writes to SQLite: the state machine, invariants I1–I7 and the migrations. `waypoint-app` is the Tauri shell that the UI calls. The Tier 1 crate `waypoint-advisory` will depend on `waypoint-read` and nothing else, so it can't even import a write function: the import fails to compile, the same way a Python `import` fails when the package isn't in your `pyproject.toml`. Advisory never writes. It hands annotations back as values, and the app asks the domain to store them. On the frontend, Waypoint uses npm, the Tauri `react-ts` template (Vite 8, React 19, TypeScript 6.0), Tailwind 4 and Vitest. Rust and Node versions are pinned so the laptop and CI build the same way. This rules out hiding writes with `pub(crate)`, Cargo features or a capability token, and it rules out any second writer to the database.

## Context

The architecture doc (v0.3, §1) splits the Rust side into `waypoint-domain`, `waypoint-advisory` and `waypoint-app`, and makes two promises:

1. **I3 proper.** The domain never depends on advisory, so state-transition code can't read AI annotations.
2. **Write hiding.** Advisory can use the domain's read API but not its write functions.

§1 left the mechanism for promise 2 open (`pub(crate)` or a `Writer` type), and §8 deferred it to this ADR.

Two research tickets supply the facts:

- [Tauri v2 scaffold defaults and current versions](https://github.com/earledotpy/waypoint/issues/3), [`docs/research/tauri-v2-scaffold.md`](https://github.com/earledotpy/waypoint/blob/research/tauri-v2-scaffold/docs/research/tauri-v2-scaffold.md)
- [Cargo workspace layout and hiding domain writes](https://github.com/earledotpy/waypoint/issues/4), [`docs/research/cargo-workspace-layout.md`](https://github.com/earledotpy/waypoint/blob/research/cargo-workspace-layout/docs/research/cargo-workspace-layout.md)

The facts that shape this decision:

- Cargo already enforces promise 1. If advisory depends on domain, domain depending on advisory would be a cycle, and Cargo rejects it. The one exception is **dev-dependency** cycles, which Cargo allows.
- Rust has no visibility level meaning "visible to one particular other crate". `pub(crate)` hides an item from *every* other crate, including `waypoint-app`, which needs the writes.
- Cargo builds each dependency once, with the union of the features every dependent asked for. A `write` feature enabled only by the app would be switched on for advisory too.
- The compiler can't see SQL strings. Any crate holding a writable `rusqlite::Connection` can write any table.
- Tauri v2 supports `src-tauri/` as one member of a larger workspace. `tauri.conf.json` has to stay in `src-tauri/`, because the CLI finds the app by looking for it.
- The reader is learning (`AGENTS.md`). When two designs work, the one that is easier to explain wins.

## Decision

### 1. Workspace layout

```
waypoint/
├── Cargo.toml            virtual workspace: [workspace] only, no [package]
├── Cargo.lock            committed
├── rust-toolchain.toml   pins Rust 1.98.1
├── package.json          npm; engines.node ">=24.15"
├── package-lock.json     committed
├── src/                  React + TypeScript
├── src-tauri/            package name: waypoint-app (folder name kept)
│   └── tauri.conf.json
└── crates/
    ├── waypoint-read/    types + read-only queries
    ├── waypoint-domain/  the one writer: state machine, I1–I7, migrations
    └── waypoint-advisory/   (Tier 1, not created yet)
```

- The root `Cargo.toml` has `members = ["src-tauri", "crates/*"]` and `resolver = "3"`, set explicitly. Shared versions go in `[workspace.dependencies]`, and `rust-version = "1.95"` goes in `[workspace.package]`. 1.95 is the minimum, set by `rusqlite_migration`.
- `[profile.*]` sections live only in the root manifest, because Cargo ignores them anywhere else. `/target` is ignored at the root.

### 2. Who may depend on whom

| Crate | Depends on (workspace crates) | Owns |
|---|---|---|
| `waypoint-read` | none | The shared types (`SkillNode`, …) and the public read queries. From Tier 1, also a read-only `open` (`SQLITE_OPEN_READ_ONLY`) for advisory. |
| `waypoint-domain` | `waypoint-read` | Every write to SQLite: `open` + migrations, the state machine, I1–I7, the artifact store, and inserting advisory annotations. |
| `waypoint-advisory` (Tier 1) | `waypoint-read` **only** | Producing annotations, `InferenceClient`, the `llama-server` sidecar. |
| `waypoint-app` | all of the above | Tauri commands. Each is thin: parse the input, call one function, return the result. |

- **Write hiding** comes from the dependency list. `waypoint-advisory` doesn't list `waypoint-domain`, so `use waypoint_domain::…` doesn't compile there. That covers dev-dependencies too.
- **Reads go in `waypoint-read`** from the first read onward, including `list_nodes` in milestone 1. Queries the domain needs *inside* a write, such as invariant checks in a transaction, stay private to `waypoint-domain`.
- **Where read queries are tested.** In `waypoint-domain`, not `waypoint-read`. `waypoint-read` has no workspace dependencies, not even dev-dependencies, and only the domain can create the schema and the rows a read test needs.
- **Read errors.** `waypoint-read` functions return `rusqlite::Result<T>`. `DomainError` already has `From<rusqlite::Error>`, so domain code can call reads with `?`. A separate `ReadError` needs its own ADR, written when a read first has a failure that isn't a database error.

### 3. Advisory never writes

There is exactly one writer to SQLite (architecture doc §1), and this ADR keeps it that way.

- `waypoint-advisory` returns each annotation as a plain value. `waypoint-app` passes it to a domain write function (e.g. `record_annotation`) in `waypoint-domain/src/annotation.rs`.
- Advisory only ever gets connections opened read-only. The writable `Connection` stays in the app's Tauri state, and advisory is never handed it.
- So `waypoint-domain` does contain code that mentions the `advisory_annotation` table, but only in `annotation.rs` and `migrations/`, and only to create the table and insert rows. I3 ("state code never reads annotations") is held by the grep test below, not by the crate graph.

### 4. How I3 is tested

These are built when `waypoint-advisory` is created, not in milestone 1. The test names start with `i3_` so a search finds them (`AGENTS.md`).

1. **`i3_crate_graph`**, a plain Rust test in `waypoint-app` (the one crate that sees every other) using the `cargo_metadata` dev-dependency. For normal, build **and** dev dependencies, it asserts:
   - `waypoint-read` depends on no workspace crate.
   - `waypoint-domain` doesn't depend on `waypoint-advisory`.
   - `waypoint-advisory` doesn't depend on `waypoint-domain`.
2. **`trybuild` compile-fail**: `crates/waypoint-advisory/tests/ui/cannot_import_domain.rs` tries `use waypoint_domain::create_node;` and must fail to compile. The expected `.stderr` depends on the exact compiler, which is why Rust is pinned. The case is only meaningful while test 1 holds, because trybuild cases can see dev-dependencies.
3. **`i3_state_code_never_mentions_annotations`**, in `waypoint-domain`: fails if the text `advisory_annotation` appears in any domain source file other than `src/annotation.rs` and `migrations/`.

### 5. Frontend tooling

| Tool | Choice |
|---|---|
| Package manager | **npm**. Commands are `npm install`, `npm run tauri dev`, `npm test`. |
| Scaffold | `create-tauri-app`, `react-ts` template: Vite 8, React 19, TypeScript `~6.0.3`, `@vitejs/plugin-react` 6 |
| CSS | Tailwind **4.3** via `@tailwindcss/vite`: no config file, `@import "tailwindcss";` in the one CSS file |
| Frontend tests | **Vitest 5** + `jsdom` + `@testing-library/react`, with IPC mocked by `mockIPC` from `@tauri-apps/api/mocks` |
| Rust tests | plain `cargo test --workspace` |
| Database | `rusqlite` (`bundled`) + `rusqlite_migration`, never `tauri-plugin-sql` (it links a second SQLite) |
| Tauri | 2.x (2.11 at time of writing). Not the 3.0 alphas. |

### 6. Pinning

- `rust-toolchain.toml` pins **Rust 1.98.1**, so the laptop, CI and `trybuild` all use the same compiler. Upgrading is a one-line PR.
- `package.json` has `"engines": { "node": ">=24.15" }`, the floor set by jsdom 30 and npm 12. CI uses Node 24 LTS.
- Dependencies keep the template's `^` / `~` ranges. `Cargo.lock` and `package-lock.json` are committed, and CI installs from them (`npm ci`), so builds are reproducible without hand-pinning every version.

## Alternatives rejected

### Split out the *writes* instead (`waypoint-domain` + `waypoint-domain-write`)

This was the research recommendation. It gives the same compiler guarantee, but the words would change meaning: "the domain is the one writer" would become "the *domain-write* crate is the one writer". The `AGENTS.md` rule "invariants live in `waypoint-domain`" would move, and the build issues' `open`, migrations and `create_node` would move with it. Splitting out the reads keeps every existing sentence true.

### A `Writer` capability token in one crate

Every write function would take a `&Writer` that only `open()` returns, and nobody can forge one. But advisory could call `open()` itself. The guarantee is "no write without a token", not "advisory has no token". The rest is convention, and every explanation needs that caveat.

### `pub(crate)` write module

`pub(crate)` hides the writes from every other crate, including `waypoint-app`, which must call them. It would only work if the Tauri commands moved into the domain crate, and that breaks the thin-app layering.

### A Cargo feature `write`, enabled only by the app

It gives no guarantee. Cargo compiles `waypoint-domain` once with the union of the requested features, so advisory sees `write` too. It also looks like `pip install pkg[extra]`, which would mislead a Python reader.

### Sealed trait

A sealed trait stops other crates from *implementing* a trait, not from *calling* it. It's the wrong tool, and a hard one to explain.

### Create `waypoint-read` later, when advisory arrives

Milestone 1 would have one crate fewer, but every read from milestones 2–6 would pile up in `waypoint-domain` and later move in one large mechanical PR that is hard to review in one sitting. It's cheaper to put each read in the right place when it's written.

### Advisory writes its own table directly

This would give advisory a writable connection "for `advisory_annotation` only". The compiler can't enforce "only", and it creates a second writer to SQLite, which undoes architecture §1's one-writer rule.

### `cargo-deny` bans, or a CI shell script, for the crate-graph check

It's unverified whether `cargo-deny` bans apply to path-only workspace crates, and it's another tool to learn. A shell script wouldn't show up when searching for `i3_`. A plain Rust test is findable and runs under `cargo test`.

### pnpm or bun

They optimise install speed and disk use, which Waypoint doesn't need. Tauri, Vite, Tailwind and Vitest docs show npm first, so with npm the repo's commands match the docs word for word. bun also adds a second JavaScript runtime.

### Jest, Tailwind 3, TypeScript 7

Jest: Tauri's own testing docs use Vitest, and Vitest reuses the Vite config. Tailwind 3 is the config-file era, and v4 needs less setup. TypeScript 7: the template pins `~6.0.3`, and fighting the template buys nothing yet.

### Renaming `src-tauri/` or making it the workspace root

Renaming the folder to `crates/waypoint-app/` fights the CLI's config discovery and makes the repo differ from every Tauri doc. Making `src-tauri/` the workspace root puts the domain crates underneath the UI shell.

## Consequences

- **Easier:** "Can advisory change a node's state?" has a one-line answer: look at its `Cargo.toml`. "Where is I3 enforced?" is a search for `i3_`. The phrase "the domain is the one writer" stays true everywhere.
- **Harder:** a new public type or query goes in `waypoint-read`, not next to the write that uses it, so a feature can touch two crates. `waypoint-domain` needs `waypoint-read` to name its types.
- **Constrains:** any new crate that must not write depends on `waypoint-read` only. Nothing but `waypoint-domain` opens a writable connection. Annotation inserts live only in `waypoint-domain/src/annotation.rs`.
- **Build issues:** milestone 1 creates `waypoint-read` and `waypoint-domain` (in "Scaffold the Tauri app in a Cargo workspace, with CI"). `SkillNode` and `list_nodes` go in `waypoint-read`, and `create_node` goes in `waypoint-domain`. The I3 tests wait for `waypoint-advisory`.
- **Docs to reword in the v0.4 pass:** architecture §1's "`pub(crate)` module or … `Writer` type" sentence and crate table (add `waypoint-read`; advisory "produces annotations, reads read-only"), §4's I3 enforcement text, and §8's deferred item. Design doc §9's fsrs-rs licence is BSD-3-Clause, not MIT.
