//! Drives the mapping fixtures that pin the HTML-to-model projection.
//!
//! A mapping fixture is a directory under `fixtures/projection/` holding:
//!
//! - `input.html` — the document to project
//! - `expected.json` — `{ "rules": [...], "model": {...}, "faults": [...] }`
//!
//! `rules` names the sections of `doc/projection.md` the fixture pins, `model` is
//! the projected model as JSON, and `faults` lists the rule numbers the document
//! violates. The fixtures are the third piece of the public contract: an
//! independent implementation reading the schema and the projection specification
//! reproduces every `model` here, which is what makes the mapping pinned rather
//! than merely described.
//!
//! The harness drives the crate's public API only, so it tests the contract and
//! not this implementation's internals.

use std::path::{Path, PathBuf};

use htmlspec_parse::{parse_document, project};
use pretty_assertions::assert_eq;
use serde_json::Value;

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/projection")
}

/// One fixture's declared expectations.
struct Expected {
    /// The projection-specification sections this fixture pins.
    rules: Vec<String>,
    /// The projected model.
    model: Value,
    /// The rule numbers the document violates, in report order.
    faults: Vec<String>,
}

fn read_expected(dir: &Path) -> Expected {
    let path = dir.join("expected.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"));
    let value: Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?} is not valid JSON: {e}"));

    let strings = |key: &str| -> Vec<String> {
        value
            .get(key)
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("{path:?} has no {key} array"))
            .iter()
            .map(|v| {
                v.as_str()
                    .unwrap_or_else(|| panic!("{path:?}: {key} holds a non-string"))
                    .to_owned()
            })
            .collect()
    };

    let rules = strings("rules");
    assert!(
        !rules.is_empty(),
        "{path:?} declares no rules, so it pins nothing"
    );
    Expected {
        rules,
        model: value
            .get("model")
            .unwrap_or_else(|| panic!("{path:?} has no model"))
            .clone(),
        faults: strings("faults"),
    }
}

/// Every fixture directory, in name order.
fn fixtures() -> Vec<PathBuf> {
    let root = fixture_root();
    let mut found: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {root:?}: {e}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    found.sort();
    assert!(!found.is_empty(), "no mapping fixtures found in {root:?}");
    found
}

fn name_of(dir: &Path) -> String {
    dir.file_name()
        .expect("a fixture directory is named")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn every_fixture_projects_to_its_expected_model() {
    for dir in fixtures() {
        let name = name_of(&dir);
        let input = dir.join("input.html");
        let html = std::fs::read_to_string(&input)
            .unwrap_or_else(|e| panic!("cannot read {input:?}: {e}"));
        let expected = read_expected(&dir);

        // A mapping fixture is about the projection, so its markup must parse
        // with no recovery. A fixture that needs recovery is a parse-error
        // fixture and belongs in `fixtures/parse-errors/`.
        let outcome = parse_document(&html);
        assert!(
            outcome.is_recovery_free(),
            "{name}: the fixture's own markup recovered: {:?}",
            outcome.recovered
        );

        let projection = project(&outcome.document);
        let observed = serde_json::to_value(&projection.model).expect("the model serializes");
        assert_eq!(
            observed, expected.model,
            "{name}: the projected model differs from the fixture"
        );

        let observed_faults: Vec<&str> = projection.faults.iter().map(|f| f.rule).collect();
        assert_eq!(
            observed_faults,
            expected
                .faults
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            "{name}: the reported faults differ from the fixture\n{:#?}",
            projection.faults
        );
    }
}

/// Every numbered rule in the projection specification, read from the
/// specification itself.
///
/// Reading the rule list from the published document rather than restating it is
/// what makes the coverage check meaningful: a rule added to the specification
/// with no fixture fails here, which is exactly what the document-schema
/// capability requires ("Projection rule without a fixture").
fn specification_rules() -> Vec<String> {
    let mut found: Vec<String> = htmlspec_schema::schema_text::PROJECTION
        .lines()
        .filter_map(|line| {
            let head = line.split_whitespace().next()?;
            // A rule line opens with its own section number, as in `4.0.1`.
            let looks_numbered = head.contains('.')
                && head
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
            // A heading also opens with a number, and is not a rule.
            let is_heading = line.starts_with('#');
            (looks_numbered && !is_heading).then(|| head.to_owned())
        })
        .collect();
    found.sort();
    found.dedup();
    assert!(
        found.len() > 40,
        "the rule scan found only {} rules, so it is not reading the specification",
        found.len()
    );
    found
}

#[test]
fn every_projection_rule_has_a_mapping_fixture() {
    let mut claimed: Vec<String> = fixtures()
        .iter()
        .flat_map(|dir| read_expected(dir).rules)
        .collect();
    claimed.sort();
    claimed.dedup();

    let uncovered: Vec<String> = specification_rules()
        .into_iter()
        .filter(|rule| !claimed.contains(rule))
        .collect();
    assert!(
        uncovered.is_empty(),
        "these projection rules have no mapping fixture: {}",
        uncovered.join(", ")
    );
}

#[test]
fn no_fixture_claims_a_rule_the_specification_does_not_state() {
    let stated = specification_rules();
    let mut unknown: Vec<String> = Vec::new();
    for dir in fixtures() {
        let name = name_of(&dir);
        for rule in read_expected(&dir).rules {
            if !stated.contains(&rule) {
                unknown.push(format!("{name} claims {rule}"));
            }
        }
    }
    assert!(
        unknown.is_empty(),
        "a fixture claims a rule the projection specification does not state: {}",
        unknown.join(", ")
    );
}

/// A fixture declaring a fault must declare it against a rule it claims, so the
/// fault list and the coverage list cannot drift apart.
#[test]
fn a_declared_fault_belongs_to_a_rule_the_fixture_claims() {
    let mut orphans: Vec<String> = Vec::new();
    for dir in fixtures() {
        let name = name_of(&dir);
        let expected = read_expected(&dir);
        for fault in &expected.faults {
            if !expected.rules.contains(fault) {
                orphans.push(format!("{name} reports {fault} without claiming it"));
            }
        }
    }
    assert!(orphans.is_empty(), "{}", orphans.join(", "));
}
