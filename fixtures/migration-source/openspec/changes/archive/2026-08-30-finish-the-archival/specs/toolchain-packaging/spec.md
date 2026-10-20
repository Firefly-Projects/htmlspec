## MODIFIED Requirements

### Requirement: The command surface names its operations as OpenSpec names them

The project SHALL provide one command-line program offering, at minimum, a subcommand to validate documents, a subcommand to format documents, a subcommand to bootstrap a project, and a subcommand to create documents. Each subcommand SHALL be usable without configuration in a conforming project.

**Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it.** This holds for the subcommand and for the options that select what it acts on. Every plausible adopter arrives from OpenSpec — HTMLSpec is a spinoff and its README says so — and a contributor who has learned one surface SHALL NOT have to learn a second spelling for the same operation. An operation OpenSpec does not provide takes the name this project chooses for it.

The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, `instructions`, and `store` name the operations OpenSpec also provides. `format` and `assets` name operations it does not. The selectors `--json`, `--specs`, `--change`, and `--store` carry OpenSpec's spellings for the same reason.

Fixing a spelling in the contract is not decoration. A requirement that describes an operation by role alone — *an operation that enumerates a project's active changes* — is satisfied by a subcommand called `changes`, or `ls`, and the instructions written against it would then name a command that does not exist. The workflow instructions this project installs invoke these commands by name, so the name is part of the contract they depend on.

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

#### Scenario: An operation OpenSpec also provides
- **WHEN** the command surface offers an operation that enumerates a project's changes, reports a change's completion, emits workflow instructions, or registers a standalone project
- **THEN** it is named `list`, `status`, `instructions`, and `store` respectively, because those are OpenSpec's names for those operations

#### Scenario: The selectors carry OpenSpec's spellings
- **WHEN** a contributor asks for structured output, for specifications rather than changes, for one named change, or for a registered standalone project
- **THEN** the options are `--json`, `--specs`, `--change`, and `--store`

#### Scenario: An operation OpenSpec does not provide
- **WHEN** the command surface offers an operation with no OpenSpec counterpart
- **THEN** it takes the name this project chooses for it, as `format` and `assets` do

#### Scenario: A renamed operation breaks the installed instructions
- **GIVEN** workflow instructions that invoke a subcommand by name
- **WHEN** that subcommand is spelled differently from what the instructions name
- **THEN** the instructions name a command that does not exist, which is why the spellings are fixed in the contract rather than left to an implementer


### Requirement: Archival produces the specification and updates the accepted contract

The command surface SHALL provide an archival operation that produces a change's specification artifact from its proposal, design, and tasks, validates it, and then applies its contract changes to the project's accepted specifications.

There SHALL be one specification artifact per change, and it SHALL be the same file as that change's as-built record. The project's accepted specifications SHALL be a separate destination outside the change directory.

The production SHALL be performed by an agent following stated instructions rather than by a deterministic extractor in the program. The program's part SHALL be the checking: it validates the produced document and reports what is wrong with it. That division is deliberate — the judgment of what a design's prose obliges is not derivable by rule, while whether the result conforms is exactly what a validator determines.

The project SHALL NOT ship a command that parses a proposal or a design and emits requirements. Those kinds carry no imposed structure, so there is nothing for such a command to parse reliably, and one written anyway would produce a specification that passes every check and misstates what the change established.

Archival SHALL NOT complete while the produced specification reports an error. A change archived with an invalid specification would leave the accepted contract in a state the project's own tools reject.

The archival command SHALL complete the operation it is named for. On success the change directory SHALL have been moved into the project's archive, and the change SHALL have recorded the version the project declared. An archival that reports success while the change remains among the project's active changes SHALL NOT be considered to have archived it.

Archival SHALL be atomic with respect to the project. Where archival refuses, or where a contributor declines to proceed, the project SHALL be left exactly as archival found it: no change moved, no accepted specification altered, and no version recorded.

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

#### Scenario: A completed archival
- **WHEN** archival succeeds for a change
- **THEN** the change directory is in the archive, it records the declared version, and it is no longer among the active changes

#### Scenario: A refused archival leaves nothing behind
- **WHEN** archival refuses
- **THEN** the change remains among the active changes, unmoved and unmodified

## ADDED Requirements

### Requirement: Archival refuses on the contract and asks about the work

Archival SHALL distinguish conditions that refuse from conditions that inform.

Archival SHALL refuse, without offering to proceed, when the declared version does not advance, when the change has no specification artifact, when that artifact declares a kind other than specification, or when validating it reports an error. These concern the contract, which archival makes permanent and against which the rest of the project is checked.

Archival SHALL report, and SHALL offer to proceed, when a change has incomplete tasks, or when a requirement stated in the change's specification appears in no accepted specification. These concern the work. A contributor SHALL be able to archive over both, because a dropped task is a legitimate outcome and a withdrawn requirement is legitimately absent from the accepted contract.

A report that offers to proceed SHALL name what it found rather than cautioning generally. It SHALL state the count of incomplete tasks and SHALL name each requirement identifier it did not locate.

#### Scenario: An invalid specification
- **WHEN** the change's specification reports an error
- **THEN** archival refuses and does not offer to proceed

#### Scenario: Incomplete tasks
- **WHEN** a change has incomplete tasks and is otherwise ready
- **THEN** archival reports how many and offers to proceed

#### Scenario: A requirement that reached no accepted specification
- **WHEN** the change's specification states a requirement that appears in no accepted specification
- **THEN** archival names that requirement's identifier and offers to proceed

#### Scenario: Declining the offer
- **WHEN** a contributor declines to proceed
- **THEN** nothing is moved and the project is unchanged

### Requirement: Archival assesses the merge and never performs it

Archival SHALL determine whether the change's contract reached the accepted specifications, and SHALL do so by comparing requirement identifiers rather than by reading requirements for meaning.

Archival SHALL NOT edit an accepted specification, SHALL NOT propose the text of a merge, and SHALL NOT treat an absent identifier as an error. Whether an absent requirement was withdrawn deliberately or omitted by mistake is a judgment, and `artifact-kinds` reserves it for an agent.

#### Scenario: The assessment is a comparison of identifiers
- **WHEN** archival assesses whether a merge occurred
- **THEN** it compares the identifiers the change's specification states against those the accepted specifications hold, and reads neither for meaning

#### Scenario: Archival does not write the accepted specifications
- **WHEN** archival completes
- **THEN** no accepted specification was modified by the program

### Requirement: An archived change is named for its date and refuses to collide

Archival SHALL name the archived directory for the date of archival followed by the change's name. Where the change's name already begins with a date, archival SHALL NOT prepend a second one.

Where the target name already exists, archival SHALL refuse and SHALL name the existing archive. It SHALL NOT write into it, merge with it, overwrite it, or derive an alternative name. A collision means the change was already archived, or that two changes share a name on one day, and both want a contributor.

#### Scenario: A change whose name carries a date
- **WHEN** a change already named with a leading date is archived
- **THEN** the archived directory carries one date rather than two

#### Scenario: An occupied target
- **WHEN** the archive already holds a directory of the target name
- **THEN** archival refuses, names the existing archive, and moves nothing

### Requirement: Archival is drivable without a person present

The offer to proceed SHALL be presented only where the operation can receive an answer. Where archival runs without an interactive stream, it SHALL NOT wait for one.

Archival SHALL accept an explicit instruction to proceed over its reports, so that an operation that would otherwise ask can be driven to completion by an agent. Absent that instruction, an archival carrying reports that would have been offered SHALL exit with a failing status having moved nothing.

Under structured output archival SHALL emit its reports and SHALL state whether the change was moved, so that a caller determines the outcome from the payload rather than from the exit status alone.

#### Scenario: A warned run with no instruction to proceed
- **GIVEN** a change with incomplete tasks
- **WHEN** archival runs without an interactive stream and without an instruction to proceed
- **THEN** it exits with a failing status and moves nothing

#### Scenario: A warned run instructed to proceed
- **GIVEN** the same change
- **WHEN** archival runs with an explicit instruction to proceed
- **THEN** it archives the change and reports what it archived over

#### Scenario: The structured payload states the outcome
- **WHEN** archival emits structured output
- **THEN** the payload carries its reports and states whether the change was moved

### Requirement: Archival supports a mode that checks without performing

The archival operation SHALL support a mode that reports whether a change is ready to be archived without moving it, without recording a version, and without altering the project in any way.

The mode SHALL apply every condition archival applies: the version comparison, the presence of the specification artifact, its declared kind, and its validation. It SHALL also report what archival would report at its offer to proceed, so that a contributor learns before archiving what they would otherwise learn during it.

The mode SHALL exit with a failing status exactly where archival would refuse, so that it serves as a continuous-integration gate. Where archival would proceed, or would offer to proceed, the mode SHALL exit successfully and SHALL distinguish the two in what it reports. A condition archival treats as a question about the work is not a condition this mode fails on, because the two would otherwise disagree about what readiness means.

This is the form the project already uses where one operation both performs and checks: the format subcommand states the same obligation, and a mode rather than a second command is what keeps the conditions from drifting apart. An operation that changed from checking to performing without offering this would take away a contributor's ability to ask the question, leaving finding out by doing as the only way to answer it.

#### Scenario: Checking a change that is ready
- **WHEN** the mode runs against a change archival would archive
- **THEN** it reports the change as ready, exits successfully, and the change remains among the active changes

#### Scenario: Checking a change archival would refuse
- **WHEN** the mode runs against a change whose specification reports an error
- **THEN** it exits with a failing status and reports the same refusal archival would

#### Scenario: Checking a change archival would ask about
- **WHEN** the mode runs against a change with incomplete tasks and no other finding
- **THEN** it exits successfully, and reports that archival would ask rather than reporting the change as ready without qualification

#### Scenario: The mode writes nothing
- **WHEN** the mode runs, whatever it reports
- **THEN** the project is unchanged: no change moved, no version recorded, and no accepted specification altered

### Requirement: The command surface reports what a project holds

The command surface SHALL provide an operation that enumerates a project's active changes, and SHALL be able to enumerate its accepted specifications instead. The enumeration SHALL be available as a structured payload.

This is the operation that lets a workflow select what it acts on. A workflow that cannot enumerate a project's changes must have its subject named for it every time, and instructions written against such a surface omit the selection step rather than performing it.

#### Scenario: Enumerating active changes
- **WHEN** a project's changes are enumerated
- **THEN** each active change is named, and archived changes are not among them

#### Scenario: Enumerating accepted specifications
- **WHEN** a project's accepted specifications are enumerated
- **THEN** each is named

#### Scenario: A structured enumeration
- **WHEN** structured output is requested
- **THEN** the enumeration is emitted as a payload a workflow can select from

### Requirement: The command surface reports a change's completion

The command surface SHALL provide an operation that reports, for one change, which of its artifacts are present and how many of its tasks are complete. The report SHALL be available as a structured payload.

Archival SHALL derive its report about incomplete work from this same operation, so that a contributor who checks a change before archiving it and the archival that follows cannot disagree about what the change holds.

#### Scenario: A change mid-implementation
- **WHEN** completion is reported for a change with a proposal, a design, tasks, and no specification
- **THEN** the report names the specification as absent and states how many tasks remain

#### Scenario: Archival and the report agree
- **WHEN** archival reports incomplete work for a change
- **THEN** the count it reports is the count the completion operation reports for that change

### Requirement: The workflow instructions are available from the command surface

The command surface SHALL emit the workflow instructions for a named stage on demand, as the same text the bootstrap operation installs.

An agent obtains the instructions without knowing which tool's directory holds them, and without the risk of reading a copy installed by an earlier version of the toolchain. A stage's instructions and the installed copy SHALL come from one source, so the two cannot state different things.

#### Scenario: Instructions for a stage
- **WHEN** the instructions for archival are requested
- **THEN** the workflow instructions for archival are emitted

#### Scenario: One source
- **WHEN** the emitted instructions and the installed instructions for one stage are compared
- **THEN** they are the same text

### Requirement: Specifications can live in a repository other than the one being worked on

The command surface SHALL allow an HTMLSpec project held in a standalone repository to be registered on a machine under an identifier, and SHALL allow the operations that read and write specifications and changes to act on a registered project instead of the one containing the working directory.

A registration SHALL be machine-level rather than part of any project, because a project that recorded where other projects live would make one repository's contents depend on another's location.

Where a selection names no registered project, operations SHALL act on the project containing the working directory. Where a registration names a location that no longer holds a project, operations SHALL report that registration as unresolvable and SHALL NOT fall back to the working directory, because silently acting on a different project than the one named is the failure this reports.

#### Scenario: Acting on a registered project
- **WHEN** an operation is directed at a registered project
- **THEN** it reads and writes that project's specifications and changes rather than those of the project containing the working directory

#### Scenario: No selection
- **WHEN** an operation names no registered project
- **THEN** it acts on the project containing the working directory

#### Scenario: A registration whose project is gone
- **WHEN** an operation is directed at a registration whose location no longer holds a project
- **THEN** it reports the registration as unresolvable and acts on nothing

## RENAMED Requirements

- FROM: `### Requirement: A single command exposes validate, format, bootstrap, and create`
- TO: `### Requirement: The command surface names its operations as OpenSpec names them`
