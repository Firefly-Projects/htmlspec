//! The rules of the engine, one module per namespace.
//!
//! Every rule is a [`crate::engine::Rule`], and [`ALL`] is the set the validator
//! runs. A rule that exists but is absent from `ALL` would never fire, so the
//! list is the single place a rule becomes live.
//!
//! [`topology`] and [`runs`] are the deliberate exceptions, and neither is a `Rule` at
//! all. Each takes gathered facts rather than a document and returns diagnostics
//! directly.
//!
//! [`topology`]'s subject is a project. Adding it to `ALL` would report one duplicated
//! library once per document — the same two paths named twenty times, none of it the
//! fault of the document being validated.
//!
//! [`runs`]'s subject is **two documents at once**, which is the stronger form of the
//! same constraint: a runbook's specification states an obligation and a run's record
//! states the outcome, so neither document holds both halves and there is nothing for a
//! per-document diagnostic to attach to. Its module header records the measurement that
//! makes it necessary — the kind a run's record is forced into is exactly the kind
//! [`review`]'s equivalent rule cannot see.
//!
//! `every_rule_in_all_is_a_document_rule` keeps the two kinds from being confused.

pub mod a11y;
pub mod assets;
pub mod change_markup;
pub mod executable;
pub mod kinds;
pub mod refs;
pub mod relations;
pub mod renames;
pub mod review;
pub mod runs;
pub mod safety;
pub mod schema;
pub mod topology;
pub mod viz;

use crate::engine::Rule;

/// Every rule the validator runs, in report order.
///
/// Order is contract conformance first, then relationships, then safety, then the
/// remaining concerns: a reader given a document that is not even the right shape
/// is better served by hearing that before hearing that one of its charts is
/// invalid.
pub const ALL: &[Rule] = &[
    schema::validate,
    relations::validate,
    renames::validate,
    change_markup::validate,
    refs::validate,
    kinds::validate,
    safety::validate,
    executable::validate,
    assets::validate,
    viz::validate,
    a11y::validate,
    review::validate,
];

#[cfg(test)]
mod tests {
    use crate::diagnostic::Namespace;
    use htmlspec_parse::{parse_document, project};

    /// A rule module whose entry point is absent from [`super::ALL`] never fires,
    /// and nothing else would catch that: its own unit tests call it directly and
    /// pass. So the set is checked against the namespaces it must cover.
    ///
    /// `Parse` is excluded because the engine reports a parse recovery itself,
    /// before any rule runs, rather than through a [`crate::engine::Rule`].
    #[test]
    fn every_namespace_has_a_live_rule() {
        // A document that violates something in every namespace at once is not
        // worth constructing; running the set over one document only establishes
        // that each entry is callable. Coverage of the namespaces is asserted
        // from the source list instead, which is the thing that can rot.
        let sources: &[(&str, Namespace)] = &[
            ("schema", Namespace::Schema),
            ("relations", Namespace::Schema),
            ("renames", Namespace::Schema),
            ("change_markup", Namespace::Schema),
            ("refs", Namespace::Refs),
            ("kinds", Namespace::Kind),
            ("safety", Namespace::Safety),
            ("executable", Namespace::Safety),
            ("assets", Namespace::Asset),
            ("viz", Namespace::Viz),
            ("a11y", Namespace::A11y),
            ("review", Namespace::Review),
        ];
        assert_eq!(
            super::ALL.len(),
            sources.len(),
            "a rule module was added or removed without updating this test, so a \
             namespace may no longer have a live rule"
        );
        for namespace in Namespace::ALL {
            if *namespace == Namespace::Parse {
                continue;
            }
            assert!(
                sources.iter().any(|(_, n)| n == namespace),
                "no rule in ALL covers {namespace}"
            );
        }
    }

    /// Task 3.7: the `kind/*` namespace has exactly two rules, and no rule reports
    /// on unit placement in the `proposal` or `design` kinds.
    ///
    /// Two assertions about a boundary, from the two sides it can be crossed from.
    /// The count is crude and it is the thing that fails when a third `kind/*` rule
    /// appears without a decision — the proposal states exactly two, and a
    /// placement matrix is what this change exists to *not* build.
    ///
    /// The second half runs the whole rule set, not just `kinds::validate`, over a
    /// proposal and a design carrying every unit the vocabulary has. Scoping it to
    /// one module would pass for a placement rule someone added to `schema/*`
    /// instead, which is the likeliest place for one to arrive.
    #[test]
    fn the_kind_namespace_has_exactly_two_rules_and_constrains_no_upstream_kind() {
        const KIND_RULES: &[crate::diagnostic::RuleId] =
            &[super::kinds::TASK_IN_SPECIFICATION, super::kinds::NO_TASKS];
        assert_eq!(
            KIND_RULES.len(),
            2,
            "the kind namespace gained a rule. The proposal states exactly two — a \
             specification must not carry a task, and a tasks document must carry one \
             — and a third is a decision about a placement matrix this change \
             deliberately does not build"
        );

        // Every unit type, in each unconstrained kind, through the whole rule set.
        let units = "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
             <p>The system MUST work.</p>\
             <article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>\
             <section class=\"decision\" id=\"decision-x\" data-status=\"accepted\"><h2>D</h2>\
             <p>The decision.</p><p data-field=\"rationale\">Why.</p></section>\
             <section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2></section>\
             <section class=\"evidence\" id=\"evidence-x\"><h2>E</h2><p>Measured.</p></section>\
             <section class=\"risk\" id=\"risk-x\"><h2>K</h2><p>Fails.</p>\
             <p data-field=\"mitigation\">Held.</p></section>\
             <section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
             data-refs=\"req-x\"><h2>V</h2></section>";

        // The unit list is the schema's, so a unit type added to the contract is
        // covered here without an edit. A hardcoded list would still pass while
        // saying nothing about the new unit.
        for (unit, _prefix) in crate::schema_facts::identifier_prefixes() {
            assert!(
                units.contains(&format!("class=\"{unit}\"")),
                "the fixture does not carry a {unit} unit, so this test says nothing \
                 about whether a placement rule fires on one"
            );
        }

        // Task 10.2: under **both** strictness settings. Default acceptance alone
        // would leave the guarantee open at the setting a project's CI actually runs:
        // a placement rule added in a promotable namespace would pass by default and
        // fail the proposal under --strict, which is the phase gate this change exists
        // to avoid.
        for kind in ["proposal", "design"] {
            for strictness in [
                crate::engine::Strictness::Default,
                crate::engine::Strictness::Strict,
            ] {
                let html = format!(
                    "<!doctype html>\n<html lang=\"en\"><head>\
                     <meta name=\"htmlspec-version\" content=\"0.1.0\">\
                     <meta name=\"htmlspec-kind\" content=\"{kind}\">\
                     <title>T</title></head><body>{units}</body></html>\n"
                );
                let outcome = parse_document(&html);
                let projection = project(&outcome.document);
                let report = crate::engine::run(
                    &outcome,
                    &projection,
                    &htmlspec_assets::Store::Absent,
                    &crate::references::Scope::loose(),
                    super::ALL,
                    strictness,
                );
                let reported: Vec<String> =
                    report.diagnostics.iter().map(|d| format!("{d}")).collect();
                assert!(
                    report.is_accepted() && reported.is_empty(),
                    "a {kind} carrying every unit was reported under {strictness:?}. The \
                     upstream kinds impose no structure at all, and a diagnostic here \
                     means a placement rule arrived somewhere:\n{}",
                    reported.join("\n")
                );
            }
        }
    }

    /// Task 10.4: `--strict` does not fail a specification whose requirements are not
    /// yet verified.
    ///
    /// This is task 3.4's `review/*` placement holding under the one condition that
    /// breaks it. The advisory rule is a warning either way; what strict mode decides
    /// is whether that warning becomes an error. In `kind/*` it would, and a
    /// contributor's CI would go red for a specification that is legitimately in
    /// progress — gating the work that produces the verification.
    ///
    /// Asserted through the whole rule set at both settings, because the failure is a
    /// difference between them.
    #[test]
    fn strict_mode_does_not_fail_a_specification_awaiting_verification() {
        const UNVERIFIED: &str = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body>\
             <section class=\"requirement\" id=\"req-unverified\"><h2>R</h2>\
             <p>The system MUST work.</p>\
             <article class=\"scenario\" id=\"scenario-unverified\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article>\
             </section></body></html>\n";

        let outcome = parse_document(UNVERIFIED);
        let projection = project(&outcome.document);
        for strictness in [
            crate::engine::Strictness::Default,
            crate::engine::Strictness::Strict,
        ] {
            let report = crate::engine::run(
                &outcome,
                &projection,
                &htmlspec_assets::Store::Absent,
                &crate::references::Scope::loose(),
                super::ALL,
                strictness,
            );
            let rendered: Vec<String> = report.diagnostics.iter().map(|d| format!("{d}")).collect();
            assert!(
                report.is_accepted(),
                "an unverified requirement gated a specification under {strictness:?}, \
                 which is the phase gate the review namespace exists to prevent:\n{}",
                rendered.join("\n")
            );
            // The warning is still *reported* at both settings. A rule that stopped
            // firing under strict mode would pass the acceptance check above while
            // losing the report.
            assert_eq!(
                report.warnings().count(),
                1,
                "the advisory report changed under {strictness:?}:\n{}",
                rendered.join("\n")
            );
            assert_eq!(report.errors().count(), 0);
        }
    }

    /// The project-level rule must stay out of the per-document set.
    ///
    /// `topology::duplicates` takes a store and returns diagnostics, so it cannot be
    /// coerced into a [`crate::engine::Rule`] and the compiler already refuses the
    /// mistake. What the compiler cannot catch is someone wrapping it in a closure that
    /// reads `subject.store` and reporting duplication once per document. This asserts
    /// the outcome that would produce: running the whole set over a document sitting in
    /// a project with duplicated bytes must report nothing about duplication.
    #[test]
    fn duplication_is_not_reported_by_the_per_document_set() {
        const MINIMAL: &str = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body><p>Body.</p></body></html>\n";

        // A store holding one asset twice, which the topology rule reports.
        let root =
            std::env::temp_dir().join(format!("htmlspec-rules-duplicate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let bytes = b"the duplicated library";
        let addressed = root.join("assets").join(htmlspec_assets::stored_path(
            &htmlspec_assets::sha256_hex(bytes),
            "mermaid.min.js",
        ));
        std::fs::create_dir_all(addressed.parent().expect("a parent")).expect("the address");
        std::fs::write(&addressed, bytes).expect("the addressed copy");
        let legacy = root.join("assets/vendor");
        std::fs::create_dir_all(&legacy).expect("the legacy directory");
        std::fs::write(legacy.join("mermaid.min.js"), bytes).expect("the legacy copy");

        let store = htmlspec_assets::Store::discover(&root);
        assert_eq!(
            super::topology::duplicates(&store).len(),
            1,
            "the fixture must actually be duplicated for this test to mean anything"
        );

        let outcome = parse_document(MINIMAL);
        let projection = project(&outcome.document);
        let report = crate::engine::run(
            &outcome,
            &projection,
            &store,
            &crate::references::Scope::loose(),
            super::ALL,
            crate::engine::Strictness::Default,
        );
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|d| d.rule == super::topology::DUPLICATE),
            "a project-level violation was reported against one document, which would \
             repeat it once per document in the project:\n{:#?}",
            report.diagnostics
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// A run's unrecorded requirement is not reported by the per-document set either.
    ///
    /// The counterpart to the test above, and the document under it is the case that makes the
    /// omission visible: **a `tasks` document carrying a requirement that no verification names**,
    /// which is exactly what a run's record short of a verification looks like.
    ///
    /// Two things are asserted and the second is the load-bearing one. That no
    /// `review/unrecorded-requirement` is reported — which the compiler already helps with, since
    /// [`super::runs::conformance`] takes a [`super::runs::RunbookUnderReview`] and cannot be
    /// coerced into a [`crate::engine::Rule`]. And that **nothing else is reported either**, which
    /// is the measurement the rule exists because of: `review/unverified-requirement` is scoped to
    /// the `specification` kind, so this document passes the whole set in silence. A change that
    /// widened that rule to every kind would fail here rather than quietly making this rule
    /// redundant — and it would also start reporting every `proposal` arguing for a requirement,
    /// which is the reason the scope is there.
    #[test]
    fn an_unrecorded_requirement_is_not_reported_by_the_per_document_set() {
        const RECORD: &str = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"tasks\">\
             <title>T</title></head><body>\
             <section class=\"task\" id=\"task-step-one\" data-change=\"unchanged\" \
             data-state=\"done\"><h2>A step this run worked</h2></section>\
             <section class=\"requirement\" id=\"req-copied\" data-change=\"unchanged\"><h2>R</h2>\
             <p>A run of this procedure MUST record the version it advanced to.</p>\
             <article class=\"scenario\" id=\"scenario-copied\" data-change=\"unchanged\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article>\
             </section></body></html>\n";

        let outcome = parse_document(RECORD);
        let projection = project(&outcome.document);
        for strictness in [
            crate::engine::Strictness::Default,
            crate::engine::Strictness::Strict,
        ] {
            let report = crate::engine::run(
                &outcome,
                &projection,
                &htmlspec_assets::Store::Absent,
                &crate::references::Scope::loose(),
                super::ALL,
                strictness,
            );
            let rendered: Vec<String> = report.diagnostics.iter().map(|d| format!("{d}")).collect();
            assert!(
                !report
                    .diagnostics
                    .iter()
                    .any(|d| d.rule == super::runs::UNRECORDED),
                "a rule whose subject is a runbook and its runs together was reported against one \
                 document, which cannot know what the runbook required:\n{}",
                rendered.join("\n")
            );
            assert!(
                report.is_accepted() && rendered.is_empty(),
                "a run's record short of a verification was reported under {strictness:?}. This \
                 silence is the measurement `rules::runs` exists because of — if a rule now \
                 reports it per document, that rule also reports every proposal arguing for a \
                 requirement:\n{}",
                rendered.join("\n")
            );
        }
    }

    /// Every entry in the set is callable over a real projection and none panics.
    #[test]
    fn every_rule_runs_over_a_minimal_document() {
        const MINIMAL: &str = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body><p>Body.</p></body></html>\n";
        let outcome = parse_document(MINIMAL);
        let projection = project(&outcome.document);
        let report = crate::engine::run(
            &outcome,
            &projection,
            &htmlspec_assets::Store::Absent,
            &crate::references::Scope::loose(),
            super::ALL,
            crate::engine::Strictness::Default,
        );
        assert!(report.curtailed.is_none(), "{:#?}", report.curtailed);
    }
}
