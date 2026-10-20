## Purpose

Defines the command-line surface contributors and agents use to work with HTMLSpec documents, and the reproducible packaging that ensures everyone — including continuous integration — builds and runs the same toolchain.

## ADDED Requirements

### Requirement: A single command exposes validate, format, and init

The project SHALL provide one command-line program offering, at minimum, a subcommand to validate documents, a subcommand to format documents, and a subcommand to initialize a new document or project. Each subcommand SHALL be usable without configuration in a conforming project.

#### Scenario: Validating a project
- **WHEN** a contributor runs the validate subcommand in a project with no arguments
- **THEN** every document in the project is validated and the results are reported

#### Scenario: Validating one document
- **WHEN** a contributor runs the validate subcommand against a single file path
- **THEN** only that document is validated

#### Scenario: Unknown subcommand
- **WHEN** a contributor invokes a subcommand that does not exist
- **THEN** the program reports the error, lists the available subcommands, and exits with a failure status

#### Scenario: Discoverable help
- **WHEN** a contributor requests help
- **THEN** every subcommand and option is documented in the output

### Requirement: Exit codes distinguish outcomes

The program SHALL exit zero when the requested operation succeeds, and nonzero when it does not. A document that fails validation SHALL be distinguishable by exit code from a failure to run the operation at all, such as an unreadable file or an invalid invocation.

#### Scenario: Clean validation
- **WHEN** validation finds no errors
- **THEN** the program exits zero

#### Scenario: Validation failure
- **WHEN** validation finds at least one error
- **THEN** the program exits nonzero with the code designated for document violations

#### Scenario: Operational failure
- **WHEN** a specified file does not exist
- **THEN** the program exits nonzero with a code distinct from that used for document violations

#### Scenario: Continuous integration gate
- **GIVEN** a continuous integration job that runs validation
- **WHEN** any document in the project is invalid
- **THEN** the job fails

### Requirement: Output is available in human and machine forms

The program SHALL produce human-readable output by default and structured machine-readable output on request. Structured output SHALL be written to standard output, and SHALL contain nothing but the structured payload so it can be consumed directly.

#### Scenario: Agent consumes structured output
- **WHEN** an agent runs validation requesting structured output
- **THEN** standard output contains only a well-formed structured payload of diagnostics

#### Scenario: Progress does not corrupt structured output
- **WHEN** structured output is requested
- **THEN** progress messages, warnings, and errors about the run itself go to standard error, leaving standard output parseable

#### Scenario: Default output is for people
- **WHEN** a contributor runs validation without requesting structured output
- **THEN** the output is formatted for reading in a terminal

### Requirement: Formatting can check without writing

The format subcommand SHALL support a mode that reports whether documents are canonically formatted without modifying any file, exiting nonzero if any document would change.

#### Scenario: Check mode on formatted files
- **WHEN** check mode runs against canonically formatted documents
- **THEN** the program exits zero and no file is modified

#### Scenario: Check mode on unformatted files
- **WHEN** check mode runs against an unformatted document
- **THEN** the program exits nonzero, names the document, and modifies nothing

#### Scenario: Write mode
- **WHEN** the format subcommand runs in its default write mode
- **THEN** documents that are not canonically formatted are rewritten, and documents already canonical are left byte-identical

### Requirement: The toolchain builds reproducibly from pinned inputs

The project SHALL be buildable from fully pinned inputs, including the compiler toolchain, every dependency, and every vendored browser-side library. Building the same revision SHALL produce equivalent results regardless of what is already installed on the machine.

#### Scenario: Clean machine build
- **GIVEN** a machine with no project toolchain installed beyond the packaging tool
- **WHEN** a contributor builds the project
- **THEN** the build succeeds without installing dependencies by hand

#### Scenario: Two machines agree
- **GIVEN** two machines building the same revision
- **WHEN** each runs the toolchain against the same document
- **THEN** the validation results and formatted output are identical

#### Scenario: Unpinned dependency rejected
- **WHEN** a dependency is introduced without a pinned version and integrity hash
- **THEN** the project's own checks fail

### Requirement: A development environment is provided

The project SHALL provide a reproducible development environment containing the compiler, test tooling, and any auxiliary tools needed to work on the project, entered with a single command.

#### Scenario: Entering the environment
- **WHEN** a contributor enters the development environment
- **THEN** the compiler, test runner, and project tooling are available at their pinned versions

#### Scenario: Environment matches the build
- **WHEN** a contributor runs the project's checks inside the development environment
- **THEN** the results match those produced by the packaged build

### Requirement: Project checks are runnable as one command

The project SHALL expose a single command that runs its full check suite — the conformance fixtures, the unit tests, and the formatting check — so that contributors and continuous integration verify the same thing.

#### Scenario: Full check suite
- **WHEN** the check command runs on a healthy revision
- **THEN** conformance fixtures, tests, and the formatting check all pass and the command exits zero

#### Scenario: Conformance regression
- **WHEN** a code change causes a valid conformance fixture to be rejected or an invalid one to be accepted
- **THEN** the check command fails and identifies the fixture and the expected rule

#### Scenario: Continuous integration parity
- **WHEN** continuous integration runs
- **THEN** it invokes the same check command a contributor would run locally, rather than a separately maintained list of steps

### Requirement: The program is distributable as a standalone artifact

The toolchain SHALL be packageable as an executable artifact that runs without a source checkout and without the packaging tool present at runtime.

#### Scenario: Running the packaged program
- **GIVEN** a machine with only the packaged artifact
- **WHEN** a user runs the validate subcommand against a document
- **THEN** it works without a source checkout or a development environment

#### Scenario: Version reporting
- **WHEN** a user asks the program for its version
- **THEN** it reports a version identifying the revision it was built from, so a diagnostic can be traced to a specific build
