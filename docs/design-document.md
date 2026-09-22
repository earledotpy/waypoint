# Waypoint — Design Document (Draft v0.3)

**Status:** Draft v0.3 (Sep 22 2026). Amends v0.2 to make the project's learning purpose explicit, fill spec gaps found in a direction review, and pull curriculum migration forward. v0.2 status, kept for history: v0.1 was reviewed by 8 independent AI research passes and 8 independent AI adversarial reviews (Claude, ChatGPT, Gemini, Mistral, MiniMax, Perplexity, Qwen, Z.ai — Sep 2026). This revision resolves the 8 original open questions with dated research and fixes the structural problems every reviewer converged on independently. It is still a draft — challenge anything below, especially the Tier 0 scope cut in §5, which is the single biggest change from v0.1.
**Origin:** Waypoint is a clean-slate desktop rewrite that takes the *idea* of SkillTrace (a Python CLI/web prototype) seriously while treating none of SkillTrace's specific implementation decisions as inherited defaults.

**Purpose (new in v0.3):** Waypoint has two goals, and they carry equal weight:

1. **A product goal.** Build a polished, desktop-native version of the evidence-gated skill tracker. SkillTrace works, but its user experience is not the one the author wants.
2. **A learning goal.** Watch closely how a desktop application is designed and built: architecture, data modelling, the Tauri IPC boundary, Rust domain code, and testing. This is groundwork for learning Rust later. The author is early in learning Python and is deliberately **not** writing Rust or TypeScript yet. Agents write the code (§12), and the project records its work so a reader can learn from it.

The product goal alone doesn't justify a rewrite, because SkillTrace already runs the core loop. The learning goal does. When the two goals conflict, for example over whether a clever abstraction is worth a harder explanation, **prefer the version that is easier to read and explain.**

**What changed from v0.2** (see §11 Decision Log #11–#16):
- The learning purpose above is now stated, and §12 adds an agent-driven build process with required learning artifacts.
- §5 gains a second, learning-oriented definition of done.
- Curriculum migration from SkillTrace and stable external node IDs move from Tier 2 to Tier 0 prerequisites (§7, OQ15).
- P2 now states the prerequisite-arithmetic exception that the architecture doc already had, so the two documents agree.
- The node state machine is complete: transitions for abandoning a node, evidencing without a session, and newly added prerequisites. It also gains a state-change audit log and a `review_log` table (architecture doc §2).
- I3 is enforced by a Cargo crate boundary rather than a lint (architecture doc §4).
- The HTML prototype's status is defined: it is the visual direction, not a behaviour spec (Decision Log #15).

**What changed from v0.1** (see §11 Decision Log for full detail and sourcing):
- Tier 0 scope cut roughly in half. AI advisory, background verification sweep, and Analytics & Portfolio are now Tier 1, not MVP. Every one of the 8 adversarial reviews independently flagged the original six-workspace-plus-local-AI Tier 0 as unbuildable by a solo developer in one pass.
- All 8 original open questions (OQ1–OQ8) are resolved below with current (Sep 2026) research, not left open.
- P3 ("AI is advisory only") is now stated honestly: what's actually structurally enforceable (AI cannot autonomously write authoritative state) is separated from what obviously isn't (AI cannot influence a human's decision once displayed). Every reviewer flagged the old claim that this was "enforced structurally" as false.
- P5 vs. the `stale` node state contradiction is resolved: `stale` is removed from `skill_node.state`. Decay is now purely a review-queue signal, never a state change.
- The AI subsystem's model recommendation is updated (Phi-4-mini-instruct is no longer an uncontested default; Qwen3-4B and Gemma 4 E4B are live alternatives; a bake-off is now specified, not implied).

---

## 1. Overview

Waypoint is a local-first, single-user desktop application for tracking skill development through **evidence of doing**, not self-reported progress. You define a skill graph, do the work, submit artifacts as evidence, and the app tracks what's actually demonstrated — with a local AI model available in an advisory role, never as an authority over your progress.

## 2. Problem & Motivation

Most learning-tracker and spaced-repetition tools fall into one of two failure modes: they reduce progress to a streak/XP number that's trivial to game, or they require constant manual bookkeeping that doesn't survive contact with a busy week. Waypoint's bet is that **evidence-gated progress** — a node in the skill graph only advances when something you actually produced backs it up — is a better foundation, and that a proper desktop GUI (not a CLI) is what makes that friction acceptable for daily use.

This is a real risk, not just a slogan: evidence-gating is exactly the kind of mechanism that can *recreate* the manual-bookkeeping failure mode it's meant to solve if the minimum unit of "evidence" is too heavy (see §5's evidence-debt note and §7 Tier 1's diagnostic work).

## 3. Target User & Builder (v1)

A single technical user (initially: the author) who is comfortable submitting artifacts (code, write-ups, screenshots, exported work) as evidence and wants a private, offline-capable system for their own skill development. Not designed for teams, classrooms, or multi-user deployments in v1.

**Explicit addition (every adversarial review flagged this ambiguity):** Waypoint is also being *built* by a single developer, in their spare capacity, on the same laptop it targets. "Single technical user" in v0.1 described only who uses the app, not who builds it — those are the same person here, and that constrains scope as much as the hardware does. §5's Tier 0 cut is a direct consequence of taking this seriously.

**Refined in v0.3:** the author directs and reviews the build but does not write its code. AI coding agents implement it against issues, and the author observes, reviews, and decides (§12). So the scarce resource is not coding hours but the author's **attention and understanding**. Scope has to stay small enough that the author can follow every merged change. A codebase the author can't explain fails the learning goal even if the app works.

## 4. Core Principles

These are the values driving the design. Each is a *position taken for this project*, not an inherited SkillTrace rule.

| # | Principle | Rationale |
|---|-----------|-----------|
| P1 | **Evidence over self-report.** A skill node advances only when it has a **linked, accepted** evidence record, not because the user clicked "done." | Prevents progress from being meaningless; the whole value proposition depends on this. (v0.2: "accepted" added explicitly — see Decision Log #3.) |
| P2 | **No automated pass/master.** Nothing — not a script, not a timer, not an AI — marks a node complete, or moves `skill_node.state` in *any* direction, without an explicit human action. **One narrow exception:** `locked ↔ available` may change automatically when prerequisite edges or prerequisite nodes change. That is pure prerequisite arithmetic, not a claim about the node itself, and it is always logged. | Keeps the human as the final authority over their own claimed competence. (v0.2: extended to *any* direction, not just advancement — closes the loophole every reviewer found in the old I2. See Decision Log #1. v0.3: the prerequisite exception, already in architecture doc §2, is now stated here too — Decision Log #13.) |
| P3 | **AI cannot autonomously write authoritative state. It may draft, flag, and propose — a human must explicitly commit.** | See §8 for the full, honest statement of what this boundary does and doesn't guarantee — v0.1's claim that this was "enforced structurally" was false as written, and every reviewer said so. |
| P4 | **Local-first and offline-capable after setup.** All core functionality works with no network connection. The AI model is fetched once on first opt-in use (§8, §10 OQ8) — that one acquisition step is the one deliberate exception, and Waypoint is fully usable with AI declined or unavailable. | Matches the single-user, personal-tool nature of the app; the offline claim is now stated precisely instead of glossed over (multiple reviewers flagged the original wording as an unacknowledged bootstrap gap). |
| P5 | **Progress does not silently regress.** Once evidence establishes `skill_node.state`, nothing automated may ever change that field, in either direction. A human can always revise their own record explicitly. | Preserves trust in the record. (v0.2: this is now airtight because P2/I2 forbid *all* automated writes to state, not just backward ones — the old wording technically permitted automated demotion, which every reviewer caught. v0.3: P2 now allows automatic `locked ↔ available` for prerequisite math, so P5's guarantee is stated precisely: automated processes never change an `in_progress` or `evidenced` node's state, and the prerequisite exception touches only `locked` and `available`.) |

Everything *not* listed here (storage format, process architecture details beyond what §9 now locks in, UI layout) is explicitly open and should be decided fresh, not carried over from SkillTrace.

## 5. Core Loop (MVP Scope — "Tier 0")

**This section changed substantially from v0.1.** The original Tier 0 was six full workspaces plus a resident local LLM — every one of the 8 independent adversarial reviews called this unbuildable as a first release for a solo developer (ChatGPT: "roadmap of a well-funded three-person startup" almost verbatim from Gemini too; Perplexity: "a small-team roadmap presented as a solo-developer MVP"). Tier 0 is now the smallest slice that tests the actual hypothesis — evidence-gated progress without self-report — end to end:

> **The core loop:** choose a node → do work → attach evidence → accept it explicitly → node becomes evidenced → a review becomes due later → you review it → repeat.

Tier 0 workspaces:

1. **Daily Brief** — due reviews, an active blocker if one exists, one recommended next action **with its driver shown inline** (which review, which blocker — not just a bare recommendation; this was identified as nearly-free and load-bearing for trusting the recommendation at all, not deferrable to Tier 1 diagnostics).
2. **Skill Graph** — DAG of skills with prerequisite edges and state per node (`locked` / `available` / `in_progress` / `evidenced` — `stale` is removed, see §10 OQ7 and Decision Log #2). Manual authoring only in Tier 0; a visual DAG renderer is a real UI risk on an unproven library and node-count ceiling (flagged by z.ai, Claude), so a flat/tree list view should exist as a fallback or companion, not be assumed away.
3. **Active Session** — logging work against a node: time-on-task, notes, a path to attach evidence when ready.
4. **Evidence Vault** — append-only evidence records with an explicit accept/reject decision. **No AI annotation in Tier 0** — human-authored notes only. This directly addresses the "evidence-debt" risk (ChatGPT's finding): the minimum evidence unit should be allowed to be small (a note, a link, a short excerpt), not require a polished artifact every time.
5. **Retention & Reviews** — FSRS-based review queue (§10 OQ11 is now resolved, not deferred — see Decision Log #7) **with an explicit workload policy from day one**, not bolted on later: a configurable daily review cap, missed-day backlog handling that doesn't dump the full backlog at once, and a manual "reset" action modeled on Anki's `Forget` (a human decision, matching P2) rather than any automatic demotion. This is a direct response to the near-unanimous finding that Waypoint's own motivating failure mode (§2) would otherwise recur inside its own review queue.

**Moved out of Tier 0, now Tier 1** (see §7): AI Advisory Subsystem, Background verification sweep, Analytics & Portfolio (the whole workspace, not just export — portfolio export's redaction design didn't exist in v0.1 and several reviewers flagged shipping an unredacted or undesigned export path as the highest-consequence gap in the whole document), diagnostic analytics.

**Definition of done for Tier 0** (absent in v0.1; every reviewer that reached "everything else" flagged its absence): the author uses the core loop, unaided, for real skill-development work, for a minimum of **four consecutive weeks**, without abandoning it for a spreadsheet or a text file. That's the actual acceptance criterion — not "six workspaces render."

**Learning definition of done (new in v0.3):** the author is observing, not yet learning Rust or TypeScript (Purpose, above). So the bar is set at **navigation and reasoning, not reading code.** Tier 0 also isn't done until the author can do the following, using the project's learning artifacts (§12) and without an agent's help:

- Using a walkthrough, follow one user action (e.g. accepting evidence) through each layer: React → Tauri command → Rust domain function → SQLite. The author should be able to name the file and function at each step.
- Explain in plain words why each invariant (I1–I7) exists, and point to the function and test that enforce it.
- Explain every ADR in `docs/adr/` and the alternative it rejected.

**Later learning track (not a Tier 0 gate):** once the author starts learning Rust, the goals become reading any domain-crate module and saying what it does, and later changing one. These are tracked separately so the product's definition of done doesn't depend on the author's Rust progress (OQ16).

**Tier 0 seed data:** manual authoring is still the only in-app authoring path. Tier 0 may also start from a one-time import of a curriculum (migrated from SkillTrace or rebuilt, see OQ15), so the four-week dogfooding period doesn't begin with an empty graph.

## 6. Explicit Non-Goals for v1

- Multi-user accounts, teams, or shared graphs
- Cloud sync / multi-device support
- Mobile app
- A public content marketplace or shared curriculum registry
- Any automated grading path that could plausibly evolve into an "AI as authority" mechanism
- Committing to a specific accessibility or platform support matrix beyond "keyboard-navigable, works on the author's Windows laptop" — a fuller matrix is Tier 1/2 work, but this floor is not optional (see §10, new OQ9)

These may be revisited later, but pulling any of them into v1 should require an explicit, argued decision — not scope creep.

## 7. Feature Tiers Beyond the Core Loop

**Tier 1 — near-term, immediately post-MVP (revised: this now includes the items cut from Tier 0 above, not just the original three):**
- **AI Advisory Subsystem** (moved from Tier 0 — see §8 for the full, updated design). Ships as an optional, fully-degradable feature: Waypoint works completely without it.
- **Background verification sweep** (moved from Tier 0's architecture diagram, where it had drifted in despite the design doc always tiering it Tier 1 — that inconsistency is fixed; see Decision Log #4). Scoped to an explicit user-controlled domain allowlist, not just `robots.txt` compliance — several reviewers noted a naive sweep risks rate-limiting/IP bans from protected sites and that `robots.txt` alone isn't a sufficient policy for arbitrary third-party URLs. Needs a triage UI so flags don't become unprocessed graveyard fuel (z.ai's phrase) — this is part of the Tier 1 definition, not a later nice-to-have.
- **Analytics & Portfolio workspace**, including export. Export requires the redaction pipeline in §10 OQ5 to actually exist before it ships — it does not exist in Tier 0.
- **Diagnostic analytics** ("why was this recommended / why am I stuck"). Note: this is explicitly *not* an AI feature by default — several reviewers pointed out that feeding advisory annotations into a "why am I stuck" synthesis is exactly the kind of broad, multi-purpose reasoning prompt §8 rules out, and that doing so would make the AI de facto authoritative over the Daily Brief. If this feature ever uses AI, it must be designed against §8's boundary explicitly, not assumed compatible with it.
- Practice profiles / mode-specific study policies.

**Moved to Tier 0 prerequisite (v0.3):** a one-way curriculum **import** in a plain-text format, with stable external IDs on every node (`skill_node.external_id`, architecture doc §2). IDs have to exist from the first migration. Adding them after real evidence records exist would mean re-keying history. The format and the migrate-versus-rebuild question are OQ15. Curriculum **export** stays in Tier 2.

**Tier 2 — exploratory, evaluate later (mostly unchanged from v0.1):**
- Curriculum export in the same plain-text format.
- Standards-based credential export (Open Badges 3.0 and similar — see §10 OQ9 below; tooling landscape confirmed thin across all 8 research passes, this remains real but non-trivial work).
- Flashcard interoperability — never as direct pass evidence.
- Generated candidate study material from the local AI, with a clear provenance chain. Never auto-promoted.
- Multi-curriculum support.

## 8. AI Advisory Subsystem (Tier 1, revised)

**This entire subsystem moved out of Tier 0.** It's designed here in full so the Tier 1 work is scoped, but nothing here blocks the Tier 0 release.

### 8.1 What "advisory only" actually means (resolves the AI-boundary contradiction every reviewer found)

v0.1 claimed the advisory boundary was "enforced structurally." It wasn't — a separate SQLite table with no foreign key stops nothing; any query in the same process can join across it. Every one of the 8 adversarial reviews said this explicitly, several using nearly the same language independently. The honest, still-strong version of the principle:

- **What's actually enforceable and is the real boundary:** no automated process — including AI inference — can write to `skill_node.state`, `evidence_record.accepted`, or `resource.last_verified_at`. This is enforced by routing every such write through a single domain-layer chokepoint (see architecture doc §1) that structurally cannot see `advisory_annotation` — not by convention, by construction (the domain layer's code has no import path to that table). This is a real invariant and it's the one that matters for P1/P2/P5.
- **What's *not* enforceable, and the design stops pretending otherwise:** once AI commentary is shown to the user, it influences their decision. That's true by definition of showing it to them, and no schema change prevents it. The mitigation is a UI rule, not a database rule: **by default, advisory commentary for a piece of evidence renders only after the accept/reject decision is recorded**, not beside the button that makes the decision. This was independently proposed by three reviewers as a cheap, concrete fix for rubber-stamp drift, and it's now the default behavior rather than an afterthought.
- **What AI is explicitly allowed to do, which is new in v0.2 and is the resolution of the steelman every reviewer ran:** AI may *draft* content a human would otherwise have to write by hand — a proposed evidence description, a flagged possible gap against a rubric, a suggested next action — as long as it is presented as a **draft requiring explicit human edit-or-accept**, never as a fact or a verdict, and never pre-populates a field that then gets silently accepted. This directly answers the friction/evidence-debt concern that six of the eight reviews raised as the real counter-argument to a pure "commentary only" boundary, without weakening P2.

### 8.2 Model (resolves OQ1 — see Decision Log #5 for full sourcing)

**No single model is an uncontested default as of Sep 2026** — this is a real change from v0.1's framing. Confirmed facts from 8 independent research passes:

- **Phi-4-mini-instruct (3.8B, MIT)** remains a credible, licensing-simplest default, but is no longer the clear class leader; several 2026 comparisons place Qwen3-4B ahead of it on instruction-following/tool-calling.
- **Qwen3-4B-Instruct-2507 (Apache 2.0)** is a strong, often-cited-superior alternative for structured output specifically. **Correction to v0.1:** the draft's "Qwen2.5-3B" candidate is wrong — that specific checkpoint ships under the non-commercial **Qwen Research License**, not Apache 2.0. If a Qwen model is used, it must be a Qwen3.x checkpoint, and the exact checkpoint's license must be verified directly against the Hugging Face model card at adoption time (Qwen's licensing varies by checkpoint, not uniformly by family).
- **Gemma 4 E2B/E4B** are real, confirmed by five independent research passes, and **Gemma 4 specifically moved to plain Apache 2.0 as of April 2, 2026** — a major and recent change from the older, more restrictive Gemma Terms of Use that still govern Gemma 1–3n. This resolves the licensing concern for *this* generation of Gemma; it does not apply to older Gemma releases.
- **Aion-1.0-Instruct** (Microsoft) is an emerging model purpose-built for exactly this hardware class (CPU-only, no dedicated GPU) but is preview-only as of this writing, with open-source release expected around end of 2026/early 2027. Worth re-checking before final lock, but not worth blocking Tier 1 on.

**Decision:** build a small evaluation harness (20–30 real evidence-record-plus-rubric cases) and run Phi-4-mini-instruct, Qwen3-4B-Instruct-2507, and Gemma 4 E4B head-to-head on the actual target laptop before picking a default. Ship with Phi-4-mini-instruct (MIT) if the bake-off doesn't show a clear winner, since it's the least legally encumbered to distribute either way.

### 8.3 Runtime (resolves OQ2 — Decision Log #6)

CPU-only inference is confirmed correct by all 8 research passes with no dissent: on this iGPU class (Intel UHD, i5-10210U), Vulkan/SYCL offload gives at best a ~2× *prefill* speedup and no meaningful *generation* speedup, because generation is memory-bandwidth-bound and the iGPU shares the same DDR4 bus as the CPU. Do not build GPU offload for v1.

For the binding itself: neither of the two competing Rust FFI wrappers (`llama-cpp-2`, `llama-cpp-4`) has a clear "more maintained" edge, and both explicitly document themselves as unsafe, unstable-API bindings — a real audit burden for a solo developer. **Decision:** run `llama.cpp`'s own `llama-server` (its OpenAI-compatible local HTTP server, same MIT-licensed codebase, built-in JSON-schema/grammar-constrained output) as a Tauri sidecar process via `tauri-plugin-shell`'s `externalBin`, rather than embedding an FFI binding in-process. This keeps Waypoint off the unsafe, fast-moving binding API, lets the inference engine version independently of the app, and gets structured output for free (constrain advisory output to a fixed schema rather than relying on model finesse — GBNF/JSON-schema grammars, not prompt engineering alone, is how "rubric-grounded" gets enforced). Define a thin `InferenceClient` interface in Rust so this choice can change later without touching the rest of the app.

### 8.4 Resource budget (new in v0.2 — every reviewer flagged the absence of this)

Explicit budgets, not implied ones:

- **Context window:** capped at 1024–2048 tokens per advisory call (one evidence record + its rubric fits comfortably under this; cutting from a 4096 default to ~1024 is close to a free 1.4–1.6× speed win per research).
- **Memory:** per-request model loading by default (not warm-kept) — on an 8GB machine already running the OS, WebView2, SQLite, and the Rust process, keeping a ~2.5–3GB model resident continuously is not a safe default. An idle-timeout warm-keep (unload after N minutes of no advisory calls) can be added later if cold-load latency proves annoying in practice, but per-request is the Tier 1 starting point.
- **Latency:** expect roughly 5–10 tokens/sec sustained generation on the i5-10210U (dual-channel DDR4, no AVX-512) per every research pass that gave concrete numbers — realistically a 45–90 second round trip for a single rubric-grounded comment including model load. This must be surfaced in the UI as a cancellable, non-blocking operation with visible progress, never a frozen screen.
- **Degraded mode is mandatory, not optional:** the app must be fully usable — including finishing the core loop — with the model absent, still downloading, or failing to load.

### 8.5 Distribution (resolves OQ8 — Decision Log #8)

**Decision:** first-run, opt-in download, not bundling into the installer. This sidesteps the Gemma-Terms redistribution question entirely if a non-Apache Gemma checkpoint is ever used, decouples model updates from app updates (avoiding multi-GB re-downloads on every app patch), and keeps installer size small. Pin by file hash (not just repo tag, since hosts re-quantize), verify checksum on download, and support a "bring your own GGUF" escape hatch for portability and future-proofing. If Phi-4-mini-instruct (MIT) is the shipped default, bundling would also be legally simple — but first-run download is still preferred for the operational reasons above.

### 8.6 Scope discipline (unchanged principle from v0.1, now with a concrete boundary)

Every advisory call is single-purpose and narrowly scoped (one evidence record + its rubric; one blocker's trace; one day's session log) — never a general-purpose chat interface, and never fed a synthesis of multiple prior annotations (that would be the diagnostic-analytics trap flagged in §7 Tier 1).

### 8.7 Data model

An "advisory annotation" is a separate, append-only record attached to an evidence record, blocker, or session (this three-target scope was already true in the architecture doc; v0.1's design doc prose narrowed it to evidence records only, which was inconsistent — fixed here). Carries provenance sufficient to actually be reproducible: model file hash (not just a name string), prompt template version, and inference parameters — a bare `model_used` string, as in v0.1's schema, isn't enough to know what actually produced a given comment if the model or prompt changes later. See architecture doc §2 for the updated `advisory_annotation` schema.

## 9. Technology Stack (revised — the central open question is now decided)

| Layer | Decision | Notes |
|---|---|---|
| Desktop shell | Tauri v2 | Unchanged from v0.1; still the right call — native webview, small footprint, local-first fit. |
| UI | React + TypeScript + Tailwind | Unchanged; reuses the existing UI prototype. Presentation-only — no direct database access from this layer. |
| **Business logic & data access** | **Rust**, via a single command-layer "domain module" that owns all writes to SQLite | **This is the resolved version of v0.1's central open question.** Every research pass and adversarial review that examined it converged on the same conclusion once pushed: the invariants in §4/architecture §4 are only actually enforceable if there's one chokepoint that owns state-changing writes. A TypeScript-heavy design (calling SQLite directly from the frontend) makes I1–I3 conventions enforced by code review, not by architecture. Rust owning this layer is what makes "AI cannot write state" a structural fact instead of a promise (§8.1). **v0.3:** Rust is also the author's intended next language. Watching a small, well-bounded Rust domain layer get built (about 11 tables, 7 invariants, one state machine) is part of the learning goal. The author won't write it yet, so it has to be written to be read (§12). |
| Persistence | SQLite via `rusqlite`, not `tauri-plugin-sql` | Direct control over transactions, the backup API, and WAL handling; migrations via `rusqlite_migration` with `PRAGMA user_version` discipline. `tauri-plugin-sql` (sqlx-based) gained a real migration framework in 2026 and remains a fine choice if a future rewrite goes TypeScript-heavy, but it's built to be called from JS, which cuts against the Rust-chokepoint decision above. |
| AI inference | `llama-server` (llama.cpp's own sidecar), driven by a Waypoint-owned `InferenceClient` Rust interface | See §8.3. Deliberately not an in-process FFI binding. |
| Spaced repetition | `fsrs-rs` (Rust) | Resolves OQ11 (see Decision Log #7) — no need to hand-roll or defer; MIT-licensed, maintained by the algorithm's own authors, ships sensible default parameters usable from a cold start. |

## 10. Open Questions — Resolved (was: "Open Questions," now closed; see §11 Decision Log for full sourcing on each)

All 8 of v0.1's open questions are now resolved above (cross-references given). Two genuinely new open questions surfaced during this research/review pass and remain open:

- **OQ9 (new):** What's the minimum accessibility and platform-support floor for a v1 that only one person will use on one known machine — is "keyboard-navigable on this Windows laptop" sufficient, or does *any* future distribution intent (even sharing the repo publicly) change that floor now rather than later? Currently answered provisionally in §6; revisit if distribution intent changes.
- **OQ10 (new):** What's the right minimum granularity for a skill node, and should Waypoint offer any authoring guidance/templates for it? No research pass found prior art to borrow here (§10 OQ9-old/OQ6 found no comparable local-first tool), so this will likely need to be answered by the author's own dogfooding during the Tier 0 definition-of-done period (§5), not by further research.

**New in v0.3, open:**

- **OQ14: What does a review actually ask?** FSRS schedules *when* a node is reviewed, not *what* the review is. The prototype shows one free-text recall prompt per review, graded Again/Hard/Good/Easy. The options are: (a) a human-authored prompt per node, stored as `review_item.prompt`; (b) re-reading your own accepted evidence and self-grading; (c) both. Tier 0 needs an answer before the Retention workspace can be built. Provisional answer: (a), with a fallback to (b) when a node has no prompt.
- **OQ15: Migrate SkillTrace's curriculum, or rebuild it?** SkillTrace has 100 nodes, 45 curated resources, and a six-slot node skeleton (SkillTrace #299). This question needs a research task, not a guess. Compare SkillTrace's node format and edge semantics against `skill_node`/`skill_edge`, list what doesn't map (manual-gate minimum counts, resource pins, spec-pending markers), then decide between migrate-and-revise and a fresh curriculum. Either way, the result is a plain-text import file with stable external IDs (§7). This also partly answers OQ10: SkillTrace's skeleton is prior art on node granularity.
- **OQ16: What are the Rust-reading prerequisites for the learning goal?** How much Rust does the author need before domain-crate walkthroughs (§12) are useful instead of noise? This is probably answered by trying one walkthrough early, on the first vertical slice.

Cross-reference table (old OQ → resolution location):

| Old # | Question | Resolved in |
|---|---|---|
| OQ1 | Best ≤4B model | §8.2 |
| OQ2 | Rust binding maturity | §8.3 |
| OQ3 | tauri-plugin-sql vs rusqlite | §9 |
| OQ4 | Backup/portability | Architecture doc §6 |
| OQ5 | Redaction before export | Architecture doc §7 |
| OQ6 | Comparable apps since SkillTrace | §11 Decision Log #9 |
| OQ7 | Permanent vs. decaying mastery | §5 item 2, §10 (old), Decision Log #2 |
| OQ8 | Bundling vs. first-run download | §8.5 |

## 11. Decision Log

This section was empty in v0.1. It's now populated from the Sep 2026 research/review pass. Each entry names what was decided, why, and which sources drove it — this is meant to stay a living record, not a one-time dump.

1. **P2/I2 rewritten to forbid all automated writes to `skill_node.state`, not just advancement.** v0.1's I2 said automated processes couldn't move state "toward a more-advanced state" — silent on demotion, which meant an automated process could legally flip `evidenced → stale` while violating P5's spirit. Every one of the 8 adversarial reviews caught this independently (several using near-identical language). Fixed by broadening P2/I2 to forbid automated writes in *any* direction.
2. **`stale` removed from `skill_node.state`.** Its presence contradicted P5 and had no defined path to being set (OQ7 was still open in v0.1, so nothing could legitimately set it). Research on comparable tools (Anki forums, Duolingo's abandoned decay mechanic, Mochi's Mastered/Put-away split) converged strongly: decay should surface as review-queue urgency only, never as a demotion of an already-evidenced node's state. `evidenced` is now permanent by default; a human can still explicitly flag a node for re-evidencing via a manual action, matching Anki's `Forget` pattern.
3. **P1 now says "linked, accepted" evidence record, not just "linked."** v0.1 was ambiguous about whether submission alone was enough (multiple reviewers flagged this); `accepted` was already in the schema but not referenced by the principle.
4. **Background sweep confirmed Tier 1 in both documents.** v0.1's architecture diagram placed it in the core native-layer box despite the design doc always tiering it Tier 1 — an inconsistency three reviewers caught. Now consistent, and given an explicit domain-allowlist + triage-UI requirement per Gemini's rate-limiting/IP-ban finding.
5. **AI model default changed from an uncontested "Phi-4-mini-instruct" to a specified bake-off among three current, correctly-licensed candidates.** All 8 research passes agreed no benchmark measures "rubric-grounded commentary" directly, so ranking must be empirical. Qwen2.5-3B (v0.1's third candidate) was flagged as licensed under the non-commercial Qwen Research License by three independent research passes — a real correction, not a nuance. Gemma 4's move to Apache 2.0 (Apr 2026) was independently confirmed by five research passes.
6. **Inference runtime changed from an in-process Rust FFI binding to a `llama-server` sidecar.** Both `llama-cpp-2` and `llama-cpp-4` are actively maintained but explicitly self-described as unsafe; z.ai and MiniMax's research independently proposed the sidecar alternative as lower solo-dev maintenance burden, and it also yields built-in structured-output support.
7. **Spaced-repetition algorithm decided as FSRS (`fsrs-rs`), not deferred.** All 8 research passes agreed with no dissent; MIT-licensed, author-maintained Rust/TS implementations exist, and FSRS's `desired_retention` parameter maps directly onto the "decay as nudge, not state change" design from #2 above.
8. **Model distribution decided as first-run download, not installer bundling.** Sidesteps Gemma Terms redistribution risk for non-Apache checkpoints, decouples model updates from app updates, avoids 2–3GB installer bloat. Independently recommended across most research passes.
9. **Positioning note (OQ6):** a cohort of cloud-based "evidence-based portfolio" apps (OnlyWorks, Fitfolio, Kynetix, and others) emerged in 2025–2026 with a similar evidence-over-self-report thesis, but none are local-first/offline-capable and most use AI in a more automated "corroborator" role than Waypoint's boundary allows. This is a genuine, currently-real differentiator worth stating in any future positioning, not just an assumption.
10. **Business logic ownership decided as Rust (§9), reversing v0.1's implicit TypeScript lean.** This was flagged by nearly every review as the single decision with the most downstream leverage, because it's what makes invariant enforcement (P2, P3/§8.1) structural rather than conventional.
**v0.3 entries** (Sep 22 2026, from a direction review against the SkillTrace repository and the author's stated goals):

11. **The project's purpose is stated as product plus learning, and the build is agent-driven.** v0.2 justified the rewrite only as product work ("a proper desktop GUI is what makes that friction acceptable"). But SkillTrace already runs the full core loop, including a browser UI, so the product argument alone is thin. The real driver is the author's goal: see how a desktop app is designed and built, as groundwork for learning Rust after Python. Consequences are in §3 (attention, not coding hours, is the scarce resource), §5 (a learning definition of done), and §12 (the build process).
12. **Curriculum import and stable external IDs pulled forward from Tier 2 to Tier 0.** The author expects to carry SkillTrace's curriculum over or rebuild it (OQ15). Either way the four-week dogfooding period needs a seeded graph. Adding external IDs after evidence records exist would force a re-key, so they have to exist from the first import.
13. **P2 now states the `locked ↔ available` prerequisite exception.** v0.2's P2 said nothing moves state "in any direction" automatically. Architecture doc §2 already allowed the automatic `locked → available` transition, so the two documents disagreed. The exception is now in both places, and it covers the reverse direction too (`available → locked` when a new hard prerequisite is added). That reverse case was unspecified; SkillTrace verified the same behaviour in its P-StudyFortnight branch B17.
14. **I3 enforced by a Cargo crate boundary instead of a lint.** v0.2 proposed a CI lint to catch imports of annotation-access code from state-transition code. Putting `waypoint-domain` and `waypoint-advisory` in separate crates of one Cargo workspace, with no dependency from domain to advisory, makes such an import a compile error. The rule then needs no maintenance, and it's easier to explain: "the domain crate cannot see the advisory crate." That also serves the learning goal.
15. **The HTML prototype (`Waypoint.dc.html`) is the visual direction, not a behaviour spec.** It shows Tier 1 surfaces (a loaded model, link sweep, portfolio, analytics) as if present. Its Daily Brief advisory card ("your last two records on this branch skipped the failure case") summarizes across several evidence records, which §8.6 forbids. Layout, typography, colour tokens, and workspace structure carry over. Where behaviour conflicts with this document, this document wins, and the prototype should be corrected before it is used as a reference for implementation issues.
16. **Evidence acceptance becomes three-state, and "current evidence" is per chain.** v0.2 left open whether a node has one supersession chain or several, and a boolean `accepted` couldn't represent "submitted, not yet decided" (which the prototype shows). Both are resolved in architecture doc §2 (v0.3).

## 12. Build Process & Learning Artifacts (new in v0.3)

Waypoint is built by AI coding agents working from issues. The author sets direction, reviews, and makes every product decision. This follows the workflow proven on SkillTrace (issues labelled research / prototype / grilling / task, ADRs, one PR per issue). The difference is that **every change also has to teach something.** Watching an agent produce a diff teaches very little. Reading an explanation of why the diff looks the way it does teaches a lot. So the process requires the following artifacts, and a change is not done without them.

### 12.1 Required artifacts

| Artifact | When | What it contains |
|---|---|---|
| **ADR** (`docs/adr/NNNN-*.md`) | Any decision that constrains future code (a crate boundary, a table shape, a library choice) | Context, the decision, **the alternatives rejected and why**, consequences. The rejected alternatives are where most of the learning is. |
| **PR explainer** (in the PR description) | Every PR | What changed; **why this shape and not the obvious alternative**; which invariant(s) it touches; one "if you only read one file, read this one" pointer. Written for a reader who knows Python but not Rust or TypeScript. |
| **Concept note** (`docs/learning/concepts/*.md`) | The first time a new idea appears in the code (e.g. Rust ownership in a function signature, `Result` and `?`, a Tauri command, a SQLite transaction, a React effect) | A short explanation of the concept *as it appears in this codebase*, with a Python comparison where one exists. Later PRs link to it instead of re-explaining. |
| **Walkthrough** (`docs/learning/walkthroughs/*.md`) | At each milestone (§12.3) | One user action traced end to end through every layer, with file and line references. This is the artifact the §5 learning definition of done is checked against. |
| **`CONTEXT.md`** | Kept current | The domain glossary (node, evidence record, chain, review item, blocker…), so agents and the author use the same words. |

### 12.2 Rules for agents working on Waypoint

These go into `AGENTS.md` / `CLAUDE.md` in the repository:

- **Keep PRs small enough to read in one sitting.** Aim for one concept per PR. Split a PR that introduces two new Rust concepts at once.
- **Prefer the plain construction.** Avoid macros, trait gymnastics, and generic abstractions unless an ADR justifies them. When there's a choice, pick explicit code over clever code.
- **Write comments that explain *why*, not *what*,** in the domain crate especially. The reader is learning.
- **Never merge without the author's review.** The author may approve a PR they don't fully understand only if a concept note or follow-up question is filed for the part they didn't follow.
- **Invariants live in one place.** Every enforcement of I1–I7 is in `waypoint-domain` and has a test named after the invariant (e.g. `i6_cannot_evidence_without_accepted_record`), so "where is I6 enforced?" is answerable with a search.

### 12.3 Milestones

Each milestone is small, ends in a walkthrough, and leaves a working app:

1. **Walking skeleton:** a Tauri window, one SQLite table (`skill_node`), one Rust command (`create_node`) called from React and rendered in a list. Its purpose is to show the whole IPC round trip at its smallest.
2. **Graph and state machine:** edges, I5 (acyclicity), the transition table, and the state-change log. No UI polish yet.
3. **Evidence:** the artifact store, evidence records, the accept/reject decision, I1/I6.
4. **Sessions and Daily Brief.**
5. **Reviews:** FSRS, `review_log`, the workload policy.
6. **Curriculum import** (after OQ15 is settled), then begin the four-week dogfooding period.

Visual polish from the prototype is applied as each workspace becomes real, not as a separate phase.
