# Cargo workspace layout and hiding domain writes

Research for [Cargo workspace layout and hiding domain writes](https://github.com/earledotpy/waypoint/issues/4), part of wayfinder map #1. Written 2026-09-22 against architecture doc v0.3 §1 and §4 (I3) and design doc §12.

This is a decision aid. The decision belongs to ADR 0001.

> **Not build-verified.** `cargo` is not installed on the machine this was written on, and nothing was installed. Every "does not compile" claim below comes from the Rust Reference, the Cargo Book or tool docs, not from a local experiment. The first implementing PR should turn each one into a real test (see §5).

## TL;DR

1. **Layout.** Make the repo root a *virtual* Cargo workspace. Keep the `create-tauri-app` folder `src-tauri/` as the `waypoint-app` member, and put the library crates under `crates/`. This is the documented, supported shape. The closest real-world match is clash-verge-rev (`members = ["src-tauri", "crates/*"]`).
2. **The ticket covers two separate guarantees, and each needs its own mechanism:**
   - **I3 proper:** `waypoint-domain` must not *depend on* `waypoint-advisory`. That is a property of the dependency graph, and Cargo already enforces it. Because `waypoint-advisory` depends on `waypoint-domain`, the reverse normal dependency would be a cycle, and Cargo rejects cycles.
   - **Write-hiding:** `waypoint-advisory` must not be able to *call* domain write functions, even though it depends on the domain crate. That is a question of visibility.
3. **`pub(crate)` does not work for write-hiding** as §1 proposes it. Rust has no visibility level that admits one other crate and excludes another. `pub(crate)` would hide the writes from `waypoint-app` as well. Two other options also fail: a Cargo feature flag (Cargo unifies features across the build) and a sealed trait (it stops other crates from *implementing* a trait, not from *calling* it).
4. **Recommendation:** split the write API into its own crate. Advisory's `Cargo.toml` then does not list it, so advisory cannot name it. This is a full compiler guarantee, and it has an honest Python analogy: *the package isn't in advisory's requirements.* The runner-up is a `Writer` capability token, which keeps three crates but gives a weaker guarantee and is harder to explain.
5. **Testing:** use two tests, one per guarantee. A plain Rust test over `cargo metadata` asserts the dependency edges. A `trybuild` compile-fail case asserts that advisory cannot reach the write API. Separately, a grep test for the SQL string `advisory_annotation` covers the one thing the compiler cannot see.

---

## 1. Workspace layout with Tauri v2

### 1.1 Facts

- **`src-tauri/` can be a workspace member.** The Tauri v2 project-structure page says `src-tauri/` "is a normal Cargo project with some extra files", and that you can "use the `src-tauri/` folder as your top level project or as a member of your Rust workspace". ([Tauri v2: Project Structure](https://v2.tauri.app/start/project-structure/))
- **`tauri.conf.json` stays inside the app crate folder.** It "is also a marker for the Tauri CLI to find the Rust project" (same page). The CLI source confirms how it finds the file. It first checks the current directory and `./src-tauri` for `tauri.conf.json`, `tauri.conf.json5` or `Tauri.toml`. If that fails, it walks subdirectories to a default depth of **3**. The environment variable `TAURI_CLI_CONFIG_DEPTH` changes that depth, and `TAURI_APP_PATH` pins the folder outright. ([tauri-cli `helpers/app_paths.rs`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-cli/src/helpers/app_paths.rs), `resolve_tauri_dir`, `lookup`.) So running `npm run tauri dev` from the repo root finds `src-tauri/` on the first check.
- **`tauri build` understands workspaces.** The current CLI runs `cargo metadata --no-deps --format-version 1` from the Tauri folder and reads `target_directory` and `workspace_root` from the result. Build output therefore goes to the shared `target/` at the workspace root. ([tauri-cli `interface/rust.rs`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-cli/src/interface/rust.rs), `get_cargo_metadata` and `get_workspace_dir`.) Older reports of the CLI looking in the wrong target folder are fixed and closed: [#2614](https://github.com/tauri-apps/tauri/issues/2614) closed in 2021 and [#6252](https://github.com/tauri-apps/tauri/issues/6252) in 2023. Setting `CARGO_TARGET_DIR` by hand was the workaround back then and is not needed now.
- **Cargo workspace rules that apply here** ([Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)):
  - All members share one `Cargo.lock` and one `target/` at the workspace root.
  - A workspace with no root package (a *virtual manifest*) "must explicitly set `resolver`".
  - `[patch]`, `[replace]` and `[profile.*]` sections take effect **only in the root manifest**. Release-profile settings that `create-tauri-app` puts in `src-tauri/Cargo.toml` have to move to the root.
  - `[workspace.dependencies]` lets each shared version (`rusqlite`, `serde`, `tauri`) be written once and inherited with `{ workspace = true }`.

### 1.2 Real open-source Tauri v2 apps that split logic into crates

Each row was checked against the repo's own `Cargo.toml` files on 2026-09-22.

| Repo | Workspace root | Tauri crate | Separate logic crate(s) | Tauri version |
|---|---|---|---|---|
| [clash-verge-rev/clash-verge-rev](https://github.com/clash-verge-rev/clash-verge-rev/blob/main/Cargo.toml) | repo root | `src-tauri` (lib target `app_lib`; `tauri.conf.json` in `src-tauri/`) | `crates/clash-verge-*` | tauri-build 2.6.3 |
| [modrinth/code](https://github.com/modrinth/code/blob/main/Cargo.toml) | repo root | `apps/app` (`theseus_gui`) | `packages/app-lib`, the crate `theseus`, which holds all launcher logic. The Tauri crate is a thin shell over it. | tauri 2.11.5 |
| [CapSoftware/Cap](https://github.com/CapSoftware/Cap/blob/main/Cargo.toml) | repo root | `apps/desktop/src-tauri` (lib `cap_desktop_lib`) | `crates/*` | tauri 2.5 |
| [spacedriveapp/spacedrive](https://github.com/spacedriveapp/spacedrive/blob/main/Cargo.toml) | repo root | `apps/tauri/src-tauri` | `core`, `apps/tauri/sd-tauri-core`, `crates/*` (Tauri is left out of `default-members` because it needs the frontend built first) | tauri 2.1 |

Modrinth is the closest match to Waypoint's *intent*: a domain library plus a thin Tauri command layer. clash-verge-rev is the closest match to Waypoint's *folder shape*. None of the four hides write APIs from sibling crates, so there was no prior art to borrow for §3.

### 1.3 Recommended layout

```
waypoint/                     ← repo root = workspace root
├─ Cargo.toml                 ← virtual manifest (no [package])
├─ Cargo.lock, target/        ← shared by every crate
├─ package.json, src/         ← React frontend (unchanged from create-tauri-app)
├─ src-tauri/                 ← package name = "waypoint-app"
│  ├─ Cargo.toml, build.rs, tauri.conf.json, capabilities/, icons/
│  └─ src/  (Tauri commands only: thin)
└─ crates/
   ├─ waypoint-domain/        ← types + read/query API (+ state machine logic)
   ├─ waypoint-domain-write/  ← only if §3 option D is chosen
   └─ waypoint-advisory/      ← Tier 1
```

```toml
# Cargo.toml (root)
[workspace]
members = ["src-tauri", "crates/*"]
resolver = "3"          # required in a virtual manifest; "3" is the edition-2024 default

[workspace.dependencies]
rusqlite = { version = "…", features = ["bundled"] }
waypoint-domain = { path = "crates/waypoint-domain" }

[profile.release]       # moved here from src-tauri/Cargo.toml; ignored anywhere else
```

The folder stays named `src-tauri/`, and only the package name changes. That keeps every Tauri doc and `create-tauri-app` default valid. The CLI's first lookup step checks exactly `./src-tauri`. Frontend paths in `tauri.conf.json`, such as `frontendDist`, are relative to `src-tauri/` and do not change.

**Gotchas to handle in the scaffolding PR:**
- Move `[profile.*]` out of `src-tauri/Cargo.toml`.
- Add `/target` to the *root* `.gitignore`. The `src-tauri/.gitignore` entry no longer matters.
- Keep `tauri.conf.json` in `src-tauri/`. Do not move it to the root, because the CLI treats its location as the app crate.

---

## 2. Two guarantees, not one

The ticket asks one question, but it contains two claims that need different machinery:

| | I3 proper | Write-hiding |
|---|---|---|
| Claim | State-determining code in `waypoint-domain` cannot reach `advisory_annotation` access code | `waypoint-advisory` can read domain data but cannot call domain *write* functions |
| Kind of fact | Dependency graph: an edge that must not exist | Visibility: an edge exists, but some items behind it must be unreachable |
| Enforced by | Cargo: a crate can only `use` crates listed in its `[dependencies]`, and normal-dependency cycles are rejected | Depends on the mechanism chosen in §3 |
| Already true with §1's three-crate plan? | **Yes** | **No.** It needs one of the §3 mechanisms. |

One caveat on I3: Cargo *does* allow a cycle through `[dev-dependencies]`. If `waypoint-domain` lists `waypoint-advisory` as a dev-dependency, domain's *tests* can import advisory, although domain's library code still cannot. Cargo's cycle detection explicitly permits dev-dependency cycles ([rust-lang/cargo#6765](https://github.com/rust-lang/cargo/issues/6765), [#10103](https://github.com/rust-lang/cargo/pull/10103)). The §5 test should check normal dependencies and also forbid that dev-dependency, because the dev-dependency would silently weaken the `trybuild` test as well.

---

## 3. Mechanisms for "readable by advisory, writable only by the app"

### 3.1 Primary facts

- **Visibility never names another crate.** The Rust Reference says `pub(crate)` "makes an item visible within the current crate", and `pub(in path)` must name "an ancestor module of the item". Outside its own crate an item is either `pub` (visible to *every* crate that depends on it) or invisible. ([Rust Reference: Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html))
- **Cargo features are unified.** "When a dependency is used by multiple packages, Cargo will use the union of all features enabled on that dependency when building it", and therefore "features should be *additive*". ([Cargo Book: Features, Feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification))
- **A sealed trait stops implementations, not calls.** Per the Rust API Guidelines ([C-SEALED](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)): "implementations of `Sealed` (and therefore `TheTrait`) only exist in the current crate." Other crates can still call its methods.
- **Private fields block construction.** A struct with a private field cannot be built with `Struct { .. }` outside its module. The only way to get one is through a function the owning crate chooses to make public (same Reference page; [C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/future-proofing.html#structs-have-private-fields-c-struct-private)).

### 3.2 Comparison

| # | Mechanism | How it works | What the compiler guarantees | Cost | Explaining it to a Python reader |
|---|---|---|---|---|---|
| A | **`pub(crate)` write module** (as §1 of the architecture doc suggests) | `mod write` inside `waypoint-domain` is `pub(crate)` | Nothing outside `waypoint-domain` can call writes, **including `waypoint-app`** | **Does not meet the requirement.** Making it work means moving the Tauri commands into the domain crate, which pulls `tauri` into the domain and removes the thin app layer. | Easy to explain ("like a leading underscore, but enforced"), but the explanation shows it hides writes from the wrong crate too. |
| B | **Cargo feature `write`** on `waypoint-domain`, enabled only by the app | `#[cfg(feature = "write")] pub mod write;` | **Nothing useful.** In a workspace build, the app enabling `write` compiles the domain crate once *with* `write`, and advisory sees it too. | **Does not work** (feature unification). | Looks like `pip install pkg[extra]`, which is exactly why it misleads: extras are per-environment, not per-importer. Worth recording as a rejected alternative. |
| C | **Sealed trait** | Write methods on a trait with a private supertrait | Only the domain crate can *implement* the trait | **Wrong tool.** Anyone who can see the trait can call it. | Hard, and the effort buys nothing here. |
| D | **Split the write API into its own crate** (`waypoint-domain` = types + reads; `waypoint-domain-write` depends on it) | `waypoint-app` depends on both. `waypoint-advisory` depends only on `waypoint-domain`. | **Total.** Advisory cannot name anything in the write crate, because the write crate is not in its `[dependencies]`. Any attempt fails with an "unresolved crate" error. The rule lives in a `Cargo.toml` diff that a reviewer can see. | A fourth crate. The invariant enforcement (I1, I2, I5–I7) and the `i5_*`/`i6_*` tests move to the write crate, so design doc §12.2 ("every enforcement … is in `waypoint-domain`") and architecture §4's test note need rewording. The read crate must not expose a raw `rusqlite::Connection` (see §3.3). | **Easiest.** "Advisory's requirements file doesn't list the write package, so `import` fails." Python can't *enforce* this (a transitive dependency is still importable in the same venv), but Rust can, which makes it a good first concept note. |
| E | **`Writer` capability token** | `pub struct Writer { _private: () }`. Every write function takes `&Writer`. The only constructor is something like `pub fn open(path) -> (Db, Writer)`, and only `main` in the app calls it. | Nobody can *forge* a `Writer`. But **anyone who can call `open`, advisory included, can get one**, and any public function that returns or stores a `Writer` leaks it. The guarantee is "no write without a token", not "advisory has no token". | Keeps three crates. The remaining gap is closed by convention and review, or by a runtime "open only once" check, which the compiler does not enforce. | Moderate. "A ticket object you have to show to write" is a fair picture, but Python has no unforgeable-object idea to compare it with, and the "but advisory could call `open` too" caveat has to be explained every time. |

### 3.3 One hole that none of A–E closes on its own

If advisory ever holds a live `rusqlite::Connection` that can write, for example one exposed as a `pub` field or returned by a getter on the read handle, it can run `UPDATE` directly and bypass every mechanism above. Rust lets you call inherent methods on a value whose type comes from a crate you do not depend on directly. (This is a Rust language fact and is unverified locally.) Two cheap defences:

- Keep the connection a **private field** of the read handle. The read API returns domain types, never the connection.
- Open advisory's connection **read-only** (`Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)` in [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/struct.OpenFlags.html)). Then SQLite itself refuses writes at runtime, as defence in depth behind the compile-time boundary.

The read-only connection only works if advisory never writes. The architecture doc currently leaves this ambiguous: §1 says advisory "owns `advisory_annotation` access", and I2 lets automated processes *append* annotations. The simplest reading that fits "the domain layer is the single writer" is this: **advisory computes annotations and returns them as values, and the app command persists them through the domain write API.** ADR 0001 should state this explicitly.

### 3.4 Recommendation

Choose **D (split write crate) plus a read-only connection for advisory.** It is the only option where the compiler guarantee is complete. The rule is visible in a four-line `Cargo.toml`, and the explanation needs no Rust-specific idea beyond "a crate can only use what it lists." That fits §12.2's "prefer the plain construction".

Choose **E** only if ADR 0001 decides the fourth crate costs too much. In that case, record in the ADR that the guarantee is partial and name the convention that covers the gap.

Reject **A, B and C** in the ADR with the reasons above. They are the tempting wrong turns, which makes them the most useful "alternatives rejected" for the learning record.

**Naming (open for the ADR):** use `waypoint-domain` + `waypoint-domain-write`, or `waypoint-model` (types + reads) + `waypoint-domain` (writes + invariants). The second keeps "domain = where invariants are enforced", so §12.2's sentence stays true, but it renames the crate that advisory depends on.

---

## 4. Consequences for other docs (for ADR 0001, not fixed here)

- Architecture §1's sentence "behind a `pub(crate)` module or a separate `Writer` type that only `waypoint-app` constructs" needs rewording. `pub(crate)` cannot work across crates, and "only `waypoint-app` constructs" is not something the compiler can enforce for a type defined in another crate.
- If D is chosen, the "invariants live in `waypoint-domain`" rule in §12.2 and the note under architecture §4 need updating to name whichever crate holds the writes.
- The annotation write path needs to be stated (§3.3).

## 5. How to test

Two tests, one for each guarantee, plus a grep for what the compiler cannot see.

**(a) Dependency-graph test (I3 proper, and for D also "advisory does not depend on the write crate").** Write a plain `#[test]` in any crate, or better in a small `xtask`/CI script, that runs `cargo metadata --format-version 1` and asserts on the edges:
- `waypoint-domain` has no dependency of *any* kind (normal, build or dev) on `waypoint-advisory`.
- `waypoint-advisory` has no dependency on `waypoint-domain-write`.

A Python reader can follow this easily: parse JSON, check a list. No new tool is required. The one-line CLI equivalent for humans is `cargo tree -p waypoint-advisory -e normal`. An alternative is [`cargo-deny`](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html) `bans` with `deny = [{ crate = "waypoint-domain-write", wrappers = ["waypoint-app"] }]` ("permits `wrappers` to have a direct dependency on the banned crate", nobody else). I did not check whether cargo-deny applies bans to path-only workspace crates, so verify that before relying on it.

**(b) `trybuild` compile-fail test (write-hiding).** [trybuild](https://docs.rs/trybuild/latest/trybuild/) (1.0.121 as of 2026-09-08) compiles each `tests/ui/*.rs` file, expects the compile to fail, and compares the compiler output with a matching `.stderr` file. Setting `TRYBUILD=overwrite` regenerates those files for review. Test cases can use the host crate's `[dependencies]` and `[dev-dependencies]`, so **where the test lives decides what it proves:**
- **Option D:** put `tests/ui/advisory_cannot_write.rs` in `waypoint-advisory`, try `use waypoint_domain_write::…;`, and expect an "unresolved import / undeclared crate" error. This proves advisory's whole dependency set, dev-dependencies included, lacks the write crate. It stays valid only while advisory never dev-depends on the write crate, and test (a) checks that.
- **Option E:** put a case in `waypoint-domain` that tries `Writer { _private: () }` and expect a private-field error. This proves the token cannot be forged, and nothing more.
- **I3:** put `tests/ui/domain_cannot_see_advisory.rs` in `waypoint-domain` with `use waypoint_advisory::…;`. It is valid only while domain has no dev-dependency on advisory (§2 caveat).

The `.stderr` text depends on the rustc version, so pin the toolchain with `rust-toolchain.toml` and regenerate on upgrade. I have not reproduced the exact error codes, since cargo is not available here.

**(c) Grep test for raw SQL.** The compiler sees Rust paths, not SQL strings. State-transition code could still contain `SELECT … FROM advisory_annotation` in a string and compile fine. A small test that fails if the text `advisory_annotation` appears in domain's state-transition modules closes that gap. Exclude `migrations/`, which legitimately creates the table.

Following §12.2, name these after the invariant, e.g. `i3_domain_does_not_depend_on_advisory`, `i3_state_code_has_no_annotation_sql`, and `advisory_cannot_reach_write_api`.

## Sources

- Tauri v2, Project Structure: https://v2.tauri.app/start/project-structure/
- tauri-cli source, `crates/tauri-cli/src/helpers/app_paths.rs` and `crates/tauri-cli/src/interface/rust.rs`: https://github.com/tauri-apps/tauri/tree/dev/crates/tauri-cli/src
- tauri-apps/tauri issues #2614, #6252 (both closed)
- The Cargo Book, Workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- The Cargo Book, Features (feature unification): https://doc.rust-lang.org/cargo/reference/features.html#feature-unification
- rust-lang/cargo #6765, #10103 (dev-dependency cycles)
- The Rust Reference, Visibility and privacy: https://doc.rust-lang.org/reference/visibility-and-privacy.html
- Rust API Guidelines, C-SEALED and C-STRUCT-PRIVATE: https://rust-lang.github.io/api-guidelines/future-proofing.html
- trybuild docs: https://docs.rs/trybuild/latest/trybuild/
- cargo-deny bans config: https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html
- rusqlite `OpenFlags`: https://docs.rs/rusqlite/latest/rusqlite/struct.OpenFlags.html
- Example repos: clash-verge-rev/clash-verge-rev, modrinth/code, CapSoftware/Cap, spacedriveapp/spacedrive (root `Cargo.toml` of each, read 2026-09-22)
