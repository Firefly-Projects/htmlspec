## Why

**An author who marks an edit with HTML's own change markup silently writes a wrong contract.**
HTML has `del` for text an edit removed and `ins` for text it added. The projection does not know
them. It treats each as an ordinary presentational element and keeps its text, and `validate`
accepts the result. `evidence/the-measurement-2026-09-30.md` measured it
against HEAD:

- A requirement whose statement reads `… and <del>nothing else</del><ins><code>store</code></ins>
  SHALL name them.` projects the statement "… and nothing elsestore SHALL name them." The
  withdrawn words are in the normative statement, run together with the new one.
- `htmlspec validate` on that project exits 0. Its only diagnostic on the specification is a
  `review/stale-reference` warning that would be there without the markup. `--strict` and `--json`
  say the same.
- A paragraph wrapped in `<del>` becomes a second statement of the requirement. A requirement
  wrapped whole in `<del>` projects as an `unchanged` requirement. A verification whose only link
  to a requirement is deleted still verifies it, and the same document with the words actually
  removed fails validation.

The projection is doing what its specification says. `crates/htmlspec-schema/doc/projection.md`
section 8.2 makes an element with no reserved class presentational, and its text contributes to
the field around it. What is missing is a rule saying a document may not use change markup where
it would change what a unit states.

**Why now.** The project's migration out of OpenSpec is staged and waiting, and the 116 HTML
documents it would promote become the project's own. The measurement found no change markup in
any of them, or in any of the 122 HTML files the repository tracks, so a rule added now breaks
nothing. Without it, this gap would be recorded as an open question in the migrated documents or
worked around there, and a contributor who reached for `<del>` in one of them would get no
warning.

## What Changes

- **Change markup inside a unit is an error.** Validation reports every `ins` and every `del`
  element that sits inside a semantic unit, with a rule identifier of its own, naming the unit and
  the element and quoting the element's text. It is an error and not a warning, because the model
  is wrong rather than incomplete.
- **Change markup around a unit is the same error.** A `del` wrapping a whole unit leaves the unit
  projecting as current. A withdrawn unit already has its own statement, `data-change="removed"`.
- **`ins` is covered as well as `del`.** An inserted word does the model no harm today. It is
  rejected because a unit that marks one side of an edit states half of a diff, and because the
  change planned to give the two elements meaning gives it to both together.
- **Every unit, every kind, every declared version.** The rule reaches tasks, decisions, and every
  other unit in all four kinds. A document cannot escape it by declaring `0.1.0` or `0.2.0`.
- **The document contract moves to `0.3.0`.** A document that validates today can stop validating.
  The versioning table puts that under a major bump, and its `0.x` clause permits it in a minor
  one. `0.1.0` and `0.2.0` stay accepted.
- **Unchanged:** the projection and every projected model. Change markup outside every unit is
  not reported: prose there projects into no unit's field, and a proposal may show a wording change
  with it. `s` and `strike` are not change markup and are not reported.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-validation`: a new requirement, "Change markup in or around a semantic unit is
  rejected". It is added rather than folded into "Structural conformance is enforced", because
  that requirement lists element-contract violations in one sentence and has no room for this
  rule's reason or its boundary. The change that follows this one,
  `mark-the-words-a-modification-changed`, is planned to modify this requirement to allow the
  markup where it gains meaning. Folding the rule into the structural requirement would make that
  change restate every structural rule to relax one.
  "The contract rules apply to every kind" is also left alone. It lists what makes a document
  open, and this rule is about what a document states. The new requirement states its own reach
  over every kind.

`document-schema` is **not** modified. Its requirement "The HTML-to-model projection is published
as a normative specification" asks the projection specification to state "which elements and
attributes create a semantic unit, and which are ignored" and "how presentational markup that
carries no semantic unit is discarded". Section 8.2 already states both, and the projection's
treatment of `del` follows it exactly. This change alters what validates, not what projects. The
projection specification gains one clause in section 10.1, which lists the validator's rules
that the projection does not decide, and that clause adds no numbered rule.

## Impact

- `crates/htmlspec-validate/src/rules/change_markup.rs`, new: the rule `schema/change-markup`. It
  reads the parse tree the projection carries, as `executable.rs` does, and recognizes a unit by
  the projection's own reserved-class test.
- `crates/htmlspec-validate/src/rules/mod.rs`: the rule joins `ALL`, and
  `every_namespace_has_a_live_rule` gains its source entry.
- `conformance/tests/conformance.rs`: `known_rules()` gains the rule.
- `conformance/src/bin/generate_fixtures.rs`: two fixtures, `invalid-schema-change-markup` and
  `valid-change-markup-outside-a-unit`. The generator also gains a table of the four hand-authored
  fixtures that its stale-fixture sweep deletes today, so running it deletes nothing.
- `crates/htmlspec-schema/doc/projection.md`: section 10.1 names the rule, and the header moves to
  `0.3.0`.
- The contract version, `0.3.0`:
  - `CONTRACT_VERSION` in `crates/htmlspec-schema/src/lib.rs`;
  - `schemaVersion` and `$id` in `crates/htmlspec-schema/schema/document.schema.json`;
  - an entry in `crates/htmlspec-schema/CHANGELOG.md`;
  - the four `assets/template-*.html` and their pins in `assets/vendor.lock.json`;
  - `README.md`;
  - the version list in `crates/htmlspec-validate/tests/schema_rust_split.rs`.
- **Not in this change**, and recorded so it is not assumed to follow:
  - Giving `ins` and `del` meaning. That is planned as `mark-the-words-a-modification-changed`.
  - `s` and `strike`, which draw a line through text without claiming an edit.
  - Change markup in the document's summary, a figure's caption, or a link's text outside every
    unit. The measurement shows each of these keeps the deleted words, and none of them states a
    contract. The design records this as the contributor's call.
  - The workflow instructions. The diagnostic states the fix, and the change that gives the markup
    meaning is the one that tells an agent how to use it.
