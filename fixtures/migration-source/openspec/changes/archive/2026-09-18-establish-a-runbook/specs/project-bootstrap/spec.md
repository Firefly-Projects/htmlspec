## MODIFIED Requirements

### Requirement: A project has a declared root holding one asset store

An HTMLSpec project SHALL be identified by a directory containing an `htmlspec/` directory. That directory SHALL hold the project's accepted specifications, its active changes, its archive, and **its runbooks**, each in its own place.

**The runbooks are a fourth place rather than a quiet extension of the first three.** A procedure that is run repeatedly is not a change and does not archive, so it cannot live in `changes/` without being destroyed by the operation that ends a change. `runbook-authority` states what a runbook is and what it holds; this requirement states only that the project's root has a place for one.

**This sentence is not declared closed, and the difference from the artifact-kind vocabulary is the argument for saying so.** The set of kinds states that it is `CLOSED` and carries a guard test whose failure message calls adding one "a decision with a compatibility cost". This sentence has neither, and `project-migration` already established that the layout beneath the project root is only partly specified. So a fifth place is a modification with a stated reason rather than a closed set being reopened, and a later change adding one owes the same reason rather than a new argument about whether it may.

A project SHALL hold exactly one asset store, in its root, and every document in the project SHALL reference that store rather than a store of its own. A project storing one copy of its runtime is the whole point of the duplicate-detection rule the contract already states; a tool that gave each change its own copy would be reporting a defect it created itself.

**A runbook's documents are documents of the project and are covered by that sentence without exception.** A runbook is a directory beneath the project root holding documents, which is the same shape a change is, and a store of its own there would be the measured 6-MB-per-change defect arriving at a new location.

#### Scenario: Bootstrapping an empty repository
- **WHEN** a contributor bootstraps a project in a directory with no project in it
- **THEN** the project root is created with places for accepted specifications, active changes, the archive, and runbooks, and one asset store is established in it

#### Scenario: Two changes share one store
- **GIVEN** a bootstrapped project
- **WHEN** two changes are created in it
- **THEN** the project holds one copy of the runtime assets, and each change's documents reference it

#### Scenario: A runbook shares the same store
- **GIVEN** a bootstrapped project holding a change and a runbook
- **WHEN** the project's asset topology is checked
- **THEN** one store is found, because a runbook references the project's store rather than holding one

#### Scenario: Documents reference the store without a network
- **WHEN** a document created in a bootstrapped project is opened from the filesystem
- **THEN** it renders with no network access, because its asset references resolve to the project's store

### Requirement: Bootstrapping is repeatable without loss

The bootstrap operation SHALL be safe to run on a project it has already established. On such a run it SHALL refresh the instructions for the selected tools, and SHALL leave the project's accepted specifications, active changes, archive, **and runbooks** unchanged.

**A runbook is named in that list rather than left to be inferred from it.** A runbook is content a contributor wrote, so the reason the first three are protected applies to it unchanged, and a list that named three of four places would leave the fourth's protection resting on a reading.

The operation SHALL NOT overwrite a project's configuration once it exists. The operation SHALL NOT delete a file it did not write.

Where the asset store is already present, the operation SHALL verify it against the pinned runtime and report a discrepancy rather than silently replacing it. Bytes that differ from the pin are a fact the contributor needs, and overwriting them would hide it.

#### Scenario: Re-running on an established project
- **WHEN** the bootstrap operation runs on a project it already established
- **THEN** the instructions for the selected tools are refreshed and the project's specifications, changes, archive, and runbooks are unchanged

#### Scenario: A runbook survives a repeated bootstrap
- **GIVEN** a project holding a runbook a contributor wrote
- **WHEN** the bootstrap operation runs again
- **THEN** the runbook's documents are as they were

#### Scenario: Existing configuration survives
- **GIVEN** a project whose configuration has been edited
- **WHEN** the bootstrap operation runs again
- **THEN** the configuration is left as it was

#### Scenario: An existing store is verified rather than replaced
- **GIVEN** a project whose stored assets differ from the pinned runtime
- **WHEN** the bootstrap operation runs again
- **THEN** the discrepancy is reported and the stored bytes are not silently replaced

#### Scenario: Nothing unowned is removed
- **WHEN** the bootstrap operation runs on a project containing instructions for tools it does not manage
- **THEN** those files are left in place
