## Purpose

Defines the normative HTML element contract for the semantic units of a specification — requirements, scenarios, decisions, tasks, evidence, risks, and verification — so that agents can address and revise individual units reliably and humans read a document rather than a data format.

## ADDED Requirements

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

A task, verification, evidence, or risk unit MAY reference the semantic units it relates to. Such a reference SHALL name the target by its identifier, and SHALL resolve to a unit that exists in the same document or in an accepted spec reachable from it.

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

#### Scenario: Reporting progress
- **WHEN** a consumer reads a document containing ten tasks of which four are `done`
- **THEN** the consumer can report progress without parsing prose or rendered markup

#### Scenario: Dropped task without a reason
- **WHEN** a task declares state `dropped` and gives no reason
- **THEN** the document is invalid

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

#### Scenario: Independent implementation reproduces the model
- **GIVEN** an implementation written from the projection specification and the schema, with no access to this project's source code
- **WHEN** it projects a conformance fixture
- **THEN** it produces the same model as this project's implementation

#### Scenario: Presentational markup is ignored predictably
- **WHEN** a document wraps a requirement's prose in additional presentational elements that carry no reserved class
- **THEN** the projection specification determines whether that markup is discarded, and the resulting model does not vary between implementations

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
