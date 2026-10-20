## ADDED Requirements

### Requirement: A partial verification states what it did not check

A verification SHALL be able to record that its check covered only part of what the verification names. That outcome is `partial`. It means the check ran against the requirement or scenario the verification names, reached part of it, and found that part to hold. A check whose covered part did not hold is `failed`, whatever else it did not reach.

The set of outcome values belongs to the published schema, and this requirement does not restate it. "The element contract is published as a machine-readable schema" makes the schema the single source of truth for permitted attribute values. This requirement names `partial` because it attaches an obligation to that one value, and a condition on a value's companion field is a validator rule, as a dropped task's reason is.

**A `partial` verification SHALL state what it did not check**, in one or more paragraphs marked `data-field="unchecked"`. Each paragraph SHALL name a part of the requirement or scenario that the check did not cover. A `partial` verification with no such paragraph SHALL make the document invalid. An unmarked paragraph SHALL NOT satisfy this, because an unmarked paragraph in a verification is a note.

The reason is the one that makes a dropped task state why, in "A task declares completion state": an outcome that says less than the whole is useful only with the rest named. A verification names units, and the smallest unit it can name is a scenario, so a check that reached one clause of a scenario cannot narrow its reference to that clause. Without `partial`, its author has two choices, and both are wrong. `passed` claims the part the check never reached. Deleting the verification loses the record of the part it did. A `partial` that named nothing would be a third way to say nothing about what is left.

The `unchecked` paragraphs SHALL project into the verification's `unchecked` field, in document order, and SHALL NOT project into its notes. A verification recording any other outcome MAY carry them, so that a check that failed or was blocked can state what it never reached.

A check whose condition, subject, or trigger differs from the one a scenario states did not reach that scenario at all, and it is not a partial verification of it. `partial` records a check of the right thing that stopped short, not a check of something nearby.

#### Scenario: A check that reached part of its scenario
- **GIVEN** a scenario whose Then states two outcomes, and a check that observed only the first
- **WHEN** a verification naming that scenario carries `data-outcome="partial"` and a `data-field="unchecked"` paragraph naming the second
- **THEN** the document is valid, and a consumer reads both the outcome and what the check did not cover without parsing prose

#### Scenario: A partial verification that does not say what it left
- **WHEN** a verification carries `data-outcome="partial"` and no `data-field="unchecked"` paragraph
- **THEN** the document is invalid, and the report names the verification

#### Scenario: An unmarked paragraph does not state it
- **WHEN** a `partial` verification carries an unmarked paragraph describing what it did not check, and no `data-field="unchecked"` paragraph
- **THEN** the document is invalid, because an unmarked paragraph in a verification is a note

#### Scenario: Two parts left unchecked
- **WHEN** a `partial` verification carries two `data-field="unchecked"` paragraphs
- **THEN** its `unchecked` field holds both, in document order, because a part dropped from the record would understate what the check missed

#### Scenario: A failed check that says what it did not reach
- **WHEN** a verification carries `data-outcome="failed"` and a `data-field="unchecked"` paragraph
- **THEN** the document is valid

#### Scenario: Any other outcome without it
- **WHEN** a verification carries `data-outcome="passed"` and no `data-field="unchecked"` paragraph
- **THEN** the document is valid, and the verification means exactly what it meant before `partial` existed
