# Spec Delta

## Purpose

Lets an operator carry a runbook to another machine and establish its prerequisites before
starting a run, while keeping dependencies external and the procedure's release identifiable.

## ADDED Requirements

### Requirement: Machine requirements are explicit and mechanically checkable

A runbook SHALL state machine prerequisites as ordinary requirement units in its specification,
with stable identifiers, normative statements, scenarios, and an explicit check declaration.
Preflight SHALL read these declarations from the existing projected requirement notes. It SHALL
NOT infer checks from prose, requirement names, or executable examples. The declaration format
SHALL be documented and restricted to built-in checks; arbitrary commands SHALL NOT be accepted.

The supported needs SHALL include a tool at a stated version, a tool supplied by a pinned
package-manager environment, an installed agent skill, and a working-directory state. Machine
requirements SHALL remain part of the logging contract: passing preflight does not remove their
obligation to have an outcome recorded. Ordinary requirements without a check declaration SHALL
remain logging obligations without becoming machine checks.

Malformed declarations, unknown check kinds, duplicate declarations on one requirement, and
unsupported format versions SHALL be reported against the requirement and SHALL prevent starting
a run. A contract with no machine declarations SHALL report zero checks, without claiming to
have established unstated prerequisites.

#### Scenario: A missing tool
- **WHEN** a declared tool cannot be found on the invocation's executable search path
- **THEN** preflight identifies the requirement and missing tool and refuses readiness

#### Scenario: A version mismatch
- **WHEN** the selected tool reports a different version from the declared one
- **THEN** preflight reports the expected and observed versions and refuses readiness

#### Scenario: A missing skill
- **WHEN** the declared skill file is absent from the declared local installation scope
- **THEN** preflight names that scope and skill and refuses readiness without installing it

#### Scenario: The bootstrap hazard
- **GIVEN** a requirement that the working directory must be outside an existing HTMLSpec project
- **WHEN** the working directory resolves to a project
- **THEN** preflight refuses readiness before bootstrap or any procedure step occurs

#### Scenario: A declaration cannot be interpreted
- **WHEN** a machine-check declaration is malformed, duplicated, or unsupported
- **THEN** preflight reports an invalid declaration rather than omitting it or treating it as passed

#### Scenario: A requirement about a later outcome
- **WHEN** an ordinary logging requirement has no machine-check declaration
- **THEN** preflight does not attempt to check that outcome and the run still owes its verification

### Requirement: Package manifests retain authority over package pins

A requirement for a package-managed toolchain SHALL identify the existing manifest, selected
environment, and tools needed from it rather than repeat the manifest's versions in the runbook.
Preflight SHALL delegate interpretation of pins to that package manager's supported read-only
adapter and check the tools the invocation would actually use against that environment.

The initial supported package-manager adapter SHALL cover a local locked Nix flake's development
shell. Preflight SHALL NOT install, build, download, update locks, enter a shell, or alter the
working environment. Missing cached dependencies, an unsupported manifest, an unavailable manager,
or an inability to establish a tool's provenance SHALL refuse readiness with an explanation.
The existence of a manifest or a shell marker alone SHALL NOT count as evidence of readiness.

#### Scenario: The wrong executable shadows a pinned tool
- **GIVEN** a locally available pinned environment supplying the required tool
- **WHEN** the invocation's search path resolves that tool to another installation
- **THEN** preflight reports the differing provenance and refuses readiness

#### Scenario: A pin changes in the manifest
- **WHEN** the manifest selects a different toolchain
- **THEN** preflight checks that toolchain without requiring a duplicate version edit in the runbook

#### Scenario: Resolving the environment would require provisioning
- **WHEN** the selected pinned environment cannot be inspected using locally available data
- **THEN** preflight reports it as unavailable and performs no provisioning or lock update

### Requirement: Preflight gates the creation of a run

Starting a run SHALL perform preflight against the selected runbook and the invocation's current
working directory and environment before creating the record or its directory. A standalone
preflight success SHALL NOT be reusable as permission to skip this check. Only a result in which
every declared machine requirement passes SHALL permit record creation; failed, unknown, timed-out,
and unsupported results SHALL refuse it. There SHALL be no force or skip-preflight option.

Preflight SHALL execute no procedure steps and SHALL not modify the runbook, existing records,
skills, or target working tree. Checks SHALL have bounded execution and captured output. An
interrupted preflight SHALL stop its probes and publish no record. Each retry SHALL collect fresh
observations. Readiness describes observations at the beginning of a run, not a guarantee about
the environment throughout the procedure.

A successfully created record SHALL carry the runbook version and a verification of each passed
machine requirement, with its observation and method. Other logging requirements SHALL remain
unverified until the run records their outcomes. Checking or editing an existing record SHALL NOT
become subject to preflight; run conformance remains advisory as established by `log-a-runbook-run`.

#### Scenario: A prerequisite fails
- **WHEN** starting a run finds an unsatisfied machine requirement
- **THEN** no record or new records directory is created and the canonical procedure is unchanged

#### Scenario: The environment changes after a standalone check
- **GIVEN** a standalone preflight passed before a required tool was removed
- **WHEN** the operator starts a run
- **THEN** the fresh preflight fails and no record is created

#### Scenario: Starting after all prerequisites pass
- **WHEN** preflight passes and the composed record validates
- **THEN** the record includes the copied procedure, the carried version, passed machine-check evidence, and unverified remaining logging obligations

#### Scenario: A check never finishes
- **WHEN** a probe exceeds its execution bound
- **THEN** it is stopped, its timeout is reported, and no run is started

#### Scenario: Reviewing a historical run on an unprepared machine
- **WHEN** an operator checks an existing run's logging conformance
- **THEN** missing current machine prerequisites do not prevent that review or promote its findings to errors

### Requirement: A carried runbook does not require its source project's history

A carried runbook SHALL be usable with its specification, procedure, version, changelog, and the
referenced shared asset store in a minimal receiving project layout. It SHALL NOT require the
source project's archive, project declaration, accepted specifications, or old run records to
perform preflight or start a new local run. The receiving layout SHALL preserve one asset store
shared by the runbooks and their records; it SHALL NOT introduce a store inside each runbook.

The runbook SHALL require tools and skills from the receiving machine rather than vendor them.
Its dependency manifests SHALL be supplied by the target work environment when required. Old run
records SHALL remain at their source unless separately copied; their absence at the destination
SHALL NOT imply they never existed or prevent a new run. Transport and history synchronization
SHALL remain outside the preflight operation.

#### Scenario: The source project is absent
- **GIVEN** a copied runbook and its shared assets in a minimal receiving layout
- **WHEN** the operator selects it and satisfies its declared machine requirements
- **THEN** preflight and run creation work without reading any source-project file

#### Scenario: A carried runbook has no prior records
- **WHEN** its first local run starts
- **THEN** a new local record is created and no source history is fabricated or required

#### Scenario: Offline viewing on the receiving machine
- **WHEN** the copied procedure and a newly created record are opened without network access
- **THEN** both use the receiving project's one copied asset store
