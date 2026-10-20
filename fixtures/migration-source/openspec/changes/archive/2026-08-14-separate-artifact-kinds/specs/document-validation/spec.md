## ADDED Requirements

### Requirement: A specification must not carry task units

Validation SHALL reject a `specification` document carrying a task unit. The report SHALL name the task and say that a specification records the obligation a task satisfied rather than the work that satisfied it.

This SHALL be the **only** placement rule in the vocabulary. No other combination of unit and kind SHALL be prohibited, and validation SHALL report nothing about any other. In particular, a requirement unit in a `proposal`, a task unit in a `design`, and a decision unit in a `tasks` document SHALL all be accepted without diagnostic.

The rule is an error because a specification is read by tools and by later changes as the statement of the contract, and a task appearing there would offer work as though it were an obligation.

#### Scenario: A task in a specification
- **WHEN** a specification carries a task unit
- **THEN** validation fails and names the unit

#### Scenario: A requirement in a proposal
- **WHEN** a proposal carries a requirement unit
- **THEN** validation reports nothing about it, and the document is accepted

#### Scenario: A task in a design
- **WHEN** a design carries a task unit
- **THEN** validation reports nothing about it, and the document is accepted

#### Scenario: Prose in any kind
- **WHEN** a document of any kind carries headings, paragraphs, and figures with no unit markup
- **THEN** no placement diagnostic is reported, because prose belongs everywhere

### Requirement: A tasks document must carry at least one task

Validation SHALL reject a `tasks` document carrying no task unit. The stage exists to enumerate work, and a tasks document that enumerates none records nothing.

This is a **completeness** rule and not a placement rule. It constrains what the document must contain rather than which units are permitted in it, so a tasks document carrying other units alongside its tasks SHALL NOT be reported for them.

#### Scenario: An empty tasks document
- **WHEN** a tasks document carries no task unit
- **THEN** validation fails

#### Scenario: A tasks document carrying other units
- **WHEN** a tasks document carries its tasks and also a decision unit
- **THEN** validation reports nothing about the decision, because the rule constrains what must be present rather than what is permitted

### Requirement: A specification requirement without verification is reported as incomplete

Validation SHALL report a requirement in a `specification` document that no verification names. A specification is an as-built record, and a requirement with nothing attesting to it records an intention rather than a result.

This SHALL be **advisory** — a warning, never an error. Verification arrives as work completes, so a specification in progress legitimately has requirements not yet verified, and a rule that failed the document would gate the work that produces the verification.

A requirement that a verification names SHALL NOT be reported. This rule SHALL apply only to the `specification` kind: a requirement unit appearing in any other kind SHALL NOT be checked for verification, because it is not part of the contract there.

#### Scenario: A requirement with no verification
- **WHEN** a specification states a requirement that no verification names
- **THEN** validation reports a warning, and the document is still accepted

#### Scenario: A fully verified specification
- **WHEN** every requirement in a specification is named by a verification
- **THEN** nothing is reported

#### Scenario: Verification is not a gate
- **WHEN** a contributor validates a specification whose verification is not yet recorded
- **THEN** validation succeeds, because recording verification is the work in progress rather than a violation

#### Scenario: An unverified requirement in a proposal
- **WHEN** a proposal carries a requirement unit that no verification names
- **THEN** nothing is reported, because a proposal's units are not part of the contract

### Requirement: The contract rules apply to every kind

The rules that make a document readable, parseable, and offline SHALL apply to all four kinds without relaxation. Freedom of structure at the proposal and design kinds SHALL NOT extend to markup that cannot be parsed unambiguously, a reference that resolves to nothing, a remote or executable dependency, or an asset reference that disagrees with the bytes beside it.

A kind decides how much *shape* validation requires. It SHALL NOT decide whether a document opens.

#### Scenario: A proposal with a remote script
- **WHEN** a proposal loads a script from outside the project
- **THEN** validation fails, exactly as it does for any other kind

#### Scenario: A design with a dangling reference
- **WHEN** a design references an identifier that no unit in the document declares
- **THEN** validation fails, because a pointer to nothing is a defect at every stage

#### Scenario: A proposal that needs error recovery
- **WHEN** a proposal's markup requires the parser to recover
- **THEN** validation fails, because a document with more than one reading has no determinate content whatever its kind
