## MODIFIED Requirements

### Requirement: A project must not store one asset's bytes more than once

Validation SHALL reject a project that stores identical asset bytes at more than one location. The report SHALL name every location holding the duplicated bytes, so a contributor can tell which copy to remove.

Two assets whose bytes differ SHALL NOT be reported as duplicates, however similar their names, versions, or purposes.

This rule takes a project as its subject rather than a document, so it SHALL be evaluated where a project is in view. A directory holding no asset store has nothing to duplicate, and SHALL NOT be reported.

**A project's stores SHALL be the asset stores within the project's own directory, `htmlspec/`, and no others.** The comparison across stores SHALL be made among those stores alone. A repository holding a project can hold other stores that are not the project's — a conformance fixture that is a project of its own, a toolchain's source copy of the runtime, a vendored copy another tool reads — and bytes those stores share with the project's are not the project storing its bytes twice. The subject of this rule is one project, and a comparison that reaches past the project's directory has changed its subject to the repository.

A store outside the project's directory SHALL still be checked on its own, so a store that holds identical bytes at two of its own locations is reported wherever it lives. It SHALL NOT be compared with the project's stores or with any other store.

The report SHALL state which stores it treated as the project's and which it checked on their own, so a contributor can tell a duplication that is not the project's from a check that did not run.

Where no project is in view, every store SHALL be checked on its own, as before.

#### Scenario: Duplicate copies of one library
- **GIVEN** a project storing the same library bytes in two locations
- **WHEN** the project is validated
- **THEN** validation fails and names both locations

#### Scenario: A copy per document is rejected
- **GIVEN** a project in which each document carries its own copy of the runtime
- **WHEN** the project is validated
- **THEN** validation fails, because the shared-runtime obligation is violated

#### Scenario: Different revisions are not duplicates
- **GIVEN** a project storing two revisions of one stylesheet whose bytes differ
- **WHEN** the project is validated
- **THEN** validation succeeds, because distinct bytes are not a duplicate

#### Scenario: One store serving many documents
- **GIVEN** many documents referencing one stored copy of each library
- **WHEN** the project is validated
- **THEN** validation succeeds

#### Scenario: A project beside a fixture that carries its own store
- **GIVEN** a repository holding a project, and a conformance fixture elsewhere in the repository whose store holds bytes identical to the project's
- **WHEN** the project's asset topology is checked
- **THEN** no duplication is reported, and the report names the fixture's store as checked on its own

#### Scenario: A project beside a toolchain's source copy of the runtime
- **GIVEN** a repository whose root holds a store of the runtime outside the project's directory, and a project whose store holds the same bytes
- **WHEN** the project's asset topology is checked
- **THEN** no duplication is reported

#### Scenario: Two stores inside the project
- **GIVEN** a project whose own directory holds two stores with identical bytes, such as one beside a change's documents
- **WHEN** the project's asset topology is checked
- **THEN** the duplication is reported and names both locations

#### Scenario: A store outside the project that duplicates within itself
- **GIVEN** a store outside the project's directory holding identical bytes at two of its own locations
- **WHEN** the project's asset topology is checked
- **THEN** that duplication is reported, and the store is not compared with the project's
