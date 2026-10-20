# The cookie recipe, built before the release runbook

The proposal says the recipe is the test of whether the artifact is what it claims to be,
because nothing in a bowl of creamed butter automates. So it was built first, before the
release procedure, and this is what building it measured.

Built in a scratch project (`/tmp/rb-scratch`, `htmlspec init --tools none` at version
`0.0.0`) rather than in this repository, because this repository is not an html-spec project
— it governs itself with `openspec/`. That detail turned out to matter and it has its own
evidence file: `the-location-is-blocked-2026-09-17.md`.

## What was built

`htmlspec/runbooks/chocolate-chip-cookies/tasks.html`, created with the real command:

```
$ htmlspec new document tasks htmlspec/runbooks/chocolate-chip-cookies/tasks.html \
    --subject "Chocolate-chip cookies"
htmlspec/runbooks/chocolate-chip-cookies/tasks.html: created a tasks
```

Nine task units, creaming the butter to the wire rack, each with a `data-field="note"`
paragraph. Five citation links to the recipes it was written from.

## The claim the proposal makes, and whether it holds

> It needed no code change.

**True, verified.** Three commands, on the binary at `HEAD` (`056df05`) with no
modification:

```
$ htmlspec validate htmlspec/runbooks/chocolate-chip-cookies/tasks.html
  no diagnostics: the document is accepted                       exit 0
$ htmlspec validate --strict htmlspec/runbooks/chocolate-chip-cookies/tasks.html
  no diagnostics: the document is accepted                       exit 0
$ htmlspec format --check htmlspec/runbooks/chocolate-chip-cookies/tasks.html
  is canonical                                                   exit 0
```

`validate --json` reports `accepted: true` with `diagnostics: []` — zero, not zero errors
with warnings. The document carries 9 task units and 5 external links.

**A runbook's documents are validated by the existing path, and nothing else was needed.**
They are ordinary documents of an existing kind — a `tasks` document with at least one task
unit, which is what *A tasks document carries the workable items* in `artifact-kinds`
requires — so the validator has no idea it is looking at a runbook and does not need one.

## The citation links survive, and the reason is a deliberate line

Five `<a href="https://…">` links, none reported. The reason is in
`crates/htmlspec-validate/src/rules/safety.rs:37`:

> `href` is deliberately absent for `a`: a hyperlink to an external page is a citation, not
> a dependency, and the document renders fine offline with it. It *is* checked on `link`,
> which loads.

The proposal says somebody drew the exact line a cited recipe needs, and that is right.

## Where the ingredients went, and why that is a finding

**The proposal predicted this gap and building the recipe hit it exactly.** The ingredients
have no home. A runbook has no notion of an input.

What was done, stated as the agent's own choice rather than as anything the project
requires: **the ingredients are a `<table>` in a document section of their own,
`#hs-ingredients`, titled "What the procedure consumes".** Prose and a table — no unit
markup at all, so nothing in the model claims they are anything.

Three options were available and this is why the other two were rejected:

1. **A task unit per ingredient, or one "assemble the ingredients" task.** This is what the
   proposal says the first draft did and calls wrong, and it is wrong: a list of quantities
   is not a unit of work, and `htmlspec status` would then count it as one. Measured: the
   document reports `0 of 9 task(s) done or dropped`. With an ingredients task it would read
   `0 of 10`, and the tenth is not work.
2. **Requirement units.** A requirement is checkable prose with `SHALL` or `MUST`, and "1
   cup of butter" is neither. It would also draw `schema/weak-modal`.
3. **Prose in a section.** Chosen. It is the cheapest to change, which is
   `contract-authority`'s second route under silence, and it commits the design to nothing:
   when inputs get a notion, this section is what gets converted.

The equipment is worse and it is recorded in the document: a stand mixer and two baking
sheets are closer to a machine requirement — which is
`carry-a-runbook-to-another-machine`'s subject — than to an ingredient. **So the input gap
and the machine-requirement gap touch here, in one paragraph of one document**, which is
the reason the proposal says the shape of the answer is not obvious.

The section carries a paragraph saying it is a finding rather than a design, so a reader of
the recipe meets the gap rather than inheriting a convention.

## What the recipe proved that the release procedure could not

Every one of the nine steps is a reading. `task-cream-the-butter` says the colour and the
volume are what you read and that 2 minutes with cold butter is not 2 minutes with warm.
`task-bake` gives 9 to 11 minutes and says the range is the baker's judgment rather than
imprecision. `task-combine` names the one irreversible step.

None of that is a command. A proposal arguing only from the release procedure invites the
reading that a runbook is a shell script with prose around it, and this document is the
counter-example: there is no `scripts/bake` that could replace it.
