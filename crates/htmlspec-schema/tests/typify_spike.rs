//! Task 1.3 spike: are the generated types usable, and which schema
//! constraints do not survive generation?
//!
//! Each test states one measured fact. The tests that assert a *loss* are
//! deliberate: they pin the gap that makes the runtime validator the
//! enforcement layer, so a later `typify` upgrade that closes the gap fails
//! here and prompts a review rather than passing silently.

use std::str::FromStr;

use htmlspec_schema::spike::{
    RequirementSpike, RequirementSpikeChange, RequirementSpikeId, RequirementSpikeStatementsItem,
    RequirementSpikeTitle, Scenario, ScenarioId, ScenarioTitle,
};

fn valid_requirement() -> RequirementSpike {
    RequirementSpike {
        unit: serde_json::json!("requirement"),
        id: RequirementSpikeId::from_str("req-theme-selection").expect("valid id"),
        change: RequirementSpikeChange::Added,
        title: RequirementSpikeTitle::from_str("Theme selection").expect("valid title"),
        statements: vec![RequirementSpikeStatementsItem::from_str(
            "The application MUST let the user select a light or dark theme.",
        )
        .expect("valid statement")],
        scenarios: vec![Scenario {
            unit: serde_json::json!("scenario"),
            id: ScenarioId::from_str("scenario-theme-persists").expect("valid id"),
            title: ScenarioTitle::from_str("The selected theme persists").expect("valid title"),
            given: vec![],
            when: vec!["The user selects the dark theme.".to_owned()],
            then: vec!["The application uses the dark theme after the next launch.".to_owned()],
        }],
    }
}

#[test]
fn generated_types_are_constructible() {
    let requirement = valid_requirement();
    assert_eq!(&*requirement.id, "req-theme-selection");
    assert_eq!(requirement.scenarios.len(), 1);
}

#[test]
fn generated_enums_are_matchable() {
    let state = match valid_requirement().change {
        RequirementSpikeChange::Added => "added",
        RequirementSpikeChange::Modified => "modified",
        RequirementSpikeChange::Removed => "removed",
        RequirementSpikeChange::Unchanged => "unchanged",
    };
    assert_eq!(state, "added");
}

#[test]
fn generated_types_round_trip_through_serde() {
    let before = valid_requirement();
    let json = serde_json::to_string(&before).expect("serializes");
    let after: RequirementSpike = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(
        serde_json::to_value(&before).expect("value"),
        serde_json::to_value(&after).expect("value")
    );
}

#[test]
fn pattern_survives_generation() {
    assert!(RequirementSpikeId::from_str("req-ok").is_ok());
    assert!(RequirementSpikeId::from_str("requirement-ok").is_err());
    assert!(RequirementSpikeId::from_str("req-Not-Lower").is_err());
    let bad: Result<RequirementSpike, _> = serde_json::from_value(serde_json::json!({
        "unit": "requirement",
        "id": "REQ-shouty",
        "title": "t",
        "statements": ["s"],
        "scenarios": []
    }));
    assert!(bad.is_err(), "deserialization enforces the id pattern");
}

#[test]
fn min_length_survives_generation() {
    assert!(RequirementSpikeTitle::from_str("").is_err());
    assert!(RequirementSpikeTitle::from_str("t").is_ok());
}

#[test]
fn additional_properties_false_survives_generation() {
    let bad: Result<Scenario, _> = serde_json::from_value(serde_json::json!({
        "unit": "scenario",
        "id": "scenario-x",
        "title": "t",
        "when": ["w"],
        "then": ["t"],
        "extra": 1
    }));
    assert!(bad.is_err(), "unknown fields are denied");
}

#[test]
fn loss_min_items_does_not_survive_generation() {
    // `minItems: 1` becomes a plain `Vec`, so the generated type accepts a
    // requirement with no scenarios and no statements.
    let empty: RequirementSpike = serde_json::from_value(serde_json::json!({
        "unit": "requirement",
        "id": "req-empty",
        "title": "t",
        "statements": [],
        "scenarios": []
    }))
    .expect("generated type accepts an empty scenario list");
    assert!(empty.scenarios.is_empty());
    assert!(empty.statements.is_empty());
}

#[test]
fn loss_const_does_not_survive_generation() {
    // `const` becomes `serde_json::Value`, which is weaker than the plain
    // `String` the design anticipated: the field accepts any JSON value.
    let wrong: RequirementSpike = serde_json::from_value(serde_json::json!({
        "unit": 42,
        "id": "req-wrong-unit",
        "title": "t",
        "statements": ["s"],
        "scenarios": [{
            "unit": {"not": "a string"},
            "id": "scenario-x", "title": "t", "when": ["w"], "then": ["t"]
        }]
    }))
    .expect("generated type accepts any value for a const field");
    assert_eq!(wrong.unit, serde_json::json!(42));
}
