## MODIFIED Requirements

### Requirement: A runbook records what its history taught

A runbook's `specification` SHALL state the hazards its procedure has met, as requirements rather
than as remarks. A hazard stated as a remark in a procedure's steps is what the project already
had: an archived tasks document, which nothing reads and which binds nothing.

Where the order of a procedure's steps is constrained by a hazard rather than by convenience,
that constraint SHALL be stated with the hazard that forces it. An order whose reason is unstated
is an order the next person reverses.

A runbook SHALL NOT be treated as stating an obligation of the project. A runbook is a procedure
the project keeps, and the project's contract is what its accepted specifications state. A
requirement in a runbook's specification states what that runbook requires of a run, not what the
project requires of itself.

**A runbook's `specification` SHALL state what its runs must record**, as requirement units of the
same kind the hazards are stated as. This is the runbook's logging contract, and it is stated where
the runbook is authored because the person who knows a cookie run needs an oven temperature is the
person writing the recipe. There is no single thing a run should record: a release run records the
version it advanced to and whether the release check passed, and neither value means anything to a
baker.

**A log requirement SHALL NOT be distinguished in the markup from a hazard requirement**, and the
consequence is that every requirement a runbook's specification states is one its runs must record.
That reading is the correct one rather than a simplification, because the sentence above already
covers both: a hazard the procedure met and a value a run must record are each *what this runbook
requires of a run*. Distinguishing them would need an attribute or a unit the artifact vocabulary
does not have, and the vocabulary is closed.

The honest cost of that, stated rather than discovered by an author: a runbook whose specification
states ten hazards obliges every run to record ten outcomes, and an author who wanted five
obligations and five notes has no way to say so. What is available is to state fewer requirements
and put the rest in the prose the specification already carries.

**A runbook that states no log requirement SHALL NOT be reported.** A procedure whose runs record
nothing is a procedure nothing checks, which is a fact about that runbook rather than a fault in it
— and a report firing on every runbook that has not yet stated a contract is one an author meets
before they have anything to say.

#### Scenario: A hazard the procedure has met
- **WHEN** a runbook's history has taught that a step has a precondition or a consequence
- **THEN** the runbook's specification states it as a requirement, rather than as a note beside the step

#### Scenario: An order a hazard forces
- **WHEN** two steps must run in one order for a reason that is not obvious from the steps
- **THEN** the reason is stated with the hazard, so the order is not reversed by someone reading only the steps

#### Scenario: A runbook cited as a project obligation
- **WHEN** an agent cites a requirement in a runbook's specification as an obligation of the project
- **THEN** the citation is incorrect, because the project's contract is what its accepted specifications state

#### Scenario: A runbook stating what its runs record
- **WHEN** a runbook is authored for a procedure whose runs produce values worth keeping
- **THEN** its specification states each of those values as a requirement, so a run's record has something to attest to

#### Scenario: A runbook that states nothing about its runs
- **GIVEN** a runbook whose specification states no requirement
- **WHEN** its runs are examined for conformance
- **THEN** nothing is reported, because a runbook that states no contract has runs nothing checks

### Requirement: The canonical procedure carries no completion state

A runbook's `tasks` document SHALL carry its steps in the state they have before anyone works
them, and that state SHALL NOT advance as a consequence of the runbook being run. The states
advance in the copy a run makes, exactly as they advance in a change.

Nothing SHALL report a runbook whose steps are all unstarted. That is the correct state of a
canonical procedure at every moment, so a report of it would fire on every runbook always, which
teaches a reader to ignore the report.

An enumeration of a project's runbooks SHALL NOT present a completion fraction for the canonical
procedure. The fraction is derivable and it is the same value forever, and a column that never
moves is read once and then not read.

**A unit in a runbook's canonical documents SHALL declare that it is unchanged.** A change state
describes a unit's relationship to a change in progress, and a runbook has none: its procedure
exists and the hazards it records were measured before the document was written. This is the same
fact as the preceding paragraphs rather than a second rule, arriving at a different attribute.

**A unit in the record of a run SHALL declare that it is unchanged for the same reason**, and this
is that sentence reaching the one other document a runbook holds rather than a new obligation. A
run's record carries the requirements it verifies, copied from the runbook, and a verification
recording an outcome against a requirement declared `added` is reported — correctly — as attesting
to an obligation that is not there yet. The obligation a run attests to existed before the run
started, so the copy declares itself unchanged.

The consequence is what makes it worth stating. A verification that records an outcome against a
requirement declared as added is reported, correctly, as attesting to an obligation that is not
there yet — so a runbook whose units are declared the way a change's units are declared reports
warnings whose cause is one attribute, and nothing states which. The operation that creates a
document SHALL NOT be expected to know the difference, because it cannot: whether a document
belongs to a change or to a runbook is not a fact about the document being written.

**The operation that starts a run is the exception, and it SHALL declare the state correctly.** It
knows what it is writing — a copy of an existing procedure, attesting to obligations that already
exist — so the reason the general document-creating operation cannot know does not apply to it.
This narrows the preceding paragraph rather than contradicting it: an author writing a run record
by hand still meets the warnings, and so does an author writing a runbook by hand.

#### Scenario: A runbook that has been run
- **GIVEN** a runbook whose procedure a person has carried out
- **WHEN** the runbook's tasks document is read
- **THEN** its steps are unstarted, because the run advanced a copy

#### Scenario: An enumeration of runbooks
- **WHEN** a contributor enumerates the project's runbooks
- **THEN** no completion fraction is presented for a canonical procedure

#### Scenario: A verification in a runbook's specification
- **GIVEN** a runbook whose specification records a hazard measured before the runbook was written
- **WHEN** a verification in it records that the hazard was checked
- **THEN** nothing reports the record as attesting to an obligation that has moved, because the requirement is declared unchanged

#### Scenario: A runbook's documents created from a template
- **WHEN** a runbook's documents are created by the operation that creates a document
- **THEN** the declared change states are the author's to settle, because the operation cannot know that the document belongs to a runbook

#### Scenario: A run's record created by the operation that starts a run
- **WHEN** the operation that starts a run writes the record
- **THEN** every unit in it declares that it is unchanged, because that operation does know what it is writing
