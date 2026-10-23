//! The command surface a release promises, read from the program's own parser, and the check that
//! no later release loses any of it.
//!
//! toolchain-packaging's command-surface stability requirement, as the change
//! `commit-the-contract-and-the-command-line` states it: within a major version every operation
//! keeps its name in both spellings, every flag its name and meaning, every exit code its value and
//! meaning, and every field of every `--json` payload its name, its type, and its meaning. A minor
//! release can add any of them, and only a major release removes or renames one.
//!
//! **One file per released minor**, `tests/surface/<major>.<minor>.json`, holding the parser
//! surface and the payload schemas together. Each file is the surface that minor released, so the
//! suite fails when the current surface lacks anything a file holds, when a later file lacks
//! anything an earlier one holds, and when the declared version's minor has no file. A file is
//! compared only with the files and the program of its own major, because a major release is the
//! one that removes. The current minor's file must also equal the current surface, so an addition
//! is recorded in the minor that makes it and binds every release after it.
//! [`write_the_current_minors_surface`] rewrites that file from the program, and refuses where the
//! program lacks anything a released file of its major holds, so it can only add.
//!
//! **The parser rather than `--help`.** The surface is what clap accepts, and help is wording, which
//! the requirement leaves free so messages can keep improving. Every spelling clap accepts is
//! recorded: each command under its name and every alias, visible or hidden, at every level, each
//! argument under its long name, its short one, and every alias of either, and what clap supplies
//! the whole program, `--help`, `--version`, and the `help` subcommand. A command no operation
//! spells fails the walk, so nothing the parser accepts can go unrecorded and later be dropped.
//!
//! **The payload schemas are recorded as published**, admitting a property they do not declare,
//! because a later minor can add one and a consumer ignores what it does not know. The suite holds
//! the program to them strictly all the same: `tests/support` reads every object as refusing an
//! undeclared property.
//!
//! Compiled for tests only, like `invocations`: it reads the parser and the published schemas, and
//! the program itself has no use for either.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use crate::exit::Outcome;
use crate::operation::Operation;

/// The payload schema an operation's `--json` output answers to, under `schema/payload/`, or `None`
/// for an operation that writes no payload.
///
/// No wildcard, so an operation added later does not compile until it states whether it writes a
/// payload.
fn payload(operation: Operation) -> Option<&'static str> {
    match operation {
        Operation::Validate => Some("validate"),
        Operation::Assets => Some("assets"),
        Operation::Archive => Some("archive"),
        Operation::List => Some("list"),
        Operation::Run => Some("run"),
        Operation::Preflight => Some("preflight"),
        Operation::Status => Some("status"),
        Operation::Sync => Some("sync"),
        Operation::Show => Some("show"),
        Operation::StoreList => Some("store-list"),
        Operation::Instructions => Some("instructions"),
        Operation::MigrateSurvey => Some("migrate-survey"),
        Operation::MigratePromote => Some("migrate-promote"),
        Operation::Format
        | Operation::Init
        | Operation::StoreAdd
        | Operation::StoreRemove
        | Operation::NewChange
        | Operation::NewDocument
        | Operation::NewRunbook => None,
    }
}

/// Every outcome the program exits with.
const OUTCOMES: [Outcome; 5] = [
    Outcome::Success,
    Outcome::DocumentViolation,
    Outcome::PrerequisiteFailure,
    Outcome::OperationalFailure,
    Outcome::Cancelled,
];

/// An outcome's name in the surface. No wildcard, so an outcome added later is named here, and the
/// test below finds it missing from [`OUTCOMES`].
fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Success => "success",
        Outcome::DocumentViolation => "document-violation",
        Outcome::PrerequisiteFailure => "prerequisite-failure",
        Outcome::OperationalFailure => "operational-failure",
        Outcome::Cancelled => "cancelled",
    }
}

/// The crate's own directory.
fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Where the published payload schemas live.
fn schemas() -> PathBuf {
    manifest().join("schema/payload")
}

/// Where the released surfaces live.
fn released_directory() -> PathBuf {
    manifest().join("tests/surface")
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()))
}

/// One argument as the surface records it, keyed as a contributor types it: `--strict` for an
/// option, `<1>` for the first positional.
fn argument(argument: &clap::Arg) -> (String, Value) {
    let key = match (
        argument.get_long(),
        argument.get_index(),
        argument.get_short(),
    ) {
        (Some(long), _, _) => format!("--{long}"),
        (None, Some(index), _) => format!("<{index}>"),
        (None, None, Some(short)) => format!("-{short}"),
        (None, None, None) => panic!(
            "{} is neither an option nor a positional",
            argument.get_id()
        ),
    };
    let mut record = Map::new();
    record.insert(
        "action".into(),
        format!("{:?}", argument.get_action()).into(),
    );
    record.insert(
        "takesValue".into(),
        argument.get_action().takes_values().into(),
    );
    record.insert("required".into(), argument.is_required_set().into());
    if let Some(short) = argument.get_short() {
        record.insert("short".into(), short.to_string().into());
    }
    let values: Vec<String> = argument
        .get_possible_values()
        .iter()
        .map(|value| value.get_name().to_owned())
        .collect();
    if !values.is_empty() {
        record.insert("values".into(), values.into());
    }
    let defaults: Vec<String> = argument
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    if !defaults.is_empty() {
        record.insert("defaults".into(), defaults.into());
    }
    let aliases: Vec<String> = argument
        .get_all_aliases()
        .unwrap_or_default()
        .into_iter()
        .map(str::to_owned)
        .collect();
    if !aliases.is_empty() {
        record.insert("aliases".into(), aliases.into());
    }
    let short_aliases: Vec<String> = argument
        .get_all_short_aliases()
        .unwrap_or_default()
        .into_iter()
        .map(|alias| alias.to_string())
        .collect();
    if !short_aliases.is_empty() {
        record.insert("shortAliases".into(), short_aliases.into());
    }
    (key, Value::Object(record))
}

/// The arguments `command` declares itself, leaving out the global ones, which the surface records
/// once, and clap's own help and version.
fn arguments(command: &clap::Command) -> Value {
    Value::Object(
        command
            .get_arguments()
            .filter(|argument| !argument.is_global_set())
            .filter(|argument| !matches!(argument.get_id().as_str(), "help" | "version"))
            .map(argument)
            .collect(),
    )
}

/// The subcommand `words` spell beneath `command`.
fn beneath<'a>(command: &'a clap::Command, words: &[&str]) -> &'a clap::Command {
    words.iter().fold(command, |current, word| {
        current
            .find_subcommand(word)
            .unwrap_or_else(|| panic!("the parser has no `{}`", words.join(" ")))
    })
}

/// A command the parser accepts, as the words of its names, and as one spelling of it.
struct Leaf {
    /// The command's names, from beneath the program down: `["store", "list"]`.
    canonical: Vec<String>,
    /// The words of one spelling clap accepts for it, an alias wherever one stands.
    spelled: Vec<String>,
}

/// Every spelling of every command beneath `command` that takes no subcommand.
///
/// At each level a spelling takes the command's name or any of its aliases, visible or hidden, so
/// an alias on a verb, a parent, or a leaf is a spelling of every operation beneath it. clap's own
/// `help` subcommand is left out, and [`program`] records it. Every command must take `--help`, and
/// none may be a flag subcommand, which the surface has no place for: either fails here rather
/// than letting a spelling go unrecorded.
fn leaves(command: &clap::Command, path: &str) -> Vec<Leaf> {
    let mut found = Vec::new();
    for subcommand in command.get_subcommands() {
        let name = subcommand.get_name();
        if name == "help" {
            continue;
        }
        let at = format!("{path} {name}");
        assert!(
            subcommand
                .get_arguments()
                .any(|argument| argument.get_id() == "help"),
            "`{at}` no longer takes --help"
        );
        assert!(
            subcommand.get_short_flag().is_none() && subcommand.get_long_flag().is_none(),
            "`{at}` is a flag subcommand, which the surface does not record"
        );
        let names: Vec<&str> = std::iter::once(name)
            .chain(subcommand.get_all_aliases())
            .collect();
        let below = leaves(subcommand, &at);
        if below.is_empty() {
            for spelling in &names {
                found.push(Leaf {
                    canonical: vec![name.to_owned()],
                    spelled: vec![(*spelling).to_owned()],
                });
            }
        }
        for leaf in below {
            for spelling in &names {
                let mut canonical = vec![name.to_owned()];
                canonical.extend(leaf.canonical.iter().cloned());
                let mut spelled = vec![(*spelling).to_owned()];
                spelled.extend(leaf.spelled.iter().cloned());
                found.push(Leaf { canonical, spelled });
            }
        }
    }
    found
}

/// What clap supplies the whole program rather than one operation: `--help` and `-h`, which the
/// walk in [`leaves`] requires of every command, `--version` and `-V`, and the `help` subcommand.
///
/// None of them is an operation, and each is relied on: a package manager's test runs
/// `htmlspec --version`, and a person types `htmlspec help`.
fn program(built: &clap::Command) -> Value {
    let builtin = |id: &str| {
        built
            .get_arguments()
            .find(|argument| argument.get_id() == id)
            .map(argument)
            .unwrap_or_else(|| panic!("the program no longer takes clap's `{id}`"))
    };
    let arguments: Map<String, Value> = [builtin("help"), builtin("version")].into_iter().collect();
    let subcommands: Vec<String> = built
        .get_subcommands()
        .filter(|subcommand| subcommand.get_name() == "help")
        .flat_map(|subcommand| {
            std::iter::once(subcommand.get_name()).chain(subcommand.get_all_aliases())
        })
        .map(str::to_owned)
        .collect();
    json!({ "arguments": arguments, "subcommands": subcommands })
}

/// The command surface as the program presents it now.
pub fn current() -> Value {
    let mut built = crate::command();
    built.build();

    let global = Value::Object(
        built
            .get_arguments()
            .filter(|argument| argument.is_global_set())
            .map(argument)
            .collect(),
    );

    let leaves = leaves(&built, "htmlspec");
    let spelling_of =
        |words: &[&str]| -> Vec<String> { words.iter().map(|word| (*word).to_owned()).collect() };
    let operation_spellings: Vec<(Operation, Vec<String>, Vec<String>)> = Operation::ALL
        .into_iter()
        .map(|operation| {
            let (verb, object) = operation.request();
            (
                operation,
                spelling_of(operation.spelling()),
                spelling_of(&[verb.word(), object]),
            )
        })
        .collect();
    for leaf in &leaves {
        assert!(
            operation_spellings
                .iter()
                .any(|(_, openspec, request)| leaf.canonical == *openspec
                    || leaf.canonical == *request),
            "`htmlspec {}` is a command no operation spells",
            leaf.spelled.join(" ")
        );
    }

    let mut operations = Map::new();
    let mut payloads = Map::new();
    for (operation, openspec, request) in &operation_spellings {
        let operation = *operation;
        let (verb, object) = operation.request();
        let named = beneath(&built, operation.spelling());
        let requested = beneath(&built, &[verb.word(), object]);
        let taken = arguments(named);
        assert_eq!(
            taken,
            arguments(requested),
            "`htmlspec {}` and `htmlspec {} {object}` take different arguments",
            operation.spelled(),
            verb.word()
        );
        // The two names first, then every spelling an alias makes, in the parser's order.
        let mut spellings = vec![openspec.join(" "), request.join(" ")];
        for leaf in &leaves {
            let spelled = leaf.spelled.join(" ");
            if (leaf.canonical == *openspec || leaf.canonical == *request)
                && !spellings.contains(&spelled)
            {
                spellings.push(spelled);
            }
        }
        let mut record = Map::new();
        record.insert("spellings".into(), json!(spellings));
        record.insert("arguments".into(), taken);
        record.insert("payload".into(), json!(payload(operation)));
        if let Some(name) = payload(operation) {
            payloads.insert(
                name.to_owned(),
                read_json(&schemas().join(format!("{name}.schema.json"))),
            );
        }
        operations.insert(operation.spelled(), Value::Object(record));
    }

    let exit_codes: Map<String, Value> = OUTCOMES
        .iter()
        .map(|outcome| (outcome_name(*outcome).to_owned(), outcome.code().into()))
        .collect();

    json!({
        "version": declared_minor_name(),
        "program": program(&built),
        "global": global,
        "operations": operations,
        "exitCodes": exit_codes,
        "payloads": payloads,
    })
}

/// The major and minor of the version this crate declares.
fn declared_minor() -> (u64, u64) {
    minor_of(env!("CARGO_PKG_VERSION")).expect("the crate's version is a semantic version")
}

fn declared_minor_name() -> String {
    let (major, minor) = declared_minor();
    format!("{major}.{minor}")
}

/// The major and minor of a semantic version: `1.0.3` is `(1, 0)`.
fn minor_of(version: &str) -> Option<(u64, u64)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

/// Every released surface, oldest first, by the minor its file is named for.
fn released() -> Vec<((u64, u64), Value)> {
    let directory = released_directory();
    let mut found: Vec<((u64, u64), Value)> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .map(|path| {
            let stem = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".json"))
                .unwrap_or_else(|| panic!("{} is not a surface file", path.display()));
            let minor = minor_of(stem)
                .unwrap_or_else(|| panic!("{} is not named <major>.<minor>.json", path.display()));
            (minor, read_json(&path))
        })
        .collect();
    found.sort_by_key(|(minor, _)| *minor);
    found
}

/// The name a `oneOf` branch is known by: its `const` properties, or else its required ones, or
/// else its type.
fn branch(index: usize, node: &Value) -> String {
    let constants: Vec<String> = node["properties"]
        .as_object()
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

/// Every fact a payload schema states that a consumer can rely on, as a path and a value.
///
/// A property's type, whether it is always present, each value an enumeration admits, a
/// constant, and a reference. Descriptions and bounds are left out: they are wording, or they
/// constrain only what the program emits.
fn schema_facts(node: &Value, at: &str, out: &mut BTreeMap<String, String>) {
    match &node["type"] {
        Value::String(kind) => {
            out.insert(format!("{at}/type"), kind.clone());
        }
        Value::Array(kinds) => {
            let mut kinds: Vec<&str> = kinds.iter().filter_map(Value::as_str).collect();
            kinds.sort_unstable();
            out.insert(format!("{at}/type"), kinds.join("|"));
        }
        _ => {}
    }
    if let Some(constant) = node.get("const") {
        out.insert(format!("{at}/const"), constant.to_string());
    }
    if let Some(reference) = node["$ref"].as_str() {
        out.insert(format!("{at}/$ref"), reference.to_owned());
    }
    for value in node["enum"].as_array().into_iter().flatten() {
        out.insert(format!("{at}/enum/{value}"), "admitted".into());
    }
    for name in node["required"].as_array().into_iter().flatten() {
        if let Some(name) = name.as_str() {
            out.insert(format!("{at}/required/{name}"), "always present".into());
        }
    }
    for (name, property) in node["properties"].as_object().into_iter().flatten() {
        schema_facts(property, &format!("{at}/properties/{name}"), out);
    }
    for key in ["items", "additionalProperties", "propertyNames"] {
        if node[key].is_object() {
            schema_facts(&node[key], &format!("{at}/{key}"), out);
        }
    }
    for (index, alternative) in node["oneOf"].as_array().into_iter().flatten().enumerate() {
        schema_facts(
            alternative,
            &format!("{at}/oneOf[{}]", branch(index, alternative)),
            out,
        );
    }
    for (name, definition) in node["definitions"].as_object().into_iter().flatten() {
        schema_facts(definition, &format!("{at}/definitions/{name}"), out);
    }
}

/// Every fact one argument's record states.
fn argument_facts(record: &Value, at: &str, out: &mut BTreeMap<String, String>) {
    for key in ["action", "takesValue", "short"] {
        if let Some(value) = record.get(key) {
            out.insert(format!("{at}/{key}"), value.to_string());
        }
    }
    // An optional argument staying optional is the fact a caller relies on. A required one may
    // become optional, which breaks nobody, so required-ness is recorded only where it is absent.
    if record["required"] == Value::Bool(false) {
        out.insert(format!("{at}/optional"), "optional".into());
    }
    if let Some(defaults) = record.get("defaults") {
        out.insert(format!("{at}/defaults"), defaults.to_string());
    }
    for key in ["values", "aliases", "shortAliases"] {
        for value in record[key].as_array().into_iter().flatten() {
            out.insert(format!("{at}/{key}/{value}"), "accepted".into());
        }
    }
}

/// Every fact a surface states, as a path and a value, so that one surface holds another exactly
/// when it states every fact the other does with the same value.
fn facts(surface: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (key, record) in surface["program"]["arguments"]
        .as_object()
        .into_iter()
        .flatten()
    {
        argument_facts(record, &format!("program/arguments/{key}"), &mut out);
    }
    for subcommand in surface["program"]["subcommands"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if let Some(subcommand) = subcommand.as_str() {
            out.insert(
                format!("program/subcommands/{subcommand}"),
                "accepted".into(),
            );
        }
    }
    for (key, record) in surface["global"].as_object().into_iter().flatten() {
        argument_facts(record, &format!("global/{key}"), &mut out);
    }
    for (name, operation) in surface["operations"].as_object().into_iter().flatten() {
        let at = format!("operations/{name}");
        for spelling in operation["spellings"].as_array().into_iter().flatten() {
            if let Some(spelling) = spelling.as_str() {
                out.insert(format!("{at}/spellings/{spelling}"), "spelled".into());
            }
        }
        for (key, record) in operation["arguments"].as_object().into_iter().flatten() {
            argument_facts(record, &format!("{at}/arguments/{key}"), &mut out);
        }
        if let Some(payload) = operation["payload"].as_str() {
            out.insert(format!("{at}/payload"), payload.to_owned());
        }
    }
    for (outcome, code) in surface["exitCodes"].as_object().into_iter().flatten() {
        out.insert(format!("exitCodes/{outcome}"), code.to_string());
    }
    for (name, schema) in surface["payloads"].as_object().into_iter().flatten() {
        schema_facts(schema, &format!("payloads/{name}"), &mut out);
    }
    out
}

/// Everything `released` states that `current` does not state with the same value.
fn losses(released: &Value, current: &Value) -> Vec<String> {
    let now = facts(current);
    facts(released)
        .into_iter()
        .filter_map(|(path, was)| match now.get(&path) {
            Some(is) if *is == was => None,
            Some(is) => Some(format!("{path}: {was}, now {is}")),
            None => Some(format!("{path}: {was}, now absent")),
        })
        .collect()
}

/// Every loss the current surface shows against a released surface of the declared major, each
/// named with the minor whose file holds it.
fn losses_in_this_major(current: &Value) -> Vec<String> {
    losses_in_major(declared_minor().0, released(), current)
}

/// Every loss `current` shows against the surfaces in `released` of the major `major`.
fn losses_in_major(major: u64, released: Vec<((u64, u64), Value)>, current: &Value) -> Vec<String> {
    released
        .into_iter()
        .filter(|((released_major, _), _)| *released_major == major)
        .flat_map(|((major, minor), surface)| {
            losses(&surface, current)
                .into_iter()
                .map(move |loss| format!("{major}.{minor}: {loss}"))
        })
        .collect()
}

/// Why the declared version cannot pass, where its minor has no released surface.
fn missing_minor(declared: (u64, u64), released: &[(u64, u64)]) -> Option<String> {
    (!released.contains(&declared)).then(|| {
        format!(
            "the crate declares {}.{}, and tests/surface holds no {}.{}.json. A release that moves \
             to a new minor records its surface: run the ignored test \
             write_the_current_minors_surface",
            declared.0, declared.1, declared.0, declared.1
        )
    })
}

/// Every node of a schema that refuses a property it does not declare.
fn refuses_undeclared(node: &Value, at: &str, out: &mut Vec<String>) {
    if node["additionalProperties"] == Value::Bool(false) {
        out.push(at.to_owned());
    }
    for (name, property) in node["properties"].as_object().into_iter().flatten() {
        refuses_undeclared(property, &format!("{at}/properties/{name}"), out);
    }
    for key in ["items", "additionalProperties"] {
        if node[key].is_object() {
            refuses_undeclared(&node[key], &format!("{at}/{key}"), out);
        }
    }
    for (index, alternative) in node["oneOf"].as_array().into_iter().flatten().enumerate() {
        refuses_undeclared(alternative, &format!("{at}/oneOf/{index}"), out);
    }
    for (name, definition) in node["definitions"].as_object().into_iter().flatten() {
        refuses_undeclared(definition, &format!("{at}/definitions/{name}"), out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The file the current minor's surface is recorded in.
    fn current_minors_file() -> PathBuf {
        released_directory().join(format!("{}.json", declared_minor_name()))
    }

    /// Against the released surfaces of the declared major only, so 2.0 can remove what 1.x
    /// recorded without deleting the record of it.
    #[test]
    fn the_current_surface_holds_every_released_surface() {
        let failures = losses_in_this_major(&current());
        assert!(
            failures.is_empty(),
            "the program no longer offers what a released surface holds. Within a major only a \
             major release removes or renames one:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn every_released_surface_holds_the_ones_before_it() {
        let released = released();
        for pair in released.windows(2) {
            let ((earlier, before), (later, after)) = (&pair[0], &pair[1]);
            if earlier.0 != later.0 {
                continue;
            }
            let lost = losses(before, after);
            assert!(
                lost.is_empty(),
                "{}.{} lacks what {}.{} holds:\n{}",
                later.0,
                later.1,
                earlier.0,
                earlier.1,
                lost.join("\n")
            );
        }
    }

    #[test]
    fn the_declared_version_has_its_surface_file() {
        let minors: Vec<(u64, u64)> = released().into_iter().map(|(minor, _)| minor).collect();
        if let Some(missing) = missing_minor(declared_minor(), &minors) {
            panic!("{missing}");
        }
    }

    /// The current minor's file is the current surface, so an addition is recorded in the minor
    /// that makes it, and a later release that drops it fails the test above.
    #[test]
    fn the_current_minors_file_records_the_current_surface() {
        let path = current_minors_file();
        let recorded = read_json(&path);
        let current = current();
        let lost = losses(&recorded, &current);
        assert!(
            lost.is_empty(),
            "the program no longer offers what {} records. Restore it in the program: within a \
             major only a major release removes or renames one, and the file is not rewritten to \
             drop it:\n{}",
            path.display(),
            lost.join("\n")
        );
        let added: Vec<String> = facts(&current)
            .into_iter()
            .filter(|(fact, value)| facts(&recorded).get(fact) != Some(value))
            .map(|(fact, value)| format!("{fact}: {value}"))
            .collect();
        assert!(
            recorded == current,
            "{} does not record what the program now adds. Run `cargo test -p htmlspec-cli --bin \
             htmlspec -- --ignored write_the_current_minors_surface` to record it, and review the \
             diff. Facts the file does not record as the program states them:\n{}",
            path.display(),
            added.join("\n")
        );
    }

    /// Rewrites the current minor's surface file from the program, refusing where the program
    /// lacks anything a released surface of its major holds, so the file can only gain.
    #[test]
    #[ignore = "writes tests/surface; run it to record an addition to the surface"]
    fn write_the_current_minors_surface() {
        let current = current();
        let lost = losses_in_this_major(&current);
        assert!(
            lost.is_empty(),
            "refusing to record a surface that lacks what a released surface of this major holds. \
             Restore it in the program: within a major only a major release removes or renames \
             one:\n{}",
            lost.join("\n")
        );
        let mut text = serde_json::to_string_pretty(&current).expect("the surface serializes");
        text.push('\n');
        std::fs::write(current_minors_file(), text).expect("the surface file is written");
    }

    /// The refusal the writer makes, on the current minor's own file: a surface that lost a flag
    /// it records is one the writer will not record.
    #[test]
    fn a_loss_against_the_current_minors_file_is_refused_by_the_writer() {
        let recorded = read_json(&current_minors_file());
        let mut later = recorded.clone();
        later["operations"]["validate"]["arguments"]
            .as_object_mut()
            .expect("validate's arguments")
            .remove("--strict");
        let refused = losses_in_this_major(&later);
        assert!(
            refused
                .iter()
                .any(|loss| loss.starts_with("1.0: operations/validate/arguments/--strict/")),
            "{refused:?}"
        );
        assert!(losses_in_this_major(&recorded).is_empty());
    }

    /// Every outcome in [`OUTCOMES`], in its place.
    ///
    /// The match has no wildcard, so an outcome added to the type does not compile here until this
    /// test places it, at the index it is to take in `OUTCOMES`. A match cannot enumerate the type,
    /// so an outcome placed here and left out of `OUTCOMES` passes: the match brings whoever adds
    /// one to the list, and the list is the next line they read.
    #[test]
    fn every_outcome_is_recorded() {
        let mut placed = BTreeSet::new();
        for (index, outcome) in OUTCOMES.into_iter().enumerate() {
            let place = match outcome {
                Outcome::Success => 0,
                Outcome::DocumentViolation => 1,
                Outcome::PrerequisiteFailure => 2,
                Outcome::OperationalFailure => 3,
                Outcome::Cancelled => 4,
            };
            assert_eq!(place, index, "{outcome:?} is out of its place in OUTCOMES");
            placed.insert(place);
        }
        assert_eq!(OUTCOMES.len(), 5, "OUTCOMES and the match above disagree");
        assert_eq!(placed.len(), OUTCOMES.len(), "an outcome is listed twice");
        assert_eq!(
            current()["exitCodes"].as_object().map(Map::len),
            Some(OUTCOMES.len())
        );
    }

    /// Every spelling the parser accepts is recorded, and clap's own options and `help` are part
    /// of it.
    #[test]
    fn the_surface_records_what_clap_supplies_the_program() {
        let surface = current();
        for key in ["--help", "--version"] {
            assert!(
                surface["program"]["arguments"][key].is_object(),
                "{key} is not recorded: {:#}",
                surface["program"]
            );
        }
        assert_eq!(surface["program"]["arguments"]["--help"]["short"], "h");
        assert_eq!(surface["program"]["arguments"]["--version"]["short"], "V");
        assert_eq!(surface["program"]["subcommands"], json!(["help"]));
    }

    #[test]
    fn a_lost_builtin_or_alias_is_a_loss() {
        let lost = edited(|surface| {
            surface["program"]["arguments"]
                .as_object_mut()
                .expect("the program's arguments")
                .remove("--version");
            surface["program"]["subcommands"] = json!([]);
        });
        assert!(
            lost.iter()
                .any(|loss| loss.starts_with("program/arguments/--version/")),
            "{lost:?}"
        );
        assert!(
            lost.contains(&"program/subcommands/help: accepted, now absent".to_owned()),
            "{lost:?}"
        );
        let short = |surface: &mut Value| {
            surface["operations"]["validate"]["arguments"]["--strict"]["shortAliases"] =
                json!(["s"]);
        };
        let mut released = current();
        short(&mut released);
        assert_eq!(
            losses(&released, &current()),
            vec!["operations/validate/arguments/--strict/shortAliases/\"s\": accepted, now absent"]
        );
    }

    // -------------------------------------------------------------------------------------------
    // What the comparison catches, each seen failing on a surface edited the one way
    // -------------------------------------------------------------------------------------------

    fn edited(edit: impl FnOnce(&mut Value)) -> Vec<String> {
        let released = current();
        let mut later = released.clone();
        edit(&mut later);
        losses(&released, &later)
    }

    #[test]
    fn a_renamed_flag_is_a_loss() {
        let lost = edited(|surface| {
            let arguments = surface["operations"]["validate"]["arguments"]
                .as_object_mut()
                .expect("validate's arguments");
            let strict = arguments
                .remove("--strict")
                .expect("validate takes --strict");
            arguments.insert("--strictly".into(), strict);
        });
        assert!(
            lost.iter()
                .any(|loss| loss.starts_with("operations/validate/arguments/--strict/")),
            "{lost:?}"
        );
    }

    #[test]
    fn a_changed_exit_code_is_a_loss() {
        let lost = edited(|surface| surface["exitCodes"]["operational-failure"] = json!(4));
        assert_eq!(lost, vec!["exitCodes/operational-failure: 2, now 4"]);
    }

    #[test]
    fn a_removed_payload_field_is_a_loss() {
        let lost = edited(|surface| {
            surface["payloads"]["status"]["properties"]["tasks"]["properties"]
                .as_object_mut()
                .expect("status's tasks")
                .remove("openTasks");
        });
        assert!(
            lost.iter().any(|loss| loss
                == "payloads/status/properties/tasks/properties/openTasks/type: array, now absent"),
            "{lost:?}"
        );
    }

    #[test]
    fn a_field_whose_type_changed_is_a_loss() {
        let lost = edited(|surface| {
            surface["payloads"]["status"]["properties"]["tasks"]["properties"]["open"] =
                json!({ "type": "string" });
        });
        assert!(
            lost.iter().any(|loss| loss
                == "payloads/status/properties/tasks/properties/open/type: integer, now string"),
            "{lost:?}"
        );
    }

    #[test]
    fn a_field_no_longer_always_present_is_a_loss() {
        let lost = edited(|surface| {
            surface["payloads"]["status"]["properties"]["tasks"]["required"]
                .as_array_mut()
                .expect("required")
                .retain(|name| name != "open");
        });
        assert!(
            lost.iter()
                .any(|loss| loss.ends_with("/tasks/required/open: always present, now absent")),
            "{lost:?}"
        );
    }

    #[test]
    fn a_withdrawn_value_and_a_lost_spelling_are_losses() {
        let lost = edited(|surface| {
            surface["payloads"]["preflight"]["properties"]["checks"]["items"]["properties"]
                ["outcome"]["enum"]
                .as_array_mut()
                .expect("an enumeration")
                .retain(|value| value != "timed-out");
            surface["operations"]["validate"]["spellings"] = json!(["validate"]);
        });
        assert!(
            lost.iter()
                .any(|loss| loss.contains("/outcome/enum/\"timed-out\"")),
            "{lost:?}"
        );
        assert!(
            lost.iter()
                .any(|loss| loss.starts_with("operations/validate/spellings/get validity")),
            "{lost:?}"
        );
    }

    #[test]
    fn an_optional_argument_made_required_is_a_loss() {
        let lost = edited(|surface| {
            surface["operations"]["validate"]["arguments"]["--strict"]["required"] = json!(true);
        });
        assert_eq!(
            lost,
            vec!["operations/validate/arguments/--strict/optional: optional, now absent"]
        );
    }

    #[test]
    fn an_addition_is_not_a_loss() {
        let lost = edited(|surface| {
            surface["operations"]["validate"]["arguments"]["--quietly"] =
                json!({ "action": "SetTrue", "takesValue": false, "required": false });
            surface["payloads"]["status"]["properties"]["tasks"]["properties"]["dropped"] =
                json!({ "type": "integer" });
            surface["exitCodes"]["interrupted"] = json!(130);
            surface["operations"]["validate"]["arguments"]["--strict"]["short"] = json!("s");
        });
        assert!(lost.is_empty(), "{lost:?}");
    }

    #[test]
    fn a_major_release_is_compared_only_with_its_own_major() {
        let released = current();
        let mut later = released.clone();
        later["operations"]["format"]["arguments"]
            .as_object_mut()
            .expect("format's arguments")
            .remove("--fallback");
        let files = || vec![((1, 0), released.clone()), ((1, 1), released.clone())];
        assert!(losses_in_major(2, files(), &later).is_empty());
        let lost = losses_in_major(1, files(), &later);
        assert!(
            lost.iter()
                .any(|loss| loss.starts_with("1.0: operations/format/arguments/--fallback/"))
                && lost
                    .iter()
                    .any(|loss| loss.starts_with("1.1: operations/format/arguments/--fallback/")),
            "{lost:?}"
        );
    }

    #[test]
    fn a_version_moved_to_a_minor_with_no_file_is_refused() {
        assert!(missing_minor((1, 1), &[(1, 0)]).is_some());
        assert!(missing_minor((2, 0), &[(1, 0), (1, 1)]).is_some());
        assert!(missing_minor((1, 0), &[(1, 0)]).is_none());
    }

    // -------------------------------------------------------------------------------------------
    // The payload schemas themselves
    // -------------------------------------------------------------------------------------------

    /// Every published payload schema, by name.
    fn published() -> BTreeMap<String, Value> {
        std::fs::read_dir(schemas())
            .expect("schema/payload is readable")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .map(|path| {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| name.strip_suffix(".schema.json"))
                    .unwrap_or_else(|| panic!("{} is not a payload schema", path.display()))
                    .to_owned();
                (name, read_json(&path))
            })
            .collect()
    }

    #[test]
    fn every_payload_schema_compiles_and_is_some_operations() {
        let named: BTreeSet<&str> = Operation::ALL.into_iter().filter_map(payload).collect();
        let published = published();
        assert_eq!(
            published
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            named,
            "schema/payload and the operations that write a payload disagree"
        );
        for (name, schema) in &published {
            assert_eq!(
                schema["$schema"], "http://json-schema.org/draft-07/schema#",
                "{name} names another draft"
            );
            if let Err(error) = jsonschema::draft7::new(schema) {
                panic!("{name}'s payload schema does not compile: {error}");
            }
        }
    }

    /// No published object refuses a property it does not declare, because a later minor can add
    /// one and a consumer validating that payload against this schema must still accept it. The
    /// suite reads every object as refusing, in `tests/support`, so the program is held to exactly
    /// what each schema declares all the same.
    #[test]
    fn every_published_payload_schema_admits_an_undeclared_property() {
        let mut closed = Vec::new();
        for (name, schema) in published() {
            refuses_undeclared(&schema, &name, &mut closed);
        }
        assert!(
            closed.is_empty(),
            "these objects refuse a property their schema does not declare, which a minor release \
             can add:\n{}",
            closed.join("\n")
        );
    }

    /// `run --as` writes the preflight report in place of a run when the machine falls short, so
    /// run's schema states that report as `preflight`'s schema does.
    #[test]
    fn a_refused_run_writes_the_preflight_report() {
        let published = published();
        let report = &published["run"]["definitions"]["preflightReport"];
        let preflight = &published["preflight"];
        for key in ["type", "required", "properties"] {
            assert_eq!(
                report[key], preflight[key],
                "run's preflightReport and the preflight payload differ in {key}"
            );
        }
        assert!(published["run"]["oneOf"]
            .as_array()
            .expect("run's shapes")
            .contains(&json!({ "$ref": "#/definitions/preflightReport" })));
    }

    /// A definition two schemas share by name says the same thing in both, because a consumer
    /// that parses a diagnostic from one payload parses it from the others.
    #[test]
    fn a_definition_shared_by_name_is_the_same_in_every_schema() {
        let mut seen: BTreeMap<String, (String, Value)> = BTreeMap::new();
        for (name, schema) in published() {
            for (definition, node) in schema["definitions"].as_object().into_iter().flatten() {
                match seen.get(definition) {
                    Some((first, held)) => assert_eq!(
                        held, node,
                        "{name} and {first} define {definition} differently"
                    ),
                    None => {
                        seen.insert(definition.clone(), (name.clone(), node.clone()));
                    }
                }
            }
        }
    }
}
