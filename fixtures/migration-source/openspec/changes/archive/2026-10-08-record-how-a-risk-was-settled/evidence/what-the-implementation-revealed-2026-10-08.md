# What the implementation revealed

**Date:** 2026-10-08
**Task:** 7.5

## The default fit every risk counted

Task 1.2 found no risk the default of design decision 1 did not fit. All 87 risks in tracked HTML and
all 11 in the staged migration carry a mitigation, so each one projects `mitigated` with the
mitigation it had, and validates as before. Task 7.2 confirmed it on the built change: every one of
the 98 projects `mitigated`, and `validate --json` is byte-identical on HEAD and on the change for
`runbooks/`, `fixtures/dogfood/`, and the staged tree.

**One risk fits the default's validity and not its author's meaning.** The risk the migration drafted
in `2026-09-06-distinguish-a-requirement-from-an-example/design.html` opens "**Accepted, and out of
scope by design.**" and carries a mitigation paragraph that calls itself a partial repair. Under the
default it projects `mitigated`, which is what every risk projected implicitly before this change, so
nothing it says is lost. Declaring it `accepted`, with its mitigation kept as the partial measure, is
the mapping `migrate-into-the-extended-vocabulary` makes. The release runbook's two risks are the
same kind, and stay as they are until a runbook release, as the design says.

## Corrections to the design

Four facts in `design.md` had moved since it was written, and none changed a decision. Each is
corrected in place:

- **The contract version.** The Migration Plan said the contract was `0.2.0` at HEAD. It was `0.4.0`
  when this change was built, and the two changes below this one in the set move it to `0.5.0` and
  `0.6.0`, so this change moves it from `0.6.0` to `0.7.0`.
- **The fixture inputs carrying the template's risk.** Context and decision 8 said 64. There are 68:
  the changes earlier in this set added four conformance fixtures, each built on the base.
- **The tracked risks.** The Migration Plan said 83. There are 87, the same four fixtures' risks
  added.
- **Where document-schema names `risk`.** Context listed the class vocabulary, the prefix table, and
  the reference rule. The scenario listing the units a `renamed-from` paragraph may not sit in names
  it too. No requirement states a risk's parts, which is what the proposal's argument for an ADDED
  requirement rests on, and that still holds.

## Built beside the design

- **The message reads `accepted` from the schema too.** Decision 5 has the rule find `mitigated`
  through `status_named`. The message also names `data-response="accepted"` as the remedy for a
  risk the change bears, and writing that value into rule code would restate a schema value, which
  `tests/schema_rust_split.rs` exists to prevent even where its list does not name the literal. So
  the rule finds `accepted` with the stem `accept` as well, runs only when both are found, and
  `every_conditional_rule_finds_its_state` checks both stems. `accepted` stays out of
  `SCHEMA_OWNED_LITERALS`, as the design says, because it is also a decision status.
- **The generator freezes the whole risk, not the two edits.** Decision 8 has `generate_fixtures.rs`
  rewrite the base's risk back to its committed form. `with_the_frozen_risk` replaces the template's
  risk element, from its opening tag to its closing one, with the committed text, so a later edit to
  the template's risk is frozen out too rather than passing into every committed input.
- **A fifth projection test.** `section_4_7_1_a_response_is_trimmed_and_an_unrecognized_one_survives`
  holds the response to section 2.3, and holds `partial` to surviving projection so the schema can
  name the permitted values: `"partial" is not one of "mitigated" or "accepted"`.

## What the tasks expected and what happened

- **The projection tests compile on HEAD.** Task 2.1 expected the first three to fail because `Risk`
  has no `response`, which would have been a compile error. They read the risk as the model's JSON,
  because the requirement is about what the model carries, and a field the struct held and
  serialization skipped would fail them too. So on HEAD they fail on their assertions, `left: Null`.
- **The generator no longer deletes the hand-authored fixtures.** Task 2.6 asked for the four to be
  restored after regenerating. The generator now keeps them through its own `HAND_AUTHORED` table,
  and nothing needed restoring.
- **`mapping_models --write` rewrites keys it did not change.** It writes each regenerated file with
  sorted keys, so it also moved the hand-placed `kind` in `named-fields` and
  `single-field-repeated`. The two files were given the one `response` line per risk by hand, and
  `mapping_models` reports them current. The `risk-response` model was written by hand at task 2.4,
  and the generator found it current on its first run.

## The rule-identifier move, seen

Variant A, a risk with a statement and nothing else, failed on HEAD with `schema/invalid`, "mitigation
is a required property", at `/risks/0`. On the change it fails with `schema/missing-mitigation` alone,
at the risk, and the message says why a risk with no attribute is held to a mitigation and names both
remedies. The changelog entry names the move beside the one `0.2.0` made.

## What the stack's assembly has to reconcile

This branch starts from `main`, where the contract is `0.4.0`, and the two changes below it are built
in parallel. Each of these places is edited by more than one of the three, and each needs the
versions in order when the stack is assembled:

- `0.7.0` follows `0.4.0` directly in the `schemaVersion` enumeration and in `schema_rust_split.rs`'s
  literals, and `0.5.0` and `0.6.0` go between;
- the contract changelog's `0.7.0` entry says "from `0.6.0`", and goes above the `0.6.0` and `0.5.0`
  entries;
- the projection specification's header and the README list the versions still accepted, and both
  gain `0.5.0` and `0.6.0`;
- the four templates, their pins in `assets/vendor.lock.json`, and the 12 `schema/rename-*`
  conformance inputs, which declare the current version, move once more, and the specification
  template's pin also covers this change's risk;
- `rules/relations.rs` gains a rule from each of the three changes, each called from `validate`
  right after `dropped_tasks_state_a_reason`, and each tasks list extends
  `every_conditional_rule_finds_its_state`; `known_rules()` and `SCHEMA_OWNED_LITERALS` gain an
  entry from each;
- projection section 10.1 lists the validator's conditional requirements, and each change adds one
  to the same sentence;
- the `named-fields` projection fixture gains this change's `response` line on its risk, and
  `record-a-partial-verification` extends its verification;
- the conformance generator's table gains fixtures from each change, in neighbouring places.

`data-response` is the only new attribute in `LEADING`, and the generator's risk helpers touch only
the risk, so neither should conflict.
