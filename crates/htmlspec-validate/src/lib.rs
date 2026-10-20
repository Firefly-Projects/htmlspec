//! The HTMLSpec rule engine and its diagnostics.
//!
//! Rules operate on the projected document model rather than on the raw tree, so
//! a diagnostic points at a unit by identifier and a rule stays independent of
//! markup incidentals.
//!
//! The schema owns the shape of a unit. This crate owns relationships between
//! units: identifier uniqueness, prefix agreement with unit type, reference
//! resolution, and references to a removed unit. [`schema_facts`] is the only
//! module that reads contract values, and it reads them from the published
//! schema rather than restating them.

pub mod diagnostic;
pub mod engine;
pub mod output;
pub mod references;
pub mod rules;
pub mod schema_facts;

pub use diagnostic::{Diagnostic, Location, Namespace, RuleId, Severity};
pub use engine::{run, Curtailment, Diagnostics, Report, Rule, Strictness, Subject};
pub use output::{
    render_human, render_json, StructuredCurtailment, StructuredDiagnostic, StructuredReport,
    Summary,
};
pub use references::{Place, Placement, Scope, Unread};
