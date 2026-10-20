## MODIFIED Requirements

### Requirement: The command surface reports what a project holds

The command surface SHALL provide an operation that enumerates a project's active changes, and SHALL be able to enumerate its accepted specifications instead. The enumeration SHALL be available as a structured payload.

This is the operation that lets a workflow select what it acts on. A workflow that cannot enumerate a project's changes must have its subject named for it every time, and instructions written against such a surface omit the selection step rather than performing it.

**Because it exists for selection, the enumeration of active changes SHALL carry how much of each change's work is complete.** A list of names is not something a workflow or a contributor can select from on any basis other than order, and a selection made at the moment of archiving is made about a directory that is about to move and a contract that is about to be rewritten.

The completion carried SHALL be a count of the change's tasks in each state the contract defines, rather than a single fraction. A fraction cannot separate a change nobody has started from one whose every task is under way, nor a change whose outstanding tasks were abandoned from one whose outstanding tasks remain to be done; those are different situations and a selection is made between them.

**Beside those counts, the enumeration SHALL carry how many of the change's `done` tasks state a deviation**, as document-schema's "A task declares completion state" defines one: the tasks done another way than their words say. Such a task SHALL be counted as `done`, in the count of that state and in any fraction, and SHALL NOT be subtracted from either. Its work happened, and a report that left it out would show finished work as outstanding. The count SHALL be carried apart from the per-state counts, and those SHALL still account for every task exactly once, because a deviation is not a state.

The count exists because a count of states cannot show it. A change that did every task as its words say and a change that did half of them another way both read as finished, and a reviewer choosing which change to check, or what to read before archiving one, is choosing between exactly those two.

Only a task whose state is `done` SHALL be counted as done differently. A task that declares another state and carries a deviation is a document that does not satisfy the schema, and counting it as done differently would attribute to it a state it never declared. This is the reading this requirement already takes for a task that declares no state.

Where the enumeration renders a change's completion for people as a fraction, the numerator SHALL remain every `done` task and the denominator every task, as they are without this count. The count of tasks done differently SHALL be shown in the change's row where it is not zero, and SHALL NOT be shown where it is zero. Most changes do every task as written, and a figure that reads zero on nearly every row teaches a reader to pass over it.

The counts, including the count of tasks done differently, SHALL be derived from the same source as the operation that reports a change's completion, so that the enumeration, that operation, and archival cannot disagree about one change. A second traversal that recounted the same tasks would be a second answer to one question, and the disagreement would be invisible to a test written against either operation alone.

A task that declares no state SHALL be counted apart from the states the contract defines, and SHALL be counted as outstanding wherever outstanding work is reported. The document schema requires a state, so such a task is a document that does not satisfy it; the counts are read from projections rather than from validated documents, and reporting an undeclared task under a state it never declared would attribute to a document something it does not say. Counting it as outstanding is the reading the report about incomplete work already takes, and the two SHALL NOT differ.

A change whose tasks document is absent SHALL be enumerated with its completion absent, rather than reported as a count of zero. A zero fraction reads as a change with no work outstanding, which is the opposite of what an unwritten tasks document means.

The enumeration of accepted specifications is unaffected by the above, which governs changes only.

The order of the enumeration SHALL NOT depend on the completion it carries. A listing that reorders itself as work progresses cannot be selected from twice.

**The enumeration of active changes SHALL also carry each change's stated predecessor and whether that predecessor is resolved.** The strongest reason not to select a change is that the change it follows is unfinished, and a listing that reports a change as ready to work on while its predecessor is open reports the opposite of what the selection needs.

The order of the enumeration SHALL NOT depend on the ordering it reports, for the same reason it does not depend on the completion it carries. A dependency order is additionally unstable in a way name order is not: a cycle has no such order at all.

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

#### Scenario: A change with a task done differently
- **GIVEN** an active change of four tasks, two of them `done`, one of which states a deviation
- **WHEN** the project's changes are enumerated
- **THEN** two of four tasks are reported done, and one task is reported as done differently, apart from the per-state counts

#### Scenario: A deviation leaves the per-state counts whole
- **GIVEN** an active change one of whose `done` tasks states a deviation
- **WHEN** the enumeration is requested as a structured payload
- **THEN** the per-state counts sum to the total, and the count of tasks done differently is carried apart from them

#### Scenario: A change whose every task was done as written
- **GIVEN** an active change none of whose tasks states a deviation
- **WHEN** the project's changes are enumerated for people
- **THEN** the change's row shows no count of tasks done differently, and the structured payload reports that count as zero

#### Scenario: A deviation on a task that is not done
- **GIVEN** an active change one of whose `todo` tasks carries a deviation
- **WHEN** the project's changes are enumerated
- **THEN** that task is counted as `todo`, and not as done differently

#### Scenario: A task that declares no state
- **GIVEN** an active change one of whose tasks declares no state
- **WHEN** the project's changes are enumerated
- **THEN** that task is counted apart from the defined states, and the change does not read as though its work were resolved

#### Scenario: The enumeration and the completion report agree
- **GIVEN** any active change
- **WHEN** its completion is read from the enumeration and from the operation that reports a change's completion
- **THEN** the two report the same counts, including how many tasks were done differently

#### Scenario: A change with no tasks document
- **GIVEN** an active change whose tasks document is absent
- **WHEN** the project's changes are enumerated
- **THEN** the change is named and its completion is reported as absent rather than as zero

#### Scenario: Order does not follow progress
- **GIVEN** a project whose changes are at differing stages of completion
- **WHEN** the changes are enumerated twice, with work completed in between
- **THEN** the order is unchanged

#### Scenario: A change whose predecessor is unfinished
- **GIVEN** an active change that follows another active change
- **WHEN** the project's changes are enumerated
- **THEN** the predecessor is named beside it and reported as unresolved

#### Scenario: A change whose predecessor is done
- **GIVEN** an active change whose stated predecessor is archived
- **WHEN** the project's changes are enumerated
- **THEN** the predecessor is named beside it and reported as resolved

#### Scenario: Order does not follow the stated sequence
- **GIVEN** a project whose changes state predecessors
- **WHEN** the changes are enumerated
- **THEN** the order is by name and does not follow the stated sequence

### Requirement: The command surface reports a change's completion

The command surface SHALL provide an operation that reports, for one change, which of its artifacts are present and how many of its tasks are complete. The report SHALL be available as a structured payload.

**The report SHALL also state how many of the change's `done` tasks state a deviation, and SHALL name each one.** For such a task the deviation, not its words, is the claim about what was done, so a reviewer checking the change before archival compares each deviation with the work. A count alone would leave them searching the tasks document for the tasks it counts. The count SHALL be the one the enumeration of changes carries, derived from the same source.

Archival SHALL derive its report about incomplete work from this same operation, so that a contributor who checks a change before archiving it and the archival that follows cannot disagree about what the change holds.

#### Scenario: A change mid-implementation
- **WHEN** completion is reported for a change with a proposal, a design, tasks, and no specification
- **THEN** the report names the specification as absent and states how many tasks remain

#### Scenario: Archival and the report agree
- **WHEN** archival reports incomplete work for a change
- **THEN** the count it reports is the count the completion operation reports for that change

#### Scenario: A change with a task done differently
- **GIVEN** a change one of whose `done` tasks states a deviation
- **WHEN** completion is reported for that change
- **THEN** the task is counted as complete, and the report states that one task was done differently and names it
