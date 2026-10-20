# The report a contributor received, and why it was obedient

Section 1, recorded before any body was changed. The run is the one that verified
`pass-the-description-through` — the description reached the agent, three good documents were
produced, and the report is what failed.

## The contributor's reaction

> If I was a user I asked to make a tally book app and I got a bunch of nothing back.

Their request had been three sentences describing a tally-mark app with two tabs. What came
back opened on a numbered task order, reported `htmlspec validate: 0 errors, 8 warnings`, and
closed by asking them to adjudicate a collapse threshold of 50 and a reset-per-box behavior
they never mentioned.

## The five bullets that produced it

`crates/htmlspec-assets/instructions/new.md` ends:

```
- Where the change landed, and the three documents it holds.
- What the proposal argues, and what it excludes.
- The decisions the design reached, and any alternative you rejected for a reason worth
  keeping.
- The task order, and why the first task is first.
- Anything you could not settle. An open question in a report is recoverable. One written
  into a design as though it were decided is not.
```

The subject of each, in order: a directory, an argument, decisions, an order, open questions.
Five bullets and not one names the thing the contributor asked about.

This is the finding. The report was not careless — it followed all five faithfully. An
instruction that produces a bad outcome when followed exactly is the same shape of defect as
the two changes before this one, and the repair is the same kind: change the instruction, not
the agent.

## The defect is in all seven bodies

Lines in each `## What to report` section whose subject is an artifact, a document, a task, a
design, a specification, or a warning:

| Body | Artifact-subjected lines |
| --- | --- |
| `apply.md` | 6 |
| `new.md` | 5 |
| `verify.md` | 5 |
| `archive.md` | 4 |
| `ff.md` | 4 |
| `continue.md` | 3 |
| `explore.md` | 2 |

`apply.md` is worse than the body that produced the complaint. `verify.md` asks for a
per-task and per-decision accounting, which is a pointer list by construction. `explore.md`
is closest to correct, because its subject — a question and an answer — is already the
contributor's.

Scoping a repair to `new.md` would fix one instance of a seven-instance defect.

## No body asks for a gloss

```
grep -ril "gloss\|plain words\|what the reader" crates/htmlspec-assets/instructions/
```

Returns nothing. This project's own explainer skill carries the rule — gloss every pointer on
first use, naming in plain words what it points at — and the workflow instructions never
received it.

## The deletion test, applied to the received report

The test: remove every pointer from the claim. If the claim still stands, the pointers
annotate a stated fact and no gloss is owed. If it collapses, the pointers *are* the claim.

**Collapses.** The report's headline list:

```
1. task-test-harness — first because every later correctness claim rests on it...
2. state module → 3. mark grouping → 4. statistics → 5. storage adapter
3. two-tab shell → 7. Tally panel → 8. Stats panel
4. corrections → 10. walk the proposal's sequence end to end
```

Delete the identifiers and nothing survives. `tasks.html` defines ten of them —
`task-test-harness`, `task-state-module`, `task-marks`, `task-stats`, `task-persistence`,
`task-tabs`, `task-tally-panel`, `task-stats-panel`, `task-corrections`, `task-walkthrough` —
and the report names exactly one. The numbering also restarts twice, so items 6 through 10 are
referenced and never shown. Reading that list requires opening the artifact and
cross-referencing ten identifiers.

**Does not collapse, and was already correct.** The same report states:

> All eight are `review/stale-reference` — each prerequisite link points at a task this change
> marks `added`, which is unavoidable in a new change where every task is new.

The warning count is glossed: the kind is named and the reason given. This half of the report
satisfies the obligation before it was written down, which is why the rule must arrive with
its bounding test rather than as a blanket requirement to gloss every identifier.

## The prose that was already written and never surfaced

`proposal.html` contains a section the 0.3.0 guidance requires:

> Add a box named Robin. It appears showing 0 and no marks. Add boxes named Sparrow and Crow.
> Three boxes are now visible, each at 0. Click Robin seven times. It reads 7, drawn as one
> struck group of five followed by two upright marks. Sparrow and Crow still read 0.

That is the answer to "what am I getting". It names Robin, Sparrow, and Crow, so a contributor
knows from it whether the agent understood them. It sat in a file while the report discussed a
threshold of 50.

The waste is the useful part of this finding. A report omitting material the agent never
produced is incomplete. A report omitting material sitting in a file the agent just wrote is
misdirected, and misdirection is a property of the instruction.

## No accepted requirement governs a report

`project-bootstrap` governs how instruction files are installed. `artifact-kinds` governs what
each document must contain. A report is neither — not written to a file, not validated, not
archived, not read by the next agent — so no requirement reaches it.

That is why seven independently-written report sections were never reconciled, and why a green
suite says nothing about this defect. It is also the argument for a new capability rather than
an extension of an existing one.

## What is not a defect

The workflow produced no application code. That is `new` behaving as designed, and no report
rewrite changes it. If a contributor expected running code, the honest repair is a report that
names the boundary — three documents exist, the app does not yet, `apply` builds it — in one
line. The received report never said that plainly, which is the register defect rather than a
separate one.
