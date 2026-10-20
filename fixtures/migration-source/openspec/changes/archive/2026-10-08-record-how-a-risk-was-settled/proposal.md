## Why

**A risk the project accepts cannot be written as a risk unit without a mitigation it does not
have.** A risk is settled in one of two ways: the change holds it down, or the project bears it as
it stands. The element contract has room for only the first. The schema's `risk` definition
requires `mitigation`, and projection section 4.7 says "A risk missing either one is invalid".

`evidence/the-measurement-2026-09-30.md` measured what that costs:

- **The attribute an author reaches for does nothing.** A risk with a statement and no mitigation
  fails `validate` with `schema/invalid`: "mitigation is a required property". Adding
  `data-response="accepted"` changes neither the diagnostic nor the model, because projection
  section 2.2 ignores an unknown `data-*` attribute.
- **An accepted risk is valid only with a mitigation that denies being one.** This repository's own
  release runbook does exactly that. `runbooks/release/specification.html` line 208 is a mitigation
  paragraph that begins "Not mitigated, and recorded rather than hidden.", and the model holds that
  sentence as the risk's mitigation.
- **The archive's authors make the distinction all the time.** Across the 24 archived designs, 15
  risk entries are marked accepted, 21 mitigated, and 3 not mitigated. Eight of the 12 headed risks
  open with "Accepted". Five risks are accepted and name a partial measure at once, so "accepted"
  does not mean "has no mitigation".
- **The migration's readers met the gap.** The reader of
  `distinguish-a-requirement-from-an-example` found that a risk unit has a mitigation field and no
  value that says whether the risk was accepted. The reader of `pass-the-description-through` kept
  the distinction as prose because it "has no field". The reader of `migrate-an-openspec-project`
  wrote the workaround in as an instruction: "the mitigation field states 'Accepted, unmitigated'
  with its reason."

**Why now.** The project's migration out of OpenSpec is staged and waiting. Without this change,
each accepted risk it carries is left as prose, recorded as an open question, or written with the
workaround above. The measurement also bounds what this change unblocks there. Shape keeps most of
the archive's risks as prose, because 88 of its 100 risk statements have no heading of their own.
The missing value for a risk's response blocks 5 headed risks. The change is worth making anyway:
the workaround sits in a document this project runs at every release, and three of the
migration's readers met the gap independently.

## What Changes

- **A risk declares how it was settled**, in a `data-response` attribute with the value
  `mitigated` or `accepted`. It projects into a `response` field that every risk in the model
  carries.
- **A mitigated risk states its mitigation, as every risk does today.** A `mitigated` risk with no
  `data-field="mitigation"` paragraph is invalid, under a new rule, `schema/missing-mitigation`.
- **An accepted risk needs no mitigation.** It may still carry one to state a partial measure, as
  five archived risks do. It needs no separate reason field. Its statement and its notes say why it
  is borne, which is where every accepted risk in the archive already says it.
- **A risk that declares no response is `mitigated`.** All 83 risk units in this repository's
  tracked HTML and all 11 in the staged migration carry a mitigation, so each keeps its meaning and
  its validity.
- **Two values, not three.** A risk that is partly held down is `mitigated`, and its mitigation
  paragraph says how far the measure reaches. The release runbook's "Partly held down and honestly
  not fully" already does this.
- **The contract moves one minor version**, because it gains an attribute, a field, and a rule.
- **The template and the instructions teach it.** The specification template's example risk
  declares `data-response="mitigated"` and names the alternative. Its sentence "A risk with no
  mitigation is a worry." stops being true, and goes. `new.md` says how a design records a risk's
  settlement.
- **Unchanged:** a risk's statement, references, and notes, every other unit, the change states,
  and the meaning of every document written before this change. A `data-response` on a unit
  other than a risk projects nowhere, as `data-state` on a decision does today.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-schema`: a new requirement, "A risk records how it was settled". It is added rather
  than folded into an existing requirement, because no accepted requirement states a risk's parts
  today: its statement and its mitigation are required only by the schema and by projection section
  4.7. Two requirements were read for a MODIFIED block, and neither needs one:
  - "Semantic units are identified by class" lists the seven class names and nothing about a unit's
    parts. The `risk` class does not change.
  - "The HTML-to-model projection is published as a normative specification" lists the minimum the
    projection specification states. It names task completion state and unit change state, and
    already leaves out decision status and verification outcome, which project the same way. So it
    is not a catalogue of attributes, and projection section 2 is where the new one is listed. A
    MODIFIED block would also make this the third change to a requirement that changes 1 and 5
    already modify.

## Impact

- `crates/htmlspec-schema/schema/document.schema.json`: the `risk` definition gains `response`
  (values `mitigated` and `accepted`, default `mitigated`, required as `change` is) and drops
  `mitigation` from `required`. `schemaVersion` gains the new version.
- `crates/htmlspec-schema/doc/projection.md`: section 2's table gains `data-response`, section
  4.7 is rewritten, section 4.7.1 is added, and section 10.1 names the conditional rule.
  `CHANGELOG.md` records the minor bump, and `src/lib.rs` moves `CONTRACT_VERSION`.
- `crates/htmlspec-parse/src/model.rs` and `src/project.rs`: `Risk` gains `response`, filled
  from the attribute or from a default constant. A test ties the constant to the schema's default,
  as one does for `DEFAULT_CHANGE_STATE`.
- `crates/htmlspec-validate/src/rules/relations.rs`: `schema/missing-mitigation`, beside
  `schema/missing-reason`. `tests/schema_rust_split.rs` lists `mitigated` as a schema-owned literal.
- `crates/htmlspec-format/src/attributes.rs`: `data-response` joins the attributes that lead,
  after `data-outcome`.
- Fixtures:
  - a projection fixture for section 4.7.1, which `crates/htmlspec-parse/tests/mapping.rs` requires;
  - regenerated models for the `named-fields` and `single-field-repeated` projection fixtures;
  - valid and invalid conformance fixtures through `conformance/src/bin/generate_fixtures.rs`, and
    the rule in `known_rules()`.
- `assets/template-specification.html`, its pin in `assets/vendor.lock.json`, and the contract
  version every template declares.
- `crates/htmlspec-assets/instructions/new.md`. Also the phrase "the risks accepted" in `apply.md`,
  `verify.md`, `archive.md`, and `ff.md`, which would now read as a filter on the new value. The
  installed copies are regenerated.
- **Not in this change**, and recorded so it is not assumed to follow:
  - Rendering the response. The stylesheet shows a decision's status and a task's state as a
    heading badge, and a risk's response gets none here. See design decision 9.
  - Editing the release runbook. Its two risks could then state `accepted`, and drop the mitigation
    paragraph that denies being one. That edit is a later release of the runbook, which moves
    `runbooks/release/VERSION`. It is not part of this change.
  - Mapping the archive's "Accepted" and "Mitigated" onto `data-response` during migration. That
    is `migrate-into-the-extended-vocabulary`.
  - A value for a risk that later evidence retired, which the reader of
    `pass-the-description-through` noted. A later change that retires a risk modifies or removes the
    unit.
  - Rejecting a per-unit attribute on the wrong unit. Measured today, `data-state` and
    `data-outcome` on a decision are ignored silently. A rule for that covers all four per-unit
    attributes, and is its own change.
  - artifact-kinds' scenario "A design that marks its decisions", which speaks of "the risks it
    accepted". It is true under either reading, so it is left as it is.
