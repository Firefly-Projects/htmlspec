//! Generates Rust types from the projected-model JSON Schema.
//!
//! Generation runs schema-to-types so the checked-in schema is the authored
//! artifact and the Rust types are its output. See design.md, "JSON Schema is
//! the source of truth; Rust types are generated from it".

use std::path::Path;

use typify::{TypeSpace, TypeSpaceSettings};

const SCHEMAS: &[&str] = &[
    "schema/document.schema.json",
    "schema/requirement-spike.schema.json",
];

fn main() {
    for schema in SCHEMAS {
        println!("cargo:rerun-if-changed={schema}");
        generate(schema);
    }
    println!("cargo:rerun-if-changed=build.rs");
}

fn generate(schema_path: &str) {
    let text = std::fs::read_to_string(schema_path)
        .unwrap_or_else(|e| panic!("cannot read {schema_path}: {e}"));
    let schema: schemars::schema::RootSchema =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("cannot parse {schema_path}: {e}"));

    let mut type_space = TypeSpace::new(TypeSpaceSettings::default().with_struct_builder(false));
    type_space
        .add_root_schema(schema)
        .unwrap_or_else(|e| panic!("cannot add {schema_path}: {e}"));

    let contents = prettyplease::unparse(
        &syn::parse2::<syn::File>(type_space.to_stream()).expect("generated code parses"),
    );

    let stem = Path::new(schema_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("schema path has a stem")
        .replace(['.', '-'], "_");
    let out =
        Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR is set")).join(format!("{stem}.rs"));
    std::fs::write(&out, contents).unwrap_or_else(|e| panic!("cannot write {out:?}: {e}"));
}
