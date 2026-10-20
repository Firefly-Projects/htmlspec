//! Rule identifiers, severities, and the diagnostics rules emit.
//!
//! A rule identifier is namespaced by concern, and the namespace carries the
//! severity policy rather than each rule restating it: `safety/*` is an error
//! that cannot be downgraded, `review/*` is a warning that never gates work, and
//! everything else is an error. That is what makes "can this be a warning?" a
//! question about the namespace instead of a judgement call per rule.

use std::fmt;

/// The concern a rule belongs to.
///
/// The namespace is a closed set, because it decides severity. Adding a concern
/// is a deliberate act that also decides whether its rules can be downgraded.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Namespace {
    /// Contract conformance: the schema and the projection specification.
    Schema,
    /// Reference resolution.
    Refs,
    /// Network dependencies and executable content.
    Safety,
    /// The relationship between a document's asset references and the bytes stored
    /// beside it.
    Asset,
    /// What a document's declared artifact kind requires of its content.
    ///
    /// Separate from [`Self::Schema`] rather than folded into it: these are not
    /// contract-shape faults, and a reader triaging a `schema/invalid` diagnostic
    /// is looking for a malformed unit. A misplaced task is a well-formed unit in
    /// the wrong document.
    Kind,
    /// Visualization specifications.
    Viz,
    /// The no-JavaScript and non-color-dependence rules.
    A11y,
    /// A dependent unit needing review after an upstream edit.
    Review,
    /// A fatal parse failure, which curtails every other rule.
    Parse,
}

impl Namespace {
    /// Every namespace, in report order.
    pub const ALL: &'static [Self] = &[
        Self::Parse,
        Self::Schema,
        Self::Refs,
        Self::Kind,
        Self::Safety,
        Self::Asset,
        Self::Viz,
        Self::A11y,
        Self::Review,
    ];

    /// The wire form, which is the identifier's prefix before the slash.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Refs => "refs",
            Self::Kind => "kind",
            Self::Safety => "safety",
            Self::Asset => "asset",
            Self::Viz => "viz",
            Self::A11y => "a11y",
            Self::Review => "review",
            Self::Parse => "parse",
        }
    }

    /// The severity a rule in this namespace carries before any policy applies.
    ///
    /// `review/*` reports a unit as needing review after an upstream edit. It is
    /// a warning because the dependency model marks work for review rather than
    /// gating it, and reconciliation clears it.
    pub const fn default_severity(self) -> Severity {
        match self {
            Self::Review => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// Whether a rule in this namespace may be reduced to a warning.
    ///
    /// `safety/*` and `asset/*` refuse. A document with a remote script or live
    /// executable content is unsafe to open, and a project must not be able to
    /// configure that away.
    ///
    /// `asset/*` refuses for a related but distinct reason: a document rendering
    /// against bytes other than the ones it names is misreporting its own
    /// provenance. The report is not a matter of degree — either the reference
    /// describes the bytes beside it or the document is lying about what it renders
    /// with — so there is nothing a warning would express. The specs state this
    /// directly: the rule "SHALL NOT be reducible to a warning".
    pub const fn is_downgradable(self) -> bool {
        !matches!(self, Self::Safety | Self::Asset)
    }

    /// Whether strict mode may promote a warning in this namespace to an error.
    ///
    /// Only `review/*` refuses, and it is the mirror of the `safety/*` floor: one
    /// namespace cannot be made lenient and one cannot be made strict.
    ///
    /// The reason is a requirement, not a preference. A `review/*` diagnostic
    /// reports that a dependent unit references something this change altered, and
    /// the specs say such a report "SHALL NOT block continued work, SHALL NOT
    /// impose a phase gate, and SHALL NOT require regeneration of the dependent
    /// artifact". A strict run that failed on one would impose exactly the gate the
    /// dependency model exists to avoid: an agent editing a requirement would find
    /// continuous integration red until every dependent was reconciled, which is
    /// the waterfall the fluid model rejects.
    ///
    /// This does narrow what "strict" means, and deliberately. Strict mode makes
    /// *checkable* warnings fatal. A review marker is not a defect to be fixed; it
    /// is a note that clears when the author settles the upstream unit.
    pub const fn is_promotable(self) -> bool {
        !matches!(self, Self::Review)
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A stable rule identifier, such as `refs/unresolved`.
///
/// The identifier is stable across versions of the validator, so a suppression
/// or a test written against one release still names the same rule in the next.
/// Both halves are `&'static str`, so a rule identifier cannot be assembled from
/// runtime data and every rule in the engine is enumerable.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuleId {
    namespace: Namespace,
    name: &'static str,
}

impl RuleId {
    /// Names a rule within a namespace.
    pub const fn new(namespace: Namespace, name: &'static str) -> Self {
        Self { namespace, name }
    }

    /// The concern this rule belongs to.
    pub const fn namespace(self) -> Namespace {
        self.namespace
    }

    /// The rule's name within its namespace.
    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.namespace, self.name)
    }
}

/// Whether a violation fails validation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Severity {
    /// Reported, and validation fails.
    Error,
    /// Reported, and validation still succeeds.
    Warning,
}

impl Severity {
    /// The wire form, used in machine-readable diagnostics.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Where in a document a diagnostic points.
///
/// A location is a unit identifier where one exists, because that is the address
/// an agent can act on and it survives an edit that moves lines around. The
/// schema path is the JSON pointer into the projected model, which is what makes
/// a `schema/*` diagnostic traceable back to the constraint that rejected it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Location {
    /// The identifier of the unit at fault, when it has one.
    pub unit_id: Option<String>,
    /// A JSON pointer into the projected model, when the rule has one.
    pub schema_path: Option<String>,
    /// The projection-specification section, for a rule that implements one.
    pub projection_rule: Option<String>,
}

impl Location {
    /// A location naming a unit.
    pub fn unit(id: impl Into<String>) -> Self {
        Self {
            unit_id: Some(id.into()),
            ..Self::default()
        }
    }

    /// A location naming a unit that may not have declared an identifier.
    ///
    /// A unit with no identifier still has faults worth reporting; the schema
    /// reports the missing property, and a relational rule reports what it found
    /// against the document rather than staying silent.
    pub fn maybe_unit(id: Option<&str>) -> Self {
        Self {
            unit_id: id.map(str::to_owned),
            ..Self::default()
        }
    }

    /// A location naming nothing more specific than the document.
    pub fn document() -> Self {
        Self::default()
    }

    /// Adds a JSON pointer into the projected model.
    pub fn at_schema_path(mut self, path: impl Into<String>) -> Self {
        self.schema_path = Some(path.into());
        self
    }

    /// Adds the projection-specification section this diagnostic implements.
    pub fn at_projection_rule(mut self, rule: impl Into<String>) -> Self {
        self.projection_rule = Some(rule.into());
        self
    }

    /// Whether this location names nothing at all.
    pub fn is_document_wide(&self) -> bool {
        self.unit_id.is_none() && self.schema_path.is_none() && self.projection_rule.is_none()
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Both halves are shown when both exist. The identifier says which unit
        // to open and the pointer says which field of it is wrong; a reader given
        // only the identifier still has to guess the field, which is the whole
        // thing a diagnostic is supposed to save them.
        match (&self.unit_id, &self.schema_path) {
            (Some(id), Some(path)) => write!(f, "{id} ({path})"),
            (Some(id), None) => write!(f, "{id}"),
            (None, Some(path)) => write!(f, "{path}"),
            (None, None) => f.write_str("document"),
        }
    }
}

/// One violation, as a rule reports it.
///
/// A rule constructs this with its intrinsic severity. Policy — strict mode, and
/// the `safety/*` floor — is applied by the engine when the run is assembled, so
/// a rule cannot accidentally decide its own downgradability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// The rule violated.
    pub rule: RuleId,
    /// Whether this fails validation, after policy.
    pub severity: Severity,
    /// Where in the document the violation is.
    pub location: Location,
    /// What is wrong, in a form a document author can act on.
    pub message: String,
    /// A limitation of the check itself, where one applies.
    ///
    /// A shallow check that reports a pass has told the reader less than it
    /// appears to. Mermaid is the case this exists for: the note travels with the
    /// diagnostic rather than living only in prose documentation.
    pub note: Option<String>,
}

impl Diagnostic {
    /// A violation at its namespace's default severity.
    pub fn new(rule: RuleId, location: Location, message: impl Into<String>) -> Self {
        Self {
            severity: rule.namespace().default_severity(),
            rule,
            location,
            message: message.into(),
            note: None,
        }
    }

    /// Adds a note recording a limitation of the check.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Whether this diagnostic fails validation.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} at {}: {}",
            self.severity, self.rule, self.location, self.message
        )?;
        if let Some(note) = &self.note {
            write!(f, "\n  note: {note}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rule_identifier_reads_as_its_namespace_and_name() {
        let rule = RuleId::new(Namespace::Refs, "unresolved");
        assert_eq!(rule.to_string(), "refs/unresolved");
    }

    #[test]
    fn a_namespace_decides_severity_rather_than_each_rule() {
        assert_eq!(Namespace::Review.default_severity(), Severity::Warning);
        for namespace in Namespace::ALL {
            if *namespace == Namespace::Review {
                continue;
            }
            assert_eq!(
                namespace.default_severity(),
                Severity::Error,
                "{namespace} must default to an error"
            );
        }
    }

    /// The non-downgradable set is exactly `safety/*` and `asset/*`.
    ///
    /// Written as an exhaustive partition rather than as two assertions, so adding a
    /// namespace forces a decision here about whether it can be configured away
    /// instead of silently defaulting to "yes".
    #[test]
    fn only_safety_and_asset_refuse_to_be_downgraded() {
        const NON_DOWNGRADABLE: &[Namespace] = &[Namespace::Safety, Namespace::Asset];
        for namespace in Namespace::ALL {
            let refuses = NON_DOWNGRADABLE.contains(namespace);
            assert_eq!(
                namespace.is_downgradable(),
                !refuses,
                "{namespace} has the wrong downgradability"
            );
        }
    }

    #[test]
    fn every_namespace_has_a_distinct_wire_form() {
        let mut forms: Vec<&str> = Namespace::ALL.iter().map(|n| n.as_str()).collect();
        let before = forms.len();
        forms.sort_unstable();
        forms.dedup();
        assert_eq!(before, forms.len(), "two namespaces share a wire form");
    }

    /// A reader needs both halves: which unit to open, and which field is wrong.
    #[test]
    fn a_location_shows_the_unit_and_the_field_when_it_knows_both() {
        let both = Location::unit("req-x").at_schema_path("/requirements/0/scenarios");
        assert_eq!(both.to_string(), "req-x (/requirements/0/scenarios)");
        assert_eq!(Location::unit("req-x").to_string(), "req-x");
        assert_eq!(
            Location::document().at_schema_path("/status").to_string(),
            "/status"
        );
        assert_eq!(Location::document().to_string(), "document");
        assert!(Location::document().is_document_wide());
        assert!(!Location::unit("req-x").is_document_wide());
    }

    #[test]
    fn a_diagnostic_takes_its_namespaces_severity() {
        let safety = Diagnostic::new(
            RuleId::new(Namespace::Safety, "remote-script"),
            Location::document(),
            "a remote script",
        );
        assert!(safety.is_error());

        let review = Diagnostic::new(
            RuleId::new(Namespace::Review, "stale-reference"),
            Location::unit("task-x"),
            "the requirement changed",
        );
        assert!(!review.is_error());
    }
}
