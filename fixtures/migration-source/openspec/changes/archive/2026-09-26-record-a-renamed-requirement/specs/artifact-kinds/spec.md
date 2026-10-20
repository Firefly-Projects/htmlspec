## ADDED Requirements

### Requirement: A rename reaches the accepted contract as a new heading on the same identifier

Where a change's specification states that a requirement replaced a previous heading, applying that
specification's contract to the project's accepted specifications SHALL give the accepted requirement
carrying that identifier the new heading, and SHALL NOT change its identifier.

The accepted specification SHALL NOT carry the `renamed-from` paragraph. The accepted contract states
what the project requires now, under the name it uses now, and the history of a heading belongs to
the archived change that renamed it. This is the arrangement OpenSpec keeps: a main specification
holds no RENAMED section, and the rename survives in the change that made it.

Where the change also modified the requirement, the merge SHALL apply the modification to the same
accepted requirement. The rename and the modification SHALL be applied as two facts about one
requirement, and neither SHALL be applied as a removal of the old requirement followed by the
addition of a new one, because that would regenerate an identifier the schema requires to be stable.

Where the accepted requirement carrying that identifier already has the new heading, applying the
rename SHALL change nothing. A rename applied before archival, by the operation that applies a
change's contract, is the ordinary case, and archival SHALL NOT treat it as a conflict.

The merge remains the agent's work under "Every synthesis and every contract update is agent-driven".
This requirement states what a correctly applied rename leaves behind, not a program that applies it.

#### Scenario: A rename applied to the accepted contract
- **GIVEN** a change whose specification states requirement `req-x` under a new heading, with a `renamed-from` paragraph naming the heading the accepted specification holds
- **WHEN** its contract is applied to the accepted specifications
- **THEN** the accepted requirement `req-x` carries the new heading, keeps its identifier, and carries no `renamed-from` paragraph

#### Scenario: A rename and a modification in one change
- **GIVEN** a change that renames and rewrites one requirement
- **WHEN** its contract is applied
- **THEN** the one accepted requirement takes both the new heading and the new content, and no requirement is removed or added

#### Scenario: A rename already applied
- **GIVEN** an accepted requirement that already carries the heading a change renames it to
- **WHEN** the change's contract is applied again
- **THEN** nothing changes, and nothing is reported as a conflict

#### Scenario: The history is read from the archive
- **WHEN** a reader wants to know what an accepted requirement was previously called
- **THEN** the archived change that renamed it states the previous heading, and the accepted specification does not
