//! Rendering a [`Report`] for a person and for a program.
//!
//! Both forms carry the same four things — the rule identifier, the severity, the
//! location, and the offending identifier where one applies — because a
//! contributor reading a terminal and an agent reading a pipe are acting on the
//! same facts and must not receive different ones.
//!
//! The machine form is a declared wire type rather than a serde derive on
//! [`Diagnostic`]. That costs a mapping and buys the thing the specs actually
//! require: the payload's field names are a decision, so renaming an internal
//! field cannot silently break a consumer. `Deserialize` is derived alongside
//! `Serialize` so the CLI's own tests parse the payload back into this type
//! instead of poking at untyped JSON.
//!
//! Nothing here performs I/O. Which stream a rendering goes to, and what exit
//! code accompanies it, belong to the CLI.

use serde::{Deserialize, Serialize};

use crate::diagnostic::{Diagnostic, Location};
use crate::engine::{Curtailment, Report};

/// One diagnostic, as a program receives it.
///
/// `namespace` is derivable from `rule` by splitting on the slash, and is present
/// anyway: it is what carries the severity policy, so a consumer deciding whether
/// a violation could ever have been a warning should not have to parse a string
/// to find out.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredDiagnostic {
    /// The stable rule identifier, such as `refs/unresolved`.
    pub rule: String,
    /// The concern the rule belongs to, being `rule` up to the slash.
    pub namespace: String,
    /// Whether this fails validation, after policy.
    pub severity: String,
    /// What is wrong, in a form a document author can act on.
    pub message: String,
    /// The identifier of the unit at fault, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_id: Option<String>,
    /// A JSON pointer into the projected model, when the rule has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_path: Option<String>,
    /// The projection-specification section, for a rule that implements one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_rule: Option<String>,
    /// A limitation of the check itself, where one applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl StructuredDiagnostic {
    /// The wire form of one diagnostic.
    ///
    /// Public because a project-level check reports diagnostics without a per-document
    /// [`StructuredReport`] around them, and a consumer parsing one shape should not
    /// have to learn a second for the same diagnostic.
    pub fn of(diagnostic: &Diagnostic) -> Self {
        let Location {
            unit_id,
            schema_path,
            projection_rule,
        } = &diagnostic.location;
        Self {
            rule: diagnostic.rule.to_string(),
            namespace: diagnostic.rule.namespace().as_str().to_owned(),
            severity: diagnostic.severity.as_str().to_owned(),
            message: diagnostic.message.clone(),
            unit_id: unit_id.clone(),
            schema_path: schema_path.clone(),
            projection_rule: projection_rule.clone(),
            note: diagnostic.note.clone(),
        }
    }
}

/// Why a run stopped before running every rule, as a program receives it.
///
/// `reason` is a stable slug and not the prose. A consumer branching on why a run
/// was curtailed must not have to match on a sentence that may be reworded.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredCurtailment {
    /// The stable reason slug, such as `fatal-parse-error`.
    pub reason: String,
    /// How many parse recoveries were reported, for a parse curtailment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovered_errors: Option<usize>,
    /// The same fact in prose, for a reader who receives only this payload.
    pub message: String,
}

impl StructuredCurtailment {
    fn of(curtailment: &Curtailment) -> Self {
        match curtailment {
            Curtailment::FatalParseError { recovered } => Self {
                reason: "fatal-parse-error".to_owned(),
                recovered_errors: Some(*recovered),
                message: curtailment.to_string(),
            },
        }
    }
}

/// How many diagnostics of each severity a run produced.
///
/// Present so a consumer can report a verdict without walking the list, and so a
/// payload truncated in a log still says how much was found.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Summary {
    /// Diagnostics that fail validation.
    pub errors: usize,
    /// Diagnostics that do not.
    pub warnings: usize,
}

/// A whole report, as a program receives it.
///
/// `accepted` is stated rather than left to be derived, because deriving it
/// wrongly is easy: an empty diagnostic list on a curtailed run does not mean the
/// document passed, it means the rules never ran.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredReport {
    /// Whether the document is accepted.
    pub accepted: bool,
    /// How many diagnostics of each severity were reported.
    pub summary: Summary,
    /// Why evaluation stopped early, when it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curtailed: Option<StructuredCurtailment>,
    /// Every diagnostic, in the order the rules produced them.
    pub diagnostics: Vec<StructuredDiagnostic>,
}

impl StructuredReport {
    /// Renders a report into its machine-readable form.
    pub fn of(report: &Report) -> Self {
        Self {
            accepted: report.is_accepted(),
            summary: Summary {
                errors: report.errors().count(),
                warnings: report.warnings().count(),
            },
            curtailed: report.curtailed.as_ref().map(StructuredCurtailment::of),
            diagnostics: report
                .diagnostics
                .iter()
                .map(StructuredDiagnostic::of)
                .collect(),
        }
    }
}

impl From<&Report> for StructuredReport {
    fn from(report: &Report) -> Self {
        Self::of(report)
    }
}

/// Renders a report as the machine-readable payload, pretty-printed.
///
/// The error is returned rather than absorbed. Serializing these types cannot
/// fail in practice, and a helper that hid a failure behind a panic or an empty
/// string would be asserting that rather than establishing it.
pub fn render_json(report: &Report) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&StructuredReport::of(report))
}

/// Renders a report for a person reading a terminal.
///
/// One diagnostic per line, in the order the rules produced them, with a note on
/// its own indented line. The per-diagnostic line is [`Diagnostic`]'s own
/// `Display`, so the human form cannot drift from the way a diagnostic reads
/// anywhere else.
///
/// The verdict comes last, where a reader who has just scrolled through the
/// diagnostics will see it, and it always states the curtailment: a reader who
/// sees three diagnostics and no verdict has no way to tell a document with three
/// faults from a document whose rules never ran.
pub fn render_human(report: &Report) -> String {
    let mut out = String::new();
    for diagnostic in &report.diagnostics {
        out.push_str(&diagnostic.to_string());
        out.push('\n');
    }

    if !report.diagnostics.is_empty() {
        out.push('\n');
    }
    out.push_str(&verdict(report));
    out.push('\n');
    out
}

/// The last line (or two) of the human form: what was found, and the outcome.
fn verdict(report: &Report) -> String {
    let errors = report.errors().count();
    let warnings = report.warnings().count();

    let mut verdict = match (errors, warnings) {
        (0, 0) => "no diagnostics".to_owned(),
        _ => format!(
            "{}, {}",
            plural(errors, "error"),
            plural(warnings, "warning")
        ),
    };

    match &report.curtailed {
        // A curtailed run is never accepted, and saying only "failed" would leave
        // the reader to wonder why the list is so short.
        Some(curtailment) => {
            verdict.push_str(": validation failed\n");
            verdict.push_str(&curtailment.to_string());
        }
        None if report.is_accepted() => verdict.push_str(": the document is accepted"),
        None => verdict.push_str(": validation failed"),
    }
    verdict
}

fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Namespace, RuleId, Severity};
    use crate::engine::{run, Diagnostics, Strictness};
    use htmlspec_parse::{parse_document, project, ParseOutcome, Projection};

    const CLEAN: &str = "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>T</title></head><body><p>Body.</p></body></html>\n";

    /// A document with no doctype, which the parser recovers from.
    const RECOVERED: &str = "<html><head><title>T</title></head><body></body></html>";

    fn parse(html: &str) -> (ParseOutcome, Projection) {
        let outcome = parse_document(html);
        let projection = project(&outcome.document);
        (outcome, projection)
    }

    fn located_rule(_: &crate::engine::Subject<'_>, diagnostics: &mut Diagnostics) {
        diagnostics.report(Diagnostic::new(
            RuleId::new(Namespace::Refs, "unresolved"),
            Location::unit("task-4")
                .at_schema_path("/tasks/0/references/0")
                .at_projection_rule("5.2"),
            "the reference names req-absent, and no unit declares it",
        ));
    }

    fn noted_warning(_: &crate::engine::Subject<'_>, diagnostics: &mut Diagnostics) {
        diagnostics.report(
            Diagnostic::new(
                RuleId::new(Namespace::Review, "stale-reference"),
                Location::unit("verify-1"),
                "the requirement it references changed",
            )
            .with_note("reconciliation clears this"),
        );
    }

    /// A warning in a namespace strict mode does promote, for contrast with the
    /// `review/*` one above.
    fn promotable_warning(_: &crate::engine::Subject<'_>, diagnostics: &mut Diagnostics) {
        let mut diagnostic = Diagnostic::new(
            RuleId::new(Namespace::A11y, "figure-without-caption"),
            Location::unit("figure-1"),
            "this figure has no caption",
        );
        diagnostic.severity = Severity::Warning;
        diagnostics.report(diagnostic);
    }

    fn document_wide_safety(_: &crate::engine::Subject<'_>, diagnostics: &mut Diagnostics) {
        diagnostics.violation(
            RuleId::new(Namespace::Safety, "remote-resource"),
            Location::document(),
            "a script is loaded from https://example.test/x.js",
        );
    }

    fn report_of(rules: &[crate::engine::Rule], html: &str, strictness: Strictness) -> Report {
        let (outcome, projection) = parse(html);
        run(
            &outcome,
            &projection,
            &htmlspec_assets::Store::Absent,
            &crate::references::Scope::loose(),
            rules,
            strictness,
        )
    }

    /// The four things the specs require of every diagnostic, in both forms.
    #[test]
    fn both_forms_carry_the_rule_severity_location_and_identifier() {
        let report = report_of(&[located_rule], CLEAN, Strictness::Default);

        let human = render_human(&report);
        for expected in [
            "error",
            "refs/unresolved",
            "task-4",
            "/tasks/0/references/0",
            "req-absent",
        ] {
            assert!(
                human.contains(expected),
                "human form lost {expected}:\n{human}"
            );
        }

        let structured = StructuredReport::of(&report);
        let diagnostic = &structured.diagnostics[0];
        assert_eq!(diagnostic.rule, "refs/unresolved");
        assert_eq!(diagnostic.namespace, "refs");
        assert_eq!(diagnostic.severity, "error");
        assert_eq!(diagnostic.unit_id.as_deref(), Some("task-4"));
        assert_eq!(
            diagnostic.schema_path.as_deref(),
            Some("/tasks/0/references/0")
        );
        assert_eq!(diagnostic.projection_rule.as_deref(), Some("5.2"));
        assert!(diagnostic.message.contains("req-absent"));
    }

    /// A diagnostic with nothing to say about a field omits it rather than
    /// sending a null a consumer has to special-case.
    #[test]
    fn an_absent_location_field_is_omitted_rather_than_null() {
        let report = report_of(&[document_wide_safety], CLEAN, Strictness::Default);
        let json = render_json(&report).expect("the payload serializes");
        assert!(
            !json.contains("null"),
            "a null reached the payload:\n{json}"
        );
        assert!(
            !json.contains("unitId"),
            "an absent unit was named:\n{json}"
        );
    }

    #[test]
    fn the_note_travels_with_the_diagnostic_in_both_forms() {
        let report = report_of(&[noted_warning], CLEAN, Strictness::Default);
        assert!(render_human(&report).contains("note: reconciliation clears this"));
        assert_eq!(
            StructuredReport::of(&report).diagnostics[0].note.as_deref(),
            Some("reconciliation clears this")
        );
    }

    #[test]
    fn a_clean_run_says_the_document_is_accepted() {
        let report = report_of(&[], CLEAN, Strictness::Default);
        let human = render_human(&report);
        assert_eq!(human, "no diagnostics: the document is accepted\n");

        let structured = StructuredReport::of(&report);
        assert!(structured.accepted);
        assert_eq!(structured.summary, Summary::default());
        assert!(structured.diagnostics.is_empty());
        assert!(structured.curtailed.is_none());
    }

    #[test]
    fn the_verdict_counts_each_severity() {
        let report = report_of(
            &[located_rule, noted_warning, document_wide_safety],
            CLEAN,
            Strictness::Default,
        );
        assert!(render_human(&report).contains("2 errors, 1 warning: validation failed"));
        assert_eq!(
            StructuredReport::of(&report).summary,
            Summary {
                errors: 2,
                warnings: 1
            }
        );
    }

    /// A warning alone is reported and the document still passes, so the verdict
    /// must not read as a failure.
    #[test]
    fn a_warning_alone_is_reported_and_still_accepted() {
        let report = report_of(&[noted_warning], CLEAN, Strictness::Default);
        assert!(render_human(&report).contains("0 errors, 1 warning: the document is accepted"));
        assert!(StructuredReport::of(&report).accepted);
    }

    /// The rendering reports the severity policy applied, not the rule's
    /// intrinsic one, so a strict run and a default run read differently.
    #[test]
    fn the_rendering_reports_the_severity_after_policy() {
        // A `review/*` warning is not promoted even under strict mode, because such
        // a report must never gate work. The rendering reports what policy decided,
        // so it still reads as a warning.
        let review = report_of(&[noted_warning], CLEAN, Strictness::Strict);
        assert_eq!(
            StructuredReport::of(&review).diagnostics[0].severity,
            Severity::Warning.as_str()
        );
        assert!(render_human(&review).starts_with("warning: review/stale-reference"));

        // A promotable warning does read as an error, which is what makes the
        // assertion above about policy rather than about rendering being inert.
        let promotable = report_of(&[promotable_warning], CLEAN, Strictness::Strict);
        assert_eq!(
            StructuredReport::of(&promotable).diagnostics[0].severity,
            Severity::Error.as_str()
        );
        assert!(render_human(&promotable).starts_with("error: a11y/figure-without-caption"));
    }

    /// The failure this exists to prevent: a short list read as a clean document.
    #[test]
    fn a_curtailed_run_says_so_in_both_forms() {
        let report = report_of(&[located_rule], RECOVERED, Strictness::Default);

        let human = render_human(&report);
        assert!(
            human.contains("no further rules were evaluated"),
            "the human form hid the curtailment:\n{human}"
        );
        assert!(human.contains("validation failed"));

        let structured = StructuredReport::of(&report);
        assert!(!structured.accepted);
        let curtailed = structured.curtailed.expect("the run was curtailed");
        assert_eq!(curtailed.reason, "fatal-parse-error");
        assert_eq!(curtailed.recovered_errors, Some(1));
    }

    /// The payload is a contract, so a consumer reads it back into these types.
    #[test]
    fn the_payload_round_trips_through_its_own_types() {
        let report = report_of(
            &[located_rule, noted_warning, document_wide_safety],
            CLEAN,
            Strictness::Default,
        );
        let json = render_json(&report).expect("the payload serializes");
        let parsed: StructuredReport = serde_json::from_str(&json).expect("the payload parses");
        assert_eq!(parsed, StructuredReport::of(&report));
    }

    /// Order is the order the rules produced, in both forms, so a diagnostic
    /// found in one can be located in the other.
    #[test]
    fn both_forms_preserve_the_order_the_rules_produced() {
        let report = report_of(
            &[document_wide_safety, located_rule, noted_warning],
            CLEAN,
            Strictness::Default,
        );
        let rules: Vec<String> = StructuredReport::of(&report)
            .diagnostics
            .iter()
            .map(|d| d.rule.clone())
            .collect();
        assert_eq!(
            rules,
            [
                "safety/remote-resource",
                "refs/unresolved",
                "review/stale-reference"
            ]
        );

        let human = render_human(&report);
        let lines: Vec<&str> = human.lines().collect();
        assert!(lines[0].contains("safety/remote-resource"));
        assert!(lines[1].contains("refs/unresolved"));
        assert!(lines[2].contains("review/stale-reference"));
    }
}
