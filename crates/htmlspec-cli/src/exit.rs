//! The exit-code contract.
//!
//! Four exit classes, five outcomes, and the distinction the specs require is between the
//! last two failures: a document that fails validation must be distinguishable by exit code
//! from a failure to run the operation at all. A tool that returned 1 for both would
//! leave a continuous-integration job unable to tell "the specification is wrong"
//! from "the file was unreadable", and those call for different responses.
//!
//! One class holds two outcomes. A document that violates the contract and a machine that cannot
//! run a procedure are both an unsatisfied contract, and `toolchain-packaging` puts both at exit 1;
//! they stay separate variants because the reports differ — one names a document's fault and the
//! other names a missing tool.
//!
//! The codes are a public contract, so they are named here once and every exit path
//! goes through this type rather than calling `std::process::exit` with a literal.

/// How the program exited.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The requested operation succeeded.
    Success,
    /// A document violated the contract: validation reported an error, or check
    /// mode found a document that is not canonically formatted.
    ///
    /// The operation itself ran correctly. This is the tool reporting on documents,
    /// which is what it is for.
    DocumentViolation,
    /// A completed readiness assessment found the machine short of what a runbook requires.
    ///
    /// Its own outcome rather than a [`Self::DocumentViolation`], and the distinction is the
    /// domain one: a document violation says *this document is wrong*, while this says *the
    /// document is fine and this machine cannot run it*. `toolchain-packaging` puts both at exit
    /// 1 — "A completed preflight with unmet requirements SHALL exit 1" — because both are the
    /// same class of answer to a caller: an unsatisfied contract rather than a broken tool.
    /// Folding the machine case into `DocumentViolation` would save an exit code and lose the
    /// only place the program can say which of the two it found, so the two variants share a
    /// code and remain distinct values.
    PrerequisiteFailure,
    /// The operation could not be run: an unreadable file, an invalid invocation, a
    /// document too ambiguous to format, a defect in the tool.
    ///
    /// Distinct from [`Self::DocumentViolation`] because it says nothing about the
    /// documents. Nothing was determined about them.
    OperationalFailure,
    /// A contributor declined a prompt, or interrupted the operation.
    ///
    /// Its own outcome because it is neither of the two failures above: nothing went
    /// wrong, and nothing was done. Mapping this onto [`Self::OperationalFailure`] would
    /// leave an automated caller unable to tell a broken tool from a person's refusal —
    /// the same distinction this type already draws between a document violation and an
    /// operational failure.
    Cancelled,
}

impl Outcome {
    /// The process exit code.
    ///
    /// `2` for an operational failure follows the convention `clap` already uses for
    /// a usage error, so an invalid invocation and an unreadable file report the same
    /// class of problem with the same code rather than two.
    ///
    /// `1` for both [`Self::DocumentViolation`] and [`Self::PrerequisiteFailure`], which is what
    /// `toolchain-packaging` requires of preflight and what `document-validation` already required
    /// of a violated contract. The codes are not the whole contract: a caller that needs to tell
    /// the two apart reads the report, which names the requirement and the missing condition.
    pub const fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::DocumentViolation => 1,
            Self::PrerequisiteFailure => 1,
            Self::OperationalFailure => 2,
            // Above the codes the tool uses for its own findings, so a caller testing
            // `!= 0` still sees a refusal as "did not complete" while a caller that
            // cares can tell it apart.
            Self::Cancelled => 3,
        }
    }

    /// Whether the operation succeeded.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "part of the outcome's API; \
        every current caller reads `code()` instead, and a caller that wants the \
        question rather than the number should not have to compare against zero"
        )
    )]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The contract the specs state: zero only for success, and every failure class reachable.
    ///
    /// **This test asserted five distinct codes, and `toolchain-packaging` now requires two
    /// outcomes to share one.** "A completed preflight with unmet requirements SHALL exit 1" is
    /// the same code a violated document exits with, so distinctness across every outcome is no
    /// longer the contract and asserting it would assert something the spec forbids. What remains
    /// load-bearing is that the *classes* stay distinguishable — success, an unsatisfied contract,
    /// an operation that could not run, and a refusal each keep their own code — and that the two
    /// outcomes sharing exit 1 are still different values, which is what lets the reports differ.
    #[test]
    fn each_outcome_class_has_its_own_code_and_only_success_is_zero() {
        let outcomes = [
            Outcome::Success,
            Outcome::DocumentViolation,
            Outcome::PrerequisiteFailure,
            Outcome::OperationalFailure,
            Outcome::Cancelled,
        ];

        for outcome in outcomes {
            assert_eq!(
                outcome.code() == 0,
                outcome.is_success(),
                "{outcome:?} disagrees about whether it succeeded"
            );
        }

        // The classes, one code each. `DocumentViolation` and `PrerequisiteFailure` are one class.
        let mut codes: Vec<i32> = outcomes.iter().map(|o| o.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(
            codes,
            vec![0, 1, 2, 3],
            "the exit classes are not the four the specs state"
        );

        // And the two that share a code are still distinct values, so a report can say which one
        // the program found. Losing this would be the "stuff machine failures into
        // DocumentViolation" shortcut the design names.
        assert_ne!(Outcome::DocumentViolation, Outcome::PrerequisiteFailure);
        assert_eq!(
            Outcome::PrerequisiteFailure.code(),
            Outcome::DocumentViolation.code(),
            "an unmet prerequisite left the unsatisfied-contract exit class"
        );
    }

    /// A machine that cannot run a procedure is not a broken tool, and the codes say so.
    #[test]
    fn an_unmet_prerequisite_is_distinguishable_from_an_operational_failure() {
        assert_ne!(
            Outcome::PrerequisiteFailure.code(),
            Outcome::OperationalFailure.code(),
            "a completed negative assessment reported as a tool failure"
        );
        assert!(!Outcome::PrerequisiteFailure.is_success());
    }

    /// The distinction the specs specifically require.
    #[test]
    fn a_document_violation_is_distinguishable_from_an_operational_failure() {
        assert_ne!(
            Outcome::DocumentViolation.code(),
            Outcome::OperationalFailure.code()
        );
    }

    /// A refusal is not a failure to run. A job that saw one code for both could not
    /// report either correctly.
    #[test]
    fn cancellation_is_distinguishable_from_every_other_outcome() {
        for other in [
            Outcome::Success,
            Outcome::DocumentViolation,
            Outcome::PrerequisiteFailure,
            Outcome::OperationalFailure,
        ] {
            assert_ne!(
                Outcome::Cancelled.code(),
                other.code(),
                "cancellation shares a code with {other:?}"
            );
        }
        assert!(!Outcome::Cancelled.is_success());
    }
}
