## MODIFIED Requirements

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
