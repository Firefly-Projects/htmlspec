## ADDED Requirements

### Requirement: The command surface reports whether a change reached the accepted contract

The command surface SHALL provide an operation that reports, for one change, whether the requirements its specification states have reached the project's accepted specifications. The report SHALL be available as a structured payload.

The operation SHALL determine this by comparing requirement identifiers, and SHALL read neither the change's specification nor the accepted specifications for meaning. This is the same comparison archival makes, and both SHALL derive it from one source, so that a contributor who runs the operation and the archival that follows cannot disagree about which requirements reached the contract.

The report SHALL distinguish three sets: the requirement identifiers the change states, those the accepted specifications already hold, and those absent from them. The third is derivable from the first two and SHALL be reported anyway, because it is the answer the operation exists to give.

**The operation SHALL NOT modify an accepted specification, and SHALL NOT propose the text of a merge.** Deciding which accepted requirement a scenario belongs under is judgment, and `artifact-kinds` reserves it for an agent. This prohibition is stated rather than left implied: the operation carries OpenSpec's name for it, that name reads like an operation that writes, and a reader who assumes it does would be assuming the program edits the project's contract unattended.

An absent requirement SHALL be a finding and not an error, and the operation SHALL succeed while any remain. A change mid-implementation legitimately states requirements that have not reached the contract, and a requirement withdrawn deliberately is absent permanently; an operation that failed on either would fail on the normal state of the work it is meant to be run during.

The operation SHALL refuse on a change that cannot be resolved, on an absent specification artifact, and on a specification that does not validate. A comparison made against an invalid contract has no meaning, and reporting it as though it did is worse than declining to report.

#### Scenario: A change whose contract has not been applied
- **GIVEN** a change whose specification states a requirement that no accepted specification holds
- **WHEN** the operation is run against that change
- **THEN** the requirement is named as absent, the operation succeeds, and no accepted specification is modified

#### Scenario: A change whose contract has been applied
- **GIVEN** a change every one of whose stated requirements appears in an accepted specification
- **WHEN** the operation is run against that change
- **THEN** the absent set is empty

#### Scenario: The comparison is of identifiers
- **WHEN** the operation determines whether a requirement reached the contract
- **THEN** it compares identifiers, and reads neither the change's specification nor the accepted specifications for meaning

#### Scenario: The operation and archival agree
- **WHEN** archival reports that a change states requirements no accepted specification holds
- **THEN** the identifiers it names are the identifiers the operation reports as absent for that change

#### Scenario: A structured report
- **WHEN** structured output is requested
- **THEN** the stated, accepted, and absent identifier sets are emitted as a payload

#### Scenario: An invalid specification
- **GIVEN** a change whose specification does not validate
- **WHEN** the operation is run against it
- **THEN** it refuses and reports why, and reports no comparison

#### Scenario: The accepted specifications are never written
- **WHEN** the operation completes, for any change and any outcome
- **THEN** no accepted specification was modified by the program

### Requirement: The workflow instructions carry a stage for applying a change's contract

The workflow instructions SHALL carry a stage for merging a change's stated requirements into the project's accepted specifications, available from the command surface as every other stage is and installed by the bootstrap operation from the same source.

The stage's instructions SHALL state that the merge is the agent's work: that both the change's specification and the accepted specifications are read for meaning before anything is written, that a change's specification is never copied into an accepted specification wholesale, and that the reporting operation is run again afterwards to confirm the absent set is empty.

The instructions SHALL state that a requirement withdrawn deliberately is left absent and reported to the contributor as withdrawn, rather than merged to empty the report.

The obligations on the *program* live in the requirement above. This one governs the text an agent reads, because the half of the operation that requires judgment cannot be stated as a rule the program enforces.

#### Scenario: The stage is available
- **WHEN** the workflow instructions for the stage are requested from the command surface
- **THEN** they are emitted, as the same text the bootstrap operation installs

#### Scenario: The stage is installed
- **WHEN** a project is bootstrapped
- **THEN** the stage's instructions are installed for each supported tool alongside the other stages

## MODIFIED Requirements

### Requirement: The command surface names its operations as OpenSpec names them

The project SHALL provide one command-line program offering, at minimum, a subcommand to validate documents, a subcommand to format documents, a subcommand to bootstrap a project, and a subcommand to create documents. Each subcommand SHALL be usable without configuration in a conforming project.

**Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it.** This holds for the subcommand and for the options that select what it acts on. Every plausible adopter arrives from OpenSpec — HTMLSpec is a spinoff and its README says so — and a contributor who has learned one surface SHALL NOT have to learn a second spelling for the same operation. An operation OpenSpec does not provide takes the name this project chooses for it.

The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, `sync`, `instructions`, and `store` name the operations OpenSpec also provides. `format` and `assets` name operations it does not. The selectors `--json`, `--specs`, `--change`, and `--store` carry OpenSpec's spellings for the same reason.

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
