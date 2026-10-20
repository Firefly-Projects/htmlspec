## Purpose

Defines how an existing OpenSpec project becomes an HTMLSpec project: what is enumerated
mechanically, what an agent must read and decide, the plan that records every decision before
anything is written, the staging location the migrated project is assembled in, the order the
stages run in, and the checks that gate promotion. It exists because HTMLSpec is a spinoff of
OpenSpec and every plausible adopter already has an OpenSpec project, and because the
conversion that looks obvious — markdown to HTML, file by file — produces a valid project that
has lost the reasoning it was migrating.

## Requirements

### Requirement: Migration surveys the source before anything is read or written

Migration SHALL begin by enumerating every entity in the source project, and the enumeration
SHALL be mechanical. The entities SHALL include the project's accepted specifications, its
active and archived changes, each change's documents, each change's delta specifications, each
change's evidence files, each change's recorded version, and the project's configuration.

The enumeration SHALL be derived from what the source holds rather than from a list of the entity
kinds migration expects. A source project holds files a mapping was not written for — content in
the destination format from an earlier hand conversion, per-change configuration, evidence
arranged in nested directories — and an enumeration built from the expected kinds omits them.
Every later stage is checked against this enumeration, so an entity absent from it is an entity
the coverage check cannot report, and the check passes while the content is lost. That is the
failure this requirement exists to prevent, and it is worse than a missing mapping because it is
silent.

Accordingly, the enumeration SHALL descend through the directories it enumerates rather than
matching their immediate contents, and where it reports a document's fields it SHALL read the
fields the document carries rather than the fields migration knows the name of.

The survey SHALL write nothing to the source project and nothing to the destination. It SHALL
be repeatable, and running it SHALL NOT depend on any earlier stage having run.

The survey SHALL be available as a structured payload, so that a later stage can be checked
against it rather than against a re-reading of the source. A coverage check that re-enumerated
the source would be checking the plan against a second opinion rather than against the list
the plan was written from.

The survey SHALL NOT interpret what it finds. Enumeration is exact because the source layout
is fixed; deciding what a document argues is not, and a survey that reported both would
present a judgment as a fact.

#### Scenario: Surveying a project with archived changes
- **WHEN** migration surveys a project holding accepted specifications and archived changes
- **THEN** every specification, every change, every document within a change, every delta specification, every evidence file, and every recorded version is enumerated

#### Scenario: A file no mapping was written for
- **WHEN** a source project holds a file that corresponds to no entity kind migration has a mapping for
- **THEN** the survey enumerates it anyway, so that the coverage check can report it

#### Scenario: Content nested below the directory that holds it
- **WHEN** a source project arranges a change's evidence in directories beneath the one the format names
- **THEN** every file is enumerated, rather than only those directly within it

#### Scenario: The survey writes nothing
- **WHEN** the survey runs
- **THEN** neither the source project nor the destination is modified

#### Scenario: The survey is repeatable
- **WHEN** the survey runs twice against an unchanged source
- **THEN** it reports the same entities both times

#### Scenario: A structured survey
- **WHEN** structured output is requested
- **THEN** the enumeration is emitted as a payload a later stage can be checked against

### Requirement: The stages that capture the work run before the stages that produce documents

Migration SHALL proceed in this order:

1. **Survey.** Enumerate the source. Nothing is read for meaning and nothing is written.
2. **Read.** Open every source document and record what each one does in the project.
3. **Gather evidence.** Collect the change evidence, the recorded versions, the archive dates,
   and the project's release record.
4. **Plan.** Write the migration plan and report it.
5. **Stage.** Assemble the migrated project in the staging location.
6. **Check.** Run the project's conformance checks over the complete staged project.
7. **Promote.** Move the staged project into place.

The first four stages SHALL produce no document in the destination format. Their output is a
plan a contributor reads, and migration SHALL stop after the plan and report it before staging
begins.

The reading stage SHALL record, for each source document, what that document is doing in the
project rather than a restatement of its content. A migration that proceeds from an
enumeration alone has the information a converter has, and a converter is what this ordering
exists to prevent.

The evidence stage SHALL run before any document is produced. A change's evidence is the
record of why the project has the shape it does, and it is the first thing a bulk conversion
loses because nothing in a source document points at it.

#### Scenario: Nothing is written before the plan
- **WHEN** migration reaches the point of reporting its plan
- **THEN** no document in the destination format exists, in the destination or in the staging location

#### Scenario: The plan is reported before staging
- **WHEN** the plan is complete
- **THEN** migration reports it and does not begin staging in the same step

#### Scenario: A reading pass that records purpose
- **WHEN** the reading stage completes
- **THEN** each source document has a recorded statement of what it does in the project, distinct from a summary of what it says

#### Scenario: Evidence gathered before documents are produced
- **WHEN** migration produces its first document in the destination format
- **THEN** the source project's evidence files, recorded versions, and archive dates have already been collected

### Requirement: The plan accounts for every surveyed entity

Migration SHALL produce a plan naming every entity the survey found, and for each one SHALL
record its destination and its disposition. A disposition SHALL be one of: converted into a
document of a named kind, folded into another artifact, carried across unchanged, or dropped.

A folded or dropped entity SHALL carry the reason it was folded or dropped. An entity that is
dropped because the destination format has nowhere to put it SHALL say so, and SHALL be
distinguishable from an entity that was empty in the source. Those are different facts about
the project a contributor is receiving.

The plan SHALL be expressed as a document of the `tasks` kind. A plan is a set of work items
with states, which is what that kind carries, and each item SHALL be individually addressable
so a contributor can reject one entity's disposition without rejecting the plan.

Migration SHALL NOT introduce an artifact kind for the plan. The set of kinds is closed, and a
working record used once per project is not grounds to reopen it.

The plan SHALL survive the migration, landing in the migrated project as the migration's own
change. A migration whose record was discarded would be the one act in the project's history
with no artifact behind it.

#### Scenario: Every entity has a disposition
- **WHEN** the plan is complete
- **THEN** every entity the survey found appears in it with a destination and a disposition

#### Scenario: An entity with nowhere to go
- **WHEN** a source entity carries content the destination format's contract forbids storing
- **THEN** the plan names it as dropped and states that the destination has no place for it

#### Scenario: An empty source field
- **WHEN** a source entity that could have carried content carried none
- **THEN** the plan records that it was empty, distinctly from recording that its content was dropped

#### Scenario: The plan is a tasks document
- **WHEN** the plan is validated
- **THEN** it is a valid document of the `tasks` kind, and each entity's disposition is an individually addressable unit

#### Scenario: The plan survives
- **WHEN** migration completes
- **THEN** the plan is present in the migrated project as part of the migration's own change

### Requirement: Promotion refuses while any surveyed entity is unaccounted for

Promotion SHALL compare the plan against the survey and SHALL refuse while any surveyed entity
has no disposition in the plan. The refusal SHALL name the unaccounted entities.

This SHALL be the division the project already uses for archival: the agent decides what each
entity becomes, and the program checks that a decision was recorded for all of them. The
program SHALL NOT judge whether a disposition is correct, because that judgment is the thing
the plan exists to put in front of a contributor.

An entity dropped with a stated reason SHALL satisfy the check. Being accounted for is what
the check tests, not being converted.

#### Scenario: An entity missing from the plan
- **WHEN** promotion runs against a plan that omits a surveyed entity
- **THEN** promotion does not complete, and the report names the entity that has no disposition

#### Scenario: A deliberate drop passes
- **WHEN** promotion runs against a plan in which an entity is dropped with a stated reason
- **THEN** the coverage check is satisfied

#### Scenario: The program does not judge a disposition
- **WHEN** a plan gives an entity a disposition a reader would disagree with
- **THEN** the coverage check reports nothing, because whether a disposition is right is what the plan is reviewed for

### Requirement: The migrated project is assembled in a staging location and promoted in one step

Migration SHALL assemble the migrated project in a staging location outside the destination
project, and the destination SHALL receive it in a single promotion.

The staging location SHALL hold a complete project — a declaration, the places the project
keeps its specifications, its changes, and its archive, and one asset store — so that the
project's own conformance checks run against it exactly as they will run after promotion. A
staging area that is not a resolvable project would push every check past promotion and make
promotion the step that discovers faults.

The staged project's internal paths SHALL be those the destination will hold, so that
promotion is a move. Each document references the project's asset store by a path relative to
its own location, so a staged tree at a different depth would require every reference to be
rewritten after the checks that would have caught a mistake in the rewriting.

The staging location SHALL NOT be inside the destination project. A project holds exactly one
asset store, and a second store within it is the condition the asset-topology check exists to
report.

Where migration is abandoned or fails at any stage before promotion, the destination SHALL be
unchanged.

#### Scenario: Checks run in the staging location
- **WHEN** the project's conformance checks run against the staged project
- **THEN** they resolve it as a project and check it as they would after promotion

#### Scenario: Promotion is a move
- **WHEN** the staged project is promoted
- **THEN** each document occupies the path it held in staging, relative to the project root, and its asset references resolve without rewriting

#### Scenario: Staging outside the destination
- **WHEN** the staging location is chosen
- **THEN** it lies outside the destination project, so the destination never holds a second asset store

#### Scenario: An abandoned migration
- **WHEN** migration stops at any stage before promotion
- **THEN** the destination project is as it was before migration began

#### Scenario: A half-migrated destination is not reachable
- **WHEN** migration fails while staging documents
- **THEN** the destination holds either the whole migrated project or none of it

### Requirement: The conformance checks run last, over the complete staged project

Migration SHALL run the project's validation, formatting, and asset-topology checks after the
staged project is complete, and SHALL NOT complete promotion while validation reports an error.

Migration SHALL NOT halt at the first document that fails a check. It SHALL complete the
staged project and then report every failure together.

The reason is that a mapping fault is systematic. A rule that is wrong for one scenario is
wrong for every scenario, and halting on the first occurrence presents a class of failure as
one document's defect — which is repaired one document at a time, in a tree that is now
partly migrated. Reporting the whole set makes the shape of the fault visible, and the shape
is what points at the repair.

Checking a document while drafting it SHALL remain permitted. What is prohibited is a
migration that ends at the first failure rather than at the last.

#### Scenario: Checks gate promotion
- **WHEN** validation reports an error against the staged project
- **THEN** promotion does not complete

#### Scenario: A systematic fault reported as a set
- **WHEN** the same mapping fault produces a diagnostic in many staged documents
- **THEN** the staged project is completed and every occurrence is reported together, rather than migration ending at the first

#### Scenario: Drafting checks are permitted
- **WHEN** an agent validates a document as it stages it
- **THEN** nothing prohibits that, because the obligation is on where migration ends

#### Scenario: A clean staged project
- **WHEN** the checks report no error against the complete staged project
- **THEN** promotion proceeds

### Requirement: Conversion of a document's meaning is agent-driven

Migration SHALL NOT provide, and SHALL NOT be performed by, a program that reads a source
proposal, design, or tasks document and emits a document in the destination format.

The reason is the one the project already holds for synthesis: those kinds carry no imposed
structure in either format, so there is no grammar to read. Deciding which paragraph of a
design states an obligation, which comparison is a table and which is a diagram, and what a
proposal's argument was actually for is reading rather than parsing. A converter would emit
documents that satisfy every check and lose the reasoning that made the migration worth doing.

An agent performing a migration SHALL preserve the argument, the reasoning, and the evidence
of each source document, and SHALL use the destination format's expressive capacity where the
source used prose because it had nothing better. A migration whose output is the source's
paragraphs wrapped in markup has moved the bytes and delivered none of the reason for moving.

#### Scenario: No converter exists
- **WHEN** the migration operation's command surface is examined
- **THEN** no command reads a proposal, a design, or a tasks document and emits a document in the destination format

#### Scenario: Reasoning that outgrew its source format
- **WHEN** a source document expresses a comparison, a flow, or a state machine in prose because its format offered nothing better
- **THEN** the migrated document expresses it in the destination format's own terms rather than as the same paragraphs in markup

#### Scenario: A migration that only re-wrapped
- **WHEN** a migrated document is the source document's paragraphs with markup around them and nothing else
- **THEN** the migration has not been performed, whatever the checks report

### Requirement: Evidence crosses unchanged

A source change's evidence files SHALL be carried into the migrated change unchanged, and
SHALL NOT be converted into the destination format.

Evidence is a transcript, a measurement, or a recorded complaint, and its worth is that nobody
edited it. Converting one is rewriting a record, and a rewritten record cannot serve the
purpose evidence serves.

The migrated change's documents SHALL reference the carried evidence, so that a reader reaches
it from the document that cites it rather than by finding it in a directory.

**Carried evidence SHALL NOT be subject to the conformance checks run over the staged project.**
Those checks are the project's own, and a change's evidence is not a document of the project.
So a record that is already in the destination format neither gates promotion nor is rewritten by
it, even at a superseded contract version or naming an asset store the project does not hold.
That includes a document preserved under "A source entity already in the destination format is
not assumed usable". A requirement to carry records unchanged, paired with a check that refuses
them, would leave a migration only two choices: edit the record, or leave it behind.

#### Scenario: Evidence carried verbatim
- **WHEN** a change carrying evidence files is migrated
- **THEN** each evidence file is present in the migrated change with its contents unchanged

#### Scenario: Evidence is referenced
- **WHEN** a migrated document makes a claim its change recorded evidence for
- **THEN** the document references the carried evidence file

#### Scenario: Carried evidence in the destination format does not gate promotion
- **GIVEN** a staged project whose documents all conform, and a migrated change whose evidence holds an HTML file declaring an earlier contract version and naming an asset store the project does not hold
- **WHEN** promotion's conformance checks run over the staged project
- **THEN** they report nothing about that file, promotion proceeds, and the file arrives byte-identical

### Requirement: A change's delta specifications collapse into that change's one specification

Where a source change carries specification deltas for several capabilities, migration SHALL
produce one specification artifact for that change, carrying every requirement the deltas
stated, with each requirement's change state recorded on the requirement itself.

A change directory in the destination format holds at most one specification artifact, so the
collapse is required rather than optional. How the collapsed document arranges several
capabilities' requirements SHALL be the agent's decision, taken with the change read, and
migration SHALL NOT impose an arrangement.

**A delta's content that is not a requirement SHALL be accounted for as well.** A source delta can
carry a statement of what its capability is for, and such a statement is neither a requirement nor
a scenario, so a collapse performed by mapping requirement and scenario units alone drops it
without reporting anything. Where several deltas each carry one, the destination document has a
single place such a statement would naturally sit and cannot hold several, which is what makes
this the part of the collapse that needs stating rather than the arrangement.

Where the collapsed specification carries text that appears nowhere in the source, migration SHALL
be able to state why it was needed. A specification is required to let a reader determine what its
change established without reading the change's other documents, and a source that states only
what each capability is for does not state what the change did — so a document collapsing several
deltas may need a statement of the change's own subject that no delta contains.

Where a unit kind the destination's specification artifact carries has no counterpart in the
source, migration SHALL omit it and record the omission rather than manufacture it. Deriving such
units from a source document that was not a record of them invents a record, and an invented
record is worse than an absent one because a reader cannot tell it apart from a real one.

Where the source change's implemented result disagrees with what its deltas stated, the
implemented result SHALL be what the migrated specification records. This is the rule archival
already applies, and a migration that restated intent while the code did otherwise would carry
that disagreement into the accepted contract.

**A requirement a source delta modified SHALL be stated whole,** as artifact-kinds' "A change's specification states a modified requirement whole" requires of every change's specification: every statement and every scenario the requirement held once the change's contract was merged, each scenario carrying its own change state and the identifier its accepted counterpart carries, and each scenario the change withdrew stated as a unit marked `removed`. A migrated specification is a change's specification like any other, and an archived one is read without the accepted specification of its day. A scenario's accepted counterpart is the scenario the migrated accepted specification holds under the same requirement with the same heading, where it holds one, so a scenario's identifier follows the obligation and not the document it is stated in.

**Where a source `MODIFIED` block left out a scenario the requirement held before, migration SHALL complete the block from the source's own merge.** An OpenSpec delta cannot say whether a scenario it leaves out was withdrawn or kept, so the delta alone does not settle it. The accepted specification as the source's merge left it does. A scenario it still held SHALL be stated `unchanged`, with the text it held. A scenario it no longer held SHALL be stated `removed`, with the text it held before the change. The accepted specification before and after the merge SHALL be read from the source's version history, at the commit that applied the change's contract. This project's own archive holds three such blocks. The merge kept the two scenarios one of them left out, and withdrew the one scenario each of the other two left out, so no single reading of an omission is right for all three.

Where the source's version history does not hold the accepted specification the change modified, migration SHALL put the omission to the plan's review as a question, naming the requirement and the scenario, and SHALL NOT resolve it by either reading. The plan SHALL record what migration did instead.

**The words a modification changed SHALL be marked.** In each requirement and each scenario the migrated specification marks `modified`, migration SHALL mark the words the change added with `ins` and the words it deleted with `del`, against the text that requirement or scenario held before the change, as document-validation's "Change markup in or around a semantic unit is rejected" permits in a change's specification. That previous text is the accepted text the change modified, read from the source's version history as the block's completion reads it. How finely to mark SHALL be the agent's decision: a paragraph rewritten throughout MAY be marked as deleted and added whole.

An author is permitted to leave a modification unmarked. A migration SHALL NOT, for three reasons. A migration has already compared each modified requirement with the text it replaced, to state each scenario's change state, so it holds the changed words without further work. A reader of an archived change cannot recover them later from the accepted specification, which has moved on since the change was merged. And without the marks, which words moved survives only where a summary happens to narrate it.

**The marks SHALL be written by the agent performing the migration, as the rest of the document is,** under "Conversion of a document's meaning is agent-driven". A comparison of the two texts is evidence the agent reads, not a document a program emits. Which words form one edit, and whether a paragraph reads better marked phrase by phrase or deleted and added whole, is a reading.

#### Scenario: A change touching several capabilities
- **WHEN** a source change carrying delta specifications for five capabilities is migrated
- **THEN** the migrated change holds one specification artifact carrying all of their requirements

#### Scenario: Change state per requirement
- **WHEN** a delta stated that a requirement was added, modified, or removed
- **THEN** the migrated specification records that state on the requirement rather than in a section heading

#### Scenario: The arrangement is the agent's
- **WHEN** several capabilities' requirements are collapsed into one specification
- **THEN** migration imposes no arrangement, and the agent decides it having read the change

#### Scenario: A delta's non-requirement content
- **GIVEN** several source deltas each stating what its capability is for
- **WHEN** they are collapsed into one specification
- **THEN** each statement is accounted for in the migrated document, rather than dropped because it is not a requirement

#### Scenario: A unit kind the source format has none of
- **WHEN** the destination's specification artifact carries a unit kind the source format does not express
- **THEN** the migrated specification omits it and the plan records why, rather than deriving units from a document that was not a record of them

#### Scenario: A modified requirement stated whole
- **WHEN** a source delta restates a requirement under `## MODIFIED Requirements`
- **THEN** the migrated requirement states every statement and every scenario it held once the change was merged, each scenario with its own change state and the identifier its accepted counterpart carries

#### Scenario: A scenario the block left out and the merge kept
- **GIVEN** a source `MODIFIED` block that leaves out a scenario the requirement held, and an accepted specification that still held that scenario after the change's merge
- **WHEN** the change is migrated
- **THEN** the migrated requirement states the scenario as `unchanged`, with the text the accepted specification held

#### Scenario: A scenario the block left out and the merge withdrew
- **GIVEN** a source `MODIFIED` block that leaves out a scenario the requirement held, and an accepted specification that no longer held it after the change's merge
- **WHEN** the change is migrated
- **THEN** the migrated requirement states the scenario as `removed`, with the text it held before the change

#### Scenario: An omission the source's history cannot settle
- **GIVEN** a source `MODIFIED` block that leaves out a scenario, and a source with no version history holding the accepted specification the change modified
- **WHEN** the plan is written
- **THEN** the plan puts the omission to its review, naming the requirement and the scenario, and migration neither keeps nor withdraws the scenario on its own reading

#### Scenario: The words a modification changed
- **GIVEN** a source delta that restates a requirement with one word added to a statement and another deleted, against the accepted text it modified
- **WHEN** the change is migrated
- **THEN** the migrated statement marks the added word with `ins` and the deleted word with `del`, the requirement's statements hold the delta's text, and its deletions hold the deleted word

#### Scenario: A modified requirement left unmarked
- **WHEN** a migrated specification marks a requirement `modified` whose statement differs from the text it replaced, and carries no change markup
- **THEN** the migration is incomplete, because the changed words were known to it and are known to no later reader

### Requirement: Migration conforms to the destination's stated layout and preserves the source's where none is stated

Where the destination format's contract states where an artifact lives or what it is named,
migration SHALL conform to it. Where the contract states no layout, migration SHALL preserve
the structure the source used, and SHALL NOT introduce a layout of its own.

A source capability addressed by a nested path SHALL retain that path in the destination, so
that a hierarchy the source expressed survives the move.

The reason is that a migration is the worst available moment to settle an open question about
the destination format. It settles it in bulk, for every artifact at once, inside a change
whose subject is something else. The convention then acquires a hundred instances and no
requirement behind it, and a later reader cannot distinguish a decision from an accident.

An example in a README, a template, or any other non-normative material SHALL NOT be treated
as a stated layout. Where migration finds only such an example, the contract is silent and the
source's shape is preserved.

Where preserving the source's shape is impossible because the destination's contract forbids
it, that conflict SHALL be recorded in the plan with the requirement that forbids it.

#### Scenario: A layout the contract states
- **WHEN** the destination's contract states where an artifact of some kind lives
- **THEN** migration places it there, whatever the source's arrangement was

#### Scenario: A layout the contract does not state
- **WHEN** the destination's contract states no layout for a kind of artifact
- **THEN** migration preserves the arrangement the source used

#### Scenario: A nested capability path
- **GIVEN** a source capability addressed by a nested path
- **WHEN** it is migrated
- **THEN** the nesting survives, rather than being flattened into a composed name

#### Scenario: An example mistaken for a contract
- **WHEN** a layout appears only in a README, a template, or other non-normative material
- **THEN** migration treats the contract as silent and preserves the source's shape

#### Scenario: A source shape the contract forbids
- **WHEN** the source's arrangement conflicts with a requirement of the destination's contract
- **THEN** the plan records the conflict and names the requirement that forbids the source's shape

### Requirement: A source distinction the destination cannot express is recorded as open, not mapped to the nearest state

Where a source entity carries a distinction the destination's contract has no value for,
migration SHALL record it as an open question and SHALL name the change that would settle it.
Migration SHALL NOT map the distinction onto the nearest available value.

The nearest available value is the tempting choice because it validates. A distinction mapped
onto it is indistinguishable, afterwards, from a source that never made the distinction — so the
information is not merely lost but unrecoverable, and no check can report it because the result
conforms.

Settling the question instead SHALL be out of scope for a migration. Adding a value to the
destination's vocabulary reaches validation, formatting, and presentation, and a migration
decides it in bulk inside a change whose subject is something else. This is the same reason
migration does not invent a layout where the contract is silent.

The plan SHALL record, for each such entity, what the source stated, that the destination has no
value for it, and what migration did instead.

**Once the destination can express a distinction, migration SHALL map it, and SHALL NOT record it as open.** An open question records a gap in the destination, and it is kept only while the gap is. A plan that went on recording a distinction the destination has since gained a value for would leave the migrated project carrying a workaround, or a question nobody needs to answer, in place of the value that settles it.

Whether the destination can express a distinction SHALL be judged against the contract the migrated project is staged and checked against, not the contract a plan was first written for. A plan written before the destination gained a value SHALL be revised to map the distinction before the documents it affects are staged.

The destination expresses each of the following, and migration maps each one where its own requirement says:

- the heading a source requirement replaced, as a `renamed-from` paragraph, under "A source rename is carried as the heading the requirement replaced";
- a task the source records as done otherwise than its words say, as a `done` task that states its deviation, under "A source task is carried in the state its record supports";
- the words a source modification changed, as change markup in the migrated specification, under "A change's delta specifications collapse into that change's one specification";
- a check that reached part of what it names, as a `partial` verification that states what it did not check, under "A check that reached part of what it names is carried as a partial verification";
- how a source risk was settled, as the risk's response, under "A source risk's stated settlement is carried as its response".

**Mapping these SHALL NOT extend the destination's vocabulary.** Each value was added by a change whose subject it was, and migration adds none of its own. A source distinction that still has no destination value is recorded as open, as before.

**A source change's creation date SHALL be neither mapped nor recorded as open.** The destination has no value for it because the project decided not to record one, since no command depends on one, and not because the change that would add one is still to come. An open question names the change that would settle it, and this one is settled. The date crosses unchanged inside the change's configuration file, wherever the plan carries that file, under "A source change's own configuration is surveyed, and its predecessor becomes its declaration".

#### Scenario: A change state the destination's vocabulary lacks
- **GIVEN** a source entity recording a distinction, and a destination whose change-state vocabulary has no value for it
- **WHEN** the entity is migrated
- **THEN** the plan records what the source stated, records that the destination has no value for it, and names the change that would add one

#### Scenario: The nearest state is not substituted silently
- **WHEN** a source distinction has no destination value
- **THEN** migration does not record it as the nearest available value and leave the plan silent, because the result would be indistinguishable from a source that never made the distinction

#### Scenario: The vocabulary is not extended by the migration
- **WHEN** migration finds a distinction the destination's vocabulary cannot express
- **THEN** migration does not add a value to that vocabulary, and names the change that would

#### Scenario: A distinction the destination has since gained a value for
- **GIVEN** a plan that recorded a source distinction as open, or carried it as a note, and a destination whose contract has since gained a value for it
- **WHEN** the documents it affects are staged
- **THEN** the distinction is carried as that value, and the plan no longer records it as open

#### Scenario: A mapped distinction is not a new value
- **WHEN** migration maps a source distinction to a value the destination expresses
- **THEN** that value was already in the destination's contract before the migration was staged

#### Scenario: A creation date is neither mapped nor open
- **WHEN** a source change's configuration file states `created:`
- **THEN** the plan records no open question for it, and the migrated change holds the date only inside the carried configuration file, where the plan carries that file

### Requirement: A source rename is carried as the heading the requirement replaced

Where a source delta carries a `## RENAMED Requirements` section, migration SHALL carry each
`FROM:`/`TO:` pair into the migrated change's specification as the renamed requirement's
`renamed-from` paragraph, stating the `FROM:` heading, on the requirement whose heading is the `TO:`
heading.

The requirement's change state SHALL be taken from the rest of the delta and not from the rename.
Where the same delta also restates the requirement under its `MODIFIED Requirements` section — which
is how OpenSpec records a rename and a rewrite together, the `MODIFIED` block naming the new heading —
the requirement SHALL carry `data-change="modified"`. Where the delta renames the requirement and says
nothing else about it, the requirement SHALL carry `data-change="unchanged"`, and its statements and
scenarios SHALL be those the requirement held before the rename.

Any prose the source's RENAMED section carried beside its pairs — an explanation of why the heading
changed — SHALL be carried into the migrated specification rather than dropped, because it is the
author's reason for the rename and no field holds it.

A rename SHALL NOT be recorded as an open question once the destination can express it. The
requirement "A source distinction the destination cannot express is recorded as open, not mapped to
the nearest state" governs distinctions the destination has no value for, and a rename is no longer
one.

#### Scenario: A rename with a rewrite
- **GIVEN** a source delta that renames a requirement in `## RENAMED Requirements` and restates it under the new heading in `## MODIFIED Requirements`
- **WHEN** the change is migrated
- **THEN** the migrated requirement carries the new heading, `data-change="modified"`, the rewritten content, and a `renamed-from` paragraph stating the previous heading

#### Scenario: A rename alone
- **GIVEN** a source delta that renames a requirement and states nothing else about it
- **WHEN** the change is migrated
- **THEN** the migrated requirement carries the new heading, `data-change="unchanged"`, the content it held before, and a `renamed-from` paragraph

#### Scenario: The author's reason for a rename
- **GIVEN** a RENAMED section carrying a paragraph that explains why the heading changed
- **WHEN** the change is migrated
- **THEN** that paragraph appears in the migrated specification

#### Scenario: A rename is not an open question
- **WHEN** a migration plan accounts for a delta carrying a RENAMED section
- **THEN** the plan records the rename's destination as a `renamed-from` paragraph, not as an open question

### Requirement: A source entity already in the destination format is not assumed usable

Where a source project holds an entity already expressed in the destination format, migration
SHALL check it against the destination's current contract rather than carrying it on the strength
of its format.

Such an entity SHALL be treated as a source entity like any other: it appears in the survey, it
receives a destination and a disposition in the plan, and where it does not conform its
disposition SHALL state why. Being in the right format is not the same as conforming to the
contract, because a document can declare a superseded contract version or reference a project
resource that no longer exists.

Where such an entity cannot be carried as a document, migration SHALL preserve it rather than
discard it, and SHALL state that the disposition is its own choice rather than the destination's
requirement.

#### Scenario: A document in the destination format at a superseded contract version
- **WHEN** a source project holds a document in the destination format declaring an earlier contract version
- **THEN** it is surveyed and given a disposition, and the plan states that its format did not make it conforming

#### Scenario: A document whose references no longer resolve
- **WHEN** a source document in the destination format references a project resource that no longer exists
- **THEN** migration does not carry it into the destination as a document, because the destination's own checks exist to report exactly that condition

#### Scenario: Preserved rather than discarded
- **WHEN** an entity already in the destination format cannot be carried as a document
- **THEN** migration preserves it and states that the disposition is the agent's choice

### Requirement: Identifiers are derived once and every reference resolves through that derivation

Migration SHALL derive each requirement's identifier once, and every reference to that
requirement anywhere in the migrated project SHALL use the derived identifier. The derivation
SHALL be recorded in the plan.

Deriving independently at each site produces different identifiers for one obligation, each
locally valid, so nothing reports the divergence and cross-references silently fail to resolve.

The derivation SHALL be recorded against the source document that stated the heading, rather than
against the heading alone. A source project can state one continuing obligation under different
headings in different changes, because a change may rename a requirement, and each change's
specification states what that change established rather than what the project later called it.
A derivation keyed on the heading alone would either merge those statements — misrepresenting
every change but the last — or report collisions where the source is consistent.

**Where the source records a rename, the renamed requirement SHALL keep the identifier it already
held.** A requirement named in a source `## RENAMED Requirements` section is one obligation under a
new heading, and document-schema's "Every semantic unit has a stable unique identifier" requires a
renamed requirement to keep its identifier. So its identifier SHALL be the one derived from the
heading under which the requirement was first established, and a chain of renames SHALL carry that
one identifier through every change in the chain and into the accepted specification. The plan SHALL
record, against the document that stated each rename, the heading it replaced, the heading it
introduced, and the identifier both resolve to.

Where the source changed a heading without recording a rename — by removing one requirement and
adding another — the two SHALL derive their identifiers independently. The source did not state that
they are one obligation, and migration SHALL NOT infer it from their wording.

The derivation SHALL read the heading in full. A derivation that shortens or normalizes a heading
maps distinct obligations onto one identifier, which manufactures a collision the source did not
contain and refers the contributor to a conflict that is an artifact of the rule.

Where two source requirements in one destination document derive the same identifier, migration
SHALL report the collision rather than resolving it by suffixing. Two distinct obligations
distinguished only by a trailing digit read as versions of one, which is a worse outcome than a
migration that stops and asks. Collapsing several of a source change's specifications into one
document SHALL be understood as the operation most likely to produce such a collision, because it
is the one that brings requirements from different parts of the source into a single identifier
namespace.

#### Scenario: One requirement referenced from several documents
- **WHEN** a requirement appears in an accepted specification and is referenced by an archived change
- **THEN** both use the same derived identifier

#### Scenario: A derivation recorded
- **WHEN** the plan is read
- **THEN** each requirement's source heading, the document that stated it, and the identifier derived from it are all recorded

#### Scenario: One obligation renamed across changes
- **GIVEN** a source project in which a later change renamed a requirement an earlier change established, and recorded the rename in a `## RENAMED Requirements` section
- **WHEN** both changes are migrated
- **THEN** both changes' specifications carry the identifier derived from the heading the earlier change established, each states the heading that change used, and this is not reported as a collision

#### Scenario: A chain of renames
- **GIVEN** a requirement renamed by one change and renamed again by a later one
- **WHEN** the project is migrated
- **THEN** every change in the chain and the accepted specification carry the identifier derived from the first heading

#### Scenario: A heading changed without a recorded rename
- **GIVEN** a source change that removes one requirement and adds another with a similar heading, and records no rename
- **WHEN** it is migrated
- **THEN** the two derive their identifiers independently, and migration does not treat them as one obligation

#### Scenario: Two requirements deriving one identifier
- **WHEN** two source requirements' headings derive the same identifier within one destination document
- **THEN** migration reports the collision and does not resolve it by adding a suffix

#### Scenario: A collision the derivation rule created
- **WHEN** two source requirements state distinct obligations whose full headings differ
- **THEN** they derive distinct identifiers, because a rule that shortened them would report a collision the source did not contain

### Requirement: Migrated archives carry their versions where the archival gate reads them

Migration SHALL record each archived change's version in the form the destination project's
archival gate reads, so that a migrated project's version history is the history the gate
compares against.

That form SHALL be the version each archived change records, and migration SHALL NOT encode a
version in an archive directory's name. A directory name is a display string, and the
destination's contract states both halves of this: the archival gate reads a recorded version
rather than deriving one from a name, and an archived directory is named for its date followed
by the change's name. A migration that composed a version into the name would produce an archive
violating the naming rule and a version the gate does not read.

The migrated project's declared version SHALL be the greatest version found among the migrated
archives, so that the next archival is required to advance past the project's real history
rather than past an initial value.

Where a source change records no version, migration SHALL NOT invent one. The migrated archive
SHALL carry none, and the plan SHALL record that it carried none in the source.

Where several source changes record one version, migration SHALL carry that version onto each of
them and SHALL NOT invent distinct versions to separate them. The destination requires recorded
versions to be distinct and provides for this case with a release-record entry naming the changes
that shipped together, so migration SHALL carry the source's release record with it. A migrated
project whose release record lost that entry fails its own release check on arrival, reporting a
version collision it cannot explain.

#### Scenario: A migrated archive's version is readable by the gate
- **GIVEN** a migrated project whose archived changes recorded versions in the source
- **WHEN** the archival gate determines the last archived version
- **THEN** it reads the greatest version among the migrated archives

#### Scenario: A version is not encoded in a directory name
- **WHEN** an archived change carrying a version is migrated
- **THEN** the migrated archive's directory name carries no version component, and the version is recorded where the gate reads it

#### Scenario: The declaration matches the migrated history
- **WHEN** migration completes
- **THEN** the declared version equals the greatest version among the migrated archives, so the next archival must advance

#### Scenario: A source change with no version
- **WHEN** a source change records no version
- **THEN** the migrated archive carries none, and the plan records that the source had none

#### Scenario: Several source changes sharing one version
- **WHEN** several source changes record the same version
- **THEN** each migrated archive carries that version, no distinct versions are invented, and the source's release record explaining the shared version is carried with them

### Requirement: The source project survives migration

Migration SHALL NOT modify or delete the source project. Removing it SHALL be a separate act
a contributor takes after reading the result.

A migration that consumed its input would make its own faults unrecoverable, and every fault
this operation can have is a judgment fault discovered by reading the output.

Migration SHALL NOT convert a source project's generated agent instructions. Those are
produced by the destination project's own bootstrap operation from its compiled workflow set,
and converted instructions would name commands the destination program does not provide.

#### Scenario: The source is untouched
- **WHEN** migration completes
- **THEN** the source project's files are byte-for-byte as they were

#### Scenario: Generated instructions are not migrated
- **WHEN** the source project holds generated agent instructions or command files
- **THEN** migration does not convert them, and the destination's own bootstrap operation writes its replacements

#### Scenario: Recovering from a bad migration
- **GIVEN** a completed migration a contributor judges wrong
- **WHEN** they correct the plan and migrate again
- **THEN** the source is still available to migrate from

### Requirement: A source task is carried in the state its record supports

Migration SHALL carry each task of a source change's tasks document as a task unit, in the state that the source's record of that task supports:

- a task whose checkbox is ticked, `- [x]`, SHALL be `done`;
- a task whose checkbox is empty, `- [ ]`, SHALL be `todo`;
- a task the source marks as abandoned, with the marker `- [~]` or by striking its words through, SHALL be `dropped`, and the reason the source states SHALL be its reason, verbatim, whichever box the source ticked;
- a `done` task whose record says the work was done otherwise than the task's words say, or only in part, or beyond them, SHALL state what was done in a `deviation` paragraph, as document-schema's "A task declares completion state" provides.

The reason is that each of these is a distinction the source makes and the destination expresses, and every other mapping misreports it to whatever counts tasks. A task struck through and carried as `done` with a note is counted as work carried out. A task done another way and carried as `done` with a note is counted, and read, as done as its words say, because a note is prose that no field and no count reads. The same task carried as `dropped` is counted as abandoned work, although the work happened.

**A task's words SHALL be carried as written, whatever state it takes.** They record what was planned, and carried evidence cites a task by them. Where the source's own text of a task carries the account of how it was done, as a task reading "**Amended during implementation.**" does, that account SHALL be the task's deviation, in the author's words, and the rest of the text SHALL remain the task's words. Where the account is elsewhere, the deviation SHALL say what was done and SHALL name where that is shown: a record the change carries, which it links, or the source at a named commit.

**A struck-through task SHALL NOT carry change markup.** Its words SHALL NOT be wrapped in `del`: `dropped` already states that they were given up, and document-validation's "Change markup in or around a semantic unit is rejected" permits change markup in no task. The strike itself MAY be carried as `s`, which is presentational and claims no edit.

**Migration SHALL NOT write a reason the source does not state.** A dropped task with no reason is invalid, and an invented reason is worse than none, because a reader cannot tell it from the author's. Where the source marks a task abandoned and says nothing of why, the plan SHALL put the task to its review as a question. Where the review supplies no reason, the task SHALL take the state OpenSpec reads from its checkbox, `done` for a ticked box and `todo` for any other marker, with a note saying that the source abandoned it and stated no reason, and the plan SHALL record what the source stated and what migration did instead.

A `done` task whose record says nothing about its result SHALL stay `done` and SHALL NOT state a deviation, because a deviation states what was done, and there nothing records it. Migration MAY say so in a note.

#### Scenario: A ticked task
- **WHEN** a source task is `- [x]` and nothing in the change records it being done another way
- **THEN** the migrated task is `done` and states no deviation

#### Scenario: A task marked dropped with its reason
- **GIVEN** a source task `- [~] 5.2 **Dropped.** …` whose text says why it was not done
- **WHEN** it is migrated
- **THEN** the task is `dropped`, and its reason is the source's words

#### Scenario: A task ticked and struck through
- **GIVEN** a source task `- [x] 4.5 ~~…~~ **No action: the premise was wrong.** …`
- **WHEN** it is migrated
- **THEN** the task is `dropped` with the source's reason, its words are carried, and no `del` element wraps them

#### Scenario: An abandoned task with no reason
- **GIVEN** a source task struck through with nothing saying why
- **WHEN** the plan is written
- **THEN** the plan puts the task to its review, and migration writes no reason of its own

#### Scenario: A task done another way
- **GIVEN** a ticked source task whose change's evidence records that the work was done by another route
- **WHEN** it is migrated
- **THEN** the task is `done`, its words are as the source wrote them, and its deviation states what the evidence records and links to it

#### Scenario: The author's own account of the difference
- **GIVEN** a source task whose text carries "**Amended during implementation.**" and the account that follows
- **WHEN** it is migrated
- **THEN** that account, in the author's words, is the task's deviation

#### Scenario: A task nothing records
- **GIVEN** a ticked source task whose result no record of the change states
- **WHEN** it is migrated
- **THEN** the task is `done` and states no deviation

### Requirement: A unit the author's words name is referenced wherever the migrated project resolves it

Where a task or a decision in a migrated change names a requirement or a scenario in its author's words, and document-schema's "Cross-references address units by identifier" resolves a reference from that task's or decision's document to the unit, the migrated task or decision SHALL carry the unit's identifier in its `data-refs`. Where the author's words carry a link to the unit, the link SHALL stay beside the reference.

The reason is that a link to another document projects into nothing. Nothing checks that it points at a unit that exists, and nothing a consumer reads records that the task serves the requirement. A reference is both: validation resolves it, and a consumer reads it. The link stays because the page is read by people too, and a reference is not a hyperlink.

**The author's words SHALL name the unit**: by its heading, or by words that plainly are its heading. Migration SHALL NOT add a reference the author did not state, whether from a capability's name, from a task's subject, or from a unit the task's work happens to touch. A reference is a relationship in the model, and a relationship the author never stated is the migration's claim presented as theirs.

**A unit the migrated document cannot reach SHALL be named by a link alone.** That covers a unit that only another change's documents declare, a unit in the change's own proposal, design, or tasks, a unit that the place resolving the reference marks `removed`, and an accepted requirement that no accepted specification holds any longer. A reference to any of these would make the document invalid, and the link already carries the author's words and a reader's way to the unit.

**A reference across a migrated change's documents SHALL be checked in the staging location as it is after promotion.** The staged project resolves as a project, as "The migrated project is assembled in a staging location and promoted in one step" requires, so a staged reference reaches the staged change's specification and the staged accepted specifications. A reference that resolves nowhere is a validation error, and it gates promotion under "The conformance checks run last, over the complete staged project".

#### Scenario: A task that names its change's requirement
- **GIVEN** a source task whose words give the heading of a requirement its change's delta states
- **WHEN** it is migrated
- **THEN** the migrated task names that requirement's identifier in its `data-refs`, and its document is valid

#### Scenario: A task that names an accepted requirement
- **GIVEN** a source task whose words give the heading of a requirement the migrated accepted specifications hold, and its change's delta does not state
- **WHEN** it is migrated
- **THEN** the migrated task names that requirement's identifier in its `data-refs`

#### Scenario: A task that names another change's requirement
- **GIVEN** a source task whose words name a requirement only another change's delta states
- **WHEN** it is migrated
- **THEN** the task links to that requirement and carries no `data-refs` naming it

#### Scenario: A task that names a withdrawn scenario
- **GIVEN** a source task whose words name a scenario its change withdraws
- **WHEN** it is migrated
- **THEN** the task links to the scenario and carries no `data-refs` naming it

#### Scenario: Words that name only a capability
- **GIVEN** a source task that names a capability and no requirement in it
- **WHEN** it is migrated
- **THEN** the task carries no reference to any requirement of that capability

#### Scenario: A reference checked before promotion
- **GIVEN** a staged project in which a migrated task names, in its `data-refs`, a requirement that only its change's staged specification declares, and another names an identifier nothing in the staged project declares
- **WHEN** promotion's conformance checks run over the staged project
- **THEN** the first reference resolves and nothing is reported about it, the second is reported, and promotion does not proceed

### Requirement: A check that reached part of what it names is carried as a partial verification

Where migration writes a verification unit from a record the source change carries, and the record shows that its check reached part of the requirement or scenario the verification names and not the rest, the verification SHALL record the outcome `partial`, with one `data-field="unchecked"` paragraph for each part the record does not show, as document-schema's "A partial verification states what it did not check" requires. Each such paragraph SHALL name the part in the words of the requirement or scenario it belongs to.

**Migration SHALL NOT record such a check as `passed`, and SHALL NOT leave it out for being partial.** `passed` claims the part the record does not show. Leaving the verification out loses the record of the part it does show. This project's own staged migration did each twice: it kept two verifications as `passed` while their method paragraphs said what the record lacked, and it removed two in review because each record reached one clause of its scenario's outcome.

A check whose condition, subject, or trigger differs from the one a scenario states did not reach that scenario, and migration SHALL write no verification of it. `partial` records a check of the right thing that stopped short, not a check of something nearby.

This requirement decides what outcome a verification records. It does not widen where one is written: migration still writes one only where a carried record states the outcome of a check, because "A change's delta specifications collapse into that change's one specification" forbids deriving units from a document that was not a record of them.

#### Scenario: A check that reached part of its scenario
- **GIVEN** a carried record of a check whose outcome shows the first of two outcomes a scenario's Then states, and not the second
- **WHEN** migration writes the verification of that scenario
- **THEN** it records `partial`, with an `unchecked` paragraph naming the second outcome in the scenario's words

#### Scenario: A partial check is not recorded as passed
- **WHEN** a carried record shows a check that reached part of what it names
- **THEN** the migrated verification does not record `passed`

#### Scenario: A partial check is not left out
- **WHEN** a carried record shows a check that reached part of what it names
- **THEN** the migrated specification carries a verification of it

#### Scenario: A check of something nearby
- **GIVEN** a carried record of a check whose trigger differs from the one a scenario states
- **WHEN** the change's specification is migrated
- **THEN** no verification of that scenario is written from that record

#### Scenario: A check that reached all of it
- **WHEN** a carried record shows a check that reached everything a scenario states, and found it to hold
- **THEN** the migrated verification records `passed`, as it did before `partial` existed

### Requirement: A source risk's stated settlement is carried as its response

Where migration carries a source risk as a risk unit, the unit SHALL declare, in `data-response`, the settlement the source states for it, as document-schema's "A risk records how it was settled" provides:

- a risk the source marks accepted, or says is not mitigated, SHALL be `accepted`. Where the source also states a measure that holds part of it down, in a paragraph of its own, that paragraph SHALL be the risk's mitigation;
- a risk the source marks mitigated SHALL be `mitigated`, and its mitigation SHALL be the paragraph that states the measure.

**Migration SHALL declare a response only where the source states one.** A risk whose source states a mitigation and no settlement is `mitigated` by the attribute's default, and declaring it would add a word the author did not write. A risk whose source states neither a settlement nor a mitigation SHALL NOT be carried as a risk unit, because as a unit it would have to be declared `accepted` on the migration's reading, or fail as a mitigated risk with no mitigation. It stays in prose.

**A source risk SHALL become a unit only where it has the shape of one**: a heading the source gave it, and its statement and any measure against it each in paragraphs of their own. A unit made by splitting an author's paragraph, or by writing a heading the author did not write, is the migration's arrangement presented as the author's. The response changes this in one respect only. An accepted risk needs no mitigation, so a headed risk that the source accepts and states no measure against now has the shape of a unit.

The reason is that a source risk's settlement is a distinction its author made and the destination now expresses. Before it could, an accepted risk with no mitigation could not be a unit at all, and an accepted risk carried as a unit read as mitigated by default, which contradicted its author. This project's own archive has five headed accepted risks with no mitigation, and its staged migration drafted one accepted risk unit whose response defaulted to mitigated.

#### Scenario: An accepted risk with no measure
- **GIVEN** a headed source risk marked "Accepted" with no measure stated against it
- **WHEN** it is migrated
- **THEN** it is a risk unit with `data-response="accepted"` and no mitigation paragraph

#### Scenario: An accepted risk that names a partial measure
- **GIVEN** a headed source risk marked "Accepted", with a paragraph of its own stating a measure that holds part of it down
- **WHEN** it is migrated
- **THEN** it is a risk unit with `data-response="accepted"`, and that paragraph is its mitigation

#### Scenario: A risk the source says is not mitigated
- **WHEN** a headed source risk says it is not mitigated
- **THEN** the risk unit declares `data-response="accepted"`

#### Scenario: A mitigated risk
- **WHEN** a headed source risk is marked mitigated and states its measure in a paragraph of its own
- **THEN** the risk unit declares `data-response="mitigated"`, and that paragraph is its mitigation

#### Scenario: A risk with no stated settlement and no measure
- **WHEN** a headed source risk states neither how it was settled nor any measure against it
- **THEN** it stays in prose, and no risk unit is written for it

#### Scenario: A risk without a heading
- **WHEN** a source risk is a line in a list, with no heading of its own
- **THEN** it stays in prose, whatever settlement it states

### Requirement: A source change's own configuration is surveyed, and its predecessor becomes its declaration

The survey SHALL report, for each source change, whether the change holds a configuration file of its own. Where it does, the survey SHALL report every key the file sets, and the value of `follows:` where the file states one. It SHALL read the keys the file sets rather than the keys migration knows the name of, as it reads the project's configuration. A change with no configuration file SHALL be reported as having none, which is a different fact from a file that states no predecessor.

The survey reports these so that a later stage carries them from the survey rather than from a second reading of the source. "Migration surveys the source before anything is read or written" makes the survey the list a later stage is checked against, and a stage that read the file again would carry a value that no check compares with what the plan was written from.

**The survey SHALL report the value of `follows:` and of no other key.** `follows:` is the one value a later stage carries, because it becomes `FOLLOWS`. Any other value in the payload would be a field nothing consumes, and project-bootstrap's "A project declares its name, version, and expected toolchain" gives the reason against one: "A declaration no command reads states nothing that can be wrong, so nothing reports it when it is". `created:` is the case in point. The project records no creation date, because no command depends on one, so a date in the payload would have no reader. Its key is still reported, so the plan accounts for it.

**Where a change's configuration states `follows:`, the migrated change SHALL state its predecessor in `FOLLOWS`,** holding the value exactly as the source wrote it, as artifact-kinds' "A change states the change it follows" provides. Where it states none, the migrated change SHALL have no `FOLLOWS`. Migration SHALL NOT infer a predecessor from a change's name, its dates, or the requirements it states, because that requirement forbids the program the same inference, and a migration that made it would write the program's forbidden answer into the project's history.

**Mapping the file's values SHALL NOT be taken for the file's disposition.** The file is a surveyed entity, and the plan owes it a disposition like any other, with a reason where it is folded or dropped. A key it sets that the destination has no place for, such as `schema:` or `created:`, SHALL be accounted for in the plan. Whether the file itself also crosses, into the change's evidence, is the plan's choice: the contract is silent on it, and the plan SHALL say so.

**A source change's creation date crosses only inside its configuration file.** The destination records no creation date, so `created:` maps to nothing, and migration SHALL NOT derive a date for a change whose file states none. Where the plan carries the file, the date crosses unchanged with it. Where the plan folds or drops a file that states `created:`, its reason SHALL say that the date goes with the file, because nothing else in the destination holds it, and a loss the plan does not state is one no reader can find.

#### Scenario: A predecessor the source states
- **GIVEN** a source change whose configuration file reads `follows: log-a-runbook-run`
- **WHEN** it is surveyed and migrated
- **THEN** the survey's payload carries `log-a-runbook-run` for that change, and the migrated change's `FOLLOWS` holds it

#### Scenario: A configuration that states no predecessor
- **WHEN** a source change's configuration file sets no `follows:`
- **THEN** the migrated change has no `FOLLOWS`, and no predecessor is inferred

#### Scenario: A change with no configuration file
- **WHEN** a source change holds no configuration file of its own
- **THEN** the survey reports that it has none, rather than omitting the fact

#### Scenario: A key the survey has no name for
- **GIVEN** a source change whose configuration file also sets `skip_specs: true`
- **WHEN** it is surveyed
- **THEN** the survey reports the key, and the plan accounts for it

#### Scenario: A creation date the source states
- **GIVEN** a source change whose configuration file reads `created: 2026-09-17`
- **WHEN** it is surveyed and migrated
- **THEN** the survey reports `created` among the file's keys and carries no value for it, and the date reaches the migrated change only inside the file, where the plan carries the file

#### Scenario: A dropped file that held a creation date
- **GIVEN** a source change whose configuration file states `created:`
- **WHEN** the plan drops or folds the file
- **THEN** the plan's reason says that the creation date goes with the file
