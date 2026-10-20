//! Pins the projection against the specification it implements.
//!
//! Every test here names the section of `doc/projection.md` it exercises. A test
//! whose section number does not resolve to a rule in that document is testing
//! this implementation's habits rather than the contract, which is the failure
//! mode these tests exist to prevent.
//!
//! The specification's word *invalid* means the projection cannot produce a
//! model and reports a fault (section 10.2). A test for an invalid construct
//! therefore asserts on the fault's rule number, not merely that something went
//! wrong.

use htmlspec_parse::model::{FigureKind, HeadingName, ReferenceOrigin, RenamedFrom, UnitKind};
use htmlspec_parse::{parse_document, project, MisplacedRename, Projection};
use pretty_assertions::assert_eq;

/// Wraps `body` in the minimum document the metadata rules need, so a test about
/// one projection rule is not also a test about section 9.
fn document(body: &str) -> Projection {
    let html = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\n\
         <meta name=\"htmlspec-kind\" content=\"specification\">\n\
         <title>Fixture</title>\n</head>\n<body>\n{body}\n</body>\n</html>\n"
    );
    let outcome = parse_document(&html);
    assert!(
        outcome.is_recovery_free(),
        "the fixture itself does not parse cleanly: {:?}",
        outcome.recovered
    );
    project(&outcome.document)
}

/// A requirement wrapping `inner`, for tests about a scenario or a field.
fn in_requirement(inner: &str) -> Projection {
    document(&format!(
        "<section class=\"requirement\" id=\"req-x\">\n<h2>A requirement</h2>\n\
         <p>The system MUST do the thing.</p>\n{inner}\n</section>"
    ))
}

/// The one scenario of the one requirement, for brevity in body tests.
fn only_scenario(projection: &Projection) -> &htmlspec_parse::model::Scenario {
    let requirement = projection
        .model
        .requirements
        .first()
        .expect("one requirement projected");
    requirement
        .scenarios
        .first()
        .expect("one scenario projected")
}

fn rules(projection: &Projection) -> Vec<&str> {
    projection.faults.iter().map(|f| f.rule).collect()
}

const MINIMAL_SCENARIO: &str = "<article class=\"scenario\" id=\"scenario-x\">\n\
     <h3>A scenario</h3>\n<dl><dt>When</dt><dd>It happens.</dd>\
     <dt>Then</dt><dd>It works.</dd></dl>\n</article>";

// --- Section 1: what creates a unit ---------------------------------------

#[test]
fn section_1_1_a_reserved_class_creates_a_unit() {
    let projection = in_requirement(MINIMAL_SCENARIO);
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let requirement = &projection.model.requirements[0];
    assert_eq!(requirement.unit, UnitKind::Requirement);
    assert_eq!(requirement.id.as_deref(), Some("req-x"));
}

#[test]
fn section_1_2_a_reserved_class_is_one_whole_case_sensitive_token() {
    // `class="requirement note"` creates a requirement.
    let matched = document(&format!(
        "<section class=\"requirement note\" id=\"req-x\"><h2>T</h2>\
         <p>MUST.</p>{MINIMAL_SCENARIO}</section>"
    ));
    assert_eq!(matched.model.requirements.len(), 1);

    // `class="requirements"` creates nothing.
    let unmatched = document("<section class=\"requirements\" id=\"req-x\"><h2>T</h2></section>");
    assert!(unmatched.model.requirements.is_empty());
    assert!(unmatched.is_clean(), "{:?}", unmatched.faults);
}

#[test]
fn section_1_3_two_reserved_classes_on_one_element_is_invalid() {
    let projection =
        document("<section class=\"requirement decision\" id=\"req-x\"><h2>T</h2></section>");
    assert_eq!(rules(&projection), vec!["1.3"]);
    // The projection does not choose one, so neither unit is produced.
    assert!(projection.model.requirements.is_empty());
    assert!(projection.model.decisions.is_empty());
}

#[test]
fn section_1_4_the_tag_name_does_not_affect_the_projection() {
    let mut projected = Vec::new();
    for tag in ["section", "article", "div", "li"] {
        let projection = document(&format!(
            "<{tag} class=\"requirement\" id=\"req-x\"><h2>T</h2><p>MUST.</p>\
             {MINIMAL_SCENARIO}</{tag}>"
        ));
        assert!(projection.is_clean(), "{tag}: {:?}", projection.faults);
        projected.push(projection.model.requirements);
    }
    for other in &projected[1..] {
        assert_eq!(&projected[0], other, "the tag name changed the model");
    }
}

// --- Section 2: attributes -------------------------------------------------

#[test]
fn section_2_an_absent_change_state_projects_as_unchanged() {
    let projection = in_requirement(MINIMAL_SCENARIO);
    assert_eq!(projection.model.requirements[0].change, "unchanged");
}

#[test]
fn section_2_1_data_refs_preserves_order_and_projects_a_repeat_once() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\" \
         data-refs=\"req-b req-a req-b\"><h2>T</h2></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let targets: Vec<&str> = projection.model.tasks[0]
        .references
        .iter()
        .map(|r| r.target.as_str())
        .collect();
    assert_eq!(targets, vec!["req-b", "req-a"]);
}

#[test]
fn section_2_2_an_unknown_data_attribute_is_ignored_rather_than_rejected() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\" \
         data-sprint=\"7\" style=\"color:red\" lang=\"en\"><h2>T</h2></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(projection.model.tasks[0].state.as_deref(), Some("todo"));
}

#[test]
fn section_2_3_an_attribute_value_is_trimmed_and_not_otherwise_normalized() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\" done \"><h2>T</h2></section>",
    );
    assert_eq!(projection.model.tasks[0].state.as_deref(), Some("done"));

    // A wrong case survives projection so the validator reports it. The
    // projection does not judge the model (section 10.1).
    let wrong =
        document("<section class=\"task\" id=\"task-x\" data-state=\"Done\"><h2>T</h2></section>");
    assert_eq!(wrong.model.tasks[0].state.as_deref(), Some("Done"));
    assert!(wrong.is_clean(), "{:?}", wrong.faults);
}

// --- Section 3: common fields ---------------------------------------------

#[test]
fn section_3_1_the_title_is_the_first_heading_among_the_units_own_descendants() {
    let projection = in_requirement(MINIMAL_SCENARIO);
    assert_eq!(
        projection.model.requirements[0].title.as_deref(),
        Some("A requirement")
    );
    // Section 3.2: the scenario's own heading belongs to the scenario.
    assert_eq!(
        only_scenario(&projection).title.as_deref(),
        Some("A scenario")
    );
}

#[test]
fn section_3_2_two_headings_at_the_same_depth_is_invalid() {
    let projection = document(
        "<section class=\"requirement\" id=\"req-x\"><h2>One</h2><h2>Two</h2>\
         <p>MUST.</p></section>",
    );
    assert!(
        rules(&projection).contains(&"3.2"),
        "{:?}",
        projection.faults
    );
}

#[test]
fn section_3_2_a_heading_in_a_presentational_wrapper_is_deeper_and_not_ambiguous() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>The name</h2>\
         <div class=\"callout\"><h2>An aside</h2></div><p>MUST.</p>\
         {MINIMAL_SCENARIO}</section>"
    ));
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.requirements[0].title.as_deref(),
        Some("The name")
    );
}

#[test]
fn section_3_3_a_unit_with_no_heading_is_invalid() {
    let projection = document("<section class=\"requirement\" id=\"req-x\"><p>MUST.</p></section>");
    assert!(
        rules(&projection).contains(&"3.3"),
        "{:?}",
        projection.faults
    );
    assert_eq!(projection.model.requirements[0].title, None);
}

// --- Section 4: unit-specific fields --------------------------------------

#[test]
fn section_4_0_an_unmarked_paragraph_supplies_the_primary_field() {
    let projection = in_requirement(MINIMAL_SCENARIO);
    assert_eq!(
        projection.model.requirements[0].statements,
        vec!["The system MUST do the thing."]
    );
}

#[test]
fn section_4_0_a_data_field_naming_no_field_of_the_unit_is_invalid() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
         <p data-field=\"mitigation\">Not a task field.</p></section>",
    );
    assert_eq!(rules(&projection), vec!["4.0"]);
}

#[test]
fn section_4_0_every_unit_accepts_a_note() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
         <p data-field=\"note\">Context.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(projection.model.tasks[0].notes, vec!["Context."]);
}

#[test]
fn section_4_0_1_two_unmarked_paragraphs_in_a_single_valued_field_is_invalid() {
    let projection = document(
        "<section class=\"risk\" id=\"risk-x\"><h2>T</h2><p>One.</p><p>Two.</p>\
         <p data-field=\"mitigation\">Held down.</p></section>",
    );
    assert_eq!(rules(&projection), vec!["4.0.1"]);
    assert_eq!(projection.model.risks[0].statement, None);
}

#[test]
fn section_4_0_2_a_unit_with_no_primary_field_projects_an_unmarked_paragraph_as_a_note() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
         <p>Some prose.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(projection.model.tasks[0].notes, vec!["Some prose."]);
    assert_eq!(projection.model.tasks[0].reason, None);
}

#[test]
fn section_4_1_statements_merge_unmarked_and_marked_paragraphs_in_document_order() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>T</h2>\
         <p>First.</p><p data-field=\"statement\">Second.</p><p>Third.</p>\
         {MINIMAL_SCENARIO}</section>"
    ));
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.requirements[0].statements,
        vec!["First.", "Second.", "Third."]
    );
}

/// The document-schema scenario "The previous heading is a field, not a statement".
#[test]
fn section_4_1_1_the_previous_heading_is_a_field_not_a_statement() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\" data-change=\"modified\">\
         <h2>The new name</h2>\
         <p data-field=\"renamed-from\">The   old\n name</p>\
         <p>The system MUST hold.</p>{MINIMAL_SCENARIO}</section>"
    ));
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let requirement = &projection.model.requirements[0];
    assert_eq!(requirement.statements, vec!["The system MUST hold."]);
    assert_eq!(requirement.previous_heading(), Some("The old name"));
    assert!(requirement.notes.is_empty(), "{:?}", requirement.notes);

    let json = serde_json::to_value(requirement).expect("the requirement serializes");
    assert_eq!(json["renamedFrom"], "The old name", "{json}");
}

/// The document-schema scenario "A requirement with no rename".
#[test]
fn section_4_1_1_a_requirement_with_no_rename_projects_no_renamed_from() {
    let projection = in_requirement(MINIMAL_SCENARIO);
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let requirement = &projection.model.requirements[0];
    assert!(requirement.renamed_from.is_empty());
    assert_eq!(requirement.previous_heading(), None);

    let json = serde_json::to_value(requirement).expect("the requirement serializes");
    assert!(
        json.get("renamedFrom").is_none(),
        "an absent rename must not serialize: {json}"
    );
}

/// An empty `renamed-from` paragraph is kept where every other empty paragraph
/// is discarded, so the validator can report it rather than never seeing it.
#[test]
fn section_4_1_1_an_empty_rename_is_kept_and_projects_nothing() {
    let projection = in_requirement(&format!(
        "<p data-field=\"renamed-from\"> \n </p>{MINIMAL_SCENARIO}"
    ));
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let requirement = &projection.model.requirements[0];
    assert_eq!(requirement.renamed_from, vec![RenamedFrom::Empty]);
    assert_eq!(requirement.previous_heading(), None);
    let json = serde_json::to_value(requirement).expect("the requirement serializes");
    assert!(json.get("renamedFrom").is_none(), "{json}");
}

/// A second `renamed-from` paragraph projects nothing, and is kept for the
/// validator. The model's field holds the first one with text, so an empty
/// first paragraph does not hide the heading a second one states.
#[test]
fn section_4_1_1_the_field_holds_the_first_paragraph_with_text() {
    let projection = in_requirement(&format!(
        "<p data-field=\"renamed-from\"></p>\
         <p data-field=\"renamed-from\">The first name</p>\
         <p data-field=\"renamed-from\">The second name</p>{MINIMAL_SCENARIO}"
    ));
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let requirement = &projection.model.requirements[0];
    assert_eq!(
        requirement.renamed_from,
        vec![
            RenamedFrom::Empty,
            RenamedFrom::Heading("The first name".to_owned()),
            RenamedFrom::Heading("The second name".to_owned()),
        ]
    );
    assert_eq!(requirement.previous_heading(), Some("The first name"));
}

/// Section 4.0's exception: a rename outside a requirement is not a field the
/// unit lacks. It projects nowhere and is recorded for the validator's own rule.
#[test]
fn section_4_1_1_a_rename_outside_a_requirement_is_recorded_rather_than_faulted() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
         <p data-field=\"renamed-from\">An old task</p>\
         <p data-field=\"renamed-from\">An older task</p></section>\
         <section class=\"requirement\" id=\"req-x\"><h2>R</h2><p>It MUST hold.</p>\
         <article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>\
         <p data-field=\"renamed-from\">An old scenario</p>\
         <dl><dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd></dl>\
         </article></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.misplaced_renames,
        vec![
            MisplacedRename {
                unit: UnitKind::Task,
                unit_id: Some("task-x".to_owned()),
            },
            MisplacedRename {
                unit: UnitKind::Scenario,
                unit_id: Some("scenario-x".to_owned()),
            },
        ],
        "one entry per unit, in document order, and none for the requirement"
    );
    // The scenario's paragraph is its own, not the enclosing requirement's.
    assert!(projection.model.requirements[0].renamed_from.is_empty());
    assert!(projection.model.tasks[0].notes.is_empty());
}

/// Section 4.1.2: the title carries the identifier shown beside a heading, and
/// the heading name does not. A mapping fixture pins the title half.
#[test]
fn section_4_1_2_the_heading_name_leaves_out_the_identifier_shown_beside_it() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>The name <span class=\"hs-id\">req-x</span></h2>\
         <p>It MUST hold.</p>{MINIMAL_SCENARIO}</section>\
         <section class=\"requirement\" id=\"req-y\"><h2>The name<span class=\"hs-id\">req-y</span></h2>\
         <p>It MUST hold.</p>{}</section>\
         <section class=\"requirement\" id=\"req-z\"><h2>The\n  name</h2>\
         <p>It MUST hold.</p>{}</section>",
        MINIMAL_SCENARIO.replace("scenario-x", "scenario-y"),
        MINIMAL_SCENARIO.replace("scenario-x", "scenario-z"),
    ));
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let requirements = &projection.model.requirements;
    assert_eq!(requirements[0].title.as_deref(), Some("The name req-x"));
    assert_eq!(requirements[1].title.as_deref(), Some("The namereq-y"));
    for requirement in requirements {
        assert_eq!(
            requirement.heading_name().as_ref().map(HeadingName::as_str),
            Some("The name"),
            "{:?}",
            requirement.title
        );
    }
}

#[test]
fn section_4_2_a_scenario_body_projects_from_a_description_list() {
    let projection = in_requirement(
        "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3><dl>\
         <dt>Given</dt><dd>A precondition.</dd><dt>Given</dt><dd>Another.</dd>\
         <dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd></dl></article>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let scenario = only_scenario(&projection);
    assert_eq!(scenario.given, vec!["A precondition.", "Another."]);
    assert_eq!(scenario.when, vec!["It happens."]);
    assert_eq!(scenario.then, vec!["It works."]);
}

#[test]
fn section_4_2_a_label_is_matched_case_insensitively_and_without_one_trailing_colon() {
    let projection = in_requirement(
        "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3><dl>\
         <dt>WHEN:</dt><dd>It happens.</dd><dt> then </dt><dd>It works.</dd></dl></article>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let scenario = only_scenario(&projection);
    assert_eq!(scenario.when, vec!["It happens."]);
    assert_eq!(scenario.then, vec!["It works."]);
}

#[test]
fn section_4_2_a_label_outside_the_set_is_invalid() {
    let projection = in_requirement(
        "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3><dl>\
         <dt>Whenever</dt><dd>It happens.</dd></dl></article>",
    );
    assert!(
        rules(&projection).contains(&"4.2"),
        "{:?}",
        projection.faults
    );
    // An entry under an unrecognized label projects nowhere rather than into a
    // guessed field.
    let scenario = only_scenario(&projection);
    assert!(scenario.when.is_empty() && scenario.then.is_empty() && scenario.given.is_empty());
}

/// An entry under an unrecognized label has a `dt`, so reporting that it has
/// none would send an author looking for the wrong defect. One fault per fault.
#[test]
fn section_4_2_an_entry_under_an_unrecognized_label_is_not_also_an_orphaned_entry() {
    let projection = in_requirement(
        "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3><dl>\
         <dt>Whenever</dt><dd>An entry.</dd><dd>Another entry.</dd>\
         <dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article>",
    );
    assert_eq!(projection.faults.len(), 1, "{:#?}", projection.faults);
    assert_eq!(projection.faults[0].rule, "4.2");
    assert!(
        projection.faults[0].detail.contains("outside the set"),
        "{:?}",
        projection.faults[0]
    );
}

#[test]
fn section_4_2_a_dt_with_no_following_dd_is_invalid() {
    let projection = in_requirement(
        "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3><dl>\
         <dt>When</dt><dd>It happens.</dd><dt>Then</dt></dl></article>",
    );
    assert!(
        rules(&projection).contains(&"4.2"),
        "{:?}",
        projection.faults
    );
}

#[test]
fn section_4_2_a_dd_with_no_preceding_dt_is_invalid() {
    let projection = in_requirement(
        "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3><dl>\
         <dd>Orphaned.</dd><dt>When</dt><dd>It happens.</dd>\
         <dt>Then</dt><dd>It works.</dd></dl></article>",
    );
    assert!(
        rules(&projection).contains(&"4.2"),
        "{:?}",
        projection.faults
    );
}

#[test]
fn section_4_3_a_scenario_outside_a_requirement_is_invalid() {
    let projection = document(
        "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3><dl>\
         <dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd></dl></article>",
    );
    assert!(
        rules(&projection).contains(&"4.3"),
        "{:?}",
        projection.faults
    );
}

#[test]
fn section_4_3_a_requirement_nested_in_a_requirement_is_invalid() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-outer\"><h2>Outer</h2><p>MUST.</p>\
         <section class=\"requirement\" id=\"req-inner\"><h3>Inner</h3><p>MUST.</p>\
         </section>{MINIMAL_SCENARIO}</section>"
    ));
    assert!(
        rules(&projection).contains(&"4.3"),
        "{:?}",
        projection.faults
    );
    // The outer requirement still projects, so a second fault elsewhere is
    // still reported.
    assert_eq!(projection.model.requirements.len(), 1);
    assert_eq!(
        projection.model.requirements[0].id.as_deref(),
        Some("req-outer")
    );
}

#[test]
fn section_4_4_a_decision_projects_a_statement_and_repeatable_rationale() {
    let projection = document(
        "<section class=\"decision\" id=\"decision-x\" data-status=\"superseded\" \
         data-superseded-by=\"decision-y\"><h2>T</h2><p>The choice.</p>\
         <p data-field=\"rationale\">Because.</p>\
         <p data-field=\"rationale\">And also.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let decision = &projection.model.decisions[0];
    assert_eq!(decision.statement.as_deref(), Some("The choice."));
    assert_eq!(decision.rationale, vec!["Because.", "And also."]);
    assert_eq!(decision.status.as_deref(), Some("superseded"));
    assert_eq!(decision.superseded_by.as_deref(), Some("decision-y"));
}

#[test]
fn section_4_5_a_task_projects_a_reason_only_from_a_marked_paragraph() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"dropped\"><h2>T</h2>\
         <p data-field=\"reason\">No requirement asks for it.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.tasks[0].reason.as_deref(),
        Some("No requirement asks for it.")
    );
}

#[test]
fn section_4_5_a_deviation_projects_into_its_field_and_not_into_the_notes() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"done\"><h2>T</h2>\
         <p data-field=\"note\">Context.</p>\
         <p data-field=\"deviation\">The list was \n  never hand-written.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let task = &projection.model.tasks[0];
    assert_eq!(
        task.deviation.as_deref(),
        Some("The list was never hand-written."),
        "a deviation is normalized text in its own field"
    );
    assert_eq!(task.notes, vec!["Context."], "a deviation is not a note");
    assert_eq!(task.reason, None, "a deviation is not a reason");
}

/// The absence is the reading every task done as written relies on, so it is stated rather
/// than left to the field's default.
#[test]
fn section_4_5_a_task_stating_no_deviation_projects_none() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"done\"><h2>T</h2>\
         <p data-field=\"note\">Done as its words say.</p>\
         <p data-field=\"deviation\">  </p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.tasks[0].deviation, None,
        "a task with no deviation, or only an empty one, reads as done as written"
    );
}

#[test]
fn section_4_5_1_a_second_deviation_is_invalid_and_the_first_is_kept() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"done\"><h2>T</h2>\
         <p data-field=\"deviation\">The first account.</p>\
         <p data-field=\"note\">Between them.</p>\
         <p data-field=\"deviation\">The second account.</p></section>",
    );
    assert_eq!(rules(&projection), vec!["4.5.1"]);
    assert_eq!(projection.faults[0].unit_id.as_deref(), Some("task-x"));
    // The first is kept because the counter reads unvalidated projections: a task whose author
    // stated two deviations was done differently, and dropping the field would count it as
    // done as written.
    assert_eq!(
        projection.model.tasks[0].deviation.as_deref(),
        Some("The first account.")
    );
    assert_eq!(projection.model.tasks[0].notes, vec!["Between them."]);
}

/// Section 4.0: the field belongs to a task, so every other unit refuses it as a field it
/// does not have, and the fault names the unit.
#[test]
fn section_4_0_a_deviation_outside_a_task_names_no_field_of_its_unit() {
    const DEVIATION: &str = "<p data-field=\"deviation\">Done another way.</p>";
    let cases = [
        (
            "req-x",
            format!(
                "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
                 <p>The system MUST do it.</p>{DEVIATION}{MINIMAL_SCENARIO}</section>"
            ),
        ),
        (
            "scenario-x",
            format!(
                "<section class=\"requirement\" id=\"req-x\"><h2>R</h2>\
                 <p>The system MUST do it.</p>\
                 <article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>{DEVIATION}\
                 <dl><dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd></dl>\
                 </article></section>"
            ),
        ),
        (
            "decision-x",
            format!(
                "<section class=\"decision\" id=\"decision-x\" data-status=\"accepted\">\
                 <h2>D</h2><p>Chose.</p><p data-field=\"rationale\">Because.</p>{DEVIATION}\
                 </section>"
            ),
        ),
        (
            "evidence-x",
            format!(
                "<section class=\"evidence\" id=\"evidence-x\"><h2>E</h2><p>Measured.</p>\
                 {DEVIATION}</section>"
            ),
        ),
        (
            "risk-x",
            format!(
                "<section class=\"risk\" id=\"risk-x\"><h2>K</h2><p>Fails.</p>\
                 <p data-field=\"mitigation\">Held.</p>{DEVIATION}</section>"
            ),
        ),
        (
            "verify-x",
            format!(
                "<section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\">\
                 <h2>V</h2>{DEVIATION}</section>"
            ),
        ),
    ];
    for (id, body) in cases {
        let projection = document(&body);
        assert_eq!(
            rules(&projection),
            vec!["4.0"],
            "{id}: {:?}",
            projection.faults
        );
        assert_eq!(
            projection.faults[0].unit_id.as_deref(),
            Some(id),
            "{:?}",
            projection.faults
        );
    }
}

#[test]
fn section_4_6_evidence_summary_merges_unmarked_and_marked_paragraphs() {
    let projection = document(
        "<section class=\"evidence\" id=\"evidence-x\"><h2>T</h2>\
         <p>What was measured.</p><p data-field=\"summary\">And more.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.evidence[0].summary,
        vec!["What was measured.", "And more."]
    );
}

#[test]
fn section_4_7_a_risk_projects_a_statement_and_a_mitigation() {
    let projection = document(
        "<section class=\"risk\" id=\"risk-x\"><h2>T</h2><p>How it fails.</p>\
         <p data-field=\"mitigation\">What holds it down.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let risk = &projection.model.risks[0];
    assert_eq!(risk.statement.as_deref(), Some("How it fails."));
    assert_eq!(risk.mitigation.as_deref(), Some("What holds it down."));
}

/// The one risk of `projection`, as the model's JSON carries it.
///
/// Section 4.7.1 is about what the model carries, and the model is the JSON: a `response`
/// the struct held and serialization skipped would leave a consumer to restate the
/// default, which is what the field being always present exists to prevent.
fn only_risk_as_json(projection: &Projection) -> serde_json::Value {
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(projection.model.risks.len(), 1, "one risk projected");
    serde_json::to_value(&projection.model.risks[0]).expect("a risk serializes")
}

#[test]
fn section_4_7_1_a_risk_that_declares_no_response_projects_mitigated() {
    let risk = only_risk_as_json(&document(
        "<section class=\"risk\" id=\"risk-x\"><h2>T</h2><p>How it fails.</p>\
         <p data-field=\"mitigation\">What holds it down.</p></section>",
    ));
    assert_eq!(risk["response"], "mitigated", "{risk}");
    assert_eq!(risk["mitigation"], "What holds it down.", "{risk}");
}

#[test]
fn section_4_7_1_an_accepted_risk_with_no_mitigation_projects_none() {
    let risk = only_risk_as_json(&document(
        "<section class=\"risk\" id=\"risk-x\" data-response=\"accepted\"><h2>T</h2>\
         <p>How it fails, and why the change bears it.</p></section>",
    ));
    assert_eq!(risk["response"], "accepted", "{risk}");
    assert!(risk.get("mitigation").is_none(), "{risk}");
    assert_eq!(
        risk["statement"], "How it fails, and why the change bears it.",
        "{risk}"
    );
}

#[test]
fn section_4_7_1_an_accepted_risk_keeps_a_mitigation_it_states() {
    let risk = only_risk_as_json(&document(
        "<section class=\"risk\" id=\"risk-x\" data-response=\"accepted\"><h2>T</h2>\
         <p>How it fails.</p><p data-field=\"mitigation\">The part something holds down.</p>\
         </section>",
    ));
    assert_eq!(risk["response"], "accepted", "{risk}");
    assert_eq!(
        risk["mitigation"], "The part something holds down.",
        "{risk}"
    );
}

/// Section 2.3 applies to the response as to every attribute: trimmed, and otherwise as
/// written, so a value outside the enumeration survives projection and the validator
/// names the permitted values.
#[test]
fn section_4_7_1_a_response_is_trimmed_and_an_unrecognized_one_survives() {
    let trimmed = only_risk_as_json(&document(
        "<section class=\"risk\" id=\"risk-x\" data-response=\" accepted \"><h2>T</h2>\
         <p>How it fails.</p></section>",
    ));
    assert_eq!(trimmed["response"], "accepted", "{trimmed}");

    let unrecognized = only_risk_as_json(&document(
        "<section class=\"risk\" id=\"risk-x\" data-response=\"partial\"><h2>T</h2>\
         <p>How it fails.</p><p data-field=\"mitigation\">Partly held.</p></section>",
    ));
    assert_eq!(unrecognized["response"], "partial", "{unrecognized}");
}

/// Section 2's row reads `data-response` (risk only), as `data-state` (task only): on any
/// other unit the attribute projects nowhere, so the decision means what it would without it.
#[test]
fn section_4_7_1_a_response_on_another_unit_projects_nowhere() {
    let decision = |attribute: &str| {
        let projection = document(&format!(
            "<section class=\"decision\" id=\"decision-x\" data-status=\"accepted\"{attribute}>\
             <h2>D</h2><p>Chose.</p><p data-field=\"rationale\">Because.</p></section>"
        ));
        assert!(projection.is_clean(), "{:?}", projection.faults);
        assert!(projection.model.risks.is_empty(), "no risk is projected");
        serde_json::to_value(&projection.model).expect("the model serializes")
    };
    assert_eq!(
        decision(" data-response=\"accepted\""),
        decision(""),
        "a response on a decision changed the model"
    );
}

#[test]
fn section_4_8_a_verification_projects_a_method_when_one_exists() {
    let with_method = document(
        "<section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
         data-refs=\"req-x\"><h2>T</h2>\
         <p data-field=\"method\">Ran the suite.</p></section>",
    );
    assert!(with_method.is_clean(), "{:?}", with_method.faults);
    assert_eq!(
        with_method.model.verifications[0].method.as_deref(),
        Some("Ran the suite.")
    );

    let without = document(
        "<section class=\"verification\" id=\"verify-x\" data-outcome=\"not-run\" \
         data-refs=\"req-x\"><h2>T</h2></section>",
    );
    assert_eq!(without.model.verifications[0].method, None);
}

// --- Section 5: references ------------------------------------------------

#[test]
fn section_5_2_a_fragment_link_projects_as_both_a_reference_and_a_link() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
         <p data-field=\"note\">See <a href=\"#req-y\">the requirement</a>.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let reference = &projection.model.tasks[0].references[0];
    assert_eq!(reference.target, "req-y");
    assert_eq!(reference.origin, ReferenceOrigin::Link);
    assert_eq!(projection.model.links[0].target, "req-y");
    assert_eq!(projection.model.links[0].text, "the requirement");
}

#[test]
fn section_5_3_a_fragment_outside_the_identifier_grammar_is_a_link_only() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
         <p data-field=\"note\">See <a href=\"#section-overview\">the overview</a> \
         and <a href=\"#figure-pipeline\">the figure</a>.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert!(
        projection.model.tasks[0].references.is_empty(),
        "{:?}",
        projection.model.tasks[0].references
    );
    let targets: Vec<&str> = projection
        .model
        .links
        .iter()
        .map(|l| l.target.as_str())
        .collect();
    assert_eq!(targets, vec!["section-overview", "figure-pipeline"]);
}

#[test]
fn section_5_4_an_absolute_href_projects_into_neither_array() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
         <p data-field=\"note\">See <a href=\"https://example.test/\">a citation</a> \
         and <a href=\"//example.test/\">another</a>.</p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert!(projection.model.tasks[0].references.is_empty());
    assert!(projection.model.links.is_empty());
}

#[test]
fn section_5_5_attribute_references_precede_link_references() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\" \
         data-refs=\"req-a req-b\"><h2>T</h2>\
         <p data-field=\"note\"><a href=\"#req-c\">c</a></p></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    let observed: Vec<(&str, ReferenceOrigin)> = projection.model.tasks[0]
        .references
        .iter()
        .map(|r| (r.target.as_str(), r.origin))
        .collect();
    assert_eq!(
        observed,
        vec![
            ("req-a", ReferenceOrigin::Attribute),
            ("req-b", ReferenceOrigin::Attribute),
            ("req-c", ReferenceOrigin::Link),
        ]
    );
}

// --- Section 6: whitespace and text ---------------------------------------

#[test]
fn section_6_1_field_text_is_whitespace_normalized() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>A\n   long\ttitle</h2>\n\
         <p>The system\n  MUST   do\nthe thing.</p>{MINIMAL_SCENARIO}</section>"
    ));
    assert_eq!(
        projection.model.requirements[0].title.as_deref(),
        Some("A long title")
    );
    assert_eq!(
        projection.model.requirements[0].statements,
        vec!["The system MUST do the thing."]
    );
}

#[test]
fn section_6_2_a_figure_source_is_preserved_byte_for_byte() {
    let projection = document(
        "<figure id=\"figure-x\"><figcaption>A diagram.</figcaption>\
         <pre class=\"mermaid\">graph LR\n  A --&gt; B\n    C</pre></figure>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.figures[0].source.as_deref(),
        Some("graph LR\n  A --> B\n    C")
    );
}

#[test]
fn section_6_3_a_character_reference_is_resolved_before_either_rule_applies() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>T</h2>\
         <p>Rock &amp; roll MUST work.</p>{MINIMAL_SCENARIO}</section>"
    ));
    assert_eq!(
        projection.model.requirements[0].statements,
        vec!["Rock & roll MUST work."]
    );
}

#[test]
fn section_6_4_text_split_by_a_comment_projects_as_the_concatenation_of_its_parts() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>T</h2>\
         <p>The system <!-- an aside -->MUST work.</p>{MINIMAL_SCENARIO}</section>"
    ));
    assert_eq!(
        projection.model.requirements[0].statements,
        vec!["The system MUST work."]
    );
}

// --- Section 7: figures ---------------------------------------------------

#[test]
fn section_7_3_the_first_pre_decides_the_figure_kind() {
    let cases: &[(&str, FigureKind, bool)] = &[
        (
            "<pre class=\"mermaid\">graph LR</pre>",
            FigureKind::Mermaid,
            true,
        ),
        (
            "<pre class=\"vega-lite\">{}</pre>",
            FigureKind::VegaLite,
            true,
        ),
        ("<pre><code>let x = 1;</code></pre>", FigureKind::Code, true),
        (
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mn>1</mn></math>",
            FigureKind::Math,
            false,
        ),
        ("<p>Just prose.</p>", FigureKind::Prose, false),
    ];
    for (content, expected, has_source) in cases {
        let projection = document(&format!(
            "<figure id=\"figure-x\"><figcaption>A caption.</figcaption>{content}</figure>"
        ));
        assert!(projection.is_clean(), "{content}: {:?}", projection.faults);
        let figure = &projection.model.figures[0];
        assert_eq!(figure.kind, *expected, "for {content}");
        assert_eq!(
            figure.source.is_some(),
            *has_source,
            "source presence for {content}"
        );
    }
}

#[test]
fn section_7_2_a_figure_with_no_figcaption_is_invalid() {
    let projection = document("<figure id=\"figure-x\"><pre>x</pre></figure>");
    assert_eq!(rules(&projection), vec!["7.2"]);
}

#[test]
fn section_7_1_a_figure_creates_no_unit_and_takes_an_optional_identifier() {
    let projection = document("<figure><figcaption>A caption.</figcaption><p>Prose.</p></figure>");
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(projection.model.figures[0].id, None);
    assert!(projection.model.requirements.is_empty());
}

#[test]
fn section_7_4_a_details_wrapper_around_the_source_is_presentational() {
    let bare = document(
        "<figure id=\"figure-x\"><figcaption>A diagram.</figcaption>\
         <pre class=\"mermaid\">graph LR</pre></figure>",
    );
    let wrapped = document(
        "<figure id=\"figure-x\"><figcaption>A diagram.</figcaption>\
         <details><summary>Diagram source</summary>\
         <pre class=\"mermaid\">graph LR</pre></details></figure>",
    );
    assert_eq!(bare.model.figures, wrapped.model.figures);
}

// --- Section 8: presentational markup -------------------------------------

#[test]
fn section_8_1_a_field_rule_reaches_through_a_presentational_wrapper() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>T</h2>\
         <div class=\"callout\"><p>The system MUST work.</p></div>\
         {MINIMAL_SCENARIO}</section>"
    ));
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.requirements[0].statements,
        vec!["The system MUST work."]
    );
}

#[test]
fn section_8_3_an_inline_element_contributes_its_text_and_not_its_markup() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>T</h2>\
         <p>The <strong>system</strong> MUST call <code>work()</code>.</p>\
         {MINIMAL_SCENARIO}</section>"
    ));
    assert_eq!(
        projection.model.requirements[0].statements,
        vec!["The system MUST call work()."]
    );
}

#[test]
fn section_8_5_a_script_or_style_contributes_no_text_to_any_field() {
    let projection = document(&format!(
        "<section class=\"requirement\" id=\"req-x\"><h2>T</h2>\
         <p>The system MUST work.<script>var hidden = 1;</script>\
         <style>p {{ color: red; }}</style><noscript>Fallback.</noscript></p>\
         {MINIMAL_SCENARIO}</section>"
    ));
    assert_eq!(
        projection.model.requirements[0].statements,
        vec!["The system MUST work."]
    );
}

// --- Section 9: document metadata -----------------------------------------

#[test]
fn section_9_1_the_subject_projects_from_the_head_title() {
    let projection = document("<p>Nothing.</p>");
    assert_eq!(projection.model.subject.as_deref(), Some("Fixture"));
}

#[test]
fn section_9_1_1_the_summary_projects_from_the_header_paragraphs_in_order() {
    let projection = document(
        "<header><h1>A subject</h1><p>The first.</p><p>The second.</p></header>\
         <main><p>Body prose.</p></main>",
    );
    assert_eq!(projection.model.summary, vec!["The first.", "The second."]);
}

#[test]
fn section_9_1_1_a_document_with_no_header_projects_an_empty_summary_and_stays_valid() {
    let projection = document("<main><p>Body prose.</p></main>");
    assert!(projection.model.summary.is_empty());
    assert!(projection.is_clean(), "{:?}", projection.faults);
}

#[test]
fn section_9_2_and_9_3_the_version_and_kind_project_from_meta_elements() {
    let projection = document("<p>Nothing.</p>");
    assert_eq!(projection.model.schema_version.as_deref(), Some("0.1.0"));
    assert_eq!(projection.model.kind.as_deref(), Some("specification"));
}

/// Section 9.3: a kind outside the enumeration still projects, so the validator
/// reports it against a schema location.
///
/// The specification calls such a kind invalid, and section 10.2 says what that
/// means: the *validator* reports it. This is not a projection fault, for the same
/// reason a wrongly cased task state is not — the projection produces the model
/// and does not judge it (section 10.1). A projection that rejected the value here
/// would lose the diagnostic.
#[test]
fn section_9_3_a_kind_outside_the_enumeration_survives_projection() {
    let html = "<!doctype html>\n<html lang=\"en\">\n<head>\
                <meta name=\"htmlspec-version\" content=\"0.1.0\">\
                <meta name=\"htmlspec-kind\" content=\"rumination\">\
                <title>T</title></head>\n<body><p>Nothing.</p></body></html>\n";
    let projection = project(&parse_document(html).document);
    assert_eq!(projection.model.kind.as_deref(), Some("rumination"));
    assert!(projection.is_clean(), "{:?}", projection.faults);
}

#[test]
fn section_9_2_and_9_3_an_absent_version_or_kind_projects_as_absent() {
    // Absent rather than defaulted: the validator reports the missing property,
    // because a document that does not declare its contract version cannot be
    // checked against one, and one that declares no kind has nothing deciding
    // which units belong in it.
    let html = "<!doctype html>\n<html lang=\"en\">\n<head><title>T</title></head>\n\
                <body><p>Nothing.</p></body></html>\n";
    let outcome = parse_document(html);
    let projection = project(&outcome.document);
    assert_eq!(projection.model.schema_version, None);
    assert_eq!(projection.model.kind, None);
}

#[test]
fn section_9_4_arrays_are_in_depth_first_document_order() {
    let projection = document(
        "<section class=\"task\" id=\"task-a\" data-state=\"todo\"><h2>A</h2></section>\
         <section class=\"task\" id=\"task-b\" data-state=\"todo\"><h2>B</h2></section>\
         <section class=\"task\" id=\"task-c\" data-state=\"todo\"><h2>C</h2></section>",
    );
    let ids: Vec<&str> = projection
        .model
        .tasks
        .iter()
        .filter_map(|t| t.id.as_deref())
        .collect();
    assert_eq!(ids, vec!["task-a", "task-b", "task-c"]);
}

// --- Section 10: what the projection does not decide ----------------------

#[test]
fn section_10_1_a_wrong_prefix_survives_projection_so_the_validator_reports_it() {
    let projection = document(
        "<section class=\"task\" id=\"risk-wrong-prefix\" data-state=\"todo\">\
         <h2>T</h2></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.tasks[0].id.as_deref(),
        Some("risk-wrong-prefix")
    );
}

#[test]
fn section_10_1_an_unresolved_reference_survives_projection() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\" \
         data-refs=\"req-does-not-exist\"><h2>T</h2></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(
        projection.model.tasks[0].references[0].target,
        "req-does-not-exist"
    );
}

#[test]
fn section_10_1_an_unrecognized_change_state_survives_projection() {
    let projection = document(
        "<section class=\"task\" id=\"task-x\" data-state=\"todo\" \
         data-change=\"pending\"><h2>T</h2></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);
    assert_eq!(projection.model.tasks[0].change, "pending");
}

// --- The model reaches rule code -----------------------------------------

#[test]
fn every_unit_reaches_rule_code_through_one_view() {
    let projection = document(
        "<section class=\"requirement\" id=\"req-x\"><h2>R</h2><p>MUST.</p>\
         <article class=\"scenario\" id=\"scenario-x\"><h3>S</h3><dl>\
         <dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></section>\
         <section class=\"decision\" id=\"decision-x\" data-status=\"accepted\"><h2>D</h2>\
         <p>Chose.</p><p data-field=\"rationale\">Because.</p></section>\
         <section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2></section>\
         <section class=\"evidence\" id=\"evidence-x\"><h2>E</h2><p>Measured.</p></section>\
         <section class=\"risk\" id=\"risk-x\"><h2>K</h2><p>Fails.</p>\
         <p data-field=\"mitigation\">Held.</p></section>\
         <section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
         data-refs=\"req-x\"><h2>V</h2></section>",
    );
    assert!(projection.is_clean(), "{:?}", projection.faults);

    let units = projection.model.units();
    let kinds: Vec<UnitKind> = units.iter().map(|u| u.kind).collect();
    assert_eq!(
        kinds,
        vec![
            UnitKind::Requirement,
            UnitKind::Scenario,
            UnitKind::Decision,
            UnitKind::Task,
            UnitKind::Evidence,
            UnitKind::Risk,
            UnitKind::Verification,
        ],
        "a scenario must be adjacent to the requirement that contains it"
    );
    assert!(
        units
            .iter()
            .all(|u| u.id.is_some() && u.change == "unchanged"),
        "every unit must reach rule code with its identifier and change state"
    );

    let references = projection.model.references();
    assert_eq!(references.len(), 1);
    assert_eq!(references[0].0.kind, UnitKind::Verification);
    assert_eq!(references[0].1.target, "req-x");
}

// --- The projected shape is the shape the schema validates ----------------

#[test]
fn the_projected_json_uses_the_schemas_property_names() {
    let projection = document(
        "<section class=\"decision\" id=\"decision-x\" data-status=\"superseded\" \
         data-superseded-by=\"decision-y\"><h2>T</h2><p>Chose.</p>\
         <p data-field=\"rationale\">Because.</p></section>",
    );
    let json = serde_json::to_value(&projection.model).expect("the model serializes");
    assert_eq!(json["schemaVersion"], "0.1.0");
    let decision = &json["decisions"][0];
    assert_eq!(decision["unit"], "decision");
    assert_eq!(decision["supersededBy"], "decision-y");
    assert!(
        decision.get("superseded_by").is_none(),
        "a snake_case property would not validate against the schema"
    );
}

// --- The hand-authored documents still project ---------------------------

#[test]
fn every_dogfood_document_projects_without_a_fault() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dogfood");
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .expect("the dogfood directory is readable")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    paths.sort();
    assert_eq!(
        paths.len(),
        6,
        "expected the six hand-authored documents: a proposal, two tasks documents, \
         two specifications, and a design"
    );

    for path in paths {
        let html = std::fs::read_to_string(&path).expect("readable");
        let outcome = parse_document(&html);
        assert!(
            outcome.is_recovery_free(),
            "{path:?} recovered: {:?}",
            outcome.recovered
        );
        let projection = project(&outcome.document);
        assert!(
            projection.is_clean(),
            "{path:?} projected with faults: {:#?}",
            projection.faults
        );
        assert_eq!(projection.model.schema_version.as_deref(), Some("0.1.0"));
        assert!(projection.model.subject.is_some());
        assert!(
            projection.model.kind.is_some(),
            "{path:?} declares no artifact kind"
        );
    }
}
