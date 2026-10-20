## MODIFIED Requirements

### Requirement: Formatting preserves meaning

Formatting SHALL NOT change a document's semantic content. It SHALL preserve every semantic unit, every identifier, every attribute value, every cross-reference, and the document order of content. It SHALL NOT add, remove, or reorder semantic units.

One kind of attribute value is excepted. Where an asset reference names bytes the project's store does not hold, formatting SHALL fill it with the address of the bytes the toolchain ships. A reference that names bytes the store holds SHALL be left as written, even when the store also holds a newer revision, because the reference records what the document was authored against.

An asset reference SHALL be the value of an attribute through which an element loads an asset. These are the same attributes that document-validation's "An asset reference must match the bytes it names" judges, so the formatter and the validator SHALL NOT disagree about which values are references. A hyperlink is a citation, not something the document loads, and SHALL NOT be filled. Text SHALL NOT be read as an asset reference. Prose that quotes one, whether as characters in a code element or escaped in an example, is content, and formatting SHALL leave it as the author wrote it.

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

#### Scenario: Prose that quotes an asset reference
- **GIVEN** a project whose store does not hold the bytes `assets/htmlspec.css` names
- **WHEN** a document's prose quotes `href="assets/htmlspec.css"` in a code element, and the document is formatted
- **THEN** the quoted words are unchanged, and only references on the document's own loading elements are filled

#### Scenario: A hyperlink to an asset
- **GIVEN** a project whose store does not hold the bytes a document's hyperlink names
- **WHEN** the document is formatted
- **THEN** the hyperlink is left as written

#### Scenario: A reference that names stored bytes
- **GIVEN** a document whose stylesheet reference names a revision the store holds, and a store that also holds a newer revision
- **WHEN** the document is formatted
- **THEN** the reference still names the revision it was authored against
