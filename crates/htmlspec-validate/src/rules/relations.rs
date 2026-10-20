//! The relational rules Rust owns.
//!
//! JSON Schema expresses the shape of a unit well and relationships between
//! units not at all. It cannot say that identifiers are unique document-wide,
//! that an identifier's prefix matches its unit's type, that a `dropped` task
//! states a reason, that only a `done` task states a deviation, that a `partial`
//! verification states what it did not check, or that a `mitigated` risk states
//! its mitigation — the last four because the contract keeps to a conservative
//! subset with no conditional composition, which is what makes type generation
//! work.
//!
//! So these rules live here, and the line is drawn where the design draws it: the
//! schema owns the shape of a unit, Rust owns relationships between units. Every
//! contract value a rule below needs is read from the schema through
//! [`crate::schema_facts`], never restated, and
//! `tests/schema_rust_split.rs` enforces that.
//!
//! Two rules the specs list under structural conformance are absent here on
//! purpose. "A requirement has at least one scenario" is `minItems` in the schema,
//! and "a scenario does not appear outside a requirement" is projection rule 4.3.
//! Restating either one in Rust would duplicate a constraint the schema or the
//! projection already owns, which is the drift this split exists to prevent;
//! `the_schema_owns_what_it_can_express` records that.

use std::collections::BTreeMap;

use htmlspec_parse::Projection;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};
use crate::schema_facts;

/// Two units, or a unit and a figure, share one identifier.
pub const DUPLICATE_ID: RuleId = RuleId::new(Namespace::Schema, "duplicate-id");

/// A unit's identifier does not carry the prefix its type reserves.
pub const PREFIX_MISMATCH: RuleId = RuleId::new(Namespace::Schema, "prefix-mismatch");

/// A `superseded` decision does not name the decision that replaces it.
pub const MISSING_SUCCESSOR: RuleId = RuleId::new(Namespace::Schema, "missing-successor");

/// A `dropped` task does not state why.
pub const MISSING_REASON: RuleId = RuleId::new(Namespace::Schema, "missing-reason");

/// A task that is not `done` states a deviation.
pub const DEVIATION_NOT_DONE: RuleId = RuleId::new(Namespace::Schema, "deviation-not-done");
/// A `partial` verification does not state what its check did not reach.
pub const MISSING_UNCHECKED: RuleId = RuleId::new(Namespace::Schema, "missing-unchecked");
/// A `mitigated` risk, declared or by default, does not state what holds it down.
pub const MISSING_MITIGATION: RuleId = RuleId::new(Namespace::Schema, "missing-mitigation");

/// A normative statement does not use the modal vocabulary the contract requires.
pub const WEAK_MODAL: RuleId = RuleId::new(Namespace::Schema, "weak-modal");

/// Runs every relational rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    identifiers_are_unique(projection, diagnostics);
    prefixes_match_unit_types(projection, diagnostics);
    superseded_decisions_name_a_successor(projection, diagnostics);
    dropped_tasks_state_a_reason(projection, diagnostics);
    deviations_are_on_done_tasks(projection, diagnostics);
    partial_verifications_state_what_they_did_not_check(projection, diagnostics);
    mitigated_risks_state_a_mitigation(projection, diagnostics);
    statements_are_normative(projection, diagnostics);
}

/// Every identifier is unique document-wide.
///
/// A figure identifier shares the namespace even though a figure is not a unit
/// (projection specification, 10.1.1), so a figure colliding with a task is a
/// duplicate. Both locations are reported, because a reviewer needs to know which
/// two things collided and not merely that something did.
fn identifiers_are_unique(projection: &Projection, diagnostics: &mut Diagnostics) {
    // Ordered, so the report is stable across runs regardless of hash iteration.
    let mut seen: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for unit in projection.model.units() {
        if let Some(id) = unit.id {
            seen.entry(id).or_default().push(unit.kind.to_string());
        }
    }
    for figure in &projection.model.figures {
        if let Some(id) = figure.id.as_deref() {
            seen.entry(id).or_default().push("figure".to_owned());
        }
    }

    for (id, holders) in seen {
        if holders.len() < 2 {
            continue;
        }
        diagnostics.violation(
            DUPLICATE_ID,
            Location::unit(id),
            format!(
                "the identifier {id} is declared {} times, by: {}. An identifier addresses \
                 exactly one thing, so a reference to this one resolves to nothing definite",
                holders.len(),
                holders.join(", ")
            ),
        );
    }
}

/// A unit's identifier carries the prefix its type reserves.
///
/// The prefix table lives in the schema and is read from it. `verify-` does not
/// match its unit name, which is exactly why a rule may not derive a prefix from
/// a unit name.
fn prefixes_match_unit_types(projection: &Projection, diagnostics: &mut Diagnostics) {
    for unit in projection.model.units() {
        let Some(id) = unit.id else {
            continue;
        };
        let expected = schema_facts::prefix_for_unit(unit.kind.as_class());
        let Some(expected) = expected else {
            // The schema defines no prefix for this unit, which is a contract
            // defect rather than a document defect. Reported rather than
            // silently skipped, because skipping would accept any identifier.
            diagnostics.violation(
                PREFIX_MISMATCH,
                Location::unit(id),
                format!(
                    "the schema reserves no identifier prefix for a {} unit, so {id} cannot \
                     be checked",
                    unit.kind
                ),
            );
            continue;
        };
        if id.starts_with(&expected) {
            continue;
        }
        diagnostics.violation(
            PREFIX_MISMATCH,
            Location::unit(id),
            format!(
                "a {} unit reserves the {expected} prefix, and {id} does not carry it. A \
                 prefix names the unit type, so a reader cannot tell what {id} addresses",
                unit.kind
            ),
        );
    }
}

/// A `superseded` decision names the decision that replaces it.
///
/// The status values are read from the schema, so a renamed status moves one file.
fn superseded_decisions_name_a_successor(projection: &Projection, diagnostics: &mut Diagnostics) {
    let Some(superseded) = SUPERSEDED.value() else {
        return;
    };
    for decision in &projection.model.decisions {
        if decision.status.as_deref() != Some(superseded.as_str()) {
            continue;
        }
        if decision.superseded_by.is_some() {
            continue;
        }
        let location = Location::maybe_unit(decision.id.as_deref());
        diagnostics.violation(
            MISSING_SUCCESSOR,
            location,
            format!(
                "a decision with status {superseded} names no replacement. A superseded \
                 decision without a successor leaves a reader with no way to find the \
                 decision that now holds"
            ),
        );
    }
}

/// A `dropped` task states why.
fn dropped_tasks_state_a_reason(projection: &Projection, diagnostics: &mut Diagnostics) {
    let Some(dropped) = DROPPED.value() else {
        return;
    };
    for task in &projection.model.tasks {
        if task.state.as_deref() != Some(dropped.as_str()) {
            continue;
        }
        if task.reason.is_some() {
            continue;
        }
        let location = Location::maybe_unit(task.id.as_deref());
        diagnostics.violation(
            MISSING_REASON,
            location,
            format!(
                "a task with state {dropped} states no reason. Work abandoned without a \
                 recorded reason invites someone to propose it again"
            ),
        );
    }
}

/// A deviation is stated only on a `done` task.
///
/// A deviation says how finished work departed from a task's words. A task still to do or under
/// way has finished nothing that could differ from them, and a dropped task's work did not happen,
/// which its reason already says. So the rule keys on the one state that permits the field rather
/// than on the states that forbid it, and a state the schema gains later is refused until someone
/// decides it may carry one.
///
/// A task that declares no state is reported too. The schema reports the missing state, and this
/// reports a second fact about the same task: a deviation stated where nobody declared the work
/// done. The count of tasks done differently reads such a task the same way, as not done.
///
/// Both states are read from the schema by a stem, as [`dropped_tasks_state_a_reason`] reads its
/// own. The stem cannot be the whole value: `tests/schema_rust_split.rs` forbids the literal.
fn deviations_are_on_done_tasks(projection: &Projection, diagnostics: &mut Diagnostics) {
    let Some(done) = DONE.value() else {
        return;
    };
    let dropped = DROPPED.value();
    for task in &projection.model.tasks {
        if task.deviation.is_none() || task.state.as_deref() == Some(done.as_str()) {
            continue;
        }
        let location = Location::maybe_unit(task.id.as_deref());
        let message = match task.state.as_deref() {
            Some(state) if Some(state) == dropped.as_deref() => format!(
                "a task with state {state} states a deviation, and only a {done} task may. A \
                 {state} task's work did not happen, and its reason says why; a task whose work \
                 partly happened is {done}, with a deviation saying which part"
            ),
            Some(state) => format!(
                "a task with state {state} states a deviation, and only a {done} task may. \
                 Nothing is finished yet that could differ from the task's words, so record the \
                 difference in a note and state it as the deviation once the task is {done}"
            ),
            None => format!(
                "a task that declares no state states a deviation, and only a {done} task may. \
                 Declare the task's state"
            ),
        };
        diagnostics.violation(DEVIATION_NOT_DONE, location, message);
    }
}

/// A `partial` verification states what its check did not reach.
///
/// The field is read, not the paragraphs: an unmarked paragraph in a verification is a note
/// (projection specification, section 4.0.2), and a consumer should find what a check missed
/// without parsing notes. The outcome is found by a stem, as the dropped task's state is, so
/// `tests/schema_rust_split.rs` holds.
fn partial_verifications_state_what_they_did_not_check(
    projection: &Projection,
    diagnostics: &mut Diagnostics,
) {
    let Some(partial) = PARTIAL.value() else {
        return;
    };
    for verification in &projection.model.verifications {
        if verification.outcome.as_deref() != Some(partial.as_str()) {
            continue;
        }
        if !verification.unchecked.is_empty() {
            continue;
        }
        let location = Location::maybe_unit(verification.id.as_deref());
        diagnostics.violation(
            MISSING_UNCHECKED,
            location,
            format!(
                "a verification with outcome {partial} states nothing it did not check. Add a \
                 paragraph marked data-field=\"unchecked\" for each part of what it names that \
                 the check did not reach, or the record claims the part it reached and leaves \
                 the rest unnamed"
            ),
        );
    }
}

/// A `mitigated` risk states what holds it down.
///
/// A risk that declares no response projects as `mitigated` (projection specification,
/// section 4.7.1), so a risk written before the response existed is held to a mitigation,
/// as every risk was. An `accepted` risk needs none. A response outside the enumeration is
/// the schema's `schema/invalid`, which names the permitted values, and this rule leaves it
/// alone, so one defect gets one diagnostic.
///
/// Both values are read from the schema: `mitigated` decides whether the rule applies, and
/// `accepted` is the remedy the message names for a risk the change bears.
fn mitigated_risks_state_a_mitigation(projection: &Projection, diagnostics: &mut Diagnostics) {
    let (Some(mitigated), Some(accepted)) = (MITIGATED.value(), ACCEPTED.value()) else {
        return;
    };
    for risk in &projection.model.risks {
        if risk.response != mitigated {
            continue;
        }
        if risk.mitigation.is_some() {
            continue;
        }
        let location = Location::maybe_unit(risk.id.as_deref());
        diagnostics.violation(
            MISSING_MITIGATION,
            location,
            format!(
                "a risk whose response is {mitigated} states no mitigation. A risk that \
                 declares no data-response is {mitigated}, so a risk written before the \
                 attribute existed is held to one. State what holds it down in a \
                 data-field=\"mitigation\" paragraph, or, if this change bears the risk as it \
                 stands, declare data-response=\"{accepted}\" and say in its statement why it \
                 is borne"
            ),
        );
    }
}

/// An enumerated value a conditional rule keys on: a stem of its name, and the schema field that
/// enumerates it.
///
/// Named once, so the rule that reads a value and `every_conditional_rule_finds_its_state` read the
/// same stem. A test that wrote its own stems passed while a rule's own stem had stopped matching,
/// which the code review of `record-how-a-risk-was-settled` found for every rule here.
#[derive(Clone, Copy, Debug)]
struct Enumerated {
    stem: &'static str,
    path: &'static str,
}

impl Enumerated {
    /// The value, read from the schema, or `None` where the stem matches no value or several.
    fn value(self) -> Option<String> {
        status_named(self.stem, self.path)
    }
}

/// A decision's `superseded` status.
const SUPERSEDED: Enumerated = Enumerated {
    stem: "supersed",
    path: "definitions/decision/properties/status",
};
/// A task's `dropped` state.
const DROPPED: Enumerated = Enumerated {
    stem: "dropp",
    path: "definitions/task/properties/state",
};
/// A task's `done` state. The stem cannot be the whole value: `tests/schema_rust_split.rs` forbids
/// the literal in rule code.
const DONE: Enumerated = Enumerated {
    stem: "don",
    path: "definitions/task/properties/state",
};
/// A verification's `partial` outcome.
const PARTIAL: Enumerated = Enumerated {
    stem: "parti",
    path: "definitions/verification/properties/outcome",
};
/// A risk's `mitigated` response.
const MITIGATED: Enumerated = Enumerated {
    stem: "mitigat",
    path: "definitions/risk/properties/response",
};
/// A risk's `accepted` response, which the mitigated-risk rule names as a remedy.
const ACCEPTED: Enumerated = Enumerated {
    stem: "accept",
    path: "definitions/risk/properties/response",
};

/// One enumerated value of a schema field, found by a stem of its name.
///
/// The value itself is read from the schema rather than written here, which is
/// what keeps `tests/schema_rust_split.rs` satisfied. The stem is a fragment of
/// the concept's English name and not the contract value: `supersed` matches
/// `superseded` and would keep matching `supersedes`.
///
/// A stem matching no value, or more than one, yields `None` — the rule that
/// needed it then does not run, and `every_conditional_rule_finds_its_state`
/// fails, so a schema rename surfaces as a failing test rather than as a rule
/// that quietly stopped firing.
fn status_named(stem: &str, path: &str) -> Option<String> {
    let mut matches = schema_facts::enum_values(path)
        .into_iter()
        .filter(|value| value.contains(stem));
    let found = matches.next()?;
    matches.next().is_none().then_some(found)
}

/// A normative statement uses `SHALL` or `MUST`.
///
/// The document-schema capability requires that a requirement's statement express
/// an obligation and not rely on `should` or `may`. The schema cannot check the
/// vocabulary of a string, so the rule lives here.
fn statements_are_normative(projection: &Projection, diagnostics: &mut Diagnostics) {
    for requirement in &projection.model.requirements {
        let normative = requirement
            .statements
            .iter()
            .any(|statement| is_normative(statement));
        if normative || requirement.statements.is_empty() {
            // An empty statement list is the schema's `minItems` fault, not this
            // rule's. Reporting both would name one defect twice.
            continue;
        }
        let location = Location::maybe_unit(requirement.id.as_deref());
        diagnostics.violation(
            WEAK_MODAL,
            location,
            "no statement of this requirement uses SHALL or MUST, so it states no \
             obligation. `should` and `may` describe a preference, which a reader cannot \
             be held to",
        );
    }
}

/// Whether one statement carries a normative modal verb.
///
/// The match is on a whole word and case sensitive: the contract writes the modal
/// in capitals precisely so that prose using the word "must" in passing does not
/// read as an obligation.
fn is_normative(statement: &str) -> bool {
    statement
        .split(|c: char| !c.is_ascii_alphabetic())
        .any(|word| word == "SHALL" || word == "MUST")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};

    /// These rules are not asset rules, so they run with no store beside them.
    const NO_STORE: htmlspec_assets::Store = htmlspec_assets::Store::Absent;
    use htmlspec_parse::model::UnitKind;
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

    const SCENARIO: &str = "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>\
         <dl><dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd></dl></article>";

    #[test]
    fn a_conforming_document_reports_nothing() {
        let report = check(&format!(
            "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
             <p>The system MUST work.</p>{SCENARIO}</section>"
        ));
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    #[test]
    fn a_duplicate_identifier_names_both_holders() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>A</h2></section>\
             <section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>B</h2></section>",
        );
        let found = fired(&report, DUPLICATE_ID);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("2 times"), "{}", found[0].message);
    }

    /// A figure shares the identifier namespace although it is not a unit
    /// (projection specification 10.1.1), so this collision is real.
    #[test]
    fn a_figure_colliding_with_a_unit_is_a_duplicate() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>A</h2></section>\
             <figure id=\"task-x\"><figcaption>A figure.</figcaption><p>Prose.</p></figure>",
        );
        let found = fired(&report, DUPLICATE_ID);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("figure"), "{}", found[0].message);
    }

    #[test]
    fn a_prefix_that_does_not_match_its_unit_type_is_reported() {
        let report = check(
            "<section class=\"task\" id=\"risk-wrong-prefix\" data-state=\"todo\">\
             <h2>T</h2></section>",
        );
        let found = fired(&report, PREFIX_MISMATCH);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(found[0].message.contains("task-"), "{}", found[0].message);
    }

    /// `verify-` does not match its unit name, so a rule deriving the prefix from
    /// the name would wrongly reject every conforming verification.
    #[test]
    fn a_verification_uses_the_prefix_the_schema_reserves_and_not_its_own_name() {
        let report = check(
            "<section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
             data-refs=\"req-x\"><h2>V</h2></section>",
        );
        assert!(
            fired(&report, PREFIX_MISMATCH).is_empty(),
            "{}",
            messages(&report)
        );

        let wrong = check(
            "<section class=\"verification\" id=\"verification-x\" data-outcome=\"passed\" \
             data-refs=\"req-x\"><h2>V</h2></section>",
        );
        assert_eq!(
            fired(&wrong, PREFIX_MISMATCH).len(),
            1,
            "{}",
            messages(&wrong)
        );
    }

    #[test]
    fn a_superseded_decision_with_no_successor_is_reported() {
        let report = check(
            "<section class=\"decision\" id=\"decision-x\" data-status=\"superseded\">\
             <h2>D</h2><p>Chose.</p><p data-field=\"rationale\">Because.</p></section>",
        );
        assert_eq!(
            fired(&report, MISSING_SUCCESSOR).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_superseded_decision_naming_a_successor_is_accepted() {
        let report = check(
            "<section class=\"decision\" id=\"decision-x\" data-status=\"superseded\" \
             data-superseded-by=\"decision-y\"><h2>D</h2><p>Chose.</p>\
             <p data-field=\"rationale\">Because.</p></section>",
        );
        assert!(
            fired(&report, MISSING_SUCCESSOR).is_empty(),
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_dropped_task_with_no_reason_is_reported() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"dropped\"><h2>T</h2></section>",
        );
        assert_eq!(
            fired(&report, MISSING_REASON).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_dropped_task_stating_a_reason_is_accepted() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"dropped\"><h2>T</h2>\
             <p data-field=\"reason\">No requirement asks for it.</p></section>",
        );
        assert!(
            fired(&report, MISSING_REASON).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// A task that is not dropped needs no reason, so the rule must be
    /// conditional on the state rather than on the field's absence.
    #[test]
    fn a_task_that_is_not_dropped_needs_no_reason() {
        for state in ["todo", "in-progress", "done"] {
            let report = check(&format!(
                "<section class=\"task\" id=\"task-x\" data-state=\"{state}\"><h2>T</h2></section>"
            ));
            assert!(
                fired(&report, MISSING_REASON).is_empty(),
                "state {state}: {}",
                messages(&report)
            );
        }
    }

    /// A task in `state` carrying a deviation, and a reason where the state needs one, so the
    /// document violates nothing but the rule under test.
    fn a_task_stating_a_deviation(state: &str) -> crate::Report {
        let reason = if state == "dropped" {
            "<p data-field=\"reason\">It was abandoned.</p>"
        } else {
            ""
        };
        check(&format!(
            "<section class=\"task\" id=\"task-x\" data-state=\"{state}\"><h2>T</h2>{reason}\
             <p data-field=\"deviation\">It was done another way.</p></section>"
        ))
    }

    /// The deviation is reported once, at the task, and the message names the state it found.
    fn assert_reported_once(state: &str) {
        let report = a_task_stating_a_deviation(state);
        let found = fired(&report, DEVIATION_NOT_DONE);
        assert_eq!(found.len(), 1, "state {state}: {}", messages(&report));
        assert_eq!(
            found[0].location.unit_id.as_deref(),
            Some("task-x"),
            "the diagnostic does not name the task: {}",
            messages(&report)
        );
        assert!(
            found[0].message.contains(state),
            "the message does not say which state the task declares: {}",
            found[0].message
        );
    }

    /// Nothing is finished yet, so no account of finished work can be true.
    #[test]
    fn a_deviation_on_a_todo_task_is_reported() {
        assert_reported_once("todo");
    }

    /// The work is still changing, and a note records a difference found along the way.
    #[test]
    fn a_deviation_on_an_in_progress_task_is_reported() {
        assert_reported_once("in-progress");
    }

    /// A dropped task's work did not happen, and its reason says why. Both would let one task
    /// claim the work happened and that it did not.
    #[test]
    fn a_deviation_on_a_dropped_task_is_reported() {
        assert_reported_once("dropped");
    }

    /// The rule keys on the state that permits a deviation, so a task declaring none is not
    /// silently let through. The schema reports the missing state on its own.
    #[test]
    fn a_deviation_on_a_task_declaring_no_state_is_reported() {
        let report = check(
            "<section class=\"task\" id=\"task-x\"><h2>T</h2>\
             <p data-field=\"deviation\">It was done another way.</p></section>",
        );
        let found = fired(&report, DEVIATION_NOT_DONE);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains("no state"),
            "{}",
            found[0].message
        );
    }

    /// The same task without its deviation is not this rule's business, in any state.
    #[test]
    fn a_task_stating_no_deviation_is_not_reported_in_any_state() {
        for state in ["todo", "in-progress", "done", "dropped"] {
            let report = check(&format!(
                "<section class=\"task\" id=\"task-x\" data-state=\"{state}\"><h2>T</h2>\
                 <p data-field=\"reason\">Why.</p><p data-field=\"note\">A note.</p></section>"
            ));
            assert!(
                fired(&report, DEVIATION_NOT_DONE).is_empty(),
                "state {state}: {}",
                messages(&report)
            );
        }
    }

    /// A verification recording `outcome`, carrying `paragraphs` after its method.
    fn verification(outcome: &str, paragraphs: &str) -> String {
        format!(
            "<section class=\"verification\" id=\"verify-x\" data-outcome=\"{outcome}\" \
             data-refs=\"req-x\"><h2>V</h2>\
             <p data-field=\"method\">Ran the suite against the built change.</p>{paragraphs}\
             </section>"
        )
    }

    const UNCHECKED: &str = "<p data-field=\"unchecked\">The Then names both locations, and \
         the record gives only the rule and the exit status.</p>";

    /// document-schema, "A partial verification that does not say what it left".
    #[test]
    fn a_partial_verification_stating_nothing_it_did_not_check_is_reported() {
        let report = check(&verification("partial", ""));
        let found = fired(&report, MISSING_UNCHECKED);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(
            found[0].location.unit_id.as_deref(),
            Some("verify-x"),
            "the report does not name the verification"
        );
        assert!(
            found[0].message.contains("data-field=\"unchecked\""),
            "the message does not name the paragraph to add: {}",
            found[0].message
        );
        assert!(!report.is_accepted(), "{}", messages(&report));
    }

    /// document-schema, "A check that reached part of its scenario", and "Two parts left
    /// unchecked" for the rule's half: one paragraph satisfies it, and so do two.
    #[test]
    fn a_partial_verification_stating_what_it_did_not_check_is_accepted() {
        for paragraphs in [UNCHECKED.to_owned(), format!("{UNCHECKED}{UNCHECKED}")] {
            let report = check(&verification("partial", &paragraphs));
            assert!(
                fired(&report, MISSING_UNCHECKED).is_empty(),
                "{}",
                messages(&report)
            );
        }
    }

    #[test]
    fn a_deviation_on_a_done_task_is_accepted() {
        let report = a_task_stating_a_deviation("done");
        assert!(report.is_accepted(), "{}", messages(&report));
        assert!(
            fired(&report, DEVIATION_NOT_DONE).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// A risk with a statement, the given attribute text on its element, and the given
    /// paragraphs after its statement.
    fn risk(attribute: &str, after_statement: &str) -> crate::Report {
        check(&format!(
            "<section class=\"risk\" id=\"risk-x\"{attribute}><h2>K</h2>\
             <p>How it fails.</p>{after_statement}</section>"
        ))
    }

    const A_MITIGATION: &str = "<p data-field=\"mitigation\">What holds it down.</p>";

    #[test]
    fn a_risk_declared_mitigated_with_no_mitigation_is_reported() {
        let report = risk(" data-response=\"mitigated\"", "");
        let found = fired(&report, MISSING_MITIGATION);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert_eq!(
            found[0].location,
            Location::unit("risk-x"),
            "{}",
            messages(&report)
        );
    }

    /// document-schema, "An unmarked paragraph does not state it". Section 4.0.2 makes an
    /// unmarked paragraph in a verification a note, and the rule reads the field.
    #[test]
    fn an_unmarked_paragraph_does_not_state_what_a_partial_check_missed() {
        for paragraph in [
            "<p>The Then names both locations, and the record gives only the rule.</p>",
            "<p data-field=\"note\">The Then names both locations, and the record gives only \
             the rule.</p>",
        ] {
            let report = check(&verification("partial", paragraph));
            assert_eq!(
                fired(&report, MISSING_UNCHECKED).len(),
                1,
                "{paragraph}: {}",
                messages(&report)
            );
        }
    }

    /// document-schema, "A failed check that says what it did not reach".
    #[test]
    fn a_failed_verification_may_state_what_it_did_not_reach() {
        let report = check(&verification("failed", UNCHECKED));
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// document-schema, "Any other outcome without it". Only `partial` obliges the paragraph,
    /// so the rule is conditional on the outcome rather than on the field's absence. Every
    /// other outcome the schema enumerates is checked, so a value added later is too.
    #[test]
    fn a_verification_that_is_not_partial_needs_no_unchecked_paragraph() {
        let others: Vec<String> =
            schema_facts::enum_values("definitions/verification/properties/outcome")
                .into_iter()
                .filter(|outcome| outcome != "partial")
                .collect();
        assert!(
            others.iter().any(|outcome| outcome == "passed"),
            "{others:?}"
        );
        for outcome in others {
            let report = check(&verification(&outcome, ""));
            assert!(
                report.is_accepted(),
                "outcome {outcome}: {}",
                messages(&report)
            );
        }
    }

    /// The case the rule exists for most: a risk written before `data-response` existed is
    /// `mitigated` by default, so it is held to a mitigation as every risk was.
    #[test]
    fn a_risk_that_declares_no_response_and_states_no_mitigation_is_reported() {
        let report = risk("", "");
        let found = fired(&report, MISSING_MITIGATION);
        assert_eq!(found.len(), 1, "{}", messages(&report));
    }

    /// The message names both remedies, and says why a risk with no attribute is held to a
    /// mitigation, because its author most likely wrote it before the attribute existed.
    #[test]
    fn the_report_names_both_remedies_and_the_default() {
        let report = risk("", "");
        let found = fired(&report, MISSING_MITIGATION);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        let message = &found[0].message;
        for expected in [
            "data-field=\"mitigation\"",
            "data-response=\"accepted\"",
            "declares no data-response is mitigated",
        ] {
            assert!(
                message.contains(expected),
                "{expected:?} is not in {message}"
            );
        }
    }

    #[test]
    fn an_accepted_risk_needs_no_mitigation() {
        for after_statement in ["", A_MITIGATION] {
            let report = risk(" data-response=\"accepted\"", after_statement);
            assert!(
                fired(&report, MISSING_MITIGATION).is_empty(),
                "with {after_statement:?}: {}",
                messages(&report)
            );
        }
    }

    #[test]
    fn a_mitigated_risk_stating_its_mitigation_is_accepted() {
        for attribute in ["", " data-response=\"mitigated\""] {
            let report = risk(attribute, A_MITIGATION);
            assert!(
                fired(&report, MISSING_MITIGATION).is_empty(),
                "with {attribute:?}: {}",
                messages(&report)
            );
        }
    }

    /// A response outside the enumeration is the schema's `schema/invalid`, which names the
    /// permitted values. Reporting a missing mitigation as well would name one defect twice.
    #[test]
    fn a_response_outside_the_enumeration_is_left_to_the_schema() {
        for value in ["partial", "Mitigated", ""] {
            let report = risk(&format!(" data-response=\"{value}\""), "");
            assert!(
                fired(&report, MISSING_MITIGATION).is_empty(),
                "with {value:?}: {}",
                messages(&report)
            );
        }
    }

    #[test]
    fn a_requirement_relying_on_should_states_no_obligation() {
        let report = check(&format!(
            "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
             <p>The system should probably work.</p>{SCENARIO}</section>"
        ));
        assert_eq!(fired(&report, WEAK_MODAL).len(), 1, "{}", messages(&report));
    }

    #[test]
    fn a_lowercase_must_in_passing_prose_is_not_an_obligation() {
        // The contract writes the modal in capitals precisely so that prose using
        // the word in passing does not read as normative.
        let report = check(&format!(
            "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
             <p>A reader must not confuse this with an obligation.</p>{SCENARIO}</section>"
        ));
        assert_eq!(fired(&report, WEAK_MODAL).len(), 1, "{}", messages(&report));
    }

    #[test]
    fn either_shall_or_must_satisfies_the_modal_rule() {
        for modal in ["SHALL", "MUST"] {
            let report = check(&format!(
                "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
                 <p>The system {modal} work.</p>{SCENARIO}</section>"
            ));
            assert!(
                fired(&report, WEAK_MODAL).is_empty(),
                "{modal}: {}",
                messages(&report)
            );
        }
    }

    /// One statement carrying the modal is enough; a requirement may explain
    /// itself in additional prose.
    #[test]
    fn one_normative_statement_among_several_satisfies_the_rule() {
        let report = check(&format!(
            "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
             <p>Context for what follows.</p><p>The system MUST work.</p>{SCENARIO}</section>"
        ));
        assert!(
            fired(&report, WEAK_MODAL).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// An empty statement list is the schema's `minItems` fault. Reporting it
    /// here as well would name one defect twice.
    #[test]
    fn a_requirement_with_no_statement_is_left_to_the_schema() {
        let report = check(&format!(
            "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>{SCENARIO}</section>"
        ));
        assert!(
            fired(&report, WEAK_MODAL).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// Each conditional rule finds the enumerated value it keys on. If a schema
    /// rename broke the lookup, the rule would silently stop firing, and every
    /// test above it would still pass.
    #[test]
    fn every_conditional_rule_finds_its_state() {
        assert_eq!(
            SUPERSEDED.value().as_deref(),
            Some("superseded"),
            "the superseded-decision rule cannot find its status and has stopped firing"
        );
        assert_eq!(
            DROPPED.value().as_deref(),
            Some("dropped"),
            "the dropped-task rule cannot find its state and has stopped firing"
        );
        assert_eq!(
            DONE.value().as_deref(),
            Some("done"),
            "the deviation rule cannot find the state it permits and has stopped firing"
        );
        assert_eq!(
            PARTIAL.value().as_deref(),
            Some("partial"),
            "the partial-verification rule cannot find its outcome and has stopped firing"
        );
        assert_eq!(
            MITIGATED.value().as_deref(),
            Some("mitigated"),
            "the mitigated-risk rule cannot find its response and has stopped firing"
        );
        assert_eq!(
            ACCEPTED.value().as_deref(),
            Some("accepted"),
            "the mitigated-risk rule cannot find the response it names as a remedy, and has \
             stopped firing"
        );
    }

    #[test]
    fn an_ambiguous_or_absent_stem_yields_nothing_rather_than_a_guess() {
        // `e` matches several states, and a rule keyed on a guess would fire on
        // the wrong documents.
        assert_eq!(status_named("e", "definitions/task/properties/state"), None);
        assert_eq!(
            status_named("nonesuch", "definitions/task/properties/state"),
            None
        );
    }

    /// The uniqueness and prefix rules reach every unit type, so a unit added to
    /// the model cannot escape them.
    #[test]
    fn the_per_unit_rules_reach_every_unit_type() {
        let report = check(
            "<section class=\"requirement\" id=\"wrong-requirement\"><h2>R</h2>\
             <p>The system MUST work.</p>\
             <article class=\"scenario\" id=\"wrong-scenario\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>\
             <section class=\"decision\" id=\"wrong-decision\" data-status=\"accepted\">\
             <h2>D</h2><p>Chose.</p><p data-field=\"rationale\">Because.</p></section>\
             <section class=\"task\" id=\"wrong-task\" data-state=\"todo\"><h2>T</h2></section>\
             <section class=\"evidence\" id=\"wrong-evidence\"><h2>E</h2><p>Measured.</p></section>\
             <section class=\"risk\" id=\"wrong-risk\"><h2>K</h2><p>Fails.</p>\
             <p data-field=\"mitigation\">Held.</p></section>\
             <section class=\"verification\" id=\"wrong-verification\" data-outcome=\"passed\" \
             data-refs=\"req-x\"><h2>V</h2></section>",
        );
        assert_eq!(
            fired(&report, PREFIX_MISMATCH).len(),
            UnitKind::ALL.len(),
            "a unit type escaped the prefix rule: {}",
            messages(&report)
        );
    }

    /// The split is a claim about what is *not* here, so it is asserted rather
    /// than left to a comment. These two constraints belong to the schema and to
    /// the projection, and duplicating them in Rust is the drift to prevent.
    #[test]
    fn the_schema_owns_what_it_can_express() {
        // Comments are excluded: the module documentation explains *why* these
        // two rules are absent, and naming them there is the point rather than a
        // violation. Only executable lines are checked.
        let code: String = include_str!("relations.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the module has a body")
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            !code.contains("scenarios"),
            "\"a requirement has at least one scenario\" is the schema's minItems, and \
             this module must not restate it"
        );
        assert!(
            !code.contains("enclos"),
            "\"a scenario does not appear outside a requirement\" is projection rule 4.3, \
             and this module must not restate it"
        );
    }

    #[test]
    fn every_dogfood_document_satisfies_the_relational_rules() {
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
                "{path:?} failed the relational rules:\n{}",
                messages(&report)
            );
        }
    }
}
