//! `kind/*`: the two rules a document's declared artifact kind imposes.
//!
//! A change is a directory of four documents and each declares which artifact it
//! is. Two of the four kinds constrain their content and two do not. This module
//! holds the constraints, and the shape of the module is decided by how few of
//! them there are.
//!
//! **Placement**: a `specification` must not carry a task unit. A specification
//! records the obligation a task satisfied rather than the work that satisfied it,
//! and a task appearing there would offer work as though it were an obligation to
//! the tools and later changes that read a specification as the contract.
//!
//! **Completeness**: a `tasks` document must carry at least one task unit. The
//! stage exists to enumerate work, and one that enumerates none records nothing.
//!
//! That is the whole of it, and the absences matter as much as the rules. Nothing
//! here reports on which units a `proposal` or a `design` carries or omits. Those
//! stages need freedom in how they are written — a proposal arguing for a
//! prospective requirement is doing a proposal's job, whether it states it in
//! prose or marks it as a unit — and the freedom is a claim about what validation
//! does *not* do, which is why fixtures carrying would-be violations pin it rather
//! than a test asserting silence.
//!
//! The third rule the proposal states lives in [`crate::rules::review`] rather
//! than here: a `specification` requirement that no verification names is a
//! warning, and it belongs in the one namespace strict mode refuses to promote.
//! In `kind/*` it would be promotable, so `--strict` would fail a specification
//! for being in progress — the phase gate the dependency model exists to avoid.
//!
//! These rules read [`htmlspec_parse::Projection::model`] rather than the tree.
//! Every input they need is projected already: the document's kind, its task
//! list, its identifiers. A rule that walked the tree for units the model holds
//! would be a second projection that could disagree with the first. `safety/*`
//! reads the tree because a remote script never reaches the model at all, and
//! that reasoning does not apply to a unit the model carries by name.
//!
//! The kind vocabulary itself is the schema's. Rule code names a kind by a stem
//! and resolves it through [`crate::schema_facts`], because a Rust `const` listing
//! the four values would restate an enumeration the schema owns — which
//! `rule_code_does_not_restate_schema_expressible_values` refuses.

use htmlspec_parse::Projection;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};
use crate::schema_facts;

/// A specification carries a task unit.
pub const TASK_IN_SPECIFICATION: RuleId = RuleId::new(Namespace::Kind, "task-in-specification");

/// A tasks document carries no task unit.
pub const NO_TASKS: RuleId = RuleId::new(Namespace::Kind, "no-tasks");

/// Runs every kind rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    let Some(kind) = projection.model.kind.as_deref() else {
        // A document declaring no kind has no kind's rules to apply. The schema
        // reports the missing property, and inventing a kind here would mean
        // reporting a placement fault against a kind the author never declared.
        return;
    };

    tasks_stay_out_of_a_specification(kind, projection, diagnostics);
    a_tasks_document_enumerates_work(kind, projection, diagnostics);
}

/// Placement: a `specification` must not carry a task unit.
///
/// Reported once per task rather than once per document, and naming the task,
/// because the fix is to move or remove one unit and a reader given only "this
/// specification has tasks" has to find them.
fn tasks_stay_out_of_a_specification(
    kind: &str,
    projection: &Projection,
    diagnostics: &mut Diagnostics,
) {
    if !is_kind(kind, "specific") {
        return;
    }
    for task in &projection.model.tasks {
        diagnostics.violation(
            TASK_IN_SPECIFICATION,
            Location::maybe_unit(task.id.as_deref()),
            "a specification must not carry a task unit. A specification records the \
             obligation a task satisfied rather than the work that satisfied it, so a \
             completed task is history: state the obligation as a requirement here, and \
             leave the task in the change's tasks document",
        );
    }
}

/// Completeness: a `tasks` document must carry at least one task unit.
///
/// A completeness rule and not a placement one. It constrains what the document
/// must contain rather than which units are permitted in it, so a tasks document
/// carrying a decision or a risk alongside its tasks is reported for nothing.
fn a_tasks_document_enumerates_work(
    kind: &str,
    projection: &Projection,
    diagnostics: &mut Diagnostics,
) {
    if !is_kind(kind, "task") || !projection.model.tasks.is_empty() {
        return;
    }
    diagnostics.violation(
        NO_TASKS,
        Location::document(),
        "a tasks document carries no task unit. The stage exists to enumerate the work \
         to be done, and one that enumerates none records nothing a later stage can act \
         on",
    );
}

/// Whether `kind` is the artifact kind the schema names with `stem`.
///
/// The comparison resolves the stem against the schema's own vocabulary rather
/// than matching the stem against the document's declared value. A document
/// declaring `specifications` or `Specification` is *not* a specification: its
/// kind is outside the enumeration, the schema reports that, and a placement rule
/// firing on it as well would report a second fault for one defect.
fn is_kind(kind: &str, stem: &str) -> bool {
    schema_facts::artifact_kind(stem).is_some_and(|named| named == kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};
    use htmlspec_parse::{parse_document, project};

    /// These rules are not asset rules, so they run with no store beside them.
    const NO_STORE: htmlspec_assets::Store = htmlspec_assets::Store::Absent;

    /// A document of `kind` carrying `body`, validated by the kind rules alone.
    ///
    /// The kind rules alone, so a fixture body that also trips a schema rule does
    /// not make the assertion about the wrong thing.
    fn check(kind: &str, body: &str) -> crate::Report {
        check_with(kind, body, Strictness::Default)
    }

    fn check_with(kind: &str, body: &str, strictness: Strictness) -> crate::Report {
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"{kind}\">\
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

    const TASK: &str = "<section class=\"task\" id=\"task-x\" data-state=\"todo\">\
         <h2>A task</h2></section>";

    const REQUIREMENT: &str = "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
         <p>The system MUST work.</p>\
         <article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>\
         <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>";

    const DECISION: &str = "<section class=\"decision\" id=\"decision-x\" \
         data-status=\"accepted\"><h2>D</h2><p>The decision.</p>\
         <p data-field=\"rationale\">Why.</p></section>";

    // --- Placement -------------------------------------------------------

    #[test]
    fn a_task_in_a_specification_is_reported_and_named() {
        let report = check("specification", TASK);
        let found = fired(&report, TASK_IN_SPECIFICATION);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(found[0].location.unit_id.as_deref(), Some("task-x"));
        assert!(found[0].is_error(), "placement is an error");
        // The message says why, not only what: the reason is what tells an author
        // where the task belongs instead.
        assert!(
            found[0].message.contains("obligation"),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn every_task_in_a_specification_is_reported() {
        let report = check(
            "specification",
            "<section class=\"task\" id=\"task-a\" data-state=\"todo\"><h2>A</h2></section>\
             <section class=\"task\" id=\"task-b\" data-state=\"done\"><h2>B</h2></section>",
        );
        assert_eq!(
            fired(&report, TASK_IN_SPECIFICATION).len(),
            2,
            "{}",
            messages(&report)
        );
    }

    /// A specification with no task reports nothing, which is the ordinary case.
    #[test]
    fn a_specification_without_tasks_reports_nothing() {
        let report = check("specification", REQUIREMENT);
        assert!(report.diagnostics.is_empty(), "{}", messages(&report));
    }

    // --- Completeness ----------------------------------------------------

    #[test]
    fn a_tasks_document_with_no_task_is_reported() {
        let report = check("tasks", "<p>Prose about work nobody enumerated.</p>");
        let found = fired(&report, NO_TASKS);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].is_error(), "completeness is an error");
        // Document-wide: there is no unit to name, because the fault is an absence.
        assert!(found[0].location.is_document_wide(), "{:?}", found[0]);
    }

    #[test]
    fn a_tasks_document_with_a_task_reports_nothing() {
        let report = check("tasks", TASK);
        assert!(report.diagnostics.is_empty(), "{}", messages(&report));
    }

    /// Task 3.3: completeness constrains what must be present, never what is
    /// permitted. A tasks document carrying other units alongside its tasks is
    /// reported for nothing.
    ///
    /// Asserted separately from the rule's own test, because the two properties
    /// fail independently: a rule rewritten as a placement matrix would still pass
    /// `a_tasks_document_with_no_task_is_reported`.
    #[test]
    fn a_tasks_document_carrying_other_units_is_reported_for_none_of_them() {
        let report = check("tasks", &format!("{TASK}{DECISION}{REQUIREMENT}"));
        assert!(
            report.diagnostics.is_empty(),
            "a tasks document was reported for a unit alongside its tasks, which \
             would make the completeness rule a placement rule:\n{}",
            messages(&report)
        );
    }

    // --- The upstream kinds are unconstrained ----------------------------

    /// Task 3.7, from inside the module: no rule here reports on unit placement in
    /// a `proposal` or a `design`.
    ///
    /// Every unit the vocabulary has, in each of the two unconstrained kinds. The
    /// conformance fixtures pin this from the outside as well, and both are worth
    /// having: a fixture proves it through the whole engine, and this proves that
    /// *this module* is where the silence comes from rather than some later
    /// filter.
    #[test]
    fn no_unit_in_a_proposal_or_a_design_is_reported() {
        let every_unit = format!(
            "{REQUIREMENT}{DECISION}{TASK}\
             <section class=\"evidence\" id=\"evidence-x\"><h2>E</h2><p>Measured.</p></section>\
             <section class=\"risk\" id=\"risk-x\"><h2>K</h2><p>Fails.</p>\
             <p data-field=\"mitigation\">Held.</p></section>\
             <section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
             data-refs=\"req-x\"><h2>V</h2></section>"
        );
        for kind in ["proposal", "design"] {
            let report = check(kind, &every_unit);
            assert!(
                report.diagnostics.is_empty(),
                "a {kind} carrying every unit was reported, and the upstream kinds \
                 impose no structure:\n{}",
                messages(&report)
            );
        }
    }

    /// Prose alone conforms in every kind except the one that requires a task.
    #[test]
    fn prose_alone_conforms_in_the_unconstrained_kinds() {
        for kind in ["proposal", "design", "specification"] {
            let report = check(kind, "<h2>An argument</h2><p>Stated in prose.</p>");
            assert!(
                report.diagnostics.is_empty(),
                "a {kind} of pure prose was reported:\n{}",
                messages(&report)
            );
        }
    }

    // --- The rules' own boundaries ---------------------------------------

    /// A document declaring no kind gets no kind rules.
    ///
    /// The schema reports the missing property. Applying a kind's rules here would
    /// mean choosing a kind the author never declared, and the most convenient
    /// choice is the strictest one — so an unmigrated proposal would be told its
    /// tasks are misplaced rather than that its kind is missing.
    #[test]
    fn a_document_with_no_declared_kind_is_left_to_the_schema() {
        let html = "<!doctype html>\n<html lang=\"en\">\n<head>\
                    <meta name=\"htmlspec-version\" content=\"0.1.0\">\
                    <title>T</title></head>\n<body>\
                    <section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
                    </section></body></html>\n";
        let outcome = parse_document(html);
        let projection = project(&outcome.document);
        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[validate],
            Strictness::Default,
        );
        assert!(report.diagnostics.is_empty(), "{}", messages(&report));
    }

    /// A kind outside the enumeration gets no kind rules either, for the same
    /// reason: the schema reports the value, and one defect earns one diagnostic.
    #[test]
    fn a_kind_outside_the_enumeration_is_left_to_the_schema() {
        let report = check("specifications", TASK);
        assert!(
            report.diagnostics.is_empty(),
            "a near-miss kind was treated as the kind it resembles:\n{}",
            messages(&report)
        );
    }

    /// Both rules are errors, and both are downgradable.
    ///
    /// `kind/*` is not `safety/*`: a project may reduce these to warnings. What it
    /// may not do is have them default to warnings, because a specification read as
    /// the contract must not offer work as an obligation.
    #[test]
    fn the_kind_rules_are_downgradable_errors() {
        assert!(Namespace::Kind.is_downgradable());
        assert!(Namespace::Kind.is_promotable());
        for rule in [TASK_IN_SPECIFICATION, NO_TASKS] {
            assert_eq!(
                rule.namespace().default_severity(),
                crate::diagnostic::Severity::Error,
                "{rule} must be an error"
            );
        }
    }

    /// The vocabulary comes from the schema, so a kind renamed there reaches these
    /// rules without an edit here.
    #[test]
    fn the_kinds_the_rules_name_are_the_schemas_own() {
        let kinds = schema_facts::artifact_kinds();
        for stem in ["specific", "task"] {
            let named = schema_facts::artifact_kind(stem)
                .unwrap_or_else(|| panic!("no kind resolves from the stem {stem:?}"));
            assert!(kinds.contains(&named), "{named} is not one of {kinds:?}");
        }
    }
}
