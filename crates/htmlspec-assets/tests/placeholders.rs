//! Task 3.4: the scanned placeholder is the summary the projection produces.
//!
//! This crate scans a template's masthead for its placeholder paragraph and normalizes it, and
//! `htmlspec-parse` projects a document's `summary` from the same paragraphs and normalizes it
//! too. **Those are two implementations of one rule** — projection specification section 6.1 —
//! and they have to be, because `htmlspec-parse` depends on this crate rather than the other way
//! round and its `normalized_text` is private.
//!
//! Two implementations of one rule drift. What makes this drift worse than most is that it is
//! silent: the check compares a scanned string against a projected one, so if the two
//! normalizations disagree by a single space the comparison stops matching and reports nothing.
//! A rule that silently stops matching is worse than one that never matched, because the first
//! run is what convinces everybody it works.
//!
//! So this asserts the two **agree** rather than asserting the rule fires. Every other test in
//! this change would keep passing through that drift: the validator's tests build their
//! documents from the same scanned set, so both sides would move together.
//!
//! Driven from `TEMPLATES` rather than from four kind names, so a template added to the set is
//! covered with no edit here.

use htmlspec_parse::{parse_document, project};

/// The relative asset path a document beside the project's `assets/` directory would use.
///
/// Any value works — the summary does not read it — but `render_kind` substitutes it and a
/// document with an unresolved `{{ASSETS}}` is not the document `htmlspec new document` writes.
const ASSETS: &str = "assets";

/// Every template's placeholder, projected out of the document that template produces.
#[test]
fn the_scanned_placeholder_is_the_projected_summary_of_a_new_document() {
    for template in htmlspec_assets::TEMPLATES {
        let subject = format!("A fresh {}", template.kind);
        let rendered = htmlspec_assets::render_kind(template.kind, &subject, ASSETS)
            .unwrap_or_else(|| panic!("no template produces the {} kind", template.kind));

        let outcome = parse_document(&rendered);
        assert!(
            outcome.is_recovery_free(),
            "the {} template does not parse cleanly, so this test would compare against a \
             recovered document: {:?}",
            template.kind,
            outcome.recovered
        );
        let projection = project(&outcome.document);
        let summary = projection.model.summary;

        assert_eq!(
            summary.len(),
            1,
            "a freshly created {} projects {} summary paragraph(s). The scan yields one per \
             template, so any other count means the two read different elements",
            template.kind,
            summary.len()
        );

        // The assertion this test exists for. `is_example_placeholder` is the function the
        // validator calls, so this pins the exact path the rule takes rather than a
        // reimplementation of it.
        assert!(
            htmlspec_assets::is_example_placeholder(&summary[0]),
            "the projected summary of a freshly created {} is not recognized as its template's \
             placeholder, so the rule is silently inert for that kind.\n  projected: {:?}\n  \
             scanned:   {:?}",
            template.kind,
            summary[0],
            htmlspec_assets::example_placeholders(),
        );

        // And the scanned set holds that exact string, which is the stronger statement:
        // `is_example_placeholder` normalizes its argument, so it would still pass if the
        // scanned side carried whitespace the projected side does not.
        assert!(
            htmlspec_assets::example_placeholders().contains(&summary[0]),
            "the {} template's scanned placeholder differs from its projected summary by \
             whitespace. The membership test normalizes and hides it, and a caller comparing \
             the raw strings would silently match nothing.\n  projected: {:?}\n  scanned:   \
             {:?}",
            template.kind,
            summary[0],
            htmlspec_assets::example_placeholders(),
        );
    }
}

/// An inline element in a masthead paragraph projects the same text the scan reads.
///
/// The scan drops tags and keeps the characters between them; the projection walks text nodes.
/// Neither inserts a separator at a tag boundary, so `<em>paragraph</em>.` is one word — which
/// I first asserted wrongly in this crate's unit tests by writing the expectation by hand. This
/// asserts it against the projection instead, so the answer comes from the implementation that
/// defines it.
///
/// No template's masthead carries inline markup today. The test matters because one might: a
/// template that gained a `<code>` in its masthead would go quietly unmatched if the two
/// disagreed here, and this is what would report it.
#[test]
fn an_inline_element_in_a_masthead_projects_the_same_text() {
    let html = "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
                <meta name=\"htmlspec-version\" content=\"0.1.0\">\n\
                <meta name=\"htmlspec-kind\" content=\"proposal\">\n\
                <title>Inline</title>\n</head>\n<body>\n\
                <header class=\"hs-masthead\"><h1>Inline</h1>\n\
                <p>Replace <code>this</code>\n  <em>paragraph</em>.</p></header>\n\
                <main><p>A body paragraph.</p></main>\n</body>\n</html>\n";
    let outcome = parse_document(html);
    assert!(outcome.is_recovery_free(), "{:?}", outcome.recovered);
    let projected = project(&outcome.document).model.summary;

    assert_eq!(
        projected,
        vec!["Replace this paragraph.".to_owned()],
        "the projection's own answer for a masthead carrying inline markup changed, so the \
         scan's expectation in htmlspec-assets is now stating something the model does not"
    );
}
