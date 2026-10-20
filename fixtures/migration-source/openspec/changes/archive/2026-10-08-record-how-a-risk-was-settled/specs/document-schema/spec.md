## ADDED Requirements

### Requirement: A risk records how it was settled

A risk unit SHALL contain a heading and one statement of how the change can fail. **It SHALL declare how the risk was settled**, in a `data-response` attribute with one of the values `mitigated` or `accepted`. A risk with no `data-response` attribute SHALL be interpreted as `mitigated`.

A `mitigated` risk is one the change holds down. It SHALL state what holds it down, in a paragraph marked `data-field="mitigation"`. Where the mitigation does not reach all the way, that paragraph SHALL say how far it reaches. The extent SHALL NOT be a value of its own.

An `accepted` risk is one the change bears as it stands. It SHALL NOT be required to state a mitigation. It MAY carry a `mitigation` paragraph, which then states the part of the risk that something does hold down. It SHALL NOT be required to state its reason in a field of its own, because its statement and its notes are where an author says why a risk is borne.

The reason is that a risk is settled in one of two ways, and only one of them has a mitigation to state. A contract that required a mitigation of every risk left the author of an accepted risk two choices: invent a mitigation, or write a mitigation paragraph that denies being one. Either choice puts a false statement in the risk's mitigation.

The default is `mitigated` because every risk written before this attribute existed had to state a mitigation. Each such risk keeps both its meaning and its validity.

The response SHALL be stated separately from the unit's change state. The change state says what a change does to the risk, and the response says how the risk is answered. Neither SHALL be inferred from the other. The response SHALL NOT be inferred from whether a mitigation paragraph is present, because an accepted risk may carry one.

The response SHALL project into the risk's `response` field, which every risk in the model carries, including a risk whose markup declares none. A `data-response` attribute on a unit other than a risk SHALL project nowhere, as a `data-state` attribute does on a unit other than a task.

The following SHALL make a document invalid:

- a risk whose response is `mitigated`, declared or by default, with no `mitigation` paragraph;
- a `data-response` value other than `mitigated` or `accepted`.

#### Scenario: A mitigated risk
- **WHEN** a risk carries `data-response="mitigated"`, a statement, and a `data-field="mitigation"` paragraph
- **THEN** the document is valid, and the risk's response is `mitigated`

#### Scenario: A risk written before the response attribute existed
- **WHEN** a risk carries a statement and a mitigation paragraph, and no `data-response` attribute
- **THEN** the document is valid, and the risk's response is `mitigated`

#### Scenario: A mitigated risk with no mitigation
- **WHEN** a risk whose response is `mitigated`, declared or by default, carries no mitigation paragraph
- **THEN** the document is invalid, and the report names `data-response="accepted"` as the declaration for a risk the change bears

#### Scenario: An accepted risk with no mitigation
- **WHEN** a risk carries `data-response="accepted"` and a statement, and no mitigation paragraph
- **THEN** the document is valid, its response is `accepted`, and it has no mitigation

#### Scenario: An accepted risk that names a partial measure
- **WHEN** a risk carries `data-response="accepted"`, a statement, and a mitigation paragraph
- **THEN** the document is valid, its response is `accepted`, and its mitigation projects as it would on a mitigated risk

#### Scenario: A risk that is partly held down
- **WHEN** a risk carries `data-response="partial"`
- **THEN** the document is invalid, because how far a mitigation reaches is stated in its paragraph and not in the response

#### Scenario: A response on another unit
- **WHEN** a decision carries `data-response="accepted"`
- **THEN** the attribute projects nowhere, and the decision means what it would mean without it
