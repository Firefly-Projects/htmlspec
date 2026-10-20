# The guidance changed the transcript

**Date:** 2026-09-06
**Tasks:** 5.2, 5.3, 5.4
**Transcripts:** `transcripts/before-answer.md`, `transcripts/after-answer.md`,
`transcripts/control-answer.md`

Three agents were given one task: produce the migration mapping table, including the row for
`specs/<capability-path>/spec.md`. Each was pointed at a contract, a README, and a scratch
OpenSpec project carrying a **nested** capability path — `specs/identity/user-auth/spec.md` —
because nesting is what the original failure destroyed.

The verification is reading. The assertion rule has no mechanical check, and this file is what
stands in its place.

## The before-state reproduced the defect, independently

The first run pointed at `80558b5` in a worktree: no `contract-authority` capability, and no
body carrying the obligation. Its first row:

| Source | Destination | Its reason |
| --- | --- | --- |
| `specs/<capability-path>/spec.md` | `htmlspec/specs/<capability-path>.html` | "…README shows `htmlspec/specs/theme-selection.html`" |

**That is the original failure, arrived at from scratch.** Flattened destination, and a
justification resting on the README. The agent had never seen the earlier draft and reached the
same place by the same route.

Two things make it the same defect rather than a similar one:

- **The cited line disclaims itself.** `README.md:5` reads "**Status:** early design proposal.
  The name, commands, and file layout are working ideas." The tree it read sits at line 81, 76
  lines *below* that sentence.
- **Nothing in that revision's contract states a layout beneath `specs/`.** Checked:
  `grep -rn "specs/" openspec/specs/project-bootstrap/spec.md` at `80558b5` returns nothing.

It also wrote `htmlspec/specs/identity/user-auth.html`, keeping the directory and dropping
`spec.md` — so the flattening is inconsistent with itself, which is the second fault task 1.2
recorded. One invention, two errors, again.

## The after-state cited or recorded silence on every row

The second run read `.claude/commands/htmlspec/new.md` — the installed copy, which carries the
obligation only because PR #21 regenerated it — and worked against the current contract.

Every destination is either cited or recorded as silence. The accepted-specification row is
`htmlspec/specs/<capability-path>/spec.html`: nesting preserved, and the choice attributed
rather than asserted.

**All 20 requirement titles it cited resolve to real requirements.** Checked each against
`grep -rl "### Requirement: <title>" openspec/specs/`; zero missing. Its one verbatim quote —
"The declaration SHALL NOT carry authoring rules or project context" — matches
`project-bootstrap/spec.md:34` exactly.

**Four points recorded as silence, not filled:** the three directory spellings, the four
document file names, `spec.html` as the leaf name, and `.html` as the extension. It took the
preserving route where a shape already existed and stated the `.html` choice as its own, which
is the asymmetry `contract-authority` exists for — a choice attributed to the agent gets
corrected, a choice attributed to the project does not get questioned.

## The strongest single result, and I did not design the test to produce it

It found `openspec/changes/migrate-an-openspec-project/specs/project-migration/spec.md`, which
addresses this migration directly and **agrees with its answer**, and refused to cite it:

> I deliberately did **not** cite [it] … it is unarchived, and *The project's obligations are
> what its accepted specifications state* makes citing it as a current obligation incorrect.
> Archiving it is what would convert those four choices into project rules.

That is requirement 2.4 firing on the one case where citing would have been *convenient and
correct on the merits*. Confirmed the document exists and is unarchived, and that
`contract-authority/spec.md:22` states a change's specification "SHALL become binding when
archival applies it".

## Task 5.3: no invented justification

Neither the after-run nor the control supplied a reason for a rule it could not find. Where the
after-run had no requirement it said so and named the route it took. The failing shape — a rule
stated bare *with reasoning attached*, which reads as a decision already taken — does not appear.

## Task 5.4: the rule did not over-fire

The counter-test matters as much as the test. A rule firing on every sentence is one an agent
learns to skim, which is what the deletion test bounds.

The clearest instance is a claim about a run, carrying **no citation**:

> Verified in a scratch project: `htmlspec validate` walks the change directory and reports
> nothing about a non-HTML file sitting in `evidence/`.

That is a fact about a command's output, not about the contract, and the guidance says it owes
nothing. The agent's own reasoning about ordering and about its four silence decisions is
likewise uncited. Citations land on destination claims and nowhere else.

| | before | after |
| --- | --- | --- |
| requirement citations | 0 | 40 |
| silence recorded | 0 mentions | 8 |
| accepted-spec row | flattened, README-justified | nesting preserved, cited |
| unarchived spec cited as binding | — | refused, with the reason |

## What this does not show

**A third run points at a limit of the test rather than of the guidance.** An agent given the
*current* contract but never told to read the instructions still cited `contract-authority` and
recorded silence — because the capability is now *in* the contract it was told to read
(`transcripts/control-answer.md`). So this cannot separate the contract's contribution from the
guidance's. Distinguishing them needs the contract present and the bodies stale, which is a
state no revision has.

**The transcript is one run of one task.** It is evidence that the guidance changed this
behavior on the case that failed, not that it changes it generally. **The assertion rule still
has no mechanical check**, and this is the fifth consecutive release to say so.
