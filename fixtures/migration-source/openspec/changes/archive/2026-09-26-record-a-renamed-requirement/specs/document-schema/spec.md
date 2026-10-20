## ADDED Requirements

### Requirement: A renamed requirement states the heading it replaced

A requirement unit SHALL be able to state the heading it replaced, in one paragraph marked
`data-field="renamed-from"` whose text is that previous heading. The requirement's own heading SHALL
be its current name. The pair is OpenSpec's `FROM:`/`TO:` rename, expressed on the unit.

A rename SHALL be stated separately from the unit's change state. The change state SHALL describe
what happened to the requirement's statements and scenarios, and the `renamed-from` paragraph SHALL
describe what happened to its heading. A requirement that is renamed and rewritten in one change
SHALL carry `data-change="modified"` and a `renamed-from` paragraph. A requirement that is renamed
and otherwise untouched SHALL carry `data-change="unchanged"` and a `renamed-from` paragraph. Neither
fact SHALL be inferred from the other.

The reason is that a rename and a modification are different facts, and a change can make either
without the other. A single value that meant "renamed" would force a requirement that was renamed and
rewritten to give up one of the two, and both of this project's own renames are that case.

A renamed requirement SHALL keep its identifier, as "Every semantic unit has a stable unique
identifier" already requires. The `renamed-from` paragraph is what lets a reader who knows the
previous heading find the requirement under its new one.

A `renamed-from` paragraph SHALL NOT supply the requirement's normative statement, and its text SHALL
project into the requirement's `renamedFrom` field rather than into its statements.

The following SHALL make a document invalid:

- a `renamed-from` paragraph in any unit other than a requirement;
- a `renamed-from` paragraph in a requirement whose change state is `added`, because a requirement
  this change introduces had no previous heading;
- a `renamed-from` paragraph in a requirement whose change state is `removed`, because a withdrawn
  requirement is not also renamed, and OpenSpec rejects a rename and a removal of one requirement;
- more than one `renamed-from` paragraph in one requirement;
- a `renamed-from` paragraph whose text is empty after whitespace normalization;
- a `renamed-from` paragraph whose text equals the requirement's own heading after whitespace
  normalization, because a rename to the same name renames nothing.

A requirement's heading, for every comparison a rename makes, SHALL be its heading name: its
projected title with the requirement's own identifier removed from the end, then trimmed. A heading
commonly shows its identifier beside the name (`<span class="hs-id">req-x</span>`), and the projected
title carries that text. A previous heading is stated as the name a reader read, without the
identifier. So a comparison with the title itself would never find the two equal.

#### Scenario: A requirement renamed and rewritten
- **WHEN** a requirement carries `data-change="modified"`, a new heading, and a `data-field="renamed-from"` paragraph naming its previous heading
- **THEN** the document is valid, and a consumer reads both that the content changed and the heading it replaced

#### Scenario: A requirement renamed and otherwise untouched
- **WHEN** a requirement carries `data-change="unchanged"` and a `data-field="renamed-from"` paragraph
- **THEN** the document is valid, and the requirement's statements and scenarios are understood as carried forward untouched under a new heading

#### Scenario: The previous heading is a field, not a statement
- **WHEN** a requirement carries a `renamed-from` paragraph and one unmarked paragraph
- **THEN** its statements are the unmarked paragraph alone, and its `renamedFrom` field is the previous heading

#### Scenario: A rename on a new requirement
- **WHEN** a requirement carrying `data-change="added"` contains a `renamed-from` paragraph
- **THEN** the document is invalid, because a requirement this change introduces had no previous heading

#### Scenario: A rename on a withdrawn requirement
- **WHEN** a requirement carrying `data-change="removed"` contains a `renamed-from` paragraph
- **THEN** the document is invalid

#### Scenario: A rename outside a requirement
- **WHEN** a scenario, decision, task, evidence, risk, or verification unit contains a `renamed-from` paragraph
- **THEN** the document is invalid

#### Scenario: Two previous headings
- **WHEN** one requirement contains two `renamed-from` paragraphs
- **THEN** the document is invalid, because one rename replaces one heading

#### Scenario: A rename to the same name
- **WHEN** a requirement's `renamed-from` paragraph states the requirement's own heading
- **THEN** the document is invalid

#### Scenario: A heading that shows its identifier
- **WHEN** a requirement's heading is `The name <span class="hs-id">req-x</span>` and its `renamed-from` paragraph states `The name`
- **THEN** the document is invalid, because the identifier shown beside a heading is not part of its name

#### Scenario: A requirement with no rename
- **WHEN** a requirement carries no `renamed-from` paragraph
- **THEN** its `renamedFrom` field is absent, and the requirement means exactly what it meant before this capability existed
