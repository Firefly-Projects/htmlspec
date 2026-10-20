//! Task 3.6: the schema owns the shape of a unit, and Rust owns relationships
//! between units. This test keeps the line drawn.
//!
//! The rule is stated as a source constraint, because "which layer is this rule
//! in?" can only be answered by looking at where the constant lives. A rule that
//! restates a schema-expressible constraint in Rust would compile, pass its own
//! tests, and drift the moment the schema changed. So this test scans the rule
//! crate's source for literals that belong to the schema.
//!
//! The narrow exception is `crates/htmlspec-validate/src/schema_facts.rs`, which
//! is permitted to read such values *out of the schema at run time*. Nothing may
//! hard-code them. The reader it exposes lives in the schema crate, because the
//! parser needs the same facts to decide whether a fragment link carries a
//! relationship, and one reader cannot disagree with itself.

use std::path::{Path, PathBuf};

/// Values the schema expresses. None may appear as a literal in rule code.
///
/// Each entry pairs the literal with the schema construct that owns it, so a
/// failure names the layer the rule belongs in rather than only the offending
/// line.
const SCHEMA_OWNED_LITERALS: &[(&str, &str)] = &[
    ("req-", "definitions.requirementId.pattern"),
    ("scenario-", "definitions.scenarioId.pattern"),
    ("decision-", "definitions.decisionId.pattern"),
    ("task-", "definitions.taskId.pattern"),
    ("evidence-", "definitions.evidenceId.pattern"),
    ("risk-", "definitions.riskId.pattern"),
    ("verify-", "definitions.verificationId.pattern"),
    ("figure-", "definitions.figureId.pattern"),
    ("in-progress", "definitions.task.properties.state.enum"),
    ("done", "definitions.task.properties.state.enum"),
    ("superseded", "definitions.decision.properties.status.enum"),
    ("mitigated", "definitions.risk.properties.response.enum"),
    ("unchanged", "definitions.changeState.enum"),
    (
        "not-run",
        "definitions.verification.properties.outcome.enum",
    ),
    (
        "partial",
        "definitions.verification.properties.outcome.enum",
    ),
    ("0.1.0", "properties.schemaVersion.enum"),
    ("0.2.0", "properties.schemaVersion.enum"),
    ("0.3.0", "properties.schemaVersion.enum"),
    ("0.4.0", "properties.schemaVersion.enum"),
    ("0.5.0", "properties.schemaVersion.enum"),
    ("0.6.0", "properties.schemaVersion.enum"),
    ("0.7.0", "properties.schemaVersion.enum"),
    ("1.0.0", "properties.schemaVersion.enum"),
    ("added", "definitions.changeState.enum"),
    ("removed", "definitions.changeState.enum"),
];

/// The one file permitted to name a schema location, because it reads values
/// from the schema instead of restating them.
const SCHEMA_FACTS: &str = "schema_facts.rs";

/// The part of a source file that is not its `#[cfg(test)]` module.
///
/// A unit test's job is to name a concrete value — `task-x` as a sample
/// identifier, `0.1.0` as a sample document's declared version. That is not
/// duplication of the contract: a test asserts a behavior for one input, and it
/// cannot drift the way a hard-coded rule can, because a test that disagrees with
/// the schema fails loudly. Only non-test code is held to the derivation rule.
///
/// The split is on the `#[cfg(test)]` attribute at the start of a line, which by
/// convention opens the last item in the file. Requiring column zero is what
/// keeps the attribute *inside a string literal* — a test may quote it — from
/// being mistaken for the module itself. A file that puts rule code after its
/// test module would go unchecked, so `no_rule_code_follows_a_test_module`
/// forbids that arrangement rather than leaving the gap open.
fn rule_code_of(text: &str) -> &str {
    match test_module_offset(text) {
        Some(at) => &text[..at],
        None => text,
    }
}

/// The byte offset of the `#[cfg(test)]` attribute that opens the test module.
///
/// Only an occurrence at the start of a line counts, so an indented one or one
/// inside a string literal does not.
fn test_module_offset(text: &str) -> Option<usize> {
    const ATTRIBUTE: &str = "#[cfg(test)]";
    if text.starts_with(ATTRIBUTE) {
        return Some(0);
    }
    text.find(&format!("\n{ATTRIBUTE}")).map(|at| at + 1)
}

fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            found.extend(rust_sources(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
    found.sort();
    found
}

#[test]
fn rule_code_does_not_restate_schema_expressible_values() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = rust_sources(&src);
    assert!(!files.is_empty(), "no rule source found to check");

    let mut offenses: Vec<String> = Vec::new();
    for file in files {
        let name = file
            .file_name()
            .expect("named file")
            .to_string_lossy()
            .into_owned();
        if name == SCHEMA_FACTS {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("source is readable");
        for (line_number, line) in rule_code_of(&text).lines().enumerate() {
            // A doc comment may quote the contract. Only code counts.
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for (literal, owner) in SCHEMA_OWNED_LITERALS {
                // The match is on the *whole* string literal. Restating a
                // contract value looks like `"task-"` — the exact value, used to
                // build or match an identifier. A longer literal that merely
                // starts with the same letters is a different string: a rule
                // named `figure-without-caption` is a diagnostic identifier this
                // crate owns, not the schema's `figure-` prefix, and flagging it
                // would push rule names into contortions for no gain.
                if code.contains(&format!("\"{literal}\"")) {
                    offenses.push(format!(
                        "{name}:{}: the literal \"{literal}\" is owned by the schema at \
                         {owner}; read it through {SCHEMA_FACTS} instead of restating it",
                        line_number + 1
                    ));
                }
            }
        }
    }

    assert!(
        offenses.is_empty(),
        "rule code duplicates schema-expressible constraints:\n{}",
        offenses.join("\n")
    );
}

/// The test-module exemption only holds if the test module is last.
///
/// `rule_code_of` stops scanning at `#[cfg(test)]`, so rule code placed after a
/// test module would escape the derivation rule entirely. Rather than parse Rust
/// to close that hole, this test forbids the arrangement: a `#[cfg(test)]` module
/// must be the final item in its file.
#[test]
fn no_rule_code_follows_a_test_module() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for file in rust_sources(&src) {
        let text = std::fs::read_to_string(&file).expect("source is readable");
        let Some(at) = test_module_offset(&text) else {
            continue;
        };
        let name = file
            .file_name()
            .expect("named file")
            .to_string_lossy()
            .into_owned();
        // Everything after the test module must be the module's own body and its
        // closing brace. A second module, or any further item, means the module is
        // not last and the scan above would have skipped real rule code.
        let after = &text[at + "#[cfg(test)]".len()..];
        assert!(
            test_module_offset(after).is_none(),
            "{name} has two test modules; the derivation scan only reaches the first, \
             so move them into one module at the end of the file"
        );
        let trailing = after
            .rsplit_once('}')
            .map(|(_, tail)| tail.trim())
            .unwrap_or_default();
        assert!(
            trailing.is_empty(),
            "{name} has code after its test module ({trailing:?}); the derivation scan \
             does not reach it, so move the test module to the end of the file"
        );
    }
}

#[test]
fn the_schema_facts_module_reads_the_schema_rather_than_copying_it() {
    // The rule engine's door onto the facts must resolve to the schema crate's
    // reader, and that reader must derive its values from the published schema
    // text. Checking the door alone would pass for a module that re-implemented
    // the facts, so both files are checked.
    let door = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(SCHEMA_FACTS);
    let door_text = std::fs::read_to_string(&door).expect("schema_facts.rs exists");
    assert!(
        door_text.contains("htmlspec_schema::facts"),
        "{SCHEMA_FACTS} must expose the schema crate's reader rather than its own"
    );

    let reader = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../htmlspec-schema/src/facts.rs")
        .canonicalize()
        .expect("the schema crate's facts module exists");
    let reader_text = std::fs::read_to_string(&reader).expect("facts.rs is readable");
    assert!(
        reader_text.contains("crate::schema_text::DOCUMENT"),
        "the facts reader must derive its values from the published schema"
    );
}

/// Every prefix the schema defines is reachable through the facts module, so a
/// new unit type cannot be added to the schema and quietly ignored by the rules.
///
/// The figure prefix is reached separately, because a figure is not a semantic
/// unit and must not appear among the unit prefixes.
#[test]
fn every_schema_prefix_is_exposed_to_rule_code() {
    let expected: Vec<&str> = SCHEMA_OWNED_LITERALS
        .iter()
        .filter(|(_, owner)| owner.ends_with("Id.pattern"))
        .map(|(literal, _)| *literal)
        .collect();
    let units = htmlspec_validate::schema_facts::identifier_prefixes();
    let figure = htmlspec_validate::schema_facts::figure_prefix();
    for prefix in expected {
        let visible = units.iter().any(|(_, p)| p == prefix) || figure.as_deref() == Some(prefix);
        assert!(
            visible,
            "the schema defines the {prefix} prefix and rule code cannot see it"
        );
    }
}
