## MODIFIED Requirements

### Requirement: Archival requires the project version to advance

Archival SHALL compare the project's declared version against the last archived version, and SHALL refuse unless the declared version is ordered after it. The accepted contract SHALL therefore carry a version history rather than a sequence of undated merges.

The comparison SHALL be an ordering rather than a test for difference. A declared version ordered before the last archived one SHALL be refused, and SHALL be reported distinctly from a version that has not moved, because the two have different causes: one is work not yet done, and the other is a version that went backwards.

The declared version SHALL therefore be a value the program can order. A version it cannot parse into an ordering SHALL be reported rather than accepted, because a gate that cannot compare its inputs is not a gate.

The last archived version SHALL be read from the version each archived change records, in the form `toolchain-packaging` requires. It SHALL NOT be derived from an archive directory's name. A directory name is a display string that is shortened and renamed, and a gate derived from one depends on a convention nothing enforces — which is how this gate came to compare against a value no project ever had.

The bootstrap operation SHALL write an initial version that is ordered before every version a project can later declare, so that a project always has a version on record and the comparison is total. Archival SHALL NOT treat an absent previous version as grounds to pass. A project's first archival SHALL advance past the initial version exactly as a later one advances past its predecessor, because a rule with an exception that applies once per project is a rule exercised once per project.

Archival SHALL NOT modify the declaration. Advancing the version SHALL be the contributor's or the agent's act, and the program's part SHALL be the comparison alone.

That division is the one archival already uses: an agent produces the change's specification and the program validates it, because the judgment is judgment and the checking is exact. Deciding what a version increment signifies, and by how much, is judgment. Whether one version is ordered after another is not.

This reservation SHALL be stated on the act rather than on the command. A command SHALL NOT perform an act the project reserves for judgment, whatever else that command also does. A command SHALL NOT be restricted from an act that requires no judgment because of what its name suggests it does. The earlier form of this obligation — that a command whose stated job is checking does not write to the project it checks — generalised a reservation about the declaration into a prohibition on every write, and that generalisation is what left archival as an operation the program declined to complete.

#### Scenario: Archiving without advancing the version
- **WHEN** a contributor archives a change while the declared version equals the last archived version
- **THEN** archival does not complete, and the reason names the version that must advance

#### Scenario: Archiving with an advanced version
- **WHEN** a contributor archives a change whose declared version is ordered after the last archived version
- **THEN** the version gate permits archival to proceed

#### Scenario: Archiving with a version that went backwards
- **WHEN** a contributor archives a change whose declared version is ordered before the last archived version
- **THEN** archival does not complete, and the report distinguishes this from a version that has not moved

#### Scenario: A version the program cannot order
- **WHEN** the declared version cannot be parsed into an ordering
- **THEN** archival does not complete, and the report names the version it could not read

#### Scenario: A project's first archival
- **GIVEN** a freshly bootstrapped project that has archived nothing, whose declared version is still the initial one
- **WHEN** a contributor archives its first change without advancing the version
- **THEN** archival is refused on the same grounds as any later archival, because the initial version is already on record

#### Scenario: The program does not advance the version itself
- **WHEN** archival runs
- **THEN** the declaration is unchanged on disk, whether archival completed or refused

#### Scenario: Archiving with no declaration
- **WHEN** archival runs in a project with no declaration
- **THEN** it reports the absent declaration and names the operation that creates one

#### Scenario: The last archived version comes from the recorded version
- **GIVEN** an archive whose directory names carry no version
- **WHEN** archival determines the last archived version
- **THEN** it reads the version each archived change recorded, and the gate compares against it

#### Scenario: A gate that cannot be satisfied by renaming
- **WHEN** an archived change's directory is renamed
- **THEN** the version the gate reads is unchanged, because the version is a record rather than part of the name

#### Scenario: Archival that writes without advancing the version
- **WHEN** archival completes and moves the change into the archive
- **THEN** the declaration is unchanged, because advancing the version is the contributor's act and archival's part is the comparison alone
