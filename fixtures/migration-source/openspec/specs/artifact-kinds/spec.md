## Purpose

Defines the four artifact kinds a change's documents take — proposal, design, tasks, and specification — the intended content of each, and the separation between authoring guidance and the small set of enforced rules. It also defines a change as a directory of kinded documents, specifies the archival sequence, and requires every synthesis and contract update to be performed by an agent rather than by a parser.

## Requirements

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

### Requirement: A change's evidence is kept in the change and is not one of its documents

A change directory MAY hold an `evidence/` directory directly within it. Every file beneath that directory, at any depth, SHALL be the change's evidence: a transcript, a measurement, a recorded complaint, or a document preserved as it was written. It is a record of why the change has the shape it does.

**Evidence SHALL NOT be a document of the project.** An evidence file declares no artifact kind, owes conformance to no contract version, and does not reference the project's asset store. The obligations this contract places on "every document in the project", including "A project has a declared root holding one asset store", do not reach it. Its format is its author's choice: Markdown, plain text, a script, or HTML.

The reason is the one project-migration's "Evidence crosses unchanged" already gives: a record's worth is that nobody edited it. A contract that held evidence to the rules for documents would require each record to be rewritten whenever the contract moved. A record written in this format at an earlier contract version, or naming an asset store that has since been removed, would have to be edited or deleted. Either act destroys what the record was kept for.

The location is positional. A directory named `evidence` is a change's evidence only where it sits directly within a change directory, active or archived. The same name anywhere else, such as a capability whose path is `evidence`, carries no special meaning.

No operation of the program SHALL rewrite a file beneath a change's evidence, including one a contributor names. Checking one is permitted, because a check writes nothing.

#### Scenario: A record in this format at a superseded version
- **GIVEN** a change whose evidence holds an HTML file declaring an earlier contract version and naming an asset store the project does not hold
- **WHEN** the project's documents are checked
- **THEN** the file is not among them, and no diagnostic names it

#### Scenario: Evidence nested below the evidence directory
- **GIVEN** a change whose evidence holds `evidence/transcripts/a-session.html`
- **WHEN** the project's documents are checked
- **THEN** the file is evidence, because every file beneath a change's evidence directory is, at any depth

#### Scenario: A capability that happens to be named evidence
- **GIVEN** a project holding an accepted specification at `htmlspec/specs/evidence/spec.html`
- **WHEN** the project's documents are checked
- **THEN** that specification is among them, because only a change's evidence directory is excluded

#### Scenario: An archived change keeps its evidence
- **GIVEN** an archived change whose evidence holds an HTML file
- **WHEN** the project's documents are checked
- **THEN** the file is not among them, because an archived change is still a change

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

A proposal's need SHALL be either a problem with something that exists or a capability the project does not have, and neither form SHALL be privileged over the other. A project's first change has nothing to measure and no error message to quote, so guidance that admits only a malfunction describes a need the earliest proposals cannot state. Authoring guidance SHALL NOT narrow a need to a defect.

An absence of capability SHALL be stated as concretely as a malfunction, and the concrete form SHALL be who or what wants the capability, what they cannot do without it, and what working looks like when it is first observed. A need SHALL NOT be stated as a category of concern, because a category cannot be argued with while a proposal is read or checked against once the work is built. The obligation to be concrete is what this requires; a particular kind of evidence is not.

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

#### Scenario: A need that is an absence
- **WHEN** a proposal argues for a capability the project does not have yet
- **THEN** it states who wants the capability, what they cannot do without it, and what working looks like, and this is a complete need with no problem to report

#### Scenario: Guidance that demands a defect
- **WHEN** authoring guidance directs an agent to state a need as a measurement, an error message, or a behavior somebody hit, and offers no other form
- **THEN** the guidance is wrong, because a change that adds a capability has none of those to state

#### Scenario: A need stated as a category
- **WHEN** a proposal states its need as a category of concern rather than as a named want or malfunction
- **THEN** the need is insufficient, whichever of the two forms it was reaching for

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

**The merge SHALL preserve every accepted requirement the change's specification does not state.** The accepted specifications hold requirements from every change before this one, and a requirement no change states is one no change decided to alter. A requirement the change modifies reaches the merge stated whole, as "A change's specification states a modified requirement whole" requires. The merge SHALL apply it as stated: the accepted requirement takes the statements and the scenarios the change states, less the scenarios it marks `removed`. The merge SHALL NOT reconstruct a modified requirement from what the change left out.

The project SHALL supply the rules and guides that direct these merges, as instructions rather than as code. What the project SHALL implement in code is the **checking**: validation reads the merged artifact and reports what is wrong with it. That division is the whole arrangement — judgment where judgment is needed, mechanical checking where a rule is exact.

#### Scenario: Synthesis leaves its sources intact
- **WHEN** an agent creates a design using a change's proposal
- **THEN** the proposal survives unchanged as its own document, because a design is informed by a proposal rather than absorbing it

#### Scenario: Merging a scenario into an existing requirement
- **GIVEN** an accepted requirement with three scenarios, and a change whose specification states it whole, with those three marked `unchanged` and a fourth marked `added`
- **WHEN** the merge applies the change's contract
- **THEN** the accepted requirement holds all four scenarios, the three with the text they already had, and its wording is the wording the change states

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

### Requirement: A rename reaches the accepted contract as a new heading on the same identifier

Where a change's specification states that a requirement replaced a previous heading, applying that
specification's contract to the project's accepted specifications SHALL give the accepted requirement
carrying that identifier the new heading, and SHALL NOT change its identifier.

The accepted specification SHALL NOT carry the `renamed-from` paragraph. The accepted contract states
what the project requires now, under the name it uses now, and the history of a heading belongs to
the archived change that renamed it. This is the arrangement OpenSpec keeps: a main specification
holds no RENAMED section, and the rename survives in the change that made it.

Where the change also modified the requirement, the merge SHALL apply the modification to the same
accepted requirement. The rename and the modification SHALL be applied as two facts about one
requirement, and neither SHALL be applied as a removal of the old requirement followed by the
addition of a new one, because that would regenerate an identifier the schema requires to be stable.

Where the accepted requirement carrying that identifier already has the new heading, applying the
rename SHALL change nothing. A rename applied before archival, by the operation that applies a
change's contract, is the ordinary case, and archival SHALL NOT treat it as a conflict.

The merge remains the agent's work under "Every synthesis and every contract update is agent-driven".
This requirement states what a correctly applied rename leaves behind, not a program that applies it.

#### Scenario: A rename applied to the accepted contract
- **GIVEN** a change whose specification states requirement `req-x` under a new heading, with a `renamed-from` paragraph naming the heading the accepted specification holds
- **WHEN** its contract is applied to the accepted specifications
- **THEN** the accepted requirement `req-x` carries the new heading, keeps its identifier, and carries no `renamed-from` paragraph

#### Scenario: A rename and a modification in one change
- **GIVEN** a change that renames and rewrites one requirement
- **WHEN** its contract is applied
- **THEN** the one accepted requirement takes both the new heading and the new content, and no requirement is removed or added

#### Scenario: A rename already applied
- **GIVEN** an accepted requirement that already carries the heading a change renames it to
- **WHEN** the change's contract is applied again
- **THEN** nothing changes, and nothing is reported as a conflict

#### Scenario: The history is read from the archive
- **WHEN** a reader wants to know what an accepted requirement was previously called
- **THEN** the archived change that renamed it states the previous heading, and the accepted specification does not

### Requirement: A described change starts without a supplied name

A workflow that creates a change SHALL start from a description of the work, and SHALL require a contributor to supply only what the description does not already carry.

Where a description carries the work but no name, the agent SHALL derive a name from the description under the naming rule that governs any name, and SHALL proceed. A name is recoverable — a change directory is renamed by moving it — so requiring one before anything is created spends an exchange on the cheapest thing to correct.

The derived name SHALL be reported, so that a contributor who disagrees can say so. A derived name a contributor never sees is a decision taken on their behalf rather than a proposal they can refuse.

A workflow SHALL NOT ask for what its input already states. Asking again for a stated scope or a stated exclusion presents a description as incomplete when it is not, and the contributor cannot tell which part was found wanting.

Where a request carries no description of the work at all, asking SHALL remain correct. The obligation is to ask for what is missing rather than to infer a need that was never stated.

#### Scenario: A description with no name
- **WHEN** a contributor describes the work and names no change
- **THEN** the agent derives a name, reports it, and creates the change without a further exchange

#### Scenario: A description already carrying its scope
- **WHEN** a contributor's description states what the change excludes
- **THEN** the workflow does not ask for the exclusion again

#### Scenario: A request with no work in it
- **WHEN** a workflow is invoked with no description of the work
- **THEN** it asks what the work is, because a need that was never stated must not be invented

#### Scenario: A derived name a contributor rejects
- **WHEN** an agent reports a name it derived and the contributor wants another
- **THEN** the change is renamed by moving its directory, because nothing depends on the first name

### Requirement: A change's specification states a modified requirement whole

A requirement that a change's specification marks `modified` SHALL state the requirement whole: every statement the requirement will hold once the change's contract is merged, and every scenario it will hold then. **Each scenario SHALL carry its own change state**: `unchanged` for a scenario carried forward as the accepted specification states it, `modified` for one whose text the change alters, and `added` for one the change introduces.

**A scenario the change withdraws SHALL be stated as well, as a unit marked `removed`.** document-schema's "Semantic units declare a change state" gives every unit, a scenario included, the state `removed`, and keeps a withdrawn unit present so that reviewers can see what is being withdrawn. The merge deletes that scenario from the accepted requirement.

The requirement SHALL carry the identifier the accepted requirement carries, and each scenario it carries forward, alters, or withdraws SHALL carry the identifier its accepted counterpart carries. An identifier is what makes a stated scenario the same scenario as an accepted one. A heading cannot do that job, because a change may alter a scenario's heading.

The reason for the rule is that only a whole statement says what it means. Where a change may state part of a requirement, a scenario it leaves out is either withdrawn or merely not restated, and the text is the same in both cases. This project's own archive holds both readings in one change: `2026-08-14-separate-artifact-kinds` left one scenario out of one requirement to withdraw it, and left two out of another that its merge kept. A whole statement also lets an archived change's specification be read without the accepted specification of its day, which "A specification states the settled contract as built" asks of it. And it lets a later comparison tell a modification the merge took from one it did not.

The statements a modified requirement states SHALL be its statements once merged. A statement paragraph carries no change state of its own. So a paragraph the change deletes is one it does not state, and a paragraph it rewrites is stated as rewritten.

**An accepted scenario that a modified requirement neither states nor marks `removed` SHALL be a defect in the change's specification.** It SHALL NOT be read as carried forward, and it SHALL NOT be read as withdrawn. Three requirements in this project's archive left a scenario out. Reading the omission as a withdrawal is wrong for one of them, and reading it as carried forward is wrong for the other two, so either default would repeat a recorded error. The agent performing the merge SHALL report the omission to the contributor, naming the scenario, and SHALL NOT resolve it by either reading.

The partial form, which states only the part of a requirement a change alters and leaves the merge to keep the rest, SHALL NOT be used. OpenSpec's spec-driven schema requires the whole block, and the `openspec-sync-specs` workflow this project copied the partial form from permitted it. This requirement takes the schema's side on purpose. It adds what a Markdown delta cannot express: a withdrawn scenario stated rather than omitted.

#### Scenario: A scenario added to an existing requirement
- **GIVEN** an accepted requirement with three scenarios
- **WHEN** a change adds a fourth scenario to it
- **THEN** the change's specification states the requirement marked `modified`, with every statement it holds, the three scenarios marked `unchanged`, and the fourth marked `added`

#### Scenario: A scenario withdrawn
- **GIVEN** an accepted requirement with three scenarios
- **WHEN** a change withdraws one of them
- **THEN** the change's specification states that scenario as a unit marked `removed`, and the merge leaves the accepted requirement holding the other two

#### Scenario: A statement rewritten
- **WHEN** a change rewrites one paragraph of an accepted requirement's statement
- **THEN** the change's specification states every statement the requirement will hold, the rewritten paragraph included, and the merge gives the accepted requirement exactly those statements

#### Scenario: A scenario keeps its identifier
- **WHEN** a change carries forward, alters, or withdraws an accepted scenario
- **THEN** the scenario it states carries the identifier the accepted scenario carries, whatever its heading

#### Scenario: An accepted scenario left out
- **GIVEN** an accepted requirement holding a scenario that a change's modified requirement neither states nor marks `removed`
- **WHEN** the change's contract is merged
- **THEN** the agent performing the merge reports the scenario to the contributor, and neither keeps it nor deletes it on its own reading

#### Scenario: A modified requirement read from the archive
- **GIVEN** an archived change that modified a requirement
- **WHEN** a reader consults only that change's specification
- **THEN** they read the whole requirement as the change left it, and which of its scenarios the change added, altered, withdrew, and carried forward

### Requirement: A marked modification reaches the accepted contract as its current text

Where a change's specification marks words in a requirement or a scenario it modifies, applying that specification's contract to the project's accepted specifications SHALL give the accepted unit its current text: the content of each `ins` element without the element, and nothing of each `del` element.

**The accepted specification SHALL NOT carry change markup.** The accepted contract states what the project requires now. Which words a change added and which it deleted is history, and it belongs to the archived change that made the edit, whose specification keeps its marks. This is the arrangement "A rename reaches the accepted contract as a new heading on the same identifier" makes for a heading, where the accepted specification does not carry the `renamed-from` paragraph.

A mark carried into the accepted contract would go on claiming that a word is new for as long as the text stands. This project's history holds the case. `2026-09-18-establish-a-runbook` bolded the words it added to two project-bootstrap requirements, the merge copied the bold into the accepted specification, and it is still there, where it now reads as emphasis. The word the same edit deleted was never marked at all. A `del` carried across would show the next change's author a deleted phrase on the page of the contract, ready to be copied back as current.

Validation holds the second half of this: an accepted specification that carries change markup in a unit is rejected, under document-validation's "Change markup in or around a semantic unit is rejected". The first half is the merge, which remains the agent's work under "Every synthesis and every contract update is agent-driven". This requirement states what a correctly applied modification leaves behind, not a program that applies it.

**The current text the merge writes is the text the change's specification projects.** document-schema's "A deleted word is recorded apart from what a unit states" keeps deleted words out of every statement and entry, and keeps inserted words in. So a comparison of a change's modified requirement with the accepted one, made on projected text, finds a merge that wrote the current text equal to the change. It finds a merge that kept a deleted word, or left out an inserted one, unequal. The marks need no reading of their own to be compared.

#### Scenario: An inserted word applied
- **GIVEN** a change whose specification states requirement `req-x` marked `modified`, with `<ins><code>sync</code>, </ins>` in one statement
- **WHEN** its contract is applied to the accepted specifications
- **THEN** the accepted requirement `req-x` states `sync` in that statement, as ordinary text, and carries no `ins` element

#### Scenario: A deleted word applied
- **GIVEN** a change whose specification states a requirement marked `modified`, with `<del>and </del>` in one statement
- **WHEN** its contract is applied
- **THEN** the accepted requirement's statement no longer holds that word, and the accepted specification carries no `del` element

#### Scenario: A deleted paragraph applied
- **GIVEN** a change whose specification states a requirement marked `modified`, with one paragraph wrapped in `del`
- **WHEN** its contract is applied
- **THEN** the accepted requirement no longer holds that paragraph

#### Scenario: Markup left in the accepted contract
- **WHEN** a merge leaves an `ins` or a `del` element in a unit of an accepted specification
- **THEN** validation of that accepted specification fails

#### Scenario: The history is read from the archive
- **WHEN** a reader wants to know which words a change added to a requirement, or deleted from it
- **THEN** the archived change's specification marks them, and the accepted specification does not

#### Scenario: A marked modification compared with the accepted text
- **GIVEN** a change whose specification marks an inserted word and a deleted word in a modified requirement, and an accepted requirement to which the merge gave the current text
- **WHEN** the two are compared as projected text
- **THEN** they are equal

#### Scenario: A merge that kept a deleted word
- **GIVEN** the same change, and an accepted requirement that still holds the deleted word
- **WHEN** the two are compared as projected text
- **THEN** they differ
