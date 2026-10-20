//! The conformance suite: tasks 8.1 through 8.4.
//!
//! A conformance fixture is a directory under `fixtures/conformance/` holding an
//! `input.html` and an `expected.json` of the form:
//!
//! ```json
//! { "valid": false, "rules": ["refs/unresolved"], "note": "why this fixture exists" }
//! ```
//!
//! A **valid** fixture declares `"valid": true` and no rules; the harness asserts the
//! document is accepted. An **invalid** fixture declares the rule identifiers it
//! violates, and the harness asserts *those exact rules* are reported — not merely
//! that something failed. A fixture that declared "invalid" and nothing more would
//! pass when the document broke for an entirely unrelated reason, which is the way a
//! conformance suite rots without anyone noticing.
//!
//! Everything here drives the crates' **public APIs only** (task 8.1), so the suite
//! tests the contract rather than this implementation, and could be pointed at an
//! independent one. Nothing reaches into a private module, and the harness has no
//! knowledge of how a rule is implemented — only of its stable identifier.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use htmlspec_assets::Store;
use htmlspec_validate::{engine, rules, Report, Strictness};

/// Where the conformance fixtures live.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/conformance")
}

/// One fixture's declared expectations.
#[derive(Debug)]
struct Expected {
    /// Whether the document must be accepted.
    ///
    /// Not the same as "reports nothing". A fixture pinning a `review/*` rule reports
    /// a warning *and* is accepted, because a review marker never gates work — so
    /// acceptance and the rule list are declared separately. A harness that inferred
    /// one from the other could not cover a warning rule at all.
    valid: bool,
    /// The rule identifiers the document violates.
    ///
    /// Non-empty for an accepted fixture only when every rule listed is a warning.
    rules: Vec<String>,
    /// Why this fixture exists, in one line. Read by a person, not by the harness.
    note: String,
}

/// Every fixture directory, sorted.
fn fixtures() -> Vec<PathBuf> {
    let root = fixture_root();
    let mut found: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {root:?}: {e}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    found.sort();
    assert!(
        !found.is_empty(),
        "no conformance fixtures found in {root:?}"
    );
    found
}

fn name_of(dir: &Path) -> String {
    dir.file_name()
        .expect("a named directory")
        .to_string_lossy()
        .into_owned()
}

fn read_expected(dir: &Path) -> Expected {
    let path = dir.join("expected.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"));
    let value: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?} is not valid JSON: {e}"));

    let valid = value
        .get("valid")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_else(|| panic!("{path:?} has no boolean `valid`"));
    let rules = value
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(|v| {
                    v.as_str()
                        .unwrap_or_else(|| panic!("{path:?} has a non-string rule"))
                        .to_owned()
                })
                .collect()
        })
        .unwrap_or_default();
    let note = value
        .get("note")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("{path:?} has no `note` saying why the fixture exists"))
        .to_owned();

    Expected { valid, rules, note }
}

fn read_input(dir: &Path) -> String {
    let path = dir.join("input.html");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"))
}

/// Validates one document through the public API, with no asset store beside it.
///
/// The overwhelming majority of fixtures are in this case: their asset references are
/// symbolic, present so the parser sees a realistic document, and no `assets/`
/// directory has ever sat beside them. See [`validate_in`] for a fixture that carries
/// its own store.
fn validate(source: &str, strictness: Strictness) -> Report {
    validate_in(source, &Store::Absent, strictness)
}

/// Validates one document against a given asset store.
fn validate_in(source: &str, store: &Store, strictness: Strictness) -> Report {
    let outcome = htmlspec_parse::parse_document(source);
    let projection = htmlspec_parse::project(&outcome.document);
    engine::run(
        &outcome,
        &projection,
        store,
        &htmlspec_validate::Scope::loose(),
        rules::ALL,
        strictness,
    )
}

/// The asset store a fixture carries, if any.
///
/// A fixture directory holding an `assets/` subdirectory is a fixture *inside a
/// project with a store*, which is the only way the `asset/*` rules come into scope.
/// This is how those rules get conformance coverage without a harness flag: the scope
/// is decided by the same mechanism in the suite as in the CLI, so a fixture cannot
/// declare itself in or out of scope, and neither can the harness.
fn store_of(dir: &Path) -> Store {
    Store::discover(dir)
}

/// The rule identifiers a report mentions, whatever their severity.
fn reported_rules(report: &Report) -> BTreeSet<String> {
    report
        .diagnostics
        .iter()
        .map(|d| d.rule.to_string())
        .collect()
}

fn describe(report: &Report) -> String {
    let mut out = String::new();
    if let Some(curtailment) = &report.curtailed {
        out.push_str(&format!("  curtailed: {curtailment}\n"));
    }
    for diagnostic in &report.diagnostics {
        out.push_str(&format!("  {diagnostic}\n"));
    }
    if out.is_empty() {
        out.push_str("  (no diagnostics)\n");
    }
    out
}

// -----------------------------------------------------------------------------
// Task 8.2: every fixture asserts the exact rule it declares
// -----------------------------------------------------------------------------

/// Every fixture's declared acceptance holds.
#[test]
fn every_fixture_is_accepted_exactly_when_it_declares_it_is() {
    let mut failures = Vec::new();
    for dir in fixtures() {
        let expected = read_expected(&dir);
        let report = validate_in(&read_input(&dir), &store_of(&dir), Strictness::Default);
        if report.is_accepted() != expected.valid {
            failures.push(format!(
                "{} ({}) declares valid={} and was {}:\n{}",
                name_of(&dir),
                expected.note,
                expected.valid,
                if report.is_accepted() {
                    "accepted"
                } else {
                    "rejected"
                },
                describe(&report)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The load-bearing assertion of the suite: a fixture reports *the rules it declares*,
/// not merely something.
///
/// Checked for accepted and rejected fixtures alike. An accepted fixture may still
/// declare rules — a `review/*` warning is reported and does not gate work — and its
/// rule is asserted the same way, which is what gives the warning namespace real
/// coverage rather than a note saying it is hard to test.
#[test]
fn every_fixture_reports_exactly_the_rules_it_declares() {
    let mut failures = Vec::new();
    for dir in fixtures() {
        let expected = read_expected(&dir);
        let name = name_of(&dir);
        if !expected.valid {
            assert!(
                !expected.rules.is_empty(),
                "{name} is rejected and declares no rule, so it would pass for any failure"
            );
        }
        if expected.rules.is_empty() {
            continue;
        }

        let report = validate_in(&read_input(&dir), &store_of(&dir), Strictness::Default);
        let reported = reported_rules(&report);
        for rule in &expected.rules {
            if !reported.contains(rule) {
                failures.push(format!(
                    "{name} ({}) declares {rule} and it was not reported:\n{}",
                    expected.note,
                    describe(&report)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A summary quoting a template's placeholder reports nothing at all.
///
/// **Named rather than driven by the fixture set, and the reason is a measurement.**
/// `valid-summary-quoting-the-placeholder` declares no rules, and
/// [`every_fixture_reports_exactly_the_rules_it_declares`] `continue`s on an empty rule list
/// (line 213) — it asserts each declared rule *is* reported and never that others are absent. So
/// a fixture whose whole purpose is that nothing fires cannot state that through the declaration
/// alone. Measured rather than reasoned: with the comparison reverted to a substring search, that
/// fixture reports `review/example-content` and the whole suite still passed.
///
/// What this pins is the decision the placeholder-summary rule turns on. A substring search would
/// report a document whose author quoted the template's wording inside their own statement — which
/// includes every document in this project that discusses the rule, and the change that introduced
/// it. A check that fires on documents about itself is one an agent learns to ignore.
///
/// Named because the property is about one document's content and not about the fixture set. The
/// general version — no accepted fixture reports an undeclared rule — is a different and larger
/// change: `valid-minimal` reports `review/example-content` seven times with an empty rule list
/// today, and tightening the harness means deciding what all 50 fixtures should declare.
#[test]
fn a_summary_quoting_the_placeholder_reports_nothing() {
    let name = "valid-summary-quoting-the-placeholder";
    let dir = fixture_root().join(name);
    assert!(
        dir.is_dir(),
        "{name} is gone, so the decision it pins is unpinned"
    );

    let expected = read_expected(&dir);
    assert!(
        expected.rules.is_empty() && expected.valid,
        "{name} declares {:?}; this test exists because it declares nothing",
        expected.rules
    );

    let input = read_input(&dir);
    // The fixture has to actually contain a placeholder, or it pins nothing. Read from the
    // templates, so a reworded template moves this rather than leaving it asserting a stale
    // string.
    assert!(
        htmlspec_assets::example_placeholders()
            .iter()
            .any(|p| input.contains(p.as_str())),
        "{name} contains no template's placeholder paragraph, so a substring search would not \
         report it and this fixture pins nothing"
    );

    let report = validate_in(&input, &store_of(&dir), Strictness::Default);
    assert!(
        report.diagnostics.is_empty(),
        "{name} reports a diagnostic, and it exists to show that a summary quoting a \
         placeholder inside the author's own statement reports none:\n{}",
        describe(&report)
    );
}

/// A fixture declaring only warning rules is accepted by default and rejected under
/// strict validation — except for `review/*`, which strict mode never promotes.
///
/// This is where the two severity exceptions are asserted from the outside: the
/// `safety/*` floor and the `review/*` ceiling are both contract, and a rule engine
/// that lost either would still pass every other test in this file.
#[test]
fn strict_mode_promotes_every_warning_except_a_review_marker() {
    for dir in fixtures() {
        let expected = read_expected(&dir);
        let name = name_of(&dir);
        if !expected.valid || expected.rules.is_empty() {
            continue;
        }

        let strict = validate_in(&read_input(&dir), &store_of(&dir), Strictness::Strict);
        let review_only = expected
            .rules
            .iter()
            .all(|rule| rule.starts_with("review/"));

        if review_only {
            assert!(
                strict.is_accepted(),
                "{name} declares only review markers and strict mode gated it, which imposes \
                 the phase gate the dependency model exists to avoid:\n{}",
                describe(&strict)
            );
        } else {
            assert!(
                !strict.is_accepted(),
                "{name} declares a promotable warning and strict mode did not gate it:\n{}",
                describe(&strict)
            );
        }
    }
}

/// A fixture may not declare a rule the validator does not have.
///
/// Without this a typo in a fixture — `refs/unresolvd` — would make the assertion
/// above vacuous for that entry, since a rule that does not exist can never be
/// reported and the fixture would look thorough while asserting nothing.
#[test]
fn no_fixture_declares_a_rule_the_validator_does_not_have() {
    let known = known_rules();
    let mut unknown = Vec::new();
    for dir in fixtures() {
        let name = name_of(&dir);
        for rule in read_expected(&dir).rules {
            if !known.contains(&rule) {
                unknown.push(format!("{name} declares {rule}"));
            }
        }
    }
    assert!(
        unknown.is_empty(),
        "a fixture declares a rule the validator does not have: {}\nKnown rules: {}",
        unknown.join(", "),
        known.iter().cloned().collect::<Vec<String>>().join(", ")
    );
}

// -----------------------------------------------------------------------------
// Task 8.3: coverage
// -----------------------------------------------------------------------------

/// Every rule the validator can report, read from the rule modules' public constants.
///
/// Listed here rather than discovered, because Rust has no reflection over constants
/// and a rule absent from this list is a rule with no coverage requirement. That is
/// the trade: the list must be maintained, and `no_fixture_declares_a_rule_the_
/// validator_does_not_have` plus the coverage test below make an unmaintained one
/// visible rather than silent.
fn known_rules() -> BTreeSet<String> {
    [
        // schema/*
        rules::schema::INVALID,
        rules::schema::PROJECTION,
        rules::schema::UNAVAILABLE,
        rules::relations::DUPLICATE_ID,
        rules::relations::PREFIX_MISMATCH,
        rules::relations::MISSING_SUCCESSOR,
        rules::relations::MISSING_REASON,
        rules::relations::DEVIATION_NOT_DONE,
        rules::relations::MISSING_UNCHECKED,
        rules::relations::MISSING_MITIGATION,
        rules::relations::WEAK_MODAL,
        rules::renames::OUTSIDE_REQUIREMENT,
        rules::renames::ON_ADDED,
        rules::renames::ON_REMOVED,
        rules::renames::REPEATED,
        rules::renames::EMPTY,
        rules::renames::TO_SAME_NAME,
        rules::change_markup::CHANGE_MARKUP,
        // kind/*
        rules::kinds::TASK_IN_SPECIFICATION,
        rules::kinds::NO_TASKS,
        // refs/*
        rules::refs::UNRESOLVED,
        rules::refs::BROKEN_LINK,
        rules::refs::REMOVED_TARGET,
        // safety/*
        rules::safety::REMOTE_RESOURCE,
        rules::safety::CONCEALED_REQUEST,
        rules::safety::REMOTE_CHART_DATA,
        rules::executable::LIVE_EXAMPLE,
        rules::executable::UNAPPROVED_SCRIPT,
        rules::executable::EVENT_HANDLER,
        rules::executable::EXECUTABLE_URL,
        // asset/*
        rules::assets::UNRESOLVED,
        rules::assets::MISMATCH,
        rules::topology::DUPLICATE,
        // viz/*
        rules::viz::CHART_NOT_JSON,
        rules::viz::CHART_INVALID,
        rules::viz::DIAGRAM_EMPTY,
        rules::viz::DIAGRAM_UNRECOGNIZED,
        rules::viz::SCHEMA_UNAVAILABLE,
        // a11y/*
        rules::a11y::SCRIPT_SUPPLIED_CONTENT,
        rules::a11y::FIGURE_WITHOUT_CAPTION,
        rules::a11y::FIGURE_WITHOUT_FALLBACK,
        rules::a11y::HIDDEN_FIGURE_SOURCE,
        // review/*
        rules::review::STALE_REFERENCE,
        rules::review::STALE_VERIFICATION,
        rules::review::UNVERIFIED_REQUIREMENT,
        rules::review::EXAMPLE_CONTENT,
        rules::runs::UNRECORDED,
        // parse/*
        engine::PARSE_RECOVERY,
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

/// Rules deliberately without an invalid fixture, each with the reason.
///
/// A rule here is one whose violation cannot be expressed as a document, so a fixture
/// for it is not merely missing — it is impossible. Every entry states why, and the
/// list is short on purpose: it is the escape hatch, and an escape hatch with vague
/// entries becomes the place uncovered rules go to hide.
const UNFIXTURABLE: &[(&str, &str)] = &[
    (
        "schema/unavailable",
        "reports that this project's own schema failed to compile, which is a defect \
         in the toolchain and not something a document can cause. \
         `the_published_schema_compiles` in the validator covers it.",
    ),
    (
        "asset/duplicate",
        "takes a project as its subject rather than a document, so no document can \
         violate it: the same document is conforming or not depending only on what sits \
         in the directory around it. Its fixtures are whole projects, in \
         conformance/tests/topology.rs, and `scripts/check` runs the rule over this one.",
    ),
    (
        "viz/schema-unavailable",
        "reports that the vendored Vega-Lite schema failed to preprocess, which again \
         no document can cause. `the_vendored_schema_does_not_compile_unmodified` \
         covers the underlying defect.",
    ),
    (
        "review/unrecorded-requirement",
        "takes a runbook and its run records *together* as its subject, so no document can \
         violate it: the same record is conforming or not depending only on what the runbook \
         beside it requires, and neither file holds both halves of the fact. One step stronger \
         than asset/duplicate, whose subject is at least a directory a document sits in. Its \
         fixtures are runbook-and-records pairs, in conformance/tests/runs.rs; the document half \
         is pinned by valid-runbook-run-record and \
         valid-runbook-run-record-missing-its-verification, which pin the record's shape and the \
         measured silence that motivates the rule.",
    ),
];

/// Task 8.3: every rule has both a valid and an invalid fixture.
///
/// "Both" matters. A rule with only an invalid fixture is not shown to leave a
/// conforming document alone, which is how an over-strict rule ships: it correctly
/// rejects what it should and also rejects things nobody tested.
#[test]
fn every_rule_has_both_a_valid_and_an_invalid_fixture() {
    let unfixturable: BTreeSet<&str> = UNFIXTURABLE.iter().map(|(rule, _)| *rule).collect();

    // Which fixtures declare each rule, and how many clean fixtures exist at all.
    // A fixture declaring a rule covers it whether or not the document is accepted:
    // a `review/*` warning is reported and accepted, and it is still coverage.
    let mut declared_by: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut clean_fixtures: Vec<String> = Vec::new();
    for dir in fixtures() {
        let name = name_of(&dir);
        let expected = read_expected(&dir);
        if expected.valid && expected.rules.is_empty() {
            clean_fixtures.push(name.clone());
        }
        for rule in expected.rules {
            declared_by.entry(rule).or_default().push(name.clone());
        }
    }

    assert!(
        !clean_fixtures.is_empty(),
        "there is no fixture that reports nothing, so no rule is shown to leave a conforming \
         document alone"
    );

    let uncovered: Vec<String> = known_rules()
        .into_iter()
        .filter(|rule| !unfixturable.contains(rule.as_str()))
        .filter(|rule| !declared_by.contains_key(rule))
        .collect();

    assert!(
        uncovered.is_empty(),
        "these rules have no fixture that pins them: {}\n\nAdd one under \
         fixtures/conformance/, or record the rule in UNFIXTURABLE with the reason a document \
         cannot violate it.",
        uncovered.join(", ")
    );
}

/// The valid side of the coverage requirement, asserted where it is meaningful.
///
/// Pairing each rule with a *dedicated* valid fixture would mean thirty near-identical
/// documents. Instead every valid fixture is run against the whole rule set, so each
/// one demonstrates that every rule leaves a conforming document alone — which is the
/// property the pairing was for, and a stronger statement than a per-rule pairing
/// would make.
#[test]
fn every_valid_fixture_is_accepted_by_the_whole_rule_set() {
    let valid: Vec<PathBuf> = fixtures()
        .into_iter()
        .filter(|dir| {
            let expected = read_expected(dir);
            expected.valid && expected.rules.is_empty()
        })
        .collect();
    assert!(
        valid.len() >= 3,
        "only {} valid fixture(s); the suite needs enough conforming documents that a \
         rule cannot pass by rejecting everything",
        valid.len()
    );

    for dir in valid {
        let report = validate_in(&read_input(&dir), &store_of(&dir), Strictness::Default);
        assert!(
            report.is_accepted(),
            "{} is a valid fixture and some rule rejected it:\n{}",
            name_of(&dir),
            describe(&report)
        );
    }
}

/// An entry in `UNFIXTURABLE` must name a rule that exists, or the exemption is
/// silently covering nothing while looking deliberate.
#[test]
fn every_unfixturable_entry_names_a_real_rule_and_states_a_reason() {
    let known = known_rules();
    for (rule, reason) in UNFIXTURABLE {
        assert!(
            known.contains(*rule),
            "UNFIXTURABLE names {rule}, which is not a rule the validator has"
        );
        assert!(
            reason.len() > 40,
            "the exemption for {rule} does not explain itself: {reason:?}"
        );
    }
}

/// Task 8.3, second half: every projection rule has a mapping fixture.
///
/// The mapping fixtures live in the parse crate's own test suite, which is where they
/// belong — they pin the HTML-to-model projection rather than the rules. This asserts
/// the suite as a whole covers them, so a reader of the conformance suite is not left
/// assuming projection coverage that lives elsewhere.
#[test]
fn the_projection_mapping_fixtures_exist_and_cover_the_specification() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/projection");
    let count = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {root:?}: {e}"))
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .count();
    assert!(
        count >= 20,
        "only {count} projection mapping fixtures; \
         `every_projection_rule_has_a_mapping_fixture` in htmlspec-parse asserts the \
         coverage itself, and this asserts the fixtures have not been deleted"
    );
}

// -----------------------------------------------------------------------------
// Task 8.4: every fixture formats idempotently
// -----------------------------------------------------------------------------

/// Task 8.4: every fixture is formatted twice and the output is byte-identical.
///
/// Run over *every* fixture, valid and invalid alike. An invalid document still has
/// one unambiguous tree, and the formatter canonicalizes it without repair — so the
/// fixed-point property must hold there too, and the invalid fixtures are the awkward
/// documents most likely to break it.
#[test]
fn every_fixture_formats_to_a_fixed_point() {
    let mut failures = Vec::new();
    for dir in fixtures() {
        let name = name_of(&dir);
        let source = read_input(&dir);

        let once = match htmlspec_format::format(&source) {
            Ok(formatted) => formatted,
            Err(error) => {
                // A fixture that pins a parse recovery is legitimately unformattable:
                // its markup has no single meaning, and the formatter refuses rather
                // than choosing one.
                let expected = read_expected(&dir);
                if expected.rules.iter().any(|rule| rule.starts_with("parse/")) {
                    continue;
                }
                failures.push(format!("{name} was refused by the formatter: {error}"));
                continue;
            }
        };
        match htmlspec_format::format(&once) {
            Ok(twice) if twice == once => {}
            Ok(twice) => failures.push(format!(
                "{name} is not a fixed point; the second pass differs:\n--- first ---\n{once}\n\
                 --- second ---\n{twice}"
            )),
            Err(error) => failures.push(format!(
                "{name}: the formatter refused its own output, which cannot be correct: {error}"
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// Formatting preserves what a fixture is about: its diagnostics do not change.
///
/// Stronger than the projection-equality property the format crate asserts, and aimed
/// at a different failure: a formatter that dropped an attribute could still project an
/// equal model if the attribute were one the model ignores, while changing what the
/// safety rules see.
#[test]
fn formatting_a_fixture_does_not_change_what_the_validator_reports() {
    let mut failures = Vec::new();
    for dir in fixtures() {
        let name = name_of(&dir);
        let source = read_input(&dir);
        let Ok(formatted) = htmlspec_format::format(&source) else {
            continue;
        };

        let store = store_of(&dir);
        let before = reported_rules(&validate_in(&source, &store, Strictness::Default));
        let after = reported_rules(&validate_in(&formatted, &store, Strictness::Default));
        if before != after {
            failures.push(format!(
                "{name}: formatting changed the diagnostics.\n  before: {:?}\n  after:  {:?}",
                before, after
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// -----------------------------------------------------------------------------
// The asset rules' scope, asserted from outside the validator
// -----------------------------------------------------------------------------

/// Exactly the `asset/*` fixtures carry a store, and no other fixture does.
///
/// This is the scoping decision stated as a checked fact. The `asset/*` rules fire only
/// where a store is present, so a fixture acquiring one by accident would silently come
/// into scope for rules it does not declare — and a fixture *losing* one would silently
/// stop testing them while still passing, which is the worse direction.
///
/// The fixtures in scope are named rather than detected, and matched exactly. Inferring
/// the set from a substring of the directory name would be wrong in both directions:
/// `valid-local-asset-reference` predates this change and carries no store, and a new
/// fixture could carry one without "asset" in its name.
const CARRIES_A_STORE: &[&str] = &[
    "valid-asset-reference-matches-stored-bytes",
    "invalid-asset-unresolved",
    "invalid-asset-mismatch",
    "invalid-asset-stale-stylesheet-reference",
];

#[test]
fn only_the_asset_fixtures_carry_a_store() {
    for dir in fixtures() {
        let name = name_of(&dir);
        let declares_asset_rule = read_expected(&dir)
            .rules
            .iter()
            .any(|rule| rule.starts_with("asset/"));
        let carries_store = store_of(&dir).is_present();

        assert_eq!(
            carries_store,
            CARRIES_A_STORE.contains(&name.as_str()),
            "{name} {} a store, which disagrees with CARRIES_A_STORE. A fixture that gains \
             one comes into scope for rules it does not declare; one that loses it stops \
             testing the rules it does declare, and still passes",
            if carries_store { "has" } else { "lacks" }
        );

        // Declaring an asset rule usually requires a store, or the rule is out of scope and
        // the fixture asserts nothing. The converse does not hold: a valid fixture carries a
        // store precisely to show the accepting side.
        //
        // One exception, and it is the point of the fixture rather than a gap: an *ascending*
        // reference with no store above it is reported. That case cannot carry a store,
        // because the absence of one is the violation. So the requirement here is a store or
        // an ascending reference — never neither.
        if declares_asset_rule {
            let ascends = read_input(&dir)
                .lines()
                .any(|line| line.contains("=\"..") || line.contains("/.."));
            assert!(
                carries_store || ascends,
                "{name} declares an asset/* rule, carries no store, and has no ascending \
                 reference — so the rule is out of scope and the fixture asserts nothing"
            );
        }
    }
}

/// The 37 pre-existing fixtures reference their assets in the unaddressed form, and
/// must keep doing so.
///
/// The generator's base was frozen to that form deliberately: `render_template` now
/// emits content addresses, and letting the base follow it would rewrite every committed
/// input. This asserts the freeze holds, so the drift is a checked fact rather than a
/// trap the next contributor to run the generator discovers.
#[test]
fn only_the_asset_fixtures_reference_an_addressed_path() {
    for dir in fixtures() {
        let name = name_of(&dir);
        let input = read_input(&dir);
        let addressed = input.contains("assets/sha256-");
        // An *ascending* addressed reference is the one storeless case that is deliberate:
        // its whole subject is a reference whose store is missing. `ASCENDS_WITHOUT_A_STORE`
        // names it, so a fixture cannot acquire the shape by accident and pass this.
        let expected = CARRIES_A_STORE.contains(&name.as_str())
            || ASCENDS_WITHOUT_A_STORE.contains(&name.as_str());
        assert_eq!(
            addressed,
            expected,
            "{name} {} an addressed asset reference, which disagrees with whether it carries \
             a store. An addressed reference in a storeless fixture would resolve to nothing \
             the moment one appeared beside it — unless it ascends, in which case its absence \
             is the violation under test",
            if addressed { "has" } else { "lacks" }
        );
    }
}

/// The fixtures whose addressed reference ascends and whose store is absent on purpose.
///
/// Named rather than detected, so a fixture that acquires an ascending reference by accident
/// fails the invariant above instead of being waved through by a pattern match.
const ASCENDS_WITHOUT_A_STORE: &[&str] = &["invalid-asset-orphaned-ascending-reference"];

/// Both faults are covered, and by *separate* fixtures.
///
/// The specs require the two be distinguishable diagnostics because their causes differ.
/// One fixture declaring both would satisfy the coverage test above while proving
/// nothing about distinguishability.
#[test]
fn the_two_asset_faults_are_pinned_separately() {
    let mut by_rule: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for dir in fixtures() {
        for rule in read_expected(&dir).rules {
            if rule.starts_with("asset/") {
                by_rule.entry(rule).or_default().push(name_of(&dir));
            }
        }
    }
    for rule in ["asset/unresolved", "asset/mismatch"] {
        let fixtures = by_rule.get(rule).map(Vec::as_slice).unwrap_or_default();
        assert!(!fixtures.is_empty(), "no fixture pins {rule}");
        for fixture in fixtures {
            let other = read_expected(&fixture_root().join(fixture)).rules;
            assert_eq!(
                other.len(),
                1,
                "{fixture} declares {other:?}; a fixture pinning {rule} alongside the other \
                 asset fault does not show the two are distinguishable"
            );
        }
    }
}

// -----------------------------------------------------------------------------
// The suite's own hygiene
// -----------------------------------------------------------------------------

/// A fixture must declare its expectations consistently.
#[test]
fn every_fixture_is_well_formed() {
    for dir in fixtures() {
        let name = name_of(&dir);
        assert!(dir.join("input.html").exists(), "{name} has no input.html");
        let expected = read_expected(&dir);
        assert!(
            !expected.note.trim().is_empty(),
            "{name} has an empty note, so nothing records why it exists"
        );
        // An accepted fixture may declare rules, but only warning ones: an error
        // fails validation by definition, so declaring one alongside acceptance is a
        // contradiction rather than a subtlety.
        if expected.valid {
            for rule in &expected.rules {
                assert!(
                    rule.starts_with("review/"),
                    "{name} is accepted and declares {rule}, which is not a warning namespace"
                );
            }
        }
    }
}

/// The documents the toolchain itself produces and ships are conformant.
///
/// Not fixtures, and checked here because the conformance suite is where "does this
/// project's own output satisfy its own contract?" belongs. The template is what every
/// contributor's first document is made from, and the dogfood documents are what the
/// vocabulary was designed against.
#[test]
fn the_template_and_the_dogfood_documents_are_conformant() {
    let template = htmlspec_assets::render_template("A subject", "assets");
    let report = validate(&template, Strictness::Default);
    assert!(
        report.is_accepted(),
        "the template init writes is not conformant:\n{}",
        describe(&report)
    );

    let dogfood = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/dogfood");
    let mut seen = 0;
    for entry in std::fs::read_dir(&dogfood).expect("the dogfood fixtures exist") {
        let path = entry.expect("readable entry").path();
        if path.extension().is_none_or(|e| e != "html") {
            continue;
        }
        seen += 1;
        let source = std::fs::read_to_string(&path).expect("readable");
        let report = validate(&source, Strictness::Default);
        assert!(
            report.is_accepted(),
            "{} is not conformant:\n{}",
            path.display(),
            describe(&report)
        );
    }
    assert_eq!(seen, 6, "expected the six dogfood documents");
}

// -----------------------------------------------------------------------------
// The contract's versioning: a later version reports no error an earlier one did not
// -----------------------------------------------------------------------------

/// The contract version `source` declares, from its `htmlspec-version` meta element.
fn declared_version(source: &str) -> Option<String> {
    let end = source.find(r#"" name="htmlspec-version">"#)?;
    let start = source[..end].rfind(r#"content=""#)? + r#"content=""#.len();
    Some(source[start..end].to_owned())
}

/// `source` redeclared at `to`, in its metadata and on its page, as a contributor would edit it.
fn redeclared(source: &str, from: &str, to: &str) -> String {
    source
        .replace(
            &format!(r#"<meta content="{from}" name="htmlspec-version">"#),
            &format!(r#"<meta content="{to}" name="htmlspec-version">"#),
        )
        .replace(&format!("<dd>{from}</dd>"), &format!("<dd>{to}</dd>"))
}

/// The errors a report holds, each as the line it prints, so two reports compare by what a
/// contributor reads.
fn errors_of(report: &Report) -> BTreeSet<String> {
    report.errors().map(|error| error.to_string()).collect()
}

/// Every conformance fixture, declared at each version the schema accepts, reports no error under
/// the newest version that it does not report under the earlier one.
///
/// The promise document-schema's versioning states for a major from 1 on, and the reason 1.0.0
/// keeps the 0.x versions: a document written under any of them keeps its verdict. Run over every
/// fixture, valid and invalid, because an invalid one shows an error staying the same error.
#[test]
fn no_document_gains_an_error_under_the_newest_contract_version() {
    let versions = htmlspec_schema::facts::contract_versions();
    let newest = versions
        .last()
        .expect("the schema accepts a version")
        .clone();
    assert_eq!(newest, htmlspec_schema::CONTRACT_VERSION);
    let mut failures = Vec::new();
    let mut compared = 0;
    for dir in fixtures() {
        let name = name_of(&dir);
        let source = read_input(&dir);
        let Some(declared) = declared_version(&source) else {
            continue;
        };
        let store = store_of(&dir);
        let at = |version: &str| {
            errors_of(&validate_in(
                &redeclared(&source, &declared, version),
                &store,
                Strictness::Default,
            ))
        };
        let under_newest = at(&newest);
        for version in &versions {
            compared += 1;
            let gained: Vec<_> = under_newest.difference(&at(version)).cloned().collect();
            if !gained.is_empty() {
                failures.push(format!(
                    "{name}, declared {version} then {newest}: {gained:?}"
                ));
            }
        }
    }
    assert!(compared > 0, "no fixture declares a contract version");
    assert!(
        failures.is_empty(),
        "a document gained an error under {newest}:\n{}",
        failures.join("\n")
    );
}
