//! The rule engine: one pass, every diagnostic, one policy.
//!
//! Rules run in a single pass and diagnostics accumulate, with one exception: a
//! fatal parse failure short-circuits, because there is no tree to reason about
//! and reporting rules against a guessed tree would be worse than reporting
//! nothing. When that happens the report says so, so a clean-looking run and a
//! curtailed one cannot be confused.
//!
//! Severity policy lives here rather than in each rule. A rule reports what it
//! found at its namespace's intrinsic severity; the engine then applies strict
//! mode and the `safety/*` floor. That ordering is what makes the floor
//! unconditional: a rule cannot opt out of it, because a rule never sees it.

use htmlspec_assets::Store;
use htmlspec_parse::{ParseOutcome, Projection};

use crate::diagnostic::{Diagnostic, Location, Namespace, RuleId, Severity};
use crate::references::{Placement, Scope};

/// How strictly a run treats warnings.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Strictness {
    /// A warning is reported and validation still succeeds.
    #[default]
    Default,
    /// A warning is promoted to an error, so continuous integration can enforce
    /// it. Promotion never runs the other way: the `safety/*` floor holds.
    Strict,
}

/// Why a run stopped before running every rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Curtailment {
    /// The parser recovered from at least one error, so the document's meaning
    /// is not determined by its markup and there is no tree worth judging.
    FatalParseError {
        /// How many recoveries the parser reported.
        recovered: usize,
    },
}

impl std::fmt::Display for Curtailment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FatalParseError { recovered } => write!(
                f,
                "no further rules were evaluated: the document produced {recovered} recovered \
                 parse error(s), so its interpretation depends on error recovery"
            ),
        }
    }
}

/// The outcome of validating one document.
#[derive(Clone, Debug)]
pub struct Report {
    /// Every diagnostic, in the order the rules produced them.
    pub diagnostics: Vec<Diagnostic>,
    /// Why evaluation stopped early, when it did.
    ///
    /// `Some` means the diagnostics are not the whole picture, and a consumer
    /// must not read a short list as a clean document.
    pub curtailed: Option<Curtailment>,
}

impl Report {
    /// Whether the document is accepted.
    ///
    /// A curtailed run is never accepted: it did not evaluate every rule, so it
    /// cannot report that every rule passed.
    pub fn is_accepted(&self) -> bool {
        self.curtailed.is_none() && !self.diagnostics.iter().any(Diagnostic::is_error)
    }

    /// Every diagnostic that fails validation.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_error())
    }

    /// Every diagnostic that does not fail validation.
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| !d.is_error())
    }
}

/// Collects diagnostics from the rules of one run.
///
/// A rule receives this and reports into it. It cannot read what other rules
/// found, which is what keeps rules independent and the pass single.
pub struct Diagnostics {
    collected: Vec<Diagnostic>,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            collected: Vec::new(),
        }
    }

    /// Reports one violation.
    pub fn report(&mut self, diagnostic: Diagnostic) {
        self.collected.push(diagnostic);
    }

    /// Reports one violation, built from its parts.
    pub fn violation(&mut self, rule: RuleId, location: Location, message: impl Into<String>) {
        self.report(Diagnostic::new(rule, location, message));
    }

    /// How many violations have been reported so far.
    ///
    /// Available so a rule can tell whether an earlier phase of its own work
    /// found anything. It says nothing about which rule reported what.
    pub fn len(&self) -> usize {
        self.collected.len()
    }

    /// Whether nothing has been reported yet.
    pub fn is_empty(&self) -> bool {
        self.collected.is_empty()
    }
}

/// The rule that reports a fatal parse recovery.
pub const PARSE_RECOVERY: RuleId = RuleId::new(Namespace::Parse, "recovered-error");

/// What a rule is given to judge: one document, and the bytes stored beside it.
///
/// The projection is the document. The store is its surroundings, and it is here
/// rather than reached for because a rule that read the filesystem itself could
/// depend on a path that exists on one machine and not another — which is the class
/// of defect the asset rules exist to remove, so the rules must not reintroduce it.
///
/// Most rules read only the projection. The store is carried for every rule anyway,
/// rather than given to a second kind of rule, so that the engine keeps one rule
/// type and one registration list. Two lists would mean a rule could be live in one
/// sense and dead in the other, and nothing would notice.
pub struct Subject<'a> {
    /// The projected model, which is the document itself.
    pub projection: &'a Projection,
    /// The asset store the document sits in, which may be [`Store::Absent`].
    ///
    /// Absent is the ordinary case for a document validated on its own — a
    /// conformance fixture, a file passed by path with no project around it. The
    /// asset rules are scoped by this and by nothing else: a document cannot declare
    /// itself exempt, because a rule a document can switch off protects nothing.
    pub store: &'a Store,
    /// Where the document sits, and what the places its references reach declare.
    ///
    /// Given rather than reached for, for the reason the store is: the command line reads the
    /// other documents, and a rule reads the identifiers they declare. [`Scope::loose`] is the
    /// ordinary case for a document validated on its own, which reaches only itself.
    pub scope: &'a Scope,
}

impl Subject<'_> {
    /// Where the document sits in its project.
    pub fn placement(&self) -> Placement {
        self.scope.placement()
    }
}

/// One rule of the engine.
///
/// A rule reads the [`Subject`] and reports into [`Diagnostics`]. It returns
/// nothing, so it cannot signal "stop" — accumulating every diagnostic in one
/// pass is the engine's guarantee and not a rule's decision.
pub type Rule = fn(&Subject<'_>, &mut Diagnostics);

/// Runs every rule over one parsed document.
///
/// `parse` is the parse outcome, which decides whether the run is curtailed.
/// `projection` is the projected model the rules read. `store` is the asset store
/// the document sits in; pass [`Store::Absent`] for a document validated with no
/// project around it, which scopes the asset rules out. `scope` is where the document
/// sits and what its references can reach; pass [`Scope::loose`] for a document that
/// reaches only itself.
pub fn run(
    parse: &ParseOutcome,
    projection: &Projection,
    store: &Store,
    scope: &Scope,
    rules: &[Rule],
    strictness: Strictness,
) -> Report {
    let mut diagnostics = Diagnostics::new();

    // Every recovery is fatal (design: "Parse with the browser's grammar, then
    // treat parser errors as fatal"), and it curtails the run: there is no
    // unambiguous tree, so a rule's verdict about it would be about a guess.
    if !parse.is_recovery_free() {
        for recovered in &parse.recovered {
            diagnostics.report(Diagnostic::new(
                PARSE_RECOVERY,
                Location::document(),
                format!(
                    "the parser recovered from an error ({}): {}",
                    recovered.code, recovered.message
                ),
            ));
        }
        return Report {
            diagnostics: apply_policy(diagnostics.collected, strictness),
            curtailed: Some(Curtailment::FatalParseError {
                recovered: parse.recovered.len(),
            }),
        };
    }

    let subject = Subject {
        projection,
        store,
        scope,
    };
    for rule in rules {
        rule(&subject, &mut diagnostics);
    }

    Report {
        diagnostics: apply_policy(diagnostics.collected, strictness),
        curtailed: None,
    }
}

/// Applies severity policy to every diagnostic of a run.
///
/// Two rules, in this order. Strict mode promotes a warning to an error, except in
/// a namespace that refuses promotion — `review/*`, whose reports must never gate
/// work. The `safety/*` floor then holds regardless: a safety diagnostic is an
/// error even if something upstream tried to make it a warning. The floor runs
/// second so it cannot be undone.
///
/// The two exceptions are mirror images and neither is configurable: one namespace
/// cannot be made lenient, one cannot be made strict.
fn apply_policy(diagnostics: Vec<Diagnostic>, strictness: Strictness) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|mut diagnostic| {
            if strictness == Strictness::Strict && diagnostic.rule.namespace().is_promotable() {
                diagnostic.severity = Severity::Error;
            }
            if !diagnostic.rule.namespace().is_downgradable() {
                diagnostic.severity = Severity::Error;
            }
            diagnostic
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use htmlspec_parse::{parse_document, project};

    /// The store a document validated on its own sits in: none.
    ///
    /// Every test here is about severity policy rather than about assets, so they all
    /// run with no store — which is also the case that must never fire an asset rule.
    const NO_STORE: Store = Store::Absent;

    fn parse(html: &str) -> (ParseOutcome, Projection) {
        let outcome = parse_document(html);
        let projection = project(&outcome.document);
        (outcome, projection)
    }

    const CLEAN: &str = "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>T</title></head><body><p>Body.</p></body></html>\n";

    fn error_rule(_: &Subject<'_>, diagnostics: &mut Diagnostics) {
        diagnostics.violation(
            RuleId::new(Namespace::Refs, "unresolved"),
            Location::unit("task-x"),
            "a dangling reference",
        );
    }

    /// A `review/*` warning, which strict mode must not promote.
    fn warning_rule(_: &Subject<'_>, diagnostics: &mut Diagnostics) {
        diagnostics.violation(
            RuleId::new(Namespace::Review, "stale-reference"),
            Location::unit("task-y"),
            "needs review",
        );
    }

    /// A warning in a namespace strict mode *does* promote.
    ///
    /// Constructed by hand rather than taken from a real rule: every namespace
    /// except `review/*` defaults to an error, so a promotable warning only arises
    /// when something has explicitly lowered one.
    fn a11y_warning_rule(_: &Subject<'_>, diagnostics: &mut Diagnostics) {
        let mut diagnostic = Diagnostic::new(
            RuleId::new(Namespace::A11y, "figure-without-caption"),
            Location::unit("figure-y"),
            "this figure has no caption",
        );
        diagnostic.severity = Severity::Warning;
        diagnostics.report(diagnostic);
    }

    fn safety_rule(_: &Subject<'_>, diagnostics: &mut Diagnostics) {
        diagnostics.violation(
            RuleId::new(Namespace::Safety, "remote-script"),
            Location::document(),
            "a remote script",
        );
    }

    #[test]
    fn a_clean_document_with_no_rules_is_accepted() {
        let (outcome, projection) = parse(CLEAN);
        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[],
            Strictness::Default,
        );
        assert!(report.is_accepted());
        assert!(report.diagnostics.is_empty());
        assert!(report.curtailed.is_none());
    }

    /// Every rule runs even though the first already failed the document.
    #[test]
    fn every_rule_runs_in_one_pass() {
        let (outcome, projection) = parse(CLEAN);
        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[error_rule, warning_rule, safety_rule],
            Strictness::Default,
        );
        assert_eq!(report.diagnostics.len(), 3, "{:#?}", report.diagnostics);
        assert_eq!(report.errors().count(), 2);
        assert_eq!(report.warnings().count(), 1);
        assert!(!report.is_accepted());
    }

    #[test]
    fn a_warning_alone_does_not_fail_validation() {
        let (outcome, projection) = parse(CLEAN);
        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[warning_rule],
            Strictness::Default,
        );
        assert!(report.is_accepted());
        assert_eq!(report.warnings().count(), 1);
    }

    #[test]
    fn strict_mode_promotes_a_warning_to_an_error() {
        let (outcome, projection) = parse(CLEAN);
        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[a11y_warning_rule],
            Strictness::Strict,
        );
        assert!(!report.is_accepted());
        assert_eq!(report.errors().count(), 1);
        assert_eq!(report.warnings().count(), 0);
    }

    /// The mirror of the `safety/*` floor: `review/*` refuses promotion, because a
    /// review marker must never gate work. A strict run that failed on one would
    /// impose the phase gate the dependency model exists to avoid.
    #[test]
    fn strict_mode_does_not_promote_a_review_warning() {
        let (outcome, projection) = parse(CLEAN);
        for strictness in [Strictness::Default, Strictness::Strict] {
            let report = run(
                &outcome,
                &projection,
                &NO_STORE,
                &crate::references::Scope::loose(),
                &[warning_rule],
                strictness,
            );
            assert!(
                report.is_accepted(),
                "a review warning gated work under {strictness:?}"
            );
            assert_eq!(report.warnings().count(), 1);
            assert_eq!(report.errors().count(), 0);
        }
    }

    /// The floor is unconditional, so it holds under every strictness.
    #[test]
    fn a_safety_diagnostic_is_an_error_under_every_policy() {
        let (outcome, projection) = parse(CLEAN);
        for strictness in [Strictness::Default, Strictness::Strict] {
            let report = run(
                &outcome,
                &projection,
                &NO_STORE,
                &crate::references::Scope::loose(),
                &[safety_rule],
                strictness,
            );
            assert!(
                report.diagnostics[0].is_error(),
                "safety was downgraded under {strictness:?}"
            );
        }
    }

    #[test]
    fn a_recovered_parse_error_curtails_the_run_and_says_so() {
        // A missing doctype is a recovery, so the document is rejected.
        let (outcome, projection) =
            parse("<html><head><title>T</title></head><body></body></html>");
        assert!(!outcome.is_recovery_free());

        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[error_rule, warning_rule],
            Strictness::Default,
        );
        assert!(!report.is_accepted());
        assert_eq!(
            report.curtailed,
            Some(Curtailment::FatalParseError { recovered: 1 })
        );
        // The parse recovery is reported and no other rule ran.
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].rule, PARSE_RECOVERY);
        assert!(report
            .curtailed
            .as_ref()
            .expect("curtailed")
            .to_string()
            .contains("no further rules were evaluated"));
    }

    /// A curtailed run with no error would otherwise read as a clean document.
    #[test]
    fn a_curtailed_run_is_never_accepted() {
        let (outcome, projection) =
            parse("<html><head><title>T</title></head><body></body></html>");
        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[],
            Strictness::Default,
        );
        assert!(
            !report.is_accepted(),
            "a run that did not evaluate every rule must not report acceptance"
        );
    }
}
