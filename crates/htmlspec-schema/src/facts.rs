//! Reads contract facts out of the published schema at run time.
//!
//! This is the only module in the workspace permitted to name a location inside
//! the schema. Everything else asks this module, so a schema change moves one
//! file rather than every consumer that happens to know a prefix or an
//! enumerated value. Nothing anywhere may hard-code such a value; the test
//! `crates/htmlspec-validate/tests/schema_rust_split.rs` enforces that for rule
//! code.
//!
//! It lives in this crate rather than in the rule engine because this crate owns
//! the schema. The parser needs the identifier grammar to decide whether a
//! fragment link is a reference, and the rule engine needs the prefixes and the
//! enumerations; one reader serves both.

use serde_json::Value;

/// The parsed schema. Parsed once, because the text is embedded at compile time
/// and cannot fail to be present.
fn document_schema() -> &'static Value {
    static SCHEMA: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    SCHEMA.get_or_init(|| {
        serde_json::from_str(crate::schema_text::DOCUMENT)
            .expect("the embedded schema is valid JSON")
    })
}

/// Every reserved class name, paired with the identifier prefix its unit uses.
///
/// Derived from the schema's `definitions`: a definition is a unit when it
/// declares a `unit` discriminator, and its prefix is read from the pattern of
/// the identifier definition its `id` field references.
pub fn identifier_prefixes() -> Vec<(String, String)> {
    let definitions = &document_schema()["definitions"];
    let Some(map) = definitions.as_object() else {
        return Vec::new();
    };

    let mut pairs: Vec<(String, String)> = map
        .iter()
        .filter_map(|(_definition_name, definition)| {
            let unit = definition["properties"]["unit"]["const"].as_str()?;
            let id_ref = definition["properties"]["id"]["$ref"].as_str()?;
            let id_name = id_ref.rsplit('/').next()?;
            let pattern = definitions[id_name]["pattern"].as_str()?;
            Some((unit.to_owned(), prefix_of(pattern)?))
        })
        .collect();
    pairs.sort();
    pairs
}

/// The identifier prefix one unit reserves, named by its discriminator.
pub fn prefix_for_unit(unit: &str) -> Option<String> {
    identifier_prefixes()
        .into_iter()
        .find(|(name, _)| name == unit)
        .map(|(_, prefix)| prefix)
}

/// Extracts the literal prefix from an anchored identifier pattern.
///
/// `^req-[a-z0-9]+(-[a-z0-9]+)*$` yields `req-`. A pattern whose literal head
/// cannot be read yields `None`, which surfaces as a missing prefix rather than
/// as a guess.
fn prefix_of(pattern: &str) -> Option<String> {
    let body = pattern.strip_prefix('^')?;
    let head: String = body
        .chars()
        .take_while(|c| c.is_ascii_lowercase() || *c == '-')
        .collect();
    if head.ends_with('-') {
        Some(head)
    } else {
        None
    }
}

/// The identifier prefix a figure uses.
///
/// A figure is not a semantic unit, so it is absent from
/// [`identifier_prefixes`]. Its identifier still shares the document namespace,
/// which is why the uniqueness rule needs this separately.
pub fn figure_prefix() -> Option<String> {
    document_schema()["definitions"]["figureId"]["pattern"]
        .as_str()
        .and_then(prefix_of)
}

/// The permitted values of an enumerated field, read from the schema.
///
/// `path` is a slash-separated JSON pointer body, as in
/// `definitions/changeState`. An unknown path yields an empty list, which fails
/// the rule that uses it rather than silently permitting everything.
pub fn enum_values(path: &str) -> Vec<String> {
    let mut node = document_schema();
    for segment in path.split('/') {
        node = &node[segment];
    }
    node["enum"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// The artifact kinds the schema enumerates, in the order it declares them.
///
/// Named rather than left to [`enum_values`] with a path, because rule code asks
/// "is this document a specification?" and the answer is one of a closed set the
/// schema owns. A Rust `const` listing the four strings would fail
/// `rule_code_does_not_restate_schema_expressible_values`, and correctly: the
/// vocabulary is an enumeration, which is what JSON Schema expresses well.
///
/// The three per-kind rules are relational — a unit's presence against its
/// document's kind — and JSON Schema cannot express any of them, so those live in
/// Rust and read their subject from here.
pub fn artifact_kinds() -> Vec<String> {
    enum_values("properties/kind")
}

/// The kind whose name contains `stem`, or `None` when no kind does.
///
/// Rule code needs to name *one* kind — the placement rule is about a
/// specification and the completeness rule is about a tasks document — and it
/// cannot write the literal. So it names a stem and this resolves it against the
/// schema's own vocabulary. A stem matching no kind, or more than one, yields
/// `None`, which fails the rule that asked rather than silently matching nothing.
pub fn artifact_kind(stem: &str) -> Option<String> {
    let mut found = artifact_kinds().into_iter().filter(|k| k.contains(stem));
    let first = found.next()?;
    found.next().is_none().then_some(first)
}

/// Every contract version the schema accepts, oldest first.
///
/// A closed enumeration rather than one pinned value since `0.2.0`: a minor bump
/// adds a version without invalidating a document that declares an earlier one.
pub fn contract_versions() -> Vec<String> {
    enum_values("properties/schemaVersion")
}

/// The contract version a new document declares: the newest the schema accepts.
///
/// The enumeration is ordered oldest first, and
/// `the_accepted_versions_are_ordered_oldest_first` in the contract tests pins
/// that, so the newest is the last entry rather than a comparison made here.
pub fn contract_version() -> Option<String> {
    contract_versions().pop()
}

/// Whether a string matches the schema's `unitId` grammar.
///
/// The projection needs this to decide whether a fragment link is also a
/// reference (projection specification, section 5.3), and it must be the
/// schema's grammar rather than an approximation of it, or the two would
/// disagree about which links carry a relationship.
///
/// A schema with no readable `unitId` pattern matches nothing, which surfaces as
/// every reference going unrecognized rather than as everything being accepted.
pub fn is_unit_identifier(candidate: &str) -> bool {
    static PATTERN: std::sync::OnceLock<Option<regress::Regex>> = std::sync::OnceLock::new();
    let compiled = PATTERN.get_or_init(|| {
        document_schema()["definitions"]["unitId"]["pattern"]
            .as_str()
            .and_then(|pattern| regress::Regex::new(pattern).ok())
    });
    compiled
        .as_ref()
        .is_some_and(|regex| regex.find(candidate).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_cover_every_unit_in_the_schema() {
        let pairs = identifier_prefixes();
        assert_eq!(pairs.len(), 7, "expected seven units, got {pairs:?}");
        for (unit, prefix) in &pairs {
            assert!(prefix.ends_with('-'), "{unit} prefix {prefix} is malformed");
        }
    }

    #[test]
    fn a_prefix_is_read_from_the_pattern_rather_than_from_the_unit_name() {
        // The prefix and the unit name deliberately differ, which is why a rule
        // may not derive one from the other.
        assert_eq!(prefix_for_unit("verification").as_deref(), Some("verify-"));
    }

    #[test]
    fn a_figure_prefix_is_available_but_is_not_a_unit_prefix() {
        let figure = figure_prefix().expect("the schema defines a figure identifier");
        assert!(
            !identifier_prefixes().iter().any(|(_, p)| *p == figure),
            "a figure must not appear among the unit prefixes"
        );
    }

    #[test]
    fn change_states_come_from_the_schema() {
        let states = enum_values("definitions/changeState");
        assert_eq!(states.len(), 4, "{states:?}");
    }

    #[test]
    fn the_artifact_kinds_come_from_the_schema() {
        let kinds = artifact_kinds();
        assert_eq!(kinds.len(), 4, "{kinds:?}");
        // Resolved by stem, which is how rule code names one kind without
        // restating the literal the schema owns.
        assert_eq!(artifact_kind("specific").as_deref(), Some("specification"));
        assert_eq!(artifact_kind("task").as_deref(), Some("tasks"));
        assert_eq!(artifact_kind("propos").as_deref(), Some("proposal"));
        assert_eq!(artifact_kind("design").as_deref(), Some("design"));
    }

    /// An ambiguous or absent stem yields nothing, so a rule fails rather than
    /// quietly taking the first kind that happened to match.
    #[test]
    fn a_stem_matching_no_kind_or_several_resolves_to_nothing() {
        assert_eq!(artifact_kind("rumination"), None);
        // Every kind contains at least one vowel, so this matches all four.
        assert_eq!(artifact_kind("a"), None);
    }

    #[test]
    fn an_unknown_path_yields_nothing_rather_than_everything() {
        assert!(enum_values("definitions/notAThing").is_empty());
    }

    #[test]
    fn the_contract_version_is_readable() {
        assert!(contract_version().is_some());
    }

    #[test]
    fn the_unit_identifier_grammar_is_the_schemas_own() {
        assert!(is_unit_identifier("req-theme-selection"));
        assert!(is_unit_identifier("verify-parse-corpus"));
        // A figure is not a semantic unit, so its identifier is not a unit
        // identifier even though it is a legal fragment.
        assert!(!is_unit_identifier("figure-pipeline"));
        assert!(!is_unit_identifier("section-overview"));
        // The pattern is anchored, so a legal identifier with trailing junk is
        // not a match.
        assert!(!is_unit_identifier("req-theme-selection extra"));
        assert!(!is_unit_identifier("req-Theme"));
        assert!(!is_unit_identifier("req-"));
    }
}
