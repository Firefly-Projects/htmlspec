//! Task 5.6: the formatter's two properties, asserted over generated documents.
//!
//! The two conformance guarantees this file exists for:
//!
//! - **Idempotence.** `format(format(x)) == format(x)`, byte for byte.
//! - **Semantic preservation.** Every unit, identifier, attribute value, and the
//!   document order of content survive formatting.
//!
//! Semantic preservation is asserted through the *projection* rather than by
//! inspecting markup. That is the strongest available statement of it: the
//! projected model is what every validator rule reads and what a consumer of the
//! contract receives, so a formatting change that the model cannot see is not a
//! semantic change, and one it can see fails here.
//!
//! The generator draws its vocabulary from the published schema through
//! `htmlspec_schema::facts` rather than restating it. A unit type or an enumerated
//! value added to the contract therefore reaches these properties without an edit
//! here, which is the point: a generator with its own private copy of the
//! vocabulary would stop covering the contract the moment the contract moved.
//!
//! The generator deliberately produces *awkward* documents — sprawling
//! indentation, attributes in shuffled order, mixed line endings, text with
//! newlines mid-sentence, quotes and ampersands in prose — because a generator
//! that only emitted already-canonical markup would assert nothing.

use std::collections::BTreeSet;

use htmlspec_parse::{parse_document, project};
use proptest::prelude::*;

/// One generated unit, before it is rendered to HTML.
#[derive(Clone, Debug)]
struct Unit {
    /// The reserved class name, which decides the unit type.
    class: String,
    /// The identifier, carrying the prefix the schema pairs with the class.
    id: String,
    /// The change state, from the schema's own enumeration.
    change: String,
    /// The unit's heading.
    title: String,
    /// The unit's paragraphs, as `(data-field, text)`. `None` is unmarked.
    paragraphs: Vec<(Option<String>, String)>,
    /// Whether the unit's attributes are written in shuffled order.
    shuffle_attributes: bool,
    /// How many spaces of source indentation the unit is written with.
    indent: usize,
}

/// Prose that exercises the escaping rules and carries author line breaks.
///
/// Every string here is legal document content, and each contains something the
/// formatter has to make a decision about.
const PROSE: &[&str] = &[
    "The system SHALL do the thing.",
    "A sentence with an & ampersand.",
    "A sentence with a \"quoted phrase\" in it.",
    "A sentence with <angle> brackets.",
    "One sentence.\nA second on its own line.",
    "One sentence.\n    A second, indented by the author.",
    "Text with a trailing space in the middle of a  run.",
    "Une phrase avec des accents: café, naïve, œuvre.",
    "A sentence with an apostrophe's apostrophe.",
];

/// A title, kept short because the heading is one line.
const TITLES: &[&str] = &[
    "First",
    "A name with an & in it",
    "A \"quoted\" name",
    "Étude",
];

/// The unit types the schema defines, as `(class, prefix)`.
///
/// Read from the schema so the generator covers the contract rather than a copy
/// of it.
fn unit_types() -> Vec<(String, String)> {
    let types = htmlspec_schema::facts::identifier_prefixes();
    assert!(
        !types.is_empty(),
        "the schema defines no unit types, so the generator would cover nothing"
    );
    types
}

/// The change states the schema enumerates.
fn change_states() -> Vec<String> {
    let states = htmlspec_schema::facts::enum_values("definitions/changeState");
    assert!(!states.is_empty(), "the schema enumerates no change states");
    states
}

/// The contract version the schema requires a document to declare.
fn contract_version() -> String {
    htmlspec_schema::facts::contract_version().expect("the schema declares a contract version")
}

/// An artifact kind the schema permits.
fn document_kind() -> String {
    htmlspec_schema::facts::artifact_kinds()
        .into_iter()
        .next()
        .expect("the schema enumerates at least one artifact kind")
}

/// Generates one unit of a schema-defined type.
fn any_unit() -> impl Strategy<Value = Unit> {
    let types = unit_types();
    let states = change_states();
    (
        0..types.len(),
        0..states.len(),
        0usize..TITLES.len(),
        prop::collection::vec(
            (
                prop::option::of(Just("note".to_owned())),
                0usize..PROSE.len(),
            ),
            1..4,
        ),
        any::<bool>(),
        0usize..9,
        1u32..999,
    )
        .prop_map(
            move |(type_at, state_at, title_at, paragraphs, shuffle, indent, serial)| {
                let (class, prefix) = types[type_at].clone();
                Unit {
                    class,
                    id: format!("{prefix}{serial}"),
                    change: states[state_at].clone(),
                    title: TITLES[title_at].to_owned(),
                    paragraphs: paragraphs
                        .into_iter()
                        .map(|(field, at)| (field, PROSE[at].to_owned()))
                        .collect(),
                    shuffle_attributes: shuffle,
                    indent,
                }
            },
        )
}

/// How a generated document's whitespace and line endings are written.
#[derive(Clone, Copy, Debug)]
struct Style {
    /// Whether the document is written with CRLF line endings.
    crlf: bool,
    /// Whether blank lines are inserted between units.
    blank_lines: bool,
}

fn any_style() -> impl Strategy<Value = Style> {
    (any::<bool>(), any::<bool>()).prop_map(|(crlf, blank_lines)| Style { crlf, blank_lines })
}

/// Renders generated units into an HTML document.
///
/// Identifiers are made unique by suffix, because a duplicate identifier is a
/// validation fault and this file is about formatting, not validity — though the
/// formatter would handle it either way.
fn render(units: &[Unit], style: Style) -> String {
    let mut body = String::new();
    let mut used: BTreeSet<String> = BTreeSet::new();

    for (position, unit) in units.iter().enumerate() {
        let mut id = unit.id.clone();
        while !used.insert(id.clone()) {
            id = format!("{id}x{position}");
        }

        let pad = " ".repeat(unit.indent);
        let attributes = if unit.shuffle_attributes {
            format!(
                r#"data-change="{}" id="{id}" class="{}""#,
                unit.change, unit.class
            )
        } else {
            format!(
                r#"class="{}" id="{id}" data-change="{}""#,
                unit.class, unit.change
            )
        };

        if style.blank_lines {
            body.push('\n');
        }
        body.push_str(&format!("{pad}<section {attributes}>\n"));
        body.push_str(&format!("{pad}  <h2>{}</h2>\n", escape_text(&unit.title)));
        for (field, text) in &unit.paragraphs {
            let marker = match field {
                Some(name) => format!(r#" data-field="{name}""#),
                None => String::new(),
            };
            body.push_str(&format!("{pad}  <p{marker}>{}</p>\n", escape_text(text)));
        }
        body.push_str(&format!("{pad}</section>\n"));
    }

    let document = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n\
         <meta name=\"htmlspec-version\" content=\"{}\">\n\
         <meta name=\"htmlspec-kind\" content=\"{}\">\n\
         <title>Generated</title>\n</head>\n<body>\n{body}</body>\n</html>\n",
        contract_version(),
        document_kind(),
    );

    if style.crlf {
        document.replace('\n', "\r\n")
    } else {
        document
    }
}

/// Escapes generated text so the rendered document parses cleanly.
///
/// This is the test's own minimal escaper, not the formatter's. Using the
/// formatter's here would make the property assert that its escaping agrees with
/// itself, which is no assertion at all.
fn escape_text(raw: &str) -> String {
    raw.replace('&', "&amp;").replace('<', "&lt;")
}

/// A generated document, together with the parts of it a property inspects.
fn any_document() -> impl Strategy<Value = String> {
    (prop::collection::vec(any_unit(), 1..5), any_style())
        .prop_map(|(units, style)| render(&units, style))
}

/// The projected model as comparable JSON, or the reason it could not be read.
fn model_of(source: &str) -> serde_json::Value {
    let outcome = parse_document(source);
    let projection = project(&outcome.document);
    serde_json::json!({
        "model": serde_json::to_value(&projection.model).expect("the model serializes"),
        "faults": projection
            .faults
            .iter()
            .map(|fault| format!("{fault}"))
            .collect::<Vec<String>>(),
    })
}

proptest! {
    // The generated documents are small and formatting is fast, so the case
    // count is raised above the default: the interesting failures are
    // combinations of style and unit type rather than deep single cases.
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// `format(format(x)) == format(x)`, byte for byte.
    #[test]
    fn formatting_is_a_fixed_point(source in any_document()) {
        let once = htmlspec_format::format(&source)
            .map_err(|e| TestCaseError::fail(format!("a generated document was refused: {e}")))?;
        let twice = htmlspec_format::format(&once)
            .map_err(|e| TestCaseError::fail(format!("the formatter refused its own output: {e}")))?;
        prop_assert_eq!(&once, &twice, "formatting is not a fixed point");

        // A third pass, because a two-cycle would satisfy a single repeat.
        let thrice = htmlspec_format::format(&twice)
            .map_err(|e| TestCaseError::fail(format!("the formatter refused its own output: {e}")))?;
        prop_assert_eq!(&twice, &thrice);
    }

    /// Every unit, identifier, attribute value, and the order of content survive.
    #[test]
    fn formatting_preserves_the_projected_model(source in any_document()) {
        let formatted = htmlspec_format::format(&source)
            .map_err(|e| TestCaseError::fail(format!("a generated document was refused: {e}")))?;
        prop_assert_eq!(
            model_of(&source),
            model_of(&formatted),
            "formatting changed the projected model"
        );
    }

    /// Two authoring styles of the same content converge on one file, which is
    /// what lets two agents edit a document without fighting over formatting.
    #[test]
    fn any_two_authoring_styles_converge(
        units in prop::collection::vec(any_unit(), 1..5),
        first in any_style(),
        second in any_style(),
    ) {
        let a = htmlspec_format::format(&render(&units, first))
            .map_err(|e| TestCaseError::fail(format!("refused: {e}")))?;
        let b = htmlspec_format::format(&render(&units, second))
            .map_err(|e| TestCaseError::fail(format!("refused: {e}")))?;
        prop_assert_eq!(a, b, "two authoring styles did not converge");
    }

    /// The file-level normalization requirement, on every generated document.
    #[test]
    fn every_formatted_document_is_normalized(source in any_document()) {
        let formatted = htmlspec_format::format(&source)
            .map_err(|e| TestCaseError::fail(format!("refused: {e}")))?;

        prop_assert!(formatted.ends_with('\n'), "no final newline");
        prop_assert!(!formatted.ends_with("\n\n"), "more than one final newline");
        prop_assert!(!formatted.contains('\r'), "a carriage return survived");
        for (number, line) in formatted.lines().enumerate() {
            // The generator emits no preserved content, so no line may end in
            // whitespace. The dogfood documents cover the preserved case, where
            // trailing whitespace is content.
            prop_assert_eq!(
                line.trim_end(),
                line,
                "line {} has trailing whitespace",
                number + 1
            );
        }
    }
}

/// The generator must actually cover every unit type the schema defines, or the
/// properties above are quietly narrower than they read.
///
/// This is a plain test rather than a property: it asserts a fact about the
/// generator's own vocabulary, which does not vary per case.
#[test]
fn the_generator_covers_every_unit_type_the_schema_defines() {
    let types = unit_types();
    let rendered = render(
        &types
            .iter()
            .enumerate()
            .map(|(at, (class, prefix))| Unit {
                class: class.clone(),
                id: format!("{prefix}{}", at + 1),
                change: change_states()[0].clone(),
                title: "Name".to_owned(),
                paragraphs: vec![(None, PROSE[0].to_owned())],
                shuffle_attributes: false,
                indent: 0,
            })
            .collect::<Vec<Unit>>(),
        Style {
            crlf: false,
            blank_lines: false,
        },
    );

    let formatted = htmlspec_format::format(&rendered).expect("the sample formats");
    for (class, _) in &types {
        assert!(
            formatted.contains(&format!(r#"class="{class}""#)),
            "the generator's vocabulary does not reach the {class} unit"
        );
    }
    assert_eq!(model_of(&rendered), model_of(&formatted));
}
