# Spec Delta

## REMOVED Requirements

### Requirement: A runbook carries no version and no changelog of its own

**Reason**: A carried runbook must identify its release without the source archive. The source
publication gate now compares versions, giving the declaration an enforcing consumer.

**Migration**: Give each existing runbook a `VERSION` and `CHANGELOG.md` at its first release under
the new contract. Record that adoption honestly; do not invent independent versions for earlier
project releases. Older run records remain readable without a carried version.

## ADDED Requirements

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
