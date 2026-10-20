## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: The command surface names its operations as OpenSpec names them

The project SHALL provide one command-line program offering, at minimum, a subcommand to validate documents, a subcommand to format documents, a subcommand to bootstrap a project, and a subcommand to create documents. Each subcommand SHALL be usable without configuration in a conforming project.

**Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it.** This holds for the subcommand and for the options that select what it acts on. Every plausible adopter arrives from OpenSpec — HTMLSpec is a spinoff and its README says so — and a contributor who has learned one surface SHALL NOT have to learn a second spelling for the same operation. An operation OpenSpec does not provide takes the name this project chooses for it.

The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, `sync`, `show`, `instructions`, and `store` name the operations OpenSpec also provides. `format` and `assets` name operations it does not. The selectors `--json`, `--specs`, `--change`, `--store`, and `--diff` carry OpenSpec's spellings for the same reason.

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
