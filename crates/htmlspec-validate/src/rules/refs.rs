//! `refs/*`: every reference and every fragment link resolves.
//!
//! Two kinds of pointer, two rules, deliberately separate. A *reference* is one
//! unit addressing another (projection specification, section 5), and it must
//! name a unit that exists. A *link* is a hyperlink into the document, and it must
//! name something the browser can scroll to — which includes a figure and a
//! plain section anchor, neither of which is a semantic unit.
//!
//! These are errors, not review warnings. A reference that resolves to nothing is
//! not a dependent unit needing reconciliation; it is a pointer to a thing that
//! does not exist, and no amount of review makes it resolve. `review/*` covers
//! the case where a reference *does* resolve but the target has moved on.

use std::collections::BTreeSet;

use htmlspec_parse::model::ReferenceOrigin;
use htmlspec_parse::Projection;
use kuchikiki::iter::NodeIterator;
use kuchikiki::NodeRef;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};
use crate::references::{searched, withdrawing, Found, Scope, Unread};
use crate::schema_facts;

/// A reference names a unit that does not exist.
pub const UNRESOLVED: RuleId = RuleId::new(Namespace::Refs, "unresolved");

/// A fragment link names no element in the document.
pub const BROKEN_LINK: RuleId = RuleId::new(Namespace::Refs, "broken-link");

/// A reference targets a unit withdrawn by this change.
pub const REMOVED_TARGET: RuleId = RuleId::new(Namespace::Refs, "removed-target");

/// Runs every reference rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    let addressable = addressable_identifiers(projection);
    let removed = removed_identifiers(projection);

    references_resolve(
        projection,
        subject.scope,
        &addressable,
        &removed,
        diagnostics,
    );
    links_resolve(projection, &anchors(projection), diagnostics);
}

/// Every identifier a *link* can resolve to.
///
/// Broader than [`addressable_identifiers`] on purpose, and read from the tree
/// rather than from the model. A link must name something the browser can scroll
/// to, and a browser scrolls to any element carrying the identifier — a section
/// heading anchor, the main content region, a table row. None of those is a
/// semantic unit, so none appears in the model.
///
/// Using the model here would reject `<a href="#hs-requirements">` pointing at
/// `<section id="hs-requirements">`, which resolves perfectly in every browser.
/// That is the defect this function exists to fix: the rule is about what a reader
/// following the link arrives at, and only the tree knows that.
fn anchors(projection: &Projection) -> BTreeSet<String> {
    let Some(document) = &projection.document else {
        // With no tree there is nothing to check against, and reporting every link
        // as broken would be worse than reporting none: a projection that could not
        // read the document already reports a `schema/*` fault.
        return BTreeSet::new();
    };
    identifiers_in(document)
}

/// Every `id` attribute in the tree.
fn identifiers_in(document: &NodeRef) -> BTreeSet<String> {
    document
        .inclusive_descendants()
        .elements()
        .filter_map(|element| {
            let attributes = element.attributes.borrow();
            attributes
                .get("id")
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(str::to_owned)
        })
        .collect()
}

/// Every identifier a link or a reference can resolve to.
///
/// A unit identifier and a figure identifier both live here, because both are
/// addressable by a link. Whether a *reference* may name a figure is decided by
/// the projection: a figure identifier does not match the schema's `unitId`
/// grammar, so it never becomes a reference in the first place (section 5.3).
fn addressable_identifiers(projection: &Projection) -> BTreeSet<&str> {
    let mut found: BTreeSet<&str> = projection
        .model
        .units()
        .into_iter()
        .filter_map(|unit| unit.id)
        .collect();
    found.extend(
        projection
            .model
            .figures
            .iter()
            .filter_map(|figure| figure.id.as_deref()),
    );
    found
}

/// Every identifier whose unit this change withdraws.
///
/// The state name is read from the schema, so a rename moves one file. A schema
/// with no such state yields an empty set, and
/// `the_removed_state_is_found_in_the_schema` fails, so the rule cannot quietly
/// stop firing.
fn removed_identifiers(projection: &Projection) -> BTreeSet<&str> {
    let Some(removed) = removed_state() else {
        return BTreeSet::new();
    };
    projection
        .model
        .units()
        .into_iter()
        .filter(|unit| unit.change == removed)
        .filter_map(|unit| unit.id)
        .collect()
}

/// The change state that means a unit is being withdrawn, read from the schema.
///
/// Found by a stem of the concept's English name rather than written out, so the
/// contract value stays in the schema. A stem matching none or several yields
/// `None`.
pub(crate) fn removed_state() -> Option<String> {
    let mut matches = schema_facts::enum_values("definitions/changeState")
        .into_iter()
        .filter(|value| value.contains("remov"));
    let found = matches.next()?;
    matches.next().is_none().then_some(found)
}

/// Every reference names a unit that exists and is not withdrawn.
///
/// Resolved first in the document that makes it, as it always was, so a reference that resolved
/// before resolves to the same unit and meets the same withdrawal check. A `data-refs` entry its
/// own document does not declare is then tried in the places `scope` reaches, in order, and the
/// first place that declares it is the one it means, withdrawal included. A fragment link is a
/// hyperlink into its own page, so it resolves there or nowhere.
fn references_resolve(
    projection: &Projection,
    scope: &Scope,
    addressable: &BTreeSet<&str>,
    removed: &BTreeSet<&str>,
    diagnostics: &mut Diagnostics,
) {
    for (source, reference) in projection.model.references() {
        let location = Location::maybe_unit(source.id);
        let target = reference.target.as_str();

        if addressable.contains(target) {
            if removed.contains(target) {
                diagnostics.violation(
                    REMOVED_TARGET,
                    location,
                    format!(
                        "this {} references {target}, which this change withdraws. Work cannot \
                         target an obligation that is being removed",
                        source.kind
                    ),
                );
            }
            continue;
        }

        let found = match reference.origin {
            ReferenceOrigin::Attribute => scope.find(target),
            ReferenceOrigin::Link => Found::Nowhere {
                searched: Vec::new(),
                unread: Vec::new(),
            },
        };
        match found {
            Found::In {
                withdrawn: false, ..
            } => {}
            Found::In {
                place,
                withdrawn: true,
            } => diagnostics.violation(
                REMOVED_TARGET,
                location,
                format!(
                    "this {} references {target}, which {}. Work cannot target an obligation \
                     that is being removed",
                    source.kind,
                    withdrawing(place)
                ),
            ),
            Found::Nowhere {
                searched: places,
                unread,
            } => diagnostics.violation(
                UNRESOLVED,
                location,
                if places.is_empty() {
                    // A document that reaches only itself keeps the message it always had.
                    format!(
                        "this {} references {target}, and no unit in the document declares that \
                         identifier. A reference that resolves to nothing records a relationship \
                         to a thing that does not exist",
                        source.kind
                    )
                } else {
                    format!(
                        "this {} references {target}, and no unit {} declares that identifier.{} \
                         A reference that resolves to nothing records a relationship to a thing \
                         that does not exist",
                        source.kind,
                        searched(&places),
                        unread_named(&unread),
                    )
                },
            ),
        }
    }
}

/// The documents a search could not read, as a sentence that follows the report's first.
///
/// Named rather than counted, and never read as declaring nothing: the unit can be in one of them,
/// so a report that no unit declares the identifier would send the author to add one that may
/// already be there.
fn unread_named(unread: &[&Unread]) -> String {
    unread
        .iter()
        .map(|document| {
            format!(
                " {} could not be read for what it declares ({}), so the unit can be there.",
                document.document, document.reason
            )
        })
        .collect()
}

/// Every fragment link names an element the browser can reach.
///
/// A link to a figure or to a plain section anchor is legitimate, so this rule
/// checks against every addressable identifier rather than against unit
/// identifiers alone. A link is not a relationship, so a link to a withdrawn unit
/// is not an error here: the reference rule covers the relationship, and a
/// reviewer following the link sees the withdrawal, which is the point of keeping
/// a removed unit in the document.
fn links_resolve(
    projection: &Projection,
    anchors: &BTreeSet<String>,
    diagnostics: &mut Diagnostics,
) {
    for link in &projection.model.links {
        if anchors.contains(&link.target) {
            continue;
        }
        diagnostics.violation(
            BROKEN_LINK,
            Location::default().at_schema_path(format!("/links/{}", link.target)),
            format!(
                "the link to #{} has no target in the document. A reader following it \
                 arrives nowhere",
                link.target
            ),
        );
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
        check_in(body, &crate::references::Scope::loose())
    }

    /// The fixture `body` wrapped as a document, projected, for a place to declare.
    fn projected(body: &str) -> htmlspec_parse::Projection {
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\">\
             <title>Fixture</title></head>\n<body>\n{body}\n</body>\n</html>\n"
        );
        project(&parse_document(&html).document)
    }

    /// What a place holding documents with `bodies` declares.
    fn declaring(bodies: &[&str]) -> crate::references::Declared {
        let projections: Vec<_> = bodies.iter().map(|body| projected(body)).collect();
        crate::references::Declared::gather(projections.iter(), Vec::new())
    }

    /// A task in one of a change's documents, whose change's specification declares `change` and
    /// whose accepted specifications declare `accepted`.
    fn check_in_a_change(
        body: &str,
        change: crate::references::Declared,
        accepted: crate::references::Declared,
    ) -> crate::Report {
        use crate::references::{Place, Placement, Scope};
        let (mut change, mut accepted) = (Some(change), Some(accepted));
        let scope = Scope::gather(Placement::ChangeDocument, |place| match place {
            Place::ChangeSpecification => change.take().unwrap_or_default(),
            Place::AcceptedSpecifications => accepted.take().unwrap_or_default(),
        });
        check_in(body, &scope)
    }

    fn check_in(body: &str, scope: &crate::references::Scope) -> crate::Report {
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
            scope,
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

    /// A requirement the fixtures below can point at.
    const TARGET: &str = "<section class=\"requirement\" id=\"req-target\"><h2>R</h2>\
         <p>The system MUST work.</p>\
         <article class=\"scenario\" id=\"scenario-target\"><h3>S</h3>\
         <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>";

    #[test]
    fn a_reference_that_resolves_reports_nothing() {
        let report = check(&format!(
            "{TARGET}<section class=\"task\" id=\"task-x\" data-state=\"todo\" \
             data-refs=\"req-target scenario-target\"><h2>T</h2></section>"
        ));
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_dangling_reference_names_the_source_and_the_target() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\" \
             data-refs=\"req-does-not-exist\"><h2>T</h2></section>",
        );
        let found = fired(&report, UNRESOLVED);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(found[0].location.unit_id.as_deref(), Some("task-x"));
        assert!(
            found[0].message.contains("req-does-not-exist"),
            "{}",
            found[0].message
        );
    }

    /// A task naming `req-target`, as a change's tasks document holds it.
    const TASK_NAMING_THE_TARGET: &str =
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\" data-refs=\"req-target\">\
         <h2>T</h2></section>";

    /// The change's specification is tried before the accepted specifications, so its statement of
    /// a requirement is the one the change's own documents mean.
    #[test]
    fn a_change_specifications_unit_comes_before_the_accepted_one() {
        let withdrawn_target = TARGET.replace(
            "id=\"req-target\"",
            "id=\"req-target\" data-change=\"removed\"",
        );
        let report = check_in_a_change(
            TASK_NAMING_THE_TARGET,
            declaring(&[TARGET]),
            declaring(&[&withdrawn_target]),
        );
        assert!(report.diagnostics.is_empty(), "{}", messages(&report));

        let report = check_in_a_change(
            TASK_NAMING_THE_TARGET,
            declaring(&[&withdrawn_target]),
            declaring(&[TARGET]),
        );
        let found = fired(&report, REMOVED_TARGET);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0]
                .message
                .contains("its change's specification withdraws"),
            "{}",
            found[0].message
        );
        assert!(
            fired(&report, UNRESOLVED).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// An identifier two accepted specifications declare resolves, as `merge_state` takes either.
    #[test]
    fn an_identifier_two_accepted_documents_declare_resolves() {
        let report = check_in_a_change(
            TASK_NAMING_THE_TARGET,
            crate::references::Declared::default(),
            declaring(&[TARGET, TARGET]),
        );
        assert!(report.diagnostics.is_empty(), "{}", messages(&report));
    }

    /// A place's document that could not be read is named, and the report does not claim that no
    /// unit declares the identifier anywhere it looked.
    #[test]
    fn an_unread_document_is_named_in_the_report() {
        let unread = crate::references::Declared::gather(
            std::iter::empty(),
            vec![crate::references::Unread {
                document: String::from("changes/x/specification.html"),
                reason: String::from("its markup needs the parser to recover"),
            }],
        );
        let report = check_in_a_change(
            TASK_NAMING_THE_TARGET,
            unread,
            crate::references::Declared::default(),
        );
        let found = fired(&report, UNRESOLVED);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains(
                "in this document, in its change's specification, or in the accepted \
                 specifications"
            ) && found[0]
                .message
                .contains("changes/x/specification.html could not be read for what it declares"),
            "{}",
            found[0].message
        );
    }

    /// A fragment link resolves only in its own page, whatever the places it would reach declare,
    /// and its message is the one a loose document gives.
    #[test]
    fn a_link_reference_is_not_resolved_through_the_scope() {
        let report = check_in_a_change(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
             <p data-field=\"note\">See <a href=\"#req-target\">it</a>.</p></section>",
            declaring(&[TARGET]),
            declaring(&[TARGET]),
        );
        let found = fired(&report, UNRESOLVED);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0]
                .message
                .contains("no unit in the document declares that identifier"),
            "{}",
            found[0].message
        );
        assert_eq!(
            fired(&report, BROKEN_LINK).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_dangling_link_reference_is_reported_like_an_attribute_one() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
             <p data-field=\"note\">See <a href=\"#req-missing\">it</a>.</p></section>",
        );
        // The pointer is both a reference and a link (section 5.2), so both rules
        // fire: the relationship records a nonexistent unit, and the hyperlink
        // arrives nowhere.
        assert_eq!(fired(&report, UNRESOLVED).len(), 1, "{}", messages(&report));
        assert_eq!(
            fired(&report, BROKEN_LINK).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_broken_fragment_link_is_reported() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
             <p data-field=\"note\">See <a href=\"#section-nowhere\">it</a>.</p></section>",
        );
        let found = fired(&report, BROKEN_LINK);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        // Not a reference: the fragment is outside the unit-identifier grammar.
        assert!(
            fired(&report, UNRESOLVED).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// A figure is addressable by a link although it is not a semantic unit.
    #[test]
    fn a_link_to_a_figure_resolves() {
        let report = check(
            "<figure id=\"figure-x\"><figcaption>A figure.</figcaption><p>Prose.</p></figure>\
             <section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
             <p data-field=\"note\">See <a href=\"#figure-x\">the figure</a>.</p></section>",
        );
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_reference_to_a_removed_unit_is_reported() {
        let report = check(
            "<section class=\"requirement\" id=\"req-withdrawn\" data-change=\"removed\">\
             <h2>R</h2><p>The system MUST work.</p>\
             <article class=\"scenario\" id=\"scenario-withdrawn\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>\
             <section class=\"task\" id=\"task-x\" data-state=\"todo\" \
             data-refs=\"req-withdrawn\"><h2>T</h2></section>",
        );
        let found = fired(&report, REMOVED_TARGET);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(found[0].location.unit_id.as_deref(), Some("task-x"));
        // It resolves, so it is not also reported as unresolved. One defect, one
        // diagnostic.
        assert!(
            fired(&report, UNRESOLVED).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// A removed unit stays in the document so a reviewer can see the withdrawal,
    /// which is only useful if a link to it still resolves.
    #[test]
    fn a_link_to_a_removed_unit_still_resolves() {
        let report = check(
            "<section class=\"requirement\" id=\"req-withdrawn\" data-change=\"removed\">\
             <h2>R</h2><p>The system MUST work.</p>\
             <article class=\"scenario\" id=\"scenario-withdrawn\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>\
             <p>See <a href=\"#req-withdrawn\">the withdrawn requirement</a>.</p>",
        );
        assert!(
            fired(&report, BROKEN_LINK).is_empty(),
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn every_dangling_reference_is_reported_in_one_pass() {
        let report = check(
            "<section class=\"task\" id=\"task-a\" data-state=\"todo\" \
             data-refs=\"req-one req-two\"><h2>A</h2></section>\
             <section class=\"task\" id=\"task-b\" data-state=\"todo\" \
             data-refs=\"req-three\"><h2>B</h2></section>",
        );
        assert_eq!(fired(&report, UNRESOLVED).len(), 3, "{}", messages(&report));
    }

    /// If a schema rename broke this lookup, every reference to a removed unit
    /// would silently start passing.
    #[test]
    fn the_removed_state_is_found_in_the_schema() {
        assert_eq!(
            removed_state().as_deref(),
            Some("removed"),
            "the removed-target rule cannot find its state and has stopped firing"
        );
    }

    #[test]
    fn every_dogfood_document_satisfies_the_reference_rules() {
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
                "{path:?} failed the reference rules:\n{}",
                messages(&report)
            );
        }
    }
}
