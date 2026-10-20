# Spec Delta

## ADDED Requirements

### Requirement: Archival checks the versions of runbooks it publishes

In addition to checking the project's declared version, archival SHALL compare each runbook's
declared version and canonical specification and procedure against its most recent published
snapshot. Snapshots SHALL be ordered by recorded project versions, not archive-directory names,
and SHALL carry the runbook version and content fingerprints in a form readable without Git.

For a runbook with no prior snapshot, publication SHALL require a version greater than `0.0.0`.
Changed canonical documents SHALL require a strictly greater runbook version. An unchanged
runbook SHALL be allowed to retain its version or advance it with a matching changelog entry.
A downgrade, reusing a version for different canonical content, or a missing version or changelog
heading SHALL refuse archival. A malformed existing snapshot SHALL be reported, not treated as
an absent predecessor. Adding or editing a run record SHALL NOT require a version increment.

Publication SHALL record snapshots in the same archival transaction as the change and project
version. A refusal, cancellation, or failed publication SHALL leave the runbook, previous archive,
and active change unchanged. Readiness-only archival SHALL apply the same comparison and write
nothing. Archival SHALL NOT choose a version increment or write changelog prose for the author.

Runbook identity for this comparison SHALL be its directory name, as already required by
`runbook-authority`. A renamed runbook SHALL begin a new publication lineage; former snapshots
SHALL remain under the former name. A deleted runbook SHALL not cause deletion of its snapshots.
An absent source archive on a receiving machine SHALL NOT prevent preflight or running a carried
release; these operations do not publish a release.

#### Scenario: A changed procedure without a version increment
- **GIVEN** a published runbook whose procedure has changed
- **WHEN** archival sees the same runbook version as the last snapshot
- **THEN** archival refuses before moving the change or recording any version

#### Scenario: A version advances
- **WHEN** a changed runbook declares a greater version and its changelog names that version
- **THEN** the runbook version gate permits publication and the completed archive records its new snapshot

#### Scenario: Only a run record changes
- **WHEN** a runbook's canonical documents are unchanged and a new run record exists
- **THEN** archival does not require that runbook's version to advance

#### Scenario: The first independent runbook release
- **WHEN** an existing runbook gains its first checked version and has no earlier snapshot
- **THEN** archival requires a version above `0.0.0` and records the adoption without fabricating older releases

#### Scenario: An archive directory is renamed
- **WHEN** archival selects the latest snapshot after an archive directory was renamed
- **THEN** the recorded project versions still determine the same predecessor

#### Scenario: A malformed prior snapshot
- **WHEN** a runbook's previous snapshot cannot be parsed
- **THEN** archival reports the defective record and does not accept the runbook as a first release

#### Scenario: Readiness-only archival
- **WHEN** archival is asked to check a changed unversioned runbook without performing
- **THEN** it reports the same refusal as publication and leaves all files unchanged

### Requirement: The release suite checks published runbook version history

The repository release suite SHALL check that every published runbook snapshot has a parseable
version and fingerprints, that changed canonical content advances its version, and that each
published version has a changelog heading in the published snapshot. Repeated snapshots of an
unchanged runbook at one version SHALL count as the same release. Distinct releases for that
runbook SHALL have increasing versions, and a lower version SHALL be rejected even when content
is unchanged.

The suite SHALL also check that each current runbook has a valid version and corresponding
changelog heading. It SHALL NOT require unarchived edits to have advanced their version yet.
Archives predating adoption of runbook publication snapshots SHALL remain valid without invented
history. Once snapshots are introduced, each later archive SHALL explicitly record its runbook
inventory, including an empty inventory, so a missing inventory cannot masquerade as old history.

#### Scenario: A published version is reused for changed content
- **WHEN** the release suite reads two successive snapshots of one runbook with different content and the same version
- **THEN** it fails and names the runbook and conflicting archives

#### Scenario: Work in progress has not chosen its next version
- **WHEN** canonical documents differ from the last snapshot while their current version still has its changelog heading
- **THEN** release hygiene passes and the archival readiness check reports the pending version increment

#### Scenario: An inventory disappears after adoption
- **WHEN** an archive at or after the first recorded adoption release lacks its runbook inventory
- **THEN** the release suite reports the missing publication record rather than applying the legacy exemption
