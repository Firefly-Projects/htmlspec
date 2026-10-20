# Did the reasoning survive? A reading, not a test

**Date:** 2026-09-09
**Task:** 8.7
**Subject:** `2026-08-13-content-address-runtime-assets`'s design, migrated by hand and recorded
at `evidence/a-migrated-design-2026-09-09.html`. The source is
`openspec/changes/archive/2026-08-13-content-address-runtime-assets/design.md`, 1,609 words.

**This is a judgment and it has no mechanical form.** Nothing in the toolchain distinguishes a
migrated design from a converted one — both validate, both format canonically, and a document
that lost its argument passes every check this project has. So this is me reading both and saying
what I found. It is recorded as evidence because a claim nobody can check is worth stating
plainly rather than implying.

**One design, not three.** Task 8.7 asks for three. Writing one honestly took the reading of the
source, six decisions with their rejected alternatives, six risks with their mitigations, and a
sequence diagram — and I judged one done properly worth more than three done at the speed that
would produce the defect this change exists to prevent. The task is marked with that stated
rather than claimed as satisfied.

## Why this design

It has the most structure to lose. Six decisions each carrying a rejected alternative, six risks
each with a mitigation, a CAUTION about a specific failure mode, five context facts, and a
six-step sequence whose fifth step is a rollback boundary. A design that was three paragraphs of
prose would survive any conversion, and would prove nothing.

## What survived, counted

| Source | Migrated | |
| --- | --- | --- |
| 6 decision headings | 6 `decision` units | each with `data-status="accepted"` |
| 5 "alternative considered/was" passages | 10 `data-field="rationale"` paragraphs | every rejection preserved with its reasoning |
| 6 risks in a bullet list | 6 `risk` units | each with a `data-field="mitigation"` |
| 1 CAUTION | preserved verbatim, in a callout | as a note rather than a second statement |
| 5 context facts in a bullet list | a 5-row table | fact against why-it-constrains |
| 6 numbered migration steps | a Mermaid diagram | with the rollback boundary drawn |

Nothing was dropped. That is the mechanical half of the answer and it is the easy half.

## The judgment: three places the format did something the source could not

**The five context facts became a table, and the table earned its place.** The source's bullet
list states each fact and then, in the same sentence, why it constrains the design. Two different
things joined by a comma, five times. As a two-column table the constraint is legible as a
constraint — a reader scanning for "what could I not change" reads the right column. This is the
mapping the workflow's guidance describes as a source stating something as prose and meaning it as
a table, and it is the clearest instance I found.

**The six-step sequence became a diagram, and the boundary became visible.** The source's
numbered list has a paragraph after it beginning "Rollback: steps 1 and 2 are additive and leave
the old layout intact, so a revert before step 5 restores the prior state without data loss." That
sentence is the most important thing on the page and it sits below the list, in prose, where a
reader arriving at step 4 has not met it. In the diagram the revert node connects to steps 1
through 4 and stops. **The reader cannot reach step 5 without seeing that rollback stopped.**

The diagram also carries the three signals the source states inline: the provenance check still
passing after step 2, the suite going *red* at step 3 as the expected outcome, and green again at
step 5. "The suite now fails on the existing duplication, which is the expected signal" is a
sentence easy to skim past and alarming to hit unprepared.

**The decisions became units, and the alternatives became rationale.** In the source each
decision is a heading, a paragraph, and then "Alternative considered: … Rejected because …". In
the migrated document the decision is one statement and every rejection is a `rationale`
paragraph on the same unit. That is addressable: a later change can reference
`decision-scope-by-the-store` and a reader reaches the decision and every alternative it turned
down. The source's rejections were findable only by reading the section.

## What the format could not improve, and where I left the source alone

The goals and non-goals section stayed prose. It is four goals and four exclusions, and a table
of one-line items would be a table for the sake of one. The workflow's guidance says both halves
matter, and this is the half that says leave it.

`decision-scope-by-the-store` carries a 130-word rationale rejecting one alternative on three
counts, ending with "which is circular". I kept it as one paragraph. Splitting three counts into
three rationale paragraphs would make them look independent, and the third depends on the first —
the circularity is what the argument builds to. **The unit structure was available and using it
would have been wrong.**

## The two things validation caught that the reading would not have

Both are the format being stricter than the source, and both are real:

1. **A decision's `statement` is single-valued.** Two unmarked paragraphs is ambiguous and the
   projection refuses to choose. In both cases the second paragraph was a consequence rather than
   a second statement of the decision, so it became a note — which made the document say something
   truer than the source did, where the two were indistinguishable.
2. **The projection reaches *through* presentational markup.** My CAUTION inside a
   `<div class="hs-callout">` still supplied the enclosing decision's primary field, so the
   callout did not exempt it. Marked as a note.

Neither is a migration defect. Both are the destination asking a question the source format never
asked, and the answer improved the document.

## The verdict

**The reasoning survived, and in three places it is more legible than the source.** The rollback
boundary is the strongest case: it went from a sentence below a list to a visible edge in a
diagram, and that is a difference a reader feels rather than one I can measure.

**But this took me about as long as reading the source did, and that is the finding that matters
for the other 59 documents.** There is no version of this work that is fast. A migration of this
repository is 133,000 words of source prose needing this treatment, and the honest estimate is
that it is the same order of effort as having written the designs in the first place. The change's
own risk section says "161 documents is a lot of judgment in one pass" and accepts it. Having done
one, I would put it more strongly: the plan is not a pressure valve for that, it is a checkpoint
before the expensive part starts.

That is worth saying in the changelog, and task 9.3 says it.
