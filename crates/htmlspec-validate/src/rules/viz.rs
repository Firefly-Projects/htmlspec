//! `viz/*`: an embedded chart or diagram is checked against its own format.
//!
//! The point is that an invalid specification is reported at authoring time
//! rather than failing silently in a browser, where the reader sees an empty
//! region and no explanation.
//!
//! The two formats get very different treatment, and the asymmetry is worth being
//! plain about. A Vega-Lite specification is validated against Vega-Lite's own
//! published schema, vendored and pinned — real validation, not a syntax check.
//! Mermaid has no Rust implementation of its grammar, so its source cannot be
//! fully validated without running Mermaid in a browser engine. This checks that
//! a diagram is non-empty and opens with a recognized keyword, and it attaches
//! that limitation to the diagnostic rather than leaving it in prose
//! documentation, because a shallow check that reports a pass has told the reader
//! less than it appears to.

use std::sync::OnceLock;

use htmlspec_parse::model::{Figure, FigureKind};
use jsonschema::Validator;
use serde_json::Value;

use crate::diagnostic::{Diagnostic, Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};

/// A chart specification is not valid JSON.
pub const CHART_NOT_JSON: RuleId = RuleId::new(Namespace::Viz, "chart-not-json");

/// A chart specification violates Vega-Lite's own schema.
pub const CHART_INVALID: RuleId = RuleId::new(Namespace::Viz, "chart-invalid");

/// A diagram source is empty.
pub const DIAGRAM_EMPTY: RuleId = RuleId::new(Namespace::Viz, "diagram-empty");

/// A diagram source does not open with a recognized diagram keyword.
pub const DIAGRAM_UNRECOGNIZED: RuleId = RuleId::new(Namespace::Viz, "diagram-unrecognized");

/// Vega-Lite's schema could not be compiled, so no chart can be checked.
///
/// A defect in this project rather than in the document, and an error because a
/// validator that cannot compile its contract must not report a chart as clean.
pub const SCHEMA_UNAVAILABLE: RuleId = RuleId::new(Namespace::Viz, "schema-unavailable");

/// The limitation every Mermaid diagnostic carries.
///
/// Recorded as a diagnostic note rather than only in documentation: a reader who
/// sees a diagram pass this check should know how little that establishes.
const MERMAID_LIMITATION: &str = "Mermaid diagrams are validated shallowly: this project checks \
     that the source is non-empty and opens with a recognized diagram keyword. No Rust \
     implementation of the Mermaid grammar exists, so a syntactically invalid diagram body \
     still reaches the browser. The visible source limits the damage — a broken diagram shows \
     its caption and its source rather than an empty region.";

/// The diagram keywords the pinned Mermaid recognizes.
///
/// Matched against the first meaningful token of the source. The list belongs to
/// the pinned library rather than to the contract, which is why it lives here and
/// not in the schema: a Mermaid upgrade can add a diagram type, and
/// `assets/vendor.lock.json` records the version this list was built against.
const DIAGRAM_KEYWORDS: &[&str] = &[
    "architecture-beta",
    "block-beta",
    "block",
    "C4Context",
    "C4Container",
    "C4Component",
    "C4Dynamic",
    "C4Deployment",
    "classDiagram-v2",
    "classDiagram",
    "erDiagram",
    "flowchart-elk",
    "flowchart",
    "gantt",
    "gitGraph",
    "graph",
    "info",
    "journey",
    "kanban",
    "mindmap",
    "packet-beta",
    "packet",
    "pie",
    "quadrantChart",
    "radar-beta",
    "requirementDiagram",
    "sankey-beta",
    "sequenceDiagram",
    "stateDiagram-v2",
    "stateDiagram",
    "timeline",
    "treemap-beta",
    "xychart-beta",
    "zenuml",
];

/// Vega-Lite's published JSON Schema, vendored and pinned.
///
/// Read from the embedded asset table by file name rather than by `include_str!` at a
/// literal path. The schema now lives at its content address, so a literal path here
/// would name a directory that changes whenever the bytes do — and it would be a second
/// place an address lives, which is the arrangement this change exists to remove. The
/// bytes are the same bytes `init` writes, verified against the lockfile by
/// `every_committed_asset_matches_the_lockfile`.
fn vega_lite_schema() -> &'static str {
    static SCHEMA: OnceLock<&'static str> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let asset = htmlspec_assets::all_assets()
            .into_iter()
            .find(|asset| asset.name() == "vega-lite-schema.json")
            .expect("the Vega-Lite schema is embedded");
        std::str::from_utf8(asset.bytes).expect("the schema is UTF-8")
    })
}

/// Runs every visualization rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    for figure in &projection.model.figures {
        match figure.kind {
            FigureKind::VegaLite => validate_chart(figure, diagnostics),
            FigureKind::Mermaid => validate_diagram(figure, diagnostics),
            // A code excerpt, an equation, and a prose figure have no
            // specification to validate.
            FigureKind::Code | FigureKind::Math | FigureKind::Prose => {}
        }
    }
}

/// Validates one chart specification against Vega-Lite's schema.
fn validate_chart(figure: &Figure, diagnostics: &mut Diagnostics) {
    let location = Location::maybe_unit(figure.id.as_deref());
    // An absent source is the a11y rules' fault to report: a figure with no
    // source is unreadable without scripting, which is the more useful thing to
    // say about it.
    let Some(source) = figure.source.as_deref() else {
        return;
    };

    let specification: Value = match serde_json::from_str(source) {
        Ok(value) => value,
        Err(e) => {
            diagnostics.violation(
                CHART_NOT_JSON,
                location,
                format!(
                    "this chart specification is not valid JSON: {e}. A browser would render \
                     an empty region with the reason only in its console"
                ),
            );
            return;
        }
    };

    let validator = match vega_lite_validator() {
        Ok(validator) => validator,
        Err(reason) => {
            diagnostics.violation(SCHEMA_UNAVAILABLE, location, reason);
            return;
        }
    };

    for error in validator.iter_errors(&specification) {
        let pointer = error.instance_path().to_string();
        let at = if pointer.is_empty() {
            "the specification root".to_owned()
        } else {
            pointer
        };
        diagnostics.violation(
            CHART_INVALID,
            location.clone(),
            format!("Vega-Lite rejects this chart at {at}: {error}"),
        );
    }
}

/// Validates one diagram source as far as this project can.
fn validate_diagram(figure: &Figure, diagnostics: &mut Diagnostics) {
    let location = Location::maybe_unit(figure.id.as_deref());
    let Some(source) = figure.source.as_deref() else {
        return;
    };

    if source.trim().is_empty() {
        diagnostics.report(
            Diagnostic::new(
                DIAGRAM_EMPTY,
                location,
                "this diagram has an empty source, so it renders nothing at all",
            )
            .with_note(MERMAID_LIMITATION),
        );
        return;
    }

    // A recognized keyword is as far as this check goes. The body is not
    // examined, which is what MERMAID_LIMITATION tells the reader.
    if leading_keyword(source).is_none() {
        diagnostics.report(
            Diagnostic::new(
                DIAGRAM_UNRECOGNIZED,
                location,
                format!(
                    "this diagram opens with {:?}, which the pinned Mermaid does not recognize \
                     as a diagram type. A browser would render an error in place of the diagram",
                    first_token(source)
                ),
            )
            .with_note(MERMAID_LIMITATION),
        );
    }
}

/// The recognized diagram keyword a source opens with, if any.
///
/// Mermaid permits directives and comments before the diagram declaration, so
/// those lines are skipped rather than treated as the opening token.
fn leading_keyword(source: &str) -> Option<&'static str> {
    let token = first_token(source);
    // Longest match first, so `stateDiagram-v2` is not matched as `stateDiagram`
    // with trailing text. The keyword may be followed by a direction or a title.
    let mut candidates: Vec<&&str> = DIAGRAM_KEYWORDS
        .iter()
        .filter(|keyword| token == **keyword)
        .collect();
    candidates.sort_by_key(|keyword| std::cmp::Reverse(keyword.len()));
    candidates.first().map(|keyword| **keyword)
}

/// The first meaningful token of a diagram source.
///
/// Skips blank lines, `%%` comments, and `%%{ ... }%%` directives. A `graph LR`
/// declaration yields `graph`, and `stateDiagram-v2` yields itself.
fn first_token(source: &str) -> String {
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }
        // A declaration may be `flowchart TD` or `pie title X` or `graph LR;`.
        let head = line
            .split(|c: char| c.is_whitespace() || c == ';' || c == ':')
            .find(|part| !part.is_empty())
            .unwrap_or(line);
        return head.to_owned();
    }
    String::new()
}

/// Vega-Lite's schema, preprocessed and compiled once per process.
fn vega_lite_validator() -> Result<&'static Validator, String> {
    static COMPILED: OnceLock<Result<Validator, String>> = OnceLock::new();
    COMPILED
        .get_or_init(|| {
            let mut schema: Value = serde_json::from_str(vega_lite_schema())
                .map_err(|e| format!("the vendored Vega-Lite schema is not valid JSON: {e}"))?;
            slugify_definition_names(&mut schema);
            jsonschema::draft7::new(&schema).map_err(|e| {
                format!(
                    "the vendored Vega-Lite schema does not compile even after preprocessing: {e}"
                )
            })
        })
        .as_ref()
        .map_err(String::clone)
}

/// Rewrites definition names, and the references to them, into legal URI references.
///
/// Vega-Lite's schema names its definitions after TypeScript types, so 126 of
/// them contain `<`, `>`, and `|` — illegal in a URI reference. Draft-07
/// metaschema validation therefore rejects the schema before any chart is
/// validated, and no `jsonschema` option bypasses it, because metaschema
/// validation runs before user options apply.
///
/// The rewrite is a pure rename: every definition and every `$ref` pointing at
/// one is slugified by the same function, so the schema means exactly what it
/// meant. `the_vendored_schema_is_rewritten_consistently` asserts no reference is
/// left dangling.
fn slugify_definition_names(schema: &mut Value) {
    if let Some(definitions) = schema.get_mut("definitions").and_then(Value::as_object_mut) {
        let renames: Vec<(String, String)> = definitions
            .keys()
            .map(|name| (name.clone(), slugify(name)))
            .filter(|(from, to)| from != to)
            .collect();
        for (from, to) in renames {
            if let Some(definition) = definitions.remove(&from) {
                definitions.insert(to, definition);
            }
        }
    }
    rewrite_refs(schema);
}

/// Applies the same rename to every local `$ref`.
fn rewrite_refs(node: &mut Value) {
    match node {
        Value::Object(map) => {
            if let Some(Value::String(reference)) = map.get_mut("$ref") {
                if let Some(name) = reference.strip_prefix("#/definitions/") {
                    *reference = format!("#/definitions/{}", slugify(name));
                }
            }
            for value in map.values_mut() {
                rewrite_refs(value);
            }
        }
        Value::Array(items) => {
            for item in items {
                rewrite_refs(item);
            }
        }
        _ => {}
    }
}

/// Replaces every character illegal in a URI reference fragment with `_`.
///
/// Deliberately not injective in principle — two type names could slugify to one
/// string — so the test below checks that this schema's names stay distinct
/// rather than assuming it.
fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
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

    fn chart(source: &str) -> crate::Report {
        check(&format!(
            "<figure id=\"figure-x\"><figcaption>A chart.</figcaption>\
             <pre class=\"vega-lite\">{source}</pre></figure>"
        ))
    }

    fn diagram(source: &str) -> crate::Report {
        check(&format!(
            "<figure id=\"figure-x\"><figcaption>A diagram.</figcaption>\
             <pre class=\"mermaid\">{source}</pre></figure>"
        ))
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

    /// The measurement the design rests on: the schema does not load unmodified,
    /// and it does load after the rename. Asserted rather than assumed, because
    /// the whole visualization rule depends on it.
    #[test]
    fn the_vendored_schema_does_not_compile_unmodified() {
        let schema: Value = serde_json::from_str(vega_lite_schema()).expect("valid JSON");
        let unmodified = jsonschema::draft7::new(&schema);
        assert!(
            unmodified.is_err(),
            "the vendored schema now compiles unmodified; the preprocessing step and the \
             design decision that justifies it should be revisited"
        );
        assert!(
            vega_lite_validator().is_ok(),
            "{:?}",
            vega_lite_validator().err()
        );
    }

    /// The rename must leave no reference dangling and must not merge two names.
    #[test]
    fn the_vendored_schema_is_rewritten_consistently() {
        let mut schema: Value = serde_json::from_str(vega_lite_schema()).expect("valid JSON");
        let before = schema["definitions"]
            .as_object()
            .expect("definitions")
            .len();
        slugify_definition_names(&mut schema);
        let definitions = schema["definitions"].as_object().expect("definitions");
        assert_eq!(
            definitions.len(),
            before,
            "two definition names slugified to the same string, so the rename lost one"
        );

        // Every local reference resolves to a definition that exists.
        let mut dangling: Vec<String> = Vec::new();
        collect_dangling(&schema, definitions, &mut dangling);
        assert!(
            dangling.is_empty(),
            "the rename left {} reference(s) dangling, first: {:?}",
            dangling.len(),
            dangling.first()
        );
    }

    fn collect_dangling(
        node: &Value,
        definitions: &serde_json::Map<String, Value>,
        found: &mut Vec<String>,
    ) {
        match node {
            Value::Object(map) => {
                if let Some(Value::String(reference)) = map.get("$ref") {
                    if let Some(name) = reference.strip_prefix("#/definitions/") {
                        if !definitions.contains_key(name) {
                            found.push(reference.clone());
                        }
                    }
                }
                for value in map.values() {
                    collect_dangling(value, definitions, found);
                }
            }
            Value::Array(items) => {
                for item in items {
                    collect_dangling(item, definitions, found);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn a_valid_chart_is_accepted() {
        let report = chart("{\"data\": {\"values\": [{\"a\": 1}]}, \"mark\": \"bar\"}");
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// Real validation, not a syntax check: this is valid JSON and an invalid
    /// chart.
    #[test]
    fn a_chart_with_an_unknown_mark_is_rejected() {
        let report = chart("{\"data\": {\"values\": []}, \"mark\": \"notamark\"}");
        assert!(
            !fired(&report, CHART_INVALID).is_empty(),
            "{}",
            messages(&report)
        );
        assert!(fired(&report, CHART_NOT_JSON).is_empty());
    }

    #[test]
    fn a_chart_that_is_not_json_is_reported_as_such() {
        let report = chart("{\"data\": {\"values\": []}, \"mark\": }");
        assert_eq!(
            fired(&report, CHART_NOT_JSON).len(),
            1,
            "{}",
            messages(&report)
        );
        // Not also reported as schema-invalid; there is nothing to validate.
        assert!(fired(&report, CHART_INVALID).is_empty());
    }

    #[test]
    fn a_chart_diagnostic_names_the_figure() {
        let report = chart("{\"data\": {\"values\": []}, \"mark\": \"notamark\"}");
        assert_eq!(
            report.diagnostics[0].location.unit_id.as_deref(),
            Some("figure-x")
        );
    }

    #[test]
    fn a_recognized_diagram_keyword_is_accepted() {
        for source in [
            "graph LR\n  A --&gt; B",
            "flowchart TD\n  A --&gt; B",
            "sequenceDiagram\n  Alice-&gt;&gt;Bob: Hi",
            "stateDiagram-v2\n  [*] --&gt; Still",
            "erDiagram\n  A ||--o{ B : has",
            "gantt\n  title A schedule",
            "pie title Distribution\n  \"A\" : 1",
        ] {
            let report = diagram(source);
            assert!(report.is_accepted(), "{source}: {}", messages(&report));
        }
    }

    #[test]
    fn an_empty_diagram_is_reported() {
        let report = diagram("   \n  ");
        assert_eq!(
            fired(&report, DIAGRAM_EMPTY).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn an_unrecognized_diagram_keyword_is_reported() {
        let report = diagram("notadiagram TD\n  A --&gt; B");
        let found = fired(&report, DIAGRAM_UNRECOGNIZED);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains("notadiagram"),
            "{}",
            found[0].message
        );
    }

    /// A shallow check that reports a pass has told the reader less than it
    /// appears to, so the limitation travels with the diagnostic.
    #[test]
    fn every_mermaid_diagnostic_carries_the_shallow_validation_note() {
        for source in ["   ", "notadiagram TD"] {
            let report = diagram(source);
            assert!(!report.diagnostics.is_empty(), "{source} reported nothing");
            for diagnostic in &report.diagnostics {
                let note = diagnostic
                    .note
                    .as_deref()
                    .unwrap_or_else(|| panic!("{source}: {} carries no note", diagnostic.rule));
                assert!(note.contains("validated shallowly"), "{note}");
            }
        }
    }

    /// Mermaid permits directives and comments before the declaration.
    #[test]
    fn a_directive_or_comment_before_the_declaration_is_skipped() {
        let report =
            diagram("%%{init: {\"theme\": \"neutral\"}}%%\n%% a comment\ngraph LR\n  A --&gt; B");
        assert!(report.is_accepted(), "{}", messages(&report));
        assert_eq!(first_token("%% comment\ngraph LR"), "graph");
        assert_eq!(first_token("\n\n  flowchart TD"), "flowchart");
        assert_eq!(first_token("graph LR;"), "graph");
        assert_eq!(first_token("pie title X"), "pie");
        assert_eq!(first_token(""), "");
    }

    /// Longest match first, so a versioned keyword is not read as its prefix.
    #[test]
    fn a_versioned_keyword_matches_itself() {
        assert_eq!(
            leading_keyword("stateDiagram-v2\n [*] --> A"),
            Some("stateDiagram-v2")
        );
        assert_eq!(
            leading_keyword("stateDiagram\n [*] --> A"),
            Some("stateDiagram")
        );
        assert_eq!(
            leading_keyword("classDiagram-v2\n class A"),
            Some("classDiagram-v2")
        );
        assert_eq!(leading_keyword("notathing\n A"), None);
    }

    #[test]
    fn a_code_or_math_or_prose_figure_has_nothing_to_validate() {
        let report = check(
            "<figure id=\"figure-a\"><figcaption>Code.</figcaption>\
             <pre><code>let x = 1;</code></pre></figure>\
             <figure id=\"figure-b\"><figcaption>Math.</figcaption>\
             <math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mn>1</mn></math></figure>\
             <figure id=\"figure-c\"><figcaption>Prose.</figcaption><p>Text.</p></figure>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// The technical-investigation document embeds both a Mermaid diagram and a
    /// Vega-Lite chart, so this exercises both paths against hand-authored content.
    #[test]
    fn every_dogfood_document_has_valid_visualizations() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dogfood");
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .expect("the dogfood directory is readable")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "html"))
            .collect();
        paths.sort();
        assert_eq!(paths.len(), 6);

        let mut charts = 0;
        let mut diagrams = 0;
        for path in paths {
            let html = std::fs::read_to_string(&path).expect("readable");
            let outcome = parse_document(&html);
            let projection = project(&outcome.document);
            for figure in &projection.model.figures {
                match figure.kind {
                    FigureKind::VegaLite => charts += 1,
                    FigureKind::Mermaid => diagrams += 1,
                    _ => {}
                }
            }
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
                "{path:?} has an invalid visualization:\n{}",
                messages(&report)
            );
        }
        assert!(
            charts > 0 && diagrams > 0,
            "expected the dogfood corpus to exercise both a chart and a diagram, found \
             {charts} chart(s) and {diagrams} diagram(s)"
        );
    }
}
