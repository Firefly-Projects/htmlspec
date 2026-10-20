## MODIFIED Requirements

### Requirement: The command surface reports what a project holds

The command surface SHALL provide an operation that enumerates a project's active changes, and SHALL be able to enumerate its accepted specifications instead. The enumeration SHALL be available as a structured payload.

This is the operation that lets a workflow select what it acts on. A workflow that cannot enumerate a project's changes must have its subject named for it every time, and instructions written against such a surface omit the selection step rather than performing it.

**Because it exists for selection, the enumeration of active changes SHALL carry how much of each change's work is complete.** A list of names is not something a workflow or a contributor can select from on any basis other than order, and a selection made at the moment of archiving is made about a directory that is about to move and a contract that is about to be rewritten.

The completion carried SHALL be a count of the change's tasks in each state the contract defines, rather than a single fraction. A fraction cannot separate a change nobody has started from one whose every task is under way, nor a change whose outstanding tasks were abandoned from one whose outstanding tasks remain to be done; those are different situations and a selection is made between them.

The counts SHALL be derived from the same source as the operation that reports a change's completion, so that the enumeration, that operation, and archival cannot disagree about one change. A second traversal that recounted the same tasks would be a second answer to one question, and the disagreement would be invisible to a test written against either operation alone.

A task that declares no state SHALL be counted apart from the states the contract defines, and SHALL be counted as outstanding wherever outstanding work is reported. The document schema requires a state, so such a task is a document that does not satisfy it; the counts are read from projections rather than from validated documents, and reporting an undeclared task under a state it never declared would attribute to a document something it does not say. Counting it as outstanding is the reading the report about incomplete work already takes, and the two SHALL NOT differ.

A change whose tasks document is absent SHALL be enumerated with its completion absent, rather than reported as a count of zero. A zero fraction reads as a change with no work outstanding, which is the opposite of what an unwritten tasks document means.

The enumeration of accepted specifications is unaffected by the above, which governs changes only.

The order of the enumeration SHALL NOT depend on the completion it carries. A listing that reorders itself as work progresses cannot be selected from twice.

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

#### Scenario: A task that declares no state
- **GIVEN** an active change one of whose tasks declares no state
- **WHEN** the project's changes are enumerated
- **THEN** that task is counted apart from the defined states, and the change does not read as though its work were resolved

#### Scenario: The enumeration and the completion report agree
- **GIVEN** any active change
- **WHEN** its completion is read from the enumeration and from the operation that reports a change's completion
- **THEN** the two report the same counts

#### Scenario: A change with no tasks document
- **GIVEN** an active change whose tasks document is absent
- **WHEN** the project's changes are enumerated
- **THEN** the change is named and its completion is reported as absent rather than as zero

#### Scenario: Order does not follow progress
- **GIVEN** a project whose changes are at differing stages of completion
- **WHEN** the changes are enumerated twice, with work completed in between
- **THEN** the order is unchanged

### Requirement: Archival assesses the merge and never performs it

Archival SHALL determine whether the change's contract reached the accepted specifications, and SHALL do so by comparing requirement identifiers rather than by reading requirements for meaning.

Archival SHALL NOT edit an accepted specification, SHALL NOT propose the text of a merge, and SHALL NOT treat an absent identifier as an error. Whether an absent requirement was withdrawn deliberately or omitted by mistake is a judgment, and `artifact-kinds` reserves it for an agent.

**Where the assessment finds the contract has not reached the accepted specifications, archival SHALL report that the change has not been synced and SHALL name the operation that reports what remains.** The report SHALL name the operation as it is invoked, rather than describing it. The prior wording told a contributor to apply the change's contract at a time when no operation did so, and a report that names a step without naming the command behind it leaves the reader searching the command surface for a name they must guess.

Where archival asks whether to proceed over its findings, and the change has not been synced, **the question SHALL say so and SHALL state that declining leaves the project unchanged.** A question that asks only whether to archive anyway invites the answer that archives a change whose contract never reached the project's specifications; a contributor deciding that should be told what proceeding skips and that declining costs them nothing.

The structured payload SHALL state whether the change has not been synced, as a value rather than as something derivable from the reported identifiers. An agent driving archival cannot answer the question a contributor is asked, so the condition that question is about SHALL be readable without inference.

Archival SHALL NOT perform the merge it names, and the report SHALL NOT be taken as authority to. This is the same prohibition stated above, restated because naming an operation is where a reader would expect an offer to run it.

Where the assessment finds the contract has reached the accepted specifications, archival SHALL NOT report anything about syncing. A report that appears whether or not it applies is one a reader learns to pass over, and the occasion it mattered would be passed over with it.

#### Scenario: The assessment is a comparison of identifiers
- **WHEN** archival assesses whether a merge occurred
- **THEN** it compares the identifiers the change's specification states against those the accepted specifications hold, and reads neither for meaning

#### Scenario: Archival does not write the accepted specifications
- **WHEN** archival completes
- **THEN** no accepted specification was modified by the program

#### Scenario: A change whose contract has not been synced
- **GIVEN** a change stating a requirement no accepted specification holds
- **WHEN** archival reports its findings
- **THEN** it reports that the change has not been synced and names the operation that reports what remains, as it is invoked

#### Scenario: The question asked about an unsynced change
- **GIVEN** a change whose contract has not reached the accepted specifications
- **WHEN** archival asks whether to proceed over its findings
- **THEN** the question states that the change has not been synced and that declining leaves the project unchanged

#### Scenario: A change whose contract was applied
- **GIVEN** a change every one of whose stated requirements an accepted specification holds
- **WHEN** archival reports its findings
- **THEN** nothing about syncing is reported

#### Scenario: An agent reads the condition rather than inferring it
- **WHEN** archival emits its structured payload
- **THEN** whether the change has not been synced is stated as a value in the payload
