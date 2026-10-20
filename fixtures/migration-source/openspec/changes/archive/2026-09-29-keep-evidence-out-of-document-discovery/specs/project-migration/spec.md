## MODIFIED Requirements

### Requirement: Evidence crosses unchanged

A source change's evidence files SHALL be carried into the migrated change unchanged, and
SHALL NOT be converted into the destination format.

Evidence is a transcript, a measurement, or a recorded complaint, and its worth is that nobody
edited it. Converting one is rewriting a record, and a rewritten record cannot serve the
purpose evidence serves.

The migrated change's documents SHALL reference the carried evidence, so that a reader reaches
it from the document that cites it rather than by finding it in a directory.

**Carried evidence SHALL NOT be subject to the conformance checks run over the staged project.**
Those checks are the project's own, and a change's evidence is not a document of the project.
So a record that is already in the destination format neither gates promotion nor is rewritten by
it, even at a superseded contract version or naming an asset store the project does not hold.
That includes a document preserved under "A source entity already in the destination format is
not assumed usable". A requirement to carry records unchanged, paired with a check that refuses
them, would leave a migration only two choices: edit the record, or leave it behind.

#### Scenario: Evidence carried verbatim
- **WHEN** a change carrying evidence files is migrated
- **THEN** each evidence file is present in the migrated change with its contents unchanged

#### Scenario: Evidence is referenced
- **WHEN** a migrated document makes a claim its change recorded evidence for
- **THEN** the document references the carried evidence file

#### Scenario: Carried evidence in the destination format does not gate promotion
- **GIVEN** a staged project whose documents all conform, and a migrated change whose evidence holds an HTML file declaring an earlier contract version and naming an asset store the project does not hold
- **WHEN** promotion's conformance checks run over the staged project
- **THEN** they report nothing about that file, promotion proceeds, and the file arrives byte-identical
