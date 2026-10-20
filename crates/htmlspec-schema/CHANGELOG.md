# HTMLSpec contract changelog

The version of the document contract — the JSON Schema in `schema/`, the
normative projection specification in `doc/`, and the conformance fixtures that
pin the mapping. All three move together under one version, because a consumer
that reads only one of them has read part of the contract.

This file is part of the published contract. `htmlspec_schema::schema_text::CHANGELOG`
embeds it, so a consumer can read the version history without a checkout.

## Versioning policy

The version is a semantic version, not a date. A date orders releases and says
nothing about whether a document written against an earlier release still
validates, which is the only question a version needs to answer.

| Bump | Meaning |
| --- | --- |
| Major | A unit, field, or projection rule is withdrawn or reshaped. An existing document can stop validating. |
| Minor | A unit, field, or projection rule is added, and warnings and review findings can be added. No document a previous version of the major accepted gains an error. |
| Patch | Wording changes and defects are fixed. No document a previous version of the major accepted gains an error. |

From `1.0.0` this is document-schema's requirement on the contract's version, and this
table describes it. Within a major version, a document that one version accepts is
accepted by every later version of that major: no later minor or patch reports an error
on it that the earlier version did not. The one exception is a defect, where an earlier
version accepted what a requirement stated at that version refused, and the release's
entry below names it. Warnings are outside the promise, because an error is what decides
whether a document is valid, and minor releases of this project have added warnings on
documents already accepted.

While the major version was `0`, the vocabulary was provisional and any release could
break a document, as `0.3.0` and `0.4.0` did. That was deliberate: groups 4 through 9 of
the foundation change still revised the contract, and revising it there cost a schema
edit rather than a migration. `1.0.0` is the release that commits the vocabulary.

A document declares one version, and the validator matches it exactly against
the versions the schema enumerates. A minor or patch bump adds its version to that
enumeration and keeps every earlier one, because a document declaring the version
before it is an existing document, and an exact match against the newest version
alone would reject them all. A major bump is where versions can leave the set, and
`1.0.0` withdrew none: `0.1.0` to `0.7.0` stay accepted beside `1.x`. This was a single
pinned version until `0.2.0`, when a second version first existed.

Each entry below records the date its version landed. That is where the calendar
lives, so a version stays orderable in time without a date inside the version.

## 1.0.0 — 2026-10-20

The major release that commits the vocabulary. **Nothing in it changes**: `1.0.0` is the
`0.7.0` vocabulary, and no unit, field, value, projection rule, or diagnostic is added,
withdrawn, or reshaped. What changes is the promise.

- **The versioning rule becomes a requirement.** document-schema states it, and the policy
  above describes it: within a major from `1` on, no later version reports an error on a
  document an earlier one accepted, except for a defect its entry names. Warnings are outside
  the promise.
- **Every earlier version stays accepted.** The enumeration is `0.1.0` to `0.7.0`, then
  `1.0.0`. A major bump may withdraw versions, and this one withdraws none, because the
  vocabulary did not change and withdrawing them would invalidate every document written
  before it. A document declaring `0.7.0` validates under `1.0.0` with exactly the
  diagnostics it had, because no rule reads the declared version.
- **A new document declares `1.0.0`**, and so do the four templates, the projection
  specification, and the schema's `$id`.
- **The conformance fixtures keep the versions they declare.** The base still declares
  `0.1.0`, the fixtures about fields `0.7.0` added still declare `0.7.0`, and one fixture
  declares `1.0.0`.
- **An older toolchain rejects a document declaring `1.0.0`**, because the version is outside
  its enumeration. Declaring `0.7.0` instead keeps the document readable there, with the same
  meaning.

## 0.7.0 — 2026-10-08

A minor bump from `0.6.0`: a risk can state how the change settled it. Every document that
validated against `0.6.0` still validates, with the same meaning, and gains one field in its
model for each risk it holds.

- **`data-response` on a risk**, with the value `mitigated` or `accepted` (projection
  specification, section 2 and section 4.7.1). A `mitigated` risk is one the change holds down,
  and an `accepted` risk is one the change bears as it stands. A risk that declares no response
  is `mitigated`, because every risk written before this version had to state a mitigation. On
  any other unit the attribute projects nowhere, as `data-state` does on a unit other than a task.
- **`response` on a risk**, which every risk in the model carries, filled from the attribute or
  from the default. The schema requires it, as it requires `change`, so a consumer never applies
  the default itself. A value outside the two is `schema/invalid`, which names the permitted
  values: how far a mitigation reaches is stated in its paragraph, not in a third value.
- **`mitigation` is required only of a mitigated risk.** The schema no longer requires it, and
  section 4.7 no longer calls a risk without one invalid. An accepted risk may carry one, which
  then states the part of the risk that something does hold down.
- **`schema/missing-mitigation`, an error**, for a risk whose response is `mitigated`, declared
  or by default, with no `data-field="mitigation"` paragraph. It is a validator rule beside
  `schema/missing-reason` and `schema/missing-successor`, because the schema keeps to a subset
  with no conditional composition. Section 10.1 names it. Its message says that a risk declaring
  no response is `mitigated`, and names both remedies: a mitigation paragraph, or
  `data-response="accepted"`.
- **A document that was already invalid reports a different rule.** A risk with no mitigation
  failed with `schema/invalid`, "mitigation is a required property". It now fails with
  `schema/missing-mitigation`, or passes if it declares `accepted`. The rule that every risk
  states a mitigation no longer exists, so the violation is not quite the same one, and this is
  the same kind of move `0.2.0` made when a `renamed-from` paragraph outside a requirement
  stopped being a section 4.0 fault and got a rule of its own.
- **What this changes in existing documents.** When this version landed, no HTML file the
  repository tracked and none of the 116 its staged migration would promote carried
  `data-response`. All 87 risks in the tracked files and all 11 in the staged ones carry a
  mitigation, so each projects `mitigated` and validates as before.
- **An older toolchain reads an accepted risk as mitigated.** It ignores the attribute, so an
  accepted risk with no mitigation fails there with `schema/invalid`, and one that keeps a
  mitigation passes as a mitigated risk, with every sentence its author wrote.
- **Every earlier version stays accepted**, and a document declaring one gets this projection
  and this rule, because the vocabulary accepted is the newest version's.

## 0.6.0 — 2026-10-08

A minor bump from `0.5.0`. A value and a field are added, and the one new rule fires only on
the new value, so every document that validated under `0.5.0` still validates.

- **A verification outcome, `partial`.** The check ran against the requirement or scenario
  the verification names, reached part of it, and found that part to hold. A check whose
  covered part did not hold is still `failed`, and a check of a different condition, subject,
  or trigger is not a partial check of the scenario at all. `outcome` now carries a
  description saying so.
- **`unchecked` on a verification** (section 4.8): one entry per paragraph marked
  `data-field="unchecked"`, in document order, each naming a part the check did not reach.
  Repeatable, because a check can miss more than one clause, and the projection keeps only
  the first of a repeated single-valued paragraph. Absent when empty, so the model of a
  document with no such paragraph is what it was. Accepted on every outcome, so a check that
  failed or was blocked can say what it never reached. An unmarked paragraph in a
  verification is still a note.
- **`schema/missing-unchecked`**: a `partial` verification with no `unchecked` paragraph is
  an error, reported at the verification, as a `dropped` task with no reason is
  `schema/missing-reason`. It is a validator rule (section 10.1), because this schema keeps
  to a subset with no conditional composition.
- **`partial` counts wherever a verification counts**, with no code reading it by name. It
  clears `review/unverified-requirement`, as every outcome does. It draws
  `review/stale-verification` rather than `review/stale-reference` when its target moves,
  because the check ran. In a run's record it is a recorded outcome, so it clears
  `review/unrecorded-requirement`.
- **What this changes in existing documents: nothing.** Before this version no HTML file the
  repository tracked, and none of the 116 its staged migration would promote, carried
  `data-outcome="partial"` or `data-field="unchecked"`.
- **`0.1.0` to `0.5.0` stay accepted**, and a document declaring one may carry `partial` and
  `unchecked`, because the vocabulary accepted is the newest version's.

## 0.5.0 — 2026-10-08

A minor bump: a `done` task can state a deviation. Every document that validated
against `0.4.0` still validates, because a `deviation` paragraph named no field of
any unit (section 4.0), so no valid document carried one.

- **`deviation` on a task**, projected from one paragraph marked
  `data-field="deviation"` (projection specification, section 4.5): what was done,
  where that departs from the task's words. The task stays `done`, because the work
  happened, and its words stay as written. A `done` task with no deviation reads as
  done as its words say. Before this version an author had two ways to write the
  difference, and both misreported it: as a note, which no field and no count reads,
  or as a `dropped` task, which counts finished work as abandoned.
- **A second deviation in one task is invalid** (section 4.5.1). The projection does
  not choose between two accounts of what was done. It keeps the first, so a reader
  of a projection that was not validated still reads the task as done differently.
- **`schema/deviation-not-done`, an error**, for a deviation on a task whose state
  is not `done`. A `todo` or `in-progress` task has finished nothing that could
  differ from its words, and a `dropped` task's work did not happen and its reason
  says why. The validator checks it, as it checks a dropped task's reason, because
  this schema keeps to a subset with no conditional composition. Section 10.1 names
  it among the validator's rules.
- **A deviation outside a task** names no field of its unit, and stays fault 4.0.
- **`0.1.0` to `0.4.0` stay accepted**, and a document declaring one may state a
  deviation, because the vocabulary accepted is the newest version's.

## 0.4.0 — 2026-10-07

A minor bump under the `0.x` clause, although `del` stops being presentational, which
the table above calls a reshaped projection rule. The constructs that can stop
validating are a figure whose caption sits wholly in a `del`, which then has no caption
(section 7.2), and a figure whose diagram source sits wholly in a `del`, or begins with
one around its first keyword, which then reports `viz/diagram-empty` or
`viz/diagram-unrecognized`. The clause below the table says that while the major is `0`,
any release can break a document, and `1.0.0` is not this release's decision.

- **Projection rule 8.6: HTML's change markup has one meaning.** `ins` stays
  presentational, because inserted text is current text. The content of a `del`
  supplies no field of anything that encloses it (8.6.1): a statement, a scenario
  entry, a title, a note, a summary, a caption, and a figure's source hold only the
  current text, and a link inside a `del` projects no reference and still projects
  into `links`. A unit or a figure inside a `del` still projects, from its own content.
- **`deletions` on a requirement and on a scenario** (8.6.2): the text of each `del`
  that belongs to the unit and that no other `del` encloses, in document order.
  Absent when empty, so the model of a document with no `del` is what it was. A `del`
  in any other unit, or outside every unit, is recorded nowhere.
- **`schema/change-markup` now permits the markup in one place:** inside a requirement
  or a scenario marked `modified`, in a change's specification, active or archived. It
  stays an error everywhere else, under the same identifier, and its message says why:
  in an accepted specification, a change's other documents, a runbook's document, a
  change's evidence, a document outside any project, a unit marked otherwise, a
  decision, evidence, risk, verification, or task unit, a unit's heading or
  `renamed-from` paragraph, and around or as a unit. Where a document sits is derived
  from its path, and a conformance fixture sits nowhere, so the permitted case is
  pinned by `conformance/tests/change_markup.rs` rather than by a fixture.
- **What this changes in existing documents.** Before this version, no HTML file the
  repository tracked held a `del` but the two conformance fixtures that pin
  `schema/change-markup`, and none of the 116 its staged migration would promote held
  an `ins` or a `del`. One fixture's statement now projects as its current text, and
  neither fixture's verdict changes.
- **`0.1.0`, `0.2.0`, and `0.3.0` stay accepted**, and a document declaring one gets
  this projection and this permission, because the vocabulary accepted is the newest
  version's.

## 0.3.0 — 2026-10-02

A minor bump under the `0.x` clause, although a document can stop validating: one
that carries an `ins` or a `del` element inside a semantic unit, around one, or as
one. The
table above puts that under a major bump, and the clause below it says that while
the major is `0`, any release can break a document. A major bump from `0.x` is
`1.0.0`, which commits the vocabulary, and that is not this release's decision.

- **`schema/change-markup`, an error.** `del` marks text an edit removed and `ins`
  text it added, and the projection gives neither a meaning. Section 8.2 makes an
  element with no reserved class presentational, so a deleted phrase became part of
  a requirement's statement, a deleted paragraph became a second statement, a
  deleted link still counted as a reference, and a unit wrapped whole in `del`
  projected as one the document still states. Validation accepted all four. The
  rule reports each element in or around a unit, or carrying a unit's class itself,
  at the unit, quoting the element's text.
- **The fix.** Write the text as it now reads, without the markup. A unit whose
  words a change edits is marked `data-change="modified"`, and a unit the change
  adds needs nothing more. For a unit wrapped whole, or a change element that is
  itself a unit, the unit goes on an element that is not change markup and is
  marked `removed` or `added`.
- **What it does not reach.** Change markup outside every unit is prose that
  projects into no unit's field, and a proposal may show a wording change with it.
  `s` and `strike` draw a line through text without claiming an edit, and stay
  presentational.
- **The projection is unchanged.** Every document that still validates projects the
  model it did before. Section 10.1 names the rule among the validator's.
- **`0.1.0` and `0.2.0` stay accepted.** Withdrawing them would reject every
  document that declares one, to catch markup none of them carries. Before this
  version, no HTML file the repository tracked and none of the 116 its staged
  migration would promote held an `ins` or a `del`. The only ones now are the two
  conformance fixtures that pin the rule. The rule reaches a document whatever
  version it declares, because a rule a document can switch off protects nothing.

## 0.2.0 — 2026-09-26

A minor bump: a requirement can state the heading it replaced. Every document
that validated against `0.1.0` still validates, and one declaring `0.1.0` is still
accepted.

- **`renamedFrom` on a requirement**, projected from one paragraph marked
  `data-field="renamed-from"` (projection specification, section 4.1.1). The
  requirement's heading is its current name and the field is the heading it
  replaced. That is OpenSpec's `FROM:`/`TO:` rename stated on the unit. The
  paragraph is never a statement.
- **The rename is separate from the change state.** The `unchanged` description
  now says that a requirement whose heading changed and nothing else stays
  `unchanged`, with a `renamed-from` paragraph. A requirement renamed and rewritten
  is `modified` with one.
- **A heading name** (section 4.1.2): the projected title without the unit's own
  identifier at its end. A rename is compared by it, because the template shows
  each identifier inside its heading and the title carries that text.
- **`schemaVersion` accepts `0.1.0` and `0.2.0`.** It was a `const`, so a bump to
  `0.2.0` alone would have invalidated every document already written, which is
  what this table says a minor bump never does.
- **Six validator rules bound a rename**, as errors: a `renamed-from` paragraph
  outside a requirement, in an `added` or `removed` requirement, repeated, empty,
  or equal to the heading name. Section 4.0 no longer reports a `renamed-from`
  paragraph outside a requirement as a field the unit lacks, because the first of
  those rules reports it.

## 0.1.0 — 2026-08-07

The initial contract. Seven semantic units — requirement, scenario, decision,
task, evidence, risk, and verification — with identifier prefixes, `data-change`
lifecycle states, and cross-reference semantics.

Authored across tasks 2.1 and 2.2 of the `html-spec` foundation change, then
revised by task 3.5 from the findings the group 3 dogfood exercise recorded at
`openspec/changes/html-spec/evidence/vocabulary-dogfood.md`. Five findings
changed the contract before anything hardened around it:

- A document-level `summary`, projected from paragraphs in the document `header`.
  Without it the argument a change rests on lived in markup the model could not
  address.
- An explicit `data-field` attribute replacing positional field projection. Under
  the draft, one inserted sentence of context pushed a risk's real mitigation out
  of the model with no diagnostic.
- A `notes` array on every unit, so explanatory prose stays addressable without
  becoming normative.
- `reason` on a task projects from `data-field="reason"`. An unmarked paragraph in
  a task is a note, so a task that is not dropped can carry prose.
- An optional `figure-` identifier on a figure, so a unit can point a reader at
  the chart or diagram it rests on.

This version was published as the date `2026-08-07` before the versioning policy
above existed. It is renamed to `0.1.0` here, with no change to the projected
model. No document outside this repository declared the date form.
