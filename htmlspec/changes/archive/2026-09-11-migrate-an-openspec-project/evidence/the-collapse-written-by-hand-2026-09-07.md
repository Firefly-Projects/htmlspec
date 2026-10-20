# The delta collapse for `2026-08-10-html-spec`, written by hand

**Date:** 2026-09-07
**Task:** 1.5
**Method:** read all five delta files of `openspec/changes/archive/2026-08-10-html-spec/specs/`
and wrote what their single `specification.html` should contain, before designing anything. This
is the hardest mapping in the set and the one with no rule; the point of doing it by hand first
is to find out whether the constraint is expressible at all.

## What the source holds

Five delta files, one per capability the project then had. Every one is `## ADDED Requirements` —
this was the founding change, so nothing was modified or removed.

| Delta | Requirements | Scenarios | Carries a `## Purpose` |
| --- | --- | --- | --- |
| `deterministic-formatting` | 7 | 19 | yes |
| `document-schema` | 12 | 34 | yes |
| `document-template` | 8 | 23 | yes |
| `document-validation` | 12 | 38 | yes |
| `toolchain-packaging` | 8 | 24 | yes |
| **total** | **47** | **138** | 5 |

47 requirements and 138 scenarios into one document. No later change comes close — the next
largest is 5 deltas at `2026-08-14-separate-artifact-kinds` with 4.

## The constraint

`artifact-kinds` — "Archival produces the specification and then updates the accepted contract"
(`openspec/specs/artifact-kinds/spec.md:256`): "A change directory SHALL hold at most one
specification artifact." So one file, not five. That is what forces the collapse.

`artifact-kinds` — "A specification states the settled contract as built" (`spec.md:193`): it
carries requirement, scenario, evidence, and verification units, and **SHALL NOT** carry task
units. It states what the change established rather than what it intended.

`document-schema` — "Every semantic unit has a stable unique identifier" (`spec.md:25`): unique
**within the document**, prefixed `req-` / `scenario-`, lowercase alphanumeric and hyphens.

## Finding 1: the five `## Purpose` blocks are the real problem, and it is not a presentation one

The design frames the difficulty as arrangement — "Deciding how one document presents five
capabilities' worth of added requirements — grouped by capability, in one flat sequence, in the
order the change made them". Working it by hand, arrangement turned out to be the easy part.
Grouping by capability is obviously right: 47 requirements in a flat sequence is unreadable, and
the capability boundary is the only grouping the source actually expresses.

The hard part is that **five capabilities' Purpose statements have nowhere to go, and they are
not requirements.** Each is a paragraph stating what a capability is for. In the source they are
the top of each capability's file. In the destination there is one document, so there is one
place a Purpose-like statement can sit — the masthead — and five statements cannot sit in it.

Worse, these five Purpose statements **are the accepted capabilities' current Purposes**. All
five survive verbatim in `openspec/specs/*/spec.md` today. So they are not incidental to the
change; they are the definitions of five of the project's nine capabilities, and the collapsed
specification is the only artifact in the migrated project that would carry the version of them
this change established.

Three options, worked:

| Option | Result |
| --- | --- |
| drop them, keep the masthead summary for the change | Five capability definitions vanish. The accepted specifications carry them, so nothing is *lost* from the project — but the archived change no longer states what it established, and `artifact-kinds`' "The contract read without the change" scenario asks that a reader determine the change's obligations from its specification alone. |
| concatenate all five into the masthead | A five-paragraph masthead that reads as one summary of nothing. The masthead's job is to say what this document is. |
| **keep each as a section-introducing paragraph above its capability's requirements** | Each Purpose sits where its requirements sit. The masthead says what the change established. Nothing is dropped and nothing is merged. |

**Taken: the third.** It follows from the grouping rather than being an extra decision — once the
requirements are grouped by capability, each group needs a heading, and the Purpose is the text
that belongs under it.

This is the finding the hand-write produced: the constraint is expressible, but **not by a rule
about requirements.** A rule covering only requirement and scenario units would leave five
paragraphs unaccounted for, and an agent following it would silently drop them. The requirement
in section 2 must state that a delta's non-requirement content is accounted for too.

## Finding 2: identifier collisions across the five, and one real one

Uniqueness is per-document, and the collapse puts 47 requirements in one document that were
previously in five. Deriving `req-<slugified-heading>` from each:

**One genuine collision.** `deterministic-formatting` states *"Formatting can be checked without
writing"* and `toolchain-packaging` states *"Formatting can check without writing"*. Both slug to
something near `req-formatting-can-check-without-writing`; a naive slug of the first gives
`req-formatting-can-be-checked-without-writing`, so they differ by two words — but they are two
distinct obligations on the same subject, one about the formatter's behavior and one about the
command surface exposing it. Both survive in the accepted specifications today
(`deterministic-formatting:55`-ish and `toolchain-packaging:55`).

This is exactly the case the identifier requirement says to report rather than suffix: "Two
distinct obligations distinguished only by a trailing digit read as versions of one." And it is
reachable *within a single change's collapse*, not only across the project — which the design
does not say. The collapse is where collisions are most likely, because it is the only operation
that puts requirements from different capabilities into one identifier namespace.

**The resolution is not a suffix and not a stop.** Both headings are distinct English; a
faithful slug of each is already unique. So the derivation must slug the **full heading** rather
than a shortened form of it, and a rule that truncates or normalizes aggressively manufactures
the collision. The plan records both derivations side by side so a reader can see they are
distinct.

**Twelve near-collisions that are not collisions.** Nine requirements begin "Formatting…", and
`document-schema` and `document-validation` both state requirements about cross-references
(*"Cross-references address units by identifier"*, *"Cross-references must resolve"*) and about
the schema. Full-heading slugs separate all of them. Recorded because a derivation rule tested
only against one capability's headings would look adequate.

## Finding 3: the change's own identity has to be stated, and the source does not state it

Five deltas each say what a capability is for. **None says what the change did.** The change's
identity lives in `proposal.md` and `design.md`, which are separate documents with separate
destinations.

But `artifact-kinds`' "The contract read without the change" scenario requires that a reader
consulting only the specification can determine the obligations the change established. For a
change that established the entire project across five capabilities, a specification opening
straight into `deterministic-formatting`'s requirements fails that: the reader does not know they
are reading the founding change.

So the masthead summary is **written, not carried.** It is derived from the proposal and the
design by an agent that read them. This is a place where the migration must compose text that
exists nowhere in the source — and it is the one place in the whole collapse where that is true.

## What the document should contain

```
specification.html
  kind: specification
  title:   HTMLSpec, the founding contract
  summary: (written) This change established HTMLSpec: the element contract for a
           specification's semantic units, the validation rules a conforming document
           satisfies, the canonical serialization that keeps its diffs readable, the page
           a reviewer opens, and the command surface that checks all of it. 47
           requirements across five capabilities, every one added.

  § deterministic-formatting        (Purpose paragraph, carried verbatim)
      req-formatting-is-idempotent                          data-change="added"
      req-formatting-preserves-meaning                       …
      req-serialization-is-canonical
      req-formatting-introduces-no-diff-churn
      req-generated-content-is-ordered-deterministically
      req-formatting-can-be-checked-without-writing      ← see Finding 2
      req-files-use-a-stable-text-encoding-and-line-ending
      (19 scenarios, scenario-* under their requirements)

  § document-schema                 (Purpose paragraph, carried verbatim)
      12 requirements, 34 scenarios

  § document-template               (Purpose paragraph, carried verbatim)
      8 requirements, 23 scenarios

  § document-validation             (Purpose paragraph, carried verbatim)
      12 requirements, 38 scenarios

  § toolchain-packaging             (Purpose paragraph, carried verbatim)
      req-a-single-command-exposes-validate-format-and-init   ← see below
      req-exit-codes-distinguish-outcomes
      req-output-is-available-in-human-and-machine-forms
      req-formatting-can-check-without-writing            ← see Finding 2
      req-the-toolchain-builds-reproducibly-from-pinned-inputs
      req-a-development-environment-is-provided
      req-project-checks-are-runnable-as-one-command
      req-the-program-is-distributable-as-a-standalone-artifact
      (24 scenarios)

  § evidence                        8 units, one per carried evidence file
      evidence-browser-verification, evidence-diff-locality, evidence-parse-errors,
      evidence-schema-generation-decision, evidence-self-conformance,
      evidence-typify-spike, evidence-vega-lite-schema, evidence-vocabulary-dogfood

  § verification                    (see Finding 5)
```

Every requirement carries `data-change="added"`, because all five deltas are `## ADDED
Requirements`. No task units — `artifact-kinds:193` forbids them, and this change's `tasks.md`
has its own destination.

## Finding 4: one requirement in this specification was later renamed, and that is visible from here

`req-a-single-command-exposes-validate-format-and-init` is the requirement that
`2026-08-17-project-bootstrap-cli` renames and `2026-08-30-finish-the-archival` renames again,
recorded in the two `## RENAMED Requirements` blocks. Its final heading in the accepted
specifications is *"The command surface names its operations as OpenSpec names them"*
(`toolchain-packaging:190`).

So this specification must carry the **original** heading and its own derived identifier, because
`artifact-kinds:193` says a specification "describe[s] what the change established rather than
what it intended" and what *this* change established was the original. The rename is a later
change's business.

That means the migrated project holds **three different `req-` identifiers for one continuous
obligation**, one per archived specification that touched it, plus a fourth in the accepted
specifications. Cross-references between changes resolve to whichever the citing change knew. The
design's "Identifiers are derived once and reused everywhere" reads as though one obligation gets
one identifier project-wide; the collapse shows that is not achievable for a renamed
requirement, and it should not be — each archived specification states its own change's contract.
The derivation table must therefore be keyed on **(change, source heading)**, not on heading
alone.

## Finding 5: verification units have no source, and the honest answer is to omit them

A `specification` carries requirement, scenario, evidence, **and verification** units. The five
deltas carry no verification content — OpenSpec's delta format has no such unit. The change's
`tasks.md` records what was checked, and its 8 evidence files record measurements.

Manufacturing 47 verification units from a tasks document is inventing a record. Omitting them
is honest and valid: `artifact-kinds:193` says a specification carries those four kinds, not that
every one must be present, and `document-validation` has an `UNVERIFIED_REQUIREMENT` diagnostic
which is a warning rather than an error.

So the collapsed specification carries evidence units — those have real sources, the 8 carried
files — and no verification units. **The plan must state that**, because a reader comparing a
migrated specification against a natively-archived one will notice the absence, and "the source
format had no verification unit" is the reason.

## Verdict: the constraint is expressible, and it is not the one the design states

One specification, requirements grouped by capability, each group introduced by its carried
Purpose, change state on every unit, identifiers slugged from full headings and recorded per
(change, heading). All of that is statable.

What the hand-write changes about the design's account:

1. **Arrangement is not the hard part.** Grouping by capability is forced by readability and by
   the source's only expressed structure. The design's "there is no rule to write here" is right
   about arrangement and wrong about what needs one.
2. **Non-requirement delta content needs a stated home.** Five Purpose paragraphs would be
   silently dropped by any rule written about requirements alone.
3. **A collapse can collide identifiers within one change**, and the resolution is to slug the
   full heading rather than to suffix or stop.
4. **The derivation table is keyed on (change, source heading)**, not heading alone, because a
   renamed requirement legitimately holds different identifiers in different archived
   specifications.
5. **Verification units are omitted, with the reason recorded**, because the source format has
   none and manufacturing them invents a record.
6. **The masthead summary is written rather than carried** — the one text in the collapse that
   exists nowhere in the source.
