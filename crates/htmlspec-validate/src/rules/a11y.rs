//! `a11y/*`: the document is readable with scripting disabled.
//!
//! One idea, two rules. Semantic content must be present in the *served markup*,
//! and a figure must carry something a reader can read in place of the rendered
//! output. Content that only appears after scripts run is treated as absent,
//! because a reviewer with scripting disabled — or reading a saved copy, or a
//! diff — sees exactly the markup and nothing else.
//!
//! The figure rule is why a visualization source is ordinary visible content
//! rather than a `<script type="application/json">` block. Such a block is
//! invisible without JavaScript, so a document using one would fail the
//! readability rule while appearing to satisfy it. The fallback is the default
//! state of the markup rather than a `<noscript>` branch: one thing to get right
//! instead of two that can disagree.

use htmlspec_parse::model::FigureKind;
use htmlspec_parse::Projection;
use kuchikiki::iter::NodeIterator;
use kuchikiki::{NodeData, NodeRef};

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};

/// A semantic unit's content is supplied by a script rather than by the markup.
pub const SCRIPT_SUPPLIED_CONTENT: RuleId = RuleId::new(Namespace::A11y, "script-supplied-content");

/// A figure has no caption.
pub const FIGURE_WITHOUT_CAPTION: RuleId = RuleId::new(Namespace::A11y, "figure-without-caption");

/// A figure offers nothing to read in place of its rendered output.
pub const FIGURE_WITHOUT_FALLBACK: RuleId = RuleId::new(Namespace::A11y, "figure-without-fallback");

/// A visualization source is hidden in a script element.
pub const HIDDEN_FIGURE_SOURCE: RuleId = RuleId::new(Namespace::A11y, "hidden-figure-source");

/// Runs every no-JavaScript readability rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    figures_are_readable(projection, diagnostics);

    let Some(document) = projection.document.as_ref() else {
        return;
    };
    content_is_in_the_markup(projection, document, diagnostics);
    sources_are_visible(document, diagnostics);
}

/// Every figure carries a caption and something to read.
fn figures_are_readable(projection: &Projection, diagnostics: &mut Diagnostics) {
    for figure in &projection.model.figures {
        let location = Location::maybe_unit(figure.id.as_deref());

        if figure.caption.is_none() {
            diagnostics.violation(
                FIGURE_WITHOUT_CAPTION,
                location.clone(),
                "a figure has no caption, so a reader who cannot see the rendered output \
                 learns nothing about what it shows",
            );
        }

        // A visualization needs its source present. A `math` figure is native
        // MathML, which renders with no script at all, and a `prose` figure is
        // already ordinary content — neither has a source to fall back to.
        let needs_source = matches!(figure.kind, FigureKind::Mermaid | FigureKind::VegaLite);
        let has_readable_source = figure
            .source
            .as_deref()
            .is_some_and(|source| !source.trim().is_empty());
        if needs_source && !has_readable_source {
            diagnostics.violation(
                FIGURE_WITHOUT_FALLBACK,
                location,
                format!(
                    "this {} figure has no source in the markup. With scripting disabled a \
                     reader sees an empty region rather than the diagram, so the source must \
                     be present as visible content",
                    kind_name(figure.kind)
                ),
            );
        }
    }
}

/// No semantic unit depends on a script for its content.
///
/// A unit whose element holds no text of its own — outside `script`, `style`,
/// `template`, and `noscript`, none of which the projection reads — has nothing
/// for a reader with scripting disabled. The check is on the *markup*, so an
/// element a script would later fill is empty here and reported.
fn content_is_in_the_markup(
    projection: &Projection,
    document: &NodeRef,
    diagnostics: &mut Diagnostics,
) {
    for unit in projection.model.units() {
        let Some(id) = unit.id else {
            continue;
        };
        let Some(element) = element_with_id(document, id) else {
            continue;
        };
        if has_readable_text(&element) {
            continue;
        }
        // A unit with a title already proved it has readable text, so reaching
        // here means the markup is genuinely empty. The schema reports the
        // missing title; this rule reports why the page is blank.
        diagnostics.violation(
            SCRIPT_SUPPLIED_CONTENT,
            Location::unit(id),
            format!(
                "this {} contains no text a reader can see. Content that appears only after \
                 scripts run is absent: a reviewer with scripting disabled, or reading a \
                 saved copy, sees this unit empty",
                unit.kind
            ),
        );
    }
}

/// No visualization source is carried in a script element.
///
/// This is the conventional way to embed a chart specification, and it is
/// rejected: the block is invisible without JavaScript, so a document using one
/// fails the readability rule while looking like it satisfies it.
fn sources_are_visible(document: &NodeRef, diagnostics: &mut Diagnostics) {
    for figure in document
        .inclusive_descendants()
        .elements()
        .filter(|e| e.name.local.as_ref().eq_ignore_ascii_case("figure"))
    {
        for script in figure
            .as_node()
            .inclusive_descendants()
            .elements()
            .filter(|e| e.name.local.as_ref().eq_ignore_ascii_case("script"))
        {
            let kind = attribute(script.as_node(), "type").unwrap_or_default();
            diagnostics.violation(
                HIDDEN_FIGURE_SOURCE,
                Location::maybe_unit(attribute(figure.as_node(), "id").as_deref()),
                format!(
                    "this figure carries content in a <script type=\"{kind}\"> block, which is \
                     invisible without JavaScript. A visualization source is ordinary visible \
                     content: put it in a <pre>, inside a <details> if it should start collapsed"
                ),
            );
        }
    }
}

/// The element declaring one identifier.
fn element_with_id(document: &NodeRef, id: &str) -> Option<NodeRef> {
    document
        .inclusive_descendants()
        .elements()
        .find(|element| attribute(element.as_node(), "id").as_deref() == Some(id))
        .map(|element| element.as_node().clone())
}

/// Whether an element holds text a reader can see.
///
/// `script`, `style`, `template`, and `noscript` contribute nothing, matching the
/// projection's own rule (section 8.5). An `img` with alt text counts: it is
/// content present in the markup.
fn has_readable_text(node: &NodeRef) -> bool {
    if let Some(element) = node.as_element() {
        let name = element.name.local.as_ref().to_lowercase();
        if matches!(name.as_str(), "script" | "style" | "template" | "noscript") {
            return false;
        }
        if name == "img" && attribute(node, "alt").is_some_and(|alt| !alt.trim().is_empty()) {
            return true;
        }
    }
    match node.data() {
        NodeData::Text(text) => !text.borrow().trim().is_empty(),
        _ => node.children().any(|child| has_readable_text(&child)),
    }
}

/// One attribute's value, trimmed.
fn attribute(node: &NodeRef, name: &str) -> Option<String> {
    let element = node.as_element()?;
    let attributes = element.attributes.borrow();
    attributes.get(name).map(|value| value.trim().to_owned())
}

/// The figure kind's name, for a diagnostic a reader can act on.
fn kind_name(kind: FigureKind) -> &'static str {
    match kind {
        FigureKind::Mermaid => "Mermaid",
        FigureKind::VegaLite => "Vega-Lite",
        FigureKind::Math => "MathML",
        FigureKind::Code => "code",
        FigureKind::Prose => "prose",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};

    /// These rules are not asset rules, so they run with no store beside them.
    const NO_STORE: htmlspec_assets::Store = htmlspec_assets::Store::Absent;
    use htmlspec_parse::{parse_document, project};

    fn check(body: &str) -> crate::Report {
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>Fixture</title></head>\n<body>\n{body}\n</body>\n</html>\n"
        );
        let outcome = parse_document(&html);
        assert!(
            outcome.is_recovery_free(),
            "the fixture itself recovered: {:?}",
            outcome.recovered
        );
        let projection = project(&outcome.document);
        run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[validate],
            Strictness::Default,
        )
    }

    fn fired(report: &crate::Report, rule: RuleId) -> Vec<&crate::Diagnostic> {
        report
            .diagnostics
            .iter()
            .filter(|d| d.rule == rule)
            .collect()
    }

    fn messages(report: &crate::Report) -> String {
        report
            .diagnostics
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_readable_document_reports_nothing() {
        let report = check(
            "<section class=\"requirement\" id=\"req-x\"><h2>A requirement</h2>\
             <p>The system MUST work.</p>\
             <article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_unit_whose_content_a_script_would_supply_is_reported() {
        let report = check("<section class=\"task\" id=\"task-x\" data-state=\"todo\"></section>");
        let found = fired(&report, SCRIPT_SUPPLIED_CONTENT);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(found[0].location.unit_id.as_deref(), Some("task-x"));
    }

    /// A script inside the unit does not make it readable, which is the whole
    /// point: the text has to be in the markup.
    #[test]
    fn text_inside_a_script_does_not_make_a_unit_readable() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\">\
             <script src=\"assets/htmlspec.js\">fill('task-x', 'A title')</script>\
             </section>",
        );
        assert_eq!(
            fired(&report, SCRIPT_SUPPLIED_CONTENT).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// A `noscript` branch is not the fallback this project uses. Text inside one
    /// is absent for a reader *with* scripting, so it does not count either.
    #[test]
    fn text_inside_a_noscript_does_not_make_a_unit_readable() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\">\
             <noscript><h2>A title</h2></noscript></section>",
        );
        assert_eq!(
            fired(&report, SCRIPT_SUPPLIED_CONTENT).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_figure_with_a_caption_and_a_visible_source_is_accepted() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A diagram of the pipeline.</figcaption>\
             <details><summary>Diagram source</summary>\
             <pre class=\"mermaid\">graph LR\n  A --&gt; B</pre></details></figure>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_figure_with_no_caption_is_reported() {
        let report =
            check("<figure id=\"figure-x\"><pre class=\"mermaid\">graph LR</pre></figure>");
        assert_eq!(
            fired(&report, FIGURE_WITHOUT_CAPTION).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_visualization_figure_with_an_empty_source_is_reported() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A diagram.</figcaption>\
             <pre class=\"mermaid\">   </pre></figure>",
        );
        assert_eq!(
            fired(&report, FIGURE_WITHOUT_FALLBACK).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// Native MathML renders with no script, so it needs no source fallback.
    #[test]
    fn a_mathml_figure_needs_no_source_fallback() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>The contrast threshold.</figcaption>\
             <math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mn>4.5</mn></math></figure>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_prose_figure_needs_no_source_fallback() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A table of measurements.</figcaption>\
             <table><tr><td>1</td></tr></table></figure>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// The conventional embedding, and the reason the contract rejects it: the
    /// block satisfies "the source is present" while being invisible.
    #[test]
    fn a_source_hidden_in_a_script_block_is_reported() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A chart.</figcaption>\
             <script type=\"application/json\">{\"mark\": \"bar\"}</script></figure>",
        );
        let found = fired(&report, HIDDEN_FIGURE_SOURCE);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains("invisible"),
            "{}",
            found[0].message
        );
        // And the figure is also reported as having no fallback, because the
        // projection reads no source from a script block.
        assert_eq!(
            fired(&report, FIGURE_WITHOUT_FALLBACK).len(),
            0,
            "a script-carried source projects as a prose figure, so the fallback rule \
             does not also fire: {}",
            messages(&report)
        );
    }

    /// A page saved from a rendered DOM keeps its source, so it stays readable.
    /// This is the arrangement the sibling-rendering decision exists to produce.
    #[test]
    fn a_saved_rendered_page_is_still_readable() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A diagram, as saved.</figcaption>\
             <div class=\"mermaid-rendered\"><svg xmlns=\"http://www.w3.org/2000/svg\">\
             <title>Rendered</title></svg></div>\
             <details><summary>Diagram source</summary>\
             <pre class=\"mermaid\">graph LR\n  A --&gt; B</pre></details></figure>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn an_image_with_alt_text_is_readable_content() {
        let report = check(
            "<section class=\"evidence\" id=\"evidence-x\">\
             <img src=\"assets/plot.png\" alt=\"A plot of the measured latency\"></section>",
        );
        assert!(
            fired(&report, SCRIPT_SUPPLIED_CONTENT).is_empty(),
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn every_dogfood_document_is_readable_without_scripting() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dogfood");
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .expect("the dogfood directory is readable")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "html"))
            .collect();
        paths.sort();
        assert_eq!(paths.len(), 6);

        for path in paths {
            let html = std::fs::read_to_string(&path).expect("readable");
            let outcome = parse_document(&html);
            let projection = project(&outcome.document);
            let report = run(
                &outcome,
                &projection,
                &NO_STORE,
                &crate::references::Scope::loose(),
                &[validate],
                Strictness::Default,
            );
            assert!(
                report.is_accepted(),
                "{path:?} is not readable without scripting:\n{}",
                messages(&report)
            );
        }
    }
}
