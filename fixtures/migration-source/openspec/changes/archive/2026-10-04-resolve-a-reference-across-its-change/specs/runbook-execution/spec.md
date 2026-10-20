## MODIFIED Requirements

### Requirement: A record carries the procedure it ran and the obligations it attests to

A run's record SHALL carry the runbook's procedure, copied, with each step's state advanced as the
run advanced it. A record that referenced the procedure instead of carrying it could not say which
version of the procedure ran, which is the question the record exists to answer: a procedure
improves, and a record naming one that has since changed describes a run that did not happen that
way.

A run's record SHALL carry the requirements it attests to, copied from the runbook's specification,
each with the scenarios that make it checkable. **This is forced rather than chosen.** A reference
from a runbook's documents addresses a unit within the document that makes it: document-schema's
"Cross-references address units by identifier" lets a reference reach beyond its own document only
from a change's documents or an accepted specification, and a runbook is neither. So a record naming
a requirement in the runbook's specification resolves to nothing and is reported as an error — a
record that pointed outward would be invalid on the day it was written.

Each copied requirement SHALL state its obligation in a form that carries an obligation, and SHALL
declare itself unchanged. Both follow from the copy being a faithful one: a requirement whose modal
is weakened states a preference, and a requirement declared as being added claims an obligation
that is not there yet.

A run's record SHALL record, for each requirement it carries, an outcome and how that outcome was
reached, precisely enough that someone else could repeat it. The outcome and the method are the
recorded values, and a record stating an outcome with no method is a claim nobody can check.

The copy SHALL be whole at the moment the run starts rather than grown as the run proceeds. A
partial copy presents itself as a complete procedure — nothing in a list of steps says there are
more — so a reader between two steps would see a version of the procedure that never existed.

A run's record SHALL be of the artifact kind that carries work, and the set of kinds SHALL NOT gain
one for a run. A record is a copy of a procedure with its states advanced, which is what that kind
already is; and the kind that carries obligations refuses a step outright, so a record declared as
one would be invalid for every step it carried.

#### Scenario: A record naming a requirement it does not carry
- **WHEN** a run's record attests to a requirement that lives only in the runbook's specification
- **THEN** the reference resolves to nothing and is reported as an error, which is why the requirement is copied

#### Scenario: A record of a procedure that has since changed
- **GIVEN** a runbook edited after a run was recorded
- **WHEN** the record is read
- **THEN** it states the procedure as it was when it ran, because the record carries the steps rather than pointing at them

#### Scenario: A record declaring its copied requirement as added
- **WHEN** a run's record declares a copied requirement as being added by a change
- **THEN** its verification is reported as attesting to an obligation that has moved, because the obligation existed before the run

#### Scenario: An outcome with no method
- **WHEN** a run records an outcome and states nothing about how it was reached
- **THEN** the record states a claim nobody can repeat, which is what the method exists to prevent

#### Scenario: A record grown as the run proceeds
- **WHEN** a record is written with only the steps the run has reached
- **THEN** it presents a procedure that never existed, so the copy is whole from the start

