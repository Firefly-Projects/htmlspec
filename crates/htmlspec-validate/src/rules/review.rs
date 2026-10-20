//! `review/*`: an upstream edit marks dependents for review.
//!
//! Artifacts depend on each other, and an edit upstream does make dependents
//! suspect. That is not a phase gate. These are warnings that never fail
//! validation, never require regenerating an artifact, and clear when an agent
//! reconciles the affected units — which is what lets exploration, proposal,
//! design, implementation, and verification loop.
//!
//! The signal is the change state a unit already declares. A unit marked `added`
//! or `modified` has moved, so every unit referencing it may now describe
//! behavior the target no longer states. That is a *stale* dependency and not a
//! broken one: the reference resolves, which is why this is separate from
//! `refs/*`.
//!
//! Reconciliation clears the warning by the author's own act: once the upstream
//! unit is settled and its state returns to `unchanged`, the dependents stop
//! being reported. Nothing here asks for a timestamp or a stored hash, because a
//! document is the whole state and a second store would be a thing to keep in
//! sync.

use std::collections::BTreeMap;

use htmlspec_parse::Projection;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};
use crate::schema_facts;

/// A unit references a unit this change alters.
pub const STALE_REFERENCE: RuleId = RuleId::new(Namespace::Review, "stale-reference");

/// A verification records an outcome for a unit this change alters.
///
/// Separate from [`STALE_REFERENCE`] because it says something stronger: a
/// recorded pass against an obligation that has since moved is not merely
/// unreviewed, it is evidence for a claim nobody made.
pub const STALE_VERIFICATION: RuleId = RuleId::new(Namespace::Review, "stale-verification");

/// A requirement in a specification that no verification names.
///
/// In `review/*` rather than `kind/*`, and the placement is the decision rather
/// than a filing choice. Verification arrives as work completes, so a
/// specification in progress legitimately has requirements not yet verified. This
/// namespace is the one strict mode refuses to promote, so the report stays
/// advisory under every strictness — in `kind/*` it would be promotable, and
/// `--strict` would fail a specification for being in progress, gating the work
/// that produces the verification.
pub const UNVERIFIED_REQUIREMENT: RuleId = RuleId::new(Namespace::Review, "unverified-requirement");

/// A unit still carries an identifier a template writes as example content.
///
/// **In `review/*` rather than `kind/*`, for the reason [`UNVERIFIED_REQUIREMENT`] is.** The
/// condition this reports is normal at one moment and wrong at another: `htmlspec new document`
/// writes the template, and `document-template` requires the result to be valid, so a document
/// carries example content the instant it exists. An error there would report a defect the
/// program itself introduced. In `kind/*` the report would also be promotable, so `--strict`
/// would restore the error and fail a freshly created document anyway.
///
/// The severity split the contract requires is therefore between *this* report and archival's
/// refusal, rather than between two strictness levels. `contract-authority` states it: "Validation
/// SHALL report a document carrying a unit whose identifier is one a template writes as an
/// example. Archival SHALL refuse while a change's specification carries one."
///
/// The identifiers come from [`htmlspec_assets::example_identifiers`], which reads them out of
/// the templates. A list here would restate a fact the templates already make and would go stale
/// the first time one gains a unit.
pub const EXAMPLE_CONTENT: RuleId = RuleId::new(Namespace::Review, "example-content");

/// Runs every review rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    requirements_are_verified(projection, diagnostics);
    example_content_is_replaced(projection, diagnostics);

    let materially_changed = materially_changed_units(projection);
    if materially_changed.is_empty() {
        return;
    }

    for (source, reference) in projection.model.references() {
        let Some(state) = materially_changed.get(reference.target.as_str()) else {
            continue;
        };
        let location = Location::maybe_unit(source.id);

        // A verification that has actually run is the strong case. One that has
        // not run yet has nothing to go stale.
        let recorded_outcome = projection
            .model
            .verifications
            .iter()
            .find(|v| v.id.as_deref() == source.id)
            .and_then(|v| v.outcome.as_deref())
            .filter(|outcome| has_run(outcome));

        match recorded_outcome {
            Some(outcome) => diagnostics.violation(
                STALE_VERIFICATION,
                location,
                format!(
                    "this verification records the outcome {outcome} against {}, which this \
                     change marks {state}. Re-check it, or the record attests to an \
                     obligation that has since moved",
                    reference.target
                ),
            ),
            None => diagnostics.violation(
                STALE_REFERENCE,
                location,
                format!(
                    "this {} references {}, which this change marks {state}. Reconcile it \
                     when convenient; work continues either way",
                    source.kind, reference.target
                ),
            ),
        }
    }
}

/// A specification's requirements are each named by some verification.
///
/// Scoped to the `specification` kind alone, which is task 3.5 and is the point of
/// the rule rather than a narrowing of it. A specification is an as-built record,
/// so a requirement with nothing attesting to it records an intention rather than
/// a result. A requirement unit in a proposal or a design is not part of the
/// contract at all — the author is arguing for it or reasoning about it — so
/// checking it for verification would report a fault against a document doing its
/// job.
///
/// The kind is read from the model and resolved through the schema's vocabulary,
/// the way `rules::kinds` does, because the four kinds are the schema's to name.
fn requirements_are_verified(projection: &Projection, diagnostics: &mut Diagnostics) {
    let Some(kind) = projection.model.kind.as_deref() else {
        return;
    };
    if schema_facts::artifact_kind("specific").as_deref() != Some(kind) {
        return;
    }

    for requirement in &projection.model.requirements {
        // A requirement with no identifier cannot be named by a verification, and
        // the schema already reports the missing property. Reporting here as well
        // would offer a second diagnostic for one defect, and its advice —
        // "record a verification" — would be wrong: the fix is an identifier.
        let Some(id) = requirement.id.as_deref() else {
            continue;
        };
        let verified = projection
            .model
            .verifications
            .iter()
            .any(|v| v.references.iter().any(|r| r.target == id));
        if verified {
            continue;
        }
        diagnostics.violation(
            UNVERIFIED_REQUIREMENT,
            Location::unit(id),
            "no verification in this specification names this requirement. A \
             specification records what the change established, so a requirement with \
             nothing attesting to it states an intention rather than a result. Record \
             the verification when the work reaches it; this never gates the work",
        );
    }
}

/// A template's example units are replaced rather than kept.
///
/// **Within a unit, the check reads the identifier, not the example's text.** Text is edited while
/// an identifier survives, so a half-replaced example carries new prose under `req-example` — and
/// that is the likely case rather than an edge one. A text comparison would pass it.
///
/// Every kind is checked. The specification is the kind whose example content can reach the
/// accepted contract, so it is the one archival refuses on, but a tasks document shipping
/// `task-example` is a task nobody wrote and the report is worth having there too.
///
/// Reported once per unit and naming it, because the fix is to replace one unit and a reader
/// told only "this document has example content" has to find it.
///
/// **Figures are checked beside the units.** A figure is not a semantic unit — the model carries
/// it in its own collection and `Model::units()` does not yield it — but `figure-example` is one
/// of the identifiers the specification template writes. Reading only the units would leave one
/// of the eight uncovered, which the derivation from the templates exists to prevent.
///
/// **And the document's summary is checked on its text, because it has no identifier to survive.**
/// See [`summary_is_the_authors_own`]: the rule about identifiers does not narrow to a masthead
/// paragraph, it does not reach it at all, which is why a document whose entire summary read
/// "Replace this paragraph with what this work delivers." was reported as carrying no diagnostics.
fn example_content_is_replaced(projection: &Projection, diagnostics: &mut Diagnostics) {
    let mut report = |id: &str, kind: &dyn std::fmt::Display| {
        if !htmlspec_assets::is_example_identifier(id) {
            return;
        }
        diagnostics.violation(
            EXAMPLE_CONTENT,
            Location::unit(id),
            format!(
                "this {kind} still carries the identifier a template writes for its example \
                 content. Example content states no obligation and enumerates no work, so \
                 replace it with the document's own. This never gates the work, and archival \
                 refuses while a change's specification carries one"
            ),
        );
    };

    for unit in projection.model.units() {
        if let Some(id) = unit.id {
            report(id, &unit.kind);
        }
    }
    for figure in &projection.model.figures {
        if let Some(id) = figure.id.as_deref() {
            report(id, &"figure");
        }
    }

    summary_is_the_authors_own(projection, diagnostics);
}

/// The document's summary is the author's rather than the template's placeholder.
///
/// Under [`EXAMPLE_CONTENT`] rather than a rule of its own. A second identifier would need its
/// own coverage requirement and would divide one obligation — a template's generated content
/// carries no authority and its survival is reported — across two names.
///
/// **The comparison is against the whole summary, not a fragment of it.** A substring search would
/// report an author who wrote "unlike the template's 'Replace this paragraph' text, this summary
/// states the argument", and it would report every document in this project that discusses this
/// rule. A check that fires on documents about itself is one an agent learns to ignore. The
/// accepted cost is the other side of that: a masthead half-edited into the placeholder plus a
/// sentence of the author's own is not reported. That is the same trade the identifier check makes
/// in the other direction.
///
/// **Joined across paragraphs** before comparing, because a header carrying several projects
/// several entries and the summary is all of them. A per-paragraph comparison would report a
/// document whose author *added* their own paragraph beside the template's — which is a
/// half-edited masthead, the case above, and reporting it here would make the boundary depend on
/// whether the author edited in place or added below.
///
/// **Located at the document**, since the summary has no identifier. A contributor told "this
/// task still carries the identifier a template writes" for a paragraph with no identifier would
/// go looking for a unit that does not exist.
///
/// An absent summary is not reported. `header_summary` yields nothing for a document with no
/// `header` or no paragraph in it, projection 9.1.1 says that stays valid, and an absent summary
/// is a different fact from a placeholder one — reported as the other, it would send an author to
/// delete a paragraph that is not there.
///
/// **The early return below is not what carries that**, and saying so is the honest version:
/// `htmlspec_assets::is_example_placeholder("")` is false, because no template writes an empty
/// masthead paragraph, so deleting the return changes nothing. Mutation testing measured exactly
/// that — the return removed, and no test in the suite failed. It is kept for what it says to a
/// reader and because it stops the rule doing work on the common case of a fragment under review,
/// not because it is load-bearing. The property lives one layer down, where
/// `only_a_summary_that_is_the_placeholder_is_reported` asserts the empty string is not a
/// placeholder, and that assertion does fail when broken.
fn summary_is_the_authors_own(projection: &Projection, diagnostics: &mut Diagnostics) {
    if projection.model.summary.is_empty() {
        return;
    }
    let whole = projection.model.summary.join(" ");
    if !htmlspec_assets::is_example_placeholder(&whole) {
        return;
    }
    diagnostics.violation(
        EXAMPLE_CONTENT,
        Location::document(),
        "this document's summary is still the paragraph a template writes as a placeholder, so \
         the document states nothing about what it is for. Replace the masthead paragraph with \
         the document's own — it projects into the summary, which is where a consumer reads the \
         argument. This never gates the work, and archival refuses while a change's \
         specification carries one",
    );
}

/// Every unit this change adds or alters, keyed by identifier.
///
/// `removed` is absent on purpose: a reference to a withdrawn unit is a `refs/*`
/// error, and reporting it here as well would offer a warning for something that
/// already fails.
fn materially_changed_units(projection: &Projection) -> BTreeMap<&str, &str> {
    let material = material_states();
    if material.is_empty() {
        return BTreeMap::new();
    }
    projection
        .model
        .units()
        .into_iter()
        .filter(|unit| material.iter().any(|state| state == unit.change))
        .filter_map(|unit| unit.id.map(|id| (id, unit.change)))
        .collect()
}

/// The change states that make a dependent suspect, read from the schema.
///
/// A unit is materially changed when it is newly introduced or altered. That is
/// every state except the two that cannot make a dependent stale: the one meaning
/// "carried forward untouched", and the one meaning "withdrawn", which `refs/*`
/// owns. Deriving the set by exclusion rather than by naming two values means a
/// state added to the schema counts as material by default, which is the safe
/// direction: a new state would otherwise silently stop marking anything.
fn material_states() -> Vec<String> {
    schema_facts::enum_values("definitions/changeState")
        .into_iter()
        .filter(|state| !state.contains("unchang") && !state.contains("remov"))
        .collect()
}

/// Whether a verification outcome records a check that actually ran.
///
/// The outcome values are read from the schema. Only the "not run" outcome means
/// nothing was checked, so it is found by a stem and everything else counts as
/// having run.
fn has_run(outcome: &str) -> bool {
    let not_run: Vec<String> =
        schema_facts::enum_values("definitions/verification/properties/outcome")
            .into_iter()
            .filter(|value| value.contains("not"))
            .collect();
    !not_run.iter().any(|value| value == outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};

    /// These rules are not asset rules, so they run with no store beside them.
    const NO_STORE: htmlspec_assets::Store = htmlspec_assets::Store::Absent;
    use htmlspec_parse::{parse_document, project};

    fn check(body: &str) -> crate::Report {
        check_with(body, Strictness::Default)
    }

    /// A `tasks` document, so these tests exercise the stale-dependency rules
    /// alone.
    ///
    /// The kind matters and `specification` would be wrong here. The advisory rule
    /// in this module reports an unverified requirement in a specification, and
    /// every fixture below carries a requirement that no verification names — so a
    /// specification would add that warning to each one and the assertions about
    /// staleness would be measuring two rules at once. A tasks document carrying a
    /// requirement is reported for nothing, which is what isolates the subject.
    fn check_with(body: &str, strictness: Strictness) -> crate::Report {
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"tasks\">\
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
            strictness,
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

    /// A requirement in one change state, with a scenario so it is well formed.
    fn requirement(state: &str) -> String {
        format!(
            "<section class=\"requirement\" id=\"req-target\" data-change=\"{state}\">\
             <h2>R</h2><p>The system MUST work.</p>\
             <article class=\"scenario\" id=\"scenario-target\" data-change=\"{state}\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>"
        )
    }

    const DEPENDENT_TASK: &str =
        "<section class=\"task\" id=\"task-x\" data-state=\"in-progress\" \
         data-refs=\"req-target\"><h2>T</h2></section>";

    #[test]
    fn a_dependent_of_a_modified_unit_is_marked_for_review() {
        let report = check(&format!("{}{DEPENDENT_TASK}", requirement("modified")));
        let found = fired(&report, STALE_REFERENCE);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(found[0].location.unit_id.as_deref(), Some("task-x"));
    }

    /// The central property: a review report never gates work.
    #[test]
    fn a_review_warning_does_not_fail_validation() {
        let report = check(&format!("{}{DEPENDENT_TASK}", requirement("modified")));
        assert!(
            report.is_accepted(),
            "a review report must not fail validation: {}",
            messages(&report)
        );
        assert_eq!(report.warnings().count(), 1, "{}", messages(&report));
        assert_eq!(report.errors().count(), 0);
    }

    /// Reconciliation is the author settling the upstream unit. Once it reads
    /// `unchanged`, the dependent stops being reported — with no artifact
    /// regenerated and nothing stored outside the document.
    #[test]
    fn reconciliation_clears_the_report() {
        let before = check(&format!("{}{DEPENDENT_TASK}", requirement("modified")));
        assert_eq!(fired(&before, STALE_REFERENCE).len(), 1);

        let after = check(&format!("{}{DEPENDENT_TASK}", requirement("unchanged")));
        assert!(
            after.diagnostics.is_empty(),
            "reconciling the upstream unit must clear the report: {}",
            messages(&after)
        );
    }

    #[test]
    fn a_unit_declaring_no_change_state_marks_nothing() {
        // An absent attribute means `unchanged`, which is the common case and must
        // not flood a document with review warnings.
        let body = "<section class=\"requirement\" id=\"req-target\"><h2>R</h2>\
             <p>The system MUST work.</p>\
             <article class=\"scenario\" id=\"scenario-target\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>\
             <section class=\"task\" id=\"task-x\" data-state=\"todo\" \
             data-refs=\"req-target\"><h2>T</h2></section>";
        let report = check(body);
        assert!(report.diagnostics.is_empty(), "{}", messages(&report));
    }

    #[test]
    fn an_added_unit_also_marks_its_dependents() {
        let report = check(&format!("{}{DEPENDENT_TASK}", requirement("added")));
        assert_eq!(
            fired(&report, STALE_REFERENCE).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// A reference to a withdrawn unit is a `refs/*` error. Offering a warning
    /// for it as well would suggest the work can continue, which it cannot.
    #[test]
    fn a_removed_unit_is_left_to_the_reference_rules() {
        let report = check(&format!("{}{DEPENDENT_TASK}", requirement("removed")));
        assert!(report.diagnostics.is_empty(), "{}", messages(&report));
    }

    #[test]
    fn a_recorded_verification_outcome_is_reported_as_stale_verification() {
        let report = check(&format!(
            "{}<section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
             data-refs=\"req-target\"><h2>V</h2></section>",
            requirement("modified")
        ));
        let found = fired(&report, STALE_VERIFICATION);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("passed"), "{}", found[0].message);
        // The weaker rule does not also fire; one situation, one diagnostic.
        assert!(
            fired(&report, STALE_REFERENCE).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// A verification that has not run has nothing to go stale.
    #[test]
    fn a_verification_that_has_not_run_is_only_a_stale_reference() {
        let report = check(&format!(
            "{}<section class=\"verification\" id=\"verify-x\" data-outcome=\"not-run\" \
             data-refs=\"req-target\"><h2>V</h2></section>",
            requirement("modified")
        ));
        assert!(
            fired(&report, STALE_VERIFICATION).is_empty(),
            "{}",
            messages(&report)
        );
        assert_eq!(
            fired(&report, STALE_REFERENCE).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// A partial check ran, so a partial verification of a moved requirement is the strong case.
    ///
    /// Design decision 6 of `record-a-partial-verification`: a recorded result against an
    /// obligation that has since moved is what `review/stale-verification` exists for, and a check
    /// that reached part of the requirement did record a result.
    #[test]
    fn a_partial_verification_of_a_moved_requirement_is_a_stale_verification() {
        let report = check(&format!(
            "{}<section class=\"verification\" id=\"verify-x\" data-outcome=\"partial\" \
             data-refs=\"req-target\"><h2>V</h2>\
             <p data-field=\"unchecked\">The second clause of the Then.</p></section>",
            requirement("modified")
        ));
        let found = fired(&report, STALE_VERIFICATION);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("partial"), "{}", found[0].message);
        assert!(
            fired(&report, STALE_REFERENCE).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// Strict mode does not promote a review warning, which is the mirror of the
    /// `safety/*` floor.
    ///
    /// The specs say a review report "SHALL NOT block continued work, SHALL NOT
    /// impose a phase gate". A strict run that failed on one would impose exactly
    /// that gate: an agent editing a requirement would find continuous integration
    /// red until every dependent was reconciled, which is the waterfall the fluid
    /// dependency model rejects.
    #[test]
    fn strict_mode_does_not_promote_a_review_warning() {
        let report = check_with(
            &format!("{}{DEPENDENT_TASK}", requirement("modified")),
            Strictness::Strict,
        );
        assert!(
            report.is_accepted(),
            "a review warning gated work under strict validation: {}",
            messages(&report)
        );
        assert_eq!(report.warnings().count(), 1, "{}", messages(&report));
        assert_eq!(report.errors().count(), 0, "{}", messages(&report));
    }

    #[test]
    fn every_dependent_of_a_changed_unit_is_reported() {
        let report = check(&format!(
            "{}<section class=\"task\" id=\"task-a\" data-state=\"todo\" \
             data-refs=\"req-target\"><h2>A</h2></section>\
             <section class=\"task\" id=\"task-b\" data-state=\"todo\" \
             data-refs=\"scenario-target\"><h2>B</h2></section>\
             <section class=\"risk\" id=\"risk-c\" data-refs=\"req-target\"><h2>C</h2>\
             <p>Fails.</p><p data-field=\"mitigation\">Held.</p></section>",
            requirement("modified")
        ));
        assert_eq!(
            fired(&report, STALE_REFERENCE).len(),
            3,
            "{}",
            messages(&report)
        );
    }

    /// A state added to the schema counts as material by default, so a new state
    /// cannot silently stop marking dependents.
    #[test]
    fn material_states_are_derived_by_exclusion() {
        let material = material_states();
        assert!(
            material.iter().any(|s| s == "added") && material.iter().any(|s| s == "modified"),
            "{material:?}"
        );
        assert!(
            !material.iter().any(|s| s == "unchanged") && !material.iter().any(|s| s == "removed"),
            "{material:?}"
        );
        // Every state is accounted for: material, or one of the two exclusions.
        assert_eq!(
            material.len() + 2,
            schema_facts::enum_values("definitions/changeState").len(),
            "a change state is neither material nor deliberately excluded"
        );
    }

    // -------------------------------------------------------------------------
    // review/example-content
    // -------------------------------------------------------------------------

    /// Task 3.5: the half-replaced case, which is the likely one rather than the edge one.
    ///
    /// The unit's text is entirely the author's and its identifier is still the template's. A
    /// check on the example's *text* passes this document; a check on the identifier fails it.
    /// That is the whole reason the rule reads the identifier.
    #[test]
    fn a_half_replaced_example_is_reported() {
        let report = check(
            "<section class=\"task\" id=\"task-example\" data-state=\"todo\">\
             <h2>Derive the example identifiers from the compiled templates</h2>\
             <p>Nothing of the template's prose survives in this unit.</p></section>",
        );
        let found = fired(&report, EXAMPLE_CONTENT);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(found[0].location.unit_id.as_deref(), Some("task-example"));
        // And it is a warning, so the document still stands.
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// Task 3.6, second half: the rule reports whatever the templates yield.
    ///
    /// The rule is checked against every identifier the templates currently write, read from the
    /// templates at run time. A hardcoded list here would quietly undo that, so the list is
    /// derived.
    ///
    /// **This half cannot show that coverage moves**, because its input and the code under test
    /// come from the same function: an identifier the derivation stopped finding is one this
    /// loop stops iterating over, and it passes either way. Measured rather than assumed — a
    /// scan altered to drop one identifier and find the rest leaves this test green. The
    /// non-empty assertion below catches only a derivation that returns *nothing*, and a
    /// template gaining a unit is the partial case by nature.
    ///
    /// The first half — `a_template_that_gains_an_example_unit_is_covered_without_a_second_edit`,
    /// in `htmlspec-assets` — drives the derivation from template text the test writes, and that
    /// same partial break fails it. The two together are the chain the task describes.
    #[test]
    fn every_identifier_the_templates_write_is_reported() {
        let identifiers = htmlspec_assets::example_identifiers();
        assert!(
            !identifiers.is_empty(),
            "the templates yielded no example identifiers, so this test checks nothing"
        );
        for id in identifiers {
            // A task unit carries any identifier, so one fixture shape covers every case.
            let report = check(&format!(
                "<section class=\"task\" id=\"{id}\" data-state=\"todo\"><h2>T</h2></section>"
            ));
            assert_eq!(
                fired(&report, EXAMPLE_CONTENT).len(),
                1,
                "{id} is written by a template and was not reported:\n{}",
                messages(&report)
            );
        }
    }

    /// Task 3.7: the rule does not over-report.
    ///
    /// An identifier that merely ends in the suffix is the author's own. A document writing
    /// about a worked example is doing ordinary work, and a rule that fired on it would be one a
    /// reader learns to skip.
    #[test]
    fn an_identifier_the_templates_do_not_write_is_not_reported() {
        for id in [
            "task-worked-example",
            "req-contract-authority",
            "task-derive-the-identifiers",
        ] {
            let report = check(&format!(
                "<section class=\"task\" id=\"{id}\" data-state=\"todo\"><h2>T</h2></section>"
            ));
            assert!(
                fired(&report, EXAMPLE_CONTENT).is_empty(),
                "{id} was reported as a template's example content:\n{}",
                messages(&report)
            );
        }
    }

    /// A figure is checked beside the units, because one of the identifiers is a figure's.
    ///
    /// `Model::units()` does not yield a figure — the model carries figures separately — so
    /// reading only the units would leave `figure-example` uncovered while the derivation
    /// reported it as in scope.
    #[test]
    fn a_figure_carrying_an_example_identifier_is_reported() {
        assert!(
            htmlspec_assets::is_example_identifier("figure-example"),
            "the specification template no longer writes figure-example, so this test's \
             subject is gone"
        );
        let report = check(
            "<figure id=\"figure-example\"><pre class=\"mermaid\">graph TD; A-->B</pre>\
             <figcaption>A caption.</figcaption></figure>",
        );
        let found = fired(&report, EXAMPLE_CONTENT);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(found[0].location.unit_id.as_deref(), Some("figure-example"));
    }

    /// Task 3.8, the validation half: the report is a warning under every strictness.
    ///
    /// This is why the rule is in `review/` rather than `kind/`. A document carries example
    /// content the instant it is created, so a promotable report would fail a freshly created
    /// document under `--strict` — and `document-template` requires that document to be valid.
    #[test]
    fn example_content_is_never_promoted_to_an_error() {
        let body = "<section class=\"task\" id=\"task-example\" data-state=\"todo\">\
                    <h2>T</h2></section>";
        for strictness in [Strictness::Default, Strictness::Strict] {
            let report = check_with(body, strictness);
            let found = fired(&report, EXAMPLE_CONTENT);
            assert_eq!(found.len(), 1, "{}", messages(&report));
            assert!(
                !found[0].is_error(),
                "example content became an error under {strictness:?}, which would fail a \
                 document the program itself just wrote"
            );
            assert!(report.is_accepted(), "{}", messages(&report));
        }
    }

    /// The message names the unit and says what to do, and it says the report does not gate.
    #[test]
    fn the_example_content_message_states_what_to_do() {
        let report = check(
            "<section class=\"task\" id=\"task-example\" data-state=\"todo\"><h2>T</h2></section>",
        );
        let text = messages(&report);
        assert!(text.contains("task-example"), "{text}");
        assert!(
            text.contains("replace it with the document's own"),
            "the message does not say what to do: {text}"
        );
        assert!(
            text.contains("never gates the work"),
            "the message does not say the report is advisory: {text}"
        );
        assert!(
            text.contains("archival refuses"),
            "the message does not name where this does refuse: {text}"
        );
    }

    // -------------------------------------------------------------------------
    // review/example-content: the masthead summary
    // -------------------------------------------------------------------------

    /// A `tasks` document whose masthead carries `summary`, and whose body is the caller's.
    ///
    /// [`check_with`] deliberately writes no `header`, so every fixture above projects an empty
    /// summary and the tests about units are not also tests about the masthead. These tests need
    /// the opposite, so they build the header here rather than widening that helper — a helper
    /// that grew a masthead would silently add a second subject to twenty existing assertions.
    fn check_summary(summary: &str, body: &str) -> crate::Report {
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"tasks\">\
             <title>Fixture</title></head>\n<body>\n\
             <header class=\"hs-masthead\"><h1>Fixture</h1>{summary}</header>\n\
             {body}\n</body>\n</html>\n"
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

    /// One of the templates' placeholder paragraphs, read from the templates.
    ///
    /// Read rather than written, so a test here cannot pass against a derivation that has stopped
    /// finding anything: the fixture and the expectation would go empty together, and an empty
    /// summary is not reported at all — so the assertion that it *is* reported still fails.
    fn a_placeholder() -> String {
        htmlspec_assets::example_placeholders()
            .first()
            .expect("the templates yield at least one placeholder summary")
            .clone()
    }

    /// The case this change exists for: every unit replaced, the masthead left alone.
    ///
    /// No unit identifier remains, so the identifier check reports nothing — and before this rule
    /// the document validated with zero diagnostics. That is not the rule being narrower here; it
    /// is the rule not reaching this at all, because a header paragraph carries no identifier.
    #[test]
    fn a_document_whose_units_were_replaced_and_whose_summary_was_not_is_reported() {
        let report = check_summary(
            &format!("<p>{}</p>", a_placeholder()),
            "<section class=\"task\" id=\"task-read-the-templates\" data-state=\"todo\">\
             <h2>Read the placeholder out of the templates</h2>\
             <p>Every unit here is the author's own.</p></section>",
        );
        let found = fired(&report, EXAMPLE_CONTENT);
        assert_eq!(
            found.len(),
            1,
            "the placeholder summary was not reported, or something else was:\n{}",
            messages(&report)
        );
        // Located at the document, because the summary has no identifier. A reader sent to look
        // for a unit would be looking for one that does not exist.
        assert_eq!(
            found[0].location.unit_id, None,
            "the summary diagnostic names a unit, and the summary has no identifier"
        );
        // A warning, so the document still stands.
        assert!(report.is_accepted(), "{}", messages(&report));

        let text = messages(&report);
        assert!(
            text.contains("summary"),
            "the message does not name the summary: {text}"
        );
        assert!(
            text.contains("masthead paragraph"),
            "the message does not say which paragraph to replace: {text}"
        );
        assert!(
            text.contains("never gates the work") && text.contains("archival refuses"),
            "the message does not state the severity split: {text}"
        );
    }

    /// A summary quoting the placeholder inside the author's own statement is not reported.
    ///
    /// **The case the substring approach would have got wrong**, and the reason the comparison is
    /// whole-summary equality. Without this test nothing stops a later change from replacing the
    /// equality with a `contains`, which would fire on this project's own explainers and on every
    /// document discussing the rule — the shape that teaches an agent to ignore a diagnostic.
    #[test]
    fn a_summary_quoting_the_placeholder_is_not_reported() {
        let placeholder = a_placeholder();
        for summary in [
            format!(
                "<p>Unlike the template's own wording — {placeholder} — this document \
                     states the check it delivers.</p>"
            ),
            format!(
                "<p>{placeholder}</p><p>And this paragraph is the author's own, added \
                     beside it.</p>"
            ),
            format!("<p>This change reports a summary that is exactly {placeholder}</p>"),
        ] {
            let report = check_summary(&summary, "");
            assert!(
                fired(&report, EXAMPLE_CONTENT).is_empty(),
                "a summary containing the placeholder inside a longer statement was reported as \
                 being the placeholder:\n{summary}\n{}",
                messages(&report)
            );
        }

        // And the whole-summary case is still reported, so this test cannot pass by the rule
        // having stopped firing altogether.
        let report = check_summary(&format!("<p>{placeholder}</p>"), "");
        assert_eq!(
            fired(&report, EXAMPLE_CONTENT).len(),
            1,
            "the rule reports nothing at all, so the assertions above hold vacuously:\n{}",
            messages(&report)
        );
    }

    /// A document with no summary is not reported by this rule.
    ///
    /// An absent summary and a placeholder summary are different facts. Reported as the other,
    /// this would send an author to replace a paragraph that is not there. Projection 9.1.1 says
    /// a document with no `header`, or a `header` with no paragraph, projects an empty summary and
    /// stays valid, so both shapes are checked.
    #[test]
    fn a_document_with_no_summary_is_not_reported() {
        // A header with no paragraph.
        let report = check_summary("", "");
        assert!(
            fired(&report, EXAMPLE_CONTENT).is_empty(),
            "a header with no paragraph was reported as carrying a placeholder:\n{}",
            messages(&report)
        );

        // No header at all, which is what every other fixture in this module projects.
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\">\
                            <h2>T</h2></section>",
        );
        assert!(
            fired(&report, EXAMPLE_CONTENT).is_empty(),
            "a document with no header was reported as carrying a placeholder:\n{}",
            messages(&report)
        );
    }

    /// Every artifact kind is covered, driven from the templates rather than from four names.
    ///
    /// A list of kind names here would go stale the first time a kind is added — the same defect
    /// one layer up, and the reason the placeholder set is derived at all. So this renders each
    /// template through the code `htmlspec new document` uses and asserts the document it produces
    /// is reported at its own document location.
    ///
    /// The rendered document also carries its template's example *units*, so the count of
    /// `review/example-content` diagnostics varies by kind. What is asserted is that exactly one
    /// of them is located at the document, which is the summary's.
    #[test]
    fn every_artifact_kind_carrying_its_placeholder_summary_is_reported() {
        assert!(
            !htmlspec_assets::TEMPLATES.is_empty(),
            "there are no templates, so this test checks nothing"
        );
        for template in htmlspec_assets::TEMPLATES {
            let rendered =
                htmlspec_assets::render_kind(template.kind, "A fresh document", "assets")
                    .unwrap_or_else(|| panic!("no template produces the {} kind", template.kind));
            let outcome = parse_document(&rendered);
            assert!(
                outcome.is_recovery_free(),
                "the {} template recovered: {:?}",
                template.kind,
                outcome.recovered
            );
            let projection = project(&outcome.document);
            let report = run(
                &outcome,
                &projection,
                &NO_STORE,
                &crate::references::Scope::loose(),
                &[validate],
                Strictness::Default,
            );
            let at_document: Vec<_> = fired(&report, EXAMPLE_CONTENT)
                .into_iter()
                .filter(|d| d.location.unit_id.is_none())
                .collect();
            assert_eq!(
                at_document.len(),
                1,
                "a freshly created {} was not reported for its placeholder summary, so the rule \
                 is inert for that kind:\n{}",
                template.kind,
                messages(&report)
            );
            // Still valid, which is `document-template`'s requirement about creation.
            assert!(
                report.is_accepted(),
                "a freshly created {} is invalid:\n{}",
                template.kind,
                messages(&report)
            );
        }
    }

    /// The summary report is a warning under every strictness, as the unit report is.
    ///
    /// Same reason, stated in [`EXAMPLE_CONTENT`]: `htmlspec new document` writes a placeholder
    /// summary every time, so an error would make every freshly created document invalid and
    /// contradict `document-template`. The conformance suite pins the `review/*` ceiling from the
    /// outside; this pins it for this diagnostic specifically.
    #[test]
    fn a_placeholder_summary_is_never_promoted_to_an_error() {
        let summary = format!("<p>{}</p>", a_placeholder());
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"tasks\">\
             <title>Fixture</title></head>\n<body>\n\
             <header class=\"hs-masthead\"><h1>Fixture</h1>{summary}</header>\n</body>\n</html>\n"
        );
        let outcome = parse_document(&html);
        assert!(outcome.is_recovery_free(), "{:?}", outcome.recovered);
        let projection = project(&outcome.document);
        for strictness in [Strictness::Default, Strictness::Strict] {
            let report = run(
                &outcome,
                &projection,
                &NO_STORE,
                &crate::references::Scope::loose(),
                &[validate],
                strictness,
            );
            let found = fired(&report, EXAMPLE_CONTENT);
            assert_eq!(found.len(), 1, "{}", messages(&report));
            assert!(
                !found[0].is_error(),
                "a placeholder summary became an error under {strictness:?}, which would fail \
                 every document the program itself just wrote"
            );
            assert!(report.is_accepted(), "{}", messages(&report));
        }
    }

    // -------------------------------------------------------------------------
    // review/unverified-requirement
    // -------------------------------------------------------------------------

    /// A `specification` document holding `body`, where the unverified-requirement rule runs.
    ///
    /// The rest of this module checks a `tasks` document, so that its staleness assertions do not
    /// also measure this rule. These tests need the rule, so they declare the kind it runs on.
    fn check_specification(body: &str) -> crate::Report {
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

    /// document-validation, "A requirement whose check failed" and "A requirement verified only
    /// in part": the report asks whether a verification names the requirement, not what it found.
    ///
    /// Every outcome the schema enumerates is checked, so a value added to the schema is held to
    /// this without an edit here. The requirement is `unchanged`, so no staleness warning mixes in.
    #[test]
    fn a_verification_of_any_outcome_clears_the_unverified_warning() {
        let requirement = requirement("unchanged");

        // The control: with nothing naming the requirement, the rule reports it, so the
        // assertions below cannot hold by the rule having stopped firing.
        let unverified = check_specification(&requirement);
        assert_eq!(
            fired(&unverified, UNVERIFIED_REQUIREMENT).len(),
            1,
            "{}",
            messages(&unverified)
        );

        let outcomes = schema_facts::enum_values("definitions/verification/properties/outcome");
        assert!(
            !outcomes.is_empty(),
            "the schema enumerates no outcome, so this test checks nothing"
        );
        for outcome in outcomes {
            let report = check_specification(&format!(
                "{requirement}<section class=\"verification\" id=\"verify-x\" \
                 data-outcome=\"{outcome}\" data-refs=\"req-target\"><h2>V</h2>\
                 <p data-field=\"method\">Checked.</p></section>"
            ));
            assert!(
                fired(&report, UNVERIFIED_REQUIREMENT).is_empty(),
                "a verification recording {outcome} names the requirement and it was still \
                 reported as unverified:\n{}",
                messages(&report)
            );
        }
    }

    #[test]
    fn the_not_run_outcome_is_found_in_the_schema() {
        assert!(
            !has_run("not-run"),
            "the not-run outcome was not recognized"
        );
        for outcome in schema_facts::enum_values("definitions/verification/properties/outcome") {
            if outcome == "not-run" {
                continue;
            }
            assert!(has_run(&outcome), "{outcome} should count as having run");
        }
    }

    /// The dogfood documents mark units `added` and reference them, so they are
    /// expected to produce review warnings — and still to be accepted.
    #[test]
    fn the_dogfood_documents_are_accepted_despite_review_warnings() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dogfood");
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .expect("the dogfood directory is readable")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "html"))
            .collect();
        paths.sort();
        assert_eq!(paths.len(), 6);

        let mut total_warnings = 0;
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
                "{path:?} was failed by a review rule, which must never gate work:\n{}",
                messages(&report)
            );
            total_warnings += report.warnings().count();
        }
        assert!(
            total_warnings > 0,
            "the dogfood documents mark units added and reference them, so the review \
             rules should have found something; finding nothing means they stopped firing"
        );
    }
}
