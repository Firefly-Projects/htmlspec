## MODIFIED Requirements

### Requirement: A specification requirement without verification is reported as incomplete

Validation SHALL report a requirement in a `specification` document that no verification names. A specification is an as-built record, and a requirement with nothing attesting to it records an intention rather than a result.

This SHALL be **advisory** — a warning, never an error. Verification arrives as work completes, so a specification in progress legitimately has requirements not yet verified, and a rule that failed the document would gate the work that produces the verification.

A requirement that a verification names SHALL NOT be reported, whatever outcome that verification records, `partial` included. The report asks whether a verification names the requirement, not what the verification found. A requirement whose check failed, was blocked, or covered only part of it is named by a record that says so, and reporting it as unverified would treat an unwelcome result as a missing one. That would teach an author to record a pass, which is the reason runbook-execution's "A run's conformance to its runbook's contract is reported and never gated" gives for a run. This rule SHALL apply only to the `specification` kind: a requirement unit appearing in any other kind SHALL NOT be checked for verification, because it is not part of the contract there.

#### Scenario: A requirement with no verification
- **WHEN** a specification states a requirement that no verification names
- **THEN** validation reports a warning, and the document is still accepted

#### Scenario: A fully verified specification
- **WHEN** every requirement in a specification is named by a verification
- **THEN** nothing is reported

#### Scenario: A requirement whose check failed
- **WHEN** a specification states a requirement whose only verification records the outcome `failed`
- **THEN** nothing is reported about that requirement, because a verification names it and its outcome states the result

#### Scenario: A requirement verified only in part
- **WHEN** a specification states a requirement whose only verification records the outcome `partial`
- **THEN** nothing is reported about that requirement, because a verification names it, and that verification states what it did not check

#### Scenario: Verification is not a gate
- **WHEN** a contributor validates a specification whose verification is not yet recorded
- **THEN** validation succeeds, because recording verification is the work in progress rather than a violation

#### Scenario: An unverified requirement in a proposal
- **WHEN** a proposal carries a requirement unit that no verification names
- **THEN** nothing is reported, because a proposal's units are not part of the contract
