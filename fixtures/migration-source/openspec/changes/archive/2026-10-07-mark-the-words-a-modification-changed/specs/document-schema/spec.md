## ADDED Requirements

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

## MODIFIED Requirements

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
