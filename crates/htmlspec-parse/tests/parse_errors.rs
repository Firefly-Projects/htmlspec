//! Pins the task 1.5 enumeration against the fixture corpus.
//!
//! Each fixture states the codes the pinned parser reports for it. A parser
//! upgrade that reclassifies a recovery fails here.

use std::path::{Path, PathBuf};

use htmlspec_parse::{parse_document, ParseDiagnosticCode, REVIEWED_HTML5EVER_VERSION};

fn fixture(name: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parse-errors")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"))
}

fn codes(name: &str) -> Vec<ParseDiagnosticCode> {
    parse_document(&fixture(name))
        .recovered
        .into_iter()
        .map(|e| e.code)
        .collect()
}

/// Task 4.2's tripwire: the enumeration is only correct for the reviewed
/// parser version, so a bump must be a deliberate act.
#[test]
fn parser_version_is_the_reviewed_one() {
    // The pin in the workspace manifest and the version the lockfile resolved
    // must both equal the version the enumeration was reviewed against. A bump
    // to either one fails here, which forces a review of the enumeration.
    let message = "html5ever moved to a version the parse-error enumeration has not \
                   been reviewed against; re-run the task 1.5 spike and update \
                   openspec/changes/html-spec/evidence/parse-errors.md";
    assert_eq!(
        env!("HTMLSPEC_PINNED_HTML5EVER_VERSION"),
        REVIEWED_HTML5EVER_VERSION,
        "{message}"
    );
    assert_eq!(
        env!("HTMLSPEC_LOCKED_HTML5EVER_VERSION"),
        REVIEWED_HTML5EVER_VERSION,
        "{message}"
    );
}

#[test]
fn a_conforming_document_produces_no_recovery() {
    let outcome = parse_document(&fixture("benign-minimal.html"));
    assert!(
        outcome.is_recovery_free(),
        "conforming document recovered: {:?}",
        outcome.recovered
    );
    assert!(!outcome.quirks);
}

#[test]
fn a_missing_doctype_is_a_recovery_and_sets_quirks_mode() {
    let outcome = parse_document(&fixture("no-doctype.html"));
    assert_eq!(
        outcome.recovered.iter().map(|e| e.code).collect::<Vec<_>>(),
        vec![ParseDiagnosticCode::UnexpectedToken]
    );
    assert!(outcome.quirks);
}

/// A legacy doctype is invisible to the parse-error rule, which is why the
/// short-doctype requirement needs a rule of our own.
#[test]
fn a_legacy_doctype_produces_no_recovery_and_no_quirks() {
    let outcome = parse_document(&fixture("bad-doctype.html"));
    assert!(outcome.is_recovery_free(), "{:?}", outcome.recovered);
    assert!(!outcome.quirks);
}

#[test]
fn exact_errors_carries_the_insertion_mode() {
    let outcome = parse_document(&fixture("no-doctype.html"));
    let message = &outcome.recovered[0].message;
    assert!(
        message.contains("in insertion mode Initial"),
        "message lost its parser state: {message}"
    );
}

#[test]
fn an_escaping_code_example_is_a_recovery() {
    let codes = codes("code-example-escapes.html");
    assert!(
        codes.contains(&ParseDiagnosticCode::UnexpectedOpenElement),
        "{codes:?}"
    );
    assert!(
        codes.contains(&ParseDiagnosticCode::NoTagToClose),
        "{codes:?}"
    );
}

#[test]
fn each_damaged_fixture_maps_to_its_enumerated_code() {
    let cases: &[(&str, ParseDiagnosticCode)] = &[
        (
            "bare-ampersand.html",
            ParseDiagnosticCode::CharacterReference,
        ),
        (
            "duplicate-attribute.html",
            ParseDiagnosticCode::DuplicateAttribute,
        ),
        (
            "misnested-tags.html",
            ParseDiagnosticCode::FormattingElement,
        ),
        (
            "stray-end-tag.html",
            ParseDiagnosticCode::SpecialTagInGenericClose,
        ),
        ("table-text.html", ParseDiagnosticCode::TableText),
        (
            "unclosed-element.html",
            ParseDiagnosticCode::UnexpectedOpenTagAtEof,
        ),
    ];
    for (name, expected) in cases {
        assert_eq!(codes(name), vec![*expected], "fixture {name}");
    }
}

/// No fixture may produce the residual code. A recovery this project has not
/// enumerated is a contract gap, not a document defect.
#[test]
fn no_fixture_produces_an_unmapped_recovery() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parse-errors");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .expect("fixture directory is readable")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".html"))
        .collect();
    names.sort();
    assert!(!names.is_empty(), "corpus is empty");
    for name in names {
        let unmapped: Vec<_> = parse_document(&fixture(&name))
            .recovered
            .into_iter()
            .filter(|e| e.code == ParseDiagnosticCode::Unmapped)
            .collect();
        assert!(
            unmapped.is_empty(),
            "fixture {name} produced messages the enumeration does not name: {unmapped:?}"
        );
    }
}
