## Purpose

Defines the canonical page a reviewer opens — its structure, its restrained visual system, its print behavior, and the pinned offline runtime that renders diagrams, charts, and equations — so that a specification is legible in a browser, on paper, and with scripting disabled.

## Requirements

### Requirement: A document is self-contained and offline

A document SHALL render completely without network access. It SHALL NOT reference remote scripts, stylesheets, fonts, images, or analytics. All referenced assets SHALL resolve to files within the project.

#### Scenario: Opened with no network
- **GIVEN** a machine with networking disabled
- **WHEN** a reviewer opens a document in a browser
- **THEN** the document renders fully, including its diagrams and charts

#### Scenario: Remote reference present
- **WHEN** a document references a script or font over `http:` or `https:`
- **THEN** the document is invalid

#### Scenario: No tracking
- **WHEN** a reviewer opens a document
- **THEN** no request leaves the machine

### Requirement: Core content is readable without JavaScript

The semantic content of a document — its requirements, scenarios, decisions, tasks, evidence, risks, and verification — SHALL be present in the served markup and legible with JavaScript disabled. JavaScript SHALL only enhance presentation.

#### Scenario: Scripting disabled
- **GIVEN** a browser with JavaScript disabled
- **WHEN** a reviewer opens a document
- **THEN** every requirement, scenario, decision, task, and verification result is readable, correctly ordered, and styled

#### Scenario: Diagram with scripting disabled
- **WHEN** a document containing a diagram is opened with JavaScript disabled
- **THEN** the diagram's source or a textual description is visible in place of the rendered figure, rather than an empty region

#### Scenario: Content injected by script
- **WHEN** a document supplies a requirement's text only by running JavaScript
- **THEN** the document is invalid

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

### Requirement: A document is navigable

A document SHALL present its sections in a stable, predictable order and SHALL provide navigation to its semantic units. Every semantic unit SHALL be addressable by a fragment link that resolves within the document.

#### Scenario: Deep link to a requirement
- **WHEN** a reviewer follows a link ending in `#req-theme-selection`
- **THEN** the browser scrolls to that requirement

#### Scenario: Navigation without scripting
- **WHEN** a document is opened with JavaScript disabled
- **THEN** its navigation still lists the document's sections and its links still resolve

#### Scenario: Broken internal link
- **WHEN** a document contains a fragment link whose target does not exist
- **THEN** the document is invalid

### Requirement: The visual system favors comprehension over decoration

A document SHALL use the project's shared visual system: flat editorial layout, serif body text, high information density, and muted color. Color SHALL NOT be the only means of conveying meaning. Styling SHALL NOT hide or reorder semantic content.

#### Scenario: Change state conveyed without color alone
- **WHEN** a reviewer with a color-vision deficiency reads a document containing added, modified, and removed requirements
- **THEN** each unit's change state is distinguishable by text or shape, not only by hue

#### Scenario: Styling does not hide content
- **WHEN** a document is styled
- **THEN** no semantic unit is visually hidden or moved out of its document order

#### Scenario: Readable at a narrow width
- **WHEN** a document is viewed in a narrow window
- **THEN** its text reflows and remains readable without horizontal scrolling of body prose

### Requirement: Documents print cleanly

A document SHALL provide print styling that produces a legible paper or PDF rendering. Navigation chrome and interactive controls SHALL be omitted from print output. Content SHALL NOT be truncated at page boundaries.

#### Scenario: Printing to PDF
- **WHEN** a reviewer prints a document to PDF
- **THEN** the result contains the full content with legible type, no clipped diagrams, and no navigation chrome

#### Scenario: Link targets in print
- **WHEN** a document containing external references is printed
- **THEN** those references remain identifiable on paper rather than appearing as unresolvable link text

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
