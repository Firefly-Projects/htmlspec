//! Every field a published payload schema declares, carried by a payload a test produces.
//!
//! toolchain-packaging's command-surface stability requirement, as the change
//! `commit-the-contract-and-the-command-line` holds it (its design's decision 5). Each run below
//! goes through `support::held`, so its payload is held to its schema like every other test's, and
//! the fields it carries are recorded. The one test then fails on any declared field no run carried:
//! a field the program stopped writing, or one only a rare input writes, which a minor release could
//! otherwise rename with every payload still valid.
//!
//! **The cases are the rare inputs**, and the review of the 1.0 plan named them: a document that
//! cannot be read, a curtailed report, the diagnostic fields only some rules supply, a listed change's
//! completion and predecessor, a cycle of predecessors, every variant of the view, and the gate's
//! findings in a promotion. Each scenario builds the smallest project that writes them, through the
//! program's own operations.

mod support;

#[path = "support/coverage.rs"]
mod coverage;

use std::path::{Path, PathBuf};
use std::process::Command;

use coverage::Coverage;
use serde_json::Value;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// The repository's own fixtures, which hold the documents the validator's rarer fields come from.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// A temporary directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-payloads-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory exists");
        std::fs::write(path, contents).expect("written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The runs a scenario makes, and the fields their payloads carried.
struct Runs {
    coverage: Coverage,
    /// The store registry every run reads, so no run touches whoever runs the suite.
    config: Scratch,
}

impl Runs {
    fn new() -> Self {
        Self {
            coverage: Coverage::of(&Path::new(env!("CARGO_MANIFEST_DIR")).join("schema/payload")),
            config: Scratch::new("config"),
        }
    }

    /// Runs the binary from `directory`, holding and recording any payload it writes.
    fn run(&mut self, directory: &Path, arguments: &[&str]) -> (i32, String, String) {
        let output = Command::new(binary())
            .current_dir(directory)
            .env("XDG_CONFIG_HOME", &self.config.0)
            .args(arguments)
            .output()
            .expect("the binary runs");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if let Some((name, payload)) = support::held(arguments, &stdout) {
            self.coverage.cover(&name, &payload);
        }
        (
            output.status.code().expect("the process exited normally"),
            stdout,
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    /// Runs an operation that must succeed, from `directory`.
    fn ok(&mut self, directory: &Path, arguments: &[&str]) -> String {
        let (code, out, err) = self.run(directory, arguments);
        assert_eq!(code, 0, "{arguments:?} failed:\n{out}\n{err}");
        out
    }

    /// Runs `arguments` with `--json`, and returns the payload, which it must write.
    fn payload(&mut self, directory: &Path, arguments: &[&str]) -> Value {
        let mut structured = vec!["--json"];
        structured.extend_from_slice(arguments);
        let (code, out, err) = self.run(directory, &structured);
        support::held(&structured, &out)
            .map(|(_, payload)| payload)
            .unwrap_or_else(|| panic!("{arguments:?} exited {code} and wrote no payload:\n{err}"))
    }
}

/// A project `init` established, with no tools.
fn project(runs: &mut Runs, label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    runs.ok(&scratch.0, &["init", "--tools", "none"]);
    scratch
}

/// Writes a specification at `relative` with `new document`, holding `requirements` in place of
/// the template's example content and the template's sections, and a summary of its own.
fn specification(runs: &mut Runs, scratch: &Scratch, relative: &str, requirements: &str) {
    runs.ok(
        &scratch.0,
        &[
            "new",
            "document",
            "specification",
            relative,
            "--subject",
            "The command surface",
        ],
    );
    let path = scratch.path(relative);
    let text = std::fs::read_to_string(&path).expect("the specification reads");
    let open = "<main class=\"hs-main\" id=\"hs-content\">";
    let start = text.find(open).expect("the template has its main") + open.len();
    let end = text.find("</main>").expect("the main closes");
    let units = format!(
        "\n        <section id=\"hs-requirements\">\n          <h2>Requirements</h2>\n          \
         {requirements}\n        </section>\n      "
    );
    let mut text = format!("{}{units}{}", &text[..start], &text[end..]);
    let placeholder = "<p>Replace this paragraph with what this document is for.";
    let start = text
        .find(placeholder)
        .expect("the template has its summary");
    let end = start + text[start..].find("</p>").expect("the summary closes") + "</p>".len();
    text.replace_range(
        start..end,
        "<p>What the command surface offers, and under which names.</p>",
    );
    let text: String = text
        .split_inclusive('\n')
        .filter(|line| {
            ![
                "#hs-decisions",
                "#hs-risks",
                "#hs-evidence",
                "#hs-verification",
            ]
            .iter()
            .any(|section| line.contains(&format!("<li><a href=\"{section}\">")))
        })
        .collect();
    std::fs::write(&path, text).expect("the specification is written");
}

/// One requirement, with `statements` and `scenarios` already marked up.
fn requirement(id: &str, change: Option<&str>, heading: &str, body: &str) -> String {
    let state = change.map_or(String::new(), |state| format!(" data-change=\"{state}\""));
    format!(
        "<section class=\"requirement\" id=\"{id}\"{state}>\n            \
         <h3>{heading} <span class=\"hs-id\">{id}</span></h3>\n            {body}\n          \
         </section>"
    )
}

/// One scenario. `given` is left out where it is empty.
fn scenario(
    id: &str,
    change: Option<&str>,
    heading: &str,
    given: &str,
    when: &str,
    then: &str,
) -> String {
    let state = change.map_or(String::new(), |state| format!(" data-change=\"{state}\""));
    let given = if given.is_empty() {
        String::new()
    } else {
        format!("<dt>Given</dt>\n                <dd>{given}</dd>\n                ")
    };
    format!(
        "<article class=\"scenario\" id=\"{id}\"{state}>\n              <h4>{heading}</h4>\n              \
         <dl>\n                {given}<dt>When</dt>\n                <dd>{when}</dd>\n                \
         <dt>Then</dt>\n                <dd>{then}</dd>\n              </dl>\n            </article>"
    )
}

const KEPT: &str = "<p>The program SHALL name each operation as OpenSpec names it.</p>";
const LISTED: &str = "<p>The names SHALL be validate, archive, and init.</p>";
const LISTED_WITH_SYNC: &str = "<p>The names SHALL be validate, archive, init, and sync.</p>";
const DELETED: &str = "<p>A name SHALL NOT change between two releases of one major version.</p>";
const ADDED: &str = "<p>Every operation MUST print its outcome as a payload when asked.</p>";

/// The accepted specification and the change's, built to write every variant of the view and
/// every set the comparison reports.
///
/// `req-a` differs four ways in its statements (held by both, paired, stated only by the change,
/// held only by the accepted side) and five ways in its scenarios (held, paired, added, withdrawn,
/// held only by the accepted side). `req-b` is held as stated, `req-c` is held by none, `req-r` is a
/// rename not taken, and `req-x` is a withdrawal not taken, which the view lists and does not
/// compare.
fn a_change_against_the_accepted_contract(runs: &mut Runs) -> Scratch {
    let scratch = project(runs, "contract");
    runs.ok(&scratch.0, &["new", "change", "a-change"]);
    runs.ok(&scratch.0, &["new", "change", "b-change"]);

    let kept = |change| {
        scenario(
            "scenario-a-kept",
            change,
            "An operation is found",
            "",
            "A contributor runs it.",
            "It is found by its name.",
        )
    };
    let altered_accepted = scenario(
        "scenario-a-altered",
        None,
        "A name is typed",
        "A project",
        "A contributor types a name.",
        "The operation runs.",
    );
    let altered = scenario(
        "scenario-a-altered",
        Some("modified"),
        "A name is typed",
        "A project",
        "A contributor types a name or its verb.",
        "The operation runs.",
    );
    let withdrawn = |change| {
        scenario(
            "scenario-a-withdrawn",
            change,
            "A name is retired",
            "",
            "A release retires a name.",
            "The name stops working.",
        )
    };
    let only_accepted = scenario(
        "scenario-a-only-accepted",
        None,
        "A name is listed",
        "",
        "A contributor asks for help.",
        "Every name is listed.",
    );
    let added = scenario(
        "scenario-a-added",
        Some("added"),
        "A payload is printed",
        "",
        "A contributor passes the flag.",
        "The payload is printed.",
    );

    let b = |change, scenario_change| {
        requirement(
            "req-b",
            change,
            "The surface reports its version",
            &format!(
                "<p>The program SHALL report its version.</p>\n            {}",
                scenario(
                    "scenario-b",
                    scenario_change,
                    "The version is asked for",
                    "",
                    "A contributor asks.",
                    "The version is printed."
                )
            ),
        )
    };
    let r = |change, heading: &str, renamed: &str, scenario_change| {
        requirement(
            "req-r",
            change,
            heading,
            &format!(
                "{renamed}<p>The program SHALL exit with a code that names the outcome.</p>\n            {}",
                scenario("scenario-r", scenario_change, "An outcome is exited", "", "An operation ends.", "The code names the outcome.")
            ),
        )
    };
    let x = |change, scenario_change| {
        requirement(
            "req-x",
            change,
            "The surface prompts",
            &format!(
                "<p>The program SHALL prompt where it cannot decide.</p>\n            {}",
                scenario(
                    "scenario-x",
                    scenario_change,
                    "A prompt is shown",
                    "",
                    "An operation cannot decide.",
                    "It asks."
                )
            ),
        )
    };

    let accepted = [
        requirement(
            "req-a",
            None,
            "The operations are named",
            &format!(
                "{KEPT}\n            {LISTED}\n            {DELETED}\n            {}\n            {altered_accepted}\n            {}\n            {only_accepted}",
                kept(None),
                withdrawn(None)
            ),
        ),
        b(None, None),
        r(None, "The surface exits", "", None),
        x(None, None),
    ]
    .join("\n          ");
    specification(
        runs,
        &scratch,
        "htmlspec/specs/surface/spec.html",
        &accepted,
    );

    let stated = [
        requirement(
            "req-a",
            Some("modified"),
            "The operations are named",
            &format!(
                "{KEPT}\n            {LISTED_WITH_SYNC}\n            {ADDED}\n            {}\n            {altered}\n            {added}\n            {}",
                kept(Some("unchanged")),
                withdrawn(Some("removed"))
            ),
        ),
        b(Some("modified"), Some("unchanged")),
        requirement(
            "req-c",
            Some("added"),
            "The surface lists what a project holds",
            &format!(
                "<p>The program SHALL list a project's changes.</p>\n            {}",
                scenario("scenario-c", Some("added"), "The changes are listed", "", "A contributor lists them.", "Each change is named.")
            ),
        ),
        r(
            Some("unchanged"),
            "The surface exits with a meaningful code",
            "<p data-field=\"renamed-from\">The surface exits</p>\n            ",
            Some("unchanged"),
        ),
        x(Some("removed"), Some("removed")),
    ]
    .join("\n          ");
    specification(
        runs,
        &scratch,
        "htmlspec/changes/a-change/specification.html",
        &stated,
    );
    let (code, out, err) = runs.run(
        &scratch.0,
        &["validate", "htmlspec/changes/a-change/specification.html"],
    );
    assert_eq!(
        code, 0,
        "the change's specification does not validate:\n{out}\n{err}"
    );

    // Each change follows the other: a predecessor for the listing and for archival, and a cycle.
    scratch.write("htmlspec/changes/a-change/FOLLOWS", "b-change\n");
    scratch.write("htmlspec/changes/b-change/FOLLOWS", "a-change\n");
    // Archival reports its findings only once the project's version has advanced.
    let declaration = std::fs::read_to_string(scratch.path("spec.nix")).expect("spec.nix reads");
    scratch.write("spec.nix", &declaration.replace("\"0.0.0\"", "\"0.1.0\""));
    scratch
}

/// A project holding the runbook `release`, which `run` can start, with a run recorded.
fn a_project_with_a_run(runs: &mut Runs) -> Scratch {
    let scratch = project(runs, "runbook");
    let runbook = scratch.path("htmlspec/runbooks/release");
    for (kind, file) in [
        ("specification", "specification.html"),
        ("tasks", "tasks.html"),
    ] {
        let path = runbook.join(file);
        runs.ok(
            &scratch.0,
            &[
                "new",
                "document",
                kind,
                &path.to_string_lossy(),
                "--subject",
                "Releasing the project",
            ],
        );
        let document = std::fs::read_to_string(&path)
            .expect("the document reads")
            .replace("-example", "-real")
            .replace(
                "Replace this paragraph with what",
                "This document states what",
            )
            .replace("data-change=\"added\"", "data-change=\"unchanged\"");
        std::fs::write(&path, document).expect("rewritten");
    }
    scratch.write("htmlspec/runbooks/release/VERSION", "0.1.0\n");
    scratch.write(
        "htmlspec/runbooks/release/CHANGELOG.md",
        "# Changelog\n\n## 0.1.0\n\nThe first release of this procedure.\n",
    );
    scratch
}

/// Declares one machine check on `release`, which no run from inside a project satisfies.
fn declare_a_machine_check(runs: &mut Runs, scratch: &Scratch) {
    let path = scratch.path("htmlspec/runbooks/release/specification.html");
    let source = std::fs::read_to_string(&path).expect("the specification reads");
    let unit = "          <section class=\"requirement\" id=\"req-outside\" data-change=\"unchanged\">\n\
         \x20           <h3>The outside prerequisite <span class=\"hs-id\">req-outside</span></h3>\n\
         \x20           <p>A run MUST start outside a project.</p>\n\
         \x20           <article class=\"scenario\" id=\"scenario-outside\" data-change=\"unchanged\">\n\
         \x20             <h4>The run starts outside</h4>\n\
         \x20             <dl>\n\
         \x20               <dt>When</dt>\n\
         \x20               <dd>A run starts.</dd>\n\
         \x20               <dt>Then</dt>\n\
         \x20               <dd>The run records that it was checked.</dd>\n\
         \x20             </dl>\n\
         \x20           </article>\n\
         \x20           <p data-field=\"note\">preflight/v2 \
         {&quot;kind&quot;:&quot;state&quot;,&quot;check&quot;:&quot;outside-project&quot;,\
         &quot;remedy&quot;:&quot;start it from a directory outside the project&quot;}</p>\n\
         \x20         </section>\n";
    let written = source.replace(
        "        </section>\n      </main>",
        &format!("{unit}        </section>\n      </main>"),
    );
    assert_ne!(written, source, "the runbook has no requirements section");
    std::fs::write(&path, written).expect("rewritten");
    runs.ok(&scratch.0, &["format", &path.to_string_lossy()]);
}

/// A migration plan whose gate finds an unknown disposition, a derivation it cannot read, and two
/// sources deriving one identifier in one destination.
const PLAN_ITEMS: &str = r#"
            <li class="task" id="task-first" data-change="added" data-state="todo">
              <h3>The first source <span class="hs-id">task-first</span></h3>
              <p data-field="note">source: openspec/specs/first/spec.md</p>
              <p data-field="note">destination: htmlspec/specs/surface/spec.html</p>
              <p data-field="note">disposition: converted</p>
              <p data-field="note">derives: The first heading -&gt; req-shared</p>
            </li>
            <li class="task" id="task-second" data-change="added" data-state="todo">
              <h3>The second source <span class="hs-id">task-second</span></h3>
              <p data-field="note">source: openspec/specs/second/spec.md</p>
              <p data-field="note">destination: htmlspec/specs/surface/spec.html</p>
              <p data-field="note">disposition: transmuted</p>
              <p data-field="note">derives: The second heading -&gt; req-shared</p>
              <p data-field="note">derives: a heading with no separator</p>
            </li>
"#;

const PLAN_SURVEY: &str = r#"{"entities":[
  {"path":"openspec/specs/first/spec.md","kind":"accepted-specification"},
  {"path":"openspec/specs/second/spec.md","kind":"accepted-specification"}
]}"#;

#[test]
fn every_field_a_payload_schema_declares_is_produced() {
    let mut runs = Runs::new();
    let here = Scratch::new("here");

    // validate: every document under the repository's fixtures, which carry a curtailed report and
    // every optional diagnostic field, and a document that cannot be read.
    let fixtures = fixtures();
    runs.payload(&here.0, &["validate", &fixtures.to_string_lossy()]);
    let unreadable = Scratch::new("unreadable");
    std::fs::write(unreadable.path("not-utf8.html"), b"<!DOCTYPE html>\xff\xfe").expect("written");
    let run = runs.payload(&here.0, &["validate", &unreadable.0.to_string_lossy()]);
    assert!(run["results"][0]["error"].is_string(), "{run}");

    // migrate survey: the repository's OpenSpec fixture, which records versions two changes share
    // and a change configuration naming a predecessor.
    runs.payload(
        &here.0,
        &[
            "migrate",
            "survey",
            &fixtures.join("migration-source").to_string_lossy(),
        ],
    );

    // instructions, assets, and the registry.
    runs.payload(&here.0, &["instructions", "apply"]);
    let contract = a_change_against_the_accepted_contract(&mut runs);
    runs.payload(&contract.0, &["assets"]);
    runs.ok(
        &here.0,
        &["store", "add", "contract", &contract.0.to_string_lossy()],
    );
    runs.payload(&here.0, &["store", "list"]);

    // The comparison, from every operation that reports it.
    let shown = runs.payload(&contract.0, &["show", "a-change", "--diff"]);
    let differs = &shown["requirements"][0]["comparison"];
    assert_eq!(differs["shown"], "differs", "{shown:#}");
    runs.payload(&contract.0, &["sync", "--change", "a-change"]);
    let archived = runs.payload(&contract.0, &["archive", "a-change", "--check"]);
    assert!(
        archived["unresolvedPredecessor"].is_object(),
        "{archived:#}"
    );
    runs.payload(&contract.0, &["status", "--change", "a-change"]);

    // The three shapes of the listing.
    let listed = runs.payload(&contract.0, &["list"]);
    assert!(
        !listed["followsCycles"]
            .as_array()
            .expect("cycles")
            .is_empty(),
        "{listed:#}"
    );
    runs.payload(&contract.0, &["list", "--specs"]);

    // A runbook: a run started, every run checked, the listing of runbooks, and a preflight.
    let runbook = a_project_with_a_run(&mut runs);
    runs.payload(&runbook.0, &["run", "release", "--as", "first"]);
    runs.payload(&runbook.0, &["run", "release", "--check"]);
    runs.payload(&runbook.0, &["list", "--runbooks"]);
    declare_a_machine_check(&mut runs, &runbook);
    let preflight = runs.payload(&runbook.0, &["preflight", "release"]);
    assert_eq!(preflight["ready"], false, "{preflight:#}");
    // A run the machine cannot start writes the preflight report in place of a run, and run's
    // schema states that shape.
    let refused = runs.payload(&runbook.0, &["run", "release", "--as", "second"]);
    assert_eq!(refused["ready"], false, "{refused:#}");
    assert!(
        refused.get("record").is_none(),
        "a refused run names no record: {refused:#}"
    );

    // A promotion's coverage gate.
    let migration = project(&mut runs, "migration");
    runs.ok(
        &migration.0,
        &[
            "new",
            "document",
            "tasks",
            "htmlspec/changes/m/tasks.html",
            "--subject",
            "Migration plan",
        ],
    );
    let path = migration.path("htmlspec/changes/m/tasks.html");
    let template = std::fs::read_to_string(&path).expect("the plan reads");
    let opening = template
        .find("<li class=\"task\" id=\"task-example\"")
        .expect("the template writes an example task");
    let closing = template.rfind("</li>").expect("the example tasks close") + "</li>".len();
    std::fs::write(
        &path,
        format!(
            "{}{PLAN_ITEMS}{}",
            &template[..opening],
            &template[closing..]
        ),
    )
    .expect("the plan is written");
    migration.write("survey.json", PLAN_SURVEY);
    runs.payload(
        &migration.0,
        &[
            "migrate",
            "promote",
            "htmlspec/changes/m/tasks.html",
            "--survey",
            "survey.json",
        ],
    );

    let uncovered = runs.coverage.uncovered();
    assert!(
        uncovered.is_empty(),
        "{} of the {} properties the payload schemas declare appear in no payload these runs \
         wrote. Add a run whose payload carries each, or the field can be renamed with every \
         payload still valid:\n{}",
        uncovered.len(),
        runs.coverage.declared(),
        uncovered.join("\n")
    );
}

/// A property a later minor adds is one a consumer's validation of the published schema admits,
/// and one the suite's reading of the same schema refuses, so the program cannot write it before
/// its schema declares it.
#[test]
fn an_added_property_passes_the_published_schema_and_fails_the_suites_reading() {
    let schema = support::published("status");
    let mut payload = serde_json::json!({
        "change": "a-change",
        "artifacts": [{ "name": "proposal.html", "present": true }],
        "tasks": {
            "total": 1,
            "open": 1,
            "openTasks": ["task-1-1"],
            "doneDifferently": 0,
            "doneDifferentlyTasks": []
        }
    });
    let published = jsonschema::draft7::new(&schema).expect("the schema compiles");
    let oracle = jsonschema::draft7::new(&support::oracle(&schema)).expect("the oracle compiles");
    for validator in [&published, &oracle] {
        let errors: Vec<String> = validator
            .iter_errors(&payload)
            .map(|error| error.to_string())
            .collect();
        assert!(
            errors.is_empty(),
            "the sample is not a status payload: {errors:?}"
        );
    }
    payload["tasks"]["addedInALaterMinor"] = serde_json::json!(1);
    payload["addedInALaterMinor"] = serde_json::json!("a value");
    assert!(
        published.is_valid(&payload),
        "the published schema refuses an added property"
    );
    assert!(
        !oracle.is_valid(&payload),
        "the suite's reading admits an undeclared property"
    );
}
