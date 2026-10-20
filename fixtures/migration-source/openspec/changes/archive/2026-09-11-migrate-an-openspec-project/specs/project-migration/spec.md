## Purpose

Defines how an existing OpenSpec project becomes an HTMLSpec project: what is enumerated
mechanically, what an agent must read and decide, the plan that records every decision before
anything is written, the staging location the migrated project is assembled in, the order the
stages run in, and the checks that gate promotion. It exists because HTMLSpec is a spinoff of
OpenSpec and every plausible adopter already has an OpenSpec project, and because the
conversion that looks obvious — markdown to HTML, file by file — produces a valid project that
has lost the reasoning it was migrating.

## ADDED Requirements

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

#### Scenario: Evidence carried verbatim
- **WHEN** a change carrying evidence files is migrated
- **THEN** each evidence file is present in the migrated change with its contents unchanged

#### Scenario: Evidence is referenced
- **WHEN** a migrated document makes a claim its change recorded evidence for
- **THEN** the document references the carried evidence file

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

#### Scenario: A change state the destination's vocabulary lacks
- **GIVEN** a source change recording that a requirement was renamed, and a destination whose change-state vocabulary has no value for a rename
- **WHEN** the change is migrated
- **THEN** the plan records the rename, records that the destination has no state for it, and names the change that would add one

#### Scenario: The nearest state is not substituted silently
- **WHEN** a source distinction has no destination value
- **THEN** migration does not record it as the nearest available value and leave the plan silent, because the result would be indistinguishable from a source that never made the distinction

#### Scenario: The vocabulary is not extended by the migration
- **WHEN** migration finds a distinction the destination's vocabulary cannot express
- **THEN** migration does not add a value to that vocabulary, and names the change that would

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
- **GIVEN** a source project in which a later change renamed a requirement an earlier change established
- **WHEN** both changes are migrated
- **THEN** each change's specification carries the identifier derived from the heading that change stated, and this is not reported as a collision

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
