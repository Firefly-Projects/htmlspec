## Purpose

Defines what a runbook is, where a project keeps one, which of its documents carries authority,
and what a runbook deliberately does not carry. It exists because the same procedural lesson was
learned at `0.10.0` and rediscovered at `0.12.0`: a procedure that is run repeatedly is not a
change and does not archive, so a lesson recorded in an archived `tasks.md` binds nothing and is
read by nobody, and the generated instruction body that would be the other candidate home is
overwritten on the next bootstrap.

## Requirements

### Requirement: A project keeps its procedures in a fourth place

A project SHALL have a place for its runbooks, beneath the project root, distinct from its
accepted specifications, its active changes, and its archive. A runbook SHALL be identified by a
directory in that place, and the directory's name SHALL be the runbook's name.

A procedure that is run repeatedly SHALL NOT be represented as a change. A change is completed
and archived once; a procedure is run again, so a representation that archives it after its first
run destroys the thing being kept.

The place SHALL be created when a project is bootstrapped, rather than when a project's first
runbook is created. A location that does not exist until something is in it is not a location a
contributor can find, and the bootstrap operation is already required to produce the project's
places rather than to acquire them on use.

The runbook's name SHALL NOT be recorded anywhere but in its directory name, and nothing SHALL
derive an ordering, a version, or a history from that name. A runbook is renamed by moving its
directory, exactly as a change is. This is narrower than the prohibition on deriving a version
from a directory name and it is not in tension with it: a version is a value a gate compares, so
reading one out of a display string depends on a convention nothing enforces, while a name is how
a thing is addressed rather than something compared to anything.

A runbook's documents SHALL reference the project's one asset store, as every other document in
the project does. A runbook SHALL NOT hold a store of its own.

#### Scenario: A bootstrapped project has the place
- **WHEN** a contributor bootstraps a project
- **THEN** the place for its runbooks exists alongside the places for its specifications, its changes, and its archive

#### Scenario: A procedure that is run again
- **GIVEN** a procedure that has been run once
- **WHEN** it is run a second time
- **THEN** the procedure is still where it was, because a runbook is not archived after a run

#### Scenario: A runbook renamed
- **WHEN** a runbook's directory is moved to another name
- **THEN** the runbook is the one at the new name, because nothing records the old one

#### Scenario: A runbook's documents share the project's store
- **WHEN** a runbook's document is opened from the filesystem
- **THEN** it renders with no network access, because its asset references resolve to the project's one store

### Requirement: A runbook is two documents of existing kinds

A runbook SHALL be a directory holding a `specification` document and a `tasks` document. The
`specification` SHALL state what the runbook is for and what its history taught. The `tasks`
document SHALL carry the ordered procedure, one task unit per step.

The set of artifact kinds SHALL NOT gain a kind for a runbook, and SHALL remain the closed four.
A kind answers which stage of one change's argument a document is; a runbook is not a stage of an
argument, it is a destination. The precedent is already in the project: an accepted
specification is a top-level artifact outside any change directory whose documents are of kind
`specification`, the same kind a change produces — so a document outside a change directory that
changes flow into is a shape that needed no kind of its own.

**The split into two documents SHALL NOT be presented as a filing convention.** A specification
SHALL NOT carry task units, which is the only placement rule in the artifact-kind vocabulary and
an error rather than a warning, so the procedure cannot live inside the canonical document. The
split is forced by a rule that already exists.

A runbook's documents SHALL be validated by whatever validates a document of their kind, and no
rule SHALL exist whose subject is that a document belongs to a runbook. A runbook's `tasks`
document with no task unit SHALL be reported, because a procedure with no steps is not a
procedure. A runbook's `specification` carrying a task unit SHALL be reported as an error,
because the procedure drifting into the canonical document is the failure the split prevents.

#### Scenario: A runbook's documents declare existing kinds
- **WHEN** a runbook's two documents are read
- **THEN** one declares the specification kind and the other declares the tasks kind, and neither declares a kind outside the closed four

#### Scenario: A procedure in the canonical document
- **WHEN** a runbook's specification carries a task unit
- **THEN** validation reports an error, because a specification records obligations rather than steps

#### Scenario: A runbook with no steps
- **WHEN** a runbook's tasks document carries no task unit
- **THEN** validation reports it, because the document enumerates no procedure

#### Scenario: A runbook validated by the ordinary path
- **WHEN** a runbook's documents are validated
- **THEN** they are checked as documents of their kinds, with no rule reading the fact that they belong to a runbook

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

### Requirement: The project enumerates the runbooks it holds

The operation that lists what a project holds SHALL be able to list its runbooks, as a selection
alongside its changes and its accepted specifications. A separate top-level command SHALL NOT be
introduced for it, because listing what the project holds is an operation that already exists and
a second command would invent a second vocabulary for it.

The enumeration SHALL be derived from what the place holds rather than from a list a contributor
maintains. A maintained list is the shape this project refuses when it forbids a gate from
reading the last archived version out of a directory name: a value kept beside the facts it
summarizes depends on a convention nothing enforces.

Each entry SHALL carry what a selector needs in order to choose between runbooks, rather than a
name alone. A runbook whose procedure is absent is the case that matters, because a name alone
cannot distinguish it from one that is complete.

The enumeration SHALL name a runbook once, whatever number of documents it holds. An enumeration
that listed each document separately would present one runbook as two entries.

#### Scenario: Listing the project's runbooks
- **WHEN** a contributor lists the project's runbooks
- **THEN** each runbook the place holds appears once, with what it holds

#### Scenario: A runbook whose procedure is absent
- **WHEN** a runbook holds no tasks document
- **THEN** the enumeration shows that it holds none, so a selector is not left to open it to find out

#### Scenario: A project with no runbooks
- **WHEN** a contributor lists the runbooks of a project holding none
- **THEN** the operation succeeds and reports that there are none

#### Scenario: An enumeration nobody maintains
- **GIVEN** a runbook added to the project by creating its directory
- **WHEN** the runbooks are enumerated
- **THEN** it appears, because the enumeration reads the place rather than a list

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

### Requirement: A runbook carries an independently checked release version

Every runbook SHALL carry a `VERSION` containing three nonnegative dot-separated integers and a
`CHANGELOG.md` whose headings identify its released versions. A release version SHALL be greater
than `0.0.0`. The version SHALL identify the runbook's release independently of the project's
version; it SHALL NOT be inferred from a directory name or an HTML schema-version declaration.

Version ordering and content changes SHALL be checked at source publication as specified by
`project-bootstrap`. A version SHALL never identify two different released procedures or contracts
for the same runbook. Distinctness SHALL be per runbook, so two different runbooks can each be at
`0.1.0`. Changelog prose SHALL remain the author's, with one heading per runbook release and a
reference to the source project release when published through a project archive.

Preflight and new run creation SHALL refuse a missing or malformed version or missing heading for
the declared version. They SHALL NOT require access to the source archive to compare versions.
A copied version identifies a release; it does not attest that the copy is authentic. Existing
records created before this obligation SHALL remain reviewable without being rewritten.

#### Scenario: Identifying a copy without the archive
- **WHEN** the operator checks a carried runbook's version
- **THEN** the version and matching changelog entry identify its release without consulting the source project

#### Scenario: Two independently released runbooks
- **WHEN** two different runbooks both declare `0.1.0`
- **THEN** neither is rejected for sharing a version number with the other

#### Scenario: Unversioned runbook after upgrading
- **WHEN** the operator tries to start a runbook without its version declaration
- **THEN** the operation refuses with migration guidance and writes no record

#### Scenario: Reviewing a record made before versioning
- **WHEN** an older run record is reviewed
- **THEN** the absence of a recorded runbook version does not prevent conformance reporting

### Requirement: A machine requirement is still an obligation of the run

A runbook's specification SHALL state machine prerequisites beside its hazards and logging
requirements, using the same requirement units. The check declaration described by
`runbook-portability` SHALL select when and how a machine condition is checked; it SHALL NOT create
a new unit kind, a new document kind, or an exemption from recording the requirement's outcome.

Ordinary document validation SHALL continue to validate the document's kind and schema without
inspecting the host environment. Machine inspection SHALL occur only through preflight and new
run creation. A failed preflight SHALL be a refusal to start; incomplete logging after a run
starts SHALL remain an advisory conformance finding.

#### Scenario: Checking a runbook on a documentation machine
- **WHEN** its documents are validated where a required tool is not installed
- **THEN** document validation does not fail because of that machine's environment

#### Scenario: Preflight evidence joins the log
- **WHEN** a new run starts after its machine checks pass
- **THEN** the record carries those requirements and their passed verifications along with its other logging obligations
