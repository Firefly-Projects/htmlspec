//! Parses a document and collects every recovered parse error.

use std::cell::RefCell;
use std::rc::Rc;

use html5ever::tendril::TendrilSink;
use kuchikiki::NodeRef;

use crate::diagnostics::{classify, ParseDiagnosticCode};
use crate::visible::shown;

/// One recovery the parser performed, with the code this project owns for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveredError {
    /// The stable project-owned code.
    pub code: ParseDiagnosticCode,
    /// The message the pinned parser produced, kept as evidence, with each character a terminal
    /// would not show as itself written as its code point, [`crate::visible::shown`].
    ///
    /// The parser quotes the character it stopped at, which is the document's own, so a document
    /// holding an escape sequence's opening would otherwise write it into every report of the
    /// recovery. Every other character is kept as the parser wrote it.
    pub message: String,
}

/// The result of parsing one document.
///
/// The tree is always present, because HTML5 parsing always produces one. A
/// non-empty `recovered` list means the document is rejected: a browser needed
/// guesswork to read it, so its interpretation is not determined by the markup.
#[derive(Clone, Debug)]
pub struct ParseOutcome {
    /// The parsed document node.
    pub document: NodeRef,
    /// Every recovery the parser reported, in report order.
    pub recovered: Vec<RecoveredError>,
    /// Whether the parser placed the document in quirks mode.
    ///
    /// Quirks mode is not a recovery and the parser reports no error for a
    /// legacy doctype, so this is surfaced separately for the rule that
    /// requires the short doctype.
    pub quirks: bool,
}

impl ParseOutcome {
    /// Whether the document parsed with no recovery at all.
    pub fn is_recovery_free(&self) -> bool {
        self.recovered.is_empty()
    }
}

/// Parses `html` with the pinned parser, collecting recoveries.
///
/// `exact_errors` is enabled on both the tokenizer and the tree builder, so a
/// message carries the offending token and the insertion mode. That is what
/// distinguishes two faults which share a terse message.
pub fn parse_document(html: &str) -> ParseOutcome {
    let collected: Rc<RefCell<Vec<RecoveredError>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&collected);

    let mut opts = kuchikiki::ParseOpts::default();
    opts.tokenizer.exact_errors = true;
    opts.tree_builder.exact_errors = true;
    opts.on_parse_error = Some(Box::new(move |message| {
        // Classified from the parser's own text, then escaped for the reports that quote it.
        let message = message.to_string();
        sink.borrow_mut().push(RecoveredError {
            code: classify(&message),
            message: shown(&message),
        });
    }));

    let document = kuchikiki::parse_html_with_options(opts)
        .one(html)
        .document_node;
    let quirks = matches!(
        document
            .as_document()
            .map(|d| d.quirks_mode())
            .unwrap_or(html5ever::tree_builder::QuirksMode::NoQuirks),
        html5ever::tree_builder::QuirksMode::Quirks
            | html5ever::tree_builder::QuirksMode::LimitedQuirks
    );

    let recovered = collected.borrow().clone();
    ParseOutcome {
        document,
        recovered,
        quirks,
    }
}
