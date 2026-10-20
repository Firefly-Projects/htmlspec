## Purpose

Defines the structural and safety rules a specification document must satisfy to be accepted, along with the diagnostics a validator reports, so that a document is both reliably machine-editable and safe to open.

## ADDED Requirements

### Requirement: Structural conformance is enforced

Validation SHALL reject any document that violates the element contract. This includes missing or duplicate identifiers, identifier prefixes that do not match the unit type, requirements without scenarios, scenarios outside requirements, missing normative statements, malformed scenario bodies, and attribute values outside their permitted sets.

#### Scenario: Duplicate identifier rejected
- **WHEN** a document contains two units sharing one identifier
- **THEN** validation fails and reports both locations

#### Scenario: Requirement without a scenario rejected
- **WHEN** a requirement contains no scenario
- **THEN** validation fails and identifies the requirement

#### Scenario: Conforming document accepted
- **WHEN** a document satisfies every structural rule
- **THEN** validation succeeds and reports no structural errors

### Requirement: An upstream edit marks dependents for review rather than gating work

Validation SHALL report a dependent unit as needing review when the unit it references changes materially. Such a report SHALL NOT block continued work, SHALL NOT impose a phase gate, and SHALL NOT require regeneration of the dependent artifact. Exploration, proposal, design, implementation, and verification SHALL be able to loop.

#### Scenario: Requirement revised during implementation
- **GIVEN** tasks and verification that reference a requirement
- **WHEN** that requirement's obligation changes while implementation is underway
- **THEN** validation reports the referencing units as needing review, and work continues

#### Scenario: Reconciliation clears the report
- **WHEN** an agent reconciles the units reported as needing review
- **THEN** validation stops reporting them, and no artifact was regenerated wholesale

#### Scenario: Stale reference is reported, not fatal
- **WHEN** a dependent unit still resolves but describes behavior the referenced requirement no longer states
- **THEN** validation reports the inconsistency as needing review rather than as a structural error

### Requirement: Cross-references must resolve

Validation SHALL verify that every reference to a semantic unit resolves to a unit that exists. This includes fragment links within the document, and references from tasks, verification, evidence, and risks to the requirements and scenarios they address.

#### Scenario: Dangling task reference
- **WHEN** a task references a requirement identifier that does not exist
- **THEN** validation fails and reports the task and the unresolved identifier

#### Scenario: Broken fragment link
- **WHEN** a document links to a fragment with no matching element
- **THEN** validation fails and reports the link

#### Scenario: Reference to a removed requirement
- **WHEN** a task or verification references a requirement marked removed
- **THEN** validation fails

### Requirement: Markup must be deterministically parseable

Validation SHALL reject a document whose markup cannot be parsed into a single unambiguous structure. Constructs that different parsers would interpret differently SHALL be rejected rather than accepted with a guess.

A conforming document SHALL produce zero recovered parse errors by default. Any recovered parse error SHALL fail validation unless the project permits that specific recovery.

#### Scenario: Improperly nested elements
- **WHEN** a document contains overlapping or improperly nested elements
- **THEN** validation fails and reports the location

#### Scenario: Ambiguous construct
- **WHEN** a document contains markup whose interpretation depends on parser error recovery
- **THEN** validation fails rather than silently accepting one interpretation

### Requirement: Permitted parser recoveries are enumerated and pinned

The project SHALL pin an exact parser version. Every permitted recovery SHALL be enumerated individually, SHALL carry a stable internal diagnostic code owned by this project, and SHALL be justified by a fixture that demonstrates the recovery is unambiguous across consumers.

Permitted recoveries SHALL NOT be derived by collecting whatever a sample of documents happens to produce. A parser version change SHALL require deliberate review of the enumeration before the new version is adopted.

#### Scenario: Diagnostic codes are owned by this project
- **GIVEN** a parser that reports a recovery as free text rather than as a typed code
- **WHEN** validation reports that recovery
- **THEN** it reports this project's own stable diagnostic code, so a reworded parser message does not change the reported code

#### Scenario: Parser rewords a message
- **WHEN** a new parser version changes the wording of a recovery message
- **THEN** the project's checks fail rather than silently widening or narrowing what documents are accepted

#### Scenario: Ambiguous recovery is not permitted
- **WHEN** a recovery is proposed for the permitted enumeration and no fixture shows it is unambiguous across consumers
- **THEN** the recovery is not permitted

#### Scenario: Coarse parser message covering two faults
- **GIVEN** a parser that reports two distinct faults with one identical message
- **WHEN** the enumeration is built
- **THEN** it distinguishes the two faults by parser state, or it permits neither

### Requirement: Remote and hidden network dependencies are rejected

Validation SHALL reject any document that depends on a resource outside the project. This includes remote scripts, stylesheets, fonts, images, and iframes, and any construct capable of initiating a network request when the document is opened.

#### Scenario: Remote script rejected
- **WHEN** a document references a script by absolute URL
- **THEN** validation fails and reports the reference

#### Scenario: Remote font rejected
- **WHEN** a document's styling loads a font from a remote origin
- **THEN** validation fails

#### Scenario: Concealed request rejected
- **WHEN** a document contains a construct that would issue a network request without a visible remote reference, such as a prefetch directive or a beacon
- **THEN** validation fails

#### Scenario: Local asset accepted
- **WHEN** a document references the project's shared runtime by relative path
- **THEN** validation succeeds

### Requirement: Code examples cannot execute

Validation SHALL ensure that example code shown in a document is inert. Code presented as content SHALL NOT be capable of executing as part of the page, and SHALL NOT be able to terminate its container and inject markup into the document.

The security property is the absence of element descendants. A code element's children SHALL contain text and comment nodes only. Validation SHALL NOT require exactly one text node, because a parser splits text around a comment and an empty example has no children at all.

#### Scenario: Script shown as an example
- **WHEN** a document displays a JavaScript snippet as an example
- **THEN** validation succeeds and the snippet does not execute when the document is opened

#### Scenario: Empty example accepted
- **WHEN** a code element has no children
- **THEN** validation succeeds, because an empty example carries no element descendant

#### Scenario: Text split by a comment accepted
- **WHEN** a parser represents a code element's content as two text nodes separated by a comment
- **THEN** validation succeeds, because no child is an element

#### Scenario: Presentational element inside an example rejected
- **WHEN** a code element contains a child element, including a purely presentational one
- **THEN** validation fails, because the example is no longer inert text

#### Scenario: Container escape rejected
- **WHEN** an example's content would close its containing element and introduce live markup
- **THEN** validation fails and reports the example

#### Scenario: Executable script accepted only from the pinned runtime
- **WHEN** a document contains an executable script that is not part of the pinned runtime
- **THEN** validation fails

### Requirement: Visualization specifications are validated

Validation SHALL check every embedded diagram or chart specification against its own format. An invalid specification SHALL be reported at authoring time rather than failing silently in the browser.

#### Scenario: Invalid chart specification
- **WHEN** a document embeds a chart specification that its format rejects
- **THEN** validation fails and reports the location and the underlying error

#### Scenario: Invalid diagram source
- **WHEN** a document embeds diagram source that cannot be parsed by the diagram library
- **THEN** validation fails

#### Scenario: Chart data is local
- **WHEN** a chart specification loads its data from a remote URL
- **THEN** validation fails, because the document would not render offline

### Requirement: No-JavaScript readability is enforced

Validation SHALL reject a document whose semantic content is not present in its markup. Content that only appears after scripts run SHALL be treated as absent.

#### Scenario: Script-injected requirement rejected
- **WHEN** a requirement's text is supplied by a script rather than present in the markup
- **THEN** validation fails

#### Scenario: Diagram without a fallback
- **WHEN** a document contains a diagram with no source or textual description present in the markup
- **THEN** validation fails

### Requirement: Diagnostics are actionable and machine-readable

Every validation failure SHALL report the rule violated, the location within the document, and the offending identifier or content where one applies. Diagnostics SHALL be available in a machine-readable form so that agents and continuous integration can consume them without parsing prose.

#### Scenario: Human-readable output
- **WHEN** a contributor validates a failing document
- **THEN** the output names the rule, the location, and what to fix

#### Scenario: Machine-readable output
- **WHEN** an agent validates a failing document and requests structured output
- **THEN** it receives each diagnostic as structured data with a stable rule identifier

#### Scenario: Stable rule identifiers
- **WHEN** the same violation is reported across versions of the validator
- **THEN** the rule identifier is unchanged, so suppressions and tests remain valid

### Requirement: All violations are reported in one pass

Validation SHALL report every violation it can detect in a single run rather than stopping at the first. Where one failure prevents further analysis, validation SHALL report what it could determine and state that analysis was curtailed.

#### Scenario: Multiple independent violations
- **WHEN** a document has a duplicate identifier, a dangling reference, and a remote script
- **THEN** all three are reported in one run

#### Scenario: Unparseable document
- **WHEN** a document cannot be parsed at all
- **THEN** validation reports the parse failure and states that no further rules were evaluated

### Requirement: Violations are classified by severity

Every rule SHALL be classified as an error or a warning. An error SHALL cause validation to fail. A warning SHALL be reported without failing validation. Rules concerning structural conformance, unresolved references, executable content, and network dependencies SHALL be errors.

#### Scenario: Warning does not fail validation
- **WHEN** a document triggers only warnings
- **THEN** validation succeeds and the warnings are reported

#### Scenario: Warnings can be made fatal
- **WHEN** a contributor requests strict validation
- **THEN** warnings are treated as errors, so continuous integration can enforce them

#### Scenario: Safety rules cannot be downgraded
- **WHEN** a document contains a remote script or executable content
- **THEN** the violation is reported as an error and cannot be reduced to a warning
