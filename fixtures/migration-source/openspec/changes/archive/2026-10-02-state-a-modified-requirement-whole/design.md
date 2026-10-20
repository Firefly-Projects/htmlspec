## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **The vocabulary already expresses a whole statement.** document-schema's "Semantic units declare
  a change state" gives every unit, a scenario included, one of `added`, `modified`, `removed`, or
  `unchanged`, and a unit with none is `unchanged`. The model holds a requirement's `statements` as a
  list of text and its `scenarios` as units, each with its own `change`
  (`crates/htmlspec-parse/src/model.rs`, `Requirement` and `Scenario`). A scratch specification
  stating a `modified` requirement with scenarios `unchanged`, `unchanged`, `removed`, and `added`
  validates today and projects exactly those four states
  (`evidence/the-measurement-2026-09-30.md`, section 5).
- **Nothing reads a restatement against the accepted text.** `sync` and archival compare
  identifiers and, for a rename, headings (toolchain-packaging's "The command surface reports
  whether a change reached the accepted contract"). The same scratch project shows a partial
  restatement and a whole one both validating, and both reported as `reached`.
- **Change states are not "material" when `unchanged`.** `material_states` in
  `crates/htmlspec-validate/src/rules/review.rs` excludes `unchanged` and `removed`. So a scenario
  restated as `unchanged` inside a `modified` requirement raises no stale-reference warning of its
  own. A reference to a `removed` scenario is `refs/removed-target`, as it is today.
- **The merge is instructions, not code.** artifact-kinds' "Every synthesis and every contract update
  is agent-driven" makes the merge an agent's work. What an agent reads is
  `crates/htmlspec-assets/instructions/sync.md` and `archive.md`, served by `htmlspec instructions`
  and installed by `init`. Both tell an agent to apply partial updates today (evidence, section 6).
- **OpenSpec 1.13.1 took the same turn.** Its `openspec-sync-specs` template now says "A MODIFIED
  block carries the whole requirement". Its `validate` and `archive` refuse a MODIFIED block that drops
  a scenario the main specification holds, matching by scenario name
  (`findMissingCurrentScenarios` in `dist/core/parsers/requirement-blocks.js`). It refused all three of
  this archive's partial blocks, including the two meant as withdrawals (evidence, section 4).

## Goals / Non-Goals

**Goals:**

- One rule for what a change's specification states about a requirement it modifies, stated in the
  contract with its reason, and the rejected rule recorded beside it.
- A withdrawn scenario that is visible in the change's specification, and not inferred from an absence.
- A merge whose result for a modified requirement is determined by the change's specification alone.
- Instructions that say the same thing as the contract, in every place an agent reads them.

**Non-Goals:**

- **A check.** A single document does not hold the accepted requirement it restates. The comparison
  belongs to `sync` and archival, and it is `see-a-modification-the-merge-did-not-take`'s.
- **Marking changed words inside a statement.** A statement paragraph has no change state. Showing
  which words moved is `mark-the-words-a-modification-changed`'s `<ins>` and `<del>`.
- **Notes, and the change states an accepted specification carries after a merge.** The contract is
  silent on both today, for added requirements as well, and nothing compares either.
- **Removed requirements.** A withdrawn requirement is not asked to restate its content.
- **Migration.** Completing a source MODIFIED block from the git record is project-migration's subject.

## Decisions

### 1. A modified requirement is stated whole, on purpose

The requirement states every statement and every scenario it will hold once merged. Each scenario
carries its own change state. A withdrawn scenario is stated as a unit marked `removed`, and the merge
deletes it. The merge applies the requirement as stated.

This is OpenSpec's schema rule, chosen deliberately and not by default. It is also the rule the
archive already follows in 31 of 34 blocks, and the rule OpenSpec 1.13.1 enforces for scenarios.

**Considered: the partial rule, as the contract states it today.** Rejected, for three reasons.

- A scenario left out is either withdrawn or kept, and the text is the same for both. The archive
  holds both readings in one change on one day: `separate-artifact-kinds` withdrew "Status visible to a
  reviewer" by leaving it out, and kept "Unknown subcommand" and "Discoverable help" after leaving them
  out.
- The merge result cannot be derived from the change's specification. A reader of an archived change
  cannot read the modified requirement as the change left it without the accepted specification of
  that day. "A specification states the settled contract as built" asks for the opposite in its
  scenario "The contract read without the change".
- A comparison of a modified requirement with the accepted text cannot mean anything, because a
  partial restatement always differs from it. That comparison is the next change's whole subject.

**Considered: the partial rule plus `removed`.** Here a scenario left out would mean "carried forward",
and a withdrawn scenario would be stated as `removed`. That removes the ambiguity for scenarios, and
it is the strongest alternative. Rejected, because it keeps the second and third reasons above, and
because it has no answer for a statement. A paragraph is not a unit and carries no change state. A
partial restatement of a requirement's statements cannot say which accepted paragraph a stated one
replaces, or whether an unstated one is kept. Answering that would need an identity for each paragraph,
which is new vocabulary and a contract bump. A whole statement needs none: the statements it states are
the statements.

**Considered: OpenSpec 1.13.1's rule exactly, which is whole blocks with every omission refused.**
Adopted in substance, with one difference. OpenSpec's Markdown has no change state for a scenario, so a
MODIFIED block cannot withdraw a scenario at all: 1.13.1 refused both of this archive's deliberate
withdrawals. html-spec can state a withdrawal, and it does.

**Considered: omission means withdrawal**, which is the schema's reading and the one the staged
migration's replay first applied (evidence, section 3). Rejected. It is how the ambiguity arose. It
hides a withdrawn scenario's text from the reviewer that document-schema keeps it present for. And it
turns an accidental omission into a silent deletion.

### 2. The rule is its own requirement

"A change's specification states a modified requirement whole" is added to artifact-kinds. "Every
synthesis and every contract update is agent-driven" loses its partial paragraph and gains two
sentences: the merge preserves what the change does not state, and applies a modified requirement as
stated.

**Considered: folding the rule into "Every synthesis and every contract update is agent-driven"**,
where the partial clause sat. That keeps the reversal in the place a reader last saw the rule. It is
rejected because the two are different subjects. That requirement is about who performs the four
operations. This rule is about what a change's specification must contain, and it needs six scenarios
of its own. The partial clause sat in the wrong requirement, as merge advice that actually governed
what a specification may omit, which is part of why it contradicted OpenSpec's schema unnoticed. A
requirement of its own gives `see-a-modification-the-merge-did-not-take` a heading to cite. It also
keeps the overlap with `mark-the-words-a-modification-changed`, which modifies the same requirement
later, to the one paragraph replaced here.

**Considered: folding it into "A specification states the settled contract as built".** Rejected. That
requirement covers every specification, accepted ones included, and a change state means nothing in an
accepted specification.

The scenario "Merging a scenario into an existing requirement" keeps its name and changes its outcome.
OpenSpec 1.13.1 refuses a MODIFIED block that drops an accepted scenario's name, so this change's own
delta could not withdraw it by omission. Keeping the example and changing the answer also shows the
reversal plainly.

### 3. An accepted scenario left out is reported, never resolved

Where a modified requirement neither states an accepted scenario nor marks it `removed`, the merging
agent reports it to the contributor, names the scenario, and does neither.

**Considered: default to keeping it.** Rejected. It is wrong for "Status visible to a reviewer" and
"One identity per workflow", which were withdrawn.

**Considered: default to deleting it.** Rejected. It is wrong for "Unknown subcommand" and "Discoverable
help", which were kept.

Each default repeats a recorded error, and only the author knows which was meant. This is the same
division "Archival assesses the merge and never performs it" draws for an absent requirement: whether it
was withdrawn or omitted by mistake is a judgment for an agent.

### 4. A stated scenario carries its accepted counterpart's identifier

A scenario the change carries forward, alters, or withdraws keeps the identifier the accepted scenario
has. That identifier is what makes the stated scenario and the accepted one the same scenario.

**Considered: matching by heading**, as OpenSpec does. Rejected. A change that alters a scenario's
heading would then look like it withdrew one scenario and added another. The identity question was
settled the same way for requirements: "A rename reaches the accepted contract as a new heading on the
same identifier" finds the accepted requirement by identifier, never by heading. **Confirm at review.**
This sentence goes one step past the rule itself. It is included because the next change's comparison
needs to know which accepted scenario a stated one continues. That change could decide it instead, and
the sentence would then move there.

### 5. No validation rule

**Considered: a `validate` rule that reads the accepted specifications and reports a restatement that
is not whole.** Rejected. A rule in `validate` would make a document's validity depend on time. An
archived change's modified requirement matches the accepted text only until a later change modifies the
same requirement. After that, the rule would report the archived change as incomplete, although nothing
in it changed. The question "did this change state the requirement whole?" is asked at the moment the
change is merged, and `sync` and archival are the operations that run then. Adding the comparison there
is `see-a-modification-the-merge-did-not-take`'s subject.

### 6. The instructions are the implementation

Nothing in the program changes. The rule is carried out by the agent that writes a specification at
archival and merges it, so the edits are to `sync.md` and `archive.md`:

- `sync.md` drops "Apply partial updates", and adds a section on merging a modified requirement. That
  section says to work from the change's specification, because a modified requirement's identifier is
  already held and so it appears in `reached` whether or not its text was merged. It says what applying
  "as stated" means, to report a scenario marked `unchanged` whose text differs from the accepted one,
  and to report an accepted scenario that is neither stated nor marked `removed`. The rename step's "as
  a partial update like any other" becomes "as the change states it".
- `archive.md` step 2 gains how to state a modified requirement, with a short example beside the one
  for a rename. Step 5 drops its quotation of OpenSpec's partial guidance and the bullet "must not
  restate the requirement".

**Considered: an example modified requirement in `assets/template-specification.html`.** Rejected. The
template's example units are placeholders that `review/example-content` asks the author to replace. A
modified-requirement example would be one more to delete in every new specification, including those of
changes that modify nothing. The template is also sha256-pinned in `assets/vendor.lock.json`, so editing
it moves a pin for no gain. The guidance belongs where the specification is written, which is
`archive.md` step 2.

**Considered: a `## MODIFIED Requirements` row in `migrate.md`.** Rejected here. A migrated specification
is bound by the new requirement like any other. But completing a source block that left a scenario out
means reading the git record of the source's merge, and that is an obligation project-migration does not
state today. It belongs to a change whose subject is migration.

## Risks / Trade-offs

- **[A whole restatement is longer, and a reader must find what changed in it.]** → Each scenario's
  change state says which scenarios moved. For statements, `mark-the-words-a-modification-changed`
  marks the changed words and `show-what-a-modification-changed` gives the diff. Until then, the change's
  proposal and design say what moved, as they do today.
- **[A stale restatement reverts another change's edit.]** A change written before another change
  modified the same requirement restates the old text. Merged as stated, it silently undoes the other
  change. Under the partial rule it overwrote only what it mentioned. → A scenario marked `unchanged`
  whose text differs from the accepted one is a visible mismatch, and `sync.md` tells the merging agent to
  report it rather than apply it. Statements have no such mark, so this project's overlap convention, a
  "1.0 Re-base" task in the later change, still carries that case. `see-a-modification-the-merge-did-not-take`
  then reports any difference at `sync` and archival. OpenSpec 1.13.1 carries the same risk, because its
  archive replaces the whole block.
- **[An agent following this repository's installed OpenSpec skills still applies the partial rule.]** →
  They are OpenSpec 1.7.0's generated files and govern this repository's OpenSpec-form work, not the
  product. This project's deltas are now written under a convention that restates a modified
  requirement whole, and OpenSpec 1.13.1's `validate --strict` refuses one that drops a scenario.
- **[Documents in other projects state a modified requirement partially.]** → They stay valid, because no
  rule is added. Reporting the difference at `sync` and archival is
  `see-a-modification-the-merge-did-not-take`'s subject.

## Migration Plan

No document changes meaning, and none stops validating. The document contract is untouched. No unit,
field, attribute, or projection rule is added or reshaped, so under the bump table in
`crates/htmlspec-schema/CHANGELOG.md` the contract version stays `0.3.0`, the version
`reject-change-markup-inside-a-unit` set when it archived. This design first said `0.2.0`, because it was
written before that change landed. The new conformance fixture
pins behavior the contract already has. Fixtures have been added before without moving the version:
`0ce5ad5`, `16df91e`, and `5ea25e7` each added conformance fixtures, and the changelog has no entry for
any of them.

The release is expected to be `0.20.0`, the minor increment after
`reject-change-markup-inside-a-unit`'s `0.19.0`. A minor increment fits because an accepted requirement is
added and the obligations on a change's specification change. A patch is arguable, because no program
behavior changes. The contributor confirms the increment at the release task.

Rollback is a revert. It restores the partial clause and the instructions, and leaves every document as
it was.
