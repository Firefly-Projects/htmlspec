# Evidence: dogfooding the draft contract (task 3.4)

**Date:** 2026-08-07
**Contract under test:** `crates/htmlspec-schema/schema/document.schema.json` and `crates/htmlspec-schema/doc/projection.md`, both at draft version `2026-08-07`
**Documents authored:** `fixtures/dogfood/simple-proposal.html` (3.1), `fixtures/dogfood/complex-change.html` (3.2), `fixtures/dogfood/technical-investigation.html` (3.3)
**Tooling used:** none for authoring. `cargo run -p htmlspec-parse --example dogfood_parse` afterwards, to confirm all three parse with zero recoveries.

All three documents parse recovery-free and in `NoQuirks`, so nothing below is a parser problem. Every finding is a contract problem.

## Finding 1: the model cannot hold a document's own prose

Each of the three documents opens with a paragraph saying what it is and why it exists. In an OpenSpec proposal that paragraph *is* the proposal — the "Why" section carries the argument, and the requirements only make it checkable.

The draft projects it nowhere. A `header` is presentational under section 8.4, so the opening paragraph contributes to no field. A consumer reading the model sees requirements and tasks and cannot see what the change is for.

**Severity: high.** This is the failure mode the whole project exists to avoid — meaning that lives in prose the model cannot address.

## Finding 2: positional field projection shifts silently

Draft rules 4.4 and 4.7 read fields off paragraph *position*: a decision's first paragraph is its statement and the rest is rationale, and a risk's first paragraph is its statement while the second is its mitigation.

While authoring `risk-closed-vocabulary` I wanted a sentence of context before the mitigation. Adding it would have made the context the mitigation and pushed the real mitigation out of the model, with no diagnostic, because both paragraphs are non-empty text. A rule whose violation is invisible is exactly what the base instructions call moving an invariant into human memory.

**Severity: high.** The failure is silent, and an agent editing prose is the most likely author of it.

## Finding 3: "every paragraph is a normative statement" is too coarse

Draft rule 4.1 projects every paragraph of a requirement into `statements`, and the validator checks that a statement uses SHALL or MUST.

`req-recovery-free-parse` needed two paragraphs, and both happen to carry a modal. `req-visible-visualization-source` also worked. But the general case does not: a requirement often needs one sentence of context that is not itself an obligation, and under the draft that sentence either fails the modal check or forces the author to bolt a modal onto explanatory prose.

**Severity: medium.** The workaround damages the prose, which is the thing this format is supposed to protect.

## Finding 4: a task has no place for anything but a drop reason

Draft rule 4.5 assigns a task's first paragraph to `reason`. A task that is not dropped therefore cannot carry a paragraph at all without projecting a `reason` that means nothing.

I wanted a note on `task-dogfood-vocabulary` explaining why it comes before the parser work. There was nowhere to put it.

**Severity: medium.** Same shape as finding 3: one field is doing two jobs.

## Finding 5: evidence cannot point at a figure

`evidence-mermaid-gap` in the investigation document exists because of the chart and the diagram above it. A figure has no identifier in the draft, so no unit can reference one. The relationship that motivated the evidence is unrepresentable.

**Severity: medium.**

## Finding 6: a heading is heavy for a task

Under rule 3.1 every unit's title comes from a heading, so a list of four tasks contains four `h3` elements inside `li` elements. It parses cleanly and it reads oddly, and it will produce a document outline full of task titles.

**Severity: low.** Uniformity is worth something, and the alternative — a per-unit title rule — is worth less. Recorded and not fixed.

## Finding 7: group headings are invisible, and that is acceptable

The documents use `h2` headings to group requirements, decisions, tasks. Those headings project nowhere, and the model's arrays are flat, so the relative order of the *groups* is lost while order within each group is preserved.

Nothing in the specs needs group order. Recorded and not fixed.

## Finding 8: `<` must be escaped inside a code example, and that surprised me twice

Writing the requirement excerpt in the investigation document means writing `&lt;section`. Getting it wrong does not produce a broken example, it produces live elements, which the parser reports as a recovery — so the failure is loud, and the safety rule catches what the parser does not.

No contract change. Recorded because it is the most likely thing an agent gets wrong, and because it argues for `init` shipping an escaped example.

## Finding 9: `data-refs` earned its keep, and links did too

Both reference forms were used naturally and for different purposes: `data-refs` for a machine relationship a reader does not need to see, and an inline `<a href="#req-theme-contrast">` inside a decision's rationale, where the reference *is* the prose. Projecting both into `references` while also projecting the anchor into `links` turned out to be right, not redundant.

No change. Recorded as a positive result, because the draft could easily have supported only one form.

## Revisions this evidence produces

Task 3.5 applies these. Findings 6, 7, 8, and 9 produce no change.

| Finding | Revision |
| --- | --- |
| 1 | Add a document-level `summary`, projected from paragraphs in the document `header`. |
| 2 | Replace positional field projection with an explicit `data-field` attribute. More than one unmarked paragraph in a unit whose primary field is single-valued is invalid. |
| 3 | Add `data-field="note"`, which opts a paragraph out of the primary field and into a new `notes` array. |
| 4 | `reason` becomes `data-field="reason"`. An unmarked paragraph in a task is a note. |
| 5 | A figure takes an optional `id` with the `figure-` prefix, and a link may resolve to it. |
