## ADDED Requirements

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

## MODIFIED Requirements

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
