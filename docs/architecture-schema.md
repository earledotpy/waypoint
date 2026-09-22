# Waypoint — Architecture & Schema Document (Draft v0.3)

**Status:** Draft v0.3 (Sep 22 2026). Companion to design doc v0.3.

**What changed from v0.2** (design doc Decision Log #12–#16):
- The node state transition table is complete. It adds abandon (`in_progress → available`), evidence without a session (`available → evidenced`), `available → locked` when a hard prerequisite is added, explicit human reopen, and the frozen state of retired nodes.
- A new `node_state_event` table gives every state change an audit row. v0.2 said the automatic transition is "logged, not silent" but had no table to log it in.
- `evidence_record.accepted` is now nullable (`NULL` = submitted, undecided). "Current evidence" is per supersession chain via a new `chain_root_id`, and a partial unique index enforces `is_latest`.
- A new `review_log` table records every review with its grade. `fsrs-rs` needs this history to fit parameters, and v0.2 stored only the latest scheduling state. `review_item` gains `prompt` and a stated creation rule.
- `skill_node` gains `external_id` for curriculum import.
- I3 is enforced by a Cargo crate boundary, not a lint. A new invariant I7 covers state-change logging. I6 is restated per chain.

v0.2 status, kept for history: Draft, revised following the Sep 2026 research/adversarial-review pass (8 independent research reports, 8 independent adversarial reviews). Companion to `waypoint-design-document.md` v0.2 — read that first for the *why* behind the decisions below; this document is the resulting *how*.

**What changed from v0.1** (full rationale in the design doc's §11 Decision Log):
- The central open question — TypeScript vs. Rust for business logic — is resolved: **Rust owns all writes.** This one decision fixes most of the invariant-enforcement problems every reviewer found.
- `skill_node.state`'s `stale` value is removed; node state and retention/decay are now cleanly separated.
- I2 is rewritten to forbid *all* automated state writes, not just advancement.
- A new acyclicity invariant (I5) and evidence-lifecycle invariant (I6) are added.
- `evidence_record` gains decision provenance fields and an O(1) "current version" pointer instead of relying on a recursive walk of `supersedes_id`.
- `advisory_annotation` gains real provenance fields and a lifecycle status; I3's enforcement claim is rewritten to describe the actual mechanism (a Rust chokepoint with no import path to this table) instead of asserting the schema alone does it.
- Artifact storage moves from arbitrary absolute paths to a managed, content-addressed store — this was flagged by five independent reviewers as making the backup/portability story (§6) unworkable as originally scoped.
- §8's deferred-decisions list is much shorter — most of it is now decided.

---

## 1. Layered Architecture (revised)

```
┌─────────────────────────────────────────────┐
│  UI layer — React + TypeScript + Tailwind    │  (existing prototype)
│  Presentation only. No direct DB access.     │
│  Workspaces: Daily Brief, Skill Graph,       │
│  Active Session, Evidence Vault,             │
│  Retention & Reviews  (Tier 0)               │
│  + Analytics & Portfolio, AI surfaces (Tier 1)│
└───────────────────┬───────────────────────────┘
                    │ Tauri command invocations (typed, one call per intent)
┌───────────────────▼───────────────────────────┐
│  Domain layer — Rust                          │
│  The single chokepoint for all writes.        │
│  Owns: state transitions, evidence lifecycle, │
│  invariant enforcement (I1–I7), migrations.   │
│  Has NO import path to advisory_annotation    │
│  from any state-determining code path.        │
└───────────────────┬───────────────────────────┘
                    │ rusqlite
        ┌───────────┴────────────┐
        │                        │
┌───────▼────────┐     ┌─────────▼──────────────┐
│  SQLite (local  │     │  Rust native processes: │
│  file, sole     │     │  - llama-server sidecar │
│  DB persistence)│◄────┤    (Tier 1, on-demand)  │
│                 │     │  - background sweep     │
│  + managed      │     │    scheduler (Tier 1,   │
│  artifact store │     │    periodic, not long-  │
│  (content-      │     │    lived)               │
│  addressed)     │     │  Both write through the │
└─────────────────┘     │  same Rust domain layer,│
                         │  never SQLite directly. │
                         └─────────────────────────┘
```

**Resolved (was open in v0.1):** business logic lives in Rust, not TypeScript, and there is exactly one writer to SQLite — the domain layer. The UI and any Tier 1 background processes (sweep, AI inference) all route writes through the same domain-layer commands; nothing talks to `rusqlite` directly except that one module. This is what makes I2, I3, and I6 below structural rather than conventions to remember. See design doc §9 and Decision Log #10 for the reasoning; this was the single most consequential decision across all 16 review documents.

**Crate layout (new in v0.3).** The Rust side is one Cargo workspace. Its crate boundaries are how the layering above gets enforced:

| Crate | Depends on | Owns |
|---|---|---|
| `waypoint-domain` | `rusqlite`, `fsrs-rs` | Every write to SQLite, the state machine, invariants I1–I7, migrations, the artifact store. |
| `waypoint-advisory` (Tier 1) | `waypoint-domain` (read-only query API only) | `advisory_annotation` access, `InferenceClient`, the `llama-server` sidecar. |
| `waypoint-app` (the Tauri crate) | both | Tauri commands. Thin: each command parses input, calls one domain or advisory function, and returns a result. |

`waypoint-domain` does **not** depend on `waypoint-advisory`, so state-transition code referencing annotations fails to compile (I3). The advisory crate can call the domain crate's read API but has no access to its write functions. To make that true by construction, the write functions sit behind a `pub(crate)` module or a separate `Writer` type that only `waypoint-app` constructs. The implementing ADR decides which.

## 2. Data Model

Table definitions below reflect the resolved decisions above. Types are illustrative (SQLite is dynamically typed; use `CHECK` constraints for enums, enforced in the Rust layer as the primary guard since SQLite's own constraint enforcement is easy to leave disabled — foreign keys in particular must be turned on explicitly per connection, which the domain layer does at every connection open).

### `skill_node`
| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | Stable, never reused, even if node is retired. |
| `external_id` | TEXT UNIQUE NULL | **New (v0.3).** The stable ID from an imported curriculum file (e.g. a migrated SkillTrace node id). Re-importing matches on this, never on `title`. `NULL` for nodes authored in-app. |
| `title` | TEXT | |
| `description` | TEXT | |
| `state` | TEXT | enum: `locked`, `available`, `in_progress`, `evidenced`. **`stale` removed (v0.2)** — see design doc §10 OQ7 / Decision Log #2. Decay is tracked separately in `review_item`, never here. |
| `created_at` | TEXT (ISO8601) | |
| `retired_at` | TEXT NULL | Supersession, not deletion (I4). |

**Resolved (was open in v0.1, §3):** `state` is a **stored** field, written only by the Rust domain layer, never derived at read time. This was left open in v0.1 despite the schema already committing to a stored column — that inconsistency (flagged by Claude's review) is resolved by making the choice explicit: stored, single-writer, with an explicit transition table (below) rather than a derivation query. Storing it is safe now specifically *because* there's one writer to enforce consistency; that wasn't true when TS/Rust ownership was undecided.

**State transition table** (new in v0.2 — v0.1 had the enum but no transition table, which several reviewers flagged as the actual gap):

| From | To | Triggering actor |
|---|---|---|
| — | `locked` | Domain layer, on node creation, if unmet hard prerequisites exist |
| — | `available` | Domain layer, on node creation, if no unmet hard prerequisites |
| `locked` | `available` | Domain layer, automatically, when the last unmet prerequisite becomes `evidenced` — **this is the one automated forward transition that's allowed**, because it reflects prerequisite math, not a claim about the node itself. Logged, not silent. |
| `available` | `in_progress` | Human, by starting an Active Session against the node (explicit user action, not silent) |
| `in_progress` | `evidenced` | Human, by explicitly accepting an evidence record against the node (I6) |
| `available` | `evidenced` | **New (v0.3).** Human, by accepting an evidence record against a node that never had a session. Design doc §5 allows small evidence units (a note, a link) that don't need a timed session, and v0.2's table made them impossible without a pointless start/stop. Same I6 check. |
| `in_progress` | `available` | **New (v0.3).** Human, via an explicit "set aside" action. Sessions and evidence already logged are kept, and nothing is deleted. Without this, a node started by mistake stays `in_progress` forever. |
| `available` | `locked` | **New (v0.3).** Domain layer, automatically, when a new `hard_prerequisite` edge is added whose source is not `evidenced`. This is the reverse of the prerequisite-math exception, and it is logged. It applies **only** to `available` nodes: `in_progress` and `evidenced` nodes keep their state when a prerequisite is added later (P5, and SkillTrace's verified B17 behaviour, "asserted progress holding"). |
| `locked` | `available` | Also triggered when the last unmet hard-prerequisite **edge is removed**, not only when a prerequisite becomes evidenced. Same automatic, logged exception. |
| `evidenced` | `in_progress` | **New (v0.3), explicit human correction.** A "reopen" action requires a free-text reason, which is stored in the `node_state_event` row. Accepted evidence records are untouched (I1). The node can only return to `evidenced` by accepting a new or superseding record. This is the "explicit human correction" v0.2 mentioned but never defined. It is separate from the review queue's "Forget", which resets `review_item` scheduling and never touches node state. |
| any | any | **No other transition is permitted, automated or otherwise.** Retired nodes (`retired_at` set) are frozen: no transition of any kind, and they are excluded from prerequisite arithmetic for other nodes. Retiring a node that is a hard prerequisite requires the user to first remove or re-point the edge. |

Every row above writes one `node_state_event` row in the same transaction (I7).

### `node_state_event` (new in v0.3)
| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | Append-only. |
| `node_id` | TEXT FK | |
| `from_state` | TEXT NULL | `NULL` on creation. |
| `to_state` | TEXT | |
| `actor` | TEXT | enum: `human`, `prerequisite_math`. No other actor exists, which is P2 written as a type. |
| `cause_ref` | TEXT NULL | The evidence record, edge, or session that caused the change. |
| `reason` | TEXT NULL | Required for `reopen`, optional otherwise. |
| `at` | TEXT | |

It serves as the audit trail for P2/P5, and as the data behind the Daily Brief's "driver shown inline" (why a node just became available).

### `skill_edge`
| Field | Type | Notes |
|---|---|---|
| `from_node_id` | TEXT FK | |
| `to_node_id` | TEXT FK | |
| `edge_type` | TEXT | enum: `hard_prerequisite`, `soft_recommendation` |

**New constraint (I5, below):** acyclicity is enforced at edge-creation time by the domain layer — every reviewer that looked at the schema flagged that `locked`/`available` computation silently assumes a DAG with no invariant actually preventing a cycle. Also new: a uniqueness constraint on `(from_node_id, to_node_id, edge_type)` — duplicate edges were previously unconstrained.

### `evidence_record`
| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | |
| `node_id` | TEXT FK | |
| `artifact_ref` | TEXT | **Changed (v0.2):** now a reference into the managed artifact store (see §2a below), not an arbitrary filesystem path or URI. |
| `submitted_at` | TEXT | |
| `description` | TEXT | Human-authored note on what this evidence shows. May be as short as one line (design doc §5, small evidence units). |
| `accepted` | BOOLEAN NULL | Human decision, never automated (P2). **Changed (v0.3): nullable.** `NULL` = submitted, awaiting decision (the prototype's "submitted" state); `1` = accepted; `0` = rejected. `decided_at` is set exactly when this leaves `NULL`, and it never returns to `NULL`. |
| `decided_at` | TEXT NULL | **New.** When the accept/reject decision was made — v0.1 had no timestamp for this, which several reviewers flagged as insufficient audit trail even for a single-user app. |
| `rejection_reason` | TEXT NULL | **New.** Free text; only set when `accepted = false`. |
| `supersedes_id` | TEXT NULL FK | Corrections are new rows, never mutations of an accepted record (I1). |
| `is_latest` | BOOLEAN | **New.** True on exactly one record per `(node_id)` supersession chain at a time; flipped to false on the prior record in the same transaction that inserts a superseding record. **Resolves a real performance/complexity bug in v0.1:** without this, reading "the current evidence for a node" requires a recursive CTE over `supersedes_id` on every read; this makes it an O(1) indexed lookup instead (flagged specifically by Gemini's review). **Clarified (v0.3):** a node can have **several independent chains**. Each distinct piece of evidence starts its own chain, and corrections extend a chain. `is_latest` is true on exactly one record *per chain*, not per node. |
| `chain_root_id` | TEXT FK | **New (v0.3).** The first record's `id` in this supersession chain; a chain's first record points to itself. Enforced by a partial unique index: `CREATE UNIQUE INDEX one_latest_per_chain ON evidence_record(chain_root_id) WHERE is_latest = 1`. The database itself then rejects a second "latest" record, independently of the Rust check. **Implementation order matters:** SQLite checks this index per statement, not at commit, so a superseding transaction must `UPDATE` the prior record to `is_latest = 0` *before* it `INSERT`s the new one. |

**Supersession and state (v0.3):** superseding an accepted record with a newer one that is later rejected does **not** move the node out of `evidenced`. Only the human "reopen" transition does that (P5). I6 is checked at the moment of transition, not continuously.

### `advisory_annotation` (Tier 1 — see design doc §8)
| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | |
| `target_type` | TEXT | enum: `evidence_record`, `blocker`, `session` |
| `target_id` | TEXT | |
| `generated_at` | TEXT | |
| `model_file_hash` | TEXT | **New, replaces `model_used`.** A version string alone (v0.1's `model_used`) isn't enough to reproduce or audit what actually produced a given annotation if the model file changes; hash it. |
| `prompt_template_version` | TEXT | **New.** Advisory prompts will change over time; this makes old annotations interpretable against the template that produced them. |
| `inference_params` | TEXT (JSON) | **New.** Context length, temperature, etc. at generation time. |
| `content` | TEXT | The commentary itself. |
| `status` | TEXT | **New.** enum: `generated`, `reviewed`, `dismissed`, `incorporated` — v0.1 had no lifecycle for annotations at all (Perplexity's finding); "append-only" alone doesn't tell you whether the user has seen or acted on one. |
| — | — | **Enforcement (rewritten, see I3 below):** no field on this table is ever queried by the domain layer's state-writing code paths. This is enforced by the domain layer's module structure having no import of this table's access code from any state-transition function, checked by a lint/test (see I3) — not by the schema alone, which cannot enforce this by itself in SQLite. |

### `session` / `session_work`
| Field | Type | Notes |
|---|---|---|
| `session.id` | TEXT PK | |
| `session.started_at` / `ended_at` | TEXT | |
| `session_work.session_id` | TEXT FK | |
| `session_work.node_id` | TEXT FK | |
| `session_work.notes` | TEXT | |
| `session_work.evidence_record_id` | TEXT NULL FK | **New.** Links a session's work to the evidence it eventually produced, if any — v0.1 had no way to trace from a logged session to the evidence it led to, which the design doc's Active Session → Evidence Vault flow assumes exists. |

### `blocker`
| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | |
| `node_id` | TEXT FK | |
| `description` | TEXT | |
| `opened_at` / `resolved_at` | TEXT | |

*(Still minimal; v0.1 left this thin and no research pass surfaced strong comparable-tool prior art for blocker taxonomy. Left as-is deliberately rather than over-designed — flag if it proves insufficient during Tier 0 dogfooding.)*

### `review_item` (Tier 0 — now fully specified, was TBD in v0.1)
| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | |
| `node_id` | TEXT FK | |
| `due_at` | TEXT | |
| `last_reviewed_at` | TEXT NULL | |
| `stability` | REAL | FSRS parameter. **Resolved (was `retention_state` TBD in v0.1):** algorithm is FSRS (`fsrs-rs`), not deferred — see design doc §10 OQ11 / Decision Log #7. |
| `difficulty` | REAL | FSRS parameter. |
| `desired_retention` | REAL | Per-node or global default (default 0.90); user-configurable, per FSRS's own design. |
| `reset_at` | TEXT NULL | **New.** Set only by explicit human action (the "Forget"-equivalent), never automatically — this is the concrete mechanism behind design doc §5's workload-policy requirement. |

| `prompt` | TEXT NULL | **New (v0.3).** The human-authored recall question for this node (design doc OQ14, provisional answer). If it's `NULL`, the review shows the node's latest accepted evidence to re-read and self-grade. |

**Creation rule (new in v0.3):** a `review_item` is created by the domain layer **in the same transaction** that moves a node to `evidenced`, with FSRS default initial state. This is not a state write on `skill_node`, so it doesn't conflict with P2. One `review_item` per node: reopening and re-evidencing a node keeps the existing item and its history.

### `review_log` (new in v0.3)
| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | Append-only. |
| `review_item_id` | TEXT FK | |
| `reviewed_at` | TEXT | |
| `rating` | INTEGER | 1–4 = Again / Hard / Good / Easy (FSRS ratings), always entered by the human. |
| `elapsed_days` / `scheduled_days` | REAL | Needed by the FSRS optimizer. |
| `stability_after` / `difficulty_after` | REAL | The new memory state this review produced (also copied onto `review_item`). |
| `kind` | TEXT | enum: `review`, `reset`. A "Forget" writes a `reset` row with no rating, so resets are visible in history rather than silent. |

Why it's needed: `review_item` alone holds only the *current* memory state. `fsrs-rs` can run on defaults from a cold start, but fitting parameters to the author's actual memory needs the per-review history, and so do the Tier 1 analytics. Capturing it later can't recover reviews already done.

**New, required for Tier 0 (design doc §5 item 5):** a review-workload policy sits above this table at the domain-layer level — a configurable daily review cap and backlog-spreading logic when `due_at` has piled up after an absence. This is application logic, not schema, but it's called out here because v0.1 treated it as entirely absent rather than deferred.

### `resource`
| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | |
| `node_id` | TEXT FK | |
| `url_or_ref` | TEXT | |
| `last_verified_at` | TEXT NULL | Set only by explicit human action or accepted evidence, never by the background sweep (I2). |
| `last_sweep_flagged_at` | TEXT NULL | Set by the background sweep (Tier 1) when it detects a problem — flag only, never a verification. |

### §2a. Artifact store (new in v0.2)

v0.1 stored `artifact_ref` as an arbitrary path or URI. Five independent reviewers flagged this as making the backup/portability story (§6) essentially fictional: a database backup that references a file at `C:\Users\jeremy\Desktop\thing.png` is worthless the moment the file moves, is renamed, or the database is restored on another machine.

**Decision:** artifacts submitted as evidence are **copied** into an app-managed directory inside the app data root, named by content hash (deduplicating identical artifacts for free and giving a free integrity check). `evidence_record.artifact_ref` points into this store, not to the original location. This is the same shape Anki, Zotero, and Joplin all converge on independently (per research §B6), and it's what makes the single-archive backup in §6 below actually self-contained.

## 3. State Machine Notes

Resolved above (§2's transition table) — this section in v0.1 posed the stored-vs-derived question and the mastery-permanence question as still open; both are now decided (design doc §10 OQ7, Decision Log #2/#10).

## 4. Key Invariants (revised)

| ID | Invariant | Change from v0.1 |
|---|---|---|
| I1 | Evidence records are immutable once **decided** (`accepted` is `1` or `0`); corrections are new rows referencing `supersedes_id`, with `is_latest` flipped in the same transaction. **v0.3:** while `accepted IS NULL` (submitted, undecided), `description` and `artifact_ref` may be edited in place, because an undecided submission is still a draft. A rejected record is as immutable as an accepted one: rejected stays rejected, and a retry is a superseding row. | Unchanged in substance; `is_latest` mechanism added. v0.3: defines the pending window that nullable `accepted` created. |
| I2 | **No automated process may write `skill_node.state` in any direction**, set `evidence_record.accepted = true`, or set `resource.last_verified_at`. The one exception is prerequisite math: the `locked ↔ available` transitions in §2's table (v0.3: both directions), which are logged via I7. Automated processes may only flag (`last_sweep_flagged_at`) or append (`advisory_annotation`). | **Rewritten.** v0.1 only forbade automated writes "toward a more-advanced state," silently permitting automated demotion — every one of the 8 reviews caught this. Now closed in both directions. |
| I3 | `advisory_annotation` is never read by any code path that determines node state, eligibility, or gating. **Enforcement mechanism, stated honestly (v0.2):** this is true because the Rust domain layer's state-transition module has no import of the annotation-access module — a structural fact about the codebase, checked by a lint/CI test that fails the build if such an import appears — not because the schema alone prevents it. v0.1 claimed the schema enforced this; it didn't, and every reviewer said so. **v0.3: the lint is replaced by the crate boundary in §1.** `waypoint-domain` has no dependency on `waypoint-advisory`, so the forbidden import doesn't compile. | **Rewritten** to describe the actual mechanism instead of overclaiming. v0.3: enforced by the compiler (design doc Decision Log #14). |
| I4 | `skill_node.id` is never reused, even after retirement — a retired node is marked `retired_at`, not deleted, so historical evidence records remain valid. | Unchanged. |
| **I5** | **New.** `skill_edge` insertions are rejected by the domain layer if they would create a cycle among `hard_prerequisite` edges. Checked at write time (a graph the size Waypoint targets makes a cycle check on insert cheap; no need for a background job). | New — closes the DAG-without-acyclicity gap five reviewers flagged. |
| **I6** | **New.** A node may only be written to `evidenced` state if, at that moment, at least one of its supersession chains has a latest record (`is_latest = true`) with `accepted = true`. (v0.3: stated per chain; see `chain_root_id`.) Enforced by the domain layer as the *only* code path that can write `state = 'evidenced'` (see §2 transition table) — never a bare UPDATE. | New — makes P1's "linked, accepted evidence" claim actually true of the schema, not just the prose (ChatGPT's finding: v0.1's schema had no constraint tying `evidenced` to any evidence record at all). |
| **I7** | **New (v0.3).** Every change to `skill_node.state`, including creation, writes exactly one `node_state_event` row in the same transaction, with `actor` set to `human` or `prerequisite_math`. A state change with no event row is a bug. | New. It makes v0.2's "logged, not silent" claim checkable, and it makes P2 auditable after the fact. |

**Tests (v0.3):** each invariant has at least one domain-crate test named after it (e.g. `i5_rejects_cycle_on_edge_insert`), per design doc §12.2.

## 5. Background Processes (Tier 1 — scope tightened)

- **Verification sweep**: periodic (not long-lived/continuous — v0.1's architecture diagram implied always-running, which doesn't fit a desktop app that's closed most of the time; design doc §7 already said "periodic," now the diagram agrees). Scoped to an **explicit, user-visible domain allowlist**, not `robots.txt` compliance alone — `robots.txt` doesn't cover arbitrary third-party URLs a user might link, and running unthrottled automated requests from a residential IP risks rate-limiting or bans from protected sites (Gemini's specific, concrete finding). Writes only to `last_sweep_flagged_at`, per I2. Needs a triage UI (design doc §7) so accumulated flags don't become unprocessed noise.
- **AI inference process**: `llama-server` sidecar, invoked on-demand (design doc §8.4 — **per-request loading, not warm-kept**, given the 8GB RAM budget). This was an open question in v0.1 ("keep warm vs. per-request... needs an explicit decision"); now decided in favor of per-request as the safer default on this hardware, revisitable if cold-load latency proves worse in practice than the RAM pressure of staying warm.

## 6. Storage & Portability (resolves OQ4 — was fully open in v0.1)

- **Primary storage:** single SQLite file (`rusqlite`, WAL mode) plus the managed, content-addressed artifact store (§2a) — both under one app-data root. This pairing, not the database file alone, is what's portable.
- **Backup mechanism:** scheduled snapshots via `VACUUM INTO` (WAL-safe — produces one consistent, compacted file with no `-wal`/`-shm` siblings, unlike a raw file copy while the app is running, which several research passes specifically warned can silently corrupt a backup). Every backup write runs `PRAGMA quick_check` before being kept; a failed check means the previous backup is *not* deleted.
- **Portable export:** a single-archive "Export backup" bundling the SQLite snapshot **and** the artifact store together — the same shape as Anki's `.colpkg`, independently converged on by four research passes as the proven pattern for exactly this DB-plus-attachments problem. A DB-only backup is not considered a complete backup, since `artifact_ref` now points into the managed store rather than an external path.
- **Restore is tested, not assumed:** a CI test performs a fresh restore from a known-good backup and asserts `integrity_check` passes. v0.1 had no backup mechanism at all, let alone a tested restore path — this was flagged as the single largest unaddressed single-point-of-failure risk across the review set.
- **Migrations:** `rusqlite_migration`, versioned via `PRAGMA user_version`, run at Rust-layer startup before any command is accepted.

## 7. Security & Privacy Considerations (resolves OQ5 — was fully open in v0.1)

- All data local by default; no network calls required for core (Tier 0) functionality. The AI model's one-time first-run download (design doc §8.5) and the Tier 1 verification sweep are the only network dependencies, and both are opt-in/deferred out of Tier 0.
- **Export redaction pipeline (new, concrete — v0.1 said only "not yet designed"):**
  - Exports are built from an explicit **allow-list of exportable fields** — never by serializing a full record and stripping dangerous bits afterward. This framing, not a specific tool, was the strongest convergent recommendation across the research (a "detect and scrub" approach on free text is unfalsifiable — you can never prove it caught everything).
  - **Free-text fields (session notes, evidence descriptions) are excluded from portfolio export by default.** If the user opts in to including one, it goes through a mandatory per-item human review-and-confirm step that shows the exact text before it leaves the machine. No automated free-text redaction is trusted as sufficient on its own — this was the strongest point of agreement across every research pass that addressed it.
  - Structured fields (paths, hostnames, usernames) get a deterministic normalization pass before export.
  - Any exported image or document artifact gets a metadata-strip pass (EXIF, author fields) — flagged by multiple passes as the thing most redaction implementations miss.
- **At-rest encryption is explicitly deferred, not designed here** — noted as a real, not-yet-addressed gap (MiniMax's research specifically flagged this as missing from the original open-questions list; `tauri-plugin-turso`'s native AES-256 option or SQLCipher via `rusqlite`'s `bundled-sqlcipher` feature are candidates to evaluate for Tier 1/2, not blocking Tier 0).
- No secrets/credentials expected in v1 scope (no cloud sync, no external accounts).

## 8. Explicitly Deferred Technical Decisions (much shorter than v0.1 — most items are now decided above)

- Exact review-workload cap default (a number, not a policy — the policy itself is now decided in §2/review_item).
- Whether to add model warm-keep as an option once real-world cold-load latency is measured (§5).
- At-rest encryption mechanism (§7).
- Skill-node granularity guidance / authoring templates (design doc §10 OQ10 — likely answered by dogfooding, not further research).
- Exact accessibility/platform support matrix beyond the Tier 0 floor stated in the design doc §6.
- **(v0.3)** Curriculum import file format and the SkillTrace-to-Waypoint field mapping (design doc OQ15). It is blocked on a research task and must be settled before milestone 6 (design doc §12.3). SkillTrace concepts without a v0.3 home yet: manual-gate minimum evidence counts, resource pins, and spec-pending markers.
- **(v0.3)** Review content (design doc OQ14). The schema supports the provisional answer (`review_item.prompt` with an evidence re-read fallback); confirm it before milestone 5.
- **(v0.3)** How domain write functions are hidden from `waypoint-advisory` (`pub(crate)` module vs. a `Writer` capability type, §1). The first ADR in the repository decides this.