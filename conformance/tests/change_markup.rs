//! Change markup at every placement: the one place `schema/change-markup` permits it.
//!
//! document-validation's "Change markup in or around a semantic unit is rejected" permits `ins` and
//! `del` inside a requirement or a scenario marked `modified`, **in a change's specification**, and
//! nowhere else. Where a document sits is not something a document states, and a conformance
//! fixture sits nowhere: `conformance.rs` validates every fixture loose. So the permitted case
//! cannot be a fixture. These cases validate committed fixtures' bytes at each placement instead,
//! which is the departure `references.rs` makes for the same reason.
//!
//! Everything drives the public API only: a scope is built with
//! [`htmlspec_validate::references::Scope::gather`], and its places declare nothing, because this
//! rule reads only where the document sits.

use std::path::{Path, PathBuf};

use htmlspec_assets::Store;
use htmlspec_parse::{parse_document, project};
use htmlspec_validate::references::{Declared, Placement, Scope};
use htmlspec_validate::{engine, rules, Report, Strictness};

/// The rule these cases are about.
const CHANGE_MARKUP: &str = "schema/change-markup";

/// Every placement, listed through a match with no wildcard, so a sixth variant fails to compile
/// here until its verdict is decided.
fn every_placement() -> [Placement; 5] {
    let decided = |placement: Placement| match placement {
        Placement::Loose
        | Placement::ChangeDocument
        | Placement::ChangeSpecification
        | Placement::AcceptedSpecification
        | Placement::RunbookDocument => placement,
    };
    [
        Placement::Loose,
        Placement::ChangeDocument,
        Placement::ChangeSpecification,
        Placement::AcceptedSpecification,
        Placement::RunbookDocument,
    ]
    .map(decided)
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/conformance")
        .join(name)
}

/// Validates the committed fixture `name` as a document sitting at `placement`.
fn validate_at(name: &str, placement: Placement) -> Report {
    let dir = fixture(name);
    let source = std::fs::read_to_string(dir.join("input.html"))
        .unwrap_or_else(|e| panic!("{name}: cannot read its input: {e}"));
    let outcome = parse_document(&source);
    let projection = project(&outcome.document);
    let scope = match placement {
        Placement::Loose => Scope::loose(),
        placement => Scope::gather(placement, |_| Declared::default()),
    };
    engine::run(
        &outcome,
        &projection,
        &Store::discover(&dir),
        &scope,
        rules::ALL,
        Strictness::Default,
    )
}

/// How many `schema/change-markup` diagnostics a report holds.
fn change_markup(report: &Report) -> usize {
    report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule.to_string() == CHANGE_MARKUP)
        .count()
}

/// Every other rule and location a report names, in order.
fn everything_else(report: &Report) -> Vec<String> {
    let mut found: Vec<String> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule.to_string() != CHANGE_MARKUP)
        .map(|diagnostic| format!("{} {:?}", diagnostic.rule, diagnostic.location))
        .collect();
    found.sort();
    found
}

/// A requirement marked `modified`, with a deleted phrase and an inserted one in its statement:
/// permitted in a change's specification and refused everywhere else, one diagnostic for each
/// element. Nothing else the document reports depends on where it sits.
#[test]
fn a_modified_requirement_may_carry_marks_only_in_a_change_specification() {
    let name = "invalid-schema-change-markup-in-a-modified-unit";
    let loose = everything_else(&validate_at(name, Placement::Loose));
    for placement in every_placement() {
        let report = validate_at(name, placement);
        let expected = if placement == Placement::ChangeSpecification {
            0
        } else {
            2
        };
        assert_eq!(
            change_markup(&report),
            expected,
            "{placement:?}: {:?}",
            report.diagnostics
        );
        assert_eq!(
            everything_else(&report),
            loose,
            "{placement:?} changed a rule other than {CHANGE_MARKUP}"
        );
    }
}

/// The same marks in a requirement marked `added`: refused at every placement, a change's
/// specification included, because a new requirement has no accepted text to mark against.
#[test]
fn an_added_requirement_may_carry_marks_nowhere() {
    let name = "invalid-schema-change-markup";
    let loose = everything_else(&validate_at(name, Placement::Loose));
    for placement in every_placement() {
        let report = validate_at(name, placement);
        assert_eq!(
            change_markup(&report),
            2,
            "{placement:?}: {:?}",
            report.diagnostics
        );
        assert_eq!(
            everything_else(&report),
            loose,
            "{placement:?} changed a rule other than {CHANGE_MARKUP}"
        );
    }
}

/// The same marks in prose outside every unit: reported nowhere, a change's specification and an
/// accepted specification alike.
#[test]
fn marks_outside_every_unit_are_reported_at_no_placement() {
    let name = "valid-change-markup-outside-a-unit";
    for placement in every_placement() {
        let report = validate_at(name, placement);
        assert_eq!(
            change_markup(&report),
            0,
            "{placement:?}: {:?}",
            report.diagnostics
        );
        assert!(
            report.is_accepted(),
            "{placement:?}: {:?}",
            report.diagnostics
        );
    }
}
