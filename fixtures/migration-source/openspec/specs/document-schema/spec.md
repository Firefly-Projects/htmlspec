## Purpose

Defines the normative HTML element contract for the semantic units of a specification — requirements, scenarios, decisions, tasks, evidence, risks, and verification — so that agents can address and revise individual units reliably and humans read a document rather than a data format.

## Requirements

### Requirement: Semantic units are identified by class

A document SHALL express each semantic unit as an HTML element carrying a reserved class name from a closed vocabulary: `requirement`, `scenario`, `decision`, `task`, `evidence`, `risk`, and `verification`. A reserved class name SHALL NOT be used on an element that is not that semantic unit.

#### Scenario: Recognized unit
- **WHEN** a document contains `<section class="requirement" id="req-theme-selection">`
- **THEN** consumers treat that element and its descendants as one requirement unit

#### Scenario: Class outside the vocabulary
- **WHEN** a document uses a class name that is not reserved, such as `class="note"`
- **THEN** the element carries no semantic-unit meaning and is treated as ordinary content

#### Scenario: Reserved class on a decorative element
- **WHEN** a document applies `class="requirement"` to an element used only for layout, with no requirement content
- **THEN** the document is invalid

### Requirement: Every semantic unit has a stable unique identifier

Every semantic unit SHALL carry an `id` attribute. The identifier SHALL be unique within the document, SHALL use a prefix reserved for its unit type, and SHALL consist of lowercase alphanumeric characters and hyphens.

#### Scenario: Prefix matches unit type
- **WHEN** a requirement is identified
- **THEN** its `id` begins with `req-`, a scenario with `scenario-`, a decision with `decision-`, a task with `task-`, evidence with `evidence-`, a risk with `risk-`, and verification with `verify-`

#### Scenario: Duplicate identifier
- **WHEN** two elements in the same document share the same `id`
- **THEN** the document is invalid

#### Scenario: Identifier is stable across edits
- **WHEN** an agent revises the prose of a requirement without changing what it requires
- **THEN** the requirement's `id` is unchanged, and references to it continue to resolve

#### Scenario: Renaming a requirement
- **WHEN** a requirement's title changes but it describes the same obligation
- **THEN** its `id` SHALL be preserved rather than regenerated from the new title

### Requirement: Semantic units declare a change state

Every semantic unit SHALL declare its lifecycle state in a `data-change` attribute with one of the values `added`, `modified`, `removed`, or `unchanged`. A unit with no `data-change` attribute SHALL be interpreted as `unchanged`.

#### Scenario: Newly proposed requirement
- **WHEN** a change introduces a requirement that does not exist in the accepted specs
- **THEN** the requirement carries `data-change="added"`

#### Scenario: Requirement being withdrawn
- **WHEN** a change removes an accepted requirement
- **THEN** the requirement carries `data-change="removed"` and remains present in the document so reviewers can see what is being withdrawn

#### Scenario: Unrecognized change state
- **WHEN** a unit carries `data-change="pending"`
- **THEN** the document is invalid

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

### Requirement: A requirement contains a title and normative statement

A requirement unit SHALL contain exactly one heading giving its name and at least one element of prose stating the obligation. The normative statement SHALL use `SHALL` or `MUST`, and SHALL NOT rely on `should` or `may` to express an obligation.

#### Scenario: Well-formed requirement
- **WHEN** a requirement contains a heading and a paragraph reading "The application MUST let the user select a light or dark theme"
- **THEN** the requirement is well formed

#### Scenario: Requirement with no normative statement
- **WHEN** a requirement contains only a heading
- **THEN** the document is invalid

#### Scenario: Requirement with multiple headings
- **WHEN** a requirement unit contains two headings at its own level
- **THEN** the document is invalid, because the requirement's name is ambiguous

### Requirement: Every requirement contains at least one scenario

A requirement SHALL contain at least one scenario unit nested within it. A scenario SHALL NOT appear outside a requirement.

#### Scenario: Requirement without scenarios
- **WHEN** a requirement declares an obligation but contains no scenario
- **THEN** the document is invalid

#### Scenario: Orphaned scenario
- **WHEN** a scenario appears as a sibling of requirements rather than nested inside one
- **THEN** the document is invalid

### Requirement: A scenario states conditions and expected outcomes

A scenario SHALL contain a heading naming it and a structured body pairing each condition with its expected outcome, using the labels `Given`, `When`, and `Then`. A scenario SHALL contain at least one `When` and at least one `Then`. `Given` is optional and MAY repeat.

#### Scenario: Minimal scenario
- **WHEN** a scenario pairs a single `When` condition with a single `Then` outcome
- **THEN** the scenario is well formed

#### Scenario: Scenario missing an outcome
- **WHEN** a scenario states a `When` condition with no `Then`
- **THEN** the document is invalid

#### Scenario: Scenario with preconditions
- **GIVEN** a scenario that depends on prior state
- **WHEN** it declares one or more `Given` labels before its `When`
- **THEN** the scenario is well formed

#### Scenario: Unrecognized label
- **WHEN** a scenario body uses a label other than `Given`, `When`, or `Then`
- **THEN** the document is invalid

### Requirement: Cross-references address units by identifier

A task, verification, evidence, or risk unit MAY reference the semantic units it relates to. Such a reference SHALL name the target by its identifier, and SHALL resolve to a unit that exists in the same document or in a document reachable from it.

**A reference written in a `data-refs` attribute SHALL be resolved in the following places, in this order, and SHALL resolve to the unit declared in the first place that declares its identifier:**

1. the document that makes the reference;
2. where that document is one of a change's documents other than its specification, that change's own specification;
3. where that document is one of a change's documents, including its specification, or is an accepted specification, the project's accepted specifications, every one of them.

A change's documents are the documents directly within a change's directory, whether the change is active or archived. The project's accepted specifications are the documents beneath the place where the project keeps them. Nothing else is reachable. A document outside any project, a file beneath a change's evidence, and a runbook's documents reach only themselves. A change's documents do not reach another change's documents, and they do not reach their own change's proposal, design, or tasks.

A change's other documents reach its specification because they exist to serve it. A proposal argues for what the specification states, a design reasons about it, and the tasks carry it out. The tasks template already tells an author that a task may reference the requirement it serves "once that requirement exists in this change's specification". The accepted specifications are reachable because they are the contract every change starts from.

**The order decides which unit a reference means when two places declare one identifier.** A change's specification restates, under the accepted identifier, each accepted requirement the change modifies or withdraws. For the change's own documents, the change's statement is the one that counts. So a task that references a requirement its change withdraws targets a withdrawn obligation, even while the accepted specification still holds that requirement unchanged.

**Every accepted specification is reachable, and an accepted specification reaches the others.** A reference names a unit, not a capability. The merge that applies a change places each requirement in the accepted specification for its capability, so a reference that resolved in a change's specification must still resolve after that change's units are merged into two different accepted specifications.

A reference written as a fragment link, `href="#…"`, SHALL resolve within the document that makes it. It is a hyperlink into that page. If it resolved somewhere else while the link arrived nowhere, the reference would be true and the hyperlink false.

Where more than one accepted specification declares an identifier, a reference to it SHALL resolve. Which of them it means is not decided here.

A reference SHALL NOT resolve to a unit that the place resolving it marks `removed`. This holds in every place, not only in the document that makes the reference. A task and the requirement it serves rarely share a document, so a rule confined to one document would miss the case the scenario "Reference to a removed requirement" names.

#### Scenario: Task implements a requirement
- **WHEN** a task declares that it satisfies `req-theme-selection`
- **THEN** consumers can report implementation progress per requirement

#### Scenario: Verification covers a scenario
- **WHEN** a verification unit references `scenario-theme-persists`
- **THEN** consumers can report which scenarios have been verified and which have not

#### Scenario: Dangling reference
- **WHEN** a task references `req-does-not-exist`
- **THEN** the document is invalid

#### Scenario: Reference to a removed requirement
- **WHEN** a task references a requirement carrying `data-change="removed"`
- **THEN** the document is invalid, because work cannot target a withdrawn obligation

#### Scenario: A task references a requirement in its change's specification
- **GIVEN** a change whose specification declares `req-x`
- **WHEN** a task in that change's tasks document names `req-x` in its `data-refs`
- **THEN** the reference resolves, and the tasks document is valid

#### Scenario: A task references a requirement only the accepted contract holds
- **GIVEN** an accepted specification declaring `req-y`, and no document in the change declaring it
- **WHEN** a task in the change names `req-y` in its `data-refs`
- **THEN** the reference resolves

#### Scenario: A change withdraws a requirement the accepted contract still holds
- **GIVEN** an accepted specification declaring `req-y` unchanged, and a change whose specification declares `req-y` with `data-change="removed"`
- **WHEN** a task in that change names `req-y` in its `data-refs`
- **THEN** the document is invalid, because the change's specification is reached first and it withdraws the obligation

#### Scenario: Accepted specifications reference one another
- **WHEN** a unit in one accepted specification names, in its `data-refs`, a requirement that another accepted specification declares
- **THEN** the reference resolves

#### Scenario: An identifier two accepted specifications declare
- **WHEN** a task names in its `data-refs` an identifier that two accepted specifications both declare
- **THEN** the reference resolves

#### Scenario: A reference to another change
- **WHEN** a task names in its `data-refs` a requirement that only another change's specification declares
- **THEN** the document is invalid, because a change's documents reach their own change's specification and no other

#### Scenario: A document that reaches only itself
- **WHEN** a document outside any project, or a file beneath a change's evidence, names in its `data-refs` an identifier it does not declare itself
- **THEN** the document is invalid, because nothing beyond it is reachable

#### Scenario: A runbook's document names an accepted requirement
- **WHEN** a runbook's tasks document names in its `data-refs` a requirement that only an accepted specification declares
- **THEN** the document is invalid, because a runbook is not a change and its documents reach only themselves

#### Scenario: A fragment link to a unit in another document
- **GIVEN** a change whose specification declares `req-x`
- **WHEN** a task in that change's tasks document links to `#req-x`
- **THEN** the document is invalid, because a fragment link resolves within its own document

### Requirement: A decision records its outcome and rationale

A decision unit SHALL contain a heading, a statement of the decision reached, and its rationale. It SHALL declare a status of `proposed`, `accepted`, or `superseded`. A superseded decision SHALL reference the decision that replaces it.

#### Scenario: Accepted decision
- **WHEN** a decision states the chosen option and why, with status `accepted`
- **THEN** the decision is well formed

#### Scenario: Superseded decision without a successor
- **WHEN** a decision declares status `superseded` but references no replacement
- **THEN** the document is invalid

### Requirement: A task declares completion state

A task unit SHALL declare whether it is complete in a machine-readable attribute, distinct from any visual checkbox. Completion state SHALL be one of `todo`, `in-progress`, `done`, or `dropped`. A dropped task SHALL state why.

**A `done` task MAY state a deviation: what was done, where that departs from the task's words.** The deviation SHALL be stated in one paragraph marked `data-field="deviation"`, and SHALL project into the task's `deviation` field. A task that states a deviation is `done`, because the work happened. A `done` task that states no deviation SHALL be read as done as its words say.

The reason is that none of the four states says "done, but not as written", and work lands that way. Marked `done` with a note, such a task claims its words were carried out, because a note is prose that no field and no count reads. Marked `dropped`, it claims the work did not happen, which is false. The deviation keeps both facts: the state says the work is finished, and the deviation says how it was done.

A task's words SHALL be kept as written when it states a deviation. They record what was planned, and designs, evidence, and later changes cite a task by them. The deviation is stated beside the words and SHALL NOT replace them.

The following SHALL make a document invalid:

- a `deviation` paragraph in a task whose state is `todo` or `in-progress`, because nothing has been done that could differ from the words;
- a `deviation` paragraph in a task whose state is `dropped`, because a dropped task's work did not happen and its reason already states why. A task whose work partly happened is `done`, and its deviation states which part;
- more than one `deviation` paragraph in one task, because a task has one account of what was done, and the projection does not choose between two.

A `deviation` paragraph in any unit other than a task names no field of that unit, and is invalid for that reason.

#### Scenario: Reporting progress
- **WHEN** a consumer reads a document containing ten tasks of which four are `done`
- **THEN** the consumer can report progress without parsing prose or rendered markup

#### Scenario: Dropped task without a reason
- **WHEN** a task declares state `dropped` and gives no reason
- **THEN** the document is invalid

#### Scenario: A task done another way than its words say
- **WHEN** a task declares state `done` and carries one `data-field="deviation"` paragraph
- **THEN** the document is valid, the task is `done`, and a consumer reads what was done from the task's `deviation` field

#### Scenario: A task done as its words say
- **WHEN** a task declares state `done` and carries no `deviation` paragraph
- **THEN** its `deviation` field is absent, and the task is read as done as its words say

#### Scenario: A deviation on a task not yet done
- **WHEN** a task whose state is `todo` or `in-progress` carries a `deviation` paragraph
- **THEN** the document is invalid

#### Scenario: A deviation on a dropped task
- **WHEN** a task whose state is `dropped` carries a reason and a `deviation` paragraph
- **THEN** the document is invalid, because a dropped task's work did not happen

#### Scenario: Two deviations in one task
- **WHEN** a `done` task carries two `deviation` paragraphs
- **THEN** the document is invalid

#### Scenario: A deviation outside a task
- **WHEN** a requirement, scenario, decision, evidence, risk, or verification unit carries a `data-field="deviation"` paragraph
- **THEN** the document is invalid, because the field belongs to a task

### Requirement: The element contract is published as a machine-readable schema

The element contract SHALL be published as a machine-readable schema that is the single source of truth for the vocabulary, required structure, identifier prefixes, and permitted attribute values of the projected document model. Tools that validate, format, or generate documents SHALL derive their understanding of that model from the schema rather than restating it.

The schema describes the projected model. It does not describe the HTML that produces the model, so the schema alone SHALL NOT be treated as the complete public contract.

#### Scenario: Contract and tooling agree
- **WHEN** the published schema is changed to add a semantic unit
- **THEN** tools reflect the new unit without a separate, independently maintained definition

#### Scenario: Third-party consumer reads the model contract
- **GIVEN** a consumer outside this project
- **WHEN** it reads the published schema
- **THEN** it can determine the full set of semantic units and their required structure in the projected model without reading this project's source code

### Requirement: The HTML-to-model projection is published as a normative specification

The project SHALL publish a normative projection specification stating how a document's HTML becomes the projected model. That specification SHALL be part of the public contract alongside the schema. It SHALL state, at minimum:

- which elements and attributes create a semantic unit, and which are ignored
- how a nested element becomes a field of its containing unit
- which content preserves whitespace exactly, and which content normalizes it
- how a link, and how an explicit reference attribute, each become a reference in the model
- how a figure, its caption, and its visualization source project
- how task completion state and unit change state project
- how presentational markup that carries no semantic unit is discarded
- how HTML's change markup, `ins` and `del`, projects: which of its text a field holds, and where deleted text is recorded

#### Scenario: Independent implementation reproduces the model
- **GIVEN** an implementation written from the projection specification and the schema, with no access to this project's source code
- **WHEN** it projects a conformance fixture
- **THEN** it produces the same model as this project's implementation

#### Scenario: Presentational markup is ignored predictably
- **WHEN** a document wraps a requirement's prose in additional presentational elements that carry no reserved class
- **THEN** the projection specification determines whether that markup is discarded, and the resulting model does not vary between implementations

#### Scenario: Change markup projects predictably
- **WHEN** a document marks an inserted word and a deleted word in a requirement's statement
- **THEN** the projection specification determines which text the statement holds and where the deleted word is recorded, and the resulting model does not vary between implementations

#### Scenario: Projection rule without a fixture
- **WHEN** a rule is added to the projection specification with no corresponding mapping fixture
- **THEN** the conformance suite is incomplete and the contract change is not accepted

### Requirement: Conformance is pinned by fixtures

The contract SHALL be accompanied by a conformance suite of example documents, each labelled valid or invalid. Every invalid fixture SHALL identify the rule it violates. Every rule in the contract SHALL be covered by at least one valid and one invalid fixture.

The suite SHALL also contain mapping fixtures, each pairing an HTML input with its expected projected model, so that the projection specification is pinned as precisely as the schema.

#### Scenario: Rule without coverage
- **WHEN** a rule is added to the contract with no corresponding fixtures
- **THEN** the conformance suite is incomplete and the contract change is not accepted

#### Scenario: Verifying an independent implementation
- **GIVEN** an independent implementation of the contract
- **WHEN** it is run against the conformance suite
- **THEN** it accepts every valid fixture and rejects every invalid fixture, identifying the expected violated rule

### Requirement: A deleted word is recorded apart from what a unit states

HTML's change markup SHALL have one meaning in the projected model, wherever a document uses it. `ins` marks text an edit added, and its text SHALL contribute to the field that encloses it, as the text of any other inline element does. `del` marks text an edit removed, and **its content SHALL contribute to no field of the model.** A paragraph inside a `del` SHALL supply no statement, note, or other field. A description-list entry inside one SHALL supply no scenario entry. A heading inside one SHALL supply no title. A link inside one SHALL project no reference. The link SHALL still project into the document's links, because the page still carries it and a reader can still follow it.

**The text of each `del` SHALL be recorded in the `deletions` of the requirement or scenario it belongs to**, one entry for each `del` that no other `del` encloses, in document order. A `del` belongs to the nearest unit that encloses it, so a deletion inside a scenario is the scenario's and not its requirement's. A `del` in any other unit, or outside every unit, SHALL be recorded nowhere. The document's summary, a figure's caption, and a link's text SHALL hold only the current text.

The reason is that a unit's statements are the contract, and the contract is the text as it now reads. Read as presentational markup, a deleted word ran into the statement beside the words that replaced it: a statement read "its active changes, and its archive, and its runbooks", which is neither the text before the edit nor the text after it. Keeping deleted words out of every field makes the normative text always the current text. Recording them keeps in the model what the page shows a reader, which is that the unit deleted words, and which words. A requirement and a scenario record them because an accepted specification holds those two units and no others, so they are the units whose accepted text a change edits.

The record is a list of deleted texts for each unit, and not one for each field. A deleted paragraph belongs to no field, and a deleted phrase's place in its sentence is in the markup, where a reader sees it. So the model does not pair a deletion with the insertion beside it.

Inserted text SHALL NOT be recorded a second time. It is current text, and it is already in its field.

The projection SHALL apply this whatever the document is and wherever it sits. The projection decides what the markup means, and validation decides where the markup is permitted, under document-validation's "Change markup in or around a semantic unit is rejected". So a document that validation rejects for its markup still projects with its deleted words out of every field.

A unit inside a `del` SHALL still project as a unit. A unit is created by its reserved class, and a withdrawn unit says so with its change state. Validation rejects the `del` around it.

#### Scenario: A word inserted into a statement
- **WHEN** a requirement's statement reads `… <code>status</code>, <ins><code>sync</code>, </ins><code>instructions</code>, and <code>store</code> name the operations OpenSpec also provides.`
- **THEN** the statement reads "… status, sync, instructions, and store name the operations OpenSpec also provides.", and the requirement records no deletion

#### Scenario: A deleted word beside an inserted phrase
- **WHEN** a requirement's statement reads `… its active changes, <del>and </del>its archive, <ins>and its runbooks, </ins>each in its own place.`
- **THEN** the statement reads "… its active changes, its archive, and its runbooks, each in its own place.", and the requirement's `deletions` holds the one entry "and"

#### Scenario: A deleted paragraph
- **WHEN** a requirement holds a paragraph wrapped in `del`
- **THEN** that paragraph is not one of the requirement's statements, and its text is an entry of the requirement's `deletions`

#### Scenario: A deletion inside a scenario
- **WHEN** an entry of a scenario inside a requirement contains a `del` element
- **THEN** the entry holds the current text, the deleted text is in the scenario's `deletions`, and the requirement's `deletions` does not hold it

#### Scenario: A deleted link
- **WHEN** a verification's method contains, inside a `del`, a link to a requirement in the same document
- **THEN** the verification has no reference from that link, and the link is still among the document's links

#### Scenario: A deletion outside every unit
- **WHEN** a paragraph of the document's summary contains a `del` element
- **THEN** the summary holds only the current text, and the deleted text is recorded nowhere

#### Scenario: A document with no change markup
- **WHEN** a document carries no `del` element
- **THEN** no unit has `deletions`, and the model is exactly what it was before this requirement

### Requirement: A partial verification states what it did not check

A verification SHALL be able to record that its check covered only part of what the verification names. That outcome is `partial`. It means the check ran against the requirement or scenario the verification names, reached part of it, and found that part to hold. A check whose covered part did not hold is `failed`, whatever else it did not reach.

The set of outcome values belongs to the published schema, and this requirement does not restate it. "The element contract is published as a machine-readable schema" makes the schema the single source of truth for permitted attribute values. This requirement names `partial` because it attaches an obligation to that one value, and a condition on a value's companion field is a validator rule, as a dropped task's reason is.

**A `partial` verification SHALL state what it did not check**, in one or more paragraphs marked `data-field="unchecked"`. Each paragraph SHALL name a part of the requirement or scenario that the check did not cover. A `partial` verification with no such paragraph SHALL make the document invalid. An unmarked paragraph SHALL NOT satisfy this, because an unmarked paragraph in a verification is a note.

The reason is the one that makes a dropped task state why, in "A task declares completion state": an outcome that says less than the whole is useful only with the rest named. A verification names units, and the smallest unit it can name is a scenario, so a check that reached one clause of a scenario cannot narrow its reference to that clause. Without `partial`, its author has two choices, and both are wrong. `passed` claims the part the check never reached. Deleting the verification loses the record of the part it did. A `partial` that named nothing would be a third way to say nothing about what is left.

The `unchecked` paragraphs SHALL project into the verification's `unchecked` field, in document order, and SHALL NOT project into its notes. A verification recording any other outcome MAY carry them, so that a check that failed or was blocked can state what it never reached.

A check whose condition, subject, or trigger differs from the one a scenario states did not reach that scenario at all, and it is not a partial verification of it. `partial` records a check of the right thing that stopped short, not a check of something nearby.

#### Scenario: A check that reached part of its scenario
- **GIVEN** a scenario whose Then states two outcomes, and a check that observed only the first
- **WHEN** a verification naming that scenario carries `data-outcome="partial"` and a `data-field="unchecked"` paragraph naming the second
- **THEN** the document is valid, and a consumer reads both the outcome and what the check did not cover without parsing prose

#### Scenario: A partial verification that does not say what it left
- **WHEN** a verification carries `data-outcome="partial"` and no `data-field="unchecked"` paragraph
- **THEN** the document is invalid, and the report names the verification

#### Scenario: An unmarked paragraph does not state it
- **WHEN** a `partial` verification carries an unmarked paragraph describing what it did not check, and no `data-field="unchecked"` paragraph
- **THEN** the document is invalid, because an unmarked paragraph in a verification is a note

#### Scenario: Two parts left unchecked
- **WHEN** a `partial` verification carries two `data-field="unchecked"` paragraphs
- **THEN** its `unchecked` field holds both, in document order, because a part dropped from the record would understate what the check missed

#### Scenario: A failed check that says what it did not reach
- **WHEN** a verification carries `data-outcome="failed"` and a `data-field="unchecked"` paragraph
- **THEN** the document is valid

#### Scenario: Any other outcome without it
- **WHEN** a verification carries `data-outcome="passed"` and no `data-field="unchecked"` paragraph
- **THEN** the document is valid, and the verification means exactly what it meant before `partial` existed

### Requirement: A risk records how it was settled

A risk unit SHALL contain a heading and one statement of how the change can fail. **It SHALL declare how the risk was settled**, in a `data-response` attribute with one of the values `mitigated` or `accepted`. A risk with no `data-response` attribute SHALL be interpreted as `mitigated`.

A `mitigated` risk is one the change holds down. It SHALL state what holds it down, in a paragraph marked `data-field="mitigation"`. Where the mitigation does not reach all the way, that paragraph SHALL say how far it reaches. The extent SHALL NOT be a value of its own.

An `accepted` risk is one the change bears as it stands. It SHALL NOT be required to state a mitigation. It MAY carry a `mitigation` paragraph, which then states the part of the risk that something does hold down. It SHALL NOT be required to state its reason in a field of its own, because its statement and its notes are where an author says why a risk is borne.

The reason is that a risk is settled in one of two ways, and only one of them has a mitigation to state. A contract that required a mitigation of every risk left the author of an accepted risk two choices: invent a mitigation, or write a mitigation paragraph that denies being one. Either choice puts a false statement in the risk's mitigation.

The default is `mitigated` because every risk written before this attribute existed had to state a mitigation. Each such risk keeps both its meaning and its validity.

The response SHALL be stated separately from the unit's change state. The change state says what a change does to the risk, and the response says how the risk is answered. Neither SHALL be inferred from the other. The response SHALL NOT be inferred from whether a mitigation paragraph is present, because an accepted risk may carry one.

The response SHALL project into the risk's `response` field, which every risk in the model carries, including a risk whose markup declares none. A `data-response` attribute on a unit other than a risk SHALL project nowhere, as a `data-state` attribute does on a unit other than a task.

The following SHALL make a document invalid:

- a risk whose response is `mitigated`, declared or by default, with no `mitigation` paragraph;
- a `data-response` value other than `mitigated` or `accepted`.

#### Scenario: A mitigated risk
- **WHEN** a risk carries `data-response="mitigated"`, a statement, and a `data-field="mitigation"` paragraph
- **THEN** the document is valid, and the risk's response is `mitigated`

#### Scenario: A risk written before the response attribute existed
- **WHEN** a risk carries a statement and a mitigation paragraph, and no `data-response` attribute
- **THEN** the document is valid, and the risk's response is `mitigated`

#### Scenario: A mitigated risk with no mitigation
- **WHEN** a risk whose response is `mitigated`, declared or by default, carries no mitigation paragraph
- **THEN** the document is invalid, and the report names `data-response="accepted"` as the declaration for a risk the change bears

#### Scenario: An accepted risk with no mitigation
- **WHEN** a risk carries `data-response="accepted"` and a statement, and no mitigation paragraph
- **THEN** the document is valid, its response is `accepted`, and it has no mitigation

#### Scenario: An accepted risk that names a partial measure
- **WHEN** a risk carries `data-response="accepted"`, a statement, and a mitigation paragraph
- **THEN** the document is valid, its response is `accepted`, and its mitigation projects as it would on a mitigated risk

#### Scenario: A risk that is partly held down
- **WHEN** a risk carries `data-response="partial"`
- **THEN** the document is invalid, because how far a mitigation reaches is stated in its paragraph and not in the response

#### Scenario: A response on another unit
- **WHEN** a decision carries `data-response="accepted"`
- **THEN** the attribute projects nowhere, and the decision means what it would mean without it
