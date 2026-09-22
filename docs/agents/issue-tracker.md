# Issue tracker: GitHub

Issues and specs for this repo live as GitHub issues in `earledotpy/waypoint`. Use the `gh` CLI for all operations.

## Editing issue bodies safely

Issue bodies use typographic punctuation (em dashes, arrows, `§`). **Never write a body through PowerShell text capture** (`>`, `Out-File`, `Set-Content`), because it corrupts non-ASCII characters. Instead, write the body to a UTF-8 file (the Write tool, or a Git Bash heredoc) and push it with `gh issue edit <n> --body-file <file>`. Before pushing, check that the file has no U+FFFD characters.

## Wayfinding operations

These conventions are carried over from SkillTrace.

- **Labels.** Exactly one open issue is labelled `wayfinder:map`. Its children carry `wayfinder:grilling` (a decision), `wayfinder:research` (an investigation), `wayfinder:prototype` (a throwaway decision aid) or `wayfinder:task` (work that unblocks a decision). Nothing else uses these labels.
- **The map is an index, not a store.** Its body sections, in order: `## Destination`, `## Notes`, `## Decisions so far`, `## Not yet specified`, `## Out of scope`. Open tickets are not listed; they are the map's open sub-issues.
- **Children are native sub-issues.** A ticket is a sub-issue of the map, and its body opens with `Part of wayfinder map #N.` List them with `gh api repos/earledotpy/waypoint/issues/<map>/sub_issues`.
- **Blocking is body text.** A ticket that can't start yet has `Blocked by: <name> #id, …` on its second line. The **frontier** is the map's open, unassigned children whose named blockers are all closed.
- **The assignee is the claim.** Run `gh issue edit <n> --add-assignee @me` before starting any work.
- **Resolution.** Post one resolution comment with the full answer, close the issue, then add a one-line gist to the map's `## Decisions so far` as `- [<name>](<url>) — <gist>`.
- **Refer by name.** In narration and in the index, refer to a ticket by its title with the link inside it, never by a bare number.
- **Artifacts are decision aids.** Research goes in `docs/research/<name>.md` on a throwaway `research/<name>` branch. Prototypes go in `prototype/` with a THROWAWAY banner.
