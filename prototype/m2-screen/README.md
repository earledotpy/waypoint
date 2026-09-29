# THROWAWAY PROTOTYPE — milestone 2's screen

For the wayfinder ticket [What milestone 2's screen shows](https://github.com/earledotpy/waypoint/issues/42).
It lives only on the `prototype/42-m2-screen` branch and is never merged.

Question: what does milestone 2's unstyled screen show, and how do you add and
remove a Prerequisite or a Recommendation on it? Is state history visible?

## Run it

```
npm install
npm run dev
```

Open http://localhost:1420/?variant=A (or B, C). Flip variants with the
arrows in the black bar at the bottom, or the ← → keys. `npm run tauri dev`
works too. The data is fake, in memory, and shared across variants, so a graph
you build in A is still there in B. Reload to reset.

## Variants

- **A — Edges on each node, no history.** Each node row lists its
  prerequisites and recommendations and has its own add form.
- **B — Sentence form, edge list, global log.** One form reads
  "[A] is a [prerequisite] of [B]", then a list of every edge, then every
  state change, newest first.
- **C — Pick a node.** A node list, and a detail panel for the one you
  picked: what it needs, what it unlocks, an add form, and its own history.

## Try

1. Walkthrough 02's path: make "Write a for loop" a prerequisite of
   "Write a function". Watch "Write a function" turn Locked.
2. Refusals: make "Write a function" a prerequisite of "Write a for loop"
   (a cycle), add the same edge twice (a duplicate), or pick one node for both
   sides (a self-edge).
3. Remove the edge and watch the node go back to Available.
4. Add a recommendation and see that nothing locks.
