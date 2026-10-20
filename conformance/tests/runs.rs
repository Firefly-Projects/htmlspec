//! The runbook-and-records fixtures for `review/unrecorded-requirement`.
//!
//! A conformance fixture in `conformance.rs` is a *document*, and the harness there asserts what the
//! validator reports about it. This rule has no document to attach to, and the reason is one step
//! stronger than `asset/duplicate`'s: duplication is a fact about the directory *around* a document,
//! while an unrecorded requirement is a fact about **two documents at once**. The runbook's
//! specification states the obligation and the run's record states the outcome, so neither file holds
//! both halves — the same record is conforming or not depending on what the runbook beside it
//! requires.
//!
//! So its fixtures are pairs: a set of requirement identifiers standing for the runbook, and a set
//! of records standing for its runs. Built here rather than committed, which is the same departure
//! `topology.rs` makes and for a related reason — a committed fixture pinning a non-conforming run
//! would be a document `scripts/check` walks and `self-runbooks` would have opinions about.
//!
//! Everything drives the public API only, exactly as the other two suites do: the subject is
//! constructed through [`htmlspec_validate::rules::runs`]'s own public types, with no reach into a
//! private module.
//!
//! **The document half of this rule's coverage is in `fixtures/conformance/`**, as
//! `valid-runbook-run-record` and `valid-runbook-run-record-missing-its-verification`. Those two
//! pin what validation says about a record — and what it says about the second one is *nothing*,
//! which is the measurement this rule exists because of.

use std::collections::BTreeSet;

use htmlspec_validate::rules::runs::{conformance, RecordedRun, RunbookUnderReview};

/// One runbook-and-records fixture, and what the rule must report about it.
struct RunbookFixture {
    /// Why this fixture exists, in one line. Read by a person, not by the harness.
    note: &'static str,
    /// How many unrecorded requirements the rule must report.
    ///
    /// A count rather than a boolean, because the rule reports one diagnostic per (run, requirement)
    /// pair and a fixture with two short runs must not pass by producing one.
    unrecorded: usize,
    /// The requirement identifiers the runbook's specification states.
    required: &'static [&'static str],
    /// Each run: its record's name, and the requirements it recorded an outcome for.
    ///
    /// **A verification declaring the "not run" outcome is absent from the second half**, which is
    /// the line the caller draws before the rule sees anything: a record full of not-yet-run entries
    /// has recorded nothing, and that is exactly what a freshly started run looks like.
    runs: &'static [(&'static str, &'static [&'static str])],
}

const FIXTURES: &[(&str, RunbookFixture)] = &[
    (
        "a-run-that-recorded-everything",
        RunbookFixture {
            note: "the arrangement a runbook wants: every requirement it states has an outcome \
                   recorded against it in the run's record",
            unrecorded: 0,
            required: &["req-version-before-regenerate", "req-the-changelog-entry"],
            runs: &[(
                "0-14-0.html",
                &["req-version-before-regenerate", "req-the-changelog-entry"],
            )],
        },
    ),
    (
        "a-run-short-one-requirement",
        RunbookFixture {
            note: "a run that recorded one of two obligations, which is the case the rule exists \
                   for and which no per-document rule can decide",
            unrecorded: 1,
            required: &["req-version-before-regenerate", "req-the-changelog-entry"],
            runs: &[("0-14-0.html", &["req-version-before-regenerate"])],
        },
    ),
    (
        "a-run-that-has-only-just-started",
        RunbookFixture {
            note: "the state the operation starting a run leaves behind: one not-yet-run entry \
                   per requirement, so the record enumerates what it owes and has recorded none \
                   of it",
            unrecorded: 2,
            required: &["req-version-before-regenerate", "req-the-changelog-entry"],
            runs: &[("0-14-0.html", &[])],
        },
    ),
    (
        "a-runbook-nobody-has-run",
        RunbookFixture {
            note: "a procedure with no records violates nothing; a rule firing here would fire \
                   on every runbook from the moment it was created",
            unrecorded: 0,
            required: &["req-version-before-regenerate"],
            runs: &[],
        },
    ),
    (
        "a-runbook-stating-nothing",
        RunbookFixture {
            note: "a runbook that states no requirement of its runs has runs nothing checks, \
                   which is a fact about that runbook rather than a fault in it",
            unrecorded: 0,
            required: &[],
            runs: &[("batch-a.html", &[])],
        },
    ),
    (
        "two-runs-each-short",
        RunbookFixture {
            note: "two short runs are two reports per requirement, so a reader fixing one is not \
                   told the runbook is now clean",
            unrecorded: 2,
            required: &["req-version-before-regenerate"],
            runs: &[("0-13-0.html", &[]), ("0-14-0.html", &[])],
        },
    ),
    (
        "a-run-that-recorded-more-than-it-owed",
        RunbookFixture {
            note: "a run that recorded an outcome for something the runbook does not require is \
                   not at fault, so the comparison runs in one direction only",
            unrecorded: 0,
            required: &["req-version-before-regenerate"],
            runs: &[(
                "0-14-0.html",
                &["req-version-before-regenerate", "req-nobody-asked-for-this"],
            )],
        },
    ),
];

fn subject(fixture: &RunbookFixture) -> (Vec<String>, Vec<RecordedRun>) {
    let required: Vec<String> = fixture.required.iter().map(|id| (*id).to_owned()).collect();
    let runs: Vec<RecordedRun> = fixture
        .runs
        .iter()
        .map(|(record, recorded)| RecordedRun {
            record: (*record).to_owned(),
            recorded: recorded
                .iter()
                .map(|id| (*id).to_owned())
                .collect::<BTreeSet<String>>(),
        })
        .collect();
    (required, runs)
}

/// Every fixture reports exactly the number of unrecorded requirements it declares.
#[test]
fn every_runbook_fixture_reports_what_it_declares() {
    let mut failures = Vec::new();
    for (name, fixture) in FIXTURES {
        let (required, runs) = subject(fixture);
        let found = conformance(&RunbookUnderReview {
            name,
            required: &required,
            runs: &runs,
        });
        if found.len() != fixture.unrecorded {
            failures.push(format!(
                "{name} ({}) declares {} unrecorded requirement(s) and reported {}:\n{}",
                fixture.note,
                fixture.unrecorded,
                found.len(),
                found
                    .iter()
                    .map(|d| format!("  {d}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// No fixture's report gates, at any severity the rule can carry.
///
/// The `review/*` ceiling asserted from outside the validator, which is where the other two suites
/// assert the `safety/*` floor. `runbook-execution` states it as an obligation: the report "SHALL NOT
/// fail validation and SHALL NOT be promotable to a failure", because a run under way legitimately
/// has requirements it has not reached.
#[test]
fn no_report_about_a_run_ever_gates() {
    for (name, fixture) in FIXTURES {
        let (required, runs) = subject(fixture);
        for diagnostic in conformance(&RunbookUnderReview {
            name,
            required: &required,
            runs: &runs,
        }) {
            assert!(
                !diagnostic.is_error(),
                "{name} produced a gating diagnostic, which would fail a run that is under way: \
                 {diagnostic}"
            );
        }
    }
}

/// A report names both the run and the requirement.
///
/// The load-bearing half. A report naming only the run leaves a reader to open it and compare against
/// the specification; one naming only the requirement leaves them to find which of forty records is
/// short. Both halves are needed precisely because the rule's subject is two documents.
#[test]
fn a_report_names_both_the_run_and_the_requirement() {
    let (name, fixture) = FIXTURES
        .iter()
        .find(|(name, _)| *name == "a-run-short-one-requirement")
        .expect("the fixture exists");
    let (required, runs) = subject(fixture);
    let found = conformance(&RunbookUnderReview {
        name,
        required: &required,
        runs: &runs,
    });

    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].location.unit_id.as_deref(),
        Some("req-the-changelog-entry"),
        "the diagnostic does not address the requirement a reader would open"
    );
    assert!(
        found[0].message.contains("0-14-0.html"),
        "the report omits which run is short: {}",
        found[0].message
    );
    assert!(
        found[0].message.contains(*name),
        "the report omits which runbook states the requirement: {}",
        found[0].message
    );
}

/// Two short runs produce two reports, each naming its own record.
///
/// A loop that reported the first short record and stopped would satisfy the count for every other
/// fixture here.
#[test]
fn each_short_run_is_named_in_its_own_report() {
    let (name, fixture) = FIXTURES
        .iter()
        .find(|(name, _)| *name == "two-runs-each-short")
        .expect("the fixture exists");
    let (required, runs) = subject(fixture);
    let found = conformance(&RunbookUnderReview {
        name,
        required: &required,
        runs: &runs,
    });

    assert_eq!(found.len(), 2, "{found:#?}");
    for record in ["0-13-0.html", "0-14-0.html"] {
        assert!(
            found.iter().any(|d| d.message.contains(record)),
            "no report names {record}: {found:#?}"
        );
    }
}

/// The document half of this rule's coverage exists and says what it is meant to say.
///
/// **Named rather than driven by the fixture set, and the reason is the same one
/// `a_summary_quoting_the_placeholder_reports_nothing` gives in `conformance.rs`:** the second
/// fixture's whole purpose is that validation reports *nothing* about it, and the document harness
/// `continue`s on an empty rule list. So a fixture whose point is silence cannot state that through
/// its declaration alone.
///
/// What this pins is the measurement the project-level rule exists because of. A record short of a
/// verification is a `tasks` document, and `review/unverified-requirement` is scoped to the
/// `specification` kind — so nothing reports it. If a change widened that rule to every kind, this
/// test would fail rather than the rule quietly becoming redundant; and that change would also start
/// reporting every proposal that argues for a requirement, which is why the scope is there.
#[test]
fn the_record_fixtures_exist_and_validation_is_silent_about_the_short_one() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/conformance");
    for name in [
        "valid-runbook-run-record",
        "valid-runbook-run-record-missing-its-verification",
    ] {
        let directory = root.join(name);
        assert!(
            directory.is_dir(),
            "{name} is gone, so the record's shape is no longer pinned as a contract"
        );
    }

    let short = root.join("valid-runbook-run-record-missing-its-verification");
    let source = std::fs::read_to_string(short.join("input.html")).expect("the fixture reads");

    // The fixture has to actually be short, or it pins nothing.
    assert!(
        source.contains("class=\"requirement\"") && !source.contains("class=\"verification\""),
        "the fixture is not a record short of its verification, so it pins nothing about the \
         silence below"
    );

    let outcome = htmlspec_parse::parse_document(&source);
    let projection = htmlspec_parse::project(&outcome.document);
    for strictness in [
        htmlspec_validate::Strictness::Default,
        htmlspec_validate::Strictness::Strict,
    ] {
        let report = htmlspec_validate::engine::run(
            &outcome,
            &projection,
            &htmlspec_assets::Store::Absent,
            &htmlspec_validate::Scope::loose(),
            htmlspec_validate::rules::ALL,
            strictness,
        );
        // **Narrowed to `review/example-content`, and the broad filter was a defect in this test.**
        // Excluding the whole `Review` namespace excluded the very thing this test says it would
        // catch: if `review/unverified-requirement` were widened to every kind, or this change's own
        // `review/unrecorded-requirement` were added to the per-document set, the diagnostic would
        // have been filtered out and the test would still have passed.
        //
        // What is legitimately filtered is one rule: the fixture is generated from a base carrying
        // the template's example identifiers, so it reports `review/example-content` eight times and
        // so does `valid-minimal`. That is inherited from the base rather than a property of the
        // record, and it is pre-existing for every generated fixture.
        let reported: Vec<String> = report
            .diagnostics
            .iter()
            .filter(|d| d.rule != htmlspec_validate::rules::review::EXAMPLE_CONTENT)
            .map(|d| format!("{d}"))
            .collect();
        assert!(
            report.is_accepted() && reported.is_empty(),
            "a run's record short of its verification was reported under {strictness:?}. That \
             silence is why a project-level rule exists; a per-document rule reporting it would \
             also report every proposal arguing for a requirement:\n{}",
            reported.join("\n")
        );
    }
}
