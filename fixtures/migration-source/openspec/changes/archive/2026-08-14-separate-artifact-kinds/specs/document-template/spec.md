## MODIFIED Requirements

### Requirement: A document declares its identity and provenance

Every document SHALL declare, in machine-readable form, the schema version it conforms to, the change or spec it represents, and **which artifact kind it is**. A reviewer SHALL be able to see this information on the page without reading source.

The declared kind replaces the per-document lifecycle status. A stage — exploring, proposed, accepted, implementing, verified, archived — is a property of a *change* as it moves, not of each file in it: a proposal is a proposal for its whole life, and a specification does not become a proposal again. Declaring a stage per document therefore recorded the wrong thing about the wrong subject, and nothing in validation ever read it.

A document SHALL NOT declare a lifecycle status. Where a reviewer needs to know how far a change has progressed, the presence of its documents reports it: a change with a specification has been built, and a change with only a proposal has not.

#### Scenario: Schema version present
- **WHEN** a consumer reads a document
- **THEN** it can determine which version of the element contract the document targets before parsing its content

#### Scenario: Kind visible to a reviewer
- **WHEN** a reviewer opens a document
- **THEN** the page shows which artifact it is, so a proposal under discussion is not mistaken for an accepted specification

#### Scenario: Kind readable without parsing content
- **WHEN** a consumer reads a document's metadata
- **THEN** it can determine the artifact kind before reading the body, so it knows which units to expect

#### Scenario: Missing schema version
- **WHEN** a document omits its schema version
- **THEN** the document is invalid

#### Scenario: Missing kind
- **WHEN** a document declares no artifact kind
- **THEN** the document is invalid, because nothing determines which units belong in it

#### Scenario: A document declaring a lifecycle status
- **WHEN** a document declares a lifecycle status
- **THEN** validation reports it, because a stage describes a change rather than one of its documents
