# Sync: the operation archival tells you to run

## The division, and why it is the same one archival already made

Archival's `unmerged_requirements` (`crates/htmlspec-cli/src/commands.rs:1290`) carries a
comment that is really the design of this change:

> **This compares identifiers and reads nothing for meaning.** Deciding which accepted
> requirement a scenario belongs under is judgment, and `artifact-kinds` reserves it for an
> agent. Observing that `req-x` appears in one set of documents and not in another is a set
> membership test.

Sync inherits that division intact. The program answers *what is absent*. The agent answers
*what to do about it*. The only thing sync adds over archival's version is that it is
available on its own, before archival, as many times as the work needs.

The consequence worth stating plainly: **sync writes no accepted specification.** A reader
expecting `sync` to mean "make the two sides match" will find that it does not, and the
naming inherited from OpenSpec does not help — `openspec sync` is also agent-driven, and its
skill says so in its second line. The requirement text must therefore state the exclusion
rather than leave it to be inferred from what the program happens not to do.

## Where the comparison comes from

There is one correct answer and it is already written. `unmerged_requirements` is the
function archival uses, and `The command surface reports a change's completion` establishes
the precedent for this exact situation: archival's report about incomplete work derives from
the same operation a contributor can run, "so that a contributor who checks a change before
archiving it and the archival that follows cannot disagree about what the change holds."

Sync is the same obligation applied to the other of archival's two findings. Extracting
`unmerged_requirements` so both call it is the whole of the mechanism; a second traversal
that recomputed the comparison would be a second answer to one question, and the day the two
disagree is the day a contributor syncs clean and archival reports unmerged requirements
anyway.

## What sync reports

Three sets, not one. Archival reports only the absent identifiers because that is all a
finding needs to be actionable. Sync is the operation someone runs *while deciding what to
merge*, so it reports:

- **stated** — every requirement identifier the change's specification carries
- **accepted** — those the project's accepted specifications already hold
- **absent** — the difference, which is what archival calls unmerged

The absent set is derivable from the other two, and it is emitted anyway. An agent that has
to compute a set difference to find its worklist will compute it, and a payload that makes
the caller do arithmetic to reach the answer the operation exists to give is a payload that
gets used wrong.

## The case that decides the exit code

A change with absent requirements is not broken. It may be mid-implementation, and a
withdrawn requirement is legitimately absent forever — archival's `Findings` type says
exactly this: "The contract refuses and the work asks."

So sync exits successfully with a non-empty absent set. It is a report, not a gate. The
alternative — non-zero while anything is unmerged — would make sync unusable in the position
it is meant for, which is *during* the merge, and would put a red exit on the normal state of
a change that has not finished.

What sync does refuse on is the same class archival refuses on: a change that does not exist,
a specification that is absent, a specification that does not validate. A comparison against
an invalid contract is a comparison whose result means nothing, and reporting it as if it
meant something is worse than refusing.

## The workflow stage

The installed stages are `explore`, `new`, `continue`, `ff`, `apply`, `verify`, `archive`
(`crates/htmlspec-assets/src/instructions.rs:87`). `sync` becomes an eighth, because
`The workflow instructions are available from the command surface` requires a stage's
instructions and its installed copy to come from one source, and because the OpenSpec skill
set this project mirrors already has one.

The stage's instructions are where the agent-driven half is written down: read both sides,
merge intelligently, never copy a change's specification into an accepted one wholesale,
re-run sync to confirm. That text is guidance for an agent and belongs in the instructions,
not in the requirement — the requirement states what the *program* does and what it must not
do.

## Rejected: sync as part of archival

Archival could run sync's report before its own findings and offer to proceed. It already
effectively does, via `report_findings`. Making that the whole answer was considered and
rejected: it leaves the operation reachable only at the moment of archiving, which is the
constraint this change exists to remove. A contributor who wants the accepted contract
updated three commits before archival still has nowhere to go.

## Rejected: a `--check` flag on sync instead of a separate report

`archive --check` exists, so the symmetry is tempting. But sync *is* the check — it never
writes. A `--check` flag on an operation that has no write mode would be a flag that changes
nothing, which is the kind of surface that makes a reader assume the flagless form writes.
