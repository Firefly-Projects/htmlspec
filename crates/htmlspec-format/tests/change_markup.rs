//! HTML's change markup, `ins` and `del`, laid out as the measurement found it.
//!
//! The formatter's element table names neither element, so each is an unknown element, and an
//! unknown element is inline. That is what a mark inside a sentence needs: in
//! `<ins><code>sync</code>, </ins>`, a line break before the comma would render as a space. A
//! block-level mark around a whole paragraph is still laid out with the paragraph on its own line.
//! `mark-the-words-a-modification-changed` gives both elements a meaning and changes nothing here,
//! so these tests pin the measured layout against a later edit to the table.

/// A specification whose one requirement holds `body` after its heading, formatted once.
fn canonical(body: &str) -> String {
    let source = format!(
        "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>Marks</title></head><body><main>\
         <section class=\"requirement\" id=\"req-places\" data-change=\"modified\">\
         <h2>Places</h2>{body}\
         <article class=\"scenario\" id=\"scenario-places\"><h3>Found</h3>\
         <dl><dt>When</dt><dd>it runs</dd><dt>Then</dt><dd>it works</dd></dl>\
         </article></section></main></body></html>\n"
    );
    htmlspec_format::format(&source).expect("the sample formats")
}

/// The statements of the measurement's section 2, with `sync` inserted into one and a word
/// deleted beside an inserted phrase in the other.
const INLINE: [&str; 2] = [
    "<p>The spellings are therefore fixed rather than left to an implementer: \
     <code>validate</code>, <code>archive</code>, <code>init</code>, <code>new</code>, \
     <code>list</code>, <code>status</code>, <ins><code>sync</code>, </ins><code>instructions</code>, \
     and <code>store</code> name the operations OpenSpec also provides.</p>",
    "<p>That directory SHALL hold the project's accepted specifications, its active changes, \
     <del>and </del>its archive, <ins>and its runbooks, </ins>each in its own place.</p>",
];

/// Inline marks are canonical as written: each statement stays on one line, byte for byte, and
/// a second run changes nothing.
#[test]
fn inline_marks_are_canonical_as_written() {
    let formatted = canonical(&INLINE.concat());
    for statement in INLINE {
        assert!(
            formatted.lines().any(|line| line.trim() == statement),
            "the statement was not kept on one line as written:\n{statement}\n\n{formatted}"
        );
    }
    assert_eq!(
        htmlspec_format::format(&formatted).expect("it formats"),
        formatted,
        "a second run changed the document"
    );
}

/// A block-level mark is laid out with its paragraph on its own line, one level deeper, and a
/// second run changes nothing.
#[test]
fn a_block_level_mark_puts_its_paragraph_on_its_own_line() {
    let formatted = canonical(
        "<p>The spellings SHALL be fixed.</p>\
         <del><p>No other spelling SHALL name a command.</p></del>\
         <ins><p>A spelling OpenSpec does not provide SHALL be one this project chooses.</p></ins>",
    );
    let lines: Vec<&str> = formatted.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.trim() == "<del>")
        .unwrap_or_else(|| panic!("no <del> on a line of its own:\n{formatted}"));
    let indent = lines[at].len() - lines[at].trim_start().len();
    let expected = [
        "<del>",
        "  <p>No other spelling SHALL name a command.</p>",
        "</del>",
        "<ins>",
        "  <p>A spelling OpenSpec does not provide SHALL be one this project chooses.</p>",
        "</ins>",
    ];
    let found: Vec<&str> = lines[at..at + expected.len()]
        .iter()
        .map(|line| &line[indent.min(line.len())..])
        .collect();
    assert_eq!(found, expected, "\n{formatted}");
    assert_eq!(
        htmlspec_format::format(&formatted).expect("it formats"),
        formatted,
        "a second run changed the document"
    );
}
