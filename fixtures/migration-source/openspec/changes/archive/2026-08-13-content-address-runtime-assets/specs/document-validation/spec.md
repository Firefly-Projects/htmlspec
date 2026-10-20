## ADDED Requirements

### Requirement: An asset reference must match the bytes it names

Validation SHALL reject a document whose asset reference does not resolve to stored bytes whose hash equals the hash named by that reference. A reference naming bytes the project does not store SHALL be reported as unresolved. A reference resolving to bytes whose hash differs from the name SHALL be reported as a mismatch, and the two SHALL be distinguishable diagnostics because they have different causes.

This rule SHALL apply only to a document validated within a project that has an asset store. A document validated outside such a project SHALL NOT have its asset references resolved, and SHALL NOT be rejected for naming bytes that no store holds. The rule states a relationship between a document and the bytes beside it, and where there are no bytes there is no relationship to violate.

A document SHALL NOT be able to exempt itself. The scope SHALL be determined by whether a store is present, never by a declaration inside the document, because a rule a document can switch off protects nothing.

This rule SHALL be an error rather than a warning, and SHALL NOT be reducible to a warning. A document that renders against bytes other than the ones it names is misreporting its own provenance.

#### Scenario: Reference matches stored bytes
- **WHEN** every asset reference in a document names the hash of the bytes stored at its location
- **THEN** validation succeeds

#### Scenario: Document validated with no store present
- **GIVEN** a document that is not part of a project with an asset store, such as a conformance fixture whose references are symbolic
- **WHEN** it is validated
- **THEN** its asset references are not resolved, and their targets being absent is not a violation

#### Scenario: A document cannot exempt itself
- **WHEN** a document declares that its asset references are not to be resolved, by any marker it carries
- **THEN** the declaration has no effect, and a document inside a project with a store is checked regardless

#### Scenario: Reference to bytes the project does not store
- **WHEN** a document references an asset hash that no stored asset matches
- **THEN** validation fails, reports the document, the reference, and that the bytes are absent

#### Scenario: Stored bytes do not match the name
- **GIVEN** an asset location whose contents were replaced with different bytes
- **WHEN** a document referencing that location is validated
- **THEN** validation fails and reports a mismatch rather than an absent asset

#### Scenario: Stale reference cannot pass
- **GIVEN** a document authored against one revision of a stylesheet
- **WHEN** it is stored beside a different revision of that stylesheet
- **THEN** validation fails, because the reference and the bytes disagree

#### Scenario: Mismatch cannot be downgraded
- **WHEN** a contributor requests that asset mismatches be treated as warnings
- **THEN** the violation is still reported as an error

### Requirement: A project must not store one asset's bytes more than once

Validation SHALL reject a project that stores identical asset bytes at more than one location. The report SHALL name every location holding the duplicated bytes, so a contributor can tell which copy to remove.

Two assets whose bytes differ SHALL NOT be reported as duplicates, however similar their names, versions, or purposes.

This rule takes a project as its subject rather than a document, so it SHALL be evaluated where a project is in view. A directory holding no asset store has nothing to duplicate, and SHALL NOT be reported.

#### Scenario: Duplicate copies of one library
- **GIVEN** a project storing the same library bytes in two locations
- **WHEN** the project is validated
- **THEN** validation fails and names both locations

#### Scenario: A copy per document is rejected
- **GIVEN** a project in which each document carries its own copy of the runtime
- **WHEN** the project is validated
- **THEN** validation fails, because the shared-runtime obligation is violated

#### Scenario: Different revisions are not duplicates
- **GIVEN** a project storing two revisions of one stylesheet whose bytes differ
- **WHEN** the project is validated
- **THEN** validation succeeds, because distinct bytes are not a duplicate

#### Scenario: One store serving many documents
- **GIVEN** many documents referencing one stored copy of each library
- **WHEN** the project is validated
- **THEN** validation succeeds

### Requirement: Asset rules are checked as part of the project's check suite

The asset reference and duplication rules SHALL be evaluated by the project's single check command, so a contributor and continuous integration verify the same asset topology. A project whose documents validate individually but whose asset topology violates these rules SHALL fail the check suite.

#### Scenario: Topology regression fails the suite
- **WHEN** a change introduces a second copy of a stored asset
- **THEN** the check command fails and identifies the duplicated locations

#### Scenario: Per-document validation is not sufficient
- **GIVEN** a project whose every document passes validation on its own
- **WHEN** two documents together hold duplicate asset bytes
- **THEN** the check command still fails, because the violation is a property of the project rather than of one document
