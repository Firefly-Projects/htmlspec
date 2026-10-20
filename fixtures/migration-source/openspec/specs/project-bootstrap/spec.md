# project-bootstrap Specification

## Purpose
TBD - created by archiving change project-bootstrap-cli. Update Purpose after archive.

## Requirements

### Requirement: A project has a declared root holding one asset store

An HTMLSpec project SHALL be identified by a directory containing an `htmlspec/` directory. That directory SHALL hold the project's accepted specifications, its active changes, its archive, and **its runbooks**, each in its own place.

**The runbooks are a fourth place rather than a quiet extension of the first three.** A procedure that is run repeatedly is not a change and does not archive, so it cannot live in `changes/` without being destroyed by the operation that ends a change. `runbook-authority` states what a runbook is and what it holds; this requirement states only that the project's root has a place for one.

**This sentence is not declared closed, and the difference from the artifact-kind vocabulary is the argument for saying so.** The set of kinds states that it is `CLOSED` and carries a guard test whose failure message calls adding one "a decision with a compatibility cost". This sentence has neither, and `project-migration` already established that the layout beneath the project root is only partly specified. So a fifth place is a modification with a stated reason rather than a closed set being reopened, and a later change adding one owes the same reason rather than a new argument about whether it may.

A project SHALL hold exactly one asset store, in its root, and every document in the project SHALL reference that store rather than a store of its own. A project storing one copy of its runtime is the whole point of the duplicate-detection rule the contract already states; a tool that gave each change its own copy would be reporting a defect it created itself.

**A runbook's documents are documents of the project and are covered by that sentence without exception.** A runbook is a directory beneath the project root holding documents, which is the same shape a change is, and a store of its own there would be the measured 6-MB-per-change defect arriving at a new location.

#### Scenario: Bootstrapping an empty repository
- **WHEN** a contributor bootstraps a project in a directory with no project in it
- **THEN** the project root is created with places for accepted specifications, active changes, the archive, and runbooks, and one asset store is established in it

#### Scenario: Two changes share one store
- **GIVEN** a bootstrapped project
- **WHEN** two changes are created in it
- **THEN** the project holds one copy of the runtime assets, and each change's documents reference it

#### Scenario: A runbook shares the same store
- **GIVEN** a bootstrapped project holding a change and a runbook
- **WHEN** the project's asset topology is checked
- **THEN** one store is found, because a runbook references the project's store rather than holding one

#### Scenario: Documents reference the store without a network
- **WHEN** a document created in a bootstrapped project is opened from the filesystem
- **THEN** it renders with no network access, because its asset references resolve to the project's store

### Requirement: A project declares its name, version, and expected toolchain

The bootstrap operation SHALL write a project declaration at the project root recording the project's name, its version, and a pinned reference to the program the project expects.

The declaration SHALL be expressed in the same language the project's build already uses, so that the pinned program reference is a build input rather than a value a bespoke parser must interpret. A second configuration language for three fields is surface the project does not need.

At least one command SHALL depend on the declaration. A declaration no command reads states nothing that can be wrong, so nothing reports it when it is, and it drifts from the project it describes without any signal.

The declaration SHALL NOT carry authoring rules or project context. Those have no consumer, and a field nothing reads is the case the preceding paragraph excludes.

#### Scenario: Bootstrapping writes the declaration
- **WHEN** a contributor bootstraps a project
- **THEN** a declaration is written at the project root recording the project's name, its version, and the program it expects

#### Scenario: The declaration is not overwritten
- **GIVEN** a project whose declaration a contributor has edited
- **WHEN** the bootstrap operation runs again
- **THEN** the declaration is left as it was

#### Scenario: The pinned program is a build input
- **WHEN** the project's build reads the declaration
- **THEN** the pinned program reference resolves as a build input, with no separate parser

### Requirement: Archival requires the project version to advance

Archival SHALL compare the project's declared version against the last archived version, and SHALL refuse unless the declared version is ordered after it. The accepted contract SHALL therefore carry a version history rather than a sequence of undated merges.

The comparison SHALL be an ordering rather than a test for difference. A declared version ordered before the last archived one SHALL be refused, and SHALL be reported distinctly from a version that has not moved, because the two have different causes: one is work not yet done, and the other is a version that went backwards.

The declared version SHALL therefore be a value the program can order. A version it cannot parse into an ordering SHALL be reported rather than accepted, because a gate that cannot compare its inputs is not a gate.

The last archived version SHALL be read from the version each archived change records, in the form `toolchain-packaging` requires. It SHALL NOT be derived from an archive directory's name. A directory name is a display string that is shortened and renamed, and a gate derived from one depends on a convention nothing enforces — which is how this gate came to compare against a value no project ever had.

The bootstrap operation SHALL write an initial version that is ordered before every version a project can later declare, so that a project always has a version on record and the comparison is total. Archival SHALL NOT treat an absent previous version as grounds to pass. A project's first archival SHALL advance past the initial version exactly as a later one advances past its predecessor, because a rule with an exception that applies once per project is a rule exercised once per project.

Archival SHALL NOT modify the declaration. Advancing the version SHALL be the contributor's or the agent's act, and the program's part SHALL be the comparison alone.

That division is the one archival already uses: an agent produces the change's specification and the program validates it, because the judgment is judgment and the checking is exact. Deciding what a version increment signifies, and by how much, is judgment. Whether one version is ordered after another is not.

This reservation SHALL be stated on the act rather than on the command. A command SHALL NOT perform an act the project reserves for judgment, whatever else that command also does. A command SHALL NOT be restricted from an act that requires no judgment because of what its name suggests it does. The earlier form of this obligation — that a command whose stated job is checking does not write to the project it checks — generalised a reservation about the declaration into a prohibition on every write, and that generalisation is what left archival as an operation the program declined to complete.

#### Scenario: Archiving without advancing the version
- **WHEN** a contributor archives a change while the declared version equals the last archived version
- **THEN** archival does not complete, and the reason names the version that must advance

#### Scenario: Archiving with an advanced version
- **WHEN** a contributor archives a change whose declared version is ordered after the last archived version
- **THEN** the version gate permits archival to proceed

#### Scenario: Archiving with a version that went backwards
- **WHEN** a contributor archives a change whose declared version is ordered before the last archived version
- **THEN** archival does not complete, and the report distinguishes this from a version that has not moved

#### Scenario: A version the program cannot order
- **WHEN** the declared version cannot be parsed into an ordering
- **THEN** archival does not complete, and the report names the version it could not read

#### Scenario: A project's first archival
- **GIVEN** a freshly bootstrapped project that has archived nothing, whose declared version is still the initial one
- **WHEN** a contributor archives its first change without advancing the version
- **THEN** archival is refused on the same grounds as any later archival, because the initial version is already on record

#### Scenario: The program does not advance the version itself
- **WHEN** archival runs
- **THEN** the declaration is unchanged on disk, whether archival completed or refused

#### Scenario: Archiving with no declaration
- **WHEN** archival runs in a project with no declaration
- **THEN** it reports the absent declaration and names the operation that creates one

#### Scenario: The last archived version comes from the recorded version
- **GIVEN** an archive whose directory names carry no version
- **WHEN** archival determines the last archived version
- **THEN** it reads the version each archived change recorded, and the gate compares against it

#### Scenario: A gate that cannot be satisfied by renaming
- **WHEN** an archived change's directory is renamed
- **THEN** the version the gate reads is unchanged, because the version is a record rather than part of the name

#### Scenario: Archival that writes without advancing the version
- **WHEN** archival completes and moves the change into the archive
- **THEN** the declaration is unchanged, because advancing the version is the contributor's act and archival's part is the comparison alone

### Requirement: Commands resolve the project from any directory beneath it

Every command whose subject is a project SHALL locate the project by searching the current directory and its ancestors, and SHALL operate on the project found rather than on the directory it was invoked in.

The search SHALL NOT ascend past the enclosing repository. A command run inside one repository SHALL NOT resolve to a project outside it, because writing into a neighboring project is worse than reporting that no project was found.

A command that requires a project and finds none SHALL say so, SHALL name the operation that would create one, and SHALL exit with the status for an operation that could not run.

#### Scenario: Running from a subdirectory
- **GIVEN** a bootstrapped project
- **WHEN** a contributor runs a project-wide command from a directory inside it
- **THEN** the command operates on the whole project

#### Scenario: The search stops at the repository boundary
- **GIVEN** a repository with no project in it, nested inside a directory that contains an unrelated project
- **WHEN** a contributor runs a project-wide command inside the repository
- **THEN** the command reports that no project was found rather than resolving to the unrelated one

#### Scenario: No project found
- **WHEN** a project-wide command runs where no project exists
- **THEN** it reports that no project was found, names the operation that creates one, and exits with the status for an operational failure

#### Scenario: Checking a project's assets
- **WHEN** the asset-topology command runs anywhere in a project
- **THEN** it checks every asset the project stores, wherever in the project they are stored

### Requirement: Bootstrapping installs agent instructions for the tools a project uses

The bootstrap operation SHALL install, for each selected AI coding tool, the instructions that tool needs to drive the HTMLSpec workflow. Where a tool supports invocable commands, the operation SHALL install those as well as the instructions.

The instructions SHALL cover the project's lifecycle from an idea to an archived change, rather than a single stage of it. A contributor who has bootstrapped a project and lists what was installed SHALL find the stage they are at. Instructions covering only the final stage leave every earlier stage to be improvised, which is the case the project's own contract for artifact kinds exists to prevent.

A workflow SHALL be installed whether or not a command performs it. Archival is already agent-driven — the program checks a specification it does not produce — so a stage whose work is judgment is the ordinary case rather than the exception.

An installed workflow's instructions SHALL NOT name a command the program does not provide, and this SHALL be checked mechanically rather than by review. Naming an absent command sends an agent to an error, and it is the hazard that a rule limiting how many workflows are installed was attempting to avoid.

The set of supported tools SHALL be data rather than logic, so that supporting an additional tool is an entry in that set. The set of workflows SHALL likewise be data, so that covering an additional stage is an entry in that set together with its instructions.

The order of the workflows SHALL be the order the lifecycle moves through them, because instructions written into a shared file are presented in that order and a reader meets them as a document.

How a tool receives instructions SHALL be a property of the tool with one value per tool, rather than a combination of independent settings. A tool for which no instructions could be written is a state the program SHALL NOT be able to reach.

Instructions SHALL record the version of the program that generated them, so an installation predating the instructions it should carry is identifiable.

Where a workflow is reached through an invocation, whatever a contributor types after that invocation SHALL reach the agent that performs the workflow. An invocation surface that accepts text and discards it makes the workflow ask for what it was already given, which is the failure a contributor experiences as being asked twice. This obligation reaches the kinds of file a tool invokes and SHALL NOT reach a kind that carries no invocation, because there is no argument to pass where there is nothing to type.

What a workflow takes SHALL be declared alongside the workflow, and SHALL be a property of that workflow rather than one description shared by all of them. Workflows differ in their input — one begins from a description of work that does not exist, another acts on a change already present — so a shared declaration describes none of them accurately.

An absent argument SHALL reach the agent as absent rather than as an error, and a workflow SHALL remain invocable with no argument at all. Where nothing was typed, the workflow's own guidance on a missing input applies; a surface that rejected a bare invocation would remove the behavior that is correct when a contributor has genuinely described nothing. An agent SHALL be able to distinguish an argument that was not supplied from one that was lost in delivery, because an agent that suspects the latter asks again for what it was sent.

An argument's declaration and its delivery SHALL both be checked against the files the operation produces, rather than against the values they derive from. Each is assembled when a file is written, so a check over the inputs passes while a writer that omitted one ships.

Every generated file's declared name SHALL be composed only of lowercase letters, digits, and hyphens, because a tool that reads it may reject any other character. This SHALL hold for each kind of file the operation writes, and SHALL be checked against the files produced rather than against the values they are derived from — a name is assembled at the moment of writing, so a fault introduced there is invisible to a check on the inputs.

A generated name SHALL NOT be the invocation a contributor types. The invocation is a property of a tool's command surface, it differs per tool, and it SHALL have one producer; a name field holding it states that fact a second time and states it where a stricter reader rejects it. The name SHALL instead identify the workflow, so that a workflow has one generated identity rather than one per file kind.

No two files the operation writes for one tool SHALL declare the same name. A declared name is a registry key for the tool that loads it, so a workflow's identity SHALL be declared once within a tool rather than repeated in each kind of file the workflow produces — a second declaration presents one workflow as two. Where a kind of file does not need a name in order to be found, the operation SHALL NOT write one; a command whose invocation is determined by its file path is such a kind. The same name appearing in two different tools' files is not a repeat, because each tool's namespace is its own.

#### Scenario: Installing for a tool with commands
- **WHEN** the bootstrap operation installs instructions for a tool that supports invocable commands
- **THEN** both the instructions and the commands are written where that tool reads them, one of each per workflow

#### Scenario: Installing for a tool without commands
- **WHEN** the bootstrap operation installs instructions for a tool that supports instructions only
- **THEN** the instructions are written and no command files are created

#### Scenario: Every selected tool receives something
- **WHEN** the bootstrap operation completes for any selected tool
- **THEN** that tool has received instructions, because a selected tool that receives nothing is not a representable outcome

#### Scenario: The lifecycle is covered
- **WHEN** a contributor lists the workflows installed by the bootstrap operation
- **THEN** the stages from exploring an idea through to archiving a completed change are each named

#### Scenario: A workflow with no command behind it
- **GIVEN** a stage of the lifecycle that no command performs
- **WHEN** the bootstrap operation installs instructions
- **THEN** that stage's instructions are installed, and they state that the work is the agent's

#### Scenario: Instructions naming an absent command
- **WHEN** an installed workflow's instructions name an invocation the program does not provide
- **THEN** the project's own checks report it, rather than the invocation reaching a contributor

#### Scenario: Workflows are presented in lifecycle order
- **WHEN** instructions for every workflow are written into one shared file
- **THEN** they appear in the order the lifecycle moves through them

#### Scenario: Generated instructions name their version
- **WHEN** instructions are installed
- **THEN** they record the version of the program that generated them

#### Scenario: An invocation carrying a description
- **WHEN** a contributor invokes a workflow and types a description of the work after it
- **THEN** that description reaches the agent, which acts on it rather than asking what the work is

#### Scenario: An invocation carrying nothing
- **WHEN** a contributor invokes a workflow and types nothing after it
- **THEN** the workflow runs, the agent observes that no input was supplied, and its guidance for a missing input applies

#### Scenario: A workflow declares what it takes
- **WHEN** the bootstrap operation installs a workflow whose input differs from another's
- **THEN** the file it writes declares that workflow's own input rather than a description shared across workflows

#### Scenario: A delivery kind with no invocation
- **WHEN** instructions are written into a shared file that no contributor invokes
- **THEN** no argument is declared, because there is no invocation to carry one

#### Scenario: A dropped argument is caught before release
- **WHEN** the operation would write a file that declares an argument its body never consumes, or consumes one it never declares
- **THEN** the project's own checks report it against the produced file, rather than a contributor's description being discarded at invocation

#### Scenario: Every generated name is loadable
- **WHEN** the bootstrap operation installs for every supported tool
- **THEN** every file it writes declares a name of lowercase letters, digits, and hyphens only

#### Scenario: A name that is not an invocation
- **WHEN** a command file is generated for a tool whose invocation carries a character outside that set
- **THEN** the file's declared name identifies the workflow and does not carry the invocation

#### Scenario: A workflow is listed once per tool
- **WHEN** a workflow produces both instructions and a command for one tool
- **THEN** the workflow's name is declared once across the two, so a contributor listing what is installed finds the workflow once rather than twice

#### Scenario: A repeated name is caught before release
- **WHEN** the bootstrap operation would write the same name into two files one tool loads
- **THEN** the project's own checks report it, rather than a contributor meeting a doubled listing when their tool loads them

#### Scenario: A malformed name is caught before release
- **WHEN** a workflow is added whose identity carries a character outside that set
- **THEN** the project's own checks report it, rather than a contributor meeting it when their tool loads the file

### Requirement: Tool selection is offered interactively and available without a terminal

The bootstrap operation SHALL detect which supported tools a project already shows evidence of, and SHALL offer the supported tools for selection with the detected ones already chosen.

Where nothing is detected, the operation SHALL offer every supported tool already chosen. An absent detection means a project nobody has configured, which is where a default matters most and where detection has least to say. A prompt that offers nothing there teaches that the prompt is decorative.

The operation SHALL accept the selection as an argument instead, including forms meaning every supported tool and no tool at all. When a selection is given as an argument, the operation SHALL NOT prompt, because the question has already been answered.

An interactive selection of no tools SHALL be refused, and the operation SHALL ask again rather than proceed. The refusal SHALL name the argument that means no tool, so that a contributor who intends it has somewhere to go.

The argument form SHALL continue to succeed. The asymmetry is the requirement: the argument is an answer to the question, and an empty confirmation at a prompt is usually a contributor who did not read the question. The prompt appears on a first run in a fresh repository, which is where fewest contributors know what the list decides. A bootstrap that installs nothing while reporting that the project is ready has answered a question nobody asked.

The operation SHALL NOT prompt when it is not running against a terminal, and SHALL NOT prompt when structured output is requested. In either case it SHALL proceed using the detected tools. A bootstrap that blocked waiting for a keypress inside an automated job would never complete.

Detection SHALL recognize a tool by any of several paths, so that a tool which has renamed its directory is still recognized by its former one.

#### Scenario: Interactive selection preselects detected tools
- **GIVEN** a project containing the configuration directories of two supported tools
- **WHEN** a contributor bootstraps the project in a terminal with no selection argument
- **THEN** the supported tools are offered for selection with those two already chosen

#### Scenario: Selection given as an argument
- **WHEN** the bootstrap operation is given a selection argument
- **THEN** it installs for exactly those tools and does not prompt

#### Scenario: Nothing detected offers everything
- **GIVEN** a fresh repository showing evidence of no supported tool
- **WHEN** a contributor bootstraps it in a terminal with no selection argument
- **THEN** every supported tool is offered already chosen, because detection has nothing to narrow the offer with

#### Scenario: Every tool and no tool
- **WHEN** the selection argument names every supported tool, or names none
- **THEN** the operation installs for all of them, or for none of them, and succeeds either way

#### Scenario: An empty interactive selection is refused
- **WHEN** a contributor confirms a selection of no tools at the prompt
- **THEN** the operation does not proceed, it asks again, and the refusal names the argument that means no tool

#### Scenario: The argument form still means no tool
- **WHEN** the selection argument names no tool
- **THEN** the operation succeeds and installs nothing, because an argument is an answer and an empty confirmation is not

#### Scenario: No terminal available
- **WHEN** the bootstrap operation runs with its input not connected to a terminal and no selection argument
- **THEN** it proceeds without prompting, using the tools it detected, and does not block

#### Scenario: Structured output requested
- **WHEN** the bootstrap operation runs with structured output requested
- **THEN** it does not prompt, and standard output contains nothing but the structured payload

#### Scenario: An unknown tool named
- **WHEN** the selection argument names a tool that is not supported
- **THEN** the operation reports the error, lists the supported tools, and writes nothing

#### Scenario: A renamed tool directory is recognized
- **GIVEN** a project containing a supported tool's former configuration directory
- **WHEN** detection runs
- **THEN** that tool is detected

### Requirement: Bootstrapping is repeatable without loss

The bootstrap operation SHALL be safe to run on a project it has already established. On such a run it SHALL refresh the instructions for the selected tools, and SHALL leave the project's accepted specifications, active changes, archive, **and runbooks** unchanged.

**A runbook is named in that list rather than left to be inferred from it.** A runbook is content a contributor wrote, so the reason the first three are protected applies to it unchanged, and a list that named three of four places would leave the fourth's protection resting on a reading.

The operation SHALL NOT overwrite a project's configuration once it exists. The operation SHALL NOT delete a file it did not write.

Where the asset store is already present, the operation SHALL verify it against the pinned runtime and report a discrepancy rather than silently replacing it. Bytes that differ from the pin are a fact the contributor needs, and overwriting them would hide it.

#### Scenario: Re-running on an established project
- **WHEN** the bootstrap operation runs on a project it already established
- **THEN** the instructions for the selected tools are refreshed and the project's specifications, changes, archive, and runbooks are unchanged

#### Scenario: A runbook survives a repeated bootstrap
- **GIVEN** a project holding a runbook a contributor wrote
- **WHEN** the bootstrap operation runs again
- **THEN** the runbook's documents are as they were

#### Scenario: Existing configuration survives
- **GIVEN** a project whose configuration has been edited
- **WHEN** the bootstrap operation runs again
- **THEN** the configuration is left as it was

#### Scenario: An existing store is verified rather than replaced
- **GIVEN** a project whose stored assets differ from the pinned runtime
- **WHEN** the bootstrap operation runs again
- **THEN** the discrepancy is reported and the stored bytes are not silently replaced

#### Scenario: Nothing unowned is removed
- **WHEN** the bootstrap operation runs on a project containing instructions for tools it does not manage
- **THEN** those files are left in place

### Requirement: Instructions installed in a shared file are confined to a marked region

Where a tool reads its instructions from a file the contributor also writes, the bootstrap operation SHALL confine what it writes to a region marked at both ends, and on a repeated run SHALL rewrite only that region.

Where the region is absent, the operation SHALL add it. Where the file's markers are incomplete, the operation SHALL report that and write nothing to that file, because a region whose end cannot be located cannot be replaced without risking the contributor's own text.

#### Scenario: First installation into a shared file
- **GIVEN** a project with a shared instructions file containing the contributor's own content
- **WHEN** the bootstrap operation installs into it
- **THEN** a marked region is added and the existing content is preserved

#### Scenario: Repeated installation into a shared file
- **WHEN** the bootstrap operation installs into a shared file that already has its marked region
- **THEN** only the content within the region changes, and the file does not grow

#### Scenario: Incomplete markers
- **GIVEN** a shared instructions file whose region is opened but not closed
- **WHEN** the bootstrap operation runs
- **THEN** it reports the incomplete region and writes nothing to that file

### Requirement: Bootstrapping ends by naming the command to run next

On success, the bootstrap operation SHALL state what was installed, where, and the literal command to invoke next for the tools that were selected.

The command named SHALL be the one that begins the lifecycle. A bootstrap is a first run, and naming the stage a contributor reaches last describes work they have not started.

Which workflow begins the lifecycle SHALL be a property of the workflow set rather than a value fixed at the point the message is produced, so that the message follows the set when the set changes.

Where the selected tools invoke the workflow by different spellings, the operation SHALL state each spelling and the tools it applies to. A single canonical form that no selected tool answers to would teach a command that does not work.

#### Scenario: Naming the next command
- **WHEN** the bootstrap operation succeeds for a single tool
- **THEN** the output states the literal command that tool answers to for the workflow that begins the lifecycle

#### Scenario: Tools with different spellings
- **WHEN** the bootstrap operation succeeds for tools that invoke the workflow differently
- **THEN** each spelling is stated with the tools it applies to

#### Scenario: The named command is the entry point
- **WHEN** the bootstrap operation succeeds
- **THEN** the workflow it names is the one that begins the lifecycle, not the one that ends it

#### Scenario: No tools selected
- **WHEN** the bootstrap operation succeeds having installed for no tools
- **THEN** it reports the project was established and does not name a command no tool answers to

### Requirement: A declined or interrupted operation is distinguishable from a failure

Where a contributor declines a prompt or interrupts the operation, the program SHALL exit with a status distinct from both success and from a failure to perform the operation. A job that cannot tell a person's refusal from a broken tool cannot report either one correctly.

An interrupted prompt SHALL restore the terminal before the program exits.

#### Scenario: Declining at a prompt
- **WHEN** a contributor declines a prompt the bootstrap operation presents
- **THEN** the program makes no further change and exits with the status for a cancelled operation

#### Scenario: Cancellation is not an operational failure
- **WHEN** a contributor cancels the operation
- **THEN** the exit status differs from the one used when an operation could not run

#### Scenario: The terminal is restored
- **WHEN** a contributor interrupts an interactive prompt
- **THEN** the terminal is left usable

### Requirement: Archival checks the versions of runbooks it publishes

In addition to checking the project's declared version, archival SHALL compare each runbook's
declared version and canonical specification and procedure against its most recent published
snapshot. Snapshots SHALL be ordered by recorded project versions, not archive-directory names,
and SHALL carry the runbook version and content fingerprints in a form readable without Git.

For a runbook with no prior snapshot, publication SHALL require a version greater than `0.0.0`.
Changed canonical documents SHALL require a strictly greater runbook version. An unchanged
runbook SHALL be allowed to retain its version or advance it with a matching changelog entry.
A downgrade, reusing a version for different canonical content, or a missing version or changelog
heading SHALL refuse archival. A malformed existing snapshot SHALL be reported, not treated as
an absent predecessor. Adding or editing a run record SHALL NOT require a version increment.

Publication SHALL record snapshots in the same archival transaction as the change and project
version. A refusal, cancellation, or failed publication SHALL leave the runbook, previous archive,
and active change unchanged. Readiness-only archival SHALL apply the same comparison and write
nothing. Archival SHALL NOT choose a version increment or write changelog prose for the author.

Runbook identity for this comparison SHALL be its directory name, as already required by
`runbook-authority`. A renamed runbook SHALL begin a new publication lineage; former snapshots
SHALL remain under the former name. A deleted runbook SHALL not cause deletion of its snapshots.
An absent source archive on a receiving machine SHALL NOT prevent preflight or running a carried
release; these operations do not publish a release.

#### Scenario: A changed procedure without a version increment
- **GIVEN** a published runbook whose procedure has changed
- **WHEN** archival sees the same runbook version as the last snapshot
- **THEN** archival refuses before moving the change or recording any version

#### Scenario: A version advances
- **WHEN** a changed runbook declares a greater version and its changelog names that version
- **THEN** the runbook version gate permits publication and the completed archive records its new snapshot

#### Scenario: Only a run record changes
- **WHEN** a runbook's canonical documents are unchanged and a new run record exists
- **THEN** archival does not require that runbook's version to advance

#### Scenario: The first independent runbook release
- **WHEN** an existing runbook gains its first checked version and has no earlier snapshot
- **THEN** archival requires a version above `0.0.0` and records the adoption without fabricating older releases

#### Scenario: An archive directory is renamed
- **WHEN** archival selects the latest snapshot after an archive directory was renamed
- **THEN** the recorded project versions still determine the same predecessor

#### Scenario: A malformed prior snapshot
- **WHEN** a runbook's previous snapshot cannot be parsed
- **THEN** archival reports the defective record and does not accept the runbook as a first release

#### Scenario: Readiness-only archival
- **WHEN** archival is asked to check a changed unversioned runbook without performing
- **THEN** it reports the same refusal as publication and leaves all files unchanged

### Requirement: The release suite checks published runbook version history

The repository release suite SHALL check that every published runbook snapshot has a parseable
version and fingerprints, that changed canonical content advances its version, and that each
published version has a changelog heading in the published snapshot. Repeated snapshots of an
unchanged runbook at one version SHALL count as the same release. Distinct releases for that
runbook SHALL have increasing versions, and a lower version SHALL be rejected even when content
is unchanged.

The suite SHALL also check that each current runbook has a valid version and corresponding
changelog heading. It SHALL NOT require unarchived edits to have advanced their version yet.
Archives predating adoption of runbook publication snapshots SHALL remain valid without invented
history. Once snapshots are introduced, each later archive SHALL explicitly record its runbook
inventory, including an empty inventory, so a missing inventory cannot masquerade as old history.

#### Scenario: A published version is reused for changed content
- **WHEN** the release suite reads two successive snapshots of one runbook with different content and the same version
- **THEN** it fails and names the runbook and conflicting archives

#### Scenario: Work in progress has not chosen its next version
- **WHEN** canonical documents differ from the last snapshot while their current version still has its changelog heading
- **THEN** release hygiene passes and the archival readiness check reports the pending version increment

#### Scenario: An inventory disappears after adoption
- **WHEN** an archive at or after the first recorded adoption release lacks its runbook inventory
- **THEN** the release suite reports the missing publication record rather than applying the legacy exemption
