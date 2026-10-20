## Why

**A modified requirement says that it changed, and nothing in it says which words.**
`evidence/the-measurement-2026-09-30.md` shows the plainest case. The archived change
`2026-08-31-sync-a-change-into-the-accepted-contract` modified toolchain-packaging's "The command
surface names its operations as OpenSpec names them", a requirement of 10 statement paragraphs and
18 scenarios. Its whole edit was `` `sync`, ``, added to a list in the third paragraph. The staged
migration marks the whole unit `modified`, and its paragraph with `sync` in it looks like the other
nine. The word is recorded in git, and in one sentence of the specification's summary.

HTML already has elements for this. `ins` marks text an edit added, `del` marks text it removed, and
every browser draws both without scripts. Today the projection reads them as presentational
markup. An inserted word comes out right. A deleted word does not: a real edit from this
repository, marked with both, projects as "its active changes, and its archive, and its runbooks",
which is neither the text before the edit nor the text after it, and `validate` exits 0.
`reject-change-markup-inside-a-unit` turns that into an error inside every unit. This change gives
the two elements their meaning in the one place where they have one.

**Authors already mark changed words by hand, and the marks go wrong.** Five of the 19 staged
changes that modify a requirement narrate in their summary what moved inside it: "Its only change …
is `sync` added", "added its last two paragraphs and its last four scenarios", "added four
paragraphs and five scenarios", "removed the clause", "is kept, and bounded to units".
`2026-09-18-establish-a-runbook` bolded the words it added. The merge copied the bold into the
accepted project-bootstrap specification, where it still is, and the word the same edit deleted
was never marked (evidence, section 4).

**Why now.** The project's migration out of OpenSpec is staged and waiting. It holds 34 modified
requirements in 19 changes, and every unit it marks `modified` sits in a change's specification.
Without this change, which words moved stays in summary prose, or is marked with bold that a merge
carries into the accepted contract as emphasis. The gap would be recorded as an open question in
the migrated documents or worked around there.

This change follows `reject-change-markup-inside-a-unit`, whose rule it relaxes. It also depends on
`resolve-a-reference-across-its-change`, whose `Placement` type is how validation knows that a
document is a change's specification.

## What Changes

- **A deleted word leaves every field.** The content of a `del` element contributes to no field of
  the model, wherever the element sits: no statement, scenario entry, title, note, summary, or
  caption, and no reference from a link inside it. The link still counts as one of the document's
  links. The text of an `ins` element stays in the field around it. So the normative text is
  always the current text.
- **A requirement and a scenario record their deleted words.** Each gains `deletions`, a list of
  the text of each `del` element that belongs to it, in document order. A deletion inside a
  scenario is the scenario's. Inserted text is not recorded twice.
- **The markup is permitted in one place.** `ins` and `del` may sit inside a requirement or a
  scenario marked `modified`, in a change's specification, active or archived. The unit that
  decides is the nearest one enclosing the markup, so a `modified` scenario in a `modified`
  requirement may carry it, and an `unchanged` one may not. Marking is permitted, not required.
- **Everywhere else it stays an error, under the same rule.** `schema/change-markup` still rejects
  the markup in an accepted specification, a proposal, a design, a tasks document, a runbook, a
  change's evidence, and a document outside any project. It also rejects it in a unit marked
  `added`, `removed`, or `unchanged`, in a decision, evidence, risk, verification, or task unit,
  in a unit's heading or `renamed-from` paragraph, and around a unit. The diagnostic says which of
  these applies.
- **The merge writes the current text.** Applying a change's contract unwraps each `ins` and drops
  each `del` with its content. The accepted specification carries no change markup, and
  validation rejects any that a merge leaves there. The archived change keeps its marks.
- **The comparison needs nothing new.** A comparison of a modified requirement with the accepted
  one, made on projected text, already sees the current text. A merge that kept a deleted word
  differs, and one that wrote the current text does not.
- **The page shows both marks by shape and says them aloud.** A new revision of the screen
  stylesheet draws `ins` with a double underline, so an inserted word does not look like a link,
  keeps `del` struck through, and gives both a start and an end marker that a screen reader
  announces and the page does not draw.
- **The instructions say how to mark and how to merge.** `archive.md` says how to mark the words of
  a modified unit when the specification is written. `sync.md` and `archive.md` say that the merge
  writes the current text.
- **The document contract moves one minor version**, expected to be `0.4.0` after
  `reject-change-markup-inside-a-unit`'s `0.3.0`.
- **Unchanged:** the model of every document without a `del` element, which is every document the
  repository tracks and every staged one. The rule identifier `schema/change-markup`. Markup in
  prose outside every unit, which is still not reported. `s` and `strike`, which are still
  presentational. The formatter, which already lays out both forms canonically. The `ins` and
  `del` attributes `cite` and `datetime`, which the projection still ignores.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-validation`: "Change markup in or around a semantic unit is rejected" gains the one
  place where the markup is permitted, the edges of that place, and a reason in the diagnostic.
  The requirement is `reject-change-markup-inside-a-unit`'s and is not accepted yet, so the block
  is written against that change's delta and is refreshed from the accepted text once it archives.
  It is modified rather than joined by a second requirement, because two requirements would each
  state half of one rule about the same elements.
- `document-schema`:
  - **Added: "A deleted word is recorded apart from what a unit states".** It states what the two
    elements mean in the model and where deleted text is recorded. It is added rather than folded
    into "The HTML-to-model projection is published as a normative specification", because that
    requirement lists what the published specification covers and states no projection behavior
    itself. This follows `record-a-renamed-requirement`, which added "A renamed requirement states
    the heading it replaced" beside it.
  - **Modified: "The HTML-to-model projection is published as a normative specification".** Its
    list gains one item, how change markup projects, and one scenario. Section 8.2 of the
    projection specification makes every unreserved element presentational, and `del` is no
    longer that, so the list's "how presentational markup … is discarded" no longer covers it.
- `artifact-kinds`: **added: "A marked modification reaches the accepted contract as its current
  text".** It is added rather than folded into "Every synthesis and every contract update is
  agent-driven", which `state-a-modified-requirement-whole` also modifies, so this change needs no
  re-base onto that edit. The precedent is "A rename reaches the accepted contract as a new heading
  on the same identifier", which states what one kind of merge leaves behind in a requirement of
  its own. The scenario about the comparison is placed here rather than in toolchain-packaging,
  because the comparison needs no change and the fact it rests on is what the merge writes.

`toolchain-packaging` and `document-template` are not modified. No command changes, and the
stylesheet revision carries out document-template's "The visual system favors comprehension over
decoration" as it stands.

## Impact

- `crates/htmlspec-parse/src/model.rs`: `Requirement` and `Scenario` gain `deletions: Vec<String>`,
  serialized only when it holds an entry. Only `project.rs` builds either type.
- `crates/htmlspec-parse/src/project.rs`: `collect_text` gives no text for a `del`. The walk over a
  unit's own descendants still finds a unit inside a `del` and supplies no field from anything else
  inside one. A new helper collects each unit's deletions.
- `crates/htmlspec-schema/doc/projection.md`: a numbered rule 8.6, with 8.6.1 and 8.6.2, the
  normalized-text list in section 6.1, and the sentence in section 10.1 that
  `reject-change-markup-inside-a-unit` adds.
- `crates/htmlspec-schema/schema/document.schema.json`: `deletions` on `requirement` and `scenario`.
- `crates/htmlspec-validate/src/rules/change_markup.rs`: the rule reads the document's placement
  from the subject, and decides each element through one pure function that returns the permission
  or the reason it is refused.
- The contract version, one minor step: `CONTRACT_VERSION`, the schema's `schemaVersion` and `$id`,
  `projection.md`'s header, `CHANGELOG.md`, the four `assets/template-*.html` with their pins in
  `assets/vendor.lock.json`, `README.md`, and `SCHEMA_OWNED_LITERALS` in
  `crates/htmlspec-validate/tests/schema_rust_split.rs`.
- `assets/`: a new revision of `htmlspec.css`, recorded under `own`, with the revision it replaces
  moved to `retained`. The print stylesheet is unchanged.
- Tests and fixtures:
  - a mapping fixture, `fixtures/projection/change-markup/`;
  - a conformance fixture, `invalid-schema-change-markup-in-a-modified-unit`, through the
    generator;
  - a suite, `conformance/tests/change_markup.rs`, that validates documents at each placement;
  - the property case `resolve-a-reference-across-its-change` adds to
    `conformance/tests/references.rs`, restated for this rule;
  - CLI tests in `crates/htmlspec-cli/tests/cli.rs`;
  - unit tests in `change_markup.rs` and `project.rs`;
  - a formatter test, a presentation test, and an instructions test.
- `crates/htmlspec-assets/instructions/archive.md` and `sync.md`, with their installed copies
  regenerated.
- **Not in this change**, and recorded so it is not assumed to follow:
  - A diff of a modified requirement against the accepted text. That is
    `show-what-a-modification-changed`.
  - Adding marks to the archive, or to the staged migration's 34 modified requirements. Mapping the
    migrated documents is `migrate-into-the-extended-vocabulary`.
  - Anything about tasks. A task never carries change markup: `dropped` with its reason already
    says that its words were given up.
  - A check that a unit's marks are complete. It needs the accepted text, which one document does
    not hold.
  - Marking a decision, evidence, risk, or verification unit. No accepted specification holds one.
    Permitting it later relaxes the rule and breaks no document.
  - Recording when a word changed. `datetime` on `ins` and `del` stays ignored.
  - Guidance in `assets/template-specification.html`. Its example units are placeholders, as
    `state-a-modified-requirement-whole` found. Only its declared version changes.
