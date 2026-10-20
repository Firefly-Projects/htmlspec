## Why

A contributor described a tally-book app, got three good documents, and read a report that
told them none of it. Their reaction:

> If I was a user I asked to make a tally book app and I got a bunch of nothing back.

The report they received led with a task order, closed on a validation warning count, and
asked them to adjudicate a collapse threshold of 50 and a reset-per-box behavior they had
never mentioned. The documents themselves were good — the proposal states the need as an
absence, names who wants it, and describes counting three bird species on a walk.

Two faults, and they compound.

**Every report obligation names an artifact.** `new.md` closes with five bullets:

> - Where the change landed, and the three documents it holds.
> - What the proposal argues, and what it excludes.
> - The decisions the design reached, and any alternative you rejected for a reason worth
>   keeping.
> - The task order, and why the first task is first.
> - Anything you could not settle.

The subject of each one is a directory, an argument, decisions, an order, and open
questions. Not one names the thing the contributor asked about. An agent that follows all
five produces exactly the report received, so that report is obedient rather than careless.

**This is not `new`'s defect.** All seven bodies carry a `## What to report` section, and
every one is artifact-subjected. Counting lines in each that name an artifact, a document, a
task, a design, a specification, or a warning:

| Body | Artifact-subjected lines |
| --- | --- |
| `apply.md` | 6 |
| `new.md` | 5 |
| `verify.md` | 5 |
| `archive.md` | 4 |
| `ff.md` | 4 |
| `continue.md` | 3 |
| `explore.md` | 2 |

`apply.md` is worse than `new.md`. `verify.md` asks for a per-task and per-decision
accounting, which is a pointer list by construction.

**Pointers arrive unglossed, and a report is where that costs most.** The received report
listed ten task identifiers as a numbered sequence. One (`task-test-harness`) was named;
the rest appeared as bare numerals, and the numbering restarted so that items 6 through 10
were referenced but never shown. Reading it required opening `tasks.html` and
cross-referencing ten identifiers.

Applying the deletion test that this project's own explainer standard already uses — remove
every pointer and see whether the sentence still states its claim — the list collapses to
nothing. The pointers *are* the claim, so each one owes a plain-words gloss. No body asks
for one: `grep -ril "gloss\|plain words"` across all seven instruction files returns
nothing.

**Why a report rather than a document.** A report is the only part of a workflow a
contributor is guaranteed to read. The documents serve the next agent and archival, and a
contributor opens them by choice. So the report carries the whole burden of answering one
question: did this understand me? Five bullets about artifacts cannot answer it, however
faithfully an agent follows them.

## What Changes

- **A report speaks in the contributor's terms.** The first thing a report says is stated in
  the vocabulary of what the contributor asked for, rather than the vocabulary of the
  artifacts that hold it. Each workflow keeps its own subject — `explore` reports on a
  question, `verify` on whether the change is sound, `archive` on what the project now
  guarantees — and the register is shared.

- **Every pointer is glossed on first use.** A task identifier, a rule number, a diagnostic
  code, or a warning count names in plain words what it points at, in the same sentence, or
  it does not appear in a report. The deletion test decides the case: where removing the
  pointers leaves the claim standing, they annotate a stated fact and no gloss is owed.

- **A report surfaces prose the workflow already wrote.** The tally proposal's *What working
  looks like* section — "Click Robin seven times. It reads 7, drawn as one struck group of
  five followed by two upright marks" — is the answer to "what am I getting", and it sat
  unread in a file while the report discussed a threshold. Where an artifact already states
  the thing in the contributor's terms, the report quotes or compresses it rather than
  describing the file.

- **A decision the agent can settle is settled, not delegated.** A collapse threshold of 50
  is arbitrary and cheap to change, so an agent picks one and says so. Scope the contributor
  did not ask for is offered for removal rather than presented as an open question. What
  remains for the contributor is what genuinely needs them.

- **The obligations live in the contract, once.** A new `workflow-reporting` capability
  states them for every workflow. A rule stated seven times in seven bodies drifts seven
  ways, and the existing divergence across those seven report sections is the evidence.

## Non-Goals

- **Making a workflow do more work.** `new` writes three documents and no code, by design,
  and that is correct. If a contributor expected running code, the repair is a report that
  names the boundary — "three documents exist, the app does not yet, `apply` builds it" — not
  a workflow that quietly starts building.

- **Removing the artifact facts.** Where the change landed and what the checks reported are
  real and a contributor sometimes needs them. They move from the opening to the end and
  from the subject to the annotation. Nothing is deleted.

- **Rewriting any body's procedure.** This change reaches the `## What to report` section of
  each body and the contract that governs it. The guidance on how to write a proposal, a
  design, or a specification is untouched.

- **A fixed report template.** Seven workflows have seven subjects, and a shape that suits
  `new` would distort `explore`, which deliberately produces no artifact at all. The
  obligation is on register and on glossing, not on a form to fill in.

- **Importing the explainer's full language standard.** The deletion test and the
  pointer-gloss rule transfer because a report has a reader with limited time. Sentence caps,
  the modal ladder, and the rest of that standard are for a different kind of document and
  are not adopted here.

- **Re-opening `pass-the-description-through` or `accept-a-described-change`.** Both are
  archived and both work. This defect was visible only once a description reached an agent
  and the agent produced a full report, which is why it surfaces now rather than earlier.
