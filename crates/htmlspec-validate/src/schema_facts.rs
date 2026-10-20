//! Contract facts, read out of the published schema at run time.
//!
//! The reader itself lives in `htmlspec_schema::facts`, because the schema crate
//! owns the schema and the parser needs the same facts to decide whether a
//! fragment link carries a relationship. This module is the rule engine's door
//! onto it, so `tests/schema_rust_split.rs` still has one file to point at when a
//! rule tries to hard-code a value the schema owns.

pub use htmlspec_schema::facts::{
    artifact_kind, artifact_kinds, contract_version, enum_values, figure_prefix,
    identifier_prefixes, is_unit_identifier, prefix_for_unit,
};
