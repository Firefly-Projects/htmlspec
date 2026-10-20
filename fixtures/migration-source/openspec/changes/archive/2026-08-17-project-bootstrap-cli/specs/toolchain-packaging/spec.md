## RENAMED Requirements

The requirement's subject changed rather than its topic. `init` used to create one document, and its subject is now the project, so a requirement naming `init` alongside `validate` and `format` no longer describes the command surface. The rename is stated rather than implied, because a MODIFIED block cannot find a header that moved.

- FROM: `### Requirement: A single command exposes validate, format, and init`
- TO: `### Requirement: A single command exposes validate, format, bootstrap, and create`

## MODIFIED Requirements

### Requirement: A single command exposes validate, format, bootstrap, and create

The project SHALL provide one command-line program offering, at minimum, a subcommand to validate documents, a subcommand to format documents, a subcommand to bootstrap a project, and a subcommand to create documents. Each subcommand SHALL be usable without configuration in a conforming project.

The bootstrap subcommand's subject SHALL be a project and it SHALL require no arguments, because the operation a contributor needs first is the one that establishes the project they are standing in. The creation subcommand's subject SHALL be a document or a change.

These SHALL be distinct subcommands. One subcommand meaning both "establish this project" and "write this file" would leave a contributor unable to ask for either without also specifying which they meant.

Creation SHALL take the artifact kind of the document to create, and SHALL produce a document already shaped for that kind. A `proposal` SHALL be initialized as a page of prose with no unit scaffolding at all, and a `specification` SHALL be initialized with the units its kind expects.

The asymmetry is the requirement. A proposal that arrived full of empty requirement scaffolding would push its author toward stating a contract at the stage meant for arguing about one, and the emptiest possible starting point is the correct one for a document whose structure is the author's to choose.

Creation SHALL be able to create a whole change — the directory and its documents — rather than one document at a time, because a change is a directory of kinded documents. A change created in a project SHALL be placed where that project keeps its active changes, and the resolved location SHALL be reported.

Documents SHALL be created referencing the project's asset store rather than establishing one of their own.

#### Scenario: Validating a project
- **WHEN** a contributor runs the validate subcommand in a project with no arguments
- **THEN** every document in the project is validated and the results are reported

#### Scenario: Validating one document
- **WHEN** a contributor runs the validate subcommand against a single file path
- **THEN** only that document is validated

#### Scenario: Bootstrapping with no arguments
- **WHEN** a contributor runs the bootstrap subcommand with no arguments
- **THEN** it establishes a project in the current repository rather than reporting a missing argument

#### Scenario: Creating a proposal
- **WHEN** a contributor creates a document of the proposal kind
- **THEN** it opens as a readable page of prose, with no semantic unit scaffolding to fill in

#### Scenario: Creating a specification
- **WHEN** a contributor creates a document of the specification kind
- **THEN** it arrives with the units that kind expects, because its shape is not the author's to invent

#### Scenario: Creating a change
- **WHEN** a contributor creates a new change in a project
- **THEN** its directory is created where the project keeps active changes, holding the documents that stage needs, each declaring its kind and each valid immediately, and its location is reported

#### Scenario: A created document uses the project's store
- **WHEN** a contributor creates a document in a project
- **THEN** it references the project's existing asset store and no second store is established

#### Scenario: Creating a change outside a project
- **WHEN** a contributor creates a change where no project exists
- **THEN** the program reports that no project was found and names the bootstrap subcommand

#### Scenario: Unknown subcommand
- **WHEN** a contributor invokes a subcommand that does not exist
- **THEN** the program reports the error, lists the available subcommands, and exits with a failure status

#### Scenario: A withdrawn spelling names its replacement
- **WHEN** a contributor invokes the bootstrap subcommand with the arguments creation formerly took
- **THEN** the program reports the error and names the subcommand that now performs it

#### Scenario: Initializing a proposal
- **GIVEN** that initialization formerly created one document and its subject is now the project
- **WHEN** a contributor names a document path on the bootstrap subcommand
- **THEN** the program refuses, states that the subject is the project, and names the creation subcommand that writes one document

#### Scenario: Initializing a specification
- **WHEN** a contributor names a document path on the bootstrap subcommand for any artifact kind
- **THEN** the refusal is the same regardless of kind, because the subject changed rather than the kind vocabulary

#### Scenario: Initializing a change
- **WHEN** a contributor names a path on the bootstrap subcommand intending to create a change
- **THEN** the program refuses and names the creation subcommand for a change, so a contributor who used the withdrawn spelling is not left guessing

#### Scenario: Discoverable help
- **WHEN** a contributor requests help
- **THEN** every subcommand and option is documented in the output

### Requirement: Exit codes distinguish outcomes

The program SHALL exit zero when the requested operation succeeds, and nonzero when it does not. A document that fails validation SHALL be distinguishable by exit code from a failure to run the operation at all, such as an unreadable file or an invalid invocation.

An operation a contributor declined or interrupted SHALL be distinguishable by exit code from both success and a failure to run. It is neither: nothing went wrong, and nothing was done.

#### Scenario: Clean validation
- **WHEN** validation finds no errors
- **THEN** the program exits zero

#### Scenario: Validation failure
- **WHEN** validation finds at least one error
- **THEN** the program exits nonzero with the code designated for document violations

#### Scenario: Operational failure
- **WHEN** a specified file does not exist
- **THEN** the program exits nonzero with a code distinct from that used for document violations

#### Scenario: Cancelled operation
- **WHEN** a contributor declines or interrupts an interactive operation
- **THEN** the program exits with a code distinct from success, from document violations, and from operational failure

#### Scenario: Continuous integration gate
- **GIVEN** a continuous integration job that runs validation
- **WHEN** any document in the project is invalid
- **THEN** the job fails

### Requirement: Output is available in human and machine forms

The program SHALL produce human-readable output by default and structured machine-readable output on request. Structured output SHALL be written to standard output, and SHALL contain nothing but the structured payload so it can be consumed directly.

Requesting structured output SHALL suppress every interactive prompt. A prompt writes to the stream the payload is promised to occupy alone, so the two cannot both be honored.

#### Scenario: Agent consumes structured output
- **WHEN** an agent runs validation requesting structured output
- **THEN** standard output contains only a well-formed structured payload of diagnostics

#### Scenario: Progress does not corrupt structured output
- **WHEN** structured output is requested
- **THEN** progress messages, warnings, and errors about the run itself go to standard error, leaving standard output parseable

#### Scenario: Structured output suppresses prompts
- **WHEN** an operation that would prompt is run with structured output requested
- **THEN** it does not prompt, and standard output contains only the payload

#### Scenario: Default output is for people
- **WHEN** a contributor runs validation without requesting structured output
- **THEN** the output is formatted for reading in a terminal
