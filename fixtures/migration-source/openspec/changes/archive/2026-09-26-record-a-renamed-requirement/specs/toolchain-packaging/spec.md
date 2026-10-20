## MODIFIED Requirements

### Requirement: Archival assesses the merge and never performs it

Archival SHALL determine whether the change's contract reached the accepted specifications, and SHALL do so by comparing requirement identifiers, and for a renamed requirement its heading, rather than by reading requirements for meaning. This SHALL be the comparison the reporting operation makes, derived from one source, including its treatment of a rename the accepted specifications have not taken.

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

#### Scenario: A change whose only unapplied work is a rename
- **GIVEN** a change every one of whose stated identifiers an accepted specification holds, one of which the change renames and the accepted specification still holds under the previous heading
- **WHEN** archival reports its findings
- **THEN** it reports that the change has not been synced, names the untaken rename, and names the operation that reports what remains

### Requirement: The command surface reports whether a change reached the accepted contract

The command surface SHALL provide an operation that reports, for one change, whether the requirements its specification states have reached the project's accepted specifications. The report SHALL be available as a structured payload.

The operation SHALL determine this by comparing requirement identifiers, and, for a renamed requirement, headings, and SHALL read neither the change's specification nor the accepted specifications for meaning. This is the same comparison archival makes, and both SHALL derive it from one source, so that a contributor who runs the operation and the archival that follows cannot disagree about which requirements reached the contract.

The report SHALL distinguish three sets: the requirement identifiers the change states, those the accepted specifications already hold, and those absent from them. The third is derivable from the first two and SHALL be reported anyway, because it is the answer the operation exists to give.

**A renamed requirement SHALL be compared by its heading as well as its identifier.** A rename keeps the requirement's identifier, so a comparison of identifiers alone reports a rename the accepted specifications never took as reached. Where a requirement the change states carries a `renamed-from` paragraph, and the accepted specifications hold its identifier, the operation SHALL compare that requirement's heading with the heading of the accepted requirement carrying the same identifier. Where they differ, the requirement SHALL be reported in a fourth set: renames the accepted specifications have not taken. The comparison SHALL be of the two heading names — each heading without the identifier shown beside it, as document-schema's "A renamed requirement states the heading it replaced" defines — after whitespace normalization, which is an exact comparison and not a reading for meaning.

A renamed requirement whose identifier the accepted specifications do not hold SHALL be reported as absent, as any other requirement is, and SHALL NOT also be reported as an untaken rename. A requirement in the fourth set SHALL be counted as not having reached the contract, so a change whose only unapplied work is a rename is reported as not synced.

**The operation SHALL NOT modify an accepted specification, and SHALL NOT propose the text of a merge.** Deciding which accepted requirement a scenario belongs under is judgment, and `artifact-kinds` reserves it for an agent. This prohibition is stated rather than left implied: the operation carries OpenSpec's name for it, that name reads like an operation that writes, and a reader who assumes it does would be assuming the program edits the project's contract unattended.

An absent requirement SHALL be a finding and not an error, and the operation SHALL succeed while any remain. A change mid-implementation legitimately states requirements that have not reached the contract, and a requirement withdrawn deliberately is absent permanently; an operation that failed on either would fail on the normal state of the work it is meant to be run during.

The operation SHALL refuse on a change that cannot be resolved, on an absent specification artifact, and on a specification that does not validate. A comparison made against an invalid contract has no meaning, and reporting it as though it did is worse than declining to report.

#### Scenario: A change whose contract has not been applied
- **GIVEN** a change whose specification states a requirement that no accepted specification holds
- **WHEN** the operation is run against that change
- **THEN** the requirement is named as absent, the operation succeeds, and no accepted specification is modified

#### Scenario: A change whose contract has been applied
- **GIVEN** a change every one of whose stated requirements appears in an accepted specification
- **WHEN** the operation is run against that change
- **THEN** the absent set is empty

#### Scenario: The comparison is of identifiers
- **WHEN** the operation determines whether a requirement reached the contract
- **THEN** it compares identifiers, and reads neither the change's specification nor the accepted specifications for meaning

#### Scenario: The operation and archival agree
- **WHEN** archival reports that a change states requirements no accepted specification holds
- **THEN** the identifiers it names are the identifiers the operation reports as absent for that change

#### Scenario: A structured report
- **WHEN** structured output is requested
- **THEN** the stated, accepted, and absent identifier sets, and the untaken renames, are emitted as a payload

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
