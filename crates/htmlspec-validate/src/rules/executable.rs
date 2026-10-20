//! `safety/*`: example code is inert, and only the pinned runtime executes.
//!
//! The interesting rule here is the one about code examples, and it is
//! *structural* rather than textual. An example that escapes its container does
//! not produce suspicious-looking text — it produces live elements in the tree,
//! because the parser has already resolved every encoding trick a regular
//! expression would read differently. So the rule is: a code element's children
//! contain text and comment nodes only, with no element descendant.
//!
//! The security property is the *absence of elements*, not a child count. A
//! prototype confirmed why that distinction matters: the safe and the dangerous
//! case both have exactly one child, and "exactly one text node" additionally
//! false-rejects an empty example and text that a parser split around a comment.
//! Both of those are valid and both have fixtures below.

use kuchikiki::iter::NodeIterator;
use kuchikiki::NodeRef;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};

/// A code example contains an element, so it is no longer inert text.
pub const LIVE_EXAMPLE: RuleId = RuleId::new(Namespace::Safety, "live-example");

/// A script that is not part of the pinned runtime.
pub const UNAPPROVED_SCRIPT: RuleId = RuleId::new(Namespace::Safety, "unapproved-script");

/// An attribute that runs script when an event fires.
pub const EVENT_HANDLER: RuleId = RuleId::new(Namespace::Safety, "event-handler");

/// A URL whose scheme executes rather than fetches.
pub const EXECUTABLE_URL: RuleId = RuleId::new(Namespace::Safety, "executable-url");

/// The path prefix a script must load from to be part of the pinned runtime.
///
/// A document renders through the project's shared `assets/` directory, so a
/// script sourced from there is the vendored runtime whose version and content
/// hash the asset lockfile records. Anything else — including an inline script,
/// which no lockfile can pin — is unapproved.
const RUNTIME_DIRECTORY: &str = "assets/";

/// URL schemes that execute rather than fetch.
const EXECUTABLE_SCHEMES: &[&str] = &["javascript", "vbscript"];

/// Runs every executable-content rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    let Some(document) = projection.document.as_ref() else {
        return;
    };
    examples_are_inert(document, diagnostics);
    scripts_belong_to_the_runtime(document, diagnostics);
    no_event_handlers(document, diagnostics);
    no_executable_urls(document, diagnostics);
}

/// A code element's children contain text and comment nodes only.
fn examples_are_inert(document: &NodeRef, diagnostics: &mut Diagnostics) {
    for element in document.inclusive_descendants().elements() {
        let name = element.name.local.as_ref().to_lowercase();
        if name != "code" {
            continue;
        }
        let node = element.as_node();
        let elements_inside: Vec<String> = node
            .children()
            .filter_map(|child| {
                child
                    .as_element()
                    .map(|e| e.name.local.as_ref().to_lowercase())
            })
            .collect();
        if elements_inside.is_empty() {
            continue;
        }
        diagnostics.violation(
            LIVE_EXAMPLE,
            Location::default().at_schema_path("/code"),
            format!(
                "a code example contains {} element(s) ({}) rather than text alone. Either \
                 the example escaped its container and is now live markup, or presentational \
                 markup was placed inside it; escape the content so the parser reads it as \
                 text",
                elements_inside.len(),
                elements_inside.join(", ")
            ),
        );
    }
}

/// Every script belongs to the pinned runtime.
fn scripts_belong_to_the_runtime(document: &NodeRef, diagnostics: &mut Diagnostics) {
    for element in document.inclusive_descendants().elements() {
        let name = element.name.local.as_ref().to_lowercase();
        if name != "script" {
            continue;
        }
        let node = element.as_node();

        // A `type` the browser does not execute makes the element a data block
        // rather than a script. It is not executable content, but the
        // no-JavaScript rules reject it separately for being invisible, so this
        // rule leaves it alone rather than reporting the wrong reason for it.
        if let Some(kind) = attribute(node, "type") {
            if !is_executable_type(&kind) {
                continue;
            }
        }

        match attribute(node, "src") {
            Some(src) if is_runtime_asset(&src) => {}
            Some(src) => diagnostics.violation(
                UNAPPROVED_SCRIPT,
                Location::default().at_schema_path("/script[src]"),
                format!(
                    "the script {src} is not part of the pinned runtime. An executable script \
                     must load from {RUNTIME_DIRECTORY}, where the asset lockfile records its \
                     version and content hash"
                ),
            ),
            None => diagnostics.violation(
                UNAPPROVED_SCRIPT,
                Location::default().at_schema_path("/script"),
                "an inline script cannot be pinned by the asset lockfile, so it is not part \
                 of the pinned runtime. Move the behavior into the shared enhancement script"
                    .to_owned(),
            ),
        }
    }
}

/// No element carries an inline event handler.
///
/// An `onclick` is executable content that no lockfile can pin, and it would slip
/// past a rule that only examined `script` elements.
fn no_event_handlers(document: &NodeRef, diagnostics: &mut Diagnostics) {
    for element in document.inclusive_descendants().elements() {
        let name = element.name.local.as_ref().to_lowercase();
        let handlers: Vec<String> = {
            let attributes = element.attributes.borrow();
            attributes
                .map
                .keys()
                .map(|key| key.local.as_ref().to_lowercase())
                .filter(|attribute| is_event_handler(attribute))
                .collect()
        };
        for handler in handlers {
            diagnostics.violation(
                EVENT_HANDLER,
                Location::default().at_schema_path(format!("/{name}[{handler}]")),
                format!(
                    "<{name} {handler}=\"...\"> runs script when the event fires. Inline \
                     handlers cannot be pinned, so behavior belongs in the shared enhancement \
                     script"
                ),
            );
        }
    }
}

/// No URL uses a scheme that executes.
fn no_executable_urls(document: &NodeRef, diagnostics: &mut Diagnostics) {
    // Every attribute is examined rather than a named list: `javascript:` executes
    // wherever a URL is accepted, and a list would be a list of the positions
    // someone thought of.
    for element in document.inclusive_descendants().elements() {
        let name = element.name.local.as_ref().to_lowercase();
        let offenders: Vec<(String, String)> = {
            let attributes = element.attributes.borrow();
            attributes
                .map
                .iter()
                .filter_map(|(key, value)| {
                    let scheme = executable_scheme(&value.value)?;
                    Some((key.local.as_ref().to_lowercase(), scheme))
                })
                .collect()
        };
        for (attribute_name, scheme) in offenders {
            diagnostics.violation(
                EXECUTABLE_URL,
                Location::default().at_schema_path(format!("/{name}[{attribute_name}]")),
                format!(
                    "<{name} {attribute_name}> uses the {scheme}: scheme, which executes \
                     rather than fetches. A document must contain no executable content \
                     outside the pinned runtime"
                ),
            );
        }
    }
}

/// The executable scheme a value uses, if any.
///
/// Leading whitespace and control characters are stripped first, because a browser
/// strips them before resolving the scheme and a check that did not would miss
/// `\tjavascript:alert(1)`.
fn executable_scheme(value: &str) -> Option<String> {
    let cleaned: String = value
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect();
    let (scheme, _) = cleaned.split_once(':')?;
    EXECUTABLE_SCHEMES
        .iter()
        .find(|executable| scheme.eq_ignore_ascii_case(executable))
        .map(|found| (*found).to_owned())
}

/// Whether a `script` `type` is one the browser executes.
///
/// An absent type means JavaScript. An empty one, and the legacy JavaScript MIME
/// types, also execute. Anything else is a data block.
fn is_executable_type(kind: &str) -> bool {
    let kind = kind.trim().to_lowercase();
    kind.is_empty()
        || kind == "module"
        || kind == "text/javascript"
        || kind == "application/javascript"
        || kind == "text/ecmascript"
        || kind == "application/ecmascript"
}

/// Whether a script source is part of the project's shared runtime.
fn is_runtime_asset(src: &str) -> bool {
    let src = src.trim();
    // A remote script is reported by the network rules; this rule reports that it
    // is not the pinned runtime, which is separately true.
    src.split('/')
        .any(|segment| !segment.is_empty() && format!("{segment}/") == RUNTIME_DIRECTORY)
        && !src.contains("://")
        && !src.starts_with("//")
}

/// Whether an attribute name is an event handler.
///
/// Matched by shape rather than against a list of names, so a handler this project
/// has not heard of is still caught.
fn is_event_handler(name: &str) -> bool {
    name.starts_with("on") && name.len() > 2
}

/// One attribute's value, trimmed.
fn attribute(node: &NodeRef, name: &str) -> Option<String> {
    let element = node.as_element()?;
    let attributes = element.attributes.borrow();
    attributes.get(name).map(|value| value.trim().to_owned())
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

    /// A JavaScript snippet shown as an example is text, and it does not run.
    #[test]
    fn a_script_snippet_shown_as_an_example_is_accepted() {
        let report = check(
            "<pre><code>&lt;script&gt;alert(1)&lt;/script&gt;</code></pre>\
             <p><code>const x = 1;</code></p>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// An empty example has no children at all, which is why the rule cannot ask
    /// for exactly one text node.
    #[test]
    fn an_empty_example_is_accepted() {
        let report = check("<pre><code></code></pre>");
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// A parser splits text around a comment, giving two text nodes. No child is
    /// an element, so the example is inert.
    #[test]
    fn text_split_by_a_comment_is_accepted() {
        let report = check("<pre><code>before<!-- an aside -->after</code></pre>");
        assert!(report.is_accepted(), "{}", messages(&report));
        let node = parse_document(
            "<!doctype html><html><head><title>T</title></head><body>\
             <code>before<!-- c -->after</code></body></html>",
        );
        let code = node
            .document
            .inclusive_descendants()
            .elements()
            .find(|e| e.name.local.as_ref() == "code")
            .expect("the fixture has a code element");
        // Three children, none of them an element: the property is the absence of
        // elements and not a child count.
        assert_eq!(code.as_node().children().count(), 3);
        assert!(
            code.as_node()
                .children()
                .all(|child| child.as_element().is_none()),
            "no child of an inert example is an element"
        );
    }

    /// The safe and the dangerous case both have exactly one child, which is the
    /// measurement that ruled out a count-based rule.
    #[test]
    fn a_presentational_element_inside_an_example_is_rejected() {
        let report = check("<pre><code><span>const x = 1;</span></code></pre>");
        let found = fired(&report, LIVE_EXAMPLE);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("span"), "{}", found[0].message);
    }

    /// An unescaped example closes its container and injects live markup. The
    /// parser resolves it into elements, so the structural rule catches it.
    #[test]
    fn an_example_that_escapes_its_container_is_rejected() {
        // The `</code>` inside closes the element, and the `<b>` becomes live.
        let report = check("<pre><code>text</code><b>live</b></code></pre>");
        // The escape shows up as either a live example or a parse recovery; both
        // reject the document, and a recovery curtails the run.
        assert!(!report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_runtime_script_from_the_shared_assets_is_accepted() {
        let report = check("<script src=\"assets/htmlspec.js\"></script>");
        assert!(report.is_accepted(), "{}", messages(&report));

        let relative = check("<script src=\"../../assets/mermaid.min.js\"></script>");
        assert!(relative.is_accepted(), "{}", messages(&relative));
    }

    #[test]
    fn an_inline_script_is_not_part_of_the_pinned_runtime() {
        let report = check("<script>document.title = 'changed';</script>");
        let found = fired(&report, UNAPPROVED_SCRIPT);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains("lockfile"),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn a_script_from_outside_the_shared_assets_is_unapproved() {
        let report = check("<script src=\"scripts/custom.js\"></script>");
        assert_eq!(
            fired(&report, UNAPPROVED_SCRIPT).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// A remote script is unapproved *and* a network dependency. Both are true and
    /// the network rule reports the other half.
    #[test]
    fn a_remote_script_is_also_unapproved() {
        let report = check("<script src=\"https://example.test/assets/x.js\"></script>");
        assert_eq!(
            fired(&report, UNAPPROVED_SCRIPT).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// A non-executing `type` makes the element a data block. It is rejected by
    /// the no-JavaScript rules for being invisible, which is a different reason,
    /// so this rule stays quiet rather than reporting the wrong one.
    #[test]
    fn a_data_block_is_not_reported_as_an_unapproved_script() {
        let report = check("<script type=\"application/json\">{\"a\": 1}</script>");
        assert!(
            fired(&report, UNAPPROVED_SCRIPT).is_empty(),
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn an_inline_event_handler_is_rejected() {
        let report = check("<button onclick=\"run()\">Go</button>");
        let found = fired(&report, EVENT_HANDLER);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("onclick"), "{}", found[0].message);
    }

    /// Matched by shape, so a handler this project has not heard of is caught.
    #[test]
    fn an_unfamiliar_event_handler_is_still_rejected() {
        let report = check("<div onsomethingnew=\"run()\">Content</div>");
        assert_eq!(
            fired(&report, EVENT_HANDLER).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// `on` alone is not a handler, and neither is an ordinary attribute that
    /// happens to start with those letters.
    #[test]
    fn an_ordinary_attribute_is_not_mistaken_for_a_handler() {
        let report = check("<div data-once=\"true\" role=\"note\">Content</div>");
        assert!(
            fired(&report, EVENT_HANDLER).is_empty(),
            "{}",
            messages(&report)
        );
        assert!(!is_event_handler("on"));
        assert!(!is_event_handler("role"));
        assert!(is_event_handler("onclick"));
    }

    #[test]
    fn a_javascript_url_is_rejected() {
        let report = check("<p><a href=\"javascript:alert(1)\">A link</a></p>");
        assert_eq!(
            fired(&report, EXECUTABLE_URL).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// A browser strips whitespace and control characters before resolving the
    /// scheme, so a check that did not would miss this.
    #[test]
    fn a_javascript_url_hidden_by_whitespace_is_rejected() {
        let report = check("<p><a href=\"  java\tscript:alert(1)\">A link</a></p>");
        assert_eq!(
            fired(&report, EXECUTABLE_URL).len(),
            1,
            "{}",
            messages(&report)
        );
        assert_eq!(
            executable_scheme("  java\tscript:alert(1)").as_deref(),
            Some("javascript")
        );
        assert_eq!(
            executable_scheme("JavaScript:alert(1)").as_deref(),
            Some("javascript")
        );
        assert_eq!(executable_scheme("assets/x.js"), None);
        assert_eq!(executable_scheme("https://example.test/"), None);
    }

    #[test]
    fn a_safety_diagnostic_here_cannot_be_downgraded() {
        let report = check("<button onclick=\"run()\">Go</button>");
        assert!(report.diagnostics.iter().all(|d| d.is_error()));
    }

    #[test]
    fn every_dogfood_document_is_inert() {
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
                "{path:?} contains executable content:\n{}",
                messages(&report)
            );
        }
    }
}
