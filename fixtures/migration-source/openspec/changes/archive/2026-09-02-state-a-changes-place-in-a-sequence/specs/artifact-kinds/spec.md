## ADDED Requirements

### Requirement: A change states the change it follows

A change SHALL be able to state that it follows another change, and the statement SHALL be a declaration the author writes rather than a relation the program infers. The declaration SHALL live with the change's other whole-directory facts rather than inside one of its documents.

The ordering between changes is frequently the argument of the work. A contract written before the changes it grades constrains them, and the same contract written afterwards describes them; the difference is the order the changes were completed in, and until now no project could record that in a form anything reads. A convention enforced only by a contributor remembering to re-read a proposal is the same fault `project-bootstrap` names when it forbids a gate derived from a directory name: a rule nothing checks is not a rule.

**The program SHALL NOT infer an ordering from a change's name, its creation date, or the requirement identifiers it states.** A change is renamed by moving its directory, and this contract states elsewhere that nothing depends on the first name — an ordering read out of names would make that false. A creation date records when a change was written and not what it depends on. Two changes stating one requirement identifier may be ordered or may conflict, and deciding which is judgment this contract reserves for an agent.

A change SHALL state at most one predecessor. One relation is what the observed need requires, and a model carrying many would be designed against no evidence.

A change that states no predecessor SHALL be complete as it stands. Most changes follow nothing, and an absent declaration SHALL NOT be reported.

#### Scenario: A change that follows another
- **WHEN** a change's whole-directory facts are read
- **THEN** the change it follows is among them, exactly as its author wrote it

#### Scenario: A change that follows nothing
- **GIVEN** a change stating no predecessor
- **WHEN** the change is validated
- **THEN** validation succeeds and nothing about ordering is reported

#### Scenario: An ordering is never derived
- **WHEN** two changes carry names, creation dates, or requirement identifiers from which an ordering could be guessed
- **THEN** no ordering holds between them unless one of them states that it follows the other

### Requirement: A stated predecessor is resolved across the archive

The program SHALL resolve a stated predecessor to one of three outcomes: a change that is archived, a change that is active, or a name matching no change in the project. The three SHALL be reported distinctly, because they call for different work — none, waiting, and a correction respectively.

**A predecessor SHALL be matched against an archived change with its leading date removed.** Archival names an archived directory for the date of archival followed by the change's name, so a successor recording its predecessor's name cannot match that directory by equality once the predecessor is archived. That is the state the successor is waiting for, so an equality test would fail in exactly the case this resolution exists to detect, and would fail silently. The date SHALL be recognised by the same test archival uses when it decides whether a name already carries one.

A name matching no change SHALL be reported as such rather than treated as an unfinished predecessor. A predecessor that was renamed and a predecessor that is merely unfinished are different situations, and a report that conflated them would send a contributor looking for work that is already done under another name.

Two or more changes that each follow the other SHALL be reported as a contradiction, and the program SHALL NOT choose an ordering between them. The recorded facts are inconsistent, and the author who wrote both declarations is the one who can say which is wrong.

#### Scenario: A predecessor that has been archived
- **GIVEN** a change that follows a change now in the archive under a dated directory name
- **WHEN** the predecessor is resolved
- **THEN** it resolves as archived, matched with the leading date removed

#### Scenario: A predecessor still being worked on
- **GIVEN** a change that follows an active change
- **WHEN** the predecessor is resolved
- **THEN** it resolves as active

#### Scenario: A predecessor that no longer exists under that name
- **GIVEN** a change that follows a name matching no active and no archived change
- **WHEN** the predecessor is resolved
- **THEN** it resolves as unmatched, reported distinctly from a predecessor that is merely unfinished

#### Scenario: Two changes that each follow the other
- **WHEN** the predecessors of a set of changes are resolved
- **THEN** a cycle among them is reported as a contradiction and no ordering is chosen

## MODIFIED Requirements

### Requirement: Archival produces the specification and then updates the accepted contract


Two documents are in play at archival and there SHALL be exactly two: the **change's specification artifact**, which lives in the change directory, and the **project's accepted specifications**, which live outside it and hold the contract for the whole project. The change's specification and its "as-built specification" SHALL be the same file. A change directory SHALL hold at most one specification artifact.

Archival SHALL proceed in this order:

1. The agent creates the change's specification artifact, or updates it if the change already has one, from that change's proposal, design, and tasks.
2. Validation checks it. Archival SHALL NOT proceed past this point while it reports an error.
3. The agent applies that specification's contract changes to the project's accepted specifications.
4. The change directory is archived intact, with all four of its documents preserved.

The change's documents SHALL survive archival. A reader SHALL be able to open an archived change and find the argument that motivated it, the reasoning that shaped it, the work that delivered it, and the contract it established — which is the whole reason for keeping the stages separate.

The specification SHALL be produced by an agent following stated instructions rather than by a deterministic extractor.

The reason is that the mapping is judgment rather than transformation, and the freedom the upstream kinds are given is what makes it so. A proposal and a design may be structured any way their authors chose, so no rule can locate the obligation inside them. Deciding which paragraph states one, what its identifier is, and which scenarios express it is reading rather than parsing. An extractor that guessed would emit documents that satisfy every check and misrepresent their source, which is worse than producing nothing.

The produced specification SHALL be validated, and the agent SHALL reconcile the diagnostics it reports. This is the division the process depends on: the agent supplies the judgment, and validation checks the artifact.

Production SHALL be idempotent in effect. Running it twice against an unchanged change SHALL NOT alter the specification's meaning.

**Where a change states a predecessor that is not archived, archival SHALL report it and SHALL name the predecessor.** Archival is the operation that ends a change, so it is the last moment the ordering can be observed, and a sequence completed out of order is a sequence whose argument was lost while every artifact validated.

That report SHALL be a finding of the kind archival already reports and asks about, and SHALL NOT refuse the archival. An abandoned sequence whose later change is completed on its own merits is a legitimate outcome, and a contributor who knows the sequence changed is the authority on it. A gate here would leave them editing a declaration to state a history that did not happen.

The structured payload SHALL state the unresolved predecessor as a value. An agent driving archival cannot answer the question a contributor is asked, so the condition SHALL be readable without inference.

#### Scenario: Archival produces the specification
- **WHEN** a change with completed tasks is archived
- **THEN** its specification artifact is created or updated from its other documents, and validated before archival proceeds

#### Scenario: The accepted contract is updated after validation
- **GIVEN** a change whose specification validates
- **WHEN** archival continues
- **THEN** the agent applies that specification's contract changes to the project's accepted specifications

#### Scenario: The change survives archival
- **WHEN** a reader opens an archived change
- **THEN** its proposal, design, tasks, and specification are all present, so the argument and the reasoning are recoverable alongside the contract

#### Scenario: One specification per change
- **WHEN** a change directory is read
- **THEN** it holds at most one specification artifact, which is also its as-built record

#### Scenario: The produced specification does not validate
- **WHEN** the produced specification reports an error
- **THEN** the agent reconciles it, so an invalid specification is never the archival's output

#### Scenario: Production is not a parser
- **WHEN** a design paragraph implies an obligation that it does not state outright
- **THEN** the agent decides how to state it as a requirement, rather than a rule extracting text mechanically

#### Scenario: Producing twice
- **GIVEN** a change whose artifacts have not changed
- **WHEN** the specification is produced a second time
- **THEN** its meaning is unchanged

#### Scenario: A change archived after its predecessor
- **GIVEN** a change whose stated predecessor is already archived
- **WHEN** it is archived
- **THEN** nothing about ordering is reported

#### Scenario: A change archived before its predecessor
- **GIVEN** a change whose stated predecessor is still active
- **WHEN** it is archived
- **THEN** archival names the predecessor, asks, and archives on an affirmative answer

#### Scenario: An agent archiving out of order
- **GIVEN** a change whose stated predecessor is not archived
- **WHEN** archival emits its structured payload
- **THEN** the unresolved predecessor is stated as a value in the payload
