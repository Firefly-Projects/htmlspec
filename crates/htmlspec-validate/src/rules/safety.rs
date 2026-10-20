//! `safety/*`: nothing reaches the network, and nothing executes.
//!
//! These rules read the raw tree rather than the projected model, because that is
//! where the danger lives. A remote script is not a semantic unit and never
//! appears in the model; a `<script>` element carrying a tracking pixel projects
//! no text at all (projection specification, section 8.5). A rule engine that only
//! ever saw the model would report such a document as clean.
//!
//! Every rule here is an error and none can be downgraded — the namespace decides
//! that, and [`crate::diagnostic::Namespace::is_downgradable`] enforces it. A
//! document that reaches outward does not render offline in perpetuity, and one
//! that executes untrusted content is unsafe to open. Neither is a matter of
//! project preference.

use htmlspec_parse::Projection;
use kuchikiki::iter::NodeIterator;
use kuchikiki::{NodeData, NodeRef};

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};

/// An element loads a resource from outside the project.
pub const REMOTE_RESOURCE: RuleId = RuleId::new(Namespace::Safety, "remote-resource");

/// A construct would issue a network request with no visible remote reference.
pub const CONCEALED_REQUEST: RuleId = RuleId::new(Namespace::Safety, "concealed-request");

/// A chart specification loads its data from a remote URL.
///
/// This needs a rule of our own: Vega-Lite permits a remote `data.url` by design,
/// so its published schema accepts what this project must reject. Validating the
/// specification against that schema would report such a chart as correct.
pub const REMOTE_CHART_DATA: RuleId = RuleId::new(Namespace::Safety, "remote-chart-data");

/// The attributes that load a resource, paired with the element they appear on.
///
/// `href` is deliberately absent for `a`: a hyperlink to an external page is a
/// citation, not a dependency, and the document renders fine offline with it. It
/// *is* checked on `link`, which loads.
const LOADING_ATTRIBUTES: &[(&str, &str)] = &[
    ("script", "src"),
    ("link", "href"),
    ("img", "src"),
    ("img", "srcset"),
    ("source", "src"),
    ("source", "srcset"),
    ("iframe", "src"),
    ("embed", "src"),
    ("object", "data"),
    ("video", "src"),
    ("audio", "src"),
    ("track", "src"),
    ("input", "src"),
    ("use", "href"),
    ("image", "href"),
];

/// `rel` values on a `link` that fetch rather than describe.
///
/// These are the concealed requests: a `prefetch` has no visible remote reference
/// in the rendered page and still reaches the network when the document opens.
const FETCHING_REL_VALUES: &[&str] = &[
    "prefetch",
    "preconnect",
    "prerender",
    "dns-prefetch",
    "preload",
    "modulepreload",
    "pingback",
];

/// Runs every network rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    let Some(document) = projection.document.as_ref() else {
        return;
    };
    remote_resources(document, diagnostics);
    concealed_requests(document, diagnostics);
    remote_chart_data(projection, diagnostics);
}

/// No element loads a resource from outside the project.
fn remote_resources(document: &NodeRef, diagnostics: &mut Diagnostics) {
    for element in document.inclusive_descendants().elements() {
        let node = element.as_node();
        let name = element.name.local.as_ref().to_lowercase();
        for (element_name, attribute_name) in LOADING_ATTRIBUTES {
            if name != *element_name {
                continue;
            }
            let Some(value) = attribute(node, attribute_name) else {
                continue;
            };
            // `srcset` holds a comma-separated candidate list, and one remote
            // candidate is enough to reach the network.
            for candidate in value.split(',') {
                let reference = candidate.split_whitespace().next().unwrap_or("").trim();
                if reference.is_empty() || !is_remote(reference) {
                    continue;
                }
                diagnostics.violation(
                    REMOTE_RESOURCE,
                    Location::default().at_schema_path(format!("/{name}[{attribute_name}]")),
                    format!(
                        "<{name} {attribute_name}=\"{reference}\"> loads a resource from \
                         outside the project. A document must render with no network access, \
                         so vendor the resource and reference it by relative path"
                    ),
                );
            }
        }
    }

    // A stylesheet can reach the network without any of the attributes above, so
    // the CSS itself is checked for remote `url()` and `@import`.
    for element in document.inclusive_descendants().elements() {
        let name = element.name.local.as_ref().to_lowercase();
        let is_style_carrier = name == "style";
        let inline_style = attribute(element.as_node(), "style");
        let css = if is_style_carrier {
            Some(text_of(element.as_node()))
        } else {
            inline_style
        };
        let Some(css) = css else {
            continue;
        };
        for reference in css_references(&css) {
            if !is_remote(&reference) {
                continue;
            }
            diagnostics.violation(
                REMOTE_RESOURCE,
                Location::default().at_schema_path(format!("/{name}[style]")),
                format!(
                    "styling loads {reference} from outside the project. A remote font or \
                     image reaches the network when the document opens, so vendor it and \
                     reference it by relative path"
                ),
            );
        }
    }
}

/// No construct issues a request without a visible remote reference.
fn concealed_requests(document: &NodeRef, diagnostics: &mut Diagnostics) {
    for element in document.inclusive_descendants().elements() {
        let node = element.as_node();
        let name = element.name.local.as_ref().to_lowercase();

        if name == "link" {
            if let Some(rel) = attribute(node, "rel") {
                for token in rel.split_whitespace() {
                    let token = token.to_lowercase();
                    if !FETCHING_REL_VALUES.contains(&token.as_str()) {
                        continue;
                    }
                    diagnostics.violation(
                        CONCEALED_REQUEST,
                        Location::default().at_schema_path("/link[rel]"),
                        format!(
                            "<link rel=\"{token}\"> issues a network request with no visible \
                             remote reference. A reader cannot see it and it defeats the \
                             offline guarantee"
                        ),
                    );
                }
            }
        }

        // `ping` fires a beacon on click, and `http-equiv=\"refresh\"` navigates.
        if let Some(ping) = attribute(node, "ping") {
            diagnostics.violation(
                CONCEALED_REQUEST,
                Location::default().at_schema_path(format!("/{name}[ping]")),
                format!(
                    "<{name} ping=\"{ping}\"> sends a beacon when followed. A document must \
                     issue no request the reader did not ask for"
                ),
            );
        }
        if name == "meta"
            && attribute(node, "http-equiv")
                .is_some_and(|value| value.eq_ignore_ascii_case("refresh"))
        {
            diagnostics.violation(
                CONCEALED_REQUEST,
                Location::default().at_schema_path("/meta[http-equiv]"),
                "<meta http-equiv=\"refresh\"> navigates without the reader asking. A \
                 document is a document and not a redirect",
            );
        }
    }
}

/// No chart specification loads its data from a remote URL.
///
/// Vega-Lite permits this by design, so the check is ours. Any `url` anywhere in
/// the specification is examined, because `data` appears at the top level, inside
/// `layer`, inside `hconcat`, and in a lookup transform.
fn remote_chart_data(projection: &Projection, diagnostics: &mut Diagnostics) {
    for figure in &projection.model.figures {
        let Some(source) = figure.source.as_deref() else {
            continue;
        };
        if figure.kind != htmlspec_parse::model::FigureKind::VegaLite {
            continue;
        }
        let Ok(specification) = serde_json::from_str::<serde_json::Value>(source) else {
            // A specification that is not JSON is the visualization rules' fault
            // to report, not this one's.
            continue;
        };
        for url in data_urls(&specification) {
            if !is_remote(&url) {
                continue;
            }
            diagnostics.violation(
                REMOTE_CHART_DATA,
                Location::maybe_unit(figure.id.as_deref()),
                format!(
                    "this chart loads its data from {url}. Vega-Lite permits a remote data \
                     URL, and this project does not: the chart would render empty with no \
                     network access"
                ),
            );
        }
    }
}

/// Every `url` value anywhere in a chart specification, in traversal order.
fn data_urls(value: &serde_json::Value) -> Vec<String> {
    let mut found = Vec::new();
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                if key == "url" {
                    if let Some(url) = child.as_str() {
                        found.push(url.to_owned());
                    }
                }
                found.extend(data_urls(child));
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                found.extend(data_urls(item));
            }
        }
        _ => {}
    }
    found
}

/// Whether a reference leaves the project.
///
/// A relative path is accepted: that is how a document reaches the shared
/// runtime. A scheme-bearing or protocol-relative reference is remote, with two
/// exceptions that fetch nothing — a `data:` URI is inline bytes, and a bare
/// fragment addresses this document.
fn is_remote(reference: &str) -> bool {
    let reference = reference.trim();
    if reference.is_empty() || reference.starts_with('#') {
        return false;
    }
    // `//example.test/x` inherits the page's scheme and is remote.
    if reference.starts_with("//") {
        return true;
    }
    let Some((scheme, _)) = reference.split_once(':') else {
        return false;
    };
    // A Windows path or a relative path holding a colon is not a scheme. A scheme
    // is letters, digits, `+`, `-`, and `.`, and starts with a letter.
    let is_scheme = scheme
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if !is_scheme {
        return false;
    }
    // `data:` is inline bytes and fetches nothing. It is still executable content
    // in some positions, which the executable-content rules cover.
    !scheme.eq_ignore_ascii_case("data")
}

/// Every reference a fragment of CSS loads, from `url()` and `@import`.
fn css_references(css: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = css;
    while let Some(at) = rest.find("url(") {
        rest = &rest[at + "url(".len()..];
        if let Some(end) = rest.find(')') {
            let reference = rest[..end].trim().trim_matches(['"', '\'']).to_owned();
            found.push(reference);
            rest = &rest[end..];
        } else {
            break;
        }
    }

    let mut rest = css;
    while let Some(at) = rest.find("@import") {
        rest = &rest[at + "@import".len()..];
        let Some(end) = rest.find(';') else { break };
        let clause = &rest[..end];
        rest = &rest[end..];
        // `@import url(x)` was already collected above; this catches the bare
        // string form, `@import "x"`.
        if clause.contains("url(") {
            continue;
        }
        let reference = clause.trim().trim_matches(['"', '\'']).to_owned();
        if !reference.is_empty() {
            found.push(reference);
        }
    }
    found
}

/// One attribute's value, trimmed.
fn attribute(node: &NodeRef, name: &str) -> Option<String> {
    let element = node.as_element()?;
    let attributes = element.attributes.borrow();
    attributes.get(name).map(|value| value.trim().to_owned())
}

/// The text content of an element, including text a comment splits.
fn text_of(node: &NodeRef) -> String {
    let mut collected = String::new();
    for descendant in node.inclusive_descendants() {
        if let NodeData::Text(text) = descendant.data() {
            collected.push_str(&text.borrow());
        }
    }
    collected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};

    /// These rules are not asset rules, so they run with no store beside them.
    const NO_STORE: htmlspec_assets::Store = htmlspec_assets::Store::Absent;
    use htmlspec_parse::{parse_document, project};

    fn check(body: &str) -> crate::Report {
        check_document(&format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>Fixture</title></head>\n<body>\n{body}\n</body>\n</html>\n"
        ))
    }

    /// Some fixtures need to place an element in the head, so the whole document
    /// is supplied.
    fn check_document(html: &str) -> crate::Report {
        let outcome = parse_document(html);
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
    fn a_relative_reference_into_the_shared_assets_is_accepted() {
        let report = check_document(
            "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title>\
             <link rel=\"stylesheet\" href=\"assets/htmlspec.css\">\
             <script src=\"../assets/mermaid.min.js\"></script>\
             </head><body><img src=\"./assets/diagram.png\" alt=\"A diagram\"></body></html>\n",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_remote_script_is_rejected() {
        let report = check("<script src=\"https://cdn.example.test/mermaid.js\"></script>");
        let found = fired(&report, REMOTE_RESOURCE);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("cdn.example.test"));
    }

    #[test]
    fn a_remote_stylesheet_and_a_remote_image_and_iframe_are_rejected() {
        let report = check(
            "<link rel=\"stylesheet\" href=\"https://example.test/a.css\">\
             <img src=\"http://example.test/b.png\" alt=\"\">\
             <iframe src=\"https://example.test/c.html\"></iframe>",
        );
        assert_eq!(
            fired(&report, REMOTE_RESOURCE).len(),
            3,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_protocol_relative_reference_is_remote() {
        let report = check("<script src=\"//cdn.example.test/x.js\"></script>");
        assert_eq!(
            fired(&report, REMOTE_RESOURCE).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_remote_font_in_a_stylesheet_is_rejected() {
        let report = check(
            "<style>@font-face { font-family: X; \
             src: url(\"https://fonts.example.test/x.woff2\"); }</style>",
        );
        let found = fired(&report, REMOTE_RESOURCE);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("fonts.example.test"));
    }

    #[test]
    fn a_remote_import_in_a_stylesheet_is_rejected() {
        let report = check("<style>@import \"https://example.test/other.css\";</style>");
        assert_eq!(
            fired(&report, REMOTE_RESOURCE).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_remote_url_in_an_inline_style_attribute_is_rejected() {
        let report =
            check("<div style=\"background: url(https://example.test/bg.png)\">Content.</div>");
        assert_eq!(
            fired(&report, REMOTE_RESOURCE).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_local_url_in_a_stylesheet_is_accepted() {
        let report = check(
            "<style>@font-face { src: url(\"assets/fonts/x.woff2\"); }\
             body { background: url(assets/bg.png); }</style>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_remote_candidate_in_a_srcset_is_rejected() {
        let report = check(
            "<img src=\"assets/a.png\" alt=\"\" \
             srcset=\"assets/a.png 1x, https://example.test/a@2x.png 2x\">",
        );
        assert_eq!(
            fired(&report, REMOTE_RESOURCE).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_prefetch_or_preconnect_is_a_concealed_request() {
        let report = check(
            "<link rel=\"prefetch\" href=\"assets/next.html\">\
             <link rel=\"preconnect\" href=\"https://example.test\">\
             <link rel=\"dns-prefetch\" href=\"https://example.test\">",
        );
        // Each is a concealed request; the two remote ones are also remote loads.
        assert_eq!(
            fired(&report, CONCEALED_REQUEST).len(),
            3,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_beacon_and_a_meta_refresh_are_concealed_requests() {
        let report = check(
            "<a href=\"assets/other.html\" ping=\"assets/track\">A link</a>\
             <meta http-equiv=\"refresh\" content=\"0; url=assets/other.html\">",
        );
        assert_eq!(
            fired(&report, CONCEALED_REQUEST).len(),
            2,
            "{}",
            messages(&report)
        );
    }

    /// An external citation is not a dependency: the document renders offline
    /// with it, and a reader chooses whether to follow it.
    #[test]
    fn an_external_hyperlink_is_a_citation_and_not_a_dependency() {
        let report = check("<p>See <a href=\"https://example.test/spec\">the spec</a>.</p>");
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_remote_chart_data_url_is_rejected_although_vega_lite_permits_it() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A chart.</figcaption>\
             <pre class=\"vega-lite\">{\"data\": {\"url\": \
             \"https://example.test/data.csv\"}, \"mark\": \"bar\"}</pre></figure>",
        );
        let found = fired(&report, REMOTE_CHART_DATA);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(found[0].location.unit_id.as_deref(), Some("figure-x"));
    }

    /// `data` appears at the top level, inside a layer, and in a lookup
    /// transform, so the whole specification is searched.
    #[test]
    fn a_remote_data_url_nested_in_a_layer_is_found() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A chart.</figcaption>\
             <pre class=\"vega-lite\">{\"layer\": [{\"data\": {\"url\": \
             \"https://example.test/a.json\"}, \"mark\": \"line\"}]}</pre></figure>",
        );
        assert_eq!(
            fired(&report, REMOTE_CHART_DATA).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn inline_chart_values_are_accepted() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A chart.</figcaption>\
             <pre class=\"vega-lite\">{\"data\": {\"values\": [{\"a\": 1}]}, \
             \"mark\": \"bar\"}</pre></figure>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// The `$schema` of a Vega-Lite specification is a remote *identifier*, not a
    /// fetch: the vendored copy is what validates it. Rejecting it would make
    /// every conforming chart invalid.
    #[test]
    fn the_chart_schema_identifier_is_not_a_data_url() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A chart.</figcaption>\
             <pre class=\"vega-lite\">{\"$schema\": \
             \"https://vega.github.io/schema/vega-lite/v5.json\", \
             \"data\": {\"values\": []}, \"mark\": \"bar\"}</pre></figure>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_data_uri_fetches_nothing_and_is_not_a_remote_resource() {
        assert!(!is_remote("data:image/png;base64,AAAA"));
        assert!(!is_remote("assets/x.js"));
        assert!(!is_remote("./assets/x.js"));
        assert!(!is_remote("../assets/x.js"));
        assert!(!is_remote("/assets/x.js"));
        assert!(!is_remote("#req-x"));
        assert!(is_remote("https://example.test/x"));
        assert!(is_remote("HTTP://example.test/x"));
        assert!(is_remote("//example.test/x"));
        assert!(is_remote("file:///etc/passwd"));
    }

    /// A safety diagnostic is an error under every policy, because a document
    /// that reaches the network is not a matter of project preference.
    #[test]
    fn a_safety_diagnostic_cannot_be_downgraded() {
        let report = check("<script src=\"https://example.test/x.js\"></script>");
        assert!(report.diagnostics[0].is_error());
        assert!(!Namespace::Safety.is_downgradable());
    }

    #[test]
    fn every_dogfood_document_reaches_no_network() {
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
                "{path:?} reaches the network:\n{}",
                messages(&report)
            );
        }
    }
}
