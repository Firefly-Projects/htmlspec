//! Which fields of the published payload schemas the payloads a test produced carry.
//!
//! The other half of what `support` holds. A payload validating against its schema shows that it
//! emits nothing the schema leaves out. It does not show that the schema's fields are still
//! emitted: a field the code stopped writing, or one only a rare input writes, leaves every payload
//! valid. So every property a schema declares has to appear in at least one payload a test
//! produced, and the change's design names the fields ordinary fixtures never produce.
//!
//! A property is named by where the schema declares it. One inside a shared definition, such as a
//! diagnostic's `note`, is named by the definition, `#diagnostic/properties/note`, so a payload of
//! any operation that carries one covers it. Everything else is named by its schema and its path,
//! `validate:/properties/results/items/properties/error`. A branch of a `oneOf` is named by what
//! tells it apart: its `const` properties, or else its required ones.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Map, Value};

/// The declared properties of every published payload schema, and those a payload has carried.
pub struct Coverage {
    schemas: BTreeMap<String, Value>,
    declared: BTreeSet<String>,
    covered: BTreeSet<String>,
}

impl Coverage {
    /// Every schema under `directory`, with nothing covered yet.
    pub fn of(directory: &Path) -> Self {
        let mut schemas = BTreeMap::new();
        for entry in std::fs::read_dir(directory).expect("the payload schemas are readable") {
            let path = entry.expect("an entry").path();
            let Some(name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".schema.json"))
            else {
                continue;
            };
            let text = std::fs::read_to_string(&path).expect("a schema is readable");
            let schema: Value = serde_json::from_str(&text).expect("a schema is JSON");
            schemas.insert(name.to_owned(), schema);
        }
        let mut declared = BTreeSet::new();
        for (name, schema) in &schemas {
            declare(schema, &format!("{name}:"), &mut declared);
            if let Some(definitions) = schema["definitions"].as_object() {
                for (definition, node) in definitions {
                    declare(node, &format!("#{definition}"), &mut declared);
                }
            }
        }
        Self {
            schemas,
            declared,
            covered: BTreeSet::new(),
        }
    }

    /// Records the properties `payload`, a payload of the schema `name`, carries.
    pub fn cover(&mut self, name: &str, payload: &Value) {
        let schema = self
            .schemas
            .get(name)
            .unwrap_or_else(|| panic!("no payload schema named {name}"))
            .clone();
        cover(
            &schema,
            &schema,
            &format!("{name}:"),
            payload,
            &mut self.covered,
        );
    }

    /// Every declared property no payload carried, by the name it is declared under.
    pub fn uncovered(&self) -> Vec<&str> {
        self.declared
            .difference(&self.covered)
            .map(String::as_str)
            .collect()
    }

    /// How many properties the schemas declare.
    pub fn declared(&self) -> usize {
        self.declared.len()
    }
}

/// The name a `oneOf` branch is known by.
fn branch(index: usize, node: &Value) -> String {
    let properties = node["properties"].as_object();
    let constants: Vec<String> = properties
        .into_iter()
        .flatten()
        .filter_map(|(name, property)| Some(format!("{name}={}", property.get("const")?)))
        .collect();
    if !constants.is_empty() {
        return constants.join(",");
    }
    if let Some(required) = node["required"].as_array() {
        let mut names: Vec<&str> = required.iter().filter_map(Value::as_str).collect();
        names.sort_unstable();
        return format!("{{{}}}", names.join(","));
    }
    match &node["type"] {
        Value::String(kind) => kind.clone(),
        _ => index.to_string(),
    }
}

/// Every property `node` declares beneath `at`, stopping at a reference, whose definition is
/// declared under its own name.
fn declare(node: &Value, at: &str, out: &mut BTreeSet<String>) {
    if node.get("$ref").is_some() {
        return;
    }
    if let Some(properties) = node["properties"].as_object() {
        for (name, property) in properties {
            let key = format!("{at}/properties/{name}");
            out.insert(key.clone());
            declare(property, &key, out);
        }
    }
    if node["items"].is_object() {
        declare(&node["items"], &format!("{at}/items"), out);
    }
    if node["additionalProperties"].is_object() {
        declare(
            &node["additionalProperties"],
            &format!("{at}/additionalProperties"),
            out,
        );
    }
    if let Some(branches) = node["oneOf"].as_array() {
        for (index, alternative) in branches.iter().enumerate() {
            declare(
                alternative,
                &format!("{at}/oneOf[{}]", branch(index, alternative)),
                out,
            );
        }
    }
}

/// Whether `value` is of the JSON type `kind` names.
fn is_type(kind: &str, value: &Value) -> bool {
    match kind {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "integer" => value.is_u64() || value.is_i64(),
        "number" => value.is_number(),
        "string" => value.is_string(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => false,
    }
}

/// Whether `value` is an instance of the `oneOf` branch `node`, judged by what tells the branches
/// of these schemas apart: the type, the required properties, the declared ones, and the constants.
fn matches(node: &Value, value: &Value) -> bool {
    let typed = match &node["type"] {
        Value::String(kind) => is_type(kind, value),
        Value::Array(kinds) => kinds
            .iter()
            .filter_map(Value::as_str)
            .any(|kind| is_type(kind, value)),
        _ => true,
    };
    if !typed {
        return false;
    }
    let (Some(properties), Some(object)) = (node["properties"].as_object(), value.as_object())
    else {
        return true;
    };
    let required = node["required"].as_array().into_iter().flatten();
    required
        .filter_map(Value::as_str)
        .all(|name| object.contains_key(name))
        && object.keys().all(|key| properties.contains_key(key))
        && properties.iter().all(|(name, property)| {
            property
                .get("const")
                .is_none_or(|constant| object.get(name) == Some(constant))
        })
}

/// Records the properties `value` carries, walking it beside `node`, a node of `root`.
fn cover(root: &Value, node: &Value, at: &str, value: &Value, out: &mut BTreeSet<String>) {
    if let Some(reference) = node["$ref"].as_str() {
        let name = reference
            .strip_prefix("#/definitions/")
            .unwrap_or_else(|| panic!("{reference} is not a local definition"));
        cover(
            root,
            &root["definitions"][name],
            &format!("#{name}"),
            value,
            out,
        );
        return;
    }
    if let Some(branches) = node["oneOf"].as_array() {
        if let Some((index, alternative)) = branches
            .iter()
            .enumerate()
            .find(|(_, alternative)| matches(alternative, value))
        {
            cover(
                root,
                alternative,
                &format!("{at}/oneOf[{}]", branch(index, alternative)),
                value,
                out,
            );
        }
    }
    match value {
        Value::Object(object) => cover_object(root, node, at, object, out),
        Value::Array(elements) if node["items"].is_object() => {
            for element in elements {
                cover(root, &node["items"], &format!("{at}/items"), element, out);
            }
        }
        _ => {}
    }
}

fn cover_object(
    root: &Value,
    node: &Value,
    at: &str,
    object: &Map<String, Value>,
    out: &mut BTreeSet<String>,
) {
    if let Some(properties) = node["properties"].as_object() {
        for (name, element) in object {
            if let Some(property) = properties.get(name) {
                let key = format!("{at}/properties/{name}");
                out.insert(key.clone());
                cover(root, property, &key, element, out);
            }
        }
    }
    if node["additionalProperties"].is_object() {
        for element in object.values() {
            cover(
                root,
                &node["additionalProperties"],
                &format!("{at}/additionalProperties"),
                element,
                out,
            );
        }
    }
}
