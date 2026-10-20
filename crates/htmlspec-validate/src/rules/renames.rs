//! The rules that bound a renamed requirement.
//!
//! A requirement states the heading it replaced in one `data-field="renamed-from"`
//! paragraph (projection specification, section 4.1.1). Six things make that
//! statement meaningless, and each is a structural fact about one document, like a
//! `dropped` task with no reason. So each is an error with its own code rather than a
//! review warning, and a reader told which of the six they hit does not have to work
//! out which one a generic message meant.
//!
//! None of the six is expressible in the schema. Placement depends on which unit a
//! paragraph sits in, two of them depend on the requirement's change state, and
//! "repeated" and "empty" are about paragraphs the model does not carry. That is why
//! the projection keeps every `renamed-from` paragraph, including the ones that
//! project nothing, and records the misplaced ones beside the model.
//!
//! A rename is compared with the requirement's *heading name* (section 4.1.2), not its
//! title. The template shows each identifier inside its heading, so the title of
//! `<h3>The name <span class="hs-id">req-x</span></h3>` is `The name req-x`, and a
//! comparison with it would never find a rename to the same name.

use htmlspec_parse::model::{HeadingName, RenamedFrom};
use htmlspec_parse::Projection;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};
use crate::schema_facts;

/// A `renamed-from` paragraph in a unit that is not a requirement.
pub const OUTSIDE_REQUIREMENT: RuleId =
    RuleId::new(Namespace::Schema, "rename-outside-requirement");

/// A `renamed-from` paragraph in a requirement this change introduces.
pub const ON_ADDED: RuleId = RuleId::new(Namespace::Schema, "rename-on-added");

/// A `renamed-from` paragraph in a requirement this change withdraws.
pub const ON_REMOVED: RuleId = RuleId::new(Namespace::Schema, "rename-on-removed");

/// More than one `renamed-from` paragraph in one requirement.
pub const REPEATED: RuleId = RuleId::new(Namespace::Schema, "rename-repeated");

/// A `renamed-from` paragraph with no text.
pub const EMPTY: RuleId = RuleId::new(Namespace::Schema, "rename-empty");

/// A `renamed-from` paragraph that states the requirement's own heading.
pub const TO_SAME_NAME: RuleId = RuleId::new(Namespace::Schema, "rename-to-same-name");

/// Runs every rename rule.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    renames_are_on_requirements(projection, diagnostics);
    renames_are_on_requirements_that_had_a_heading(projection, diagnostics);
    a_requirement_states_one_rename(projection, diagnostics);
    a_rename_states_a_heading(projection, diagnostics);
    a_rename_changes_the_heading(projection, diagnostics);
}

/// A rename is stated only on a requirement.
///
/// OpenSpec renames requirements and nothing else, and nothing here addresses any
/// other unit across changes by its heading. A scenario's heading can change under
/// `modified` today.
fn renames_are_on_requirements(projection: &Projection, diagnostics: &mut Diagnostics) {
    for misplaced in &projection.misplaced_renames {
        diagnostics.violation(
            OUTSIDE_REQUIREMENT,
            Location::maybe_unit(misplaced.unit_id.as_deref()),
            format!(
                "a {} states a renamed-from paragraph, and only a requirement can state the \
                 heading it replaced. Remove the paragraph; a {} whose heading changed says so \
                 with data-change=\"modified\"",
                misplaced.unit, misplaced.unit
            ),
        );
    }
}

/// A rename is not stated on a requirement this change introduces or withdraws.
///
/// An added requirement had no previous heading to replace. A removed one is being
/// withdrawn, not renamed, and OpenSpec rejects a rename and a removal of one
/// requirement for the same reason. The state names are read from the schema.
fn renames_are_on_requirements_that_had_a_heading(
    projection: &Projection,
    diagnostics: &mut Diagnostics,
) {
    let added = change_state("add");
    let removed = change_state("remov");
    for requirement in &projection.model.requirements {
        if requirement.renamed_from.is_empty() {
            continue;
        }
        let location = || Location::maybe_unit(requirement.id.as_deref());
        if added.as_deref() == Some(requirement.change.as_str()) {
            diagnostics.violation(
                ON_ADDED,
                location(),
                format!(
                    "a requirement with change state {} states the heading it replaced, and a \
                     requirement this change introduces had no previous heading. If it replaces \
                     an accepted requirement, give it that requirement's identifier and mark it \
                     modified or unchanged; otherwise remove the renamed-from paragraph",
                    requirement.change
                ),
            );
        }
        if removed.as_deref() == Some(requirement.change.as_str()) {
            diagnostics.violation(
                ON_REMOVED,
                location(),
                format!(
                    "a requirement with change state {} states the heading it replaced, and a \
                     requirement being withdrawn is not also renamed. Remove the renamed-from \
                     paragraph, and state the heading the accepted specification holds",
                    requirement.change
                ),
            );
        }
    }
}

/// A requirement states at most one previous heading.
///
/// One rename replaces one heading. A chain of renames inside one change is stated as
/// its first heading, because the ones between were never accepted.
fn a_requirement_states_one_rename(projection: &Projection, diagnostics: &mut Diagnostics) {
    for requirement in &projection.model.requirements {
        let count = requirement.renamed_from.len();
        if count < 2 {
            continue;
        }
        diagnostics.violation(
            REPEATED,
            Location::maybe_unit(requirement.id.as_deref()),
            format!(
                "a requirement carries {count} renamed-from paragraphs, and one rename replaces \
                 one heading. Keep the one naming the heading the accepted specification holds"
            ),
        );
    }
}

/// A rename states the heading it replaced.
fn a_rename_states_a_heading(projection: &Projection, diagnostics: &mut Diagnostics) {
    for requirement in &projection.model.requirements {
        if !requirement.renamed_from.contains(&RenamedFrom::Empty) {
            continue;
        }
        diagnostics.violation(
            EMPTY,
            Location::maybe_unit(requirement.id.as_deref()),
            "a requirement carries an empty renamed-from paragraph, so it says it was renamed \
             without saying from what. State the previous heading, or remove the paragraph",
        );
    }
}

/// A rename names a heading other than the requirement's own.
///
/// Compared by heading name (projection specification, section 4.1.2) after whitespace
/// normalization, and case-sensitively, so a rename that changes only case is a rename.
fn a_rename_changes_the_heading(projection: &Projection, diagnostics: &mut Diagnostics) {
    for requirement in &projection.model.requirements {
        let Some(own) = requirement.heading_name() else {
            // No heading is projection fault 3.3, reported by `schema/*`.
            continue;
        };
        let same = requirement
            .renamed_from
            .iter()
            .any(|paragraph| match paragraph {
                RenamedFrom::Heading(previous) => HeadingName::of(previous, None) == own,
                RenamedFrom::Empty => false,
            });
        if !same {
            continue;
        }
        diagnostics.violation(
            TO_SAME_NAME,
            Location::maybe_unit(requirement.id.as_deref()),
            format!(
                "a requirement's renamed-from paragraph states its own heading, \"{own}\", so it \
                 renames nothing. State the heading it replaced, or remove the paragraph"
            ),
        );
    }
}

/// The change state whose name contains `stem`, read from the schema.
///
/// Found by a stem of the concept's English name rather than written out, so the
/// contract value stays in the schema, as `refs::removed_state` does. A stem
/// matching none or several yields `None`, and the rule that needed it stops firing,
/// which `the_added_and_removed_states_are_found_in_the_schema` makes a failing test.
fn change_state(stem: &str) -> Option<String> {
    let mut matches = schema_facts::enum_values("definitions/changeState")
        .into_iter()
        .filter(|value| value.contains(stem));
    let found = matches.next()?;
    matches.next().is_none().then_some(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};

    const NO_STORE: htmlspec_assets::Store = htmlspec_assets::Store::Absent;
    use htmlspec_parse::{parse_document, project};

    fn check(body: &str) -> crate::Report {
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.2.0\">\
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

    /// A requirement in `change` state, headed `heading`, carrying `paragraphs` as its
    /// `renamed-from` paragraphs.
    fn requirement(change: &str, heading: &str, paragraphs: &[&str]) -> String {
        let renames: String = paragraphs
            .iter()
            .map(|text| format!("<p data-field=\"renamed-from\">{text}</p>"))
            .collect();
        format!(
            "<section class=\"requirement\" id=\"req-x\" data-change=\"{change}\">\
             <h2>{heading} <span class=\"hs-id\">req-x</span></h2>{renames}\
             <p>The system MUST work.</p>{SCENARIO}</section>"
        )
    }

    /// The document-schema scenarios "A requirement renamed and rewritten" and "A
    /// requirement renamed and otherwise untouched": both are valid.
    #[test]
    fn a_rename_on_a_modified_or_unchanged_requirement_reports_nothing() {
        for change in ["modified", "unchanged"] {
            let report = check(&requirement(change, "The new name", &["The old name"]));
            assert!(
                report.diagnostics.is_empty(),
                "{change}: {}",
                messages(&report)
            );
        }
    }

    /// A requirement with no rename is untouched by every rule here, whatever its state.
    #[test]
    fn a_requirement_with_no_rename_reports_nothing() {
        for change in ["added", "modified", "removed", "unchanged"] {
            let report = check(&requirement(change, "The name", &[]));
            assert!(
                report.diagnostics.is_empty(),
                "{change}: {}",
                messages(&report)
            );
        }
    }

    /// "A rename outside a requirement", for every unit the scenario names.
    #[test]
    fn a_rename_outside_a_requirement_is_reported_once_per_unit() {
        let rename = "<p data-field=\"renamed-from\">An old name</p>";
        let units = [
            format!(
                "<section class=\"requirement\" id=\"req-x\"><h2>R</h2><p>It MUST work.</p>\
                 <article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>{rename}{rename}\
                 <dl><dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd></dl>\
                 </article></section>"
            ),
            format!(
                "<section class=\"decision\" id=\"decision-x\" data-status=\"accepted\">\
                 <h2>D</h2><p>Chosen.</p><p data-field=\"rationale\">Why.</p>{rename}</section>"
            ),
            format!(
                "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>{rename}</section>"
            ),
            format!(
                "<section class=\"evidence\" id=\"evidence-x\"><h2>E</h2><p>Seen.</p>{rename}</section>"
            ),
            format!(
                "<section class=\"risk\" id=\"risk-x\"><h2>K</h2><p>It fails.</p>\
                 <p data-field=\"mitigation\">Held.</p>{rename}</section>"
            ),
            format!(
                "<section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
                 data-refs=\"req-x\"><h2>V</h2>{rename}</section>"
            ),
        ];
        for unit in units {
            let report = check(&unit);
            let found = fired(&report, OUTSIDE_REQUIREMENT);
            assert_eq!(found.len(), 1, "{unit}\n{}", messages(&report));
            assert_eq!(
                report.diagnostics.len(),
                1,
                "one misplaced rename is one diagnostic: {}",
                messages(&report)
            );
        }
    }

    /// "A rename on a new requirement".
    #[test]
    fn a_rename_on_an_added_requirement_is_reported() {
        let report = check(&requirement("added", "The new name", &["The old name"]));
        assert_eq!(fired(&report, ON_ADDED).len(), 1, "{}", messages(&report));
    }

    /// "A rename on a withdrawn requirement".
    #[test]
    fn a_rename_on_a_removed_requirement_is_reported() {
        let report = check(&requirement("removed", "The new name", &["The old name"]));
        assert_eq!(fired(&report, ON_REMOVED).len(), 1, "{}", messages(&report));
    }

    /// "Two previous headings".
    #[test]
    fn two_renames_on_one_requirement_are_reported_once() {
        let report = check(&requirement(
            "modified",
            "The new name",
            &["The old name", "An older name", "The oldest name"],
        ));
        assert_eq!(fired(&report, REPEATED).len(), 1, "{}", messages(&report));
    }

    /// An empty paragraph is a rename stated without its heading.
    #[test]
    fn an_empty_rename_is_reported() {
        let report = check(&requirement("modified", "The new name", &[" \n "]));
        assert_eq!(fired(&report, EMPTY).len(), 1, "{}", messages(&report));
        // The only paragraph is empty, so there is no second one to call repeated.
        assert!(fired(&report, REPEATED).is_empty(), "{}", messages(&report));
    }

    /// "A rename to the same name", and "A heading that shows its identifier": the
    /// heading is `The name <span class="hs-id">req-x</span>`, whose title is
    /// `The name req-x`, and the rename states `The name`.
    #[test]
    fn a_rename_to_the_requirements_own_heading_name_is_reported() {
        let report = check(&requirement("modified", "The name", &["The  name"]));
        assert_eq!(
            fired(&report, TO_SAME_NAME).len(),
            1,
            "{}",
            messages(&report)
        );
    }

    /// OpenSpec allows a rename that changes only case, and so does this.
    #[test]
    fn a_rename_that_changes_only_case_is_a_rename() {
        let report = check(&requirement("modified", "The Name", &["The name"]));
        assert!(
            fired(&report, TO_SAME_NAME).is_empty(),
            "{}",
            messages(&report)
        );
    }

    /// Each fault is its own code, so one requirement can report several.
    #[test]
    fn independent_faults_on_one_requirement_are_each_reported() {
        let report = check(&requirement("added", "The name", &["The name", ""]));
        for rule in [ON_ADDED, REPEATED, EMPTY, TO_SAME_NAME] {
            assert_eq!(
                fired(&report, rule).len(),
                1,
                "{rule}: {}",
                messages(&report)
            );
        }
    }

    /// The two states these rules name are read from the schema. If a schema rename
    /// broke either lookup, the rule depending on it would stop firing silently, so
    /// finding them is asserted rather than assumed.
    #[test]
    fn the_added_and_removed_states_are_found_in_the_schema() {
        assert!(change_state("add").is_some());
        assert!(change_state("remov").is_some());
    }

    #[test]
    fn a_diagnostic_names_the_requirement() {
        let report = check(&requirement("removed", "The new name", &["The old name"]));
        let found = fired(&report, ON_REMOVED);
        assert_eq!(found[0].location.unit_id.as_deref(), Some("req-x"));
    }
}
