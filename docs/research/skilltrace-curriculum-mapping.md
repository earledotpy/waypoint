# SkillTrace curriculum format vs. Waypoint schema

**Ticket:** SkillTrace curriculum format vs. Waypoint schema (earledotpy/waypoint#2), part of wayfinder map #1.
**Feeds:** OQ15 (migrate or rebuild). This is a decision aid: facts and a draft mapping, not a verdict.
**Sources:** SkillTrace repo at `origin/main` commit `f8ec443` (2026-09-22), read with `git show` / `git archive` (the clone was not modified); SkillTrace issues #291, #295, #298, #299; the audit doc `docs/research/curriculum-renewal-audit.md` on SkillTrace branch `research/curriculum-renewal-audit` (commit 76cc477); Waypoint `docs/architecture-schema.md` v0.3 §2.
Counts marked *(computed)* come from parsing the YAML and Markdown at `f8ec443` myself, not from SkillTrace's own docs.

---

## 1. Headline

- **The IDs are safe to key on.** SkillTrace node IDs are immutable by doctrine, and the history confirms it: no node file was ever deleted or renamed. Waypoint v0.3's `skill_node.external_id` fits them directly.
- **There is no learner history to migrate.** SkillTrace has 0 evidence records, 0 attempts, 0 sessions, 0 blockers, 0 reviews, and no asserted progress. Its only data is the curriculum (nodes, edges, resources, specs, gates) and derived readiness.
- **The curriculum is uniform.** All 100 nodes follow the six-slot body skeleton. The node set matches the v1.8/v1.9 specs, and SkillTrace's own audit found no structural invariant violations.
- **The mapping loses the most in five places:** the whole evidence-definition layer (ArtifactSpec and ValidationGate, including minimum counts and objective gates) has no home in Waypoint. The `remediation` edge type has no home. SkillTrace's shared many-to-many resource registry doesn't fit Waypoint's per-node `resource` rows. The node body and about ten frontmatter keys collapse into one `description` field. Fixed-interval reviews with a pass/fail outcome don't match FSRS.
- **One conflict is about doctrine, not a missing field:** SkillTrace's 23 objective gates accept evidence automatically (`accepted_by: objective_gate`). Waypoint P2/I6 make acceptance human-only, so that path can't be represented.

## 2. What the curriculum looks like as data

| File | What it holds | Size at `f8ec443` *(computed)* |
|---|---|---|
| `graph/nodes/<id>.md` | One Markdown file per node: YAML frontmatter plus a six-slot body | 100 files |
| `graph/edges.yaml` | Every node relationship. Nodes never list their own prerequisites | 157 edges |
| `graph/resources.yaml` | LearningResource registry, one entry per material, with `supports:` node lists | 45 entries |
| `evidence/artifact_specs.yaml` | What evidence a node needs (kind, count, required) | 81 specs, exactly 1 per gated node |
| `evidence/validation_gates.yaml` | Who or what judges the evidence (`manual` / `objective` + `command`) | 81 gates (58 manual, 23 objective) |
| `evidence/checks/*_check.py` | Checker scripts that objective gates run | 23 checkers + 3 shared harnesses (`_loader`, `_sql`, `_pandas`) |
| `graph/state.yaml` | Progress store (ADR 0001), kept separate from curriculum | 100 entries: 49 `available`, 51 `locked`, none asserted, no `transitions` |
| `evidence/evidence_records.yaml`, `attempts.yaml` | Learner evidence and attempts | empty |
| `execution/{sessions,session_work,blockers,reviews,remediation_actions}.yaml` | Learner execution history | all empty |
| `execution/events.yaml` | Command audit log | 34 events: 30 `verify-resource`, 4 `sync` |
| `policy/*.yaml` | Tunable values (track weights, review cadence, remediation threshold, staleness window) | 12 files |

Sources: `docs/SCHEMA_REFERENCE.md`, `docs/curriculum-authoring.md`, and the files themselves.

### 2.1 Node file format

SkillTrace's frontmatter schema is **open**: unknown keys are tolerated, and only `state`, `prerequisites`, `unlocks`, and `node_type` cause a load error (`SCHEMA_REFERENCE.md`, `src/skilltrace/graph/nodes.py`). Keys actually in use *(computed)*:

| Key | Nodes | Notes |
|---|---|---|
| `id`, `title`, `summary`, `domain`, `track`, `tags`, `estimated_effort{min_minutes,max_minutes}`, `micro_session_fit{can_fit_15_min,can_fit_30_min,requires_long_block}`, `created_at`, `updated_at` | 100 | The "lean node template" in `curriculum-authoring.md` |
| `roadmap_anchors[]` (`phase`, `phase_label`, `month_range`, `roadmap_topic`, `source_role: reference_only`) | 43 | Metadata only. Never controls locking or ranking |
| `source_metadata{primary_source, canonical_url, source_version, supporting_sources}`, `regeneration_key`, `section_provenance[]` | 19 | The v1.8 `ml.*` and v1.9 `agents.*` cohorts |

- `track` values: 81 `foundational`, 10 `portfolio`, 5 `remediation`, 4 `consolidation`. The weights are 3.0 / 1.0 / 0.5 / 2.0 (with `core` at 2.0 but unused) and live in `policy/recommendation.yaml`. The engine treats track names as opaque.
- `domain` values: mathematics 34, data 20, programming 18, agents 12, ml 7, tooling 6, communication 3.
- `micro_session_fit` is derived from `min_minutes` using a fixed table (`curriculum-authoring.md`).

**Body: the six-slot skeleton** (decided in G-CurriculumDirection #295, learner-approved 2026-09-17, and written into `curriculum-authoring.md` by G-AuthoringPattern #299, commit 522ba8c): `## What this skill is` / `## Why this skill` / `## What passing requires` (a pointer only; the spec and gate are the canonical surface) / `## How to work on it` / `## Resources` (registry IDs, not URLs) / `## Notes` (provenance, spec-pending marker, review date; never learner state).
**Conformance: 100 of 100 nodes have all six headings** *(computed)*.

### 2.2 Node ID scheme and stability

- Format: two or more dot-separated `[a-z0-9]+` segments, with the last segment ending in `_\d+`, e.g. `math.algebra.linear_equations_01` (`SCHEMA_REFERENCE.md`). The suffix is a sequence number, not a version (`curriculum-authoring.md`). All 100 current IDs end in `_01`.
- Doctrine: an ID is immutable and never reused. Wording or label drift is edited in place. A change of skill substance gets a new ID, and asserted progress is never inherited (`curriculum-authoring.md`, "Migration criteria", from #295/#299).
- **History** *(computed from `git log origin/main`)*: 100 node files were ever added under `graph/nodes/`, 0 were deleted, and 0 were renamed (first commit a51f4b6, 2026-07-01). Across every revision of `graph/edges.yaml`, 157 distinct edge IDs ever existed and all 157 still exist. Nothing has been retired or superseded.
- Other ID shapes: edges `edge.<source>__<target>`, specs `spec.<...>`, gates `gate.<...>.closure`, resources kebab-slug (`khan-algebra`), evidence `ev.<node_id>.NNN`.

### 2.3 Edges

Edge schema is **closed**: `id`, `source`, `target`, `edge_type`, `reason` (required), `active` (bool), `created_at`, `updated_at` (`SCHEMA_REFERENCE.md`).

| `edge_type` | Count | Semantics |
|---|---|---|
| `hard_prerequisite` | 57 | "The only wall": locks the target until the source is `passed`/`mastered`. Authored with the minimal-locking test, and the `reason` must say why the target is *incoherent* without the source (`curriculum-authoring.md`). |
| `soft_prerequisite` | 81 | Never locks. Affects recommendation only. |
| `remediation` | 19 | Runs from a remediation node to the skill it rescues. Inactive at rest. It activates at runtime (derived, never stored) when the target has an open Blocker or reaches `failed_attempt_threshold` (3) failed attempts, and it deactivates when the remediation node is passed or the blocker is resolved. It boosts recommendation priority only (CONTEXT.md, `policy/remediation.yaml`, `src/skilltrace/policy/remediation_edges.py`). |

All 157 edges have `active: true`. `active` is an authoring switch; remediation activation is computed separately. Readiness reads only active `hard_prerequisite` edges (`src/skilltrace/graph/readiness.py`).

### 2.4 Resources (45 entries)

LearningResource schema is **closed**: `id` (kebab-slug), `cost` (`free`|`paid`, required), `url` / `local_path` (at least one), `free_tier`, `certificate`, `license`, `supports[]` (node IDs), `last_verified` (ISO date, absent = unverified), `broken{date, reason}` (`SCHEMA_REFERENCE.md`).

*(computed)*:
- 45 entries, all `cost: free`, all with `url` and `last_verified`. 16 carry `license`, 3 `certificate`, 1 `free_tier`, 0 `broken`, 0 `local_path`.
- It's a shared **many-to-many** registry: 220 (resource, node) pairs in total. Each node has 1–8 resources (18 nodes ×1, 52 ×2, 26 ×3, 3 ×4, 1 ×8), and one resource supports up to 13 nodes (`khan-algebra`).
- **Resource pins** have no schema field of their own. They're written as prose inside `license` on the 16 ML/agents entries: source/version, section list, and a `regeneration key <tool>-<version>` (e.g. `openai-agents-0.22.3`, `langgraph-1.2.11`, `mcp-2025-06-18`). The 19 ML/agents nodes repeat the pin in `source_metadata.source_version` and `regeneration_key`. G-ResourceRevisions (#301, commit a17dfa5) last re-pinned them on 2026-09-18.
- **Verification:** `last_verified` dates are 29 × 2026-07-10, 4 × 2026-09-05, 8 × 2026-09-06, 4 × 2026-09-18. Only the 29 foundations entries plus `hf-agents-course` have a matching `verify-resource` event (30 events). The other 15 ML/agents dates were written directly in seed commits. 12 of the 16 ML/agents `license` strings still say "unverified by design, re-pin at spec time", and 11 of those also say "human last_verified pending". That text contradicts the entry's own `last_verified` value, even for `hf-agents-course`, which has a verify event. Status (unverified / verified / stale after 180 days) is derived, never stored, and advisory only (`policy/resource_verification.yaml`). Doctrine says only a human runs `verify-resource` (`curriculum-authoring.md`).
- **Body vs registry drift:** in 18 of 100 nodes, the body's `## Resources` IDs don't match the registry's `supports` lists *(computed)*. Example: `ml.framing.ml_workflow_01` lists `islp-python-edition` in its body, but the registry maps `kaggle-learn-ml` to it instead. The registry is the canonical store per doctrine. The audit didn't report this.

### 2.5 Gates, specs, and markers

- **ValidationGate** (closed): `id`, `node_id`, `authority` (`manual`|`objective`), `command` (required if objective, forbidden if manual), `title`, `description`. An `ai` authority can't be represented, by design.
- **ArtifactSpec** (closed): `id`, `node_id`, `title`, `artifact_kind` (free text, 15 values in use, e.g. `problem_set` 33, `code_snippet` 26), `required`, `minimum_count` (≥1), `description`, `expected_location_hint`, `example_filename`, `acceptance_summary`.
- **Minimum evidence counts** *(computed)*: 31 manual specs need 3 records, 27 manual specs need 1, and all 23 objective specs need 1. All 81 specs are `required: true`.
- **Pass eligibility** is derived: every required spec has at least `minimum_count` accepted, non-superseded records. Passing and mastering are explicit learner commands. Mastery eligibility means passed, plus at least one satisfactory review, on a different day from the pass (CONTEXT.md, `src/skilltrace/evidence/eligibility.py`).
- **Spec-pending markers**: 19 nodes (all 12 `agents.*` and all 7 `ml.*`) have no spec and no gate. Each has a prose marker in `## Notes` ("Spec-pending: no artifact spec or validation gate exists yet…"), and their `What passing requires` slot says nothing can be submitted yet. **The ungated set and the marked set are identical** *(computed)*. The audit counted 6 gate-less nodes before G-SpecPendingMarkers landed, so its count predates the markers. The marker is free text, not a field.

## 3. Size and quality signals from SkillTrace's own audit

From R-CurriculumAudit #291 (resolution comment and audit doc) and D-ReviseVsRebuild #298:

- The audit's leaning was **keep the structure**: 0 forbidden frontmatter keys, a clean progress store, every edge and `supports` ID resolves, no duplicate URLs, no isolated nodes, and the snapshot matches spec-v1.9 (100 nodes, 157 edges).
- The node sets were human-locked with provenance, and **no material change since lock** was found. All drift was at the label/pin level, which is an in-place edit.
- Revise items: the two-sources-of-truth risk around "what passing requires" (since addressed by the canonical-surface rule and the six-slot fill); resource pin/label drift (since addressed by #301); docs drift.
- #298 was closed as a duplicate of G-CurriculumDirection #295. #295 chose **revise, not rebuild** for SkillTrace's own curriculum, and graduated G-AuthoringPattern, G-FoundationsFill, G-SpecPendingMarkers, and G-ResourceRevisions.
- **Discrepancy:** the audit reports 46 resources, but `graph/resources.yaml` has 45 entries both at the audit's own snapshot commit (6dafd42) and at `f8ec443`. 45 is the verified count.
- Remaining uncertainty from the audit: effort hours are seed estimates, never measured, and some version pins (FastAPI, PydanticAI sections) weren't tied to dated release records.

## 4. Draft field-by-field mapping onto Waypoint v0.3

Fidelity key: **clean** = direct; **lossy** = maps with information dropped or distorted; **no home** = no v0.3 field.

### `skill_node`

| SkillTrace | Waypoint v0.3 | Fidelity |
|---|---|---|
| `id` | `external_id` (re-import key); Waypoint mints its own `id` | **clean**. IDs verified stable (§2.2) |
| `title` | `title` | clean |
| `summary` + six-slot body | `description` (one TEXT) | **lossy**: the slot structure and `summary` as a separate sentence are flattened unless `description` holds Markdown |
| `created_at` | `created_at` | clean (SkillTrace uses date-only values) |
| `updated_at` | — | no home |
| `state.yaml` `locked`/`available` | `state` `locked`/`available` | clean, but recomputable. Nothing is asserted, so nothing needs carrying |
| `active` / `passed` / `mastered` | `in_progress` / `evidenced` / — | **lossy**. `mastered` has no home. Unused in current data |
| `domain`, `tags` | — | no home |
| `track` + track weights | — | no home (recommendation weighting) |
| `estimated_effort`, `micro_session_fit` | — | no home (session-fit scoring) |
| `roadmap_anchors` (43 nodes) | — | no home |
| `source_metadata`, `regeneration_key`, `section_provenance` (19 nodes) | — | no home |
| Spec-pending marker (19 nodes, prose) | — | no home as a field. Survives only as description text |

### `skill_edge`

| SkillTrace | Waypoint v0.3 | Fidelity |
|---|---|---|
| `source`, `target` | `from_node_id`, `to_node_id` | clean |
| `hard_prerequisite` (57) | `hard_prerequisite` | clean. Same locking meaning |
| `soft_prerequisite` (81) | `soft_recommendation` | clean. A rename only; neither locks |
| `remediation` (19) | — | **no home**. The enum has two values, and runtime activation (blocker / 3 failed attempts) has no Waypoint mechanism |
| `reason` (required on all 157) | — | **no home**. The authored justification the minimal-locking test depends on |
| `id`, `active`, `created_at`, `updated_at` | — | no home |

Both schemas require a DAG. Waypoint enforces acyclicity and `(from, to, type)` uniqueness when an edge is created. SkillTrace's audit found no duplicate edge IDs.

### `resource`

| SkillTrace | Waypoint v0.3 | Fidelity |
|---|---|---|
| Registry entry × `supports[]` | one row per (resource, node) with `node_id` FK | **lossy**: 45 shared entries become about 220 rows, and resource identity across nodes is lost |
| `url` / `local_path` | `url_or_ref` | clean |
| `last_verified` | `last_verified_at` | **lossy**: I2 reserves this for human action or accepted evidence, but 15 of the 45 dates were written in seed commits with no verify event, and 12 entries' `license` text calls them unverified |
| `broken{date, reason}` | `last_sweep_flagged_at` | **lossy**: Waypoint's field is a sweep-set timestamp with no reason. SkillTrace's `broken` is a human marker. Unused in current data |
| `id` (slug), `cost`, `free_tier`, `certificate`, `license` (incl. pins and regeneration keys) | — | **no home** |
| Body `## Resources` lines | — | Redundant with `supports` and out of sync in 18 nodes. You have to pick one as truth when migrating |

### `review_item` / `review_log`

| SkillTrace | Waypoint v0.3 | Fidelity |
|---|---|---|
| Review rows: fixed 1/3/7-day intervals scheduled all at once on pass (`policy/review_cadence.yaml`) | one `review_item` per node, FSRS `due_at` / `stability` / `difficulty` | **structurally different**: several scheduled rows vs one scheduler row |
| `outcome` `satisfactory`/`unsatisfactory` | `review_log.rating` 1–4 | **lossy**: binary to four-point has no defined mapping |
| — | `review_item.prompt` | SkillTrace has no per-node recall prompt |
| Review data | — | **none exists** (0 reviews), so there is nothing to seed FSRS parameters with |

### `evidence_record` and the evidence-definition layer

| SkillTrace | Waypoint v0.3 | Fidelity |
|---|---|---|
| ArtifactSpec (81): `artifact_kind`, `required`, **`minimum_count`** (31 × 3, 50 × 1), `expected_location_hint`, `example_filename`, `acceptance_summary` | — | **no home**. Waypoint moves a node to `evidenced` on one human-accepted record (I6). It has no per-node count or kind requirement |
| ValidationGate (81): `authority`, `command`, plus 23 checker scripts | — | **no home** |
| `accepted_by: objective_gate` | — | **conflicts with doctrine**: P2/I6 make acceptance human-only |
| `accepted_by: learner_manual` | `accepted` (human) + `decided_at` | clean in shape. No records exist |
| `supersedes` + `supersede_reason` | `supersedes_id` (+ `chain_root_id`, `is_latest`) | lossy (no reason field). No records exist |
| `artifact_hash`, `location` | `artifact_ref` into a content-hash store | similar in spirit. No records exist |
| AssessmentAttempt (`passed`/`failed`) | — | no home. Feeds the remediation trigger. No records exist |

### Execution and policy (not in the ticket's four tables, listed for completeness)

`blocker` maps closely except `resolution_summary` and `status`. `session_work.minutes` and `blocked` have no home. RemediationAction, `execution/events.yaml` (the audit log, incl. 30 `verify-resource` events), and all `policy/*.yaml` values (track weights, review cadence, remediation threshold, 180-day staleness window, mastery spacing) have no home. All learner-side tables are empty.

## 5. Concepts with no home in Waypoint v0.3

1. **Evidence definitions per node**: artifact kind, **minimum evidence counts** (3 for 31 manual-gated nodes), required/optional, and acceptance summary text.
2. **Gate definitions**: manual vs objective authority, gate commands, and the 23 shipped checkers. Objective auto-acceptance also conflicts with P2/I6.
3. **Spec-pending markers**: a "can't be passed yet" state on 19 nodes, currently prose only.
4. **Remediation edges** and their runtime activation (blocker or 3 failed attempts), plus AssessmentAttempt and RemediationAction.
5. **Mastery**: a second asserted level above passed, with spaced-review eligibility.
6. **Edge `reason`**: the required authored rationale on every edge.
7. **Tracks and weights, domain, tags, effort estimates, session fit**: the inputs to SkillTrace's recommendation scoring.
8. **Resource-level metadata**: shared resource identity, cost / free-tier / certificate / license, the version pins and regeneration keys, and the human `broken` marker with its reason.
9. **Provenance metadata**: `roadmap_anchors` (reference_only), `source_metadata`, `section_provenance`.
10. **Body structure**: the six named slots, which Waypoint's single `description` field doesn't preserve.

## 6. Residual uncertainty

- The mapping is against architecture-schema v0.3 as written. Whether Waypoint should gain fields for any of §5 belongs to OQ15 and later tickets, and isn't judged here.
- "No home" for `description` depends on whether Waypoint renders `description` as Markdown. v0.3 doesn't say.
- The 18 body/registry resource mismatches were found by comparing ID sets. Which side is intended wasn't checked node by node.
- Counts are as of SkillTrace `f8ec443` (2026-09-22). SkillTrace is still under active change (G-FoundationsFill and later).
