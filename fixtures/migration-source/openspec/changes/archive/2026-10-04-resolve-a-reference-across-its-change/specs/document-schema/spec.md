## MODIFIED Requirements

### Requirement: Cross-references address units by identifier

A task, verification, evidence, or risk unit MAY reference the semantic units it relates to. Such a reference SHALL name the target by its identifier, and SHALL resolve to a unit that exists in the same document or in a document reachable from it.

**A reference written in a `data-refs` attribute SHALL be resolved in the following places, in this order, and SHALL resolve to the unit declared in the first place that declares its identifier:**

1. the document that makes the reference;
2. where that document is one of a change's documents other than its specification, that change's own specification;
3. where that document is one of a change's documents, including its specification, or is an accepted specification, the project's accepted specifications, every one of them.

A change's documents are the documents directly within a change's directory, whether the change is active or archived. The project's accepted specifications are the documents beneath the place where the project keeps them. Nothing else is reachable. A document outside any project, a file beneath a change's evidence, and a runbook's documents reach only themselves. A change's documents do not reach another change's documents, and they do not reach their own change's proposal, design, or tasks.

A change's other documents reach its specification because they exist to serve it. A proposal argues for what the specification states, a design reasons about it, and the tasks carry it out. The tasks template already tells an author that a task may reference the requirement it serves "once that requirement exists in this change's specification". The accepted specifications are reachable because they are the contract every change starts from.

**The order decides which unit a reference means when two places declare one identifier.** A change's specification restates, under the accepted identifier, each accepted requirement the change modifies or withdraws. For the change's own documents, the change's statement is the one that counts. So a task that references a requirement its change withdraws targets a withdrawn obligation, even while the accepted specification still holds that requirement unchanged.

**Every accepted specification is reachable, and an accepted specification reaches the others.** A reference names a unit, not a capability. The merge that applies a change places each requirement in the accepted specification for its capability, so a reference that resolved in a change's specification must still resolve after that change's units are merged into two different accepted specifications.

A reference written as a fragment link, `href="#…"`, SHALL resolve within the document that makes it. It is a hyperlink into that page. If it resolved somewhere else while the link arrived nowhere, the reference would be true and the hyperlink false.

Where more than one accepted specification declares an identifier, a reference to it SHALL resolve. Which of them it means is not decided here.

A reference SHALL NOT resolve to a unit that the place resolving it marks `removed`. This holds in every place, not only in the document that makes the reference. A task and the requirement it serves rarely share a document, so a rule confined to one document would miss the case the scenario "Reference to a removed requirement" names.

#### Scenario: Task implements a requirement
- **WHEN** a task declares that it satisfies `req-theme-selection`
- **THEN** consumers can report implementation progress per requirement

#### Scenario: Verification covers a scenario
- **WHEN** a verification unit references `scenario-theme-persists`
- **THEN** consumers can report which scenarios have been verified and which have not

#### Scenario: Dangling reference
- **WHEN** a task references `req-does-not-exist`
- **THEN** the document is invalid

#### Scenario: Reference to a removed requirement
- **WHEN** a task references a requirement carrying `data-change="removed"`
- **THEN** the document is invalid, because work cannot target a withdrawn obligation

#### Scenario: A task references a requirement in its change's specification
- **GIVEN** a change whose specification declares `req-x`
- **WHEN** a task in that change's tasks document names `req-x` in its `data-refs`
- **THEN** the reference resolves, and the tasks document is valid

#### Scenario: A task references a requirement only the accepted contract holds
- **GIVEN** an accepted specification declaring `req-y`, and no document in the change declaring it
- **WHEN** a task in the change names `req-y` in its `data-refs`
- **THEN** the reference resolves

#### Scenario: A change withdraws a requirement the accepted contract still holds
- **GIVEN** an accepted specification declaring `req-y` unchanged, and a change whose specification declares `req-y` with `data-change="removed"`
- **WHEN** a task in that change names `req-y` in its `data-refs`
- **THEN** the document is invalid, because the change's specification is reached first and it withdraws the obligation

#### Scenario: Accepted specifications reference one another
- **WHEN** a unit in one accepted specification names, in its `data-refs`, a requirement that another accepted specification declares
- **THEN** the reference resolves

#### Scenario: An identifier two accepted specifications declare
- **WHEN** a task names in its `data-refs` an identifier that two accepted specifications both declare
- **THEN** the reference resolves

#### Scenario: A reference to another change
- **WHEN** a task names in its `data-refs` a requirement that only another change's specification declares
- **THEN** the document is invalid, because a change's documents reach their own change's specification and no other

#### Scenario: A document that reaches only itself
- **WHEN** a document outside any project, or a file beneath a change's evidence, names in its `data-refs` an identifier it does not declare itself
- **THEN** the document is invalid, because nothing beyond it is reachable

#### Scenario: A runbook's document names an accepted requirement
- **WHEN** a runbook's tasks document names in its `data-refs` a requirement that only an accepted specification declares
- **THEN** the document is invalid, because a runbook is not a change and its documents reach only themselves

#### Scenario: A fragment link to a unit in another document
- **GIVEN** a change whose specification declares `req-x`
- **WHEN** a task in that change's tasks document links to `#req-x`
- **THEN** the document is invalid, because a fragment link resolves within its own document
