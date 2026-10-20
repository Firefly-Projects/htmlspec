## MODIFIED Requirements

### Requirement: Bootstrapping installs agent instructions for the tools a project uses

The bootstrap operation SHALL install, for each selected AI coding tool, the instructions that tool needs to drive the HTMLSpec workflow. Where a tool supports invocable commands, the operation SHALL install those as well as the instructions.

The instructions SHALL cover the project's lifecycle from an idea to an archived change, rather than a single stage of it. A contributor who has bootstrapped a project and lists what was installed SHALL find the stage they are at. Instructions covering only the final stage leave every earlier stage to be improvised, which is the case the project's own contract for artifact kinds exists to prevent.

A workflow SHALL be installed whether or not a command performs it. Archival is already agent-driven — the program checks a specification it does not produce — so a stage whose work is judgment is the ordinary case rather than the exception.

An installed workflow's instructions SHALL NOT name a command the program does not provide, and this SHALL be checked mechanically rather than by review. Naming an absent command sends an agent to an error, and it is the hazard that a rule limiting how many workflows are installed was attempting to avoid.

The set of supported tools SHALL be data rather than logic, so that supporting an additional tool is an entry in that set. The set of workflows SHALL likewise be data, so that covering an additional stage is an entry in that set together with its instructions.

The order of the workflows SHALL be the order the lifecycle moves through them, because instructions written into a shared file are presented in that order and a reader meets them as a document.

How a tool receives instructions SHALL be a property of the tool with one value per tool, rather than a combination of independent settings. A tool for which no instructions could be written is a state the program SHALL NOT be able to reach.

Instructions SHALL record the version of the program that generated them, so an installation predating the instructions it should carry is identifiable.

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
