//! `schema/*`: the projected model must satisfy the published JSON Schema.
//!
//! This is the enforcement layer. The types `typify` generates from the schema
//! are a convenience for writing rule code and nothing more: generation drops
//! `minItems` into a plain `Vec` and `const` into a plain field, so a document
//! with an empty scenario list or a wrong `unit` discriminator type-checks and is
//! still invalid. Every document passes through here, and the fixtures below
//! exercise exactly the constraints that vanish in generation.
//!
//! Two rules live in this namespace. `schema/invalid` reports what the schema
//! rejected, mapped onto a unit identifier and a JSON pointer. `schema/projection`
//! reports what the projection could not read at all — the specification's word
//! for that is *invalid*, and it is a contract-conformance failure like any other.

use std::sync::OnceLock;

use jsonschema::Validator;
use serde_json::Value;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};

/// The projected model violates the published schema.
pub const INVALID: RuleId = RuleId::new(Namespace::Schema, "invalid");

/// The projection could not read the document's HTML.
pub const PROJECTION: RuleId = RuleId::new(Namespace::Schema, "projection");

/// The schema could not be compiled, so no document can be checked against it.
///
/// This reports a defect in this project rather than in the document. It is an
/// error because a validator that cannot compile its own contract must not
/// report a document as clean.
pub const UNAVAILABLE: RuleId = RuleId::new(Namespace::Schema, "unavailable");

/// The compiled schema, built once per process.
///
/// Compilation is deferred rather than done at startup so that a tool which
/// never validates does not pay for it, and the result is cached because
/// compiling per document would dominate the run.
fn validator() -> Result<&'static Validator, String> {
    static COMPILED: OnceLock<Result<Validator, String>> = OnceLock::new();
    COMPILED
        .get_or_init(|| {
            let document: Value = serde_json::from_str(htmlspec_schema::schema_text::DOCUMENT)
                .map_err(|e| format!("the published schema is not valid JSON: {e}"))?;
            // Draft 07 is what the schema declares. Naming it explicitly means a
            // schema that changes drafts fails here rather than being validated
            // against a dialect it was not written for.
            jsonschema::draft7::new(&document)
                .map_err(|e| format!("the published schema does not compile: {e}"))
        })
        .as_ref()
        .map_err(String::clone)
}

/// Validates the projected model against the published schema.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let projection = subject.projection;
    // A projection fault means the document's HTML could not be read into a
    // model at all. Reported first, because a model the projection could not
    // finish will also fail the schema, and the projection fault is the one that
    // names the cause.
    for fault in &projection.faults {
        let mut location = Location::default().at_projection_rule(fault.rule);
        location.unit_id = fault.unit_id.clone();
        diagnostics.violation(PROJECTION, location, fault.detail.clone());
    }

    let validator = match validator() {
        Ok(validator) => validator,
        Err(reason) => {
            diagnostics.violation(UNAVAILABLE, Location::document(), reason);
            return;
        }
    };

    let instance = match serde_json::to_value(&projection.model) {
        Ok(instance) => instance,
        Err(e) => {
            diagnostics.violation(
                UNAVAILABLE,
                Location::document(),
                format!("the projected model could not be serialized for validation: {e}"),
            );
            return;
        }
    };

    // Every error, not the first: the engine reports every violation it can
    // detect in one pass, and a document with three schema faults should show
    // three.
    for error in validator.iter_errors(&instance) {
        let pointer = error.instance_path().to_string();
        let mut location = Location::default().at_schema_path(if pointer.is_empty() {
            "/".to_owned()
        } else {
            pointer.clone()
        });
        location.unit_id = unit_id_at(&instance, &pointer);
        diagnostics.violation(INVALID, location, error.to_string());
    }
}

/// The identifier of the unit a JSON pointer falls inside, when there is one.
///
/// A schema error arrives with a pointer such as `/requirements/0/scenarios/1/when`.
/// A diagnostic that says only that is hard to act on, so the pointer is walked
/// from the outside in and the *deepest* enclosing object carrying an `id` wins:
/// for that example the scenario, not the requirement, because the scenario is
/// where the fault is.
fn unit_id_at(instance: &Value, pointer: &str) -> Option<String> {
    let mut node = instance;
    let mut found: Option<String> = None;
    for token in pointer.split('/').filter(|t| !t.is_empty()) {
        // A JSON pointer escapes `/` as `~1` and `~0` as `~`, and the order
        // matters: unescaping `~0` first would turn `~01` into `/`.
        let token = token.replace("~1", "/").replace("~0", "~");
        node = match node {
            Value::Object(map) => map.get(&token)?,
            Value::Array(items) => items.get(token.parse::<usize>().ok()?)?,
            _ => return found,
        };
        if let Some(id) = node.get("id").and_then(Value::as_str) {
            found = Some(id.to_owned());
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};

    /// These rules are not asset rules, so they run with no store beside them.
    const NO_STORE: htmlspec_assets::Store = htmlspec_assets::Store::Absent;
    use htmlspec_parse::{parse_document, project};

    /// Runs the schema rules over one document body.
    fn check(body: &str) -> crate::Report {
        let html = format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>Fixture</title></head>\n<body>\n{body}\n</body>\n</html>\n"
        );
        let outcome = parse_document(&html);
        assert!(
            outcome.is_recovery_free(),
            "the fixture itself recovered: {:?}",
            outcome.recovered
        );
        let projection = project(&outcome.document);
        run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[validate],
            Strictness::Default,
        )
    }

    fn messages(report: &crate::Report) -> String {
        report
            .diagnostics
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    const MINIMAL_SCENARIO: &str = "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>\
         <dl><dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd></dl></article>";

    #[test]
    fn the_published_schema_compiles() {
        // A validator that cannot compile its contract would report every
        // document as unvalidatable, so this is checked directly rather than
        // inferred from another test passing.
        assert!(validator().is_ok(), "{:?}", validator().err());
    }

    #[test]
    fn a_conforming_document_reports_nothing() {
        let report = check(&format!(
            "<section class=\"requirement\" id=\"req-x\"><h2>A requirement</h2>\
             <p>The system MUST work.</p>{MINIMAL_SCENARIO}</section>"
        ));
        assert!(report.is_accepted(), "{}", messages(&report));
    }

    /// `minItems: 1` is one of the two constraints type generation drops: the
    /// generated `scenarios` field is a plain `Vec`, which accepts the empty list
    /// this rejects. The runtime validator is what makes the constraint real.
    #[test]
    fn an_empty_required_array_is_rejected_although_generated_types_accept_it() {
        let report = check(
            "<section class=\"requirement\" id=\"req-x\"><h2>A requirement</h2>\
             <p>The system MUST work.</p></section>",
        );
        assert!(!report.is_accepted());
        let text = messages(&report);
        assert!(
            text.contains("scenarios"),
            "the diagnostic must name the empty array: {text}"
        );
        assert!(
            report.diagnostics.iter().any(|d| d.rule == INVALID),
            "{text}"
        );
    }

    /// `const` is the other dropped constraint. A document declaring the wrong
    /// contract version must fail schema validation with no Rust rule involved.
    #[test]
    fn a_wrong_contract_version_is_rejected_by_the_schema_alone() {
        let html = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"9.9.9\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body><p>Body.</p></body></html>\n";
        let outcome = parse_document(html);
        let projection = project(&outcome.document);
        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[validate],
            Strictness::Default,
        );
        assert!(!report.is_accepted());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.location.schema_path.as_deref() == Some("/schemaVersion")),
            "{}",
            messages(&report)
        );
    }

    #[test]
    fn a_missing_required_property_is_rejected() {
        // No `htmlspec-kind` meta, so `kind` is absent from the model. The schema
        // requires it and states no default, so this is the fault an unmigrated
        // document produces: named, and about the missing declaration itself.
        let html = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <title>T</title></head><body><p>Body.</p></body></html>\n";
        let outcome = parse_document(html);
        let projection = project(&outcome.document);
        let report = run(
            &outcome,
            &projection,
            &NO_STORE,
            &crate::references::Scope::loose(),
            &[validate],
            Strictness::Default,
        );
        assert!(!report.is_accepted());
        // The diagnostic names the missing property itself. This is the failure an
        // unmigrated document produces, and it has to say "kind" rather than
        // something about misplaced units: the schema requires the field and states
        // no default, so the fix is to declare one.
        assert!(messages(&report).contains("kind"), "{}", messages(&report));
    }

    #[test]
    fn a_value_outside_its_enumeration_is_rejected() {
        let report = check(
            "<section class=\"task\" id=\"task-x\" data-state=\"almost\"><h2>T</h2></section>",
        );
        assert!(!report.is_accepted());
        let text = messages(&report);
        assert!(text.contains("almost"), "{text}");
    }

    /// The pointer names the deepest unit, so a diagnostic points at the scenario
    /// that is wrong rather than at the requirement that contains it.
    #[test]
    fn a_diagnostic_names_the_deepest_enclosing_unit() {
        let report = check(
            "<section class=\"requirement\" id=\"req-outer\"><h2>R</h2><p>MUST.</p>\
             <article class=\"scenario\" id=\"scenario-inner\"><h3>S</h3>\
             <dl><dt>When</dt><dd>It happens.</dd></dl></article></section>",
        );
        assert!(!report.is_accepted(), "a scenario with no Then is invalid");
        let named: Vec<&str> = report
            .diagnostics
            .iter()
            .filter_map(|d| d.location.unit_id.as_deref())
            .collect();
        assert!(
            named.contains(&"scenario-inner"),
            "expected the scenario to be named, got {named:?}: {}",
            messages(&report)
        );
    }

    /// Every schema fault is reported, not just the first.
    #[test]
    fn several_independent_schema_faults_are_reported_together() {
        let report = check(
            "<section class=\"requirement\" id=\"req-a\"><h2>A</h2><p>MUST.</p></section>\
             <section class=\"requirement\" id=\"req-b\"><h2>B</h2><p>MUST.</p></section>\
             <section class=\"task\" id=\"task-c\" data-state=\"almost\"><h2>C</h2></section>",
        );
        assert!(report.errors().count() >= 3, "{}", messages(&report));
    }

    /// A projection fault is a contract-conformance failure, reported under
    /// `schema/projection` with the specification section that names it.
    #[test]
    fn a_projection_fault_is_reported_with_its_specification_section() {
        let report = check(
            "<section class=\"requirement\" id=\"req-x\"><h2>One</h2><h2>Two</h2>\
             <p>MUST.</p></section>",
        );
        let projection_faults: Vec<&crate::Diagnostic> = report
            .diagnostics
            .iter()
            .filter(|d| d.rule == PROJECTION)
            .collect();
        assert_eq!(projection_faults.len(), 1, "{}", messages(&report));
        assert_eq!(
            projection_faults[0].location.projection_rule.as_deref(),
            Some("3.2")
        );
        assert_eq!(
            projection_faults[0].location.unit_id.as_deref(),
            Some("req-x")
        );
    }

    #[test]
    fn every_dogfood_document_satisfies_the_schema() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dogfood");
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .expect("the dogfood directory is readable")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "html"))
            .collect();
        paths.sort();
        assert_eq!(paths.len(), 6);

        for path in paths {
            let html = std::fs::read_to_string(&path).expect("readable");
            let outcome = parse_document(&html);
            let projection = project(&outcome.document);
            let report = run(
                &outcome,
                &projection,
                &NO_STORE,
                &crate::references::Scope::loose(),
                &[validate],
                Strictness::Default,
            );
            assert!(
                report.is_accepted(),
                "{path:?} failed schema validation:\n{}",
                messages(&report)
            );
        }
    }
}
