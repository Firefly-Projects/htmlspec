## Purpose

Defines the canonical page a reviewer opens — its structure, its restrained visual system, its print behavior, and the pinned offline runtime that renders diagrams, charts, and equations — so that a specification is legible in a browser, on paper, and with scripting disabled.

## ADDED Requirements

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

### Requirement: A document declares its identity and provenance

Every document SHALL declare, in machine-readable form, the schema version it conforms to, the change or spec it represents, and its lifecycle status. A reviewer SHALL be able to see this information on the page without reading source.

#### Scenario: Schema version present
- **WHEN** a consumer reads a document
- **THEN** it can determine which version of the element contract the document targets before parsing its content

#### Scenario: Status visible to a reviewer
- **WHEN** a reviewer opens a document representing a proposal under review
- **THEN** the page shows its status, so an unapproved proposal is not mistaken for accepted specification

#### Scenario: Missing schema version
- **WHEN** a document omits its schema version
- **THEN** the document is invalid

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

#### Scenario: Starting a new change
- **WHEN** a contributor initializes a new change document
- **THEN** the resulting file passes validation immediately and opens as a readable page

#### Scenario: Initializing into a project without assets
- **WHEN** a document is created in a project whose shared assets are not yet present
- **THEN** the assets are established as part of initialization, so the document renders offline without a further step
