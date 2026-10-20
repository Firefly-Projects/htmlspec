## MODIFIED Requirements

### Requirement: Cross-references must resolve

Validation SHALL verify that every reference to a semantic unit resolves to a unit that exists. This includes fragment links within the document, and references from tasks, verification, evidence, and risks to the requirements and scenarios they address.

A reference resolves in the places that document-schema's "Cross-references address units by identifier" makes reachable. **Validation SHALL read the other documents a reference can reach for the identifiers they declare, and SHALL report nothing about them.** A diagnostic belongs to the document being validated. The documents it reaches are reported when they are validated themselves, so a defect in one of them is reported once, against the document that has it.

A reference that resolves nowhere SHALL be reported against the unit that makes it, and the report SHALL name the places that were searched. "No unit in the document declares that identifier" is true of a document that reaches only itself. For a task in a change it would hide that the change's specification and the accepted specifications were searched too.

**A reachable document that cannot be read, or whose markup needs the parser to recover, declares nothing a reference can rely on.** A reference that resolves in no other place SHALL be reported, and the report SHALL name that document as unread. It SHALL NOT claim that no unit declares the identifier. A document with more than one reading has no determinate content, so identifiers taken from it are a guess. And a report that no unit declares the identifier would send the author to add a unit that may already be there.

A change that has no specification yet is a change in progress. Its specification is then a place that declares nothing, and it SHALL NOT be reported as a document that could not be read.

#### Scenario: Dangling task reference
- **WHEN** a task references a requirement identifier that does not exist
- **THEN** validation fails and reports the task and the unresolved identifier

#### Scenario: Broken fragment link
- **WHEN** a document links to a fragment with no matching element
- **THEN** validation fails and reports the link

#### Scenario: Reference to a removed requirement
- **WHEN** a task or verification references a requirement marked removed
- **THEN** validation fails

#### Scenario: A reference resolved by the change's specification
- **GIVEN** a change whose specification declares `req-x`, and also declares a requirement with no scenario
- **WHEN** the change's tasks document, whose task names `req-x` in its `data-refs`, is validated
- **THEN** validation of the tasks document succeeds, and it reports nothing about the reference or about the specification

#### Scenario: A dangling reference in a change says where it looked
- **WHEN** a task in a change names in its `data-refs` an identifier that no reachable place declares
- **THEN** validation fails and reports the task, the identifier, and the places that were searched

#### Scenario: A reachable document that cannot be read
- **GIVEN** a change whose specification's markup needs the parser to recover
- **WHEN** a task in that change names in its `data-refs` a requirement that no other reachable place declares
- **THEN** validation of the tasks document fails, and the report names the specification as the document that could not be read

#### Scenario: A change with no specification yet
- **GIVEN** a change that has no specification, and an accepted specification declaring `req-y`
- **WHEN** a task in that change names `req-y` in its `data-refs`
- **THEN** the reference resolves, and nothing is reported about the missing specification

### Requirement: The contract rules apply to every kind

The rules that make a document readable, parseable, and offline SHALL apply to all four kinds without relaxation. Freedom of structure at the proposal and design kinds SHALL NOT extend to markup that cannot be parsed unambiguously, a reference that resolves to nothing, a remote or executable dependency, or an asset reference that disagrees with the bytes beside it.

A kind decides how much *shape* validation requires. It SHALL NOT decide whether a document opens.

A kind SHALL NOT decide what a reference can reach, either. That is decided by where a document sits, so a change's proposal, design, and tasks reach the same documents.

#### Scenario: A proposal with a remote script
- **WHEN** a proposal loads a script from outside the project
- **THEN** validation fails, exactly as it does for any other kind

#### Scenario: A design with a dangling reference
- **WHEN** a design references an identifier that no unit declares, in the design or in any document it can reach
- **THEN** validation fails, because a pointer to nothing is a defect at every stage

#### Scenario: A design references its change's specification
- **WHEN** a decision in a change's design names in its `data-refs` a requirement that the change's specification declares
- **THEN** the reference resolves, exactly as it would from a task in the same change

#### Scenario: A proposal that needs error recovery
- **WHEN** a proposal's markup requires the parser to recover
- **THEN** validation fails, because a document with more than one reading has no determinate content whatever its kind
