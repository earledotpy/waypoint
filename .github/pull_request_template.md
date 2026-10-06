Closes #

## What changed

<!-- Plain first, then precise: two or three sentences a reader with no programming vocabulary can follow, then the technical version if it's needed. Define each technical word in plain words or link it to docs/learning/glossary.md. -->

## Why this shape

<!-- Name the obvious alternative and say why it lost. -->

## Invariants touched

<!-- Which of I1–I7 this change enforces or relies on, with the test that covers each. Or "none". -->

## Read this one file

<!-- If the reviewer reads only one file, which one, and what to look for in it. -->

## Concepts

<!-- New concept notes written in this PR, and existing ones the diff relies on (link each). Or "none". -->

## Questions you might have

<!-- One to three questions about this PR's code, in plain words, mixing "what does this do?" and "why is it this way?". Each links to where its answer lives (a concept note, a glossary entry, a file and function, a section of this description) without giving the answer. -->

Tick any question you can't answer, or reword one in a comment. The agent files each ticked question as a `learning:question` issue and answers it in this PR, before merge.

- [ ] <!-- question --> (answer in: <!-- link -->)

## Check it yourself

<!-- Steps the author can run by hand to see the change working. Write them as PowerShell commands, run from the author's main checkout (not an agent's worktree).
An agent's worktree may still have this PR's branch checked out, and git lets only one folder hold a branch at a time, so a plain `git switch <branch>` can fail. Start with:
    git fetch origin
    git switch --detach origin/<branch>
and end with `git switch main`.
Say what output to expect, including the pass count, and what a wrong result (such as "0 passed") would mean. -->

