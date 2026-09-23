# ADR 0003 — serde, and which derives need no ADR

Status: Proposed
Date: 2026-09-23

## In plain words

Waypoint uses the `serde` library whenever a Rust value is turned into another format (JSON for the frontend now, TOML for curriculum files later) or read back from one. A type gets that code by writing `#[derive(Serialize)]` or `#[derive(Deserialize)]`, never by hand. Seven derives may be used anywhere with no further ADR: `Debug`, `Clone`, `Copy`, `PartialEq` and `Eq` from the standard library, and `Serialize` and `Deserialize` from serde. Any other derive or macro still needs its own ADR (`AGENTS.md`, "Plain construction"). Field names are never renamed for serde, so a field's name is the same in SQL, Rust and TypeScript. This rules out hand-written conversion code, other serialization libraries, and a separate ADR for each derive.

## Context

- **Tauri requires serde.** A Tauri command sends its return value to the frontend as JSON, and Tauri only accepts return types that implement serde's `Serialize` trait. Command arguments arrive the same way and need `Deserialize`. So the moment `SkillNode` crosses into the app (issues #18 and #19), serde is not optional.
- **serde is already compiled into the app.** Tauri depends on it, so `Cargo.lock` held `serde` 1.0.229 before issue #17 added it to `waypoint-read`. Naming it directly adds no new third-party code, only the permission to use it.
- **The curriculum importer will need it too.** Curriculum files are TOML (design doc §11, Decision Log #18), and Rust's TOML libraries read into Rust types through serde's `Deserialize`. The design doc already chose TOML over YAML partly because `serde_yaml` is archived.
- **`AGENTS.md` asks for an ADR before any macro.** `#[derive(…)]` is a macro: it makes the compiler write an `impl` block (see `docs/learning/concepts/traits-and-impl.md`). The rule exists so the reader never meets code that seems to come from nowhere. Deciding each derive separately would mean an ADR for `Debug`, which `error.rs` has used since issue #16, and for each data type that follows.
- **The reader is learning** (`AGENTS.md`). A derive hides code. It's acceptable only if what it writes is predictable from the type's fields, with nothing left to decide.

## Decision

1. **serde, with its `derive` feature, is the one serialization library.** It's a workspace dependency (`serde = { version = "1", features = ["derive"] }`), and a crate that needs it writes `serde = { workspace = true }`. Format crates (such as `serde_json` or a TOML crate) are added when first needed, and each is a library choice (`AGENTS.md`).
2. **These seven derives need no further ADR:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`. What each writes is fixed by the type's fields: print every field, copy every field, compare every field, write or read every field under its own name. The first time a derive appears, the concept note says what it writes (`docs/learning/concepts/struct-and-derive.md`).
3. **Every other derive or macro still needs an ADR.** That includes standard derives whose output depends on a hidden choice. `PartialOrd`/`Ord` sort by field order, so reordering fields silently changes the sort. `Default` invents a value, such as an empty title, that may break a domain rule. It also includes third-party macros such as `thiserror`, whose hand-written equivalent `error.rs` keeps on purpose.
4. **No serde renaming.** No `#[serde(rename_all = …)]` and no `#[serde(rename = …)]`. A field's Rust name is its SQL column name and its JSON key, so one word (`created_at`) finds it in every layer. A field whose natural name is a Rust keyword uses a raw identifier (`r#type`), which serde already writes as `type`. Any other `#[serde(…)]` attribute (`skip`, `default`, `flatten` and so on) changes what the derive writes, so it needs a comment on the field saying why.

## Alternatives rejected

### Write `impl Serialize` by hand

This follows the "Plain construction" rule to the letter, like `error.rs`'s hand-written `From` impls. It lost because a hand-written `Serialize` for `SkillNode` is about twenty lines of serde's own API (a serializer, a struct, one call per field) that the reader would have to learn first. It says nothing the struct doesn't already say, and it has to be kept in step with every field added later. The `From` impls in `error.rs` are different: each makes a real choice (which variant wraps which error), and each is three lines.

### Build JSON strings by hand

`format!("{{\"id\": \"{}\", …}}", node.id)`. It lost because it's wrong as soon as a title contains a quote or a newline (escaping), and Tauri still needs a `Serialize` type, so it would add work without removing serde.

### Another serialization library (`miniserde`, `nanoserde`, `borsh`)

Smaller and faster to compile, but Tauri accepts only serde's traits, and Rust's TOML libraries read through serde. A second library would mean two ways to do one thing, and serde is compiled in anyway.

### One ADR per derive, or per type

This keeps the macro rule with no exceptions. It lost because the seven derives above have nothing to decide: each ADR would repeat this one. The rule still applies where there is something to decide (decision 3).

### Allow `rename_all = "camelCase"`

JavaScript usually spells keys in camelCase (`createdAt`), and serde can rename every field in one line. It lost because the author would then search for `created_at` in SQL and Rust but `createdAt` in TypeScript. Keeping one spelling is worth looking unusual in TypeScript.

## Consequences

- **Easier:** any data type can cross to the frontend or be read from a curriculum file with one line (`#[derive(Serialize)]` or `#[derive(Deserialize)]`), and a field added later is picked up automatically.
- **Easier:** "where does this JSON key come from?" has one answer: the struct field of the same name.
- **Harder:** the code serde writes isn't in the repo. A reader who wants to see it can run `cargo expand` (a separate tool) or read the concept note, but it can't be stepped through line by line like hand-written code.
- **Harder:** the TypeScript side uses snake_case keys (`node.created_at`), which is unusual for TypeScript. If a linter is added later, it will need its naming rule relaxed for these keys.
- **Constrains:** a new derive outside the seven, or any `#[serde(rename…)]`, needs a new ADR that supersedes or extends this one. `SkillNode` (`crates/waypoint-read/src/node.rs`) and `DomainError` (`crates/waypoint-domain/src/error.rs`) already follow it.
