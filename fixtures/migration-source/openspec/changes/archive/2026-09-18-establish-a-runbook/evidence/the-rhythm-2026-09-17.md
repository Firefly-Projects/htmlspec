# A change whose output is a procedure, against this workflow's rhythm

The proposal names this open and says the result is worth recording either way:

> **A change that produces no deliverable has not been tested.** The
> proposal→design→tasks→specification rhythm assumes the work builds something. A change whose
> output is only a procedure is a shape that rhythm has not met.

**The premise turned out to be wrong, and that is the first result.** This change *did* build
something: 62 lines in `project.rs`, 527 in `commands.rs`, 12 in `main.rs`, and eleven tests.
The runbook is the artifact the change is *about*, but the change that establishes a place for
runbooks is an ordinary code change. So the rhythm was never tested against a change with no
deliverable — a genuinely deliverable-free change would be one that *edits* an existing runbook,
which is the shape the sixth release will have.

That said, the rhythm was tested against something the proposal did not name, and it strained in
one specific place. Recorded below because it is the useful half.

## Where the rhythm fit

**`proposal.md` → `design.md` fit unusually well, because the proposal had already done the
measurement.** It arrived carrying the cookie recipe's validation results, the `status --change`
correction, and the five archived section numbers. The design's job was the four questions the
proposal explicitly left open — layout, constant naming, what `list --runbooks` prints, and how
validation happens — and each had an answer that reading the code produced. Nothing in the design
is a preference.

**`design.md` → `tasks.md` fit.** Nine numbered sections, and the ordering constraint was real
rather than imposed: section 1 builds the recipe *before* section 5 touches code, because the
recipe is what establishes that no code change is needed for validation. A tasks document that
had put the code first would have made section 1 a confirmation rather than a test.

**The `## N. Release` section is where the change eats its own output**, and this is the closest
thing to the observation the proposal names as its success criterion. Section 9's four steps are
the runbook's four steps, and running them from the runbook rather than from the archive is what
found both hazards live. Next change's release section is where the proposal's real test lands:
four hand-written items replaced by one that says to run the runbook.

## Where it strained: the specification stage, and it strained twice

**First, the change's specification and the runbook's specification are two documents of the same
kind with completely different subjects, and the rhythm has one word for both.**

- `openspec/changes/establish-a-runbook/specs/runbook-authority/spec.md` states what the *project*
  now guarantees about runbooks. It is a delta, it merges into the accepted specifications, and it
  becomes contract at archival.
- `runbooks/release/specification.html` states what the *release runbook* requires of a run. It is
  not a delta, it merges into nothing, and `runbook-authority` had to state explicitly that it is
  not an obligation of the project.

Writing both in one change, both called a specification, is where the confusion is available. The
delta now carries a paragraph closing it — a requirement in a runbook's specification states what
that runbook requires of a run, not what the project requires of itself — and that paragraph exists
because writing the two side by side is what showed it was needed.

**Second, `data-change` had no correct value and the rhythm supplied the wrong one.** A change's
specification marks its units `added`, because that is what the change is doing. A runbook's
canonical specification states hazards measured two releases ago, so `added` is false and the
validator said so — seven warnings, five of them correct and pointing at one attribute. Full
measurement in `the-canonical-documents-change-state-2026-09-17.md`.

This is the rhythm's assumption showing: every stage of it assumes the document belongs to a change
in progress. A document that belongs to a destination rather than to a stage inherits an attribute
that means nothing for it.

## What would have made this easier, and why it was not done

A workflow. `htmlspec new document` cannot know it is writing a runbook's document, so an author
writing one by hand meets the `data-change` problem with nothing to guide them. A workflow would
know, and would write `unchanged`.

**The proposal requires this change to be provable without one**, and it was: every artifact here
was produced with `new document`, `validate`, `format`, and `list`. The cost is exactly the finding
above, and it is now stated in the contract rather than left for the next author to rediscover —
which is the same repair this whole change is about, applied to itself.

## The honest summary

The rhythm fit. The place it strained was not "there is no deliverable" but "a specification of a
destination is not a specification of a change, and one word covers both". That is a smaller
finding than the proposal expected and it is a real one, and it produced a contract paragraph
rather than a note.
