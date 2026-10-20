## MODIFIED Requirements

### Requirement: Documents share one pinned runtime

Diagram, chart, and equation support SHALL come from a fixed set of approved libraries at versions pinned by the project. The runtime SHALL be shared across documents in a project rather than duplicated into each file. Arbitrary generated scripts SHALL NOT be embedded in a document.

Each runtime asset SHALL be stored at a location derived from the hash of its bytes. Identical bytes SHALL therefore occupy exactly one location in a project, and bytes that differ SHALL occupy distinct locations rather than overwriting one another. A document SHALL reference each asset by the hash it was authored against, so the bytes a document renders with cannot change without changing the document.

An asset location SHALL NOT be rewritten in place. Replacing a library SHALL add a location and leave existing locations intact, so a document authored earlier continues to render as authored.

#### Scenario: Approved libraries
- **WHEN** a document renders a diagram or a chart
- **THEN** it does so through the project's pinned diagram or charting library, and equations use native MathML with no library at all

#### Scenario: Shared assets
- **GIVEN** a project containing many documents
- **WHEN** the runtime is measured
- **THEN** each library appears once in the project rather than once per document

#### Scenario: Unapproved script
- **WHEN** a document embeds an inline script that is not part of the pinned runtime
- **THEN** the document is invalid

#### Scenario: Runtime version is recorded
- **WHEN** a reviewer inspects a project's assets
- **THEN** each vendored library's exact version and content hash are recorded, so a rendered document can be reproduced later

#### Scenario: Two documents needing the same bytes
- **GIVEN** two documents that reference the same library version
- **WHEN** the project's assets are measured
- **THEN** one stored copy serves both documents

#### Scenario: Upgrading a library
- **GIVEN** an existing document that references a library
- **WHEN** a newer version of that library is added to the project
- **THEN** the new bytes are stored at a new location, the existing document's references are unchanged, and it renders with the bytes it was authored against

#### Scenario: Two documents needing different bytes of one file
- **GIVEN** two documents authored against different revisions of the same stylesheet
- **WHEN** both are stored in one project
- **THEN** both revisions are retained at distinct locations and neither document is silently re-pointed at the other's bytes

#### Scenario: An asset reference identifies its bytes
- **WHEN** a consumer reads a document's asset reference
- **THEN** it can determine the exact bytes the document expects without consulting any other file

### Requirement: A project can be initialized with a conforming document

The project SHALL provide a way to create a new document that already conforms to the element contract, the visual system, and the runtime conventions, without hand-assembly.

Initialization SHALL write asset references that match the bytes it stores. It SHALL NOT copy a runtime asset that the project already stores, and SHALL NOT produce a document whose references and stored bytes disagree.

#### Scenario: Starting a new change
- **WHEN** a contributor initializes a new change document
- **THEN** the resulting file passes validation immediately and opens as a readable page

#### Scenario: Initializing into a project without assets
- **WHEN** a document is created in a project whose shared assets are not yet present
- **THEN** the assets are established as part of initialization, so the document renders offline without a further step

#### Scenario: Initializing into a project that already has the runtime
- **GIVEN** a project already storing the runtime a new document needs
- **WHEN** a contributor initializes that document
- **THEN** it references the stored bytes and no second copy of any asset is written

#### Scenario: Initialization cannot emit a stale reference
- **WHEN** initialization writes a document and its asset references
- **THEN** every reference matches the hash of the bytes the project stores, so a document cannot be created already pointing at bytes it does not get
