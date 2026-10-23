//! The HTMLSpec canonical serializer.
//!
//! For any given document content there is exactly one canonical serialization.
//! Attribute order, indentation, block line breaking, quoting, escaping, tag
//! closing, and case are all determined by the rules in [`serialize`], never by
//! authoring choices or generator behavior.
//!
//! What the formatter does *not* do is rewrap prose. Author line breaks inside
//! text-bearing content are preserved exactly, and whitespace-sensitive content —
//! `pre`, `code`, `textarea` — survives byte for byte. [`serialize`] explains why
//! that decision is load-bearing.
//!
//! This crate is not a general-purpose HTML tidy tool. It canonicalizes documents
//! that conform to the contract.

pub mod attributes;
pub mod element;
pub mod escape;
pub mod fallback;
pub mod references;
pub mod relocate;
pub mod serialize;
mod tags;

use std::fmt;

use htmlspec_parse::parse_document;

pub use references::{
    asset_references, rewrite, upgrade, KeptReason, ReferenceChange, RewriteOutcome,
    SourceReference, UpgradeOutcome, UpgradedReference,
};
pub use relocate::{relocate, Move, NotAbsolute, Relocated, Rewritten};
pub use serialize::LINE_ENDING;

/// Why a document could not be formatted.
///
/// There is one reason, and it is the one the specs name: formatting a document
/// whose markup does not determine its own meaning. The formatter *refuses and
/// reports why* rather than canonicalizing one of the readings — which would be a
/// silent repair, and the specs forbid that.
///
/// This is deliberately not "the document failed validation". A document with a
/// dangling reference, a duplicate identifier, or a missing scenario has one
/// unambiguous tree, and the formatter canonicalizes it without altering meaning:
/// refusing there would mean a document could not be tidied until it was already
/// correct, which is backwards. Only ambiguity is fatal, because only ambiguity
/// leaves the formatter without a single answer to write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormatError {
    /// The parser recovered from at least one error, so the document has more
    /// than one plausible reading and formatting would pick one.
    Ambiguous {
        /// Every recovery, in report order, as evidence for the refusal.
        recovered: Vec<String>,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ambiguous { recovered } => {
                write!(
                    f,
                    "refusing to format: the document's markup requires error recovery, so it \
                     has no single meaning to preserve ({} recovered error(s))",
                    recovered.len()
                )?;
                for message in recovered {
                    write!(f, "\n  {message}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for FormatError {}

/// Whether a document is already canonically formatted.
///
/// Returned rather than a bare `bool` so that a caller cannot accidentally read a
/// refusal as "already formatted". The canonical text travels with the verdict, so
/// a caller that wants to write it does not format twice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckOutcome {
    /// Whether the input is byte-identical to its canonical form.
    pub is_canonical: bool,
    /// The canonical form, whether or not the input matched it.
    pub canonical: String,
}

/// Formats a document into its canonical form.
///
/// The result is UTF-8 by the type, uses one prescribed line ending throughout,
/// ends with exactly one newline, and carries no trailing whitespace outside
/// preserved content — the file-level normalization requirement, satisfied by
/// construction in [`serialize::document`] rather than by a cleanup pass.
///
/// Input line endings are normalized before parsing, so a document authored with
/// CRLF and the same document authored with LF produce byte-identical output.
pub fn format(source: &str) -> Result<String, FormatError> {
    let normalized = normalize_line_endings(source);
    let outcome = parse_document(&normalized);
    if !outcome.is_recovery_free() {
        return Err(FormatError::Ambiguous {
            recovered: outcome
                .recovered
                .iter()
                .map(|error| format!("{}: {}", error.code, error.message))
                .collect(),
        });
    }
    Ok(serialize::document(&outcome.document))
}

/// Determines whether a document is canonical, without writing to it.
///
/// Nothing here touches the filesystem: the caller owns reading and writing, so
/// "check mode" is not a second implementation of formatting that could disagree
/// with the first. It is the same function, and the comparison.
pub fn check(source: &str) -> Result<CheckOutcome, FormatError> {
    let canonical = format(source)?;
    Ok(CheckOutcome {
        is_canonical: source == canonical,
        canonical,
    })
}

/// Replaces every CRLF and every lone CR with the prescribed line ending.
///
/// This runs before parsing rather than after serializing. A CR inside `pre` is
/// preserved content, and normalizing after the fact would either corrupt it or
/// require the serializer to know which bytes came from where. Normalizing the
/// input means the tree never contains a CR to begin with, so cross-platform
/// convergence follows from there being nothing to converge.
fn normalize_line_endings(source: &str) -> String {
    if !source.contains('\r') {
        return source.to_owned();
    }
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            // CRLF collapses to one line ending; a lone CR becomes one too.
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push_str(LINE_ENDING);
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>T</title></head><body><p>Body.</p></body></html>\n";

    fn formatted(source: &str) -> String {
        format(source).expect("the sample formats")
    }

    /// The fixed-point property, on a document that starts unformatted.
    #[test]
    fn formatting_is_a_fixed_point() {
        let once = formatted(MINIMAL);
        let twice = formatted(&once);
        assert_eq!(once, twice);
        // And a third pass, because a two-cycle would satisfy a single repeat.
        assert_eq!(twice, formatted(&twice));
    }

    #[test]
    fn a_canonical_document_checks_clean() {
        let canonical = formatted(MINIMAL);
        let outcome = check(&canonical).expect("the check runs");
        assert!(outcome.is_canonical);
        assert_eq!(outcome.canonical, canonical);
    }

    #[test]
    fn an_unformatted_document_fails_the_check_and_is_not_rewritten() {
        let outcome = check(MINIMAL).expect("the check runs");
        assert!(!outcome.is_canonical);
        assert_ne!(outcome.canonical, MINIMAL);
        // Check mode reports; it does not write. The input is a `&str`, so this is
        // structural, and the canonical form is handed back for the caller to use.
        assert_eq!(outcome.canonical, formatted(MINIMAL));
    }

    /// Mixed line endings converge, which is the cross-platform requirement.
    #[test]
    fn mixed_line_endings_normalize_to_one_form() {
        let crlf = MINIMAL.replace('\n', "\r\n");
        assert_eq!(formatted(&crlf), formatted(MINIMAL));

        let cr_only = MINIMAL.replace('\n', "\r");
        assert_eq!(formatted(&cr_only), formatted(MINIMAL));

        assert!(!formatted(&crlf).contains('\r'));
    }

    /// A CR inside preserved content is normalized rather than surviving as a
    /// stray byte, because a file with one prescribed line ending cannot contain
    /// two forms and still be canonical.
    #[test]
    fn a_carriage_return_inside_preserved_content_is_normalized() {
        let html = "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
             <body><pre>one\r\ntwo</pre></body></html>\n";
        let output = formatted(html);
        assert!(!output.contains('\r'), "a CR survived: {output:?}");
        assert!(output.contains("<pre>one\ntwo</pre>"), "{output}");
        assert_eq!(output, formatted(&output));
    }

    /// The specs' rule for an invalid document: canonicalize without altering
    /// meaning, or refuse with a reason — never silently repair. A document that
    /// is merely *invalid* has one meaning, so it is canonicalized.
    #[test]
    fn an_invalid_but_unambiguous_document_is_formatted_without_repair() {
        // Two units share an identifier and a requirement has no scenario. Both
        // fail validation; neither makes the markup ambiguous.
        let invalid = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body>\
             <section class=\"requirement\" id=\"req-1\"><h2>A</h2><p>SHALL a.</p></section>\
             <section class=\"requirement\" id=\"req-1\"><h2>B</h2><p>SHALL b.</p></section>\
             </body></html>\n";
        let output = formatted(invalid);
        // Nothing was repaired: both units, both identifiers, both statements
        // survive, and the document is still invalid.
        assert_eq!(output.matches(r#"id="req-1""#).count(), 2, "{output}");
        assert!(output.contains("SHALL a."), "{output}");
        assert!(output.contains("SHALL b."), "{output}");
        assert_eq!(output, formatted(&output));
    }

    /// An ambiguous document is refused, and the refusal says why.
    #[test]
    fn an_ambiguous_document_is_refused_with_a_reason() {
        // Misnested tags: the parser must guess, so there is no single meaning.
        let ambiguous = "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
             <body><p><strong>text</p></strong></body></html>\n";
        let error = format(ambiguous).expect_err("an ambiguous document is refused");
        let FormatError::Ambiguous { recovered } = &error;
        assert!(!recovered.is_empty(), "the refusal carried no evidence");

        let reported = error.to_string();
        assert!(reported.contains("refusing to format"), "{reported}");
        assert!(reported.contains("no single meaning"), "{reported}");
        // check() refuses for the same reason rather than reporting a verdict.
        assert_eq!(check(ambiguous).expect_err("check refuses too"), error);
    }

    /// Semantic preservation, asserted through the projection rather than by
    /// eyeballing the markup: the model of a formatted document equals the model
    /// of the document it came from.
    #[test]
    fn the_projected_model_survives_formatting() {
        let source = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>Subject</title></head><body>\
             <header><p>Why this exists.</p></header>\
             <section data-change=\"added\" id=\"req-1\" class=\"requirement\"><h2>First</h2>\
             <p>The system SHALL do the thing.</p>\
             <article class=\"scenario\" id=\"scenario-1\"><h3>Works</h3>\
             <dl><dt>When</dt><dd>it runs</dd><dt>Then</dt><dd>it works</dd></dl>\
             </article></section>\
             <li class=\"task\" id=\"task-1\" data-state=\"done\" data-refs=\"req-1\">\
             <h2>Do it</h2></li>\
             </body></html>\n";
        let before = htmlspec_parse::project(&htmlspec_parse::parse_document(source).document);
        let after_source = formatted(source);
        let after =
            htmlspec_parse::project(&htmlspec_parse::parse_document(&after_source).document);

        assert_eq!(
            serde_json::to_value(&before.model).expect("the model serializes"),
            serde_json::to_value(&after.model).expect("the model serializes"),
            "formatting changed the projected model:\n{after_source}"
        );
        assert_eq!(before.faults, after.faults);
    }

    /// Every hand-authored dogfood document formats, is a fixed point, and keeps
    /// its model. These are the documents the contract was tested against, so they
    /// exercise arrangements the unit samples above do not.
    #[test]
    fn every_dogfood_document_formats_idempotently_and_keeps_its_model() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dogfood");
        let mut seen = 0;
        for entry in std::fs::read_dir(&dir).expect("the dogfood fixtures exist") {
            let path = entry.expect("the entry is readable").path();
            if path.extension().is_none_or(|e| e != "html") {
                continue;
            }
            seen += 1;
            let name = path.file_name().expect("named file").to_string_lossy();
            let source = std::fs::read_to_string(&path).expect("the fixture is readable");

            let once = format(&source).unwrap_or_else(|e| panic!("{name} did not format: {e}"));
            let twice =
                format(&once).unwrap_or_else(|e| panic!("{name} refused its own output: {e}"));
            assert_eq!(once, twice, "{name} is not a fixed point");

            let before = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);
            let after = htmlspec_parse::project(&htmlspec_parse::parse_document(&once).document);
            assert_eq!(
                serde_json::to_value(&before.model).expect("the model serializes"),
                serde_json::to_value(&after.model).expect("the model serializes"),
                "{name} lost model content in formatting"
            );

            for (number, line) in once.lines().enumerate() {
                if line.trim_end() == line {
                    continue;
                }
                // Trailing whitespace is only permitted inside preserved content,
                // where it is content. Anything else is a formatter defect.
                assert!(
                    is_inside_preserved(&once, number),
                    "{name}:{} has trailing whitespace outside preserved content",
                    number + 1
                );
            }
        }
        assert!(seen >= 3, "expected the six dogfood documents, saw {seen}");
    }

    /// Whether the given line index falls inside a `pre` or `textarea` block.
    ///
    /// A crude scan over the formatted output, which is enough because the
    /// formatter writes each opening tag at the start of a line's content.
    fn is_inside_preserved(text: &str, line_index: usize) -> bool {
        let mut depth = 0usize;
        for (number, line) in text.lines().enumerate() {
            if number == line_index {
                return depth > 0;
            }
            for (open, close) in [("<pre", "</pre>"), ("<textarea", "</textarea>")] {
                depth += line.matches(open).count();
                depth = depth.saturating_sub(line.matches(close).count());
            }
        }
        false
    }
}
