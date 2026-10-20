## Purpose

Defines the command-line surface contributors and agents use to work with HTMLSpec documents, and the reproducible packaging that ensures everyone — including continuous integration — builds and runs the same toolchain.

## Requirements

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

The project SHALL expose a single command that runs its full check suite — the conformance fixtures, the unit tests, the formatting check, and the project's release hygiene — so that contributors and continuous integration verify the same thing.

Every stage of the suite SHALL read the working tree rather than a repository's history. The build sandbox the suite runs in receives a filtered source snapshot with no repository, so a stage using version control history would pass on a contributor's machine and fail in continuous integration — the divergence a single shared suite exists to prevent. This SHALL constrain every stage the suite gains, not only the ones it has.

Every file a stage reads SHALL be present in that snapshot, which requires both that the source filter admit it and that version control track it. An untracked file and a file the filter excludes are indistinguishable from an absent one inside the sandbox, and both produce a stage that passes locally and fails in continuous integration. A stage that reads a file whose name the filter does not already admit SHALL be accompanied by the filter change that admits it, in the same change.

A stage SHALL either pass or fail, because the suite has no advisory level. A rule whose violation is a legitimate state for work in progress SHALL therefore be stated as a property of a settled record rather than of work under way, so that the rule has a correct answer at every moment. A stage that would report on unfinished work has no way to do so without failing, and a stage that fails throughout a change's life is one contributors learn to disregard.

A stage's name SHALL be distinct from every other stage's, so that a reported failure names one stage unambiguously.

#### Scenario: Full check suite
- **WHEN** the check command runs on a healthy revision
- **THEN** conformance fixtures, tests, the formatting check, and the release check all pass and the command exits zero

#### Scenario: Conformance regression
- **WHEN** a code change causes a valid conformance fixture to be rejected or an invalid one to be accepted
- **THEN** the check command fails and identifies the fixture and the expected rule

#### Scenario: Continuous integration parity
- **WHEN** continuous integration runs
- **THEN** it invokes the same check command a contributor would run locally, rather than a separately maintained list of steps

#### Scenario: A stage that would need history
- **WHEN** a stage would answer its question by reading commits or diffs
- **THEN** it is stated against the working tree instead, because the sandbox has no repository to read

#### Scenario: A rule that cannot fail cleanly on work in progress
- **WHEN** a rule's violation is a legitimate state part-way through a change
- **THEN** the stage asks it of a settled record rather than of the working change, since the suite offers no way to report without failing

#### Scenario: A file the source filter excludes
- **WHEN** a stage reads a file whose name the source filter does not admit
- **THEN** the stage fails in the sandbox while passing locally, so the filter change that admits the file belongs to the same change as the stage

### Requirement: The program is distributable as a standalone artifact

The toolchain SHALL be packageable as an executable artifact that runs without a source checkout and without the packaging tool present at runtime.

#### Scenario: Running the packaged program
- **GIVEN** a machine with only the packaged artifact
- **WHEN** a user runs the validate subcommand against a document
- **THEN** it works without a source checkout or a development environment

#### Scenario: Version reporting
- **WHEN** a user asks the program for its version
- **THEN** it reports a version identifying the revision it was built from, so a diagnostic can be traced to a specific build

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

### Requirement: The command surface names its operations as OpenSpec names them

The project SHALL provide one command-line program offering, at minimum, a subcommand to validate documents, a subcommand to format documents, a subcommand to bootstrap a project, and a subcommand to create documents. Each subcommand SHALL be usable without configuration in a conforming project.

**Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it.** This holds for the subcommand and for the options that select what it acts on. Every plausible adopter arrives from OpenSpec — HTMLSpec is a spinoff and its README says so — and a contributor who has learned one surface SHALL NOT have to learn a second spelling for the same operation. An operation OpenSpec does not provide takes the name this project chooses for it.

The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, `sync`, `show`, `instructions`, and `store` name the operations OpenSpec also provides. `format` and `assets` name operations it does not. The selectors `--json`, `--specs`, `--change`, `--store`, and `--diff` carry OpenSpec's spellings for the same reason.

Fixing a spelling in the contract is not decoration. A requirement that describes an operation by role alone — *an operation that enumerates a project's active changes* — is satisfied by a subcommand called `changes`, or `ls`, and the instructions written against it would then name a command that does not exist. The workflow instructions this project installs invoke these commands by name, so the name is part of the contract they depend on.

**Each operation SHALL also answer to a request-verb spelling, which says what the operation does to the files it acts on.** The verb is one of htmx's five request keywords without their `hx-` prefix, `get`, `post`, `put`, `patch`, and `delete`, and it carries the meaning HTTP gives it: `get` for an operation that never writes, `post` for one that creates something that was not there, `put` for one that creates or replaces a whole thing and leaves it as one run did when run again, `patch` for one that changes something that exists, and `delete` for one that removes something. The verb is followed by the operation's object. These spellings are fixed as the OpenSpec spellings are: `get validity`, `get list`, `get status`, `get sync`, `get show`, `get assets`, `get preflight`, `get instructions`, `get registrations`, and `get survey`; `post change`, `post document`, `post registration`, `post run`, and `post promotion`; `put project` and `put format`; `patch archive`; and `delete registration`. The verb names the most the operation writes in any invocation, so `get` spells only an operation that never writes, and an operation that writes only with an option takes the verb of that write. An option keeps its meaning under both spellings: `put format --check` writes nothing, as `format --check` does.

**A request-verb spelling SHALL be a second spelling of an operation and never its replacement.** It SHALL run the operation its OpenSpec spelling runs, with the same arguments and options, and SHALL produce the same output and exit status. The OpenSpec spellings stay the ones this requirement fixes above, and the program's own messages and the workflow instructions it installs SHALL name them, so a contributor who arrives from OpenSpec still has one spelling to learn. A name says what an operation is about, and a verb says whether it can change the tree, which the names do not say and which differs between the two tools for two of them: OpenSpec's sync and archive write the accepted specifications, and this program's `sync` and `archive` never do. The verbs are vocabulary only. No operation sends a request, and none needs a server or a network.

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
- **THEN** only that document is validated and reported on, and the other documents its references can reach are read only for the identifiers they declare

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
- **WHEN** the command surface offers an operation that enumerates a project's changes, reports a change's completion, emits workflow instructions, registers a standalone project, or shows a change's modified requirements beside the accepted ones
- **THEN** it is named `list`, `status`, `instructions`, `store`, and `show` respectively, because those are OpenSpec's names for those operations

#### Scenario: The selectors carry OpenSpec's spellings
- **WHEN** a contributor asks for structured output, for specifications rather than changes, for one named change, for a registered standalone project, or for a change's modified requirements beside the accepted ones
- **THEN** the options are `--json`, `--specs`, `--change`, `--store`, and `--diff`

#### Scenario: An operation OpenSpec does not provide
- **WHEN** the command surface offers an operation with no OpenSpec counterpart
- **THEN** it takes the name this project chooses for it, as `format` and `assets` do

#### Scenario: A renamed operation breaks the installed instructions
- **GIVEN** workflow instructions that invoke a subcommand by name
- **WHEN** that subcommand is spelled differently from what the instructions name
- **THEN** the instructions name a command that does not exist, which is why the spellings are fixed in the contract rather than left to an implementer

#### Scenario: A request-verb spelling runs the same operation
- **WHEN** a contributor runs `htmlspec get validity` in a project with no arguments
- **THEN** every document in the project is validated and reported exactly as `htmlspec validate` reports them, with the same exit status

#### Scenario: A get operation writes nothing
- **WHEN** a contributor runs any operation through a `get` spelling, with any of the options it accepts
- **THEN** no file is created, changed, or removed

#### Scenario: An option keeps its meaning under the verb
- **WHEN** a contributor runs `htmlspec put format --check` on a document that is not canonical
- **THEN** nothing is written, and the report and exit status are those of `htmlspec format --check`

#### Scenario: The program names the OpenSpec spelling
- **WHEN** the program tells a contributor which operation to run, as it does when no project is found
- **THEN** it names the OpenSpec spelling, such as `init`, and not the request-verb spelling

#### Scenario: Every operation has both spellings
- **WHEN** a contributor reads the program's help
- **THEN** every operation it lists has one request-verb spelling, and every request-verb spelling runs exactly one operation it lists

### Requirement: An operation over a directory collects documents and not a change's evidence

When `validate` or `format`, in either its writing or its checking mode, is given a directory or no path at all, it SHALL operate on every `.html` file beneath that directory, except files beneath a change's evidence as artifact-kinds' "A change's evidence is kept in the change and is not one of its documents" places it. Every other operation that collects a project's documents by walking a directory SHALL make the same exclusion from the same definition.

**A path the contributor names SHALL be checked as named.** A file means that file. A directory that is itself a change's evidence directory, or lies beneath one, means that directory, and `validate` and `format --check` SHALL check every `.html` file beneath it. The exclusion applies to what a walk comes across and never to what a contributor asked for. A contributor who wants to know whether a record still conforms can ask, and the answer is the ordinary one.

**Checking is not writing.** `format` in its writing mode SHALL NOT write a file beneath a change's evidence, including one a contributor names, because artifact-kinds states that no operation rewrites one. Given such a path, it SHALL name each record, write nothing at all, and exit with the status for an operation that could not run. It SHALL NOT write the other documents and skip the records. A partial write would report success for a request it half ignored.

Where the walked directory is not within a project, no directory is a change's evidence, and every `.html` file beneath it SHALL be operated on. A conformance fixture and a loose directory of documents keep the behavior they have.

The exclusion SHALL be defined once and reached by every such operation. It SHALL NOT be restated per command. A second definition that drifts from the first would have `validate` and `format` disagree about which files are documents. That is the fault this requirement exists to remove, arriving by another route.

#### Scenario: Validating a project that holds HTML evidence
- **GIVEN** a project whose documents all conform, and a change whose evidence holds an HTML file that does not
- **WHEN** a contributor runs `validate` with no path
- **THEN** it succeeds and reports no diagnostic naming the evidence file

#### Scenario: Formatting leaves evidence untouched
- **GIVEN** a change whose evidence holds an HTML file that is not in canonical form
- **WHEN** a contributor runs `format` over the project, and then `format --check`
- **THEN** the evidence file is byte-identical to what it was, and the check succeeds

#### Scenario: Naming an evidence file checks it
- **GIVEN** a change whose evidence holds an HTML file that does not conform
- **WHEN** a contributor runs `validate` naming that file
- **THEN** the file is validated and its diagnostics are reported

#### Scenario: Naming the evidence directory checks what it holds
- **GIVEN** a change whose evidence holds an HTML file that does not conform
- **WHEN** a contributor runs `validate` naming the change's evidence directory
- **THEN** the file is validated and its diagnostics are reported

#### Scenario: Formatting a named record is refused
- **GIVEN** a change whose evidence holds an HTML file that is not in canonical form
- **WHEN** a contributor runs `format` naming that file, or the evidence directory holding it
- **THEN** nothing is written, each record is named, and the exit is the one for an operation that could not run, while `format --check` naming the same path still reports the file

#### Scenario: Outside a project every file is a document
- **GIVEN** a directory that is not within a project, holding `evidence/a-record.html`
- **WHEN** a contributor runs `validate` naming that directory
- **THEN** the file is validated, because outside a project there is no change for it to be the evidence of

### Requirement: The project records what changed in each version it ships

The project SHALL keep a changelog with one entry per version it has shipped, and each entry's heading SHALL name its version so that a check can locate it.

An entry's prose SHALL remain the author's. A check SHALL establish that an entry for a version exists and SHALL NOT impose a structure on what the entry says, because a changelog exists to be read by a person.

An entry reconstructed after the fact SHALL say that it is a reconstruction. A history assembled from records by someone who was not present is more useful labelled than unlabelled, and more honest than a confident account.

This obligation is on this repository. It SHALL be distinguishable from any obligation the program places on a project it bootstraps, and satisfying one SHALL NOT be taken to satisfy the other.

#### Scenario: A version with no entry
- **WHEN** the project has shipped a version the changelog does not name
- **THEN** the release check fails and names that version

#### Scenario: An entry's contents
- **WHEN** the release check locates an entry for a version
- **THEN** it reads the entry's presence and applies no rule to the entry's prose

#### Scenario: A reconstructed entry
- **WHEN** an entry is written after the fact from archived records
- **THEN** the entry states that it is a reconstruction

### Requirement: An archived change records the version it shipped in

Each archived change SHALL record the version the project declared when it was archived, in a form the project's own suite can read without version control history.

Recorded versions SHALL be distinct, so that each release names the change that produced it. Where the project has archived several changes under one version before this obligation existed, the changelog SHALL carry a single baseline entry naming them, and the suite SHALL treat that baseline as one release. The baseline SHALL cease to apply once the version advances past it, so it is a statement about a known past rather than a permanent exception.

Versions SHALL NOT be invented to satisfy distinctness. Assigning a separate version to each of several changes that shipped together records releases that never existed, which is worse than recording the shared version honestly.

#### Scenario: An archive with no recorded version
- **WHEN** an archived change records no version
- **THEN** the release check fails and names that change

#### Scenario: Two archives claiming one version
- **WHEN** two archived changes above the baseline record the same version
- **THEN** the release check fails, because a version names the change that produced it

#### Scenario: The baseline
- **WHEN** several changes were archived under one version before this obligation existed
- **THEN** one changelog entry names them all and the suite treats them as one release

#### Scenario: The baseline retires
- **WHEN** the project's version advances past the baseline
- **THEN** every subsequent change is checked for a distinct version of its own

### Requirement: Archival refuses on the contract and asks about the work

Archival SHALL distinguish conditions that refuse from conditions that inform.

Archival SHALL refuse, without offering to proceed, when the declared version does not advance, when the change has no specification artifact, when that artifact declares a kind other than specification, or when validating it reports an error. These concern the contract, which archival makes permanent and against which the rest of the project is checked.

Archival SHALL report, and SHALL offer to proceed, when a change has incomplete tasks, or when the change's contract has not reached the accepted specifications as "The command surface reports whether a change reached the accepted contract" judges it: a requirement the change adds, modifies, or carries forward that appears in no accepted specification, or a rename, a requirement's content, or a removal that the accepted specifications have not taken. These concern the work. A contributor SHALL be able to archive over both, because a dropped task is a legitimate outcome, and whether a part of the contract that did not arrive was left out on purpose is a judgment `artifact-kinds` reserves for an agent.

A report that offers to proceed SHALL name what it found rather than cautioning generally. It SHALL state the count of incomplete tasks and SHALL name each requirement identifier it did not locate, and each requirement whose rename, content, or removal the accepted specifications have not taken.

**A requirement the change marks `removed` that no accepted specification holds SHALL NOT be reported.** The change withdraws it, and its absence is the merge having done what the change asked. Reported as a requirement archival did not locate, it asks a contributor to archive over a withdrawal that was applied correctly, and says the change has not been synced when it has.

#### Scenario: An invalid specification
- **WHEN** the change's specification reports an error
- **THEN** archival refuses and does not offer to proceed

#### Scenario: Incomplete tasks
- **WHEN** a change has incomplete tasks and is otherwise ready
- **THEN** archival reports how many and offers to proceed

#### Scenario: A requirement that reached no accepted specification
- **WHEN** the change's specification states a requirement it adds, modifies, or carries forward, and that requirement appears in no accepted specification
- **THEN** archival names that requirement's identifier and offers to proceed

#### Scenario: Declining the offer
- **WHEN** a contributor declines to proceed
- **THEN** nothing is moved and the project is unchanged

#### Scenario: A requirement whose content did not arrive
- **WHEN** an accepted specification holds a requirement the change modifies, and its statement lacks a word the change's specification adds
- **THEN** archival names that requirement's identifier and offers to proceed

#### Scenario: A withdrawal the merge applied
- **GIVEN** a change whose specification marks a requirement `removed`
- **WHEN** no accepted specification holds that requirement
- **THEN** archival reports nothing about it and does not ask on its account

### Requirement: Archival assesses the merge and never performs it

Archival SHALL determine whether the change's contract reached the accepted specifications, and SHALL do so by comparing requirement identifiers, for a renamed requirement its heading, and for a requirement the change adds or modifies its statements and scenarios as text, rather than by reading requirements for meaning. This SHALL be the comparison the reporting operation makes, derived from one source, including its treatment of a rename, a requirement's content, and a removal that the accepted specifications have not taken.

Archival SHALL NOT edit an accepted specification, SHALL NOT propose the text of a merge, and SHALL NOT treat an absent identifier, or any other part of the contract that has not arrived, as an error. Whether an absent requirement was abandoned deliberately or omitted by mistake is a judgment, and `artifact-kinds` reserves it for an agent. A requirement the change marks `removed` is not such a case: the change states the withdrawal, and its absence is what reaching the contract means for it.

**Where the assessment finds the contract has not reached the accepted specifications, archival SHALL report that the change has not been synced and SHALL name the operation that reports what remains.** The report SHALL name the operation as it is invoked, rather than describing it. The prior wording told a contributor to apply the change's contract at a time when no operation did so, and a report that names a step without naming the command behind it leaves the reader searching the command surface for a name they must guess.

Where archival asks whether to proceed over its findings, and the change has not been synced, **the question SHALL say so and SHALL state that declining leaves the project unchanged.** A question that asks only whether to archive anyway invites the answer that archives a change whose contract never reached the project's specifications; a contributor deciding that should be told what proceeding skips and that declining costs them nothing.

The structured payload SHALL state whether the change has not been synced, as a value rather than as something derivable from the reported identifiers. An agent driving archival cannot answer the question a contributor is asked, so the condition that question is about SHALL be readable without inference.

Archival SHALL NOT perform the merge it names, and the report SHALL NOT be taken as authority to. This is the same prohibition stated above, restated because naming an operation is where a reader would expect an offer to run it.

Where the assessment finds the contract has reached the accepted specifications, archival SHALL NOT report anything about syncing. A report that appears whether or not it applies is one a reader learns to pass over, and the occasion it mattered would be passed over with it.

#### Scenario: The assessment is a comparison of identifiers
- **WHEN** archival assesses whether a merge occurred
- **THEN** it compares the identifiers the change's specification states against those the accepted specifications hold, the headings of a stated rename, and the text of each requirement the change adds or modifies, and reads neither for meaning

#### Scenario: Archival does not write the accepted specifications
- **WHEN** archival completes
- **THEN** no accepted specification was modified by the program

#### Scenario: A change whose contract has not been synced
- **GIVEN** a change stating a requirement it does not mark `removed`, which no accepted specification holds
- **WHEN** archival reports its findings
- **THEN** it reports that the change has not been synced and names the operation that reports what remains, as it is invoked

#### Scenario: The question asked about an unsynced change
- **GIVEN** a change whose contract has not reached the accepted specifications
- **WHEN** archival asks whether to proceed over its findings
- **THEN** the question states that the change has not been synced and that declining leaves the project unchanged

#### Scenario: A change whose contract was applied
- **GIVEN** a change every one of whose stated requirements has reached the accepted specifications, as the reporting operation judges it
- **WHEN** archival reports its findings
- **THEN** nothing about syncing is reported

#### Scenario: An agent reads the condition rather than inferring it
- **WHEN** archival emits its structured payload
- **THEN** whether the change has not been synced is stated as a value in the payload

#### Scenario: A change whose only unapplied work is a rename
- **GIVEN** a change every one of whose stated identifiers an accepted specification holds, one of which the change renames and the accepted specification still holds under the previous heading
- **WHEN** archival reports its findings
- **THEN** it reports that the change has not been synced, names the untaken rename, and names the operation that reports what remains

#### Scenario: A change whose only unapplied work is a word
- **GIVEN** a change every one of whose stated identifiers an accepted specification holds, one of which the change modifies by adding a word to a statement that the accepted requirement does not carry
- **WHEN** archival reports its findings
- **THEN** it reports that the change has not been synced, names the requirement, and names the operation that reports what remains

#### Scenario: A change whose only unapplied work is a withdrawal
- **GIVEN** a change whose contract has otherwise reached the accepted specifications, and which marks `removed` a requirement that an accepted specification still holds
- **WHEN** archival reports its findings
- **THEN** it reports that the change has not been synced, names the requirement, and names the operation that reports what remains

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

**Because it exists for selection, the enumeration of active changes SHALL carry how much of each change's work is complete.** A list of names is not something a workflow or a contributor can select from on any basis other than order, and a selection made at the moment of archiving is made about a directory that is about to move and a contract that is about to be rewritten.

The completion carried SHALL be a count of the change's tasks in each state the contract defines, rather than a single fraction. A fraction cannot separate a change nobody has started from one whose every task is under way, nor a change whose outstanding tasks were abandoned from one whose outstanding tasks remain to be done; those are different situations and a selection is made between them.

**Beside those counts, the enumeration SHALL carry how many of the change's `done` tasks state a deviation**, as document-schema's "A task declares completion state" defines one: the tasks done another way than their words say. Such a task SHALL be counted as `done`, in the count of that state and in any fraction, and SHALL NOT be subtracted from either. Its work happened, and a report that left it out would show finished work as outstanding. The count SHALL be carried apart from the per-state counts, and those SHALL still account for every task exactly once, because a deviation is not a state.

The count exists because a count of states cannot show it. A change that did every task as its words say and a change that did half of them another way both read as finished, and a reviewer choosing which change to check, or what to read before archiving one, is choosing between exactly those two.

Only a task whose state is `done` SHALL be counted as done differently. A task that declares another state and carries a deviation is a document that does not satisfy the schema, and counting it as done differently would attribute to it a state it never declared. This is the reading this requirement already takes for a task that declares no state.

Where the enumeration renders a change's completion for people as a fraction, the numerator SHALL remain every `done` task and the denominator every task, as they are without this count. The count of tasks done differently SHALL be shown in the change's row where it is not zero, and SHALL NOT be shown where it is zero. Most changes do every task as written, and a figure that reads zero on nearly every row teaches a reader to pass over it.

The counts, including the count of tasks done differently, SHALL be derived from the same source as the operation that reports a change's completion, so that the enumeration, that operation, and archival cannot disagree about one change. A second traversal that recounted the same tasks would be a second answer to one question, and the disagreement would be invisible to a test written against either operation alone.

A task that declares no state SHALL be counted apart from the states the contract defines, and SHALL be counted as outstanding wherever outstanding work is reported. The document schema requires a state, so such a task is a document that does not satisfy it; the counts are read from projections rather than from validated documents, and reporting an undeclared task under a state it never declared would attribute to a document something it does not say. Counting it as outstanding is the reading the report about incomplete work already takes, and the two SHALL NOT differ.

A change whose tasks document is absent SHALL be enumerated with its completion absent, rather than reported as a count of zero. A zero fraction reads as a change with no work outstanding, which is the opposite of what an unwritten tasks document means.

The enumeration of accepted specifications is unaffected by the above, which governs changes only.

The order of the enumeration SHALL NOT depend on the completion it carries. A listing that reorders itself as work progresses cannot be selected from twice.

**The enumeration of active changes SHALL also carry each change's stated predecessor and whether that predecessor is resolved.** The strongest reason not to select a change is that the change it follows is unfinished, and a listing that reports a change as ready to work on while its predecessor is open reports the opposite of what the selection needs.

The order of the enumeration SHALL NOT depend on the ordering it reports, for the same reason it does not depend on the completion it carries. A dependency order is additionally unstable in a way name order is not: a cycle has no such order at all.

#### Scenario: Enumerating active changes
- **WHEN** a project's changes are enumerated
- **THEN** each active change is named, and archived changes are not among them

#### Scenario: Enumerating accepted specifications
- **WHEN** a project's accepted specifications are enumerated
- **THEN** each is named

#### Scenario: A structured enumeration
- **WHEN** structured output is requested
- **THEN** the enumeration is emitted as a payload a workflow can select from

#### Scenario: A change's progress is visible in the listing
- **GIVEN** an active change some of whose tasks are done
- **WHEN** the project's changes are enumerated
- **THEN** the change is named and how much of its work is complete is reported beside it

#### Scenario: Two changes at the same fraction in different states
- **GIVEN** one change whose tasks are all `todo` and another the same size whose tasks are all `in-progress`
- **WHEN** the project's changes are enumerated
- **THEN** the two are distinguishable from the enumeration alone

#### Scenario: Outstanding work that was abandoned
- **GIVEN** a change whose remaining tasks are all `dropped`
- **WHEN** the project's changes are enumerated
- **THEN** it is distinguishable from a change whose remaining tasks are still to be done

#### Scenario: A change with a task done differently
- **GIVEN** an active change of four tasks, two of them `done`, one of which states a deviation
- **WHEN** the project's changes are enumerated
- **THEN** two of four tasks are reported done, and one task is reported as done differently, apart from the per-state counts

#### Scenario: A deviation leaves the per-state counts whole
- **GIVEN** an active change one of whose `done` tasks states a deviation
- **WHEN** the enumeration is requested as a structured payload
- **THEN** the per-state counts sum to the total, and the count of tasks done differently is carried apart from them

#### Scenario: A change whose every task was done as written
- **GIVEN** an active change none of whose tasks states a deviation
- **WHEN** the project's changes are enumerated for people
- **THEN** the change's row shows no count of tasks done differently, and the structured payload reports that count as zero

#### Scenario: A deviation on a task that is not done
- **GIVEN** an active change one of whose `todo` tasks carries a deviation
- **WHEN** the project's changes are enumerated
- **THEN** that task is counted as `todo`, and not as done differently

#### Scenario: A task that declares no state
- **GIVEN** an active change one of whose tasks declares no state
- **WHEN** the project's changes are enumerated
- **THEN** that task is counted apart from the defined states, and the change does not read as though its work were resolved

#### Scenario: The enumeration and the completion report agree
- **GIVEN** any active change
- **WHEN** its completion is read from the enumeration and from the operation that reports a change's completion
- **THEN** the two report the same counts, including how many tasks were done differently

#### Scenario: A change with no tasks document
- **GIVEN** an active change whose tasks document is absent
- **WHEN** the project's changes are enumerated
- **THEN** the change is named and its completion is reported as absent rather than as zero

#### Scenario: Order does not follow progress
- **GIVEN** a project whose changes are at differing stages of completion
- **WHEN** the changes are enumerated twice, with work completed in between
- **THEN** the order is unchanged

#### Scenario: A change whose predecessor is unfinished
- **GIVEN** an active change that follows another active change
- **WHEN** the project's changes are enumerated
- **THEN** the predecessor is named beside it and reported as unresolved

#### Scenario: A change whose predecessor is done
- **GIVEN** an active change whose stated predecessor is archived
- **WHEN** the project's changes are enumerated
- **THEN** the predecessor is named beside it and reported as resolved

#### Scenario: Order does not follow the stated sequence
- **GIVEN** a project whose changes state predecessors
- **WHEN** the changes are enumerated
- **THEN** the order is by name and does not follow the stated sequence

### Requirement: The command surface reports a change's completion

The command surface SHALL provide an operation that reports, for one change, which of its artifacts are present and how many of its tasks are complete. The report SHALL be available as a structured payload.

**The report SHALL also state how many of the change's `done` tasks state a deviation, and SHALL name each one.** For such a task the deviation, not its words, is the claim about what was done, so a reviewer checking the change before archival compares each deviation with the work. A count alone would leave them searching the tasks document for the tasks it counts. The count SHALL be the one the enumeration of changes carries, derived from the same source.

Archival SHALL derive its report about incomplete work from this same operation, so that a contributor who checks a change before archiving it and the archival that follows cannot disagree about what the change holds.

#### Scenario: A change mid-implementation
- **WHEN** completion is reported for a change with a proposal, a design, tasks, and no specification
- **THEN** the report names the specification as absent and states how many tasks remain

#### Scenario: Archival and the report agree
- **WHEN** archival reports incomplete work for a change
- **THEN** the count it reports is the count the completion operation reports for that change

#### Scenario: A change with a task done differently
- **GIVEN** a change one of whose `done` tasks states a deviation
- **WHEN** completion is reported for that change
- **THEN** the task is counted as complete, and the report states that one task was done differently and names it

### Requirement: The command surface reports whether a change reached the accepted contract

The command surface SHALL provide an operation that reports, for one change, whether the requirements its specification states have reached the project's accepted specifications. The report SHALL be available as a structured payload.

The operation SHALL determine this by comparing requirement identifiers; for a renamed requirement, headings; and for a requirement the change adds or modifies, its statements and scenarios as text. It SHALL read neither the change's specification nor the accepted specifications for meaning. This is the same comparison archival makes, and both SHALL derive it from one source, so that a contributor who runs the operation and the archival that follows cannot disagree about which requirements reached the contract.

The report SHALL distinguish three sets: the requirement identifiers the change states, those that have reached the accepted specifications, and those absent from them that the change does not withdraw. The third SHALL be reported although a caller could derive it, because it is the answer the operation exists to give.

**Each requirement SHALL be judged by the change state the change declares on it**, as document-schema's "Semantic units declare a change state" provides it. An identifier the accepted specifications hold says only that a requirement carrying it exists. Whether the change's part of that requirement arrived depends on what the change says it did to it:

- **A requirement the change marks `added` or `modified` SHALL have reached the contract only where an accepted specification holds its identifier with the content the change states for it.** Its content, for this comparison, is its statements and its scenarios, less the scenarios the change marks `removed`, and the accepted requirement SHALL hold no scenario beyond those. Where the identifier is held and the content differs, the requirement SHALL be reported in a further set: content the accepted specifications have not taken. A modified requirement's identifier is held before the merge begins, so a comparison of identifiers alone reports every modification as reached whether or not anyone merged it. An added requirement is compared the same way, because its content is text the change establishes and the merge writes, as a modified requirement's is. The comparison means something because artifact-kinds' "A change's specification states a modified requirement whole" has the change state every statement and scenario the requirement will hold, so a difference from the accepted text is a difference, and not a part the change left out.
- **A requirement the change marks `removed` SHALL have reached the contract where no accepted specification holds its identifier.** Where one still does, the requirement SHALL be reported in a further set: removals the accepted specifications have not taken. It SHALL NOT be reported as absent. Its absence is what the change asks for, and a comparison that reports a correctly applied withdrawal as absent, and an unapplied one as reached, has both answers the wrong way round.
- **A requirement the change marks `unchanged` SHALL be compared by its identifier alone**, and by its heading where it states a rename. A requirement a change carries forward untouched may differ from the accepted text because another change modified it since, and comparing its text would report that change's work as this one's. A requirement marked `unchanged` that holds a scenario the change marks `added`, `modified`, or `removed` is not carried forward untouched: the change states content inside it, so it SHALL be compared as a requirement the change modifies is. Nothing else requires such a requirement to be marked `modified`, and judged by its own mark alone, a scenario added inside it would reach the contract unmerged.

**The content SHALL be compared as text after whitespace normalization, which is an exact comparison and not a reading for meaning**, as the heading comparison below is. The text SHALL be the text the requirement's projection holds, not its markup, so a merge that writes the same words with other presentational markup has taken them. The statements SHALL be compared in order, as one sequence of paragraphs, because each is read after the one before it, and moving one changes what the next refers to. The scenarios SHALL be compared as a collection, without regard to their order or their identifiers, each by its heading name and by its given, when, and then entries in order. A scenario is a unit a reader checks on its own, so its place among the others obliges nothing, and an identifier says which scenario a text belongs to, not whether the accepted requirement says it. A requirement's notes and its `renamed-from` paragraph are not content and SHALL NOT be compared. Neither is the requirement's own heading, which only a stated rename compares.

**A renamed requirement SHALL be compared by its heading as well as its identifier.** A rename keeps the requirement's identifier, so a comparison of identifiers alone reports a rename the accepted specifications never took as reached. Where a requirement the change states carries a `renamed-from` paragraph, and the accepted specifications hold its identifier, the operation SHALL compare that requirement's heading with the heading of the accepted requirement carrying the same identifier. Where they differ, the requirement SHALL be reported in a further set: renames the accepted specifications have not taken. The comparison SHALL be of the two heading names — each heading without the identifier shown beside it, as document-schema's "A renamed requirement states the heading it replaced" defines — after whitespace normalization, which is an exact comparison and not a reading for meaning.

A renamed requirement whose identifier the accepted specifications do not hold SHALL be reported as absent, as any other requirement is, and SHALL NOT also be reported as an untaken rename. A requirement the change adds or modifies whose identifier is not held SHALL likewise be reported as absent, and SHALL NOT also be reported as content not taken. A requirement in any of the further sets SHALL be counted as not having reached the contract, so a change whose only unapplied work is a rename, a word, a scenario, or a withdrawal is reported as not synced. A requirement the change both renames and modifies SHALL be compared for both, and where the accepted specifications took neither it SHALL be reported in both sets, because the rename and the modification are two facts about one requirement, as artifact-kinds' "A rename reaches the accepted contract as a new heading on the same identifier" states.

**The operation SHALL NOT modify an accepted specification, and SHALL NOT propose the text of a merge.** Deciding which accepted requirement a scenario belongs under is judgment, and `artifact-kinds` reserves it for an agent. This prohibition is stated rather than left implied: the operation carries OpenSpec's name for it, that name reads like an operation that writes, and a reader who assumes it does would be assuming the program edits the project's contract unattended.

**Each requirement in the content set SHALL be reported with where its content differs, and without the text of either side.** The report SHALL name the first statement paragraph at which the change's statements and the accepted requirement's differ, counted from one, or state that the statements agree. It SHALL name each scenario the change states and does not mark `removed` that the accepted requirement does not hold, by the identifier it carries in the change's specification. And it SHALL name each scenario the accepted requirement holds that is not among those, by the identifier it carries in the accepted specification, or by its heading where it carries none. A position and an identifier say where to read. The text a merge would write is the change's to state and the agent's to apply, and a report that carried it would be proposing the merge this requirement forbids.

Every requirement that has not reached the contract SHALL be a finding and not an error, and the operation SHALL succeed while any remain. A change mid-implementation legitimately states requirements that have not reached the contract, and a requirement the change stated whose work went another way is absent permanently; an operation that failed on either would fail on the normal state of the work it is meant to be run during.

The operation SHALL refuse on a change that cannot be resolved, on an absent specification artifact, and on a specification that does not validate. A comparison made against an invalid contract has no meaning, and reporting it as though it did is worse than declining to report.

#### Scenario: A change whose contract has not been applied
- **GIVEN** a change whose specification states a requirement it does not mark `removed`, which no accepted specification holds
- **WHEN** the operation is run against that change
- **THEN** the requirement is named as absent, the operation succeeds, and no accepted specification is modified

#### Scenario: A change whose contract has been applied
- **GIVEN** a change every one of whose stated requirements appears in an accepted specification
- **WHEN** the operation is run against that change
- **THEN** the absent set is empty

#### Scenario: The comparison is of identifiers
- **WHEN** the operation determines whether a requirement reached the contract
- **THEN** it compares identifiers, the headings of a stated rename, and the statements and scenarios of each requirement the change adds or modifies as text, and reads neither the change's specification nor the accepted specifications for meaning

#### Scenario: The operation and archival agree
- **WHEN** archival reports that a change states requirements no accepted specification holds
- **THEN** the identifiers it names are the identifiers the operation reports as absent for that change

#### Scenario: A structured report
- **WHEN** structured output is requested
- **THEN** the stated, reached, and absent identifier sets, and the untaken renames, content, and removals, are emitted as a payload

#### Scenario: An invalid specification
- **GIVEN** a change whose specification does not validate
- **WHEN** the operation is run against it
- **THEN** it refuses and reports why, and reports no comparison

#### Scenario: The accepted specifications are never written
- **WHEN** the operation completes, for any change and any outcome
- **THEN** no accepted specification was modified by the program

#### Scenario: A rename the accepted specifications have not taken
- **GIVEN** a change whose specification states `req-x` under a new heading with a `renamed-from` paragraph, and an accepted specification holding `req-x` under the previous heading
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported as a rename the accepted specifications have not taken, not as reached, and the operation succeeds

#### Scenario: A rename the accepted specifications have taken
- **GIVEN** an accepted specification holding `req-x` under the heading the change renames it to
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported as reached and in no other set

#### Scenario: A renamed requirement whose identifier is not held
- **GIVEN** a change stating a renamed requirement whose identifier no accepted specification holds
- **WHEN** the operation is run against that change
- **THEN** the requirement is reported as absent and not as an untaken rename

#### Scenario: A structured report carries the untaken renames
- **WHEN** structured output is requested
- **THEN** the untaken renames are emitted as their own set, each naming the identifier, the heading the change states, and the heading the accepted specification holds

#### Scenario: A word the accepted specifications have not taken
- **GIVEN** a change whose specification states `req-x` marked `modified`, with one word added to its first statement paragraph, and an accepted specification holding `req-x` without that word
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported as content the accepted specifications have not taken, differing from its first statement paragraph, and not as reached, and the operation succeeds

#### Scenario: A scenario the accepted specifications have not taken
- **GIVEN** a change whose specification states `req-x` marked `modified`, with a scenario marked `added` that the accepted `req-x` does not hold
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported as content the accepted specifications have not taken, naming that scenario by the identifier it carries in the change's specification

#### Scenario: A withdrawn scenario the accepted requirement still holds
- **GIVEN** a change whose specification states `req-x` marked `modified`, with a scenario marked `removed` that the accepted `req-x` still holds
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported as content the accepted specifications have not taken, naming that scenario by the identifier it carries in the accepted specification

#### Scenario: A modification the accepted specifications have taken
- **GIVEN** an accepted `req-x` holding the statements the change states, in the change's order, and the scenarios the change states less those it marks `removed`, in another order and beside a note the change does not state
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported as reached and in no other set

#### Scenario: Statements in another order
- **GIVEN** an accepted `req-x` holding the statements the change states, with two of them in the other order
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported as content the accepted specifications have not taken

#### Scenario: An added requirement merged with other content
- **GIVEN** a change whose specification states `req-new` marked `added`, and an accepted specification holding `req-new` without one of the scenarios the change states for it
- **WHEN** the operation is run against that change
- **THEN** `req-new` is reported as content the accepted specifications have not taken, and not as reached

#### Scenario: A requirement carried forward is compared by its identifier
- **GIVEN** a change whose specification states `req-z` marked `unchanged`, and an accepted `req-z` whose statements differ from the ones the change states
- **WHEN** the operation is run against that change
- **THEN** `req-z` is reported as reached and in no other set

#### Scenario: A scenario changed inside a requirement marked unchanged
- **GIVEN** a change whose specification states `req-x` marked `unchanged`, holding a scenario marked `added` that the accepted `req-x` does not hold
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported as content the accepted specifications have not taken, naming that scenario, and not as reached

#### Scenario: A removal the accepted specifications have taken
- **GIVEN** a change whose specification states `req-y` marked `removed`, and no accepted specification holding `req-y`
- **WHEN** the operation is run against that change
- **THEN** `req-y` is reported as reached, and not as absent

#### Scenario: A removal the accepted specifications have not taken
- **GIVEN** a change whose specification states `req-y` marked `removed`, and an accepted specification that still holds `req-y`
- **WHEN** the operation is run against that change
- **THEN** `req-y` is reported as a removal the accepted specifications have not taken, and not as reached

#### Scenario: A rename and a modification, neither taken
- **GIVEN** a change whose specification states `req-x` marked `modified` with a `renamed-from` paragraph, and an accepted `req-x` holding neither the new heading nor the new content
- **WHEN** the operation is run against that change
- **THEN** `req-x` is reported both as a rename and as content the accepted specifications have not taken, and not as reached

#### Scenario: The report says where the content differs, not what it says
- **WHEN** structured output is requested for a change with a requirement whose content was not taken
- **THEN** that requirement's entry names its identifier, the first statement paragraph that differs or none, and each differing scenario by identifier, and carries no statement or scenario text from either side

#### Scenario: The operation and archival agree about what did not arrive
- **WHEN** archival reports a rename, a requirement's content, or a removal that the accepted specifications have not taken
- **THEN** the requirements it names are the ones the operation reports in the same set for that change

### Requirement: The workflow instructions are available from the command surface

The command surface SHALL emit the workflow instructions for a named stage on demand, as the same text the bootstrap operation installs.

An agent obtains the instructions without knowing which tool's directory holds them, and without the risk of reading a copy installed by an earlier version of the toolchain. A stage's instructions and the installed copy SHALL come from one source, so the two cannot state different things.

#### Scenario: Instructions for a stage
- **WHEN** the instructions for archival are requested
- **THEN** the workflow instructions for archival are emitted

#### Scenario: One source
- **WHEN** the emitted instructions and the installed instructions for one stage are compared
- **THEN** they are the same text

### Requirement: The workflow instructions carry a stage for applying a change's contract

The workflow instructions SHALL carry a stage for merging a change's stated requirements into the project's accepted specifications, available from the command surface as every other stage is and installed by the bootstrap operation from the same source.

The stage's instructions SHALL state that the merge is the agent's work: that both the change's specification and the accepted specifications are read for meaning before anything is written, that a change's specification is never copied into an accepted specification wholesale, and that the reporting operation is run again afterwards to confirm that the absent set is empty and that no rename, content, or removal remains untaken.

The instructions SHALL state that a requirement the change stated whose work went another way is left absent and reported to the contributor as abandoned, rather than merged to empty the report. They SHALL state that a requirement the change marks `removed` is withdrawn by deleting it from the accepted specification that holds it, and that the reporting operation then counts it as reached.

**The instructions SHALL say what to do about every set the reporting operation's structured payload carries.** A set the payload carries and the instructions never mention is work an agent is shown and not told how to do.

The obligations on the *program* live in the requirement above. This one governs the text an agent reads, because the half of the operation that requires judgment cannot be stated as a rule the program enforces.

#### Scenario: The stage is available
- **WHEN** the workflow instructions for the stage are requested from the command surface
- **THEN** they are emitted, as the same text the bootstrap operation installs

#### Scenario: The stage is installed
- **WHEN** a project is bootstrapped
- **THEN** the stage's instructions are installed for each supported tool alongside the other stages

#### Scenario: Every reported set is worked
- **WHEN** the stage's instructions are read beside the reporting operation's structured payload
- **THEN** every set the payload carries is named in the instructions, with what the agent does about it

#### Scenario: A withdrawal is merged by deleting
- **GIVEN** a change whose specification marks a requirement `removed`
- **WHEN** an agent follows the stage's instructions
- **THEN** it deletes that requirement from the accepted specification holding it, and the reporting operation then reports it as reached

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

### Requirement: The command surface checks a runbook's machine prerequisites

The command `htmlspec preflight <runbook>` SHALL report whether the selected runbook can start
in the invocation's working environment. It SHALL support the existing `--store` and `--json`
selectors. Store selection SHALL locate the runbook and its records, while machine requirements
SHALL be evaluated against the invocation's working directory and environment, not the store's
directory. Names SHALL follow the existing runbook-name rules.

The report SHALL identify the runbook, carried version, source location, working directory,
aggregate readiness, and each checked requirement with its expected condition, observation,
outcome, and remedy for a refusal. Results SHALL follow document order. Human output SHALL lead
with readiness. JSON output SHALL contain one structured report with no probe output interleaved.

A completed preflight with unmet requirements SHALL exit 1, a successful one SHALL exit 0, an
invalid invocation, invalid contract, or operational failure SHALL exit 2, and cancellation SHALL
exit 3. Unknown observations such as an unavailable manifest environment or a timed-out probe
SHALL be reported as unmet readiness rather than as success. An invalid declaration SHALL remain
distinguishable from a valid check whose condition is false.

`htmlspec run <runbook> --as <record>` SHALL apply the same checks and exit distinctions before
creating a run. `htmlspec run <runbook> --check` SHALL continue to report existing run conformance
without checking the present machine. Existing JSON fields on successful run creation SHALL be
preserved, with version and preflight information added.

#### Scenario: An agent inspects readiness
- **WHEN** the operator requests preflight with `--json`
- **THEN** standard output is one parseable readiness report, no prompt occurs, and probe output stays inside the report

#### Scenario: The selected store differs from the working directory
- **WHEN** a stored runbook is checked from a separate target checkout
- **THEN** the report identifies both locations and evaluates working-tree assumptions in the target checkout

#### Scenario: Several prerequisites fail
- **WHEN** several valid checks report unsatisfied conditions
- **THEN** one report names all of them in document order and the command exits 1

#### Scenario: The contract is malformed
- **WHEN** a selected runbook contains an invalid machine declaration
- **THEN** the command exits 2 and identifies the requirement and invalid field

#### Scenario: Reviewing existing records
- **WHEN** the operator runs `htmlspec run <runbook> --check` on a machine missing a prerequisite
- **THEN** it reports run conformance under its existing advisory contract

### Requirement: The command surface shows a modified requirement beside the accepted one

The command surface SHALL provide `htmlspec show <change> --diff`. For each requirement whose content "The command surface reports whether a change reached the accepted contract" compares — one the change's specification marks `added` or `modified`, and one it marks `unchanged` that holds a scenario the change marks `added`, `modified`, or `removed` — it SHALL show what the change states beside what the accepted specifications hold for the requirement carrying the same identifier, paragraph by paragraph and scenario by scenario. The view SHALL be available in human form and as a structured payload.

The reason is the reviewer. artifact-kinds' "A change's specification states a modified requirement whole" has a change restate every statement and scenario of a requirement it modifies, and such a requirement can run to fifteen paragraphs and eighteen scenarios. A reviewer who sees `modified` on it otherwise has to find the edit alone, and an edit of one word looks like no edit at all. An added requirement is compared too, because the reporting operation compares it. Before its merge no accepted specification holds it, and after a merge that took it the accepted text holds it as stated, so each is one line. Where an accepted requirement already carries its identifier with other text, the merge took something else, or the identifier names another obligation, and both texts are what an agent needs to tell which.

**The view SHALL read and SHALL NOT write.** It SHALL NOT modify the change, an accepted specification, or any other file. A view that wrote would be a merge under another name, and artifact-kinds reserves the merge for an agent.

**The view SHALL pair and compare as "The command surface reports whether a change reached the accepted contract" does, from the same source.** The accepted requirement it shows SHALL be the one that operation compares the change's requirement with, and the view SHALL show a requirement as differing exactly where that operation reports its content as not taken. Two comparisons of one pair of texts drift apart, and a view that showed a difference the reporting operation did not report, or hid one it did, would give a contributor two answers to one question.

**Statements SHALL be shown paragraph by paragraph.** A paragraph both sides hold, in the same order, SHALL be shown as held by both and SHALL NOT be reprinted. The paragraphs that remain SHALL be paired, without crossing and as many as can be, where a paragraph the change states and one the accepted requirement holds share at least half of each one's words. Each pair SHALL be shown as both texts, with the words only one side holds marked on that side. A paragraph with no partner SHALL be shown whole, as stated only by the change or held only by the accepted specification. A pair that shares fewer than half its words would be marked more than it is kept, and reads better as two whole paragraphs.

A word SHALL be a run of the projected text between whitespace. So presentational markup is neither compared nor shown, as the reporting operation compares text and not markup. Where the heading the change states differs from the accepted heading, both headings SHALL be shown.

**Marks an author wrote SHALL be shown as written wherever they account for the whole difference.** A requirement or scenario marked `modified` in a change's specification may carry `ins` and `del` elements, as document-validation's "Change markup in or around a semantic unit is rejected" permits. Where a requirement's statements, or a scenario's entries, carry them, and that text read with each deletion kept and each insertion left out equals the accepted text, the view SHALL mark the words the author marked, where the author marked them. Where it does not equal the accepted text, the view SHALL mark words by its own comparison and SHALL say that the author's marks do not account for the difference. An author's marks say which words the edit meant, and a computed alignment can pair the same words differently. But a mark claims only what it marks, and only the accepted text shows what an author left unmarked, so the view shows the author's marks where the accepted text confirms them and nowhere else.

**Scenarios SHALL be shown scenario by scenario.** A scenario whose text the accepted requirement holds SHALL be shown as held by both. A scenario the change states that differs from its accepted counterpart SHALL be paired with it by identifier, as artifact-kinds' "A change's specification states a modified requirement whole" has every scenario a change carries forward, alters, or withdraws keep its accepted identifier, and its heading and entries SHALL be shown as statements are. A scenario the change marks `removed` SHALL be shown as withdrawn by the change, saying whether the accepted requirement still holds it. A scenario the change states with no accepted counterpart SHALL be shown whole as stated only by the change, and an accepted scenario the change neither states nor marks `removed` SHALL be shown whole as held only by the accepted specification. A scenario is checked on its own, so it is paired by what it is rather than by where it sits.

A requirement the view compares whose identifier no accepted specification holds SHALL be shown as held by none, with nothing compared. One the accepted specification holds as the change states it SHALL be shown as held as stated, with no text. The first is the reporting operation's absent requirement, and the second is content the merge already took.

**A requirement the change marks `removed`, and one it marks `unchanged` whose every scenario it marks `unchanged`, SHALL be listed with its identifier, its heading, and the state the change declares on it, and SHALL NOT be compared.** The reporting operation compares neither by its content. A removed requirement is withdrawn whole, and whether it is gone is that operation's question. One carried forward declares that it states the accepted text, and where the two differ, the difference is another change's later edit, which is not this change's to show. Listing them keeps the view an account of every requirement the change states, so that none is passed over without a word.

**Nothing the view shows SHALL read as the text of a merge.** Every text SHALL be labelled with the document that holds it, as what the change states or what the accepted specification holds. No text SHALL combine the two sides, and none SHALL be labelled as a result, a replacement, or something to apply. The human form SHALL NOT be a patch: it SHALL carry no hunk header and no file header, and it SHALL NOT mark a whole line as added or removed. The reporting operation and archival each "SHALL NOT propose the text of a merge", because what the accepted specification says next is an agent's judgment. A view is a different thing. It restates two texts that already exist, each labelled with where it is, and states no third. A patch is exactly a third text: the edit, in a form a tool applies. So the view may carry both texts, and SHALL NOT carry the edit.

**`htmlspec show <change>` without `--diff` SHALL refuse as an invalid invocation, and SHALL name `--diff`.** OpenSpec's `show` prints a change's proposal when given no view. Here a proposal is a page a browser already renders, and `status` already reports what a change holds. A default view added now would be an output nobody has asked for and a contract to keep. Refusing leaves the bare form free for a later change to define, without changing what any caller already receives.

**An archived change SHALL be refused**, with a report that says it is archived and why the view does not apply to it. The accepted specifications hold an archived change's edit together with every later change's. A comparison with them shows nothing where no later change touched the requirement, and shows a later change's edit, reversed, where one did. Neither is what the archived change modified.

The view SHALL refuse on a change that cannot be resolved, on an absent specification artifact, and on a specification that does not validate, as the reporting operation does and for its reason. Under "Exit codes distinguish outcomes", it SHALL exit zero whenever it shows the view, whatever the view shows. It SHALL exit with the code for a document violation where the change's specification does not validate, and with the code for an operation that could not run on every other refusal, the missing `--diff` and an archived change included. A difference is not a failure. A change's modified requirements differ from the accepted text for the whole of the change's life before its merge, and a view that failed on that would fail on the state it exists to show.

Structured output SHALL follow "Output is available in human and machine forms": standard output SHALL hold the payload and nothing else. With `--store`, the view SHALL read the registered project's change and accepted specifications, as "Specifications can live in a repository other than the one being worked on" provides.

#### Scenario: A one-word modification
- **GIVEN** a change whose specification marks `req-x` `modified`, stating ten statement paragraphs and eighteen scenarios, and an accepted `req-x` whose third paragraph lacks one word the change's third paragraph holds
- **WHEN** a contributor runs `htmlspec show <change> --diff`
- **THEN** the view shows nine paragraphs and eighteen scenarios as held by both, and the third paragraph as the accepted text beside the change's text with that word marked on the change's side, and it exits zero

#### Scenario: The view writes nothing
- **WHEN** the view runs, for any change and any outcome
- **THEN** no file in the project was modified

#### Scenario: A paragraph rewritten throughout
- **GIVEN** a modified requirement whose change text replaces one paragraph with a paragraph that shares fewer than half its words
- **WHEN** the view is shown
- **THEN** the accepted paragraph is shown whole as held only by the accepted specification, and the change's paragraph whole as stated only by the change, with no word marked inside either

#### Scenario: A paragraph added beside an edited one
- **GIVEN** a modified requirement whose change text adds a paragraph immediately before a paragraph it edits by one word
- **WHEN** the view is shown
- **THEN** the added paragraph is shown whole as stated only by the change, and the edited paragraph is paired with its accepted counterpart with the word marked

#### Scenario: The author's marks account for the difference
- **GIVEN** a modified requirement whose change text marks an inserted word with `ins` and a deleted phrase with `del`, and whose text read with the deletion kept and the insertion left out equals the accepted text
- **WHEN** the view is shown
- **THEN** the words marked are the ones the author marked, and the view says the marks are the author's

#### Scenario: The author's marks leave part of the difference unmarked
- **GIVEN** a modified requirement whose change text marks one inserted word with `ins`, and also leaves out a word the accepted text holds without marking it
- **WHEN** the view is shown
- **THEN** the words marked are found by the view's own comparison, both words among them, and the view says the author's marks do not account for the difference

#### Scenario: A scenario the change alters
- **GIVEN** a modified requirement stating a scenario marked `modified` that carries its accepted counterpart's identifier and differs from it by one word of its THEN entry
- **WHEN** the view is shown
- **THEN** that scenario is paired with its accepted counterpart with the word marked in the THEN entry, and every other scenario both hold is shown as held by both

#### Scenario: Scenarios added, withdrawn, and left out
- **GIVEN** a modified requirement stating a scenario with no accepted counterpart and a scenario marked `removed`, and an accepted requirement that also holds a scenario the change neither states nor marks `removed`
- **WHEN** the view is shown
- **THEN** the first is shown as stated only by the change, the second as withdrawn by the change and still held by the accepted specification, and the third as held only by the accepted specification

#### Scenario: The view and the reporting operation agree
- **WHEN** the view and the reporting operation are run against the same change
- **THEN** the requirements the view shows as differing are exactly those the operation reports as content not taken, and for each of them the first paragraph the view does not show as held by both is the first differing paragraph the operation names, or neither names one

#### Scenario: A modification the accepted specification already holds
- **GIVEN** an accepted `req-x` holding the statements and scenarios the change states for it
- **WHEN** the view is shown
- **THEN** `req-x` is shown as held as stated, with no text and no marks

#### Scenario: A modified requirement no accepted specification holds
- **GIVEN** a change marking `req-x` `modified`, and no accepted specification holding `req-x`
- **WHEN** the view is shown
- **THEN** `req-x` is shown as held by no accepted specification, nothing is compared, and the view exits zero

#### Scenario: A renamed requirement
- **GIVEN** a modified requirement stating a new heading with a `renamed-from` paragraph, and an accepted requirement holding the previous heading
- **WHEN** the view is shown
- **THEN** both headings are shown, each labelled with the document that holds it

#### Scenario: An added requirement the accepted specification holds otherwise
- **GIVEN** a change whose specification marks `req-x` `added`, and an accepted `req-x` whose statement lacks one word the change's holds
- **WHEN** the view is shown
- **THEN** `req-x` is shown with its declared state, `added`, and compared as a modified requirement is, with that word marked on the change's side

#### Scenario: A scenario changed inside an unchanged requirement
- **GIVEN** a requirement marked `unchanged` that holds a scenario marked `modified`, which differs from its accepted counterpart by one word of its THEN entry
- **WHEN** the view is shown
- **THEN** the requirement is shown with its declared state, `unchanged`, and compared, and that scenario is paired with its accepted counterpart with the word marked

#### Scenario: Requirements that are not compared
- **GIVEN** a change whose specification also states a requirement marked `removed`, and one marked `unchanged` whose every scenario is marked `unchanged`
- **WHEN** the view is shown
- **THEN** each is listed with its identifier, its heading, and its declared state, and neither is compared

#### Scenario: Nothing reads as merge text
- **WHEN** the view is shown, in either form
- **THEN** every text is labelled as what the change states or what the accepted specification holds, no text combines the two, and the human form carries no hunk header and no file header, and marks no whole line as added or removed

#### Scenario: Showing a change without a view
- **WHEN** a contributor runs `htmlspec show <change>` without `--diff`
- **THEN** the program shows nothing, names `--diff`, and exits with the code for an operation that could not run

#### Scenario: An archived change
- **GIVEN** a change that has been archived
- **WHEN** a contributor runs the view against its directory
- **THEN** it refuses, says that the change is archived and why the view does not apply to it, and exits with the code for an operation that could not run

#### Scenario: An invalid specification
- **GIVEN** a change whose specification does not validate
- **WHEN** the view is run against it
- **THEN** it refuses and reports why, shows no comparison, and exits with the code for a document violation

#### Scenario: A structured view
- **WHEN** structured output is requested
- **THEN** standard output holds only a payload that lists each requirement the change states with its declared state, and, for each requirement it compares, the accepted specification it was compared with and each of its paragraphs and scenarios as held by both, paired, or held by one side, with the text of each that is not held by both and the words only one side holds

#### Scenario: A registered standalone project
- **WHEN** the view is run with `--store` naming a registered project
- **THEN** it reads that project's change and accepted specifications, and nothing of the project containing the working directory
