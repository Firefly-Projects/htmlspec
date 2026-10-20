## ADDED Requirements

### Requirement: A project has a declared root holding one asset store

An HTMLSpec project SHALL be identified by a directory containing an `htmlspec/` directory. That directory SHALL hold the project's accepted specifications, its active changes, and its archive, each in its own place.

A project SHALL hold exactly one asset store, in its root, and every document in the project SHALL reference that store rather than a store of its own. A project storing one copy of its runtime is the whole point of the duplicate-detection rule the contract already states; a tool that gave each change its own copy would be reporting a defect it created itself.

#### Scenario: Bootstrapping an empty repository
- **WHEN** a contributor bootstraps a project in a directory with no project in it
- **THEN** the project root is created with places for accepted specifications, active changes, and the archive, and one asset store is established in it

#### Scenario: Two changes share one store
- **GIVEN** a bootstrapped project
- **WHEN** two changes are created in it
- **THEN** the project holds one copy of the runtime assets, and each change's documents reference it

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

The bootstrap operation SHALL write an initial version that is ordered before every version a project can later declare, so that a project always has a version on record and the comparison is total. Archival SHALL NOT treat an absent previous version as grounds to pass. A project's first archival SHALL advance past the initial version exactly as a later one advances past its predecessor, because a rule with an exception that applies once per project is a rule exercised once per project.

Archival SHALL NOT modify the declaration. Advancing the version SHALL be the contributor's or the agent's act, and the program's part SHALL be the comparison alone.

That division is the one archival already uses: an agent produces the change's specification and the program validates it, because the judgment is judgment and the checking is exact. Deciding what a version increment signifies, and by how much, is judgment. Whether one version is ordered after another is not. A command whose stated job is checking SHALL NOT write to the project it checks.

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

The set of supported tools SHALL be data rather than logic, so that supporting an additional tool is an entry in that set.

How a tool receives instructions SHALL be a property of the tool with one value per tool, rather than a combination of independent settings. A tool for which no instructions could be written is a state the program SHALL NOT be able to reach.

Instructions SHALL record the version of the program that generated them, so an installation predating the instructions it should carry is identifiable.

#### Scenario: Installing for a tool with commands
- **WHEN** the bootstrap operation installs instructions for a tool that supports invocable commands
- **THEN** both the instructions and the commands are written where that tool reads them

#### Scenario: Installing for a tool without commands
- **WHEN** the bootstrap operation installs instructions for a tool that supports instructions only
- **THEN** the instructions are written and no command files are created

#### Scenario: Every selected tool receives something
- **WHEN** the bootstrap operation completes for any selected tool
- **THEN** that tool has received instructions, because a selected tool that receives nothing is not a representable outcome

#### Scenario: Generated instructions name their version
- **WHEN** instructions are installed
- **THEN** they record the version of the program that generated them

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

The bootstrap operation SHALL be safe to run on a project it has already established. On such a run it SHALL refresh the instructions for the selected tools, and SHALL leave the project's accepted specifications, active changes, and archive unchanged.

The operation SHALL NOT overwrite a project's configuration once it exists. The operation SHALL NOT delete a file it did not write.

Where the asset store is already present, the operation SHALL verify it against the pinned runtime and report a discrepancy rather than silently replacing it. Bytes that differ from the pin are a fact the contributor needs, and overwriting them would hide it.

#### Scenario: Re-running on an established project
- **WHEN** the bootstrap operation runs on a project it already established
- **THEN** the instructions for the selected tools are refreshed and the project's specifications, changes, and archive are unchanged

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

Where the selected tools invoke the workflow by different spellings, the operation SHALL state each spelling and the tools it applies to. A single canonical form that no selected tool answers to would teach a command that does not work.

#### Scenario: Naming the next command
- **WHEN** the bootstrap operation succeeds for a single tool
- **THEN** the output states the literal command that tool answers to

#### Scenario: Tools with different spellings
- **WHEN** the bootstrap operation succeeds for tools that invoke the workflow differently
- **THEN** each spelling is stated with the tools it applies to

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
