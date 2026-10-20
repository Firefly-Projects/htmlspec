## MODIFIED Requirements

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
