# Waypoint

Waypoint tracks skill development through evidence of doing. A skill advances only when something you produced backs it up and you explicitly accept it. This glossary fixes the words for that domain. It holds meanings only: tables, fields and code live in `docs/architecture-schema.md`.

## Graph

**Skill graph**:
The set of skill nodes and the edges between them.
_Avoid_: tree, roadmap

**Skill node**:
One demonstrable capability in the skill graph. It advances only through accepted evidence. "Node" is fine as the short form, and a retired node is still a node.
_Avoid_: skill (that's your ability, not the record), card, topic, task

**Prerequisite**:
A node that must be evidenced before a dependent node becomes available. Written as "A is a prerequisite of B".
_Avoid_: dependency, hard edge

**Recommendation**:
A suggested earlier node that never gates anything. B stays available even if its recommended node isn't evidenced.
_Avoid_: soft prerequisite, soft edge

**Node state**:
Where a node stands. It is always exactly one of Locked, Available, In progress or Evidenced, and only a human changes it, except through prerequisite arithmetic.
_Avoid_: status, progress, level

**Locked**:
At least one of the node's prerequisites isn't evidenced.
_Avoid_: blocked

**Available**:
Every prerequisite is evidenced (or there are none), and work hasn't started.
_Avoid_: unlocked, open

**In progress**:
Work has started and there's no accepted evidence yet, or the node was reopened.
_Avoid_: started, active

**Evidenced**:
A human accepted evidence for the node. It's permanent unless a human reopens it.
_Avoid_: mastered, done, complete, passed, stale

**Prerequisite arithmetic**:
The only automatic state change: Locked ↔ Available, driven purely by prerequisites being evidenced, added or removed. It is always logged.
_Avoid_: auto-unlock

**Retired**:
A node permanently taken out of the active graph without being deleted. It is frozen in its last state, keeps its history, and never counts as anyone's prerequisite. Retirement is final: a replacement is a new node. Retired is a mark on the node, not a fifth state.
_Avoid_: deleted, archived

**Curriculum**:
An importable set of nodes and edges, written in a plain-text file.
_Avoid_: course, syllabus

**External ID**:
A node's stable identity in a curriculum. A re-import matches nodes on this, never on the title.
_Avoid_: slug, key

## Evidence

**Evidence record**:
One human-submitted claim that an artifact demonstrates a node. A node can have many.
_Avoid_: proof, submission (as a noun)

**Artifact**:
The thing you produced that an evidence record points at: a file copied into Waypoint's own store, or a link. Every evidence record has exactly one artifact, and a bare note is saved as a text artifact. The record's description is not the artifact.
_Avoid_: attachment, upload, file (for links)

**Submit**:
To create an evidence record. It starts out Pending.
_Avoid_: upload, log

**Decision**:
The one-time human Accept or Reject of a pending evidence record. It's final: a rejected record stays rejected.
_Avoid_: review (that's a retention word), grading, approval

**Pending**:
Submitted and not yet decided. Only a pending record can be edited.
_Avoid_: submitted (every record is submitted), draft, undecided

**Accepted**:
A human decided the evidence demonstrates the node.
_Avoid_: approved, verified, passed

**Rejected**:
A human decided the evidence doesn't demonstrate the node. A retry is a superseding record, never an edit.
_Avoid_: failed, declined

**Supersede**:
To correct a decided evidence record by adding a new record that replaces it. The old record is kept unchanged. A pending record is edited in place instead.
_Avoid_: edit, update, replace, version

**Supersession chain**:
One piece of evidence and all its corrections. Each new piece of evidence starts its own chain.
_Avoid_: history, thread, revision chain

**Chain root**:
The first record in a supersession chain.
_Avoid_: original, parent

**Latest record**:
The one record in a chain that nothing has superseded yet.
_Avoid_: head, current version

**Current accepted evidence**:
A node's latest records that are Accepted, one per chain at most. If a chain's latest record is rejected, that chain contributes nothing, even when an earlier record in it was accepted.
_Avoid_: active evidence, best evidence

## Work

**Session**:
One timed block of work. It can cover several nodes.
_Avoid_: study session, log entry

**Session work**:
The part of a session spent on one node, with its notes. Adding a node to a session moves an Available node to In progress.
_Avoid_: session entry, time log

**Blocker**:
An obstacle a human recorded on one node, open until the human resolves it. It never changes node state and has nothing to do with prerequisites: a node waiting on prerequisites is Locked, not blocked.
_Avoid_: issue, impediment, blocked (as a node state)

**Reopen**:
An explicit human move of an Evidenced node back to In progress, with a stated reason. Its evidence records are untouched, and it returns to Evidenced only by accepting new or superseding evidence.
_Avoid_: un-evidence, demote, revoke

**Set aside**:
An explicit human move of an In progress node back to Available. All its sessions and evidence are kept.
_Avoid_: abandon, cancel, stop

**Resource**:
Learning material you consume for a node. You produce an artifact; you consume a resource.
_Avoid_: link, reference, material

**Verified resource**:
A resource a human has confirmed still works and is still right, with the date they confirmed it. Nothing automatic ever verifies a resource.
_Avoid_: checked, valid

**Daily Brief**:
The day's starting view: due reviews, an open blocker if there is one, and one recommended next action with its driver.
_Avoid_: dashboard, home

**Driver**:
The concrete reason shown with a recommendation or a state change: which review, which blocker, which prerequisite.
_Avoid_: rationale, explanation

## Retention

**Review item**:
A node's retention schedule. There's one per node, created when the node is first evidenced and kept through reopens.
_Avoid_: card, flashcard, deck

**Due**:
A review item whose scheduled time has passed.
_Avoid_: overdue, stale

**Review**:
A scheduled retrieval attempt on an evidenced node. You attempt its prompt, then see the reveal. Every review is recorded.
_Avoid_: quiz, test, check

**Prompt**:
The one human-written do-or-explain task for a node's reviews. It isn't trivia. A node may have none, and then its review falls back to re-reading the evidence.
_Avoid_: question, card front

**Attempt**:
Your optional written answer to the prompt, made before the reveal.
_Avoid_: answer, response

**Reveal**:
The node's current accepted evidence, shown after the attempt as the answer key.
_Avoid_: answer, card back

**Grading rubric**:
What Again, Hard, Good and Easy mean when rating a review of a skill.
_Avoid_: score, grade scale

**Suspended review item**:
A review item kept out of the queue because its node is reopened or retired. It follows from the node's state, and nobody sets it by hand.
_Avoid_: paused, suspended review

**Reset**:
An explicit human restart of a review item's schedule from scratch. It never touches node state, and it's recorded like a review.
_Avoid_: forget (except when comparing to Anki's Forget), demote

**Workload policy**:
The rules that keep reviews manageable: a daily cap, and spreading a backlog out after an absence instead of dumping it all at once.
_Avoid_: throttling

**Daily cap**:
The most reviews Waypoint puts in front of you in one day.
_Avoid_: limit, quota

## Advisory

**Advisory annotation**:
AI-generated commentary attached to an evidence record, a blocker or a session. It can draft or flag but never decides anything, and it never influences node state.
_Avoid_: AI feedback, verdict, AI review
