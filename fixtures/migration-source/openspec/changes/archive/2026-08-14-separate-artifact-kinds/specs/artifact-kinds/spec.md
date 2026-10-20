## ADDED Requirements

### Requirement: A change is a directory of kinded documents

A change SHALL be represented as a directory of separate documents rather than as one document carrying every stage. Each document SHALL declare which artifact it is, and the set of kinds SHALL be closed: `proposal`, `design`, `tasks`, and `specification`.

The separation is the point rather than a filing convention. Each kind has a different job and a different degree of formality, and a single document carrying all of them cannot hold a proposal to a looser standard than a specification.

A change SHALL NOT be required to have every kind present. A change may be proposed before it is designed, and designed before its tasks exist.

#### Scenario: A change with four documents
- **WHEN** a contributor reads a change directory
- **THEN** the proposal, the design, the tasks, and the specification are separate documents, each declaring its kind

#### Scenario: A change that has only been proposed
- **GIVEN** a change whose design does not exist yet
- **WHEN** the change is validated
- **THEN** validation succeeds, because a missing later stage is a change in progress rather than a defect

#### Scenario: A kind outside the vocabulary
- **WHEN** a document declares a kind that is not one of the four
- **THEN** the document is invalid, because the set of kinds is closed

### Requirement: Intended content is guidance and enforced structure is validity

Each artifact kind SHALL have an *intended content* and an *enforced structure*, and the two SHALL be distinguishable by a reader. Intended content directs an author or an agent writing the document. Enforced structure is what validation checks, and it is the only thing that decides validity.

The canonical mapping SHALL be:

| Kind | Intended content | Enforced structure |
|---|---|---|
| `proposal` | Value, need, scope, alternatives | None |
| `design` | Architecture and technical reasoning | None |
| `tasks` | Workable implementation items | At least one task unit |
| `specification` | Requirements, scenarios, evidence, verification | No task units; a requirement with no verification is warned |

Where a kind's enforced structure is `None`, a document of that kind SHALL NOT be invalid for carrying any unit, nor for carrying none. For those kinds, "belongs" in the intended-content column is authoring guidance and never a validity boundary.

A unit's appearance in a kind SHALL NOT be prohibited unless this specification names that combination. The only named prohibition is a task unit in a `specification`.

#### Scenario: Guidance departed from
- **WHEN** a proposal or a design carries a unit outside its intended content
- **THEN** the document is valid, because intended content is guidance rather than a validity boundary

#### Scenario: The enforced rules
- **WHEN** a tasks document or a specification is validated
- **THEN** its enforced structure is checked, because later stages and tools consume those two kinds

#### Scenario: An unnamed combination
- **WHEN** a document carries a unit whose combination with its kind this specification does not name
- **THEN** validation reports nothing about it

### Requirement: A proposal argues value and the tool does not shape it

A `proposal` document SHALL be free to use any structure its author finds effective. Its job is to argue value and need, and validation SHALL NOT require any semantic unit in it.

Validation SHALL NOT report a proposal for carrying any unit either. An author who finds it clearest to state a prospective requirement while arguing for it SHALL be able to, in prose or as a unit, with no diagnostic.

The distinction is between permitted and authoritative. A requirement unit in a proposal SHALL be valid, and it SHALL NOT be treated as part of the contract: the contract is what a `specification` states. A consumer determining the project's obligations SHALL read specifications and SHALL NOT read a proposal's units as binding. This is what "requirements do not belong in a proposal" means, and it is a statement about authority rather than about validity.

Prose SHALL be sufficient. A document of headings, paragraphs, figures, and tables SHALL be a conforming proposal with no unit markup at all.

The contract rules that apply to every document — parseable markup, resolvable references, no remote or executable content — SHALL still apply. Freedom of structure is not freedom to ship a document that will not open or render offline.

#### Scenario: A proposal of pure prose
- **WHEN** a proposal contains only headings, paragraphs, and a diagram
- **THEN** it is valid, because arguing a case does not require semantic units

#### Scenario: A proposal that states a draft requirement
- **WHEN** a proposal carries a requirement unit while arguing for it
- **THEN** validation reports nothing about the unit, because the tool does not shape a proposal

#### Scenario: A proposal carrying figures
- **WHEN** a proposal presents a comparison as a chart and a data flow as a diagram
- **THEN** it is valid, because a proposal's freedom of form includes its visualizations

#### Scenario: A proposal that will not open
- **WHEN** a proposal loads a remote script
- **THEN** it is invalid, because the rules that keep every document readable offline are not relaxed by kind

### Requirement: A design reasons in whatever form the reasoning takes

A `design` document SHALL move a proposal's argument into architecture and technical consideration, and SHALL be free to structure that however the reasoning requires.

Validation SHALL NOT require any semantic unit in a design, and SHALL NOT report a design for carrying any unit. Architectural reasoning is continuous — the connections between considerations carry as much as the considerations do — and a tool that pushed an author to fragment it into addressable units would destroy the thing being written.

A design that records its decisions and risks as units SHALL be valid, and so SHALL a design that reasons the same ground in prose and marks nothing. Which of those an author chooses SHALL be their judgment rather than the tool's.

#### Scenario: A design of prose alone
- **WHEN** a design argues an approach in continuous prose and marks no units
- **THEN** it is valid, because a design is not required to fragment its reasoning

#### Scenario: A design that marks its decisions
- **WHEN** a design records the decisions it reached and the risks it accepted as units
- **THEN** it is valid, and each is individually addressable

#### Scenario: A design that reaches into a later stage
- **WHEN** a design carries a task unit or a requirement unit
- **THEN** validation reports nothing about it, because designing is where the shape of the work becomes clear

### Requirement: A tasks document carries the workable items

A `tasks` document SHALL carry at least one task unit. This is a **completeness** rule rather than a placement rule — it constrains what the document must contain rather than which units are permitted in it — and a tasks document carrying other units alongside its tasks SHALL NOT be reported for them.

A task SHALL be individually addressable, so that a state change is a change to one unit rather than to a list.

#### Scenario: Tasks with states
- **WHEN** a tasks document lists items with their states
- **THEN** it is valid, and each task is individually addressable

#### Scenario: A tasks document with no tasks
- **WHEN** a tasks document carries no task unit
- **THEN** validation reports it, because the document enumerates no work

#### Scenario: A tasks document carrying other units
- **WHEN** a tasks document carries its tasks and also a decision unit
- **THEN** validation reports nothing about the decision, because the rule is completeness rather than placement

### Requirement: A specification states the settled contract as built

A `specification` document SHALL carry requirement, scenario, evidence, and verification units. These four are retrospective: evidence and verification can only be recorded after building, and a requirement reaches its final wording once the built shape is known.

A specification SHALL be the artifact that survives its change. It SHALL describe what the change established rather than what it intended, so a reader of the accepted contract reads what was built.

A specification SHALL NOT carry task units, and validation SHALL report one that does as an error. A task that is done is history, and the specification records the obligation the task satisfied rather than the work that satisfied it. This SHALL be the **only** placement rule in the vocabulary, and it is an error because a specification is read by tools and by later changes as the statement of the contract.

#### Scenario: A specification with traced verification
- **WHEN** a specification states a requirement, its scenarios, and the verification that checked them
- **THEN** it is valid, and a consumer can trace the requirement to the evidence for it

#### Scenario: A specification carrying tasks
- **WHEN** a specification carries task units
- **THEN** validation fails, because the specification records obligations rather than the work that satisfied them

#### Scenario: The contract read without the change
- **GIVEN** an archived change
- **WHEN** a reader consults only its specification
- **THEN** they can determine the obligations the change established without reading its proposal, design, or tasks

### Requirement: Every synthesis and every contract update is agent-driven

Four operations move content between artifacts, and each SHALL be performed by an agent following stated rules and guides rather than by a program that parses one artifact and emits another:

1. Creating a **design**, using the proposal.
2. Creating **tasks**, using the proposal and the design.
3. Producing the change's **specification**, from the completed change.
4. **Updating the project's accepted specifications**, from that specification.

The first three are **synthesis**: they produce a new artifact from artifacts that remain intact and separate. A design does not absorb its proposal — it is informed by it, and both survive as their own documents. Calling these merges would misdescribe the four-document model, in which no stage is folded into another.

The fourth is a **merge**, and it is the only one. It applies a change's contract into the project's accepted specifications, which already exist and hold content this change did not touch.

The reason all four need an agent follows from the freedom the upstream kinds are given. A proposal and a design may be structured any way their authors chose, so there is no grammar to parse. Each operation is an act of reading: deciding what a paragraph obliges, which existing requirement a new scenario belongs under, and what to leave alone. A parser cannot do that, and one written to try produces artifacts that satisfy every check while misrepresenting their source.

The merge SHALL apply partial updates rather than replacing wholesale. Adding one scenario to an existing requirement SHALL be expressible without restating the requirement, and content the merge does not mention SHALL be preserved.

The project SHALL supply the rules and guides that direct these merges, as instructions rather than as code. What the project SHALL implement in code is the **checking**: validation reads the merged artifact and reports what is wrong with it. That division is the whole arrangement — judgment where judgment is needed, mechanical checking where a rule is exact.

#### Scenario: Synthesis leaves its sources intact
- **WHEN** an agent creates a design using a change's proposal
- **THEN** the proposal survives unchanged as its own document, because a design is informed by a proposal rather than absorbing it

#### Scenario: Merging a scenario into an existing requirement
- **WHEN** a merge adds one scenario to a requirement that already has three
- **THEN** the three existing scenarios are preserved without being restated, and the requirement's wording is unchanged

#### Scenario: Content the merge does not mention
- **WHEN** a merge names two of a capability's requirements
- **THEN** every other requirement in that capability survives untouched

#### Scenario: A merge that cannot be derived by rule
- **WHEN** a design paragraph implies an obligation that it does not state outright
- **THEN** an agent decides how to state it, because no rule locates an obligation in prose whose structure was the author's choice

#### Scenario: The tool checks rather than merges
- **WHEN** a merged artifact is produced
- **THEN** validation reports what is wrong with it, and the merge itself was not performed by a parser

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
