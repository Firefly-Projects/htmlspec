## Purpose

Defines what states the project's obligations and what does not, what an agent owes when it
asserts that the project requires something, and what it does when the contract says nothing
at all. It exists because a README example was cited as a contract, a justification was
written for it, and every check passed — the failure is invisible to the pipeline, so the
repair is a stated obligation rather than a stricter check.

## Requirements

### Requirement: The project's obligations are what its accepted specifications state

The project's contract SHALL be what its accepted specifications state, and nothing else SHALL
be treated as stating an obligation of the project.

The following SHALL be non-normative: a README or any other descriptive document, a template
and the example content it writes, a changelog, an archived change's proposal or design, a
comment or documentation string in the code, and the output of any earlier run. Each of these
can describe the project accurately and none of them binds it.

A change's own specification artifact SHALL state what that change establishes rather than
what the project currently requires, and it SHALL become binding when archival applies it to
the accepted specifications. Before that point, citing it as a current obligation SHALL be
incorrect.

Accuracy SHALL NOT confer authority. A description that matches the project exactly is still
a description, and the reason is that nothing keeps it matching: no check compares a README
against the contract, so a reader who treats one as binding is relying on a correspondence
the project never promised to maintain.

#### Scenario: A layout stated only in a README
- **WHEN** a layout, a name, or a rule appears only in a README or other descriptive document
- **THEN** it is not an obligation of the project, whatever it says and however plausible it is

#### Scenario: A description that happens to be correct
- **WHEN** a non-normative document describes the project's current behavior accurately
- **THEN** it is still non-normative, because nothing checks that the description stays correct

#### Scenario: A change's specification before archival
- **WHEN** an agent cites a change's own specification artifact as a current obligation of the project
- **THEN** the citation is incorrect, because that artifact becomes binding when archival applies it

#### Scenario: An archived change's argument
- **WHEN** an agent cites an archived change's proposal or design as an obligation
- **THEN** the citation is incorrect, because the contract that change established is in the accepted specifications

### Requirement: An assertion about the contract names the requirement it rests on

Where an agent states that the project requires, forbids, or fixes something, it SHALL name
the requirement that states it. Where it cannot name one, it SHALL state that the contract is
silent on the point rather than supply a rule.

This obligation SHALL fall on the assertion rather than on every sentence. An agent's account
of its own reasoning, a fact about a command's output, and a description of what an agent did
assert no obligation of the project and SHALL owe nothing.

The boundary SHALL be settled by removing the claim's subject: where what remains states
something the project requires, a requirement is owed; where what remains is the agent's own
choice or a fact about a run, none is. Without this bound the obligation fires on every
sentence an agent writes, which produces padded output and an agent that learns to ignore the
rule.

A named requirement SHALL be identified precisely enough that a reader can find it without
searching. Naming a capability where a requirement is meant leaves the reader the search the
citation was supposed to remove.

An agent SHALL NOT supply a justification for a rule it could not find. A rule with reasoning
attached reads as a decision the project already took, so an invented justification is worse
than an invented rule stated bare.

#### Scenario: An asserted layout with no requirement behind it
- **WHEN** an agent states where an artifact lives, or what it is named, as a fact about the project
- **THEN** it names the requirement that fixes it, or states that the contract is silent

#### Scenario: An agent's own reasoning
- **WHEN** an agent explains why it ordered its work as it did
- **THEN** no requirement is owed, because the sentence asserts nothing about what the project requires

#### Scenario: A fact about a run
- **WHEN** an agent reports what a command produced
- **THEN** no requirement is owed, because the claim is about the run rather than about the contract

#### Scenario: A capability named where a requirement is meant
- **WHEN** an agent cites a capability without naming which of its requirements states the point
- **THEN** the citation is insufficient, because the reader still has to search

#### Scenario: An invented justification
- **WHEN** an agent states a rule it could not find and supplies a reason for it
- **THEN** this is the worst available outcome, because the reasoning presents the invention as a settled decision

### Requirement: Silence is recorded and not filled

Where a decision needs a rule the contract does not state, an agent SHALL record that the
contract is silent, and SHALL then take one of exactly three routes:

1. **Preserve what exists.** Where the thing being decided already has a shape, keep it.
2. **Choose reversibly and own the choice.** Where nothing exists and something must be
   chosen, take the option that is cheapest to change and state that the choice is the
   agent's rather than the project's.
3. **Name the change that would settle it.** Where a reversible choice would mislead because
   the question is large, record it as open and name the change that closes it.

An agent SHALL NOT present its own choice as an obligation of the project. That is the
failure this requirement exists to prevent, and it is distinct from choosing badly: a choice
attributed to the agent is corrected by a contributor who disagrees, and a choice attributed
to the project is not questioned at all.

Recording silence SHALL NOT be grounds to stop. The three routes are what follows the
recording, so an agent that reports silence and produces nothing has not finished its work.

Where a decision is taken under silence at scale, across many artifacts at once, the
preserving route SHALL be preferred. A choice made once and applied to a hundred artifacts
acquires a hundred instances and no requirement behind it, which is the condition that makes
a later correction expensive.

#### Scenario: Silence where a shape already exists
- **WHEN** the contract states no rule and the thing being decided already has a shape
- **THEN** the existing shape is preserved and the silence is recorded

#### Scenario: Silence where nothing exists
- **WHEN** the contract states no rule and something must be chosen
- **THEN** the agent takes the cheapest-to-change option and states that the choice is its own

#### Scenario: Silence too large for a reversible choice
- **WHEN** the contract states no rule and any choice would be read as settling a large question
- **THEN** the agent records the question as open and names the change that would settle it

#### Scenario: A choice presented as the project's rule
- **WHEN** an agent takes a decision under silence and states it as what the project requires
- **THEN** the report is wrong, because a choice attributed to the project is not questioned

#### Scenario: Silence used to stop
- **WHEN** an agent records that the contract is silent and produces nothing further
- **THEN** the work is unfinished, because the recording is a step rather than an exit

#### Scenario: Silence across many artifacts at once
- **WHEN** an operation decides one point under silence and applies it to many artifacts
- **THEN** the preserving route is taken, because a choice repeated at scale is expensive to correct

### Requirement: Generated example content is non-normative and its survival is reported

A template's example content SHALL carry no authority. An example requirement writes no
obligation, an example scenario states no behavior, an example task enumerates no work, and **a
template's placeholder summary states nothing about what a document is for**.

Validation SHALL report a document carrying a unit whose identifier is one a template writes
as an example. **Validation SHALL also report a document whose summary is text a template
writes as a placeholder.** **Every operation that puts a change's contract into the accepted
specifications SHALL refuse while that change's specification carries either.** Archival is one
such operation and SHALL refuse. So SHALL any other, including one that reports the merge for
an agent to perform rather than performing it itself.

Stated as a property of the operation rather than as a list of commands, because a list goes
stale the way a count does. The comparison operation is the case that proves it: it never
edits an accepted specification, so a rule naming only archival leaves it uncovered — and its
report names each requirement no accepted specification holds and directs an agent to merge
it. On a specification straight from a template that direction reads "merge the example
requirement", which is the obligation nobody wrote entering the contract by the hand of an
agent doing as it was told.

The two SHALL differ in severity, and the reason is where each fires. A document carries
example content the moment it is created, because creation writes a template and the result
is required to be valid, so an error at that moment would report a defect the program itself
introduced. Example content reaching an operation that applies a contract is an obligation
nobody wrote entering the accepted specifications, which is what the refusal exists to
prevent.

Both the identifiers and the placeholder text SHALL be derived from the templates rather than
listed separately. A list beside the templates states the same fact twice and goes stale the first
time a template gains a unit or rewords a placeholder.

**Within a unit**, the check SHALL be on the identifier rather than on the example's text. Text is
edited while an identifier survives, so a half-replaced example carries new prose under the
example's identifier — and that is the likely case rather than an edge one.

**A document's summary SHALL be checked on its text, because it has no identifier to survive.** The
summary projects from the paragraphs of the document header, which carry no identifier of their own,
so the rule about units does not narrow to this case — it does not reach it at all. That is a
measured gap rather than a hypothetical one: a document whose entire summary read *"Replace this
paragraph with what this work delivers."* was reported as carrying no diagnostics, and a person
found it by opening the file.

The comparison SHALL be against the whole projected summary rather than against a fragment of it. A
summary an author has written SHALL NOT be reported for quoting a template's wording inside a
longer statement, because an author explaining the placeholder is not carrying it. What is reported
is a summary that *is* the placeholder and nothing else.

Where a document's summary is absent, that SHALL be reported by whatever rule governs an absent
summary and SHALL NOT be reported as example content. An absent summary and a placeholder summary
are different facts about a document, and a rule reporting one as the other would send an author to
delete a paragraph that is not there.

#### Scenario: A newly created document
- **WHEN** a document is created from a template and validated
- **THEN** its example content is reported as a warning, and the document is valid

#### Scenario: A document whose units were replaced and whose summary was not
- **GIVEN** a document created from a template
- **WHEN** every example unit is replaced with the author's own and the masthead paragraph is left as the template wrote it
- **THEN** validation reports the summary as a template's placeholder, because no unit identifier remains to report

#### Scenario: A summary that quotes the placeholder within an author's own statement
- **WHEN** a document's summary states what the document is for and mentions the template's wording inside a longer statement
- **THEN** it is not reported, because the summary is the author's rather than the template's

#### Scenario: A placeholder summary reaching an operation that applies a contract
- **WHEN** archival runs against a change whose specification summary is a template's placeholder
- **THEN** archival does not complete, and the report names the summary rather than a unit

#### Scenario: Every artifact kind
- **WHEN** a document of any artifact kind carries its template's placeholder summary
- **THEN** it is reported, because every template writes one and a rule covering a single kind leaves the others unchecked

#### Scenario: An absent summary
- **WHEN** a document carries no summary at all
- **THEN** it is not reported as example content, because an absent summary is a different fact from a placeholder one

#### Scenario: A specification reaching archival with example content
- **WHEN** archival runs against a change whose specification carries a template's example identifier
- **THEN** archival does not complete, and the report names the unit

#### Scenario: A comparison against the accepted specifications
- **WHEN** the operation that compares a change's contract to the accepted specifications runs against a specification carrying a template's example identifier
- **THEN** it refuses before reporting anything to merge, because its report would otherwise direct an agent to merge the example itself

#### Scenario: A half-replaced example
- **WHEN** a unit's text is rewritten but the template's example identifier is kept
- **THEN** it is reported, because within a unit the check reads the identifier rather than the text

#### Scenario: A template that gains a unit
- **WHEN** a template is changed to write an additional example unit
- **THEN** the check covers the new identifier without a separate list being updated

#### Scenario: A template that rewords its placeholder
- **WHEN** a template's masthead paragraph is reworded
- **THEN** the check covers the new wording without a separate list being updated, because the text is read from the templates

#### Scenario: An example unit cited as an obligation
- **WHEN** an agent cites a template's example requirement as something the project requires
- **THEN** the citation is incorrect, because example content carries no authority
### Requirement: Every workflow carries this obligation in one wording

Every installed workflow's instructions SHALL state the obligation to name a requirement or
name the silence, and the wording SHALL be identical across all of them.

An agent reads one instructions file and no other, so each body SHALL be self-contained and
the statement SHALL therefore be repeated. Repetition is what the identical-wording rule
bounds: independently maintained statements of one rule diverge, and this project has
already measured that divergence in its report guidance.

Stated without a count, deliberately. A count goes stale the first time a workflow is added —
`sync` made an earlier count of seven wrong one release after it was written — and an
obligation on "all seven bodies" is satisfied while an eighth carries nothing.

The identical wording SHALL be checked mechanically against the bodies, so that a workflow
added later cannot ship without it.

#### Scenario: Every body states it
- **WHEN** the installed workflow instructions are examined
- **THEN** each one states the obligation, because an agent reads only the body for the workflow it is performing

#### Scenario: One wording across all of them
- **WHEN** the statement is compared across the bodies
- **THEN** it is identical in each, rather than one paraphrase per body

#### Scenario: A workflow added without it
- **WHEN** a workflow is added whose body omits the statement
- **THEN** the project's own checks report it before release
