//! Task 5.7: the formatter introduces no diff churn.
//!
//! The property: editing, adding, or removing a unit must not cause the formatter
//! to reflow or reindent any *other* unit. Every line outside the affected unit is
//! byte-identical before and after.
//!
//! This is narrower than diff minimality, and the distinction is the point. The
//! formatter has no opinion about where a paragraph's lines break, so an edit
//! cannot make it rewrite the surrounding paragraph. It does *not* follow that the
//! Git diff is small: a paragraph held on one long line diffs as a whole line when
//! one sentence changes, and that cost belongs to the line structure rather than to
//! the formatter. Task 5.8 measures that separately.
//!
//! Each test works on already-canonical documents, because that is the situation
//! the property is about: a contributor edits a formatted document, runs the
//! formatter, and reviews the diff. Comparing two unformatted documents would
//! measure the initial canonicalization instead.

/// A canonical document built from the given unit bodies.
///
/// Each entry is `(id, title, statement)`. The document is formatted before it is
/// returned, so every comparison below is between two canonical files.
fn document(units: &[(&str, &str, &str)]) -> String {
    let mut body = String::new();
    for (id, title, statement) in units {
        body.push_str(&format!(
            "<section class=\"requirement\" id=\"{id}\"><h2>{title}</h2><p>{statement}</p>\
             <article class=\"scenario\" id=\"scenario-{id}\"><h3>Works</h3>\
             <dl><dt>When</dt><dd>it runs</dd><dt>Then</dt><dd>it works</dd></dl>\
             </article></section>"
        ));
    }
    let source = format!(
        "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>Churn</title></head><body>{body}</body></html>\n"
    );
    htmlspec_format::format(&source).expect("the sample formats")
}

/// The lines of `text` that belong to the unit carrying `id`, and those that do
/// not, as `(inside, outside)`.
///
/// A unit's extent is found by its opening line and the matching closing line at
/// the same indentation, which the canonical form makes unambiguous: the formatter
/// writes every block's open and close tag on its own line at its own depth.
fn split_around(text: &str, id: &str) -> (Vec<String>, Vec<String>) {
    let needle = format!(r#"id="{id}""#);
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.contains(&needle))
        .unwrap_or_else(|| panic!("no line declares {id}:\n{text}"));
    let indent = lines[start].len() - lines[start].trim_start().len();
    let close = format!("{}</section>", " ".repeat(indent));
    let end = lines
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, line)| **line == close)
        .map(|(at, _)| at)
        .unwrap_or_else(|| panic!("the unit {id} has no closing line:\n{text}"));

    let mut inside = Vec::new();
    let mut outside = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if at >= start && at <= end {
            inside.push((*line).to_owned());
        } else {
            outside.push((*line).to_owned());
        }
    }
    (inside, outside)
}

/// The specs' scenario: revising one requirement's prose changes no line outside
/// that requirement.
#[test]
fn editing_one_unit_leaves_every_other_line_untouched() {
    let before = document(&[
        ("req-1", "First", "The system SHALL do the first thing."),
        ("req-2", "Second", "The system SHALL do the second thing."),
        ("req-3", "Third", "The system SHALL do the third thing."),
    ]);
    let after = document(&[
        ("req-1", "First", "The system SHALL do the first thing."),
        (
            "req-2",
            "Second",
            "The system SHALL do the second thing, revised.",
        ),
        ("req-3", "Third", "The system SHALL do the third thing."),
    ]);

    let (inside_before, outside_before) = split_around(&before, "req-2");
    let (inside_after, outside_after) = split_around(&after, "req-2");

    assert_eq!(
        outside_before, outside_after,
        "the formatter changed a line outside the edited unit"
    );
    assert_ne!(
        inside_before, inside_after,
        "the edit did not reach the unit it was made in, so this test proves nothing"
    );
}

/// The specs' second scenario: inserting a unit between two existing ones does not
/// rewrite the surrounding units.
#[test]
fn inserting_a_unit_between_two_others_rewrites_neither() {
    let before = document(&[
        ("req-1", "First", "The system SHALL do the first thing."),
        ("req-3", "Third", "The system SHALL do the third thing."),
    ]);
    let after = document(&[
        ("req-1", "First", "The system SHALL do the first thing."),
        ("req-2", "Second", "The system SHALL do the second thing."),
        ("req-3", "Third", "The system SHALL do the third thing."),
    ]);

    // Neither surrounding unit's lines changed: the diff is the insertion alone.
    for id in ["req-1", "req-3"] {
        let (kept_before, _) = split_around(&before, id);
        let (kept_after, _) = split_around(&after, id);
        assert_eq!(kept_before, kept_after, "inserting a unit rewrote {id}");
    }

    // And the insertion really is the only difference, line for line.
    let added: Vec<&str> = after
        .lines()
        .filter(|line| !before.lines().any(|kept| kept == *line))
        .collect();
    assert!(
        added.iter().all(|line| line.contains("req-2")
            || line.contains("Second")
            || line.contains("second thing")
            || line.trim().starts_with("<article")
            || line.trim().starts_with("<h3")
            || line.trim().starts_with("<dl")
            || line.trim().starts_with("<dt")
            || line.trim().starts_with("<dd")
            || line.trim() == "</dl>"
            || line.trim() == "</article>"
            || line.trim() == "</section>"),
        "a line unrelated to the inserted unit changed: {added:#?}"
    );
}

/// Removing a unit leaves the rest alone, which is the third case the requirement
/// names and the one an archive operation performs.
#[test]
fn removing_a_unit_leaves_every_other_line_untouched() {
    let before = document(&[
        ("req-1", "First", "The system SHALL do the first thing."),
        ("req-2", "Second", "The system SHALL do the second thing."),
        ("req-3", "Third", "The system SHALL do the third thing."),
    ]);
    let after = document(&[
        ("req-1", "First", "The system SHALL do the first thing."),
        ("req-3", "Third", "The system SHALL do the third thing."),
    ]);

    for id in ["req-1", "req-3"] {
        let (kept_before, _) = split_around(&before, id);
        let (kept_after, _) = split_around(&after, id);
        assert_eq!(kept_before, kept_after, "removing a unit rewrote {id}");
    }
    assert!(!after.contains("req-2"), "the unit was not removed");
}

/// The whole-line change the specs explicitly permit: a paragraph held on one long
/// line diffs as that whole line, and the formatter is still conformant, because it
/// did not touch anything else.
#[test]
fn a_whole_line_change_inside_a_long_paragraph_is_not_churn() {
    let long = "The system SHALL do the thing. It SHALL also do the second thing. \
                And the third. And the fourth, at some length, on one line.";
    let edited = "The system SHALL do the thing, revised. It SHALL also do the second thing. \
                  And the third. And the fourth, at some length, on one line.";

    let before = document(&[
        ("req-1", "First", long),
        ("req-2", "Second", "The system SHALL do the second thing."),
    ]);
    let after = document(&[
        ("req-1", "First", edited),
        ("req-2", "Second", "The system SHALL do the second thing."),
    ]);

    let (_, outside_before) = split_around(&before, "req-1");
    let (_, outside_after) = split_around(&after, "req-1");
    assert_eq!(
        outside_before, outside_after,
        "the formatter reflowed content outside the edited unit"
    );

    // Exactly one line differs, and it is the paragraph. That the whole line
    // changed is the line structure's cost, not the formatter's.
    let changed: Vec<&str> = after
        .lines()
        .filter(|line| !before.lines().any(|kept| kept == *line))
        .collect();
    assert_eq!(
        changed.len(),
        1,
        "more than the paragraph changed: {changed:#?}"
    );
    assert!(changed[0].contains("revised"), "{changed:#?}");
}

/// A unit's own edit must not shift the indentation of the units after it, which
/// is the churn a formatter that re-derived depth from content would produce.
#[test]
fn an_edit_does_not_shift_the_indentation_of_later_units() {
    let before = document(&[
        ("req-1", "First", "The system SHALL do the first thing."),
        ("req-2", "Second", "The system SHALL do the second thing."),
    ]);
    // The edit adds a second statement, making the first unit taller.
    let taller = htmlspec_format::format(&before.replace(
        "<p>The system SHALL do the first thing.</p>",
        "<p>The system SHALL do the first thing.</p>\
             <p>The system SHALL also do another thing.</p>",
    ))
    .expect("the edited sample formats");

    let (kept_before, _) = split_around(&before, "req-2");
    let (kept_after, _) = split_around(&taller, "req-2");
    assert_eq!(
        kept_before, kept_after,
        "a taller first unit reindented the second"
    );
}
