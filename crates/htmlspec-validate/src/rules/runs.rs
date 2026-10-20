//! `review/unrecorded-requirement`: a runbook's runs record what the runbook says they must.
//!
//! **This is not a [`crate::engine::Rule`], and the reason is sharper here than for
//! [`super::topology`].** A duplicated asset is a fact about a project that no single document
//! causes; an unrecorded requirement is a fact about **two documents at once** — the runbook's
//! specification states the obligation and the run's record states the outcome, and neither
//! document holds both halves. So there is no document to attach the diagnostic to: validating the
//! record alone cannot know what was required, and validating the specification alone cannot know
//! what was recorded.
//!
//! That is why it is absent from [`super::ALL`], and the per-document alternative fails twice over.
//! It would report one violation once per document, as duplication would; and it would report it
//! against a document that is not at fault, since a run record missing a verification is a run in
//! progress rather than a malformed document.
//!
//! **The gap this closes was measured rather than assumed.** [`super::review`]'s
//! `review/unverified-requirement` is the rule whose subject this is, and its body returns early
//! unless the document declares the `specification` kind — correctly, because a requirement unit in
//! a proposal is an author arguing for one. But `runbook-execution` forces a run's record to be a
//! `tasks` document, because a `specification` carrying a task unit is a `kind/*` error on every
//! step. So the kind a record is forced into is exactly the kind the existing rule cannot see:
//! a `tasks` document whose copied requirement no verification names reports **nothing**, at either
//! strictness. Measured on the binary at `056df05`; see
//! `openspec/changes/log-a-runbook-run/evidence/the-shape-is-forced-2026-09-18.md`.
//!
//! The severity comes from the namespace rather than from this rule.
//! [`crate::diagnostic::Namespace::Review`] is a warning that never gates and that strict mode
//! cannot promote, and here that is required rather than convenient: `runbook-execution` states
//! that a run is one record from start to finish, so a record legitimately records nothing while the
//! run is under way. A rule that failed the batch halfway through the bake is one an operator
//! learns to skip.

use std::collections::BTreeSet;

use crate::diagnostic::{Diagnostic, Location, Namespace, RuleId};

/// A run's record records no outcome for a requirement its runbook states.
pub const UNRECORDED: RuleId = RuleId::new(Namespace::Review, "unrecorded-requirement");

/// One run's record, reduced to what deciding conformance needs.
///
/// The record's name and the requirements it recorded an outcome for, and nothing else. A type
/// carrying the whole projection would let this rule read a record's tasks, its prose, or its
/// figures — and `AGENTS.md` §6 asks for the weakest sufficient constraint, which here is one set of
/// identifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedRun {
    /// How the record is addressed, for the report. A file name, as the caller found it.
    pub record: String,
    /// The requirement identifiers this record recorded an outcome for.
    ///
    /// **A verification whose outcome says the check did not run does not belong here**, and that
    /// is the decision rather than a filing detail. `runbook-execution` requires a record to state
    /// *an outcome and how that outcome was reached*; a verification declaring the "not run"
    /// outcome states neither, so a record full of them has recorded nothing. That is exactly what
    /// the operation starting a run writes — one per requirement, so the record enumerates what it
    /// owes — and a rule counting them would report every run as conforming from the moment it
    /// started. [`super::review`]'s `has_run` draws the same line for the same reason, and reads
    /// the outcome values out of the schema rather than naming them.
    ///
    /// A set rather than a list: two verifications recording one requirement is a record saying so
    /// twice, which is not a fault and not a second fact.
    pub recorded: BTreeSet<String>,
}

/// One runbook and its runs, as the caller gathered them.
///
/// **Gathered facts rather than paths, which is [`super::topology::duplicates_across`]'s shape and
/// for its stated reason:** the walk stays in the caller and this stays a pure function over what
/// the walk found. This rule needs no filesystem capability, so it is not given one — and the
/// consequence is that its own tests construct the subject directly rather than building a project
/// on disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunbookUnderReview<'a> {
    /// The runbook's name, which is its directory name and the only place it is recorded.
    pub name: &'a str,
    /// Every requirement identifier the runbook's specification states, in document order.
    ///
    /// **All of them, because a runbook does not distinguish a log requirement from a hazard.**
    /// `runbook-authority` states that explicitly: a hazard the procedure met and a value a run
    /// must record are each *what this runbook requires of a run*, and distinguishing them would
    /// need an attribute the closed vocabulary does not have.
    pub required: &'a [String],
    /// The runbook's records, in the order the caller found them.
    pub runs: &'a [RecordedRun],
}

/// Every requirement a run's record recorded no outcome for.
///
/// One diagnostic per (run, requirement) pair, naming both. A diagnostic naming only the run leaves
/// a reader to open it and compare against the specification; one naming only the requirement
/// leaves them to find which of forty records is short. The same reasoning
/// [`super::topology::duplicates`] gives for naming every location holding duplicated bytes.
///
/// Empty for a runbook with no runs, and empty for a runbook that states no requirement. Both are
/// `runbook-authority` and `runbook-execution` requirements rather than conveniences: a procedure
/// nobody has run violates nothing, and a report that fired from the moment a runbook was created
/// is one a reader learns to ignore.
///
/// **Whether the outcome was good is not this rule's business, and the distinction from "whether
/// there is an outcome" is exact.** A verification recording `failed` or `blocked` recorded an
/// outcome, so it satisfies the obligation and nothing is reported: a batch that reached 200°F when
/// the recipe says 375°F is a bad batch and a correct record of one, and a rule that treated a
/// recorded failure as a document fault would teach an operator to record a pass. A verification
/// recording "not run" recorded *no* outcome, so it does not — see [`RecordedRun::recorded`], where
/// the caller draws that line rather than this function.
///
/// A record attesting to something the runbook does not require is likewise not reported. A run
/// that recorded more than it owed is not at fault, and the set difference runs in one direction
/// only.
pub fn conformance(subject: &RunbookUnderReview<'_>) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    for run in subject.runs {
        for requirement in subject.required {
            if run.recorded.contains(requirement) {
                continue;
            }
            found.push(Diagnostic::new(
                UNRECORDED,
                // The requirement, because that is the unit a reader opens. The record is named in
                // the message: a `Location` addresses a unit within a document and cannot say
                // which document, and here the document is half the fact.
                Location::unit(requirement.clone()),
                format!(
                    "the run recorded in {} records no outcome for {requirement}, which the \
                     runbook {} states. A run's record is what makes the procedure answerable \
                     later, and a requirement nothing attests to leaves that run unable to say \
                     whether the obligation was met. Record a verification naming it, with the \
                     outcome and how it was reached. This never gates the run, because a run \
                     under way legitimately has requirements it has not reached",
                    run.record, subject.name
                ),
            ));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(record: &str, recorded: &[&str]) -> RecordedRun {
        RecordedRun {
            record: record.to_owned(),
            recorded: recorded.iter().map(|id| (*id).to_owned()).collect(),
        }
    }

    fn required(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    fn messages(found: &[Diagnostic]) -> String {
        found
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// A run attesting to everything the runbook states reports nothing.
    ///
    /// The accepting direction first, because a rule that reported nothing ever would pass every
    /// test below.
    #[test]
    fn a_conforming_run_reports_nothing() {
        let required = required(&[
            "req-version-before-regenerate",
            "req-the-increment-is-judgment",
        ]);
        let runs = [run(
            "0.14.0.html",
            &[
                "req-version-before-regenerate",
                "req-the-increment-is-judgment",
            ],
        )];
        let found = conformance(&RunbookUnderReview {
            name: "release",
            required: &required,
            runs: &runs,
        });
        assert!(found.is_empty(), "{}", messages(&found));
    }

    /// The case the rule exists for, and the one no existing rule reports.
    #[test]
    fn a_run_missing_a_verification_is_reported_naming_both() {
        let required = required(&[
            "req-version-before-regenerate",
            "req-the-increment-is-judgment",
        ]);
        let runs = [run("0.14.0.html", &["req-version-before-regenerate"])];
        let found = conformance(&RunbookUnderReview {
            name: "release",
            required: &required,
            runs: &runs,
        });

        assert_eq!(found.len(), 1, "{}", messages(&found));
        assert_eq!(
            found[0].location.unit_id.as_deref(),
            Some("req-the-increment-is-judgment"),
            "the diagnostic does not address the requirement a reader would open"
        );
        // Both halves in the message, which is the point of the rule's subject being two
        // documents: a reader told only one has to find the other.
        assert!(
            found[0].message.contains("0.14.0.html"),
            "the report does not name which run is short: {}",
            found[0].message
        );
        assert!(
            found[0].message.contains("release"),
            "the report does not name the runbook stating the requirement: {}",
            found[0].message
        );
        assert!(
            !found[0].is_error(),
            "an unrecorded requirement gated the run, which would fail a run that is under way"
        );
    }

    /// A runbook nobody has run violates nothing.
    ///
    /// Without this the rule would fire from the moment a runbook was created, which is the `0/9`
    /// column the previous change refused to print, in diagnostic form.
    #[test]
    fn a_runbook_with_no_runs_reports_nothing() {
        let required = required(&["req-version-before-regenerate"]);
        let found = conformance(&RunbookUnderReview {
            name: "release",
            required: &required,
            runs: &[],
        });
        assert!(found.is_empty(), "{}", messages(&found));
    }

    /// A runbook that states no contract has runs nothing checks.
    #[test]
    fn a_runbook_stating_nothing_reports_nothing() {
        let runs = [run("batch-a.html", &[])];
        let found = conformance(&RunbookUnderReview {
            name: "chocolate-chip-cookies",
            required: &[],
            runs: &runs,
        });
        assert!(found.is_empty(), "{}", messages(&found));
    }

    /// A run that recorded a failure recorded an outcome, which is the whole obligation.
    ///
    /// **Which outcome it was is not representable in [`RecordedRun`]**, so this asserts a property
    /// of the type as much as of the function: there is no way for the rule to distinguish `passed`
    /// from `failed`, which makes "the obligation is that the run recorded an outcome, not that the
    /// outcome was good" structural rather than a behaviour someone could quietly change.
    ///
    /// The line the type *does* draw is between recording an outcome and recording none, and
    /// [`a_run_that_has_recorded_nothing_yet_is_reported`] is the other side of it.
    #[test]
    fn a_run_recording_a_failure_reports_nothing() {
        let required = required(&["req-version-before-regenerate"]);
        // A `failed` verification and a `passed` one reach this identically, deliberately.
        let runs = [run("0.14.0.html", &["req-version-before-regenerate"])];
        let found = conformance(&RunbookUnderReview {
            name: "release",
            required: &required,
            runs: &runs,
        });
        assert!(found.is_empty(), "{}", messages(&found));
    }

    /// A run that has recorded nothing yet is reported, requirement by requirement.
    ///
    /// **This is the state the operation that starts a run leaves behind**: one verification per
    /// requirement, each declaring the "not run" outcome, so the record enumerates what it owes.
    /// The caller excludes those from `recorded`, so a freshly started run is reported as having
    /// recorded nothing — which is true, and which is why the rule cannot gate.
    ///
    /// Without this test, a caller that counted a "not run" verification as an outcome would pass
    /// every other test here while making the rule report every run as conforming from the moment
    /// it started.
    #[test]
    fn a_run_that_has_recorded_nothing_yet_is_reported() {
        let required = required(&["req-one", "req-two"]);
        let runs = [run("0.14.0.html", &[])];
        let found = conformance(&RunbookUnderReview {
            name: "release",
            required: &required,
            runs: &runs,
        });

        assert_eq!(found.len(), 2, "{}", messages(&found));
        assert!(
            found.iter().all(|d| !d.is_error()),
            "a run that has only just started gated something: {}",
            messages(&found)
        );
    }

    /// A run that recorded more than it owed is not at fault.
    #[test]
    fn a_run_attesting_to_something_unrequired_reports_nothing() {
        let required = required(&["req-version-before-regenerate"]);
        let runs = [run(
            "0.14.0.html",
            &["req-version-before-regenerate", "req-nobody-asked-for-this"],
        )];
        let found = conformance(&RunbookUnderReview {
            name: "release",
            required: &required,
            runs: &runs,
        });
        assert!(found.is_empty(), "{}", messages(&found));
    }

    /// Every run is judged, not only the first.
    ///
    /// A loop that reported the first short record and stopped would pass every test above.
    #[test]
    fn each_short_run_is_reported_separately() {
        let required = required(&["req-one", "req-two"]);
        let runs = [
            run("first.html", &["req-one"]),
            run("second.html", &[]),
            run("third.html", &["req-one", "req-two"]),
        ];
        let found = conformance(&RunbookUnderReview {
            name: "release",
            required: &required,
            runs: &runs,
        });

        assert_eq!(found.len(), 3, "{}", messages(&found));
        let rendered = messages(&found);
        assert!(rendered.contains("first.html"), "{rendered}");
        assert!(rendered.contains("second.html"), "{rendered}");
        assert!(
            !rendered.contains("third.html"),
            "a conforming run was reported: {rendered}"
        );
    }

    /// The severity is the namespace's, and strict mode cannot promote it.
    ///
    /// Asserted rather than read, because `runbook-execution` states it as an obligation: the
    /// report "SHALL NOT fail validation and SHALL NOT be promotable to a failure".
    #[test]
    fn an_unrecorded_requirement_is_a_warning_strict_mode_cannot_promote() {
        assert_eq!(UNRECORDED.namespace(), Namespace::Review);
        assert!(!Namespace::Review.is_promotable());
        assert_eq!(
            Namespace::Review.default_severity(),
            crate::diagnostic::Severity::Warning
        );
    }
}
