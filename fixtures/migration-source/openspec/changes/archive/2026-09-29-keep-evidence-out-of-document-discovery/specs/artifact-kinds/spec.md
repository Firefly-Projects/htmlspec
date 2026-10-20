## ADDED Requirements

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
