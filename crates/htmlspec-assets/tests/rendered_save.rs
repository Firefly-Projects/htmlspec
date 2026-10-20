//! Task 6.7: a figure's source survives a save of the rendered page.
//!
//! This is the failure the design confirmed on its own output: the file on disk kept
//! its diagram source, and a copy saved from a live page did not, because Mermaid's
//! default behavior replaces the source block with rendered SVG. A saved page is
//! exactly how these artifacts get shared, so the defect destroys the source in the
//! copy that circulates.
//!
//! There is no browser in this test suite, so what is asserted is the property on a
//! *saved DOM* rather than the act of saving. The DOM below is constructed the way
//! the enhancement script constructs it — a rendering element inserted as a sibling,
//! the source's disclosure closed — and then projected. If the source did not
//! survive, the projection of the saved copy would differ from the projection of the
//! file on disk, and that is what fails here.
//!
//! `the_enhancement_script_renders_into_a_sibling_element` in the crate's unit tests
//! asserts the script's side of the same property: no destructive write to a source
//! element, and Mermaid's own scan disabled.

use htmlspec_parse::{parse_document, project};

/// The diagram source used throughout, kept in one place so the assertions compare
/// against the same bytes the document declares.
const DIAGRAM: &str = "graph LR\n  A[Authored HTML] --&gt; B[Parse]\n  B --&gt; C[Model]";

/// The document as it sits on disk: caption, and source visible in an open
/// disclosure. This is the state a reader with scripting disabled sees.
fn on_disk() -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"proposal\">\
         <title>Saved render</title></head>\n<body>\
         <figure id=\"figure-pipeline\">\
         <figcaption>The toolchain pipeline.</figcaption>\
         <details open><summary>Diagram source</summary>\
         <pre class=\"mermaid\">{DIAGRAM}</pre>\
         </details></figure>\
         </body>\n</html>\n"
    )
}

/// The same document as saved from a live page, rendered correctly.
///
/// The rendering is a sibling inserted as the figure's first child, and the source's
/// disclosure is closed rather than emptied — which is what the enhancement script
/// does.
fn saved_correctly() -> String {
    on_disk()
        .replace(
            "<figcaption>",
            "<div class=\"hs-figure-render\"><svg><g><text>rendered</text></g></svg></div>\
             <figcaption>",
        )
        .replace("<details open>", "<details>")
}

/// The same document as saved from a live page by a renderer that replaced the
/// source. This is the defect, reproduced so the test can show it is caught.
fn saved_destructively() -> String {
    on_disk().replace(
        &format!("<pre class=\"mermaid\">{DIAGRAM}</pre>"),
        "<svg><g><text>rendered</text></g></svg>",
    )
}

/// The figure entries of a document's projected model.
fn figures(source: &str) -> Vec<(Option<String>, String, Option<String>)> {
    let outcome = parse_document(source);
    assert!(
        outcome.is_recovery_free(),
        "the sample did not parse cleanly: {:#?}",
        outcome.recovered
    );
    project(&outcome.document)
        .model
        .figures
        .into_iter()
        .map(|figure| (figure.id, figure.caption.unwrap_or_default(), figure.source))
        .collect()
}

/// The property: a page saved from a rendered DOM projects the same model as the
/// file on disk, because the source is still there.
#[test]
fn a_saved_rendered_page_projects_the_same_model_as_the_file_on_disk() {
    assert_eq!(
        figures(&on_disk()),
        figures(&saved_correctly()),
        "the saved copy of a rendered page lost figure content"
    );
}

/// The diagram source is present in the saved markup, stated directly rather than
/// only through the model, so a reader of this test can see what is being claimed.
#[test]
fn the_diagram_source_is_still_present_in_the_saved_markup() {
    let saved = saved_correctly();
    assert!(
        saved.contains(&format!("<pre class=\"mermaid\">{DIAGRAM}</pre>")),
        "the diagram source is not in the saved page:\n{saved}"
    );
    // And the rendering is there too: the point is that both survive, not that
    // rendering was skipped.
    assert!(saved.contains("hs-figure-render"), "{saved}");
}

/// The destructive save is caught. Without this, the test above would pass for a
/// document that had no figure at all, and the suite would be asserting nothing.
#[test]
fn a_renderer_that_replaced_the_source_is_detected() {
    let destroyed = figures(&saved_destructively());
    let intact = figures(&on_disk());
    assert_ne!(
        intact, destroyed,
        "a save that replaced the diagram source went undetected, so this suite would not \
         catch the defect it exists for"
    );
    // Specifically: the source is gone and the kind fell back, because there is no
    // longer a `pre` for the projection to read (projection specification, 7.3).
    assert_eq!(destroyed.len(), 1);
    assert!(
        destroyed[0].2.is_none(),
        "the destroyed copy still reports a source: {destroyed:#?}"
    );
}

/// The collapsed disclosure is a presentational change, so closing it must not move
/// the model. If it did, a reader with scripting enabled and one without would be
/// looking at different documents.
#[test]
fn collapsing_the_source_does_not_change_the_model() {
    let open = on_disk();
    let closed = open.replace("<details open>", "<details>");
    assert_eq!(figures(&open), figures(&closed));
}
