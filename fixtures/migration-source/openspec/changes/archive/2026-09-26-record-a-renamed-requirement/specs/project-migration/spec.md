## ADDED Requirements

### Requirement: A source rename is carried as the heading the requirement replaced

Where a source delta carries a `## RENAMED Requirements` section, migration SHALL carry each
`FROM:`/`TO:` pair into the migrated change's specification as the renamed requirement's
`renamed-from` paragraph, stating the `FROM:` heading, on the requirement whose heading is the `TO:`
heading.

The requirement's change state SHALL be taken from the rest of the delta and not from the rename.
Where the same delta also restates the requirement under its `MODIFIED Requirements` section — which
is how OpenSpec records a rename and a rewrite together, the `MODIFIED` block naming the new heading —
the requirement SHALL carry `data-change="modified"`. Where the delta renames the requirement and says
nothing else about it, the requirement SHALL carry `data-change="unchanged"`, and its statements and
scenarios SHALL be those the requirement held before the rename.

Any prose the source's RENAMED section carried beside its pairs — an explanation of why the heading
changed — SHALL be carried into the migrated specification rather than dropped, because it is the
author's reason for the rename and no field holds it.

A rename SHALL NOT be recorded as an open question once the destination can express it. The
requirement "A source distinction the destination cannot express is recorded as open, not mapped to
the nearest state" governs distinctions the destination has no value for, and a rename is no longer
one.

#### Scenario: A rename with a rewrite
- **GIVEN** a source delta that renames a requirement in `## RENAMED Requirements` and restates it under the new heading in `## MODIFIED Requirements`
- **WHEN** the change is migrated
- **THEN** the migrated requirement carries the new heading, `data-change="modified"`, the rewritten content, and a `renamed-from` paragraph stating the previous heading

#### Scenario: A rename alone
- **GIVEN** a source delta that renames a requirement and states nothing else about it
- **WHEN** the change is migrated
- **THEN** the migrated requirement carries the new heading, `data-change="unchanged"`, the content it held before, and a `renamed-from` paragraph

#### Scenario: The author's reason for a rename
- **GIVEN** a RENAMED section carrying a paragraph that explains why the heading changed
- **WHEN** the change is migrated
- **THEN** that paragraph appears in the migrated specification

#### Scenario: A rename is not an open question
- **WHEN** a migration plan accounts for a delta carrying a RENAMED section
- **THEN** the plan records the rename's destination as a `renamed-from` paragraph, not as an open question

## MODIFIED Requirements

### Requirement: A source distinction the destination cannot express is recorded as open, not mapped to the nearest state

Where a source entity carries a distinction the destination's contract has no value for,
migration SHALL record it as an open question and SHALL name the change that would settle it.
Migration SHALL NOT map the distinction onto the nearest available value.

The nearest available value is the tempting choice because it validates. A distinction mapped
onto it is indistinguishable, afterwards, from a source that never made the distinction — so the
information is not merely lost but unrecoverable, and no check can report it because the result
conforms.

Settling the question instead SHALL be out of scope for a migration. Adding a value to the
destination's vocabulary reaches validation, formatting, and presentation, and a migration
decides it in bulk inside a change whose subject is something else. This is the same reason
migration does not invent a layout where the contract is silent.

The plan SHALL record, for each such entity, what the source stated, that the destination has no
value for it, and what migration did instead.

#### Scenario: A change state the destination's vocabulary lacks
- **GIVEN** a source entity recording a distinction, and a destination whose change-state vocabulary has no value for it
- **WHEN** the entity is migrated
- **THEN** the plan records what the source stated, records that the destination has no value for it, and names the change that would add one

#### Scenario: The nearest state is not substituted silently
- **WHEN** a source distinction has no destination value
- **THEN** migration does not record it as the nearest available value and leave the plan silent, because the result would be indistinguishable from a source that never made the distinction

#### Scenario: The vocabulary is not extended by the migration
- **WHEN** migration finds a distinction the destination's vocabulary cannot express
- **THEN** migration does not add a value to that vocabulary, and names the change that would

### Requirement: Identifiers are derived once and every reference resolves through that derivation

Migration SHALL derive each requirement's identifier once, and every reference to that
requirement anywhere in the migrated project SHALL use the derived identifier. The derivation
SHALL be recorded in the plan.

Deriving independently at each site produces different identifiers for one obligation, each
locally valid, so nothing reports the divergence and cross-references silently fail to resolve.

The derivation SHALL be recorded against the source document that stated the heading, rather than
against the heading alone. A source project can state one continuing obligation under different
headings in different changes, because a change may rename a requirement, and each change's
specification states what that change established rather than what the project later called it.
A derivation keyed on the heading alone would either merge those statements — misrepresenting
every change but the last — or report collisions where the source is consistent.

**Where the source records a rename, the renamed requirement SHALL keep the identifier it already
held.** A requirement named in a source `## RENAMED Requirements` section is one obligation under a
new heading, and document-schema's "Every semantic unit has a stable unique identifier" requires a
renamed requirement to keep its identifier. So its identifier SHALL be the one derived from the
heading under which the requirement was first established, and a chain of renames SHALL carry that
one identifier through every change in the chain and into the accepted specification. The plan SHALL
record, against the document that stated each rename, the heading it replaced, the heading it
introduced, and the identifier both resolve to.

Where the source changed a heading without recording a rename — by removing one requirement and
adding another — the two SHALL derive their identifiers independently. The source did not state that
they are one obligation, and migration SHALL NOT infer it from their wording.

The derivation SHALL read the heading in full. A derivation that shortens or normalizes a heading
maps distinct obligations onto one identifier, which manufactures a collision the source did not
contain and refers the contributor to a conflict that is an artifact of the rule.

Where two source requirements in one destination document derive the same identifier, migration
SHALL report the collision rather than resolving it by suffixing. Two distinct obligations
distinguished only by a trailing digit read as versions of one, which is a worse outcome than a
migration that stops and asks. Collapsing several of a source change's specifications into one
document SHALL be understood as the operation most likely to produce such a collision, because it
is the one that brings requirements from different parts of the source into a single identifier
namespace.

#### Scenario: One requirement referenced from several documents
- **WHEN** a requirement appears in an accepted specification and is referenced by an archived change
- **THEN** both use the same derived identifier

#### Scenario: A derivation recorded
- **WHEN** the plan is read
- **THEN** each requirement's source heading, the document that stated it, and the identifier derived from it are all recorded

#### Scenario: One obligation renamed across changes
- **GIVEN** a source project in which a later change renamed a requirement an earlier change established, and recorded the rename in a `## RENAMED Requirements` section
- **WHEN** both changes are migrated
- **THEN** both changes' specifications carry the identifier derived from the heading the earlier change established, each states the heading that change used, and this is not reported as a collision

#### Scenario: A chain of renames
- **GIVEN** a requirement renamed by one change and renamed again by a later one
- **WHEN** the project is migrated
- **THEN** every change in the chain and the accepted specification carry the identifier derived from the first heading

#### Scenario: A heading changed without a recorded rename
- **GIVEN** a source change that removes one requirement and adds another with a similar heading, and records no rename
- **WHEN** it is migrated
- **THEN** the two derive their identifiers independently, and migration does not treat them as one obligation

#### Scenario: Two requirements deriving one identifier
- **WHEN** two source requirements' headings derive the same identifier within one destination document
- **THEN** migration reports the collision and does not resolve it by adding a suffix

#### Scenario: A collision the derivation rule created
- **WHEN** two source requirements state distinct obligations whose full headings differ
- **THEN** they derive distinct identifiers, because a rule that shortened them would report a collision the source did not contain
