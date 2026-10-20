//! `asset/*`: a document's asset references name the bytes stored beside it.
//!
//! A document references its runtime by relative path, which is what lets it open
//! from disk with no network. Content addressing makes that reference a *claim*:
//! `assets/sha256-3e4014c3004abea5/htmlspec.css` says "the stylesheet I was authored
//! against hashes to 3e4014c3…". These rules check the claim against the store.
//!
//! Two faults, two identifiers, because the causes differ and so do the repairs.
//! [`UNRESOLVED`] means nothing is stored at the address — the bytes were never
//! written, or an archived document names a revision the project no longer has.
//! [`MISMATCH`] means bytes *are* there and hash to something else — something
//! replaced them in place, which is the more alarming of the two because the
//! document renders and looks fine. Collapsing them into one diagnostic would tell a
//! contributor a reference is wrong without telling them which repair applies.
//!
//! **Scope is the store's presence and nothing else.** A conformance fixture
//! references `assets/htmlspec.css` and no such file has ever sat beside it: the
//! reference is symbolic, present so the parser sees a realistic document. Resolving
//! against a store that is absent would reject every fixture in the suite, and those
//! inputs are the pin for every rule this project has. So the rules read
//! [`Store::is_present`] and return early when there is no store.
//!
//! A document **cannot exempt itself**. There is no marker, class, or attribute this
//! module reads — the scope is a property of the surroundings, not a switch the
//! document carries, because a rule a document can switch off protects nothing. The
//! rule exists because a real archived page shipped against a stylesheet it did not
//! match, and a `class="fixture"` escape hatch would have let that page keep
//! shipping. `a_marker_inside_a_document_cannot_exempt_it` pins this.
//!
//! These are errors and cannot be downgraded — [`Namespace::is_downgradable`]
//! enforces it. A document rendering against bytes other than the ones it names is
//! misreporting its own provenance, and that is not a matter of project preference.

use htmlspec_assets::{Resolution, Store, REFERENCING_ATTRIBUTES};
use kuchikiki::iter::NodeIterator;
use kuchikiki::NodeRef;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};

/// A reference names bytes the store does not hold.
pub const UNRESOLVED: RuleId = RuleId::new(Namespace::Asset, "unresolved");

/// A reference resolves to stored bytes that hash differently from its name.
pub const MISMATCH: RuleId = RuleId::new(Namespace::Asset, "mismatch");

/// Runs every asset rule.
///
/// Returns immediately when no store sits beside the document. That early return is
/// the whole scoping mechanism, and it is deliberately the first thing here: a reader
/// asking "why did this not fire on a fixture?" finds the answer in one line.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let Some(document) = subject.projection.document.as_ref() else {
        // No tree, so there are no references to read. A projection that could not
        // read the document already reports a `schema/*` fault, and reporting every
        // reference as unresolved on top of that would bury it.
        return;
    };

    if !subject.store.is_present() {
        // No store found anywhere above the document. Almost always that means the
        // document claims no provenance — a conformance fixture, whose references are
        // symbolic and whose store has never existed.
        //
        // But not always, and the exception has to be caught here or nowhere. A reference
        // that *ascends* out of its own directory asserts a store sits above the document,
        // and no store above it contradicts the reference rather than placing the document
        // outside this rule. Measured: a change directory copied away from its project
        // carried seven dead references and `--strict` reported zero diagnostics.
        orphaned_references(document, diagnostics);
        return;
    }
    references_match_stored_bytes(document, subject.store, diagnostics);
}

/// Ascending references with no store above them.
///
/// Scoped by the **shape of the reference**, which is what separates the two cases exactly.
/// None of the 341 asset references in the conformance fixtures ascends, and every
/// reference the toolchain generates for a document inside a project does — so this fires on
/// an orphaned change and stays silent on a fixture.
///
/// A document cannot use this to exempt itself. Reference shape is written by whatever
/// generated the document, and editing a reference to stop ascending breaks the reference
/// it describes: the document would then name a store beside itself that is not there.
fn orphaned_references(document: &NodeRef, diagnostics: &mut Diagnostics) {
    for (element, attribute_name, reference) in referenced_assets(document) {
        if !ascends(&reference) {
            continue;
        }
        // Only an addressed reference makes a claim about bytes. An ascending path to
        // something unaddressed is a document or an image, and not this rule's subject.
        if !names_an_address(&reference) {
            continue;
        }
        diagnostics.violation(
            UNRESOLVED,
            location(&element, attribute_name),
            format!(
                "<{element} {attribute_name}=\"{reference}\"> reaches above this document for \
                 an asset store, and there is none in any directory above it. The reference \
                 says the store is there, so either this document was separated from the \
                 project holding its store — a change directory copied out of its project \
                 renders against nothing — or the store was removed"
            ),
        );
    }
}

/// Whether a reference climbs out of its own directory.
fn ascends(reference: &str) -> bool {
    reference.split('/').any(|component| component == "..")
}

/// Whether a reference names a content address.
///
/// Read from the reference's own text rather than resolved against a store, because in this
/// case there is no store to resolve against — which is the whole situation being reported.
fn names_an_address(reference: &str) -> bool {
    let prefix = format!("{}-", htmlspec_assets::address::ALGORITHM);
    reference
        .split('/')
        .any(|component| component.starts_with(&prefix))
}

/// Every addressed reference resolves to bytes whose hash it names.
fn references_match_stored_bytes(document: &NodeRef, store: &Store, diagnostics: &mut Diagnostics) {
    for (element, attribute_name, reference) in referenced_assets(document) {
        match store.resolve(&reference) {
            // Not an addressed reference, so this rule has nothing to say about it.
            // A document may reference an unaddressed path, another document, or an
            // anchor; none of those claims a hash.
            Resolution::NotAddressed | Resolution::Match => {}
            Resolution::Absent => diagnostics.violation(
                UNRESOLVED,
                location(&element, attribute_name),
                format!(
                    "<{element} {attribute_name}=\"{reference}\"> names bytes this project does \
                     not store. The address is a claim about the bytes it points at, and nothing \
                     is stored there — so the document would render against a file that is \
                     missing, or against whatever later takes that path"
                ),
            ),
            Resolution::Elsewhere { leads_to, store } => diagnostics.violation(
                UNRESOLVED,
                location(&element, attribute_name),
                format!(
                    "<{element} {attribute_name}=\"{reference}\"> leads to {leads_to} from this \
                     document, and the project's store is at {store}. A browser follows the path, \
                     not the address at its end, so the document loads nothing from the project's \
                     store. A change moved without its references being re-expressed leaves this \
                     behind: point the reference at {store}"
                ),
            ),
            Resolution::Mismatch { addressed, found } => diagnostics.violation(
                MISMATCH,
                location(&element, attribute_name),
                format!(
                    "<{element} {attribute_name}=\"{reference}\"> names {addressed} and the bytes \
                     stored there hash to {found}. Something replaced the stored file in place, \
                     so the document renders against content it does not describe — which is \
                     worse than a missing file, because nothing looks wrong"
                ),
            ),
        }
    }
}

/// Where an asset diagnostic points.
///
/// An asset reference sits in the document head or on a figure, not inside a semantic
/// unit, so there is no unit identifier to name. The element and attribute are what a
/// contributor searches for, in the same form `safety/*` reports them.
fn location(element: &str, attribute_name: &str) -> Location {
    Location::default().at_schema_path(format!("/{element}[{attribute_name}]"))
}

/// Every `(element, attribute, value)` in the tree that references an asset.
///
/// The pairs read are [`REFERENCING_ATTRIBUTES`], which the formatter reads too when it fills
/// a reference. The formatter finds them in the source rather than the tree, and the tests
/// below hold what it finds to what this finds.
fn referenced_assets(document: &NodeRef) -> Vec<(String, &'static str, String)> {
    let mut found = Vec::new();
    for element in document.inclusive_descendants().elements() {
        let name = element.name.local.as_ref().to_lowercase();
        for referencing in REFERENCING_ATTRIBUTES {
            if name != referencing.element {
                continue;
            }
            let attributes = element.attributes.borrow();
            let Some(value) = attributes.get(referencing.attribute) else {
                continue;
            };
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            found.push((name.clone(), referencing.attribute, value.to_owned()));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};
    use htmlspec_assets::{address, sha256_hex};
    use htmlspec_parse::parse_document;
    use std::path::PathBuf;

    /// A scratch project holding a real store, removed when the test ends.
    ///
    /// The rules resolve against a [`Store`], and a store is discovered from a
    /// directory — so these tests write real bytes rather than constructing a store
    /// by hand. A hand-built store would let a test pass while `discover` disagreed
    /// with it, and `discover` is what the CLI uses.
    struct Project(PathBuf);

    impl Project {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-assetrule-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(path.join("assets")).expect("the scratch store is created");
            Self(path)
        }

        /// Stores `bytes` at their own address and returns the reference a document
        /// would use.
        fn store(&self, name: &str, bytes: &[u8]) -> String {
            let stored = address::stored_path(&sha256_hex(bytes), name);
            let path = self.0.join("assets").join(&stored);
            std::fs::create_dir_all(path.parent().expect("an address directory"))
                .expect("the address exists");
            std::fs::write(&path, bytes).expect("the asset is written");
            format!("assets/{stored}")
        }

        /// The reference a document would use for bytes that are *not* stored.
        fn reference_for(&self, name: &str, bytes: &[u8]) -> String {
            format!("assets/{}", address::stored_path(&sha256_hex(bytes), name))
        }

        /// Writes `bytes` at the address of `addressed`, which is the in-place
        /// replacement a mismatch reports.
        fn store_wrong(&self, name: &str, addressed: &[u8], actual: &[u8]) -> String {
            let stored = address::stored_path(&sha256_hex(addressed), name);
            let path = self.0.join("assets").join(&stored);
            std::fs::create_dir_all(path.parent().expect("an address directory"))
                .expect("the address exists");
            std::fs::write(&path, actual).expect("the asset is written");
            format!("assets/{stored}")
        }

        fn store_of(&self) -> Store {
            Store::discover(&self.0)
        }
    }

    impl Drop for Project {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Validates a document whose head carries `head`, against `store`.
    fn check(head: &str, store: &Store) -> crate::Report {
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>Fixture</title>{head}</head>\n<body>\n<p>Body.</p>\n</body>\n</html>\n"
        );
        let outcome = parse_document(&html);
        assert!(
            outcome.is_recovery_free(),
            "the fixture itself recovered: {:?}",
            outcome.recovered
        );
        let projection = htmlspec_parse::project(&outcome.document);
        run(
            &outcome,
            &projection,
            store,
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

    /// A stylesheet link for `reference`.
    fn link(reference: &str) -> String {
        format!("<link rel=\"stylesheet\" href=\"{reference}\">")
    }

    // -------------------------------------------------------------------------
    // Task 3.4: scope
    // -------------------------------------------------------------------------

    /// The case every existing conformance fixture is in, and the reason the rules
    /// are scoped at all. If this fails, the 37 fixtures fail with it.
    #[test]
    fn a_reference_in_a_project_with_no_store_is_not_resolved() {
        let report = check(
            &link("assets/sha256-0123456789abcdef/htmlspec.css"),
            &Store::Absent,
        );
        assert!(
            report.is_accepted(),
            "an absent store must not resolve references:\n{}",
            messages(&report)
        );
    }

    /// The unaddressed form the existing fixtures actually use.
    #[test]
    fn an_unaddressed_reference_is_not_this_rules_business() {
        let project = Project::new("unaddressed");
        project.store("htmlspec.css", b"stored bytes");
        let store = project.store_of();
        // A store *is* present, so the rules are in scope — and these references
        // still report nothing, because none of them claims a hash.
        for reference in [
            "assets/htmlspec.css",
            "assets/vendor/mermaid.min.js",
            "other.html",
        ] {
            let report = check(&link(reference), &store);
            assert!(
                report.is_accepted(),
                "{reference} was resolved and should not have been:\n{}",
                messages(&report)
            );
        }
    }

    // -------------------------------------------------------------------------
    // An ascending reference with no store above it
    // -------------------------------------------------------------------------

    /// The measured defect. A change directory copied out of its project keeps references
    /// that climb three levels for a store that is no longer there, and strict validation
    /// used to accept it with zero diagnostics.
    #[test]
    fn an_ascending_reference_with_no_store_above_it_is_reported() {
        let report = check(
            &link("../../../assets/sha256-0123456789abcdef/htmlspec.css"),
            &Store::Absent,
        );
        assert!(
            !report.is_accepted(),
            "an ascending reference with no store above it was accepted:\n{}",
            messages(&report)
        );
        assert_eq!(fired(&report, UNRESOLVED).len(), 1);
        // The report says what happened, so a contributor knows the document moved rather
        // than that its markup is wrong.
        assert!(
            messages(&report).contains("reaches above this document"),
            "{}",
            messages(&report)
        );
    }

    /// The distinction the rule rests on, asserted as a pair. A fixture's reference does not
    /// ascend and stays silent; an orphan's does and reports. Both with no store.
    #[test]
    fn ascent_is_what_separates_an_orphan_from_a_fixture() {
        let fixture = check(
            &link("assets/sha256-0123456789abcdef/htmlspec.css"),
            &Store::Absent,
        );
        assert!(
            fixture.is_accepted(),
            "a non-ascending reference was reported, which would fail every fixture:\n{}",
            messages(&fixture)
        );

        let orphan = check(
            &link("../assets/sha256-0123456789abcdef/htmlspec.css"),
            &Store::Absent,
        );
        assert!(
            !orphan.is_accepted(),
            "one level of ascent was not enough to report:\n{}",
            messages(&orphan)
        );
    }

    /// An ascending reference to something *unaddressed* claims no bytes, so it is not this
    /// rule's subject even when it ascends.
    #[test]
    fn an_ascending_unaddressed_reference_is_not_reported() {
        for reference in ["../assets/htmlspec.css", "../../other.html", "../logo.png"] {
            let report = check(&link(reference), &Store::Absent);
            assert!(
                report.is_accepted(),
                "{reference} claims no address and must not be reported:\n{}",
                messages(&report)
            );
        }
    }

    /// Reference shape is not an escape hatch. Editing a reference to stop ascending does
    /// not silence the rule for free — the document then names a store beside itself, and
    /// with one present that reference is resolved and reported.
    #[test]
    fn flattening_a_reference_to_escape_the_rule_breaks_the_reference() {
        let project = Project::new("flattened");
        project.store("htmlspec.css", b"stored bytes");
        let store = project.store_of();

        // The document rewrote `../../assets/...` to `assets/...` to stop ascending. Now a
        // store is beside it, the reference resolves, and it names bytes that are not there.
        let report = check(&link("assets/sha256-0123456789abcdef/htmlspec.css"), &store);
        assert!(
            !report.is_accepted(),
            "flattening the reference silenced the rule with no consequence:\n{}",
            messages(&report)
        );
    }

    /// The rule fires once per broken reference, so a contributor sees the whole list.
    #[test]
    fn every_ascending_reference_is_reported() {
        let head = format!(
            "{}{}",
            link("../../assets/sha256-0123456789abcdef/htmlspec.css"),
            "<script src=\"../../assets/sha256-fedcba9876543210/htmlspec.js\"></script>"
        );
        let report = check(&head, &Store::Absent);
        assert_eq!(
            fired(&report, UNRESOLVED).len(),
            2,
            "only some of the dead references were reported:\n{}",
            messages(&report)
        );
    }

    // -------------------------------------------------------------------------
    // Task 3.6: a document cannot exempt itself
    // -------------------------------------------------------------------------

    /// The scope is a property of the surroundings, never a switch the document
    /// carries. Every marker a contributor might reach for is tried, and each must
    /// leave the violation reported.
    ///
    /// This is the test that keeps the rejected `class="fixture"` design rejected. If
    /// someone reintroduces a document-side exemption, this fails.
    #[test]
    fn a_marker_inside_a_document_cannot_exempt_it() {
        let project = Project::new("no-exemption");
        project.store("htmlspec.css", b"the stored revision");
        let store = project.store_of();
        let missing = project.reference_for("htmlspec.css", b"a revision never stored");

        let markers = [
            "<meta name=\"htmlspec-fixture\" content=\"true\">",
            "<meta name=\"htmlspec-assets\" content=\"unresolved\">",
            "<meta name=\"htmlspec-skip-asset-check\" content=\"yes\">",
        ];
        for marker in markers {
            let report = check(&format!("{marker}{}", link(&missing)), &store);
            assert_eq!(
                fired(&report, UNRESOLVED).len(),
                1,
                "the marker {marker} suppressed the rule:\n{}",
                messages(&report)
            );
        }

        // And on the referencing element itself, which is the other place a
        // suppression switch would naturally be put.
        let report = check(
            &format!(
                "<link class=\"fixture\" data-htmlspec-ignore=\"true\" rel=\"stylesheet\" \
                 href=\"{missing}\">"
            ),
            &store,
        );
        assert_eq!(
            fired(&report, UNRESOLVED).len(),
            1,
            "an attribute on the element suppressed the rule:\n{}",
            messages(&report)
        );
    }

    // -------------------------------------------------------------------------
    // Task 3.1: absent bytes
    // -------------------------------------------------------------------------

    #[test]
    fn a_reference_matching_stored_bytes_is_accepted() {
        let project = Project::new("match");
        let reference = project.store("htmlspec.css", b"--hs-measure: 36rem;");
        let report = check(&link(&reference), &project.store_of());
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_reference_to_bytes_the_project_does_not_store_is_unresolved() {
        let project = Project::new("absent");
        project.store("htmlspec.css", b"one revision");
        let missing = project.reference_for("htmlspec.css", b"another revision");

        let report = check(&link(&missing), &project.store_of());
        let found = fired(&report, UNRESOLVED);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains(&missing),
            "the report does not name the reference: {}",
            found[0].message
        );
        // The two faults are distinguishable, so the absent case is not also a
        // mismatch.
        assert!(fired(&report, MISMATCH).is_empty(), "{}", messages(&report));
    }

    // -------------------------------------------------------------------------
    // Task 3.2: stored bytes disagreeing with the name
    // -------------------------------------------------------------------------

    /// The distinguishability the specs require: absent bytes were never written, a
    /// mismatch was replaced, and the repairs differ.
    #[test]
    fn stored_bytes_disagreeing_with_their_name_are_a_mismatch_not_an_absence() {
        let project = Project::new("mismatch");
        let reference = project.store_wrong("htmlspec.css", b"the addressed bytes", b"other bytes");

        let report = check(&link(&reference), &project.store_of());
        let found = fired(&report, MISMATCH);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            fired(&report, UNRESOLVED).is_empty(),
            "a mismatch was also reported as absent, so the two are not distinguishable:\n{}",
            messages(&report)
        );
        // The report names the digest actually present, which is what tells a
        // contributor the file was replaced rather than never written.
        let actual = sha256_hex(b"other bytes");
        assert!(
            found[0]
                .message
                .contains(&actual[..address::ADDRESS_HEX_LEN]),
            "the report does not name the digest found: {}",
            found[0].message
        );
    }

    /// The defect that started this change: a document authored against one
    /// stylesheet revision, stored beside another. Task 3.8's fixture pins the same
    /// property from outside the crate.
    #[test]
    fn the_stale_stylesheet_defect_is_caught() {
        let project = Project::new("stale");
        // The project stores the current stylesheet, which has the skip-link rule.
        project.store("htmlspec.css", b".hs-skip-link { left: -9999px }");
        // The archived document names the older revision, which has no such rule.
        let older = project.reference_for("htmlspec.css", b"/* no skip link rule */");

        let report = check(&link(&older), &project.store_of());
        assert!(
            !report.is_accepted(),
            "a document naming bytes the project does not store must not pass:\n{}",
            messages(&report)
        );
        assert_eq!(fired(&report, UNRESOLVED).len(), 1, "{}", messages(&report));
    }

    // -------------------------------------------------------------------------
    // Task 3.3: non-downgradable errors
    // -------------------------------------------------------------------------

    /// Both rules are errors under every strictness, and neither can be reduced to a
    /// warning. Asserted through the engine, which is where policy is applied.
    #[test]
    fn an_asset_violation_is_an_error_under_every_policy() {
        let project = Project::new("severity");
        let missing = project.reference_for("htmlspec.css", b"never stored");
        let replaced = project.store_wrong("htmlspec.js", b"addressed", b"actual");
        let store = project.store_of();

        for strictness in [Strictness::Default, Strictness::Strict] {
            let html = format!(
                "<!doctype html>\n<html lang=\"en\">\n<head>\
                 <meta charset=\"utf-8\">\
                 <meta name=\"htmlspec-version\" content=\"0.1.0\">\
                 <meta name=\"htmlspec-kind\" content=\"specification\">\
                 <title>T</title>{}<script src=\"{replaced}\"></script>\
                 </head>\n<body>\n<p>Body.</p>\n</body>\n</html>\n",
                link(&missing)
            );
            let outcome = parse_document(&html);
            let projection = htmlspec_parse::project(&outcome.document);
            let report = run(
                &outcome,
                &projection,
                &store,
                &crate::references::Scope::loose(),
                &[validate],
                strictness,
            );

            assert!(
                !report.is_accepted(),
                "an asset violation did not fail validation under {strictness:?}:\n{}",
                messages(&report)
            );
            for diagnostic in &report.diagnostics {
                assert!(
                    diagnostic.is_error(),
                    "{} was not an error under {strictness:?}, so it was downgradable",
                    diagnostic.rule
                );
            }
        }
    }

    /// The namespace refuses downgrading, which is what makes the rule above hold
    /// for any future rule in it rather than only for these two.
    #[test]
    fn the_asset_namespace_refuses_to_be_downgraded() {
        assert!(!Namespace::Asset.is_downgradable());
        for rule in [UNRESOLVED, MISMATCH] {
            assert_eq!(rule.namespace(), Namespace::Asset, "{rule}");
        }
    }

    // -------------------------------------------------------------------------
    // Breadth
    // -------------------------------------------------------------------------

    /// Every reference is reported in one pass, not just the first.
    #[test]
    fn every_bad_reference_is_reported_in_one_pass() {
        let project = Project::new("many");
        let a = project.reference_for("htmlspec.css", b"absent one");
        let b = project.reference_for("htmlspec-print.css", b"absent two");
        let c = project.store_wrong("htmlspec.js", b"addressed", b"replaced");

        let report = check(
            &format!("{}{}<script src=\"{c}\"></script>", link(&a), link(&b)),
            &project.store_of(),
        );
        assert_eq!(fired(&report, UNRESOLVED).len(), 2, "{}", messages(&report));
        assert_eq!(fired(&report, MISMATCH).len(), 1, "{}", messages(&report));
    }

    /// A document several directories below the store references it by a relative
    /// path, and depth must not defeat the rule.
    #[test]
    fn a_reference_from_a_nested_document_resolves() {
        let project = Project::new("nested");
        let reference = project.store("htmlspec.css", b"bytes");
        let deep = format!("../../{reference}");
        let report = check(&link(&deep), &project.store_of());
        assert!(
            report.is_accepted(),
            "depth is the document's business, not the store's:\n{}",
            messages(&report)
        );
    }

    /// The defect archival exposed: a reference whose address is stored and whose path leads
    /// somewhere else.
    ///
    /// An archived change's documents referenced `../../assets/`, which from the archive is a
    /// directory that does not exist, and the rule accepted them because the address and the file
    /// name at the end of the path were stored above the document. Seen from the document, the
    /// store is three levels up, and the reference is unresolved.
    #[test]
    fn a_reference_whose_path_reaches_another_directory_is_unresolved() {
        let project = Project::new("elsewhere");
        let reference = project.store("htmlspec.css", b"bytes");
        let misplaced = format!("../../{reference}");
        let store = project.store_of().seen_from(&project.0.join("a/b/c"));

        let report = check(&link(&misplaced), &store);
        let found = fired(&report, UNRESOLVED);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains("../../assets/")
                && found[0].message.contains("../../../assets/"),
            "the report does not name where the path leads and where the store is: {}",
            found[0].message
        );
        assert!(fired(&report, MISMATCH).is_empty(), "{}", messages(&report));
    }

    /// The same reference, spelled to reach the store, is accepted, however it is spelled.
    #[test]
    fn a_reference_whose_path_reaches_the_store_is_accepted() {
        let project = Project::new("reaches");
        let reference = project.store("htmlspec.css", b"bytes");
        let store = project.store_of().seen_from(&project.0.join("a/b/c"));
        for spelled in [
            format!("../../../{reference}"),
            format!("./../../../{reference}"),
            format!("../../elsewhere/../../{reference}"),
        ] {
            let report = check(&link(&spelled), &store);
            assert!(report.is_accepted(), "{spelled}:\n{}", messages(&report));
        }
    }

    /// A reference beside its store, from a document beside it, reaches it.
    #[test]
    fn a_reference_from_beside_the_store_reaches_it() {
        let project = Project::new("beside");
        let reference = project.store("htmlspec.css", b"bytes");
        let report = check(&link(&reference), &project.store_of().seen_from(&project.0));
        assert!(report.is_accepted(), "{}", messages(&report));

        let report = check(
            &link(&format!("../{reference}")),
            &project.store_of().seen_from(&project.0),
        );
        assert_eq!(fired(&report, UNRESOLVED).len(), 1, "{}", messages(&report));
    }

    /// A script reference is checked as well as a stylesheet: the whole runtime is
    /// addressed, and a stale script is as wrong as a stale stylesheet.
    #[test]
    fn a_script_reference_is_checked_too() {
        let project = Project::new("script");
        let missing = project.reference_for("htmlspec.js", b"never stored");
        let report = check(
            &format!("<script src=\"{missing}\"></script>"),
            &project.store_of(),
        );
        assert_eq!(fired(&report, UNRESOLVED).len(), 1, "{}", messages(&report));
    }

    /// An empty store is present, so the rules apply and an addressed reference
    /// resolves to nothing. This is the distinction `Store::discover` exists to draw.
    #[test]
    fn an_empty_store_is_present_and_resolves_references() {
        let project = Project::new("empty");
        let missing = project.reference_for("htmlspec.css", b"anything");
        let store = project.store_of();
        assert!(store.is_present(), "the assets directory exists");

        let report = check(&link(&missing), &store);
        assert_eq!(
            fired(&report, UNRESOLVED).len(),
            1,
            "a present-but-empty store must still resolve:\n{}",
            messages(&report)
        );
    }

    /// A hyperlink is a citation rather than a dependency, so it claims nothing about
    /// bytes even when its path looks addressed.
    #[test]
    fn a_hyperlink_is_not_an_asset_reference() {
        let project = Project::new("hyperlink");
        project.store("htmlspec.css", b"stored");
        let missing = project.reference_for("other.html", b"never stored");

        let outcome = parse_document(&format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head>\n<body>\n\
             <p>See <a href=\"{missing}\">the other document</a>.</p>\n</body>\n</html>\n"
        ));
        let projection = htmlspec_parse::project(&outcome.document);
        let report = run(
            &outcome,
            &projection,
            &project.store_of(),
            &crate::references::Scope::loose(),
            &[validate],
            Strictness::Default,
        );
        assert!(
            report.is_accepted(),
            "a hyperlink was treated as an asset reference:\n{}",
            messages(&report)
        );
    }

    /// The references the formatter finds in `source`, in the terms [`referenced_assets`]
    /// reports them.
    fn found_by_the_formatter(source: &str) -> Vec<(String, &'static str, String)> {
        htmlspec_format::asset_references(source)
            .into_iter()
            .map(|reference| {
                (
                    reference.on.element.to_owned(),
                    reference.on.attribute,
                    reference.value.to_owned(),
                )
            })
            .collect()
    }

    /// Every HTML document the repository keeps.
    ///
    /// Read from the directories that hold the repository's HTML rather than from the whole
    /// checkout, so that a contributor's untracked notes elsewhere do not decide the test.
    ///
    /// A place that is missing fails the test rather than being skipped. With `openspec/`, one
    /// of these places until 0.30.1, moved aside, the test still passed, reading seven fewer
    /// documents and saying nothing.
    fn repository_documents() -> Vec<PathBuf> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let places = [
            "README.html",
            "crates/htmlspec-assets/assets",
            "fixtures",
            "htmlspec",
        ];
        for place in places {
            assert!(
                root.join(place).exists(),
                "the repository holds no {place}, so this test would read less than it says. \
                 Name where its documents are now"
            );
        }
        let mut pending: Vec<PathBuf> = places.iter().map(|place| root.join(place)).collect();
        let mut found = Vec::new();
        while let Some(path) = pending.pop() {
            if path.is_dir() {
                for entry in std::fs::read_dir(&path).expect("a readable directory") {
                    pending.push(entry.expect("a directory entry").path());
                }
            } else if path
                .extension()
                .is_some_and(|extension| extension == "html")
            {
                found.push(path);
            }
        }
        found.sort();
        found
    }

    /// The formatter fills exactly the references these rules judge, in every document the
    /// repository keeps.
    ///
    /// The formatter reads the source, because a fill must change one value's bytes and
    /// nothing else, and these rules read the tree. The tree is what the parser reads, so it
    /// is the authority on what is an element and what is text, and this holds the
    /// formatter to it. The formatter once searched the source for `href="`, and filled an
    /// author's quotation of one.
    ///
    /// A document the parser recovered an error in is skipped. The formatter refuses to
    /// format one, so what its rewrite would have filled is never written.
    #[test]
    fn the_formatter_finds_the_references_these_rules_judge_in_every_repository_document() {
        let mut compared = 0;
        let mut references = 0;
        for path in repository_documents() {
            let source = std::fs::read_to_string(&path).expect("a readable document");
            let outcome = parse_document(&source);
            if !outcome.is_recovery_free() {
                continue;
            }
            let judged = referenced_assets(&outcome.document);
            assert_eq!(
                found_by_the_formatter(&source),
                judged,
                "the formatter and the asset rules disagree about {}",
                path.display()
            );
            compared += 1;
            references += judged.len();
        }
        // Floors, so that a moved directory cannot make this pass by comparing nothing.
        assert!(compared >= 100, "only {compared} documents were compared");
        assert!(
            references >= 100,
            "only {references} references were compared"
        );
    }

    /// Bodies a generated document is built from. `{v}` is any value, and `{p}` a value that
    /// can stand unquoted.
    ///
    /// Each places a reference where a search of the source would find it, rightly or
    /// wrongly. None makes the parser recover, so every document built from them is one the
    /// formatter would format.
    const FRAGMENTS: &[&str] = &[
        "<link rel=\"stylesheet\" href=\"{v}\">",
        "<link rel='stylesheet' href='{v}'>",
        "<link title=\"a > b\" href=\"{v}\" rel=\"stylesheet\">",
        "<script src=\"{v}\"></script>",
        "<SCRIPT SRC=\"{v}\"></SCRIPT>",
        "<img alt=\"\" src={p}>",
        "<object data=\"{v}\"></object>",
        "<embed src=\"{v}\">",
        "<input type=\"image\" alt=\"\" src=\"{v}\">",
        "<video src=\"{v}\"><track src=\"{v}\"></video>",
        "<svg><use href=\"{v}\"></use><image href=\"{v}\"></image></svg>",
        "<svg><![CDATA[ <image href=\"{v}\"/> ]]></svg>",
        "<p><code>href=\"{v}\"</code> and <code>src=\"{v}\"</code></p>",
        "<pre><code>&lt;link href=\"{v}\"&gt;</code></pre>",
        "<!-- <script src=\"{v}\"></script> -->",
        "<script>const s = '<img src=\"{v}\">';</script>",
        "<style>/* <link href=\"{v}\"> */</style>",
        "<textarea><link href=\"{v}\"></textarea>",
        "<title>&lt;img src=\"{v}\"&gt;</title>",
        "<iframe src=\"{v}\"><link href=\"{v}\"></iframe>",
        "<noscript><img alt=\"\" src=\"{v}\"></noscript>",
        "<noembed><img alt=\"\" src=\"{v}\"></noembed>",
        "<noframes><img alt=\"\" src=\"{v}\"></noframes>",
        "<xmp><img alt=\"\" src=\"{v}\"></xmp>",
        "<template><link rel=\"stylesheet\" href=\"{v}\"></template>",
        "<p><a href=\"{v}\">a citation</a></p>",
        "<img alt=\"\" data-src=\"{v}\">",
        "<abbr title='written as href=\"{v}\"'>a link</abbr>",
        "<svg><style><use href=\"{v}\"></use></style></svg>",
        "<svg><title><link rel=\"stylesheet\" href=\"{v}\"></title></svg>",
        "<svg><template><use href=\"{v}\"></use></template></svg>",
        "<svg><foreignObject><style>/* <img src=\"{v}\"> */</style><img alt=\"\" src=\"{v}\"></foreignObject></svg>",
        "<math><mi><style>/* <img src=\"{v}\"> */</style></mi></math>",
        "<math><annotation-xml encoding=\"text/html\"><style><svg><use href=\"{v}\"></use></svg></style></annotation-xml></math>",
        "<math><annotation-xml><svg><title><style>/* <link href=\"{v}\"> */</style></title></svg></annotation-xml></math>",
        "<math><mrow><svg><title><style><use href=\"{v}\"></use></style></title></svg></mrow></math>",
        "<svg><math><mi><style><use href=\"{v}\"></use></style></mi></math></svg>",
        "<script><!--<script></script><img alt=\"\" src=\"{v}\"></script>",
        "<script><!--<script></script>--></script>",
        "<script><!--><script></script><img alt=\"\" src=\"{v}\">",
        "<math><mi><mglyph><style><use href=\"{v}\"></use></style></mglyph></mi></math>",
    ];

    /// Values a fragment may hold, padded and empty among them.
    const VALUES: &[&str] = &[
        "assets/htmlspec.css",
        "../../assets/sha256-03627137bcbb81c9/htmlspec.css",
        "  assets/htmlspec.js  ",
        "",
        "#a-fragment",
        "https://example.org/remote.css",
    ];

    /// Values that can stand unquoted.
    const PLAIN_VALUES: &[&str] = &[
        "assets/htmlspec.css",
        "../assets/sha256-84427613110957a7/htmlspec.js",
        "#a-fragment",
    ];

    /// A document whose body is the chosen fragments, in order.
    fn generated_document(pieces: &[(usize, usize, usize)]) -> String {
        let body: String = pieces
            .iter()
            .filter_map(|(fragment, value, plain)| {
                Some(
                    FRAGMENTS
                        .get(*fragment)?
                        .replace("{v}", VALUES.get(*value)?)
                        .replace("{p}", PLAIN_VALUES.get(*plain)?)
                        + "\n",
                )
            })
            .collect();
        format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title>\
             <link rel=\"stylesheet\" href=\"assets/htmlspec.css\"></head>\n<body>\n{body}</body>\n</html>\n"
        )
    }

    /// Every fragment, alone and with every value, on every run.
    ///
    /// The property test draws its combinations at random, so whether it meets a given fragment
    /// in a given run is chance. Each fragment is a case some reading got wrong, so each is
    /// checked here without leaving it to the draw.
    #[test]
    fn every_fragment_alone_is_read_as_the_tree_reads_it() {
        for fragment in 0..FRAGMENTS.len() {
            for value in 0..VALUES.len() {
                for plain in 0..PLAIN_VALUES.len() {
                    let source = generated_document(&[(fragment, value, plain)]);
                    let outcome = parse_document(&source);
                    assert!(
                        outcome.is_recovery_free(),
                        "fragment {fragment} makes the parser recover: {:?}\n{source}",
                        outcome.recovered
                    );
                    assert_eq!(
                        found_by_the_formatter(&source),
                        referenced_assets(&outcome.document),
                        "fragment {fragment}:\n{source}"
                    );
                }
            }
        }
    }

    proptest::proptest! {
        // The documents are small and both readings are fast. What fails is a combination,
        // such as a raw-text element after a template, so the case count is raised.
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(512))]

        /// The formatter and these rules agree on every generated document.
        #[test]
        fn the_formatter_finds_the_references_these_rules_judge_in_generated_documents(
            pieces in proptest::collection::vec(
                (0..FRAGMENTS.len(), 0..VALUES.len(), 0..PLAIN_VALUES.len()),
                0..8,
            )
        ) {
            let source = generated_document(&pieces);
            let outcome = parse_document(&source);
            proptest::prop_assert!(
                outcome.is_recovery_free(),
                "a fragment makes the parser recover, so it is outside what the formatter \
                 formats: {:?}\n{source}",
                outcome.recovered
            );
            proptest::prop_assert_eq!(
                found_by_the_formatter(&source),
                referenced_assets(&outcome.document),
                "{}",
                source
            );
        }
    }
}
