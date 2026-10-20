## MODIFIED Requirements

### Requirement: A single command exposes validate, format, and init

The project SHALL provide one command-line program offering, at minimum, a subcommand to validate documents, a subcommand to format documents, and a subcommand to initialize a new document or project. Each subcommand SHALL be usable without configuration in a conforming project.

Initialization SHALL take the artifact kind of the document to create, and SHALL produce a document already shaped for that kind. A `proposal` SHALL be initialized as a page of prose with no unit scaffolding at all, and a `specification` SHALL be initialized with the units its kind expects.

The asymmetry is the requirement. A proposal that arrived full of empty requirement scaffolding would push its author toward stating a contract at the stage meant for arguing about one, and the emptiest possible starting point is the correct one for a document whose structure is the author's to choose.

Initialization SHALL be able to create a whole change — the directory and its documents — rather than one document at a time, because a change is a directory of kinded documents.

#### Scenario: Validating a project
- **WHEN** a contributor runs the validate subcommand in a project with no arguments
- **THEN** every document in the project is validated and the results are reported

#### Scenario: Validating one document
- **WHEN** a contributor runs the validate subcommand against a single file path
- **THEN** only that document is validated

#### Scenario: Initializing a proposal
- **WHEN** a contributor initializes a document of the proposal kind
- **THEN** it opens as a readable page of prose, with no semantic unit scaffolding to fill in

#### Scenario: Initializing a specification
- **WHEN** a contributor initializes a document of the specification kind
- **THEN** it arrives with the units that kind expects, because its shape is not the author's to invent

#### Scenario: Initializing a change
- **WHEN** a contributor initializes a new change
- **THEN** its directory is created with the documents that stage needs, each declaring its kind and each valid immediately

## ADDED Requirements

### Requirement: Archival produces the specification and updates the accepted contract

The command surface SHALL provide an archival operation that produces a change's specification artifact from its proposal, design, and tasks, validates it, and then applies its contract changes to the project's accepted specifications.

There SHALL be one specification artifact per change, and it SHALL be the same file as that change's as-built record. The project's accepted specifications SHALL be a separate destination outside the change directory.

The production SHALL be performed by an agent following stated instructions rather than by a deterministic extractor in the program. The program's part SHALL be the checking: it validates the produced document and reports what is wrong with it. That division is deliberate — the judgment of what a design's prose obliges is not derivable by rule, while whether the result conforms is exactly what a validator determines.

The project SHALL NOT ship a command that parses a proposal or a design and emits requirements. Those kinds carry no imposed structure, so there is nothing for such a command to parse reliably, and one written anyway would produce a specification that passes every check and misstates what the change established.

Archival SHALL NOT complete while the produced specification reports an error. A change archived with an invalid specification would leave the accepted contract in a state the project's own tools reject.

#### Scenario: Archiving a completed change
- **WHEN** a contributor archives a change whose tasks are complete
- **THEN** its specification artifact is produced, validated, applied to the project's accepted specifications, and archived alongside the change's other documents

#### Scenario: The change's documents survive
- **WHEN** archival completes
- **THEN** the change's proposal, design, tasks, and specification are all still readable in the archive

#### Scenario: Archiving with an invalid specification
- **WHEN** the produced specification reports an error
- **THEN** archival does not complete, and the error is reported for reconciliation

#### Scenario: The tool does not extract requirements itself
- **WHEN** the archival operation runs
- **THEN** the program validates the specification an agent wrote rather than deriving requirements from the source documents by rule
