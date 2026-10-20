//! Every structured payload an integration test produces, held to the published schema of the
//! operation that wrote it.
//!
//! toolchain-packaging's command-surface stability requirement, as the change
//! `commit-the-contract-and-the-command-line` holds it (its design's decision 5): each operation
//! that offers `--json` publishes a JSON Schema for its payload under `schema/payload/`, listing
//! every field it can emit. **Two readings of one schema.** The published one admits a property it
//! does not declare, because a later 1.x release can add one and a consumer ignores what it does not
//! know. The suite reads the same file as its [`oracle`], with every object refusing an undeclared
//! property, because the program is held to exactly what its schema declares. A payload carrying a
//! field the schema does not declare, or a declared field of another type, fails the test that
//! produced it here, and so does one the published schema refuses, such as a payload two branches of
//! a `oneOf` admit once neither refuses the other's fields.
//!
//! Every helper in `tests/` that runs the binary passes its arguments and its standard output to
//! [`held`], so no test can produce a payload this does not read. Which operation an invocation
//! spells, and which schema its payload answers to, is read from the newest file under
//! `tests/surface/`, the command surface the suite holds the program to.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::Value;

/// The crate's own directory, where `schema/` and `tests/surface/` live.
fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every operation's spellings, and the payload each writes, from the newest released surface.
struct Spelled {
    /// Each spelling as the words that follow `htmlspec`, OpenSpec's and the request verb's.
    spellings: Vec<Vec<String>>,
    /// The payload schema's name, or `None` for an operation that writes no payload.
    payload: Option<String>,
}

/// The surface the suite holds the program to: the file of the newest released minor.
fn spelled() -> &'static Vec<Spelled> {
    static SPELLED: OnceLock<Vec<Spelled>> = OnceLock::new();
    SPELLED.get_or_init(|| {
        let directory = manifest().join("tests/surface");
        let newest = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter_map(|path| Some((released_minor(&path)?, path)))
            .max()
            .map(|(_, path)| path)
            .expect("tests/surface holds a released surface");
        let surface: Value = read_json(&newest);
        surface["operations"]
            .as_object()
            .expect("the surface names its operations")
            .values()
            .map(|operation| Spelled {
                spellings: operation["spellings"]
                    .as_array()
                    .expect("an operation lists its spellings")
                    .iter()
                    .map(|spelling| {
                        spelling
                            .as_str()
                            .expect("a spelling is a string")
                            .split(' ')
                            .map(str::to_owned)
                            .collect()
                    })
                    .collect(),
                payload: operation["payload"].as_str().map(str::to_owned),
            })
            .collect()
    })
}

/// The major and minor a file under `tests/surface/` is named for: `1.0.json` is `(1, 0)`.
fn released_minor(path: &Path) -> Option<(u64, u64)> {
    let stem = path.file_name()?.to_str()?.strip_suffix(".json")?;
    let (major, minor) = stem.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()))
}

/// The published payload schema named `name`.
fn schema_path(name: &str) -> PathBuf {
    manifest()
        .join("schema/payload")
        .join(format!("{name}.schema.json"))
}

/// `schema` as the suite reads it: every object that declares properties refuses any other.
///
/// The published schema leaves undeclared properties admitted, for consumers. An object whose
/// `additionalProperties` the schema states already, a map's value schema, keeps it.
pub fn oracle(schema: &Value) -> Value {
    match schema {
        Value::Object(node) => {
            let mut strict: serde_json::Map<String, Value> = node
                .iter()
                .map(|(key, value)| (key.clone(), oracle(value)))
                .collect();
            if node.get("properties").is_some_and(Value::is_object)
                && !node.contains_key("additionalProperties")
            {
                strict.insert("additionalProperties".into(), Value::Bool(false));
            }
            Value::Object(strict)
        }
        Value::Array(elements) => Value::Array(elements.iter().map(oracle).collect()),
        other => other.clone(),
    }
}

/// The published payload schema named `name`, as consumers read it.
pub fn published(name: &str) -> Value {
    read_json(&schema_path(name))
}

/// One payload schema, compiled as published and as the suite's oracle.
struct Compiled {
    published: jsonschema::Validator,
    oracle: jsonschema::Validator,
}

/// Every payload schema, compiled once per test binary.
fn validators() -> &'static BTreeMap<String, Compiled> {
    static COMPILED: OnceLock<BTreeMap<String, Compiled>> = OnceLock::new();
    COMPILED.get_or_init(|| {
        spelled()
            .iter()
            .filter_map(|operation| operation.payload.clone())
            .map(|name| {
                let schema = published(&name);
                let compile = |schema: &Value| {
                    jsonschema::draft7::new(schema).unwrap_or_else(|error| {
                        panic!("{name}'s payload schema does not compile: {error}")
                    })
                };
                let compiled = Compiled {
                    published: compile(&schema),
                    oracle: compile(&oracle(&schema)),
                };
                (name, compiled)
            })
            .collect()
    })
}

/// Whether `arguments` ask for structured output, and the words left once the two global
/// options are taken out: `--json`, and `--store` with its value.
fn invoked<'a>(arguments: &[&'a str]) -> (bool, Vec<&'a str>) {
    let mut structured = false;
    let mut words = Vec::new();
    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        match *argument {
            "--json" => structured = true,
            "--store" => {
                rest.next();
            }
            store if store.starts_with("--store=") => {}
            word => words.push(word),
        }
    }
    (structured, words)
}

/// Holds the standard output of one run of the binary to the schema of the payload it wrote.
///
/// Returns the payload's schema name and the payload, where the run wrote one, so a test that
/// needs the payload reads the value this checked. Panics, failing the test that ran the binary,
/// where a payload does not conform, where standard output under `--json` is not a payload, or
/// where an operation with no payload schema wrote one.
pub fn held(arguments: &[&str], stdout: &str) -> Option<(String, Value)> {
    let (structured, words) = invoked(arguments);
    if !structured || stdout.trim().is_empty() {
        return None;
    }
    let operation = spelled().iter().find(|operation| {
        operation.spellings.iter().any(|spelling| {
            words.len() >= spelling.len()
                && spelling
                    .iter()
                    .zip(&words)
                    .all(|(spelled, word)| spelled == word)
        })
    });
    let parsed: Result<Value, _> = serde_json::from_str(stdout);
    let Some(operation) = operation else {
        // Help, a version, or a usage error, none of which an operation wrote. A payload with no
        // operation behind it is the one case that is not.
        assert!(
            parsed.is_err(),
            "{arguments:?} wrote a payload, and no operation in the surface spells it:\n{stdout}"
        );
        return None;
    };
    let payload = parsed.unwrap_or_else(|error| {
        panic!(
            "{arguments:?} asked for structured output and wrote no payload ({error}):\n{stdout}"
        )
    });
    let name = operation.payload.clone().unwrap_or_else(|| {
        panic!("{arguments:?} wrote a payload, and its operation publishes no payload schema:\n{stdout}")
    });
    let compiled = validators()
        .get(&name)
        .unwrap_or_else(|| panic!("no compiled schema for the {name} payload"));
    for (reading, validator) in [
        ("the suite's reading of", &compiled.oracle),
        ("the published", &compiled.published),
    ] {
        let errors: Vec<String> = validator
            .iter_errors(&payload)
            .map(|error| format!("at {}: {error}", error.instance_path()))
            .collect();
        assert!(
            errors.is_empty(),
            "{arguments:?} wrote a payload {reading} {} does not admit:\n{}\n{stdout}",
            schema_path(&name).display(),
            errors.join("\n")
        );
    }
    Some((name, payload))
}
