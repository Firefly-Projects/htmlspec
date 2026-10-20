## MODIFIED Requirements

### Requirement: An asset reference must match the bytes it names

Validation SHALL reject a document whose asset reference does not resolve to stored bytes whose hash equals the hash named by that reference. A reference naming bytes the project does not store SHALL be reported as unresolved. A reference resolving to bytes whose hash differs from the name SHALL be reported as a mismatch, and the two SHALL be distinguishable diagnostics because they have different causes.

This rule SHALL apply only to a document validated within a project that has an asset store. A document validated outside such a project SHALL NOT have its asset references resolved, and SHALL NOT be rejected for naming bytes that no store holds. The rule states a relationship between a document and the bytes beside it, and where there are no bytes there is no relationship to violate.

Where a document's asset reference ascends out of the document's own directory, and no asset store is found in any directory above it, validation SHALL report that reference. Such a reference asserts that a store sits above the document, and the absence of one contradicts the reference rather than placing the document outside the rule's scope.

A reference that does not ascend SHALL remain unreported when no store is found. This is what keeps a document whose references are symbolic — a conformance fixture, or a document written to be read rather than rendered — outside the rule, and the distinction is exact: a fixture's references name a store beside the document that has never existed, while an ascending reference names a store the document was generated against.

A document SHALL NOT be able to exempt itself. The scope SHALL be determined by whether a store is present and by the shape of the reference, never by a declaration inside the document, because a rule a document can switch off protects nothing. Reference shape is not such a declaration: it is written by whatever generated the document, and changing it to silence the rule breaks the reference it describes.

This rule SHALL be an error rather than a warning, and SHALL NOT be reducible to a warning. A document that renders against bytes other than the ones it names is misreporting its own provenance.

#### Scenario: Reference matches stored bytes
- **WHEN** every asset reference in a document names the hash of the bytes stored at its location
- **THEN** validation succeeds

#### Scenario: Document validated with no store present
- **GIVEN** a document that is not part of a project with an asset store, such as a conformance fixture whose references are symbolic
- **WHEN** it is validated
- **THEN** its asset references are not resolved, and their targets being absent is not a violation

#### Scenario: An ascending reference with no store above it
- **GIVEN** a document whose asset references ascend out of its own directory, such as a change directory copied away from the project that holds its store
- **WHEN** it is validated
- **THEN** validation reports the ascending references, because each one names a store that is not there

#### Scenario: Strict validation does not accept dead ascending references
- **GIVEN** a document carrying only ascending asset references and no store in any directory above it
- **WHEN** it is validated in strict mode
- **THEN** validation does not report the document as accepted

#### Scenario: A document cannot exempt itself
- **WHEN** a document declares that its asset references are not to be resolved, by any marker it carries
- **THEN** the declaration has no effect, and a document inside a project with a store is checked regardless

#### Scenario: Reference to bytes the project does not store
- **WHEN** a document references an asset hash that no stored asset matches
- **THEN** validation fails and reports the reference as unresolved

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
