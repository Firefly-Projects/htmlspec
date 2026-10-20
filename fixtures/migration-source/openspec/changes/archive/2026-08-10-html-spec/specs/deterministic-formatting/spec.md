## Purpose

Defines the canonical serialization of a specification document so that rich HTML still produces small, meaningful Git diffs, and so two agents editing the same document independently do not fight over formatting.

## ADDED Requirements

### Requirement: Formatting is idempotent

Formatting a document SHALL be a fixed point: formatting an already-formatted document SHALL produce byte-identical output. Applying formatting repeatedly SHALL never continue to change the file.

#### Scenario: Second pass changes nothing
- **WHEN** a document is formatted twice in succession
- **THEN** the output of the second pass is byte-identical to the output of the first

#### Scenario: Formatted file in version control
- **GIVEN** a formatted document committed to Git
- **WHEN** formatting is run again with no content edits
- **THEN** Git reports no changes

### Requirement: Formatting preserves meaning

Formatting SHALL NOT change a document's semantic content. It SHALL preserve every semantic unit, every identifier, every attribute value, every cross-reference, and the document order of content. It SHALL NOT add, remove, or reorder semantic units.

#### Scenario: Identifiers survive formatting
- **WHEN** a document is formatted
- **THEN** every semantic unit retains the identifier it had before

#### Scenario: Content order preserved
- **WHEN** a document with requirements in a deliberate order is formatted
- **THEN** the requirements appear in the same order

#### Scenario: Whitespace-sensitive content untouched
- **WHEN** a document contains a preformatted code excerpt
- **THEN** formatting leaves that content's internal whitespace and line breaks exactly as authored

#### Scenario: Formatting an invalid document
- **WHEN** formatting is applied to a document that fails validation
- **THEN** the formatter either produces canonically formatted output without altering meaning, or refuses and reports why — it never silently repairs or discards content

### Requirement: Serialization is canonical

For any given document content, there SHALL be exactly one canonical serialization. Attribute order, indentation, line breaking, quoting, character escaping, tag closing, and case SHALL be fully determined by the formatting rules rather than by authoring choices or generator behavior.

#### Scenario: Two agents converge
- **GIVEN** two agents that produce the same semantic document with different attribute order and indentation
- **WHEN** both outputs are formatted
- **THEN** the two files are byte-identical

#### Scenario: Attribute order is fixed
- **WHEN** a formatted element carries a class, an identifier, and a change state
- **THEN** those attributes appear in the order the formatting rules prescribe, regardless of the order they were written in

#### Scenario: Escaping is consistent
- **WHEN** content contains characters that may be written either literally or as entities
- **THEN** the formatted output uses one prescribed form for each such character

### Requirement: Formatting introduces no diff churn

Formatting SHALL NOT widen a diff beyond the units a content edit touched. Editing, adding, or removing a unit SHALL NOT cause the formatter to reflow or reindent unrelated parts of the document.

This requirement constrains the formatter only. It does not promise a minimal Git diff for a given edit. Preserved author line breaks mean a one-line paragraph produces a whole-line change when one sentence changes, and that cost belongs to the line structure rather than to the formatter. Diff minimality SHALL be measured as a separate property.

#### Scenario: Editing one requirement
- **WHEN** the prose of one requirement is revised and the document is formatted
- **THEN** the formatter changes no line outside that requirement

#### Scenario: Whole-line change is not a violation
- **WHEN** one sentence changes inside a paragraph held on a single long line
- **THEN** the diff covers that whole line, and the formatter is still conformant

#### Scenario: Inserting a requirement
- **WHEN** a new requirement is inserted between two existing ones
- **THEN** the diff shows the added requirement and does not rewrite the surrounding requirements

#### Scenario: Long prose reflow
- **WHEN** a sentence within a paragraph is edited
- **THEN** the diff does not rewrap the entire surrounding paragraph or section

### Requirement: Generated content is ordered deterministically

Where a document contains content whose order is not authored — such as collected references, indexes, or navigation — that order SHALL be determined by a stable rule. Regenerating such content from unchanged inputs SHALL produce identical output.

#### Scenario: Regenerating navigation
- **WHEN** a document's navigation is regenerated with no content change
- **THEN** the navigation is byte-identical to before

#### Scenario: No incidental ordering
- **WHEN** generated content is produced twice from the same inputs in different runs
- **THEN** the ordering does not vary with iteration order, timestamps, or filesystem order

### Requirement: Formatting can be checked without writing

It SHALL be possible to determine whether a document is canonically formatted without modifying it, so that continuous integration can reject unformatted documents.

#### Scenario: Check passes
- **WHEN** a canonically formatted document is checked
- **THEN** the check reports success and the file is not modified

#### Scenario: Check fails
- **WHEN** an unformatted document is checked
- **THEN** the check reports failure, identifies the document, and the file is not modified

### Requirement: Files use a stable text encoding and line ending

Formatted documents SHALL use UTF-8 encoding, SHALL use a single prescribed line ending, SHALL end with exactly one final newline, and SHALL NOT contain trailing whitespace on any line.

#### Scenario: Line endings normalized
- **WHEN** a document authored with mixed line endings is formatted
- **THEN** the output uses the prescribed line ending throughout

#### Scenario: Cross-platform round trip
- **GIVEN** contributors on different operating systems
- **WHEN** each formats the same document
- **THEN** the results are byte-identical
