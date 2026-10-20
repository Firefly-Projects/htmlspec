## ADDED Requirements

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

## MODIFIED Requirements

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
