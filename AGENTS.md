# AGENTS.md — Waypoint

Waypoint is a local-first desktop skill tracker (Tauri v2, React + TypeScript, a Rust domain layer over SQLite). It has two goals of equal weight: the product, and the author's learning. The author knows Python and is watching this app get built as groundwork for learning Rust. They review and merge every change but don't write the code.

So **the reader is learning.** Every diff, comment, PR description and doc is written for someone who knows Python but not Rust or TypeScript. When two designs work equally well, pick the one that is easier to read and explain.

## Read first

- `CONTEXT.md` has the domain glossary. Use its terms exactly. When a term is added or changes, update it in the same PR.
- `docs/design-document.md` covers the product and the reasons behind it. `docs/architecture-schema.md` covers the crates, schema and invariants I1–I7.
- `docs/adr/` records the decisions that constrain code. Follow them, and write a new ADR to change one.
- `docs/agents/issue-tracker.md` explains how issues, wayfinder maps and labels work.

## Workflow

1. Work only from an issue labelled `ready-for-agent`. Claim it with `gh issue edit <n> --add-assignee @me`.
2. Branch from `main` as `<issue#>-<slug>`, e.g. `14-create-node-command`.
3. Commit in the plain imperative mood (`Add create_node command`).
4. Before opening the PR, run every check the repo defines (tests, lints, formatters) and make sure they all pass locally.
5. Open a PR that fills in every section of `.github/pull_request_template.md`.
6. The author reviews and squash-merges. Nothing goes to `main` any other way.

A PR is ready only when every artifact it triggers (the table below) is in the PR.

## Rules

- **One concept per PR.** Keep PRs small enough to read in one sitting. If a change introduces two new Rust concepts, split it into two PRs.
- **Plain construction.** Write explicit code rather than clever code. A macro, trait machinery or generic abstraction needs an ADR that justifies it.
- **Comments say why.** Explain intent and constraints, especially in `waypoint-domain`, and leave the syntax to the concept notes.
- **The author merges.** Only the author approves and merges PRs. When they approve something they didn't fully follow, they file a `learning:question` issue, and it gets answered with a concept note.
- **Invariants live in one place.** Enforce I1–I7 only in `waypoint-domain`. Each invariant has at least one test named after it (`i6_cannot_evidence_without_accepted_record`), so "where is I6 enforced?" can be answered with a search.

## Learning artifacts

| Artifact | Write it when | Template |
|---|---|---|
| ADR, `docs/adr/NNNN-<slug>.md` | A decision constrains future code: a crate boundary, a table shape, a library choice | `docs/templates/adr.md` |
| Concept note, `docs/learning/concepts/<concept>.md` | A concept appears in the code for the first time (`Result` and `?`, a Tauri command, a SQLite transaction, a React effect) | `docs/templates/concept-note.md` |
| Walkthrough, `docs/learning/walkthroughs/NN-<user-action>.md` | Its own issue, the last one of each milestone | `docs/templates/walkthrough.md` |

When a concept already has a note, link to it rather than explaining it again. If your change moves code that a concept note cites, update the note in the same PR. ADRs are never edited after acceptance: a change of mind is a new ADR that supersedes the old one, and the only edit to the old ADR is its `Status:` line.
