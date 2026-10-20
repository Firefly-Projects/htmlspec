//! The published contract is readable, self-consistent, and complete.
//!
//! These tests guard the pieces of the contract that no schema validator can
//! check: that the schema and the projection specification agree about the
//! vocabulary, and that neither one presents itself as the whole contract.

use serde_json::Value;

fn schema() -> Value {
    serde_json::from_str(htmlspec_schema::schema_text::DOCUMENT).expect("schema is valid JSON")
}

const UNITS: [&str; 7] = [
    "requirement",
    "scenario",
    "decision",
    "task",
    "evidence",
    "risk",
    "verification",
];

const PREFIXES: [(&str, &str); 7] = [
    ("requirement", "req-"),
    ("scenario", "scenario-"),
    ("decision", "decision-"),
    ("task", "task-"),
    ("evidence", "evidence-"),
    ("risk", "risk-"),
    ("verification", "verify-"),
];

#[test]
fn the_schema_defines_exactly_the_seven_units() {
    let definitions = schema()["definitions"].clone();
    for unit in UNITS {
        let def = &definitions[unit];
        assert!(def.is_object(), "schema has no definition for {unit}");
        assert_eq!(
            def["properties"]["unit"]["const"], unit,
            "{unit} has the wrong discriminator"
        );
        assert_eq!(
            def["properties"]["unit"]["type"], "string",
            "{unit}'s discriminator must declare a string type, so the generated \
             field is a String rather than an arbitrary JSON value"
        );
    }
}

#[test]
fn every_unit_requires_an_identifier_a_change_state_and_a_title() {
    let definitions = schema()["definitions"].clone();
    for unit in UNITS {
        let required = definitions[unit]["required"]
            .as_array()
            .expect("required is a list")
            .iter()
            .map(|v| v.as_str().expect("string").to_owned())
            .collect::<Vec<_>>();
        for field in ["unit", "id", "change", "title"] {
            assert!(
                required.contains(&field.to_owned()),
                "{unit} does not require {field}"
            );
        }
    }
}

#[test]
fn every_identifier_prefix_is_expressed_as_a_pattern() {
    let definitions = schema()["definitions"].clone();
    for (unit, prefix) in PREFIXES {
        let name = match unit {
            "verification" => "verificationId".to_owned(),
            other => format!("{other}Id"),
        };
        let pattern = definitions[&name]["pattern"]
            .as_str()
            .unwrap_or_else(|| panic!("{name} has no pattern"));
        assert!(
            pattern.starts_with(&format!("^{prefix}")),
            "{name} pattern {pattern} does not pin the {prefix} prefix"
        );
    }
}

#[test]
fn the_generic_identifier_pattern_accepts_every_prefix_and_nothing_else() {
    let pattern = schema()["definitions"]["unitId"]["pattern"]
        .as_str()
        .expect("unitId has a pattern")
        .to_owned();
    for (_, prefix) in PREFIXES {
        let stem = prefix.trim_end_matches('-');
        assert!(
            pattern.contains(stem),
            "unitId pattern omits the {prefix} prefix"
        );
    }
    assert!(
        !pattern.contains("scenarios"),
        "unitId pattern must name prefixes, not field names"
    );
}

#[test]
fn the_schema_states_that_it_is_not_the_whole_contract() {
    let description = schema()["description"]
        .as_str()
        .expect("the schema has a description")
        .to_owned();
    assert!(
        description.contains("projection specification"),
        "the schema must cross-reference the projection specification, so that a \
         reader cannot mistake it for the whole contract"
    );
    assert!(
        description.contains("NOT the HTML"),
        "the schema must say that it does not describe the HTML"
    );
}

#[test]
fn the_projection_specification_covers_every_required_topic() {
    let projection = htmlspec_schema::schema_text::PROJECTION;
    // The topics the document-schema capability requires the projection to state.
    for topic in [
        "What creates a unit",
        "Attributes that project",
        "References",
        "Whitespace and text",
        "Figures and visualization sources",
        "Presentational markup",
        "Document metadata",
    ] {
        assert!(
            projection.contains(topic),
            "the projection specification does not cover: {topic}"
        );
    }
}

#[test]
fn the_projection_specification_declares_the_contract_version() {
    assert!(
        htmlspec_schema::schema_text::PROJECTION.contains(htmlspec_schema::CONTRACT_VERSION),
        "the projection specification does not declare the contract version"
    );
    assert_eq!(
        accepted_versions().last().map(String::as_str),
        Some(htmlspec_schema::CONTRACT_VERSION),
        "the schema's newest accepted version and the crate disagree about the contract version"
    );
}

/// The versions the schema's `schemaVersion` enumeration accepts, in its order.
fn accepted_versions() -> Vec<String> {
    schema()["properties"]["schemaVersion"]["enum"]
        .as_array()
        .expect("schemaVersion enumerates the versions it accepts")
        .iter()
        .map(|version| {
            version
                .as_str()
                .expect("an accepted version is a string")
                .to_owned()
        })
        .collect()
}

/// A version's numeric components, for ordering two of them.
fn components(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| part.parse().expect("a version component is numeric"))
        .collect()
}

#[test]
fn the_accepted_versions_are_ordered_oldest_first() {
    // `facts::contract_version` takes the last entry as the newest rather than
    // comparing, so the order is part of the contract and is pinned here.
    let versions = accepted_versions();
    assert!(!versions.is_empty(), "the schema accepts no version at all");
    for pair in versions.windows(2) {
        assert!(
            components(&pair[0]) < components(&pair[1]),
            "{} is listed before {}, so the enumeration is not oldest first",
            pair[0],
            pair[1]
        );
    }
}

/// The `0.x` versions 1.0.0 kept. 1.0.0 is the 0.7.0 vocabulary unchanged, so it withdrew none of
/// them: every document written before it declares one of these.
const KEPT_BEFORE_1_0_0: [&str; 7] = [
    "0.1.0", "0.2.0", "0.3.0", "0.4.0", "0.5.0", "0.6.0", "0.7.0",
];

/// The versions the changelog dates, from its `## <version> — <date>` headings.
fn dated_versions() -> Vec<String> {
    htmlspec_schema::schema_text::CHANGELOG
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .filter_map(|rest| rest.split_whitespace().next())
        .filter(|word| {
            word.split('.').count() == 3
                && word
                    .split('.')
                    .all(|part| part.bytes().all(|b| b.is_ascii_digit()))
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_minor_bump_keeps_accepting_the_versions_it_succeeded() {
    // A minor or patch bump keeps every version it succeeded, and a document
    // declares its version. So every released version of the newest major must
    // still be accepted, or the bump invalidated every document that declared it.
    // Until 1.0.0 this test also refused any version from another major, because a
    // major bump is where versions can leave the set. 1.0.0 is the major that left
    // none: the 0.x versions stay accepted beside 1.x, as the contract's versioning
    // states, and nothing else from another major is.
    let versions = accepted_versions();
    for kept in KEPT_BEFORE_1_0_0 {
        assert!(
            versions.iter().any(|v| v == kept),
            "{kept} is no longer accepted, so every document declaring it fails: {versions:?}"
        );
    }
    let newest = components(htmlspec_schema::CONTRACT_VERSION);
    for version in dated_versions() {
        if components(&version)[0] == newest[0] {
            assert!(
                versions.contains(&version),
                "{version} is a released version of the newest major and is no longer accepted: \
                 {versions:?}"
            );
        }
    }
    for version in &versions {
        assert!(
            components(version)[0] == newest[0] || KEPT_BEFORE_1_0_0.contains(&version.as_str()),
            "{version} is accepted beside {}, from a major the contract does not keep",
            htmlspec_schema::CONTRACT_VERSION
        );
    }
}

#[test]
fn the_changelog_records_every_accepted_version() {
    // A version the schema accepts and the changelog does not date is a version a
    // reader cannot place in time.
    let changelog = htmlspec_schema::schema_text::CHANGELOG;
    for version in accepted_versions() {
        let heading = format!("## {version} ");
        assert!(
            changelog.lines().any(|line| line.starts_with(&heading)),
            "the changelog has no entry for {version}, which the schema accepts"
        );
    }
}

#[test]
fn the_contract_version_is_a_semantic_version_and_not_a_date() {
    // The version answers a compatibility question, and a date cannot. This test
    // exists because the contract shipped a date first, so a date is the
    // regression to guard against rather than a hypothetical one.
    let version = htmlspec_schema::CONTRACT_VERSION;
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "{version} is not major.minor.patch");
    for part in &parts {
        assert!(
            !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()),
            "{version} has a non-numeric component"
        );
        assert!(
            *part == "0" || !part.starts_with('0'),
            "{version} has a leading zero, which two strings can share as one version"
        );
    }
}

#[test]
fn the_schema_identifier_carries_the_contract_version() {
    // A published `$id` names the version it describes, so two versions of the
    // schema cannot collide at one URL.
    let id = schema()["$id"]
        .as_str()
        .expect("the schema has an $id")
        .to_owned();
    assert!(
        id.contains(htmlspec_schema::CONTRACT_VERSION),
        "the schema $id {id} does not name the contract version"
    );
}

#[test]
fn the_changelog_records_the_current_version_with_the_date_it_landed() {
    // The version carries no date, so the changelog is where the calendar lives.
    // A version with no entry leaves a reader unable to order two releases.
    let changelog = htmlspec_schema::schema_text::CHANGELOG;
    let heading = format!("## {}", htmlspec_schema::CONTRACT_VERSION);
    let line = changelog
        .lines()
        .find(|l| l.starts_with(&heading))
        .unwrap_or_else(|| panic!("the changelog has no entry for {heading}"));

    // `## 0.1.0 — 2026-08-07`: the entry dates the release.
    let date = line
        .split_whitespace()
        .last()
        .expect("the entry heading has a final field");
    let parts: Vec<&str> = date.split('-').collect();
    assert!(
        parts.len() == 3 && parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit())),
        "the {heading} entry does not end in an ISO date, found {date:?}"
    );
}

#[test]
fn the_schema_states_the_version_bump_policy() {
    // The schema owns the meaning of a version, exactly as it owns the prefix
    // table. A reader must not have to infer the policy from the number.
    let description = schema()["properties"]["schemaVersion"]["description"]
        .as_str()
        .expect("schemaVersion has a description")
        .to_owned();
    for term in ["major", "minor", "patch", "CHANGELOG.md"] {
        assert!(
            description.contains(term),
            "the schemaVersion description does not explain {term}"
        );
    }
    assert!(
        description.contains("exact"),
        "the schema must state that the version match is exact"
    );
}

#[test]
fn every_unit_can_carry_notes() {
    // Task 3.5, finding 3: explanatory prose stays addressable without becoming
    // normative, so every unit has a `notes` array.
    let definitions = schema()["definitions"].clone();
    for unit in UNITS {
        assert!(
            definitions[unit]["properties"]["notes"].is_object(),
            "{unit} cannot carry a note"
        );
    }
}

#[test]
fn the_document_can_hold_its_own_prose() {
    // Task 3.5, finding 1.
    assert!(
        schema()["properties"]["summary"].is_object(),
        "the document model has no place for the argument a change rests on"
    );
}

#[test]
fn a_figure_is_addressable_but_is_not_a_unit() {
    // Task 3.5, finding 5.
    let s = schema();
    assert!(
        s["definitions"]["figure"]["properties"]["id"].is_object(),
        "a figure cannot be addressed by a link"
    );
    let unit_pattern = s["definitions"]["unitId"]["pattern"]
        .as_str()
        .expect("unitId has a pattern");
    assert!(
        !unit_pattern.contains("figure"),
        "a figure identifier must not be a reference target"
    );
}

#[test]
fn no_field_is_projected_by_position() {
    // Task 3.5, finding 2. Position decided a field in the draft, and the
    // failure was silent. The projection specification must say so explicitly.
    let projection = htmlspec_schema::schema_text::PROJECTION;
    assert!(
        projection.contains("Fields are named, never positional"),
        "the projection specification must state that position does not decide a field"
    );
    assert!(
        projection.contains("data-field"),
        "the projection specification must document the data-field attribute"
    );
}

#[test]
fn the_prefix_table_lives_in_the_schema_and_nowhere_else() {
    // Task 3.7. One table, in the schema's own descriptions. The projection
    // specification must point at it rather than restate it, so the two cannot
    // disagree about which prefix belongs to which unit.
    let description = schema()["definitions"]["unitId"]["description"]
        .as_str()
        .expect("unitId has a description")
        .to_owned();
    for (unit, prefix) in PREFIXES {
        assert!(
            description.contains(prefix) && description.contains(unit),
            "the schema's prefix table omits {unit} or {prefix}"
        );
    }
    assert!(
        description.contains("verify-` does not match its unit name")
            || description.contains("does not match its unit name"),
        "the table must warn that a prefix cannot be derived from a unit name"
    );

    let projection = htmlspec_schema::schema_text::PROJECTION;
    assert!(
        !projection.contains("`verify-` verification"),
        "the projection specification restates the prefix table instead of \
         pointing at the schema"
    );
    assert!(
        projection.contains("owns the identifier prefix table"),
        "the projection specification must name the schema as the owner of the \
         prefix table"
    );
}

#[test]
fn change_state_semantics_are_documented_in_the_schema() {
    // Task 3.7. Each state says what it means, including the two rules a reader
    // cannot guess: a removed unit stays in the document, and an absent
    // attribute means unchanged.
    let description = schema()["definitions"]["changeState"]["description"]
        .as_str()
        .expect("changeState has a description")
        .to_owned();
    for state in ["added", "modified", "removed", "unchanged"] {
        assert!(description.contains(state), "state {state} is undocumented");
    }
    assert!(
        description.contains("STAYS in the document"),
        "the schema must say that a removed unit remains visible"
    );
    assert!(
        description.contains("gates nothing"),
        "the schema must say that a change state is not a workflow phase"
    );
}

/// Task 1.3: the artifact kinds are exactly four, and the set is closed.
///
/// The count is the assertion. Every per-kind rule this change adds is stated
/// against one of these four names, so a fifth kind arriving without a decision
/// would be a kind for which no rule says anything — valid by default, and
/// silently outside the vocabulary the three downstream rules were written for.
/// A closed set is what makes "no other unit-and-kind combination is reported" a
/// statement someone can check.
#[test]
fn the_schema_enumerates_exactly_the_four_artifact_kinds() {
    let schema = schema();
    let kind = schema["properties"]["kind"].clone();
    let declared: Vec<&str> = kind["enum"]
        .as_array()
        .expect("kind is an enumeration")
        .iter()
        .map(|v| v.as_str().expect("a kind is a string"))
        .collect();
    assert_eq!(
        declared,
        vec!["proposal", "design", "tasks", "specification"],
        "the artifact kinds changed; adding or removing one is a decision with a \
         compatibility cost, and every per-kind rule is stated against these four"
    );

    // Required and undefaulted, which is what makes an unmigrated document fail
    // with "no kind declared" rather than becoming whichever kind is convenient.
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required is a list")
        .iter()
        .map(|v| v.as_str().expect("a string"))
        .collect();
    assert!(
        required.contains(&"kind"),
        "kind must be required: an optional kind with a default would make every \
         unmigrated document silently become that default"
    );
    assert!(
        kind.get("default").is_none(),
        "kind must have no default, or a proposal would acquire the strictest \
         kind's placement rule without its author declaring anything"
    );

    // The lifecycle status is gone from the model, not merely unused. A schema
    // still carrying it would leave a field a reader would reasonably trust.
    assert!(
        schema["properties"].get("status").is_none(),
        "the document-level status must be removed: a stage describes a change as \
         it moves rather than one of its documents"
    );
    assert!(
        !required.contains(&"status"),
        "the document-level status must not be required"
    );
}

/// The kind's intended content is documented where the vocabulary lives.
///
/// An author reading the schema has to be able to tell which artifact each name
/// means, and which two kinds constrain structure. Stated in the schema rather
/// than only in the specs, for the same reason the change states are.
#[test]
fn artifact_kind_semantics_are_documented_in_the_schema() {
    let description = schema()["properties"]["kind"]["description"]
        .as_str()
        .expect("kind has a description")
        .to_owned();
    for kind in ["proposal", "design", "tasks", "specification"] {
        assert!(description.contains(kind), "kind {kind} is undocumented");
    }
    assert!(
        description.contains("CLOSED"),
        "the schema must say the set of kinds is closed"
    );
    assert!(
        description.contains("no enforced structure"),
        "the schema must say that two kinds constrain nothing, which is the \
         freedom the upstream stages depend on"
    );
    assert!(
        description.contains("not authoritative"),
        "the schema must distinguish permitted from authoritative: a requirement \
         in a proposal is valid and is not part of the contract"
    );
}

#[test]
fn each_piece_of_the_contract_points_at_the_others() {
    let description = schema()["description"]
        .as_str()
        .expect("schema description")
        .to_owned();
    assert!(
        description.contains("doc/projection.md") && description.contains("conformance fixtures"),
        "the schema must name the other two pieces of the contract"
    );
    assert!(
        htmlspec_schema::schema_text::PROJECTION.contains("The contract is three pieces"),
        "the projection specification must name the other two pieces"
    );
}
