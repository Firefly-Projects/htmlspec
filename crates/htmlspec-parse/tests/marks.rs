//! Holds the reader of change markup to the projection, over every mapping fixture.
//!
//! The reader keeps the `ins` and `del` boundaries the projection drops, and it must select the
//! same statement paragraphs and scenario entries the projection does. So for every unit it lists,
//! its current text is the projected text, and it lists exactly the units whose statements or
//! entries carry the markup. Which units carry it is decided here from the tree, by a rule written
//! apart from the reader's, so the two cannot agree by sharing a mistake.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use htmlspec_parse::model::{DocumentModel, UnitKind};
use htmlspec_parse::{marked_units, parse_document, project, MarkedText, MarkedUnit, Run};
use kuchikiki::NodeRef;

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/projection")
}

/// Every mapping fixture's input, by fixture name.
fn fixtures() -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = std::fs::read_dir(fixture_root())
        .expect("the fixtures directory exists")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.join("input.html").is_file())
        .map(|path| {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let source = std::fs::read_to_string(path.join("input.html"))
                .expect("the fixture's input is readable");
            (name, source)
        })
        .collect();
    found.sort();
    found
}

const HTML: &str = "http://www.w3.org/1999/xhtml";

fn is_change_markup(node: &NodeRef) -> bool {
    node.as_element().is_some_and(|element| {
        &*element.name.ns == HTML && matches!(element.name.local.as_ref(), "ins" | "del")
    })
}

fn unit_kinds(node: &NodeRef) -> Vec<UnitKind> {
    node.as_element()
        .and_then(|element| element.attributes.borrow().get("class").map(str::to_owned))
        .map(|class| {
            class
                .split_whitespace()
                .filter_map(UnitKind::from_class)
                .collect()
        })
        .unwrap_or_default()
}

/// The unit `node` belongs to: its nearest ancestor that is a unit.
fn owner(node: &NodeRef) -> Option<NodeRef> {
    node.ancestors()
        .find(|ancestor| !unit_kinds(ancestor).is_empty())
}

fn local_name(node: &NodeRef) -> Option<String> {
    node.as_element()
        .map(|element| element.name.local.as_ref().to_owned())
}

/// Whether `content` carries change markup for its unit: an `ins` or a `del` inside it, or one
/// around it below the unit.
fn carries_markup(content: &NodeRef, unit: &NodeRef) -> bool {
    content.descendants().any(|node| is_change_markup(&node))
        || content
            .ancestors()
            .take_while(|ancestor| ancestor != unit)
            .any(|ancestor| is_change_markup(&ancestor))
}

/// Whether `node` is content the comparison reads for the unit `kinds` names: a statement
/// paragraph of a requirement, or a scenario's entry.
fn is_content(node: &NodeRef, kinds: &[UnitKind]) -> bool {
    let field = node.as_element().and_then(|element| {
        element
            .attributes
            .borrow()
            .get("data-field")
            .map(str::to_owned)
    });
    match local_name(node).as_deref() {
        Some("p") => {
            kinds == [UnitKind::Requirement]
                && field.as_deref().is_none_or(|field| field == "statement")
        }
        Some("dd") => kinds == [UnitKind::Scenario],
        _ => false,
    }
}

/// The identifiers of the requirements and scenarios whose content carries change markup, among
/// those `projected` holds.
///
/// Which units exist is the projection's decision (section 4.3 drops a requirement nested in
/// another unit and a scenario no requirement encloses), and the reader follows it. Which of them
/// carry the markup is decided here.
fn units_carrying_markup(
    document: &NodeRef,
    projected: &BTreeMap<String, [Vec<String>; 4]>,
) -> BTreeSet<String> {
    let mut carrying = BTreeSet::new();
    for node in document.descendants() {
        let Some(unit) = owner(&node) else {
            continue;
        };
        let kinds = unit_kinds(&unit);
        if !is_content(&node, &kinds) || !carries_markup(&node, &unit) {
            continue;
        }
        if let Some(id) = unit
            .as_element()
            .and_then(|element| element.attributes.borrow().get("id").map(str::to_owned))
            .filter(|id| projected.contains_key(id))
        {
            carrying.insert(id);
        }
    }
    carrying
}

/// The current texts of `texts` that hold a word, which are what the projection projects.
fn current(texts: &[MarkedText]) -> Vec<String> {
    texts
        .iter()
        .map(MarkedText::current)
        .filter(|text| !text.is_empty())
        .collect()
}

/// Every requirement and scenario of `model` with an identifier, as (statements, given, when, then).
fn projected_units(model: &DocumentModel) -> BTreeMap<String, [Vec<String>; 4]> {
    let mut units = BTreeMap::new();
    for requirement in &model.requirements {
        if let Some(id) = &requirement.id {
            units.insert(
                id.clone(),
                [
                    requirement.statements.clone(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ],
            );
        }
        for scenario in &requirement.scenarios {
            if let Some(id) = &scenario.id {
                units.insert(
                    id.clone(),
                    [
                        Vec::new(),
                        scenario.given.clone(),
                        scenario.when.clone(),
                        scenario.then.clone(),
                    ],
                );
            }
        }
    }
    units
}

#[test]
fn the_marks_reader_agrees_with_the_projection_on_every_mapping_fixture() {
    let mut checked = 0;
    for (name, source) in fixtures() {
        let parsed = parse_document(&source);
        let model = project(&parsed.document).model;
        let marked = marked_units(&parsed.document);

        let projected = projected_units(&model);
        let listed: BTreeSet<String> = marked.keys().cloned().collect();
        assert_eq!(
            listed,
            units_carrying_markup(&parsed.document, &projected),
            "{name}: the reader lists other units than those whose content carries the markup"
        );

        for (id, unit) in &marked {
            let Some([statements, given, when, then]) = projected.get(id) else {
                panic!("{name}: {id} is marked and the projection has no such unit");
            };
            let MarkedUnit {
                statements: marked_statements,
                given: marked_given,
                when: marked_when,
                then: marked_then,
            } = unit;
            assert_eq!(
                &current(marked_statements),
                statements,
                "{name}: {id}'s statements"
            );
            assert_eq!(&current(marked_given), given, "{name}: {id}'s given");
            assert_eq!(&current(marked_when), when, "{name}: {id}'s when");
            assert_eq!(&current(marked_then), then, "{name}: {id}'s then");
            checked += 1;
        }
    }
    assert!(
        checked >= 2,
        "no fixture carries change markup in a requirement and a scenario, so nothing was checked"
    );
}

/// A paragraph wholly inside a `del` projects no statement, and the reader keeps it, as deleted
/// text only. Its neighbour wholly inside an `ins` is inserted text only.
#[test]
fn a_paragraph_wholly_deleted_is_kept_as_deleted_runs_only() {
    let source = std::fs::read_to_string(fixture_root().join("change-markup/input.html"))
        .expect("the change-markup fixture is readable");
    let parsed = parse_document(&source);
    let marked = marked_units(&parsed.document);
    let places = marked
        .get("req-places")
        .expect("the requirement carrying the marks is listed");

    let deleted = places
        .statements
        .iter()
        .find(|text| text.before() == "No other place SHALL hold a document.")
        .expect("the deleted paragraph is read");
    assert!(
        deleted
            .runs()
            .iter()
            .all(|run| matches!(run, Run::Deleted(_))),
        "{deleted:?}"
    );
    assert_eq!(deleted.current(), "");

    let inserted = places
        .statements
        .iter()
        .find(|text| {
            text.current() == "A runbook SHALL keep its documents in a directory of its own."
        })
        .expect("the inserted paragraph is read");
    assert!(
        inserted
            .runs()
            .iter()
            .all(|run| matches!(run, Run::Inserted(_))),
        "{inserted:?}"
    );
    assert_eq!(inserted.before(), "");
}
