## MODIFIED Requirements

### Requirement: Project checks are runnable as one command

The project SHALL expose a single command that runs its full check suite — the conformance fixtures, the unit tests, the formatting check, and the project's release hygiene — so that contributors and continuous integration verify the same thing.

Every stage of the suite SHALL read the working tree rather than a repository's history. The build sandbox the suite runs in receives a filtered source snapshot with no repository, so a stage using version control history would pass on a contributor's machine and fail in continuous integration — the divergence a single shared suite exists to prevent. This SHALL constrain every stage the suite gains, not only the ones it has.

Every file a stage reads SHALL be present in that snapshot, which requires both that the source filter admit it and that version control track it. An untracked file and a file the filter excludes are indistinguishable from an absent one inside the sandbox, and both produce a stage that passes locally and fails in continuous integration. A stage that reads a file whose name the filter does not already admit SHALL be accompanied by the filter change that admits it, in the same change.

A stage SHALL either pass or fail, because the suite has no advisory level. A rule whose violation is a legitimate state for work in progress SHALL therefore be stated as a property of a settled record rather than of work under way, so that the rule has a correct answer at every moment. A stage that would report on unfinished work has no way to do so without failing, and a stage that fails throughout a change's life is one contributors learn to disregard.

A stage's name SHALL be distinct from every other stage's, so that a reported failure names one stage unambiguously.

#### Scenario: Full check suite
- **WHEN** the check command runs on a healthy revision
- **THEN** conformance fixtures, tests, the formatting check, and the release check all pass and the command exits zero

#### Scenario: Conformance regression
- **WHEN** a code change causes a valid conformance fixture to be rejected or an invalid one to be accepted
- **THEN** the check command fails and identifies the fixture and the expected rule

#### Scenario: Continuous integration parity
- **WHEN** continuous integration runs
- **THEN** it invokes the same check command a contributor would run locally, rather than a separately maintained list of steps

#### Scenario: A stage that would need history
- **WHEN** a stage would answer its question by reading commits or diffs
- **THEN** it is stated against the working tree instead, because the sandbox has no repository to read

#### Scenario: A rule that cannot fail cleanly on work in progress
- **WHEN** a rule's violation is a legitimate state part-way through a change
- **THEN** the stage asks it of a settled record rather than of the working change, since the suite offers no way to report without failing

#### Scenario: A file the source filter excludes
- **WHEN** a stage reads a file whose name the source filter does not admit
- **THEN** the stage fails in the sandbox while passing locally, so the filter change that admits the file belongs to the same change as the stage

## ADDED Requirements

### Requirement: The project records what changed in each version it ships

The project SHALL keep a changelog with one entry per version it has shipped, and each entry's heading SHALL name its version so that a check can locate it.

An entry's prose SHALL remain the author's. A check SHALL establish that an entry for a version exists and SHALL NOT impose a structure on what the entry says, because a changelog exists to be read by a person.

An entry reconstructed after the fact SHALL say that it is a reconstruction. A history assembled from records by someone who was not present is more useful labelled than unlabelled, and more honest than a confident account.

This obligation is on this repository. It SHALL be distinguishable from any obligation the program places on a project it bootstraps, and satisfying one SHALL NOT be taken to satisfy the other.

#### Scenario: A version with no entry
- **WHEN** the project has shipped a version the changelog does not name
- **THEN** the release check fails and names that version

#### Scenario: An entry's contents
- **WHEN** the release check locates an entry for a version
- **THEN** it reads the entry's presence and applies no rule to the entry's prose

#### Scenario: A reconstructed entry
- **WHEN** an entry is written after the fact from archived records
- **THEN** the entry states that it is a reconstruction

### Requirement: An archived change records the version it shipped in

Each archived change SHALL record the version the project declared when it was archived, in a form the project's own suite can read without version control history.

Recorded versions SHALL be distinct, so that each release names the change that produced it. Where the project has archived several changes under one version before this obligation existed, the changelog SHALL carry a single baseline entry naming them, and the suite SHALL treat that baseline as one release. The baseline SHALL cease to apply once the version advances past it, so it is a statement about a known past rather than a permanent exception.

Versions SHALL NOT be invented to satisfy distinctness. Assigning a separate version to each of several changes that shipped together records releases that never existed, which is worse than recording the shared version honestly.

#### Scenario: An archive with no recorded version
- **WHEN** an archived change records no version
- **THEN** the release check fails and names that change

#### Scenario: Two archives claiming one version
- **WHEN** two archived changes above the baseline record the same version
- **THEN** the release check fails, because a version names the change that produced it

#### Scenario: The baseline
- **WHEN** several changes were archived under one version before this obligation existed
- **THEN** one changelog entry names them all and the suite treats them as one release

#### Scenario: The baseline retires
- **WHEN** the project's version advances past the baseline
- **THEN** every subsequent change is checked for a distinct version of its own
