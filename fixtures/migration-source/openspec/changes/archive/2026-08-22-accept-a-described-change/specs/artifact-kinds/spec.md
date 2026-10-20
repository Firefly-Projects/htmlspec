## MODIFIED Requirements

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

## ADDED Requirements

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
