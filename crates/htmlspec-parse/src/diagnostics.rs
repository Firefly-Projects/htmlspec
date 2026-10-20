//! Stable, project-owned diagnostic codes for recovered HTML parse errors.
//!
//! `html5ever` reports a recovery as free text through `TreeSink::parse_error`,
//! not as a typed code. This module owns the mapping from the pinned parser's
//! message text onto codes this project controls, so a parser upgrade changes
//! the mapping and never the diagnostic surface.
//!
//! Every recovery is fatal. No recovery is permitted. The enumeration and the
//! measurements behind it live in
//! `openspec/changes/html-spec/evidence/parse-errors.md`.

/// The `html5ever` version this enumeration was built against.
///
/// [`crate::diagnostics`] is only correct for this version. The test
/// `parser_version_is_the_reviewed_one` fails when the pin moves, which forces
/// the enumeration to be reviewed rather than assumed.
pub const REVIEWED_HTML5EVER_VERSION: &str = "0.35.0";

/// A stable diagnostic code for one class of recovered parse error.
///
/// The variants are a closed set. [`ParseDiagnosticCode::Unmapped`] is the
/// residual: a message this enumeration does not name is still fatal, so a new
/// parser message cannot quietly widen what the validator accepts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParseDiagnosticCode {
    /// A token appeared in an insertion mode that does not accept it.
    UnexpectedToken,
    /// An element was still open when another element closed.
    UnexpectedOpenElement,
    /// An element was never closed before the end of the body.
    UnexpectedOpenTagAtEof,
    /// An end tag had no matching open tag.
    NoTagToClose,
    /// Misnested inline formatting elements.
    FormattingElement,
    /// A stray end tag forced the parser out of an open element.
    SpecialTagInGenericClose,
    /// One attribute name appeared twice on one element.
    DuplicateAttribute,
    /// A malformed character reference.
    CharacterReference,
    /// Text appeared where table structure is required, and was relocated.
    TableText,
    /// A malformed doctype, or a doctype outside the initial insertion mode.
    BadDoctype,
    /// A character illegal in the current tokenizer state.
    BadCharacter,
    /// Input ended inside a construct.
    UnexpectedEof,
    /// A self-closing marker on an element where it carries no meaning.
    SelfClosing,
    /// Attributes on an end tag, which the parser discards.
    EndTagAttributes,
    /// A structural fault specific to one element family.
    NestedElement,
    /// A recovery this enumeration does not name.
    Unmapped,
}

impl ParseDiagnosticCode {
    /// The stable wire form of the code, used in machine-readable diagnostics.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnexpectedToken => "parse/unexpected-token",
            Self::UnexpectedOpenElement => "parse/unexpected-open-element",
            Self::UnexpectedOpenTagAtEof => "parse/unexpected-open-tag-at-eof",
            Self::NoTagToClose => "parse/no-tag-to-close",
            Self::FormattingElement => "parse/formatting-element",
            Self::SpecialTagInGenericClose => "parse/special-tag-in-generic-close",
            Self::DuplicateAttribute => "parse/duplicate-attribute",
            Self::CharacterReference => "parse/character-reference",
            Self::TableText => "parse/table-text",
            Self::BadDoctype => "parse/bad-doctype",
            Self::BadCharacter => "parse/bad-character",
            Self::UnexpectedEof => "parse/unexpected-eof",
            Self::SelfClosing => "parse/self-closing",
            Self::EndTagAttributes => "parse/end-tag-attributes",
            Self::NestedElement => "parse/nested-element",
            Self::Unmapped => "parse/unmapped-recovery",
        }
    }

    /// Every code except the residual, in wire-form order.
    pub const ALL_MAPPED: &'static [Self] = &[
        Self::UnexpectedToken,
        Self::UnexpectedOpenElement,
        Self::UnexpectedOpenTagAtEof,
        Self::NoTagToClose,
        Self::FormattingElement,
        Self::SpecialTagInGenericClose,
        Self::DuplicateAttribute,
        Self::CharacterReference,
        Self::TableText,
        Self::BadDoctype,
        Self::BadCharacter,
        Self::UnexpectedEof,
        Self::SelfClosing,
        Self::EndTagAttributes,
        Self::NestedElement,
    ];
}

impl std::fmt::Display for ParseDiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Message prefixes owned by the pinned parser, in match order.
///
/// Matching is by prefix, because the exact form of a message embeds `Debug`
/// output that a patch release can reword. A reworded prefix falls through to
/// [`ParseDiagnosticCode::Unmapped`], which is fatal and loud. It cannot land on
/// the wrong specific code, because a specific code is only reachable by prefix.
const PREFIX_TABLE: &[(&str, ParseDiagnosticCode)] = &[
    ("Unexpected token", ParseDiagnosticCode::UnexpectedToken),
    (
        "Unexpected open element",
        ParseDiagnosticCode::UnexpectedOpenElement,
    ),
    (
        "Unexpected open tag",
        ParseDiagnosticCode::UnexpectedOpenTagAtEof,
    ),
    ("No <p> tag to close", ParseDiagnosticCode::NoTagToClose),
    (
        "No matching tag to close",
        ParseDiagnosticCode::NoTagToClose,
    ),
    ("No heading tag to close", ParseDiagnosticCode::NoTagToClose),
    ("Formatting element", ParseDiagnosticCode::FormattingElement),
    (
        "Found special tag while closing generic tag",
        ParseDiagnosticCode::SpecialTagInGenericClose,
    ),
    (
        "Duplicate attribute",
        ParseDiagnosticCode::DuplicateAttribute,
    ),
    (
        "Character reference",
        ParseDiagnosticCode::CharacterReference,
    ),
    (
        "Invalid character reference",
        ParseDiagnosticCode::CharacterReference,
    ),
    (
        "Numeric character reference without digits",
        ParseDiagnosticCode::CharacterReference,
    ),
    (
        "EOF after '#' in character reference",
        ParseDiagnosticCode::CharacterReference,
    ),
    (
        "EOF in numeric character reference",
        ParseDiagnosticCode::CharacterReference,
    ),
    ("Non-space table text", ParseDiagnosticCode::TableText),
    ("Bad DOCTYPE", ParseDiagnosticCode::BadDoctype),
    ("DOCTYPE in", ParseDiagnosticCode::BadDoctype),
    ("Bad character", ParseDiagnosticCode::BadCharacter),
    ("Saw EOF in state", ParseDiagnosticCode::UnexpectedEof),
    ("Saw ", ParseDiagnosticCode::BadCharacter),
    ("Unexpected EOF", ParseDiagnosticCode::UnexpectedEof),
    (
        "Unacknowledged self-closing tag",
        ParseDiagnosticCode::SelfClosing,
    ),
    ("Self-closing end tag", ParseDiagnosticCode::SelfClosing),
    (
        "Attributes on an end tag",
        ParseDiagnosticCode::EndTagAttributes,
    ),
    ("Nested <nobr>", ParseDiagnosticCode::NestedElement),
    ("nested buttons", ParseDiagnosticCode::NestedElement),
    ("nested forms", ParseDiagnosticCode::NestedElement),
    ("nested heading tags", ParseDiagnosticCode::NestedElement),
    (
        "Closing wrong heading tag",
        ParseDiagnosticCode::NestedElement,
    ),
    (
        "Bad open element on </form>",
        ParseDiagnosticCode::NestedElement,
    ),
    (
        "Form element not in scope on </form>",
        ParseDiagnosticCode::NestedElement,
    ),
    (
        "Null form element pointer on </form>",
        ParseDiagnosticCode::NestedElement,
    ),
    (
        "bookmark not found in active formatting elements",
        ParseDiagnosticCode::NestedElement,
    ),
    (
        "expected to close <td> or <th> with cell",
        ParseDiagnosticCode::NestedElement,
    ),
    (
        "</body> with no <body> in scope",
        ParseDiagnosticCode::NestedElement,
    ),
    (
        "</html> with no <body> in scope",
        ParseDiagnosticCode::NestedElement,
    ),
];

/// Classifies one recovered parser message.
///
/// An unrecognized message becomes [`ParseDiagnosticCode::Unmapped`], which is
/// fatal. This function never reports that a recovery is acceptable.
pub fn classify(message: &str) -> ParseDiagnosticCode {
    PREFIX_TABLE
        .iter()
        .find(|(prefix, _)| message.starts_with(prefix))
        .map_or(ParseDiagnosticCode::Unmapped, |(_, code)| *code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mapped_code_is_reachable_from_the_prefix_table() {
        for code in ParseDiagnosticCode::ALL_MAPPED {
            assert!(
                PREFIX_TABLE.iter().any(|(_, mapped)| mapped == code),
                "{code} has no prefix in the table, so it can never be reported"
            );
        }
    }

    #[test]
    fn an_unknown_message_is_unmapped_rather_than_ignored() {
        assert_eq!(
            classify("A message no released html5ever has ever emitted"),
            ParseDiagnosticCode::Unmapped
        );
    }

    #[test]
    fn wire_forms_are_unique() {
        let mut seen: Vec<&str> = ParseDiagnosticCode::ALL_MAPPED
            .iter()
            .map(|c| c.as_str())
            .collect();
        seen.push(ParseDiagnosticCode::Unmapped.as_str());
        let before = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(before, seen.len(), "two codes share a wire form");
    }
}
