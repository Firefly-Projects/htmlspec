//! Measures diff locality, the property task 5.8 separates from formatter churn.
//!
//! Two different questions, and conflating them is what this exists to prevent.
//!
//! *Formatter churn* asks whether the formatter rewrites lines the edit did not
//! touch. `tests/no_churn.rs` asserts it does not, and that is a conformance
//! requirement.
//!
//! *Diff locality* asks how many lines a Git diff actually shows for one content
//! edit. It is a consequence of the document's line structure, not of the
//! formatter, and it is measured rather than asserted: there is no threshold a
//! document must clear, because line structure is the author's choice.
//!
//! Run with `cargo run -p htmlspec-format --example diff_locality`.

use htmlspec_format::format as canonicalize;

/// One measured edit.
struct Measurement {
    /// What was edited, and how the paragraph was laid out.
    label: &'static str,
    /// How many lines the paragraph occupies.
    paragraph_lines: usize,
    /// Lines the diff removes.
    removed: usize,
    /// Lines the diff adds.
    added: usize,
    /// Bytes across every changed line, added and removed together.
    ///
    /// This is the measure that separates the two layouts. A changed-line *count*
    /// treats a 40-byte line and a 160-byte line alike, and a reviewer does not:
    /// the whole of a changed line is what they must re-read to find the edit.
    changed_bytes: usize,
    /// Bytes the edit itself accounts for, being the shortest run that differs.
    ///
    /// The ratio of `changed_bytes` to this is how much more a reviewer reads than
    /// the edit strictly required.
    edit_bytes: usize,
    /// Lines the document holds in total.
    total: usize,
}

fn main() {
    let measurements = vec![
        one_line_paragraph(),
        multi_line_paragraph(),
        edit_near_the_end_of_a_long_line(),
        insert_a_unit(),
        change_a_change_state(),
        edit_inside_a_preserved_block(),
    ];

    println!(
        "| Edit | Paragraph lines | Removed | Added | Changed bytes | Edit bytes | Read ratio | \
         Total lines |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- | --- |");
    for m in &measurements {
        let ratio = if m.edit_bytes == 0 {
            0.0
        } else {
            m.changed_bytes as f64 / m.edit_bytes as f64
        };
        println!(
            "| {} | {} | {} | {} | {} | {} | {ratio:.1}× | {} |",
            m.label, m.paragraph_lines, m.removed, m.added, m.changed_bytes, m.edit_bytes, m.total
        );
    }
}

/// A document holding one requirement whose statement is `statement`.
fn document(statement: &str) -> String {
    let source = format!(
        "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>Locality</title></head><body>\
         <section class=\"requirement\" id=\"req-1\"><h2>First</h2><p>{statement}</p>\
         <article class=\"scenario\" id=\"scenario-1\"><h3>Works</h3>\
         <dl><dt>When</dt><dd>it runs</dd><dt>Then</dt><dd>it works</dd></dl>\
         </article></section>\
         <section class=\"requirement\" id=\"req-2\"><h2>Second</h2>\
         <p>The system SHALL do the second thing.</p>\
         <article class=\"scenario\" id=\"scenario-2\"><h3>Works</h3>\
         <dl><dt>When</dt><dd>it runs</dd><dt>Then</dt><dd>it works</dd></dl>\
         </article></section>\
         </body></html>\n"
    );
    canonicalize(&source).expect("the sample formats")
}

/// Counts the lines a diff between `before` and `after` would show.
///
/// A line-set difference rather than a real LCS diff. That understates nothing
/// that matters here: it counts a line as changed when it appears in one file and
/// not the other, which is what a reviewer reads.
fn measure(label: &'static str, paragraph_lines: usize, before: &str, after: &str) -> Measurement {
    let removed_lines: Vec<&str> = before
        .lines()
        .filter(|line| !after.lines().any(|other| other == *line))
        .collect();
    let added_lines: Vec<&str> = after
        .lines()
        .filter(|line| !before.lines().any(|other| other == *line))
        .collect();

    let changed_bytes = removed_lines
        .iter()
        .chain(&added_lines)
        .map(|l| l.len())
        .sum();
    Measurement {
        label,
        paragraph_lines,
        removed: removed_lines.len(),
        added: added_lines.len(),
        changed_bytes,
        edit_bytes: edit_size(before, after),
        total: after.lines().count(),
    }
}

/// The size of the edit itself: the bytes that differ once the shared prefix and
/// suffix are removed.
///
/// This is the floor a diff could theoretically show. Comparing it with
/// `changed_bytes` is what makes the cost of line structure visible.
fn edit_size(before: &str, after: &str) -> usize {
    let a = before.as_bytes();
    let b = after.as_bytes();
    let prefix = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    // The larger of the two remainders: an insertion changes one side only, and a
    // replacement changes both.
    (a.len() - prefix - suffix).max(b.len() - prefix - suffix)
}

/// Four sentences on one line, one word inserted near the start.
///
/// This is the case the design's prototype measured. The whole line diffs.
fn one_line_paragraph() -> Measurement {
    let before = "The system SHALL do the thing. It SHALL also do the second thing. \
                  And the third. And the fourth.";
    let after = "The system SHALL always do the thing. It SHALL also do the second thing. \
                 And the third. And the fourth.";
    measure(
        "One word inserted near the start of a four-sentence paragraph on one line",
        1,
        &document(before),
        &document(after),
    )
}

/// The same four sentences, one per line, and the same edit.
///
/// The author's line breaks are preserved, so only the edited line diffs. This is
/// the comparison that shows the cost belongs to the line structure.
fn multi_line_paragraph() -> Measurement {
    let before = "The system SHALL do the thing.\nIt SHALL also do the second thing.\n\
                  And the third.\nAnd the fourth.";
    let after = "The system SHALL always do the thing.\nIt SHALL also do the second thing.\n\
                 And the third.\nAnd the fourth.";
    measure(
        "The same edit, with the four sentences one per line",
        4,
        &document(before),
        &document(after),
    )
}

/// The same one-line paragraph, edited at the end rather than the start.
///
/// A rewrapping formatter's cascade needs room to show, so an edit at the end is
/// the case where rewrapping looks cheapest. Here it makes no difference at all,
/// because nothing is rewrapped either way.
fn edit_near_the_end_of_a_long_line() -> Measurement {
    let before = "The system SHALL do the thing. It SHALL also do the second thing. \
                  And the third. And the fourth.";
    let after = "The system SHALL do the thing. It SHALL also do the second thing. \
                 And the third. And the fourth, revised.";
    measure(
        "One phrase appended at the end of the same one-line paragraph",
        1,
        &document(before),
        &document(after),
    )
}

/// Inserting a whole unit, for scale: the diff is the unit and nothing else.
fn insert_a_unit() -> Measurement {
    let before = document("The system SHALL do the thing.");
    let with_third = before.replace(
        "  </body>",
        "    <section class=\"requirement\" id=\"req-3\">\n\
         \x20     <h2>Third</h2>\n\
         \x20     <p>The system SHALL do the third thing.</p>\n\
         \x20   </section>\n\
         \x20 </body>",
    );
    let after = canonicalize(&with_third).expect("the edited sample formats");
    measure(
        "A requirement inserted between two others",
        1,
        &before,
        &after,
    )
}

/// Changing one attribute value, which is the smallest edit the contract has.
fn change_a_change_state() -> Measurement {
    let before = document("The system SHALL do the thing.");
    let after = canonicalize(&before.replace(
        r#"<section class="requirement" id="req-1">"#,
        r#"<section class="requirement" id="req-1" data-change="modified">"#,
    ))
    .expect("the edited sample formats");
    measure("A change state added to one unit", 1, &before, &after)
}

/// One line changed inside a preserved block, which is the case where line
/// structure is the author's and the formatter must not touch it.
fn edit_inside_a_preserved_block() -> Measurement {
    let with_source = |line: &str| {
        canonicalize(&format!(
            "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>Locality</title></head><body>\
             <figure id=\"figure-1\"><figcaption>A code excerpt.</figcaption>\
             <pre><code>fn main() {{\n    let a = 1;\n    {line}\n    let c = 3;\n}}</code></pre>\
             </figure></body></html>\n"
        ))
        .expect("the sample formats")
    };
    measure(
        "One line changed inside a four-line preserved code excerpt",
        4,
        &with_source("let b = 2;"),
        &with_source("let b = 22;"),
    )
}
