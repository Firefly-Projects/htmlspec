//! The projected-model schema and the Rust types generated from it.
//!
//! The JSON Schema in `schema/` is the authored source of truth. The types in
//! [`spike`] are generated from it at build time by `typify`. Generated types
//! are a convenience for writing rule code. They are not the enforcement layer:
//! `typify` drops constraints Rust cannot express, so every document must still
//! pass the runtime validator. Task 1.4 records that decision, and
//! `openspec/changes/html-spec/evidence/typify-spike.md` records the measured
//! constraint losses.

pub mod facts;

/// Types generated from `schema/document.schema.json`, the contract vocabulary.
//
// The lint allowances apply to generated code only; see the note on [`spike`].
#[allow(clippy::derivable_impls, clippy::unwrap_used)]
pub mod model {
    include!(concat!(env!("OUT_DIR"), "/document_schema.rs"));
}

/// Types generated from `schema/requirement-spike.schema.json`.
///
/// Spike output for task 1.3. This is not the contract vocabulary.
//
// The lint allowances below apply to generated code only. `typify` emits a
// hand-written `Default` impl and `unwrap` on a compiled regex literal. Neither
// is editable here, and narrowing the allowance to this module keeps the
// workspace lints in force for every hand-written line.
#[allow(clippy::derivable_impls, clippy::unwrap_used)]
pub mod spike {
    include!(concat!(env!("OUT_DIR"), "/requirement_spike_schema.rs"));
}

/// The published contract, embedded so consumers can read it without a checkout.
///
/// The contract is three pieces, not one: the schema for the projected model,
/// the normative HTML-to-model projection specification, and the conformance
/// fixtures that pin the mapping. The schema alone is not the whole contract,
/// because it describes the model and not the HTML that produces it.
pub mod schema_text {
    /// `doc/projection.md`, the normative HTML-to-model projection.
    pub const PROJECTION: &str = include_str!("../doc/projection.md");

    /// `schema/document.schema.json`, the contract vocabulary.
    pub const DOCUMENT: &str = include_str!("../schema/document.schema.json");

    /// `schema/requirement-spike.schema.json`.
    pub const REQUIREMENT_SPIKE: &str = include_str!("../schema/requirement-spike.schema.json");

    /// `CHANGELOG.md`, the version history and the versioning policy.
    ///
    /// The version is semantic and carries no date, so this file is where the
    /// calendar lives. A consumer reading the schema can determine when a
    /// version landed without a checkout.
    pub const CHANGELOG: &str = include_str!("../CHANGELOG.md");
}

/// The contract version a new document declares: the newest the schema accepts.
///
/// A semantic version, not a date. A major bump withdraws or reshapes part of
/// the contract, a minor bump adds vocabulary and can add warnings, and a patch
/// bump changes wording and fixes defects. Within a major from `1` on, a later
/// version reports no error on a document an earlier one accepted, which is
/// document-schema's requirement on the contract's version rather than this
/// comment's. While the major was `0` the vocabulary was provisional, and any
/// release could break a document, as `0.3.0` and `0.4.0` did. `1.0.0` is the
/// `0.7.0` vocabulary, unchanged, and commits it. The match is exact against the
/// schema's enumeration of accepted versions, so a document declaring a version
/// outside it fails schema validation with no Rust rule involved. That
/// enumeration keeps every earlier version a minor or patch bump succeeded,
/// because a document declaring one is still an existing document, and it keeps
/// `0.1.0` to `0.7.0` beside `1.x`, because `1.0.0` withdrew none of them.
/// `CHANGELOG.md` records the date each version landed.
pub const CONTRACT_VERSION: &str = "1.0.0";
