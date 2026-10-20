## MODIFIED Requirements

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
