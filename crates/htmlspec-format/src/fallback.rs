//! Giving a document the toolchain's fallback stylesheet, when a contributor asks.
//!
//! Every document the toolchain writes now carries [`htmlspec_assets::fallback::ELEMENT`] in its
//! head, so it stays readable where its stylesheet cannot load. A document written earlier lacks it,
//! and gains it only through `htmlspec format --fallback`. [`crate::format`] never calls this.
//!
//! **Why never unasked.** A runbook's release is fingerprinted by the hash of its canonical
//! document, so a formatter that added the block would change every runbook's fingerprint on the
//! first format after an upgrade, and archival would then refuse until each runbook raised its
//! version for a release that changes no step. It would also rewrite every archived document, which
//! deterministic-formatting's "Formatting preserves meaning" already refuses for asset references:
//! an archived page renders with what it was reviewed against. Asked for, the edit is the
//! contributor's, and archival asking for a new runbook version is the right answer to it.
//!
//! **On the tree, then serialized.** The block goes into the parsed head and the canonical
//! serializer places it, so the result is canonical in one pass. A textual insertion would have to
//! know where the head's children start and what indentation they take, which the serializer
//! already decides.

use htmlspec_assets::fallback;
use htmlspec_parse::parse_document;
use kuchikiki::NodeRef;

use crate::{serialize, FormatError};

/// What a request did to one document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackChange {
    /// The document had no fallback, and now carries the current one.
    Added,
    /// The document carried a fallback whose text differed from the current one, and now carries
    /// the current one.
    Refreshed,
    /// The document already carried the current fallback, and is returned as it was.
    Current,
}

/// A document after a request, and what the request did to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackOutcome {
    /// What the request did.
    pub change: FallbackChange,
    /// The document's source: canonical where the request changed it, as given where it did not.
    pub source: String,
}

/// Gives `source` the current fallback stylesheet, adding it or replacing a stale one.
///
/// Placed before the head's first `<link>`, where a template places it, or at the end of the head
/// when it has none. Refuses what [`crate::format`] refuses, a document whose markup needs error
/// recovery, for the same reason: it has more than one reading to edit.
pub fn apply(source: &str) -> Result<FallbackOutcome, FormatError> {
    let outcome = parse_document(source);
    if !outcome.is_recovery_free() {
        return Err(FormatError::Ambiguous {
            recovered: outcome
                .recovered
                .iter()
                .map(|error| format!("{}: {}", error.code, error.message))
                .collect(),
        });
    }
    let document = &outcome.document;
    let Some(head) = element_named(document, "head") else {
        // The parser always builds a head, so a document without one is not reachable here. Left
        // unchanged rather than invented: a request adds a stylesheet, not a head.
        return Ok(FallbackOutcome {
            change: FallbackChange::Current,
            source: source.to_owned(),
        });
    };

    let existing = head
        .children()
        .filter(|child| is_element(child, "style"))
        .find(|style| fallback::is_fallback_text(&style.text_contents()));
    let change = match existing {
        Some(style) if style.text_contents() == fallback::css() => {
            return Ok(FallbackOutcome {
                change: FallbackChange::Current,
                source: source.to_owned(),
            });
        }
        Some(style) => {
            for child in style.children().collect::<Vec<_>>() {
                child.detach();
            }
            style.append(NodeRef::new_text(fallback::css()));
            FallbackChange::Refreshed
        }
        None => {
            let element = fallback_element();
            match head.children().find(|child| is_element(child, "link")) {
                Some(link) => link.insert_before(element),
                None => head.append(element),
            }
            FallbackChange::Added
        }
    };
    Ok(FallbackOutcome {
        change,
        source: serialize::document(document),
    })
}

/// A detached `<style>` element holding the current fallback, parsed from the constant itself.
///
/// Parsed rather than built node by node, so the element is exactly what a template writes and no
/// second construction of it exists to drift.
fn fallback_element() -> NodeRef {
    let carrier = parse_document(&format!(
        "<!DOCTYPE html><html lang=\"en\"><head>{}</head><body></body></html>",
        fallback::ELEMENT
    ));
    let style = element_named(&carrier.document, "style")
        .expect("the fallback element parses as a <style> element");
    style.detach();
    style
}

fn element_named(root: &NodeRef, name: &str) -> Option<NodeRef> {
    root.descendants().find(|node| is_element(node, name))
}

fn is_element(node: &NodeRef, name: &str) -> bool {
    node.as_element()
        .is_some_and(|element| element.name.local.as_ref() == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A document as `new document` wrote one before the fallback existed.
    fn without_fallback() -> String {
        htmlspec_assets::render_kind("specification", "A subject", "assets")
            .expect("the template renders")
            .replace(&format!("    {}\n", fallback::ELEMENT), "")
    }

    #[test]
    fn a_document_without_the_fallback_gains_it_canonically() {
        let before = without_fallback();
        assert!(
            !before.contains(fallback::LAYER),
            "the fixture lacks the fallback"
        );
        let outcome = apply(&before).expect("it applies");
        assert_eq!(outcome.change, FallbackChange::Added);
        assert_eq!(
            outcome.source,
            htmlspec_assets::render_kind("specification", "A subject", "assets")
                .expect("the template renders"),
            "the result is not what the template writes, so it is not canonical"
        );
    }

    #[test]
    fn a_document_with_the_current_fallback_is_returned_as_it_was() {
        let current = htmlspec_assets::render_kind("tasks", "A subject", "assets")
            .expect("the template renders");
        let outcome = apply(&current).expect("it applies");
        assert_eq!(outcome.change, FallbackChange::Current);
        assert_eq!(outcome.source, current);
    }

    #[test]
    fn a_stale_fallback_is_replaced_and_nothing_else_moves() {
        let current = htmlspec_assets::render_kind("design", "A subject", "assets")
            .expect("the template renders");
        let stale = current.replace("line-height: 1.55;", "line-height: 1.5;");
        assert_ne!(stale, current, "the fixture is stale");
        let outcome = apply(&stale).expect("it applies");
        assert_eq!(outcome.change, FallbackChange::Refreshed);
        assert_eq!(outcome.source, current);
    }

    #[test]
    fn a_second_request_changes_nothing() {
        let once = apply(&without_fallback()).expect("it applies").source;
        let twice = apply(&once).expect("it applies");
        assert_eq!(twice.change, FallbackChange::Current);
        assert_eq!(twice.source, once);
    }

    #[test]
    fn another_style_element_is_left_alone() {
        let before = without_fallback().replace(
            "    <link ",
            "    <style>\n      .mine { color: teal; }\n    </style>\n    <link ",
        );
        let outcome = apply(&before).expect("it applies");
        assert_eq!(outcome.change, FallbackChange::Added);
        assert!(outcome.source.contains(".mine { color: teal; }"));
        assert_eq!(outcome.source.matches(fallback::LAYER).count(), 1);
    }

    #[test]
    fn a_document_with_no_link_gains_it_at_the_end_of_its_head() {
        let before =
            "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\">\n    \
                      <title>T</title>\n  </head>\n  <body>\n    <main>\n      <p>Text.</p>\n    \
                      </main>\n  </body>\n</html>\n";
        let outcome = apply(before).expect("it applies");
        assert_eq!(outcome.change, FallbackChange::Added);
        let head = &outcome.source[..outcome.source.find("</head>").expect("a head")];
        assert!(head.contains(fallback::ELEMENT), "{}", outcome.source);
    }

    #[test]
    fn an_ambiguous_document_is_refused() {
        assert!(apply("<p>unclosed <b>bold <i>both</b> italic</i></p>").is_err());
    }
}
