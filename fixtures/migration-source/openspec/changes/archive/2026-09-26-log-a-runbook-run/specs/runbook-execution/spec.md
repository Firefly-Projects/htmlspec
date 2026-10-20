## ADDED Requirements

### Requirement: A run leaves a record where the runbook keeps them

A run of a runbook SHALL produce a record, and that record SHALL be a document beneath the
runbook's own directory, in a place the runbook keeps for its records. A run that leaves no record
SHALL NOT be treated as having happened, because the question a runbook exists to answer — which
version of this procedure ran, and what did it produce — has no other source.

The place SHALL be derived from the runbook rather than composed by whatever operation writes into
it, exactly as the runbook's own location is derived from the project. A location composed by a
caller is a second statement of where a thing lives, and two statements agree until the day the
layout moves.

The place SHALL be created by the operation that writes the first record rather than when the
project is bootstrapped or when the runbook is created. This is narrower than the obligation on the
project's own places and it is not in tension with it: the project's places are the four a
contributor looks for and one of them cannot exist before its runbook does.

A run's record SHALL reference the project's one asset store, as every other document in the
project does.

A run's record SHALL NOT be resolvable as a change, and the operations that take a change SHALL
refuse it. A record archived is a run's history destroyed, which is the same failure archiving a
runbook is, one directory further down.

#### Scenario: A run that has happened
- **WHEN** a runbook's procedure has been carried out
- **THEN** a record of that run exists beneath the runbook, because a run that left nothing is a run nothing can be asked about

#### Scenario: The place for records before the first run
- **GIVEN** a runbook nobody has run
- **WHEN** the project is examined
- **THEN** the runbook has no place for records yet, and that is not reported, because nothing is missing

#### Scenario: A record offered where a change is expected
- **WHEN** an operation that takes a change is pointed at a run's record or at the place holding them
- **THEN** it refuses, because archival moves a directory and a run's history is not a change's

#### Scenario: A record's assets
- **WHEN** a run's record is opened from the filesystem
- **THEN** it renders with no network access, because its asset references resolve to the project's one store

### Requirement: A record carries the procedure it ran and the obligations it attests to

A run's record SHALL carry the runbook's procedure, copied, with each step's state advanced as the
run advanced it. A record that referenced the procedure instead of carrying it could not say which
version of the procedure ran, which is the question the record exists to answer: a procedure
improves, and a record naming one that has since changed describes a run that did not happen that
way.

A run's record SHALL carry the requirements it attests to, copied from the runbook's specification,
each with the scenarios that make it checkable. **This is forced rather than chosen.** A reference
addresses a unit within one document, so a record naming a requirement in the runbook's
specification resolves to nothing and is reported as an error — a record that pointed outward would
be invalid on the day it was written.

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

### Requirement: A run is one record from start to finish

A run interrupted and continued SHALL be one record rather than two, and a run's record SHALL
therefore be editable after it is written. A procedure abandoned at step six and picked up an hour
later is one piece of work, and two records would each be wrong about it: the first would state six
steps done and four unstarted, and the second could state neither for the six it did not do.

A record SHALL be able to state that a step is under way. The vocabulary for a step's state
includes that value, and a record written once and never touched could never carry it honestly —
which would make a state in the vocabulary unreachable for the one document whose steps actually
advance.

The operation that starts a run SHALL NOT overwrite an existing record. A run continued is that
record edited; a run started is a new one. An operation that wrote over a record would destroy the
history the record exists to hold.

**A run's record SHALL NOT be required to state when the run happened.** A recorded time nothing
compares is accuracy without authority, which is the same objection that keeps a runbook from
declaring a version of its own. The consequence, stated rather than left to be found: two runs a
week apart are indistinguishable from one run interrupted for a week, and the question of when a
run happened is settled where a runbook's history is settled.

#### Scenario: A procedure abandoned and continued
- **GIVEN** a run that stopped partway through
- **WHEN** the same work is continued later
- **THEN** the existing record is advanced, because splitting one piece of work across two records makes both of them wrong about it

#### Scenario: A step under way
- **WHEN** a run's record states that a step is in progress
- **THEN** that is a legitimate state of a record rather than an incomplete one, because a run is resumed

#### Scenario: A run started over an existing record
- **WHEN** the operation that starts a run is given the name of a record that exists
- **THEN** it refuses, because overwriting a record destroys a run's history

### Requirement: A run's conformance to its runbook's contract is reported and never gated

A run whose record does not attest to every requirement the runbook states SHALL be reported, and
the report SHALL name both the run and each requirement nothing in it attests to. A report naming
only the run leaves a reader to find which requirement, and a report naming only the requirement
leaves them to find which run.

**The report SHALL NOT fail validation and SHALL NOT be promotable to a failure.** A run under way
legitimately has requirements nothing has verified yet, because a run is one record from start to
finish, so a check that failed the run halfway through would gate the work that produces the very
records it asks for. This is the same policy that keeps a specification in progress from failing:
the report marks work for attention rather than blocking it.

**The subject of the report SHALL be a runbook and its runs together**, so the check SHALL be
evaluated where both are in view rather than as part of validating one document. Neither document
alone can decide it: the specification states the requirements and the record states the outcomes.
A check that ran over each document would report one violation once per document, and none of it
would be a fault of the document being validated.

The report SHALL NOT be produced for:

- a runbook nobody has run. A procedure with no runs violates nothing, and a report firing from the
  moment a runbook is created is one a reader learns to ignore.
- a run whose recorded outcome states that the check did not pass. **The obligation is that the run
  recorded an outcome, not that the outcome was good.** A batch that came out wrong is a bad batch
  and a correct record of one, and a report that treated a recorded failure as a document fault
  would teach an operator to record a pass.
- a run attesting to something the runbook does not require. A run that recorded more than it owed
  is not at fault.

#### Scenario: A run missing a verification
- **GIVEN** a runbook stating two requirements of its runs
- **WHEN** a run's record attests to one of them
- **THEN** the other is reported, naming the run and the requirement, and validation still succeeds

#### Scenario: A run under way
- **GIVEN** a run whose record attests to nothing yet
- **WHEN** conformance is checked under the strictest setting available
- **THEN** the report is made and nothing fails, because a run in progress is not a defect

#### Scenario: A runbook nobody has run
- **WHEN** conformance is checked for a runbook with no records
- **THEN** nothing is reported

#### Scenario: A run that recorded a failure
- **GIVEN** a run whose record states that a requirement's check did not pass
- **WHEN** conformance is checked
- **THEN** nothing is reported about that requirement, because the run recorded the outcome it had

#### Scenario: A check evaluated over one document
- **WHEN** a run's record is validated on its own
- **THEN** no conformance report is produced, because the check's subject is the runbook and its runs and neither document holds both halves

### Requirement: A run's record is history rather than contract

A run's record SHALL NOT be treated as stating an obligation, of the project or of the runbook. It
records what one run did. The requirements it carries are copies of the runbook's, and the
runbook's are what that runbook requires of a run rather than what the project requires of itself.

A run's record SHALL NOT be compared against the project's accepted specifications, and the
operation that performs that comparison SHALL refuse one. A record's copied requirements belong in
no accepted specification, so a report directing an agent to merge them would put obligations
nobody wrote into the contract by the hand of an agent doing as it was told.

A count of a runbook's runs SHALL be derived from the records rather than stored. A maintained
count is a value kept beside the facts it summarizes, which depends on a convention nothing
enforces.

#### Scenario: A record's requirement cited as an obligation
- **WHEN** an agent cites a requirement in a run's record as an obligation of the project or of the runbook
- **THEN** the citation is incorrect, because the record states what one run did

#### Scenario: A record offered to the comparison
- **WHEN** the operation comparing a change against the accepted specifications is pointed at a run's record
- **THEN** it refuses, because the record's requirements belong in no accepted specification

#### Scenario: How many times a runbook has run
- **WHEN** a reader asks how many times a runbook has been run
- **THEN** the answer is derived from the records the runbook holds, rather than read from a count anybody maintained
