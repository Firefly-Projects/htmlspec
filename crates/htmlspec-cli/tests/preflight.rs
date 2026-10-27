//! The machine-requirement fixtures: two runbooks that both conform, only one of which is ready.
//!
//! The conformance suite in `conformance/` asks what the *validator* reports about a document, and its
//! answer for both of these fixtures is the same — a machine-check declaration is an ordinary note on
//! an ordinary requirement unit, so the document is unchanged by it. That is `runbook-authority`'s
//! requirement rather than a convenience: "Ordinary document validation SHALL continue to validate the
//! document's kind and schema without inspecting the host environment." The fixtures live under
//! `fixtures/conformance/` and the suite there asserts the document half; this file asserts the half
//! the validator deliberately cannot see.
//!
//! So the interesting difference is in preflight, and it is the reason there are two fixtures: **one
//! declares a condition its host satisfies and the other declares one it does not.** A single fixture
//! would pass whether or not the declaration were read at all.
//!
//! # The host is stated, not inherited
//!
//! Neither fixture's outcome depends on what the machine running the suite has installed, which
//! `runbook-portability` needs and which is why both declarations are `state` checks: a state is about
//! a directory, and a directory is something a test can arrange. A fixture declaring `cargo` would
//! pass on a contributor's machine and fail on a clean one, testing the contributor's installation
//! rather than the assessment.
//!
//! The arrangement is real rather than scripted — an actual project directory is created beside an
//! actual non-project — so the probes under test are the production ones.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The binary under test, built by cargo before the test runs.
fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// Where the fixtures live.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/conformance")
}

/// A temporary directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-preflight-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path.canonicalize().expect("the scratch path resolves"))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run_in(directory: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    support::held(arguments, &stdout);
    (
        output.status.code().expect("the process exited normally"),
        stdout,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The two fixtures, and which of them a given host satisfies.
const SATISFIABLE: &str = "valid-runbook-machine-requirements";
const UNSATISFIABLE: &str = "valid-runbook-machine-requirements-unsatisfiable";

/// A project holding one runbook built from a committed fixture.
///
/// The fixture's document is copied rather than authored here, so what is checked is the document the
/// suite ships — a fixture that drifted from the test would otherwise pass against a sample. The
/// release metadata beside it is written here, because a fixture directory under `fixtures/` is not a
/// runbook and carries no version of its own.
fn a_project_holding(fixture: &str, scratch: &Scratch, runbook_name: &str) -> PathBuf {
    let project = scratch.path().to_owned();
    let (code, out, err) = run_in(&project, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "init failed:\n{out}{err}");

    let runbook = project.join("htmlspec").join("runbooks").join(runbook_name);
    std::fs::create_dir_all(&runbook).expect("the runbook directory exists");
    let source = fixtures().join(fixture).join("input.html");
    let document = std::fs::read_to_string(&source)
        .unwrap_or_else(|error| panic!("{} is missing: {error}", source.display()));
    // The fixture references no asset store, and a document in a project must: the runbook's own
    // references are what `self-runbooks` checks in this repository, and a copy under a project has to
    // resolve the same way.
    std::fs::write(runbook.join("specification.html"), document).expect("written");

    // A procedure, which starting a run requires and preflight does not.
    let (code, out, err) = run_in(
        &project,
        &[
            "new",
            "document",
            "tasks",
            &runbook.join("tasks.html").to_string_lossy(),
            "--subject",
            "A procedure worth repeating",
        ],
    );
    assert_eq!(code, 0, "new document failed:\n{out}{err}");
    let procedure = std::fs::read_to_string(runbook.join("tasks.html")).expect("readable");
    std::fs::write(
        runbook.join("tasks.html"),
        procedure
            .replace("-example", "-real")
            .replace(
                "Replace this paragraph with what",
                "This document states what",
            )
            .replace("data-change=\"added\"", "data-change=\"unchanged\""),
    )
    .expect("rewritten");

    std::fs::write(runbook.join("VERSION"), "0.1.0\n").expect("written");
    std::fs::write(
        runbook.join("CHANGELOG.md"),
        "# Changelog\n\n## 0.1.0\n\nThe first release of this procedure.\n",
    )
    .expect("written");
    project
}

/// A directory that is not inside a project, and holds a repository boundary so the search stops.
fn outside_a_project(scratch: &Scratch) -> PathBuf {
    let directory = scratch.path().join("outside");
    std::fs::create_dir_all(directory.join(".git")).expect("a repository boundary");
    directory
}

/// Both fixtures are ordinary documents, so both validate — and neither validation says anything
/// about the machine.
///
/// `runbook-authority`'s scenario "Checking a runbook on a documentation machine" is the obligation
/// here: a document whose declared tool is not installed still validates, because a machine-dependent
/// rule in the per-document validator would make the answer depend on where validation ran.
#[test]
fn both_fixtures_conform_without_inspecting_the_host() {
    for fixture in [SATISFIABLE, UNSATISFIABLE] {
        let path = fixtures().join(fixture).join("input.html");
        let (code, out, err) = run_in(
            path.parent().expect("a directory"),
            &["validate", &path.to_string_lossy()],
        );
        assert_eq!(
            code, 0,
            "{fixture} does not validate, so the pair is not about readiness:\n{out}{err}"
        );
        // The one diagnostic either document carries is the review warning a specification with no
        // verification always carries, which never gates and says nothing about a machine.
        assert!(
            !out.contains("error:"),
            "{fixture} reports an error:\n{out}"
        );
    }
}

/// **The pair: one fixture is ready and the other is not, on the same machine.**
///
/// This is the assertion the two fixtures exist for. Both are checked from a directory outside any
/// project, so the difference between them is the declaration's and nothing else — one asks about the
/// project boundary, which that directory satisfies, and one asks about a worktree, which a directory
/// that is not a repository cannot satisfy.
#[test]
fn only_the_satisfied_fixture_passes_preflight() {
    let scratch = Scratch::new("pair");
    let satisfied = a_project_holding(SATISFIABLE, &scratch, "satisfied");
    let unsatisfied = a_project_holding(UNSATISFIABLE, &scratch, "unsatisfied");
    let outside = outside_a_project(&scratch);

    // Registered so both runbooks are reachable from the one working directory, which is what makes
    // this a comparison between two declarations rather than between two environments. The registry is
    // a scratch directory, for the reason `run_with_registry` gives: the real one lives under
    // `XDG_CONFIG_HOME`, so a test using the ambient environment would write into the config directory
    // of whoever ran the suite, and would fail outright in a build sandbox where `$HOME` is read-only.
    let config = scratch.path().join("config");
    std::fs::create_dir_all(&config).expect("the registry's home exists");
    for (id, project) in [("satisfied", &satisfied), ("unsatisfied", &unsatisfied)] {
        let (code, out, err) = run_with_registry(
            project,
            &config,
            &["store", "add", id, &project.to_string_lossy()],
        );
        assert_eq!(code, 0, "the store was not registered:\n{out}{err}");
    }

    let (code, out, err) = run_with_registry(
        &outside,
        &config,
        &["--store", "satisfied", "preflight", "satisfied"],
    );
    let ready_code = code;
    let ready_out = out.clone();
    assert_eq!(
        code, 0,
        "the satisfiable fixture was refused from a directory that satisfies it:\n{out}{err}"
    );
    assert!(
        out.starts_with("ready:"),
        "the report does not lead with readiness:\n{out}"
    );
    assert!(
        out.contains("req-outside-a-project"),
        "the report does not name the requirement it checked:\n{out}"
    );

    let (code, out, err) = run_with_registry(
        &outside,
        &config,
        &["--store", "unsatisfied", "preflight", "unsatisfied"],
    );
    assert_eq!(
        code, 1,
        "the unsatisfiable fixture was reported as ready:\n{out}{err}"
    );
    assert!(
        out.starts_with("not ready:"),
        "the refusal does not lead with readiness:\n{out}"
    );
    assert!(
        out.contains("req-a-clean-worktree"),
        "the refusal does not name the requirement that failed:\n{out}"
    );
    assert!(
        out.contains("unavailable") || out.contains("unmet"),
        "the refusal does not say how the check ended:\n{out}"
    );
    assert!(
        out.contains("remedy:"),
        "the refusal names no way forward:\n{out}"
    );

    // The pair, stated as the difference it exists to show: same working directory, same binary, two
    // declarations, two answers.
    assert_ne!(
        ready_code, code,
        "both fixtures reported the same readiness, so the declaration decided nothing.\n\
         satisfied: {ready_out}\nunsatisfied: {out}"
    );

    for id in ["satisfied", "unsatisfied"] {
        let _ = run_with_registry(&satisfied, &config, &["store", "remove", id]);
    }
}

/// The same fixture changes its answer when the host does, which is what makes the pair above about
/// the declaration rather than about the fixture.
#[test]
fn a_fixtures_readiness_follows_the_host_and_not_the_document() {
    let scratch = Scratch::new("host-decides");
    let project = a_project_holding(SATISFIABLE, &scratch, "release");

    // From outside a project: ready. The registry is a scratch directory, so this test neither reads
    // nor writes the config directory of whoever runs the suite.
    let config = scratch.path().join("config");
    std::fs::create_dir_all(&config).expect("the registry's home exists");
    let outside = outside_a_project(&scratch);
    let (code, out, err) = run_with_registry(
        &project,
        &config,
        &["store", "add", "fixture", &project.to_string_lossy()],
    );
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, err) = run_with_registry(
        &outside,
        &config,
        &["--store", "fixture", "preflight", "release"],
    );
    assert_eq!(code, 0, "ready from outside a project:\n{out}{err}");

    // From inside one: refused, and the report names where the project was found. This is the
    // bootstrap hazard this project measured on itself, arriving as a refusal before any step runs.
    // No `--store` here: the runbook is in the project this directory is inside, which is the case the
    // hazard is about.
    let (code, out, err) = run_with_registry(&project, &config, &["preflight", "release"]);
    assert_eq!(code, 1, "ready from inside a project:\n{out}{err}");
    assert!(
        out.contains(&project.display().to_string()) || out.contains("project at"),
        "the refusal does not say where the project was found:\n{out}"
    );
    let _ = run_with_registry(&project, &config, &["store", "remove", "fixture"]);
}

/// Preflight over a fixture writes nothing, which is what lets it be run before a decision is made.
#[test]
fn preflight_over_a_fixture_writes_nothing() {
    let scratch = Scratch::new("read-only");
    let project = a_project_holding(SATISFIABLE, &scratch, "release");
    let before = walk(&project);

    let (code, out, err) = run_in(&project, &["preflight", "release"]);
    assert_eq!(
        code, 1,
        "expected the bootstrap hazard to refuse:\n{out}{err}"
    );
    assert_eq!(
        before,
        walk(&project),
        "preflight changed something while reporting that the run could not start"
    );
}

/// Every file beneath `root` with its length, for a before-and-after comparison.
fn walk(root: &Path) -> Vec<(PathBuf, u64)> {
    let mut found = Vec::new();
    collect(root, &mut found);
    found.sort();
    found
}

fn collect(directory: &Path, into: &mut Vec<(PathBuf, u64)>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, into);
        } else {
            let length = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            into.push((path, length));
        }
    }
}

/// Every declaration example the README carries parses, and names a check this program provides.
///
/// **A documented example nothing parses is a defect the reader meets first.** The README is where a
/// contributor learns the format, and `toolchain-packaging` requires the command surface to be
/// documented — so the examples are read out of the shipped file rather than restated here, and a
/// format that moves without its documentation fails this test instead of failing somebody's
/// runbook.
#[test]
fn every_declared_example_in_the_readme_parses() {
    let readme = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    let source = std::fs::read_to_string(&readme)
        .unwrap_or_else(|error| panic!("{} is missing: {error}", readme.display()));

    // The marker and the object after it, which is the whole wire format. **Taken from inside markup
    // rather than from prose**: the README explains the format in sentences that also contain the
    // marker, and a sentence is not a declaration. What an author copies is the example in the
    // document, so that is what is checked.
    //
    // Each example keeps its own marker, `preflight/v1`, `preflight/v2`, or `preflight/v3`, and is
    // checked under it: a v2 example wrapped in a v1 note would be refused for its remedy, a v3
    // example wrapped in a v2 note would be refused for its `run` kind, and an older example wrapped
    // in a newer note would never test that the older format still reads.
    let examples: Vec<(&str, &str)> = source
        .split("data-field=\"note\">preflight/")
        .skip(1)
        .filter_map(|rest| {
            let end = rest.find("</p>")?;
            let (version, example) = rest[..end].split_once(' ')?;
            Some((version, example))
        })
        .collect();
    assert!(
        !examples.is_empty(),
        "the README documents no declaration, so there is nothing for a contributor to copy"
    );
    for version in ["v1", "v2", "v3"] {
        assert!(
            examples.iter().any(|(found, _)| *found == version),
            "the README documents no preflight/{version} declaration"
        );
    }

    for (version, example) in examples {
        // Read through a document rather than through a private parser, so the example is checked the
        // way an author's declaration is: as a note, projected, then parsed.
        assert!(
            example.starts_with('{') && example.ends_with('}'),
            "the documented example {example} is not one JSON object"
        );
        let document = format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"0.1.0\" name=\"htmlspec-version\">\
             <meta content=\"specification\" name=\"htmlspec-kind\"><title>E</title></head>\
             <body><main>\
             <section class=\"requirement\" id=\"req-documented\" data-change=\"unchanged\">\
             <h3>A documented check <span class=\"hs-id\">req-documented</span></h3>\
             <p>A run MUST satisfy it.</p>\
             <p data-field=\"note\">preflight/{} {}</p>\
             <article class=\"scenario\" id=\"scenario-documented\" data-change=\"unchanged\">\
             <h3>Its scenario</h3>\
             <dl><dt>When</dt><dd>A run starts.</dd><dt>Then</dt><dd>It is checked.</dd></dl>\
             </article></section></main></body></html>",
            htmlspec_assets::escape_text(version),
            htmlspec_assets::escape_text(example)
        );
        let parsed = htmlspec_parse::parse_document(&document);
        assert!(
            parsed.is_recovery_free(),
            "the documented example {example} needs error recovery"
        );
        // The projection is not enough: a note can project cleanly and still declare nothing. So the
        // example is driven through the binary over a project holding it, and the answer must be a
        // readiness report rather than an invalid-declaration refusal.
        let scratch = Scratch::new("readme-example");
        let project = scratch.path().to_owned();
        let (code, out, err) = run_in(&project, &["init", "--tools", "none"]);
        assert_eq!(code, 0, "init failed:\n{out}{err}");
        let runbook = project.join("htmlspec/runbooks/documented");
        std::fs::create_dir_all(&runbook).expect("the runbook directory exists");
        std::fs::write(runbook.join("specification.html"), &document).expect("written");
        let (code, out, err) = run_in(
            &project,
            &[
                "new",
                "document",
                "tasks",
                &runbook.join("tasks.html").to_string_lossy(),
                "--subject",
                "A documented procedure",
            ],
        );
        assert_eq!(code, 0, "new document failed:\n{out}{err}");
        std::fs::write(runbook.join("VERSION"), "0.1.0\n").expect("written");
        std::fs::write(
            runbook.join("CHANGELOG.md"),
            "# Changelog\n\n## 0.1.0\n\nFirst.\n",
        )
        .expect("written");

        let (code, out, err) = run_in(&project, &["preflight", "documented"]);
        assert_ne!(
            code, 2,
            "the documented example {example} is not a declaration this program can read:\n\
             {out}{err}"
        );
    }
}

/// A specification whose one requirement carries `note` as its declaration.
fn a_specification_declaring(note: &str) -> String {
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta content=\"0.1.0\" name=\"htmlspec-version\">\
         <meta content=\"specification\" name=\"htmlspec-kind\"><title>E</title></head>\
         <body><main>\
         <section class=\"requirement\" id=\"req-outside\" data-change=\"unchanged\">\
         <h3>The run starts outside a project <span class=\"hs-id\">req-outside</span></h3>\
         <p>A run MUST start outside a project.</p>\
         <p data-field=\"note\">{}</p>\
         <article class=\"scenario\" id=\"scenario-outside\" data-change=\"unchanged\">\
         <h3>Its scenario</h3>\
         <dl><dt>When</dt><dd>A run starts.</dd><dt>Then</dt><dd>It is checked.</dd></dl>\
         </article></section></main></body></html>",
        htmlspec_assets::escape_text(note)
    )
}

/// A runbook's remedy reaches the operator through the binary: after the program's own remedy in
/// the human report, as `runbookRemedy` in the payload, and nowhere under `preflight/v1`.
///
/// Run from inside the project, so the declared `outside-project` state is unmet.
#[test]
fn a_runbooks_remedy_reaches_the_report_and_v1_refuses_one() {
    let scratch = Scratch::new("remedy");
    let project = a_project_holding(SATISFIABLE, &scratch, "remedied");
    let specification = project.join("htmlspec/runbooks/remedied/specification.html");
    std::fs::write(
        &specification,
        a_specification_declaring(
            "preflight/v2 {\"kind\":\"state\",\"check\":\"outside-project\",\
             \"remedy\":\"cd to the parent of the project first\"}",
        ),
    )
    .expect("written");

    let (code, out, err) = run_in(&project, &["preflight", "remedied"]);
    assert_eq!(code, 1, "an unmet check did not exit 1:\n{out}{err}");
    let report = format!("{out}{err}");
    let program = report
        .find("remedy: run this from a directory that is not inside a project")
        .unwrap_or_else(|| panic!("the program's remedy is missing:\n{report}"));
    let runbook = report
        .find("the runbook says: cd to the parent of the project first")
        .unwrap_or_else(|| panic!("the runbook's remedy is missing:\n{report}"));
    assert!(
        program < runbook,
        "the runbook's words came first:\n{report}"
    );

    let (code, out, err) = run_in(&project, &["--json", "preflight", "remedied"]);
    assert_eq!(code, 1, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("one JSON payload");
    assert_eq!(
        payload["checks"][0]["runbookRemedy"],
        serde_json::json!("cd to the parent of the project first"),
        "{payload:#}"
    );

    // A run the machine cannot start writes the same report, remedy included, and no record.
    let (code, out, err) = run_in(&project, &["--json", "run", "remedied", "--as", "first"]);
    assert_eq!(code, 1, "{out}{err}");
    let refusal: serde_json::Value = serde_json::from_str(&out).expect("one JSON payload");
    assert_eq!(
        refusal["checks"][0]["runbookRemedy"],
        serde_json::json!("cd to the parent of the project first"),
        "{refusal:#}"
    );
    assert!(
        !project
            .join("htmlspec/runbooks/remedied/runs/first.html")
            .exists(),
        "a refused run wrote a record"
    );

    // The same declaration under v1 is a contract this program cannot read: exit 2, naming the
    // field, rather than an unmet check.
    std::fs::write(
        &specification,
        a_specification_declaring(
            "preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\",\
             \"remedy\":\"cd to the parent of the project first\"}",
        ),
    )
    .expect("written");
    let (code, out, err) = run_in(&project, &["preflight", "remedied"]);
    assert_eq!(code, 2, "v1 read a remedy:\n{out}{err}");
    assert!(
        err.contains("`remedy` is not a field this form has"),
        "the refusal does not name the field:\n{err}"
    );
}

/// A declared manifest path holding an escape sequence reaches the report by code point, in both
/// the message for an absent manifest and the one for a manifest with no `flake.lock` beside it.
///
/// The archived scenario "A declared name holding an escape sequence": the probe uses the path as
/// written, and the operator's terminal receives no escape byte from it.
#[cfg(unix)]
#[test]
fn a_manifest_path_holding_an_escape_sequence_reaches_the_report_by_code_point() {
    let scratch = Scratch::new("manifest-escape");
    let project = a_project_holding(SATISFIABLE, &scratch, "escaped");
    let specification = project.join("htmlspec/runbooks/escaped/specification.html");
    std::fs::write(
        &specification,
        a_specification_declaring(
            "preflight/v1 {\"kind\":\"toolchain\",\"manifest\":\"a\\u001b[2Jb/flake.nix\",\
             \"shell\":\"default\",\"tools\":[\"cargo\"]}",
        ),
    )
    .expect("written");

    let report = |label: &str| {
        let (code, out, err) = run_in(&project, &["preflight", "escaped"]);
        assert_eq!(
            code, 1,
            "{label}: an unavailable manifest did not exit 1:\n{out}{err}"
        );
        let report = format!("{out}{err}");
        assert!(
            !report.contains('\u{1b}'),
            "{label}: an escape byte reached the report:\n{report:?}"
        );
        assert!(
            report.contains("a<U+001B>[2Jb/flake.nix"),
            "{label}: the path is not named by code point:\n{report}"
        );
        report
    };

    let absent = report("absent");
    assert!(absent.contains("no manifest at"), "{absent}");

    // The manifest present and unlocked: the second message, which names the same path.
    let directory = project.join("a\u{1b}[2Jb");
    std::fs::create_dir_all(&directory).expect("the manifest's directory");
    std::fs::write(directory.join("flake.nix"), "{ outputs = _: { }; }\n").expect("the manifest");
    let unlocked = report("unlocked");
    assert!(
        unlocked.contains("has no `flake.lock` beside it"),
        "{unlocked}"
    );
}

/// Every command the README's portability section names is one the binary provides.
///
/// The same guarantee `invocations.rs` holds for the installed instruction bodies, applied to the
/// document a contributor reads before they install anything.
#[test]
fn every_command_the_portability_documentation_names_exists() {
    let readme = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    let source = std::fs::read_to_string(&readme).expect("the README is readable");
    let section = &source[source
        .find("### Carrying a runbook to another machine")
        .expect("the section the change added")..];

    for name in ["preflight", "store", "run"] {
        assert!(
            section.contains(&format!("htmlspec {name}"))
                || section.contains(&format!("htmlspec --store portable {name}")),
            "the portability documentation does not name `{name}`, which the operation needs"
        );
    }
    // And nothing it names is absent. Checked against the binary's own help rather than a list here,
    // so a withdrawn command fails this test instead of a contributor's first run.
    let (code, out, _) = run_in(Path::new("."), &["--help"]);
    assert_eq!(code, 0, "the binary has no help");
    for name in [
        "preflight",
        "store",
        "run",
        "archive",
        "validate",
        "init",
        "new",
        "list",
    ] {
        assert!(out.contains(name), "`--help` does not list {name}:\n{out}");
    }
}

// -----------------------------------------------------------------------------
// Carrying a runbook to another machine.
//
// The scenario `runbook-portability` states: a copied runbook and its shared assets in a minimal
// receiving layout, with the source project *unavailable*, and a run started from a separate working
// directory. "Unavailable" is arranged rather than simulated — the source is removed — because a test
// that left the source in place would pass by reading it, and reading it is the thing under test.
// -----------------------------------------------------------------------------

/// A minimal receiving layout, copied from a source project that is then made unavailable.
///
/// Returns the receiving project's root and the working directory the run is started from. The copy is
/// exactly the shape the documentation names: one shared asset store, the runbook's two documents and
/// its release metadata, and no archive, no declaration, no accepted specifications, and no records.
fn a_receiving_layout(scratch: &Scratch, fixture: &str) -> (PathBuf, PathBuf) {
    let source = scratch.path().join("source");
    std::fs::create_dir_all(&source).expect("the source directory exists");
    let project = a_project_holding_at(fixture, &source, "release");

    let portable = scratch.path().join("portable");
    let receiving = portable.join("htmlspec");
    std::fs::create_dir_all(&receiving).expect("the receiving project exists");

    // One asset store, shared by the runbook and by any record started here. Copied rather than
    // regenerated, because a store per runbook is the topology the asset check exists to refuse.
    copy_tree(&project.join("htmlspec/assets"), &receiving.join("assets"));
    copy_tree(
        &project.join("htmlspec/runbooks/release"),
        &receiving.join("runbooks/release"),
    );

    // **The source is made unavailable, not merely unused.** A test that left it on disk would pass by
    // reading it, which is the one thing the requirement forbids: preflight and run creation must work
    // "without reading any source-project file".
    std::fs::remove_dir_all(&project).expect("the source project is removed");
    assert!(
        !project.exists(),
        "the source project is still readable, so this test proves nothing about carrying"
    );

    // The work happens somewhere else entirely, which is the point of registering a store.
    let target = scratch.path().join("target");
    std::fs::create_dir_all(target.join(".git")).expect("a repository boundary, not a project");
    (portable, target)
}

/// A project holding one runbook built from a committed fixture, at `root`.
fn a_project_holding_at(fixture: &str, root: &Path, runbook_name: &str) -> PathBuf {
    let (code, out, err) = run_in(root, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "init failed:\n{out}{err}");

    let runbook = root.join("htmlspec").join("runbooks").join(runbook_name);
    std::fs::create_dir_all(&runbook).expect("the runbook directory exists");
    let source = fixtures().join(fixture).join("input.html");
    let document = std::fs::read_to_string(&source)
        .unwrap_or_else(|error| panic!("{} is missing: {error}", source.display()));
    // Referenced against the one store the receiving layout will carry, so the copy renders offline.
    let assets = root.join("htmlspec").join("assets");
    std::fs::create_dir_all(&assets).expect("the store exists");
    std::fs::write(runbook.join("specification.html"), &document).expect("written");

    let (code, out, err) = run_in(
        root,
        &[
            "new",
            "document",
            "tasks",
            &runbook.join("tasks.html").to_string_lossy(),
            "--subject",
            "A procedure worth repeating",
        ],
    );
    assert_eq!(code, 0, "new document failed:\n{out}{err}");
    let procedure = std::fs::read_to_string(runbook.join("tasks.html")).expect("readable");
    std::fs::write(
        runbook.join("tasks.html"),
        procedure
            .replace("-example", "-real")
            .replace(
                "Replace this paragraph with what",
                "This document states what",
            )
            .replace("data-change=\"added\"", "data-change=\"unchanged\""),
    )
    .expect("rewritten");

    std::fs::write(runbook.join("VERSION"), "0.1.0\n").expect("written");
    std::fs::write(
        runbook.join("CHANGELOG.md"),
        "# Changelog\n\n## 0.1.0\n\nThe first release of this procedure.\n",
    )
    .expect("written");
    root.to_owned()
}

/// Copies a tree, which is what an operator does with `cp -r` and what this test does with `std`.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the destination directory exists");
    let entries = std::fs::read_dir(from).expect("the source directory is readable");
    for entry in entries.flatten() {
        let path = entry.path();
        let destination = to.join(entry.file_name());
        if path.is_dir() {
            copy_tree(&path, &destination);
        } else {
            std::fs::copy(&path, &destination).expect("the file copies");
        }
    }
}

/// A carried runbook is checked and run with its source project gone.
///
/// **The load-bearing test in this file.** Everything else asserts a part of preflight; this one
/// asserts the thing the change exists for — that a runbook which arrived by copying, in a layout
/// holding no archive and no declaration, can establish its prerequisites and start a run.
#[test]
fn a_carried_runbook_starts_a_run_with_its_source_project_absent() {
    let scratch = Scratch::new("carried");
    let (portable, target) = a_receiving_layout(&scratch, SATISFIABLE);

    // What the receiving layout does *not* hold, asserted rather than assumed: a reader who thinks
    // this test passed because something travelled with the copy should be able to see that nothing
    // did.
    for absent in [
        "htmlspec/changes",
        "htmlspec/specs",
        "htmlspec/runbooks/release/runs",
        "spec.nix",
    ] {
        assert!(
            !portable.join(absent).exists(),
            "the receiving layout holds {absent}, so this test is not about a minimal copy"
        );
    }
    assert!(
        portable.join("htmlspec/assets").is_dir(),
        "the receiving layout has no asset store, so a record started here could not render"
    );

    let config = scratch.path().join("config");
    std::fs::create_dir_all(&config).expect("the registry's home exists");
    let (code, out, err) = run_with_registry(
        &target,
        &config,
        &["store", "add", "portable", &portable.to_string_lossy()],
    );
    assert_eq!(
        code, 0,
        "the receiving project was not registered:\n{out}{err}"
    );

    // Preflight from the target directory, against the store's documents.
    let (code, out, err) = run_with_registry(
        &target,
        &config,
        &["--store", "portable", "preflight", "release"],
    );
    assert_eq!(
        code, 0,
        "a carried runbook was refused with its source absent:\n{out}{err}"
    );
    assert!(
        out.contains("0.1.0"),
        "the report does not name the carried release, which is the only way to identify it \
         without the archive:\n{out}"
    );
    assert!(
        out.contains(&target.display().to_string()),
        "the report does not name the working directory it checked:\n{out}"
    );

    // And a run, started from the target and recorded at the destination.
    let (code, out, err) = run_with_registry(
        &target,
        &config,
        &[
            "--store",
            "portable",
            "run",
            "release",
            "--as",
            "first-here",
        ],
    );
    assert_eq!(
        code, 0,
        "a carried runbook could not start a run with its source absent:\n{out}{err}"
    );
    let record = portable.join("htmlspec/runbooks/release/runs/first-here.html");
    assert!(
        record.is_file(),
        "the run left no record at {}",
        record.display()
    );

    // The record is a document of the receiving project: valid, canonical, and rendering against the
    // one store that travelled.
    let (code, out, err) = run_with_registry(
        &portable,
        &config,
        &["validate", "--strict", &record.to_string_lossy()],
    );
    assert_eq!(
        code, 0,
        "the record started on the receiving machine does not validate:\n{out}{err}"
    );
    let written = std::fs::read_to_string(&record).expect("the record reads");
    assert!(
        written.contains("0.1.0 release"),
        "the record cannot say which release of the procedure ran:\n{written}"
    );
    assert!(
        written.contains("data-outcome=\"passed\""),
        "the machine check that passed was not recorded:\n{written}"
    );
    // One asset store, referenced relatively, so the record opens with no network.
    assert!(
        written.contains("href=\"../../../assets/"),
        "the record does not reference the receiving project's one asset store:\n{written}"
    );
    assert!(
        !written.contains("http://") && !written.contains("https://"),
        "the record reaches the network:\n{written}"
    );

    // No source history was fabricated: the destination holds this run and no other.
    let records = std::fs::read_dir(record.parent().expect("the records directory"))
        .expect("readable")
        .flatten()
        .count();
    assert_eq!(
        records, 1,
        "the receiving layout holds {records} records, and only one run was started here"
    );
}

/// A carried runbook whose prerequisites are unmet is refused on the receiving machine too.
///
/// The other half of carrying: the contract travels, and so does the refusal. A copy that could be run
/// anywhere regardless of the machine would be a bundle rather than a contract.
#[test]
fn a_carried_runbook_is_refused_where_its_prerequisites_are_unmet() {
    let scratch = Scratch::new("carried-refused");
    // The satisfiable fixture declares `outside-project`, so the receiving layout is arranged to
    // violate it: the run is started from inside a project.
    let (portable, _) = a_receiving_layout(&scratch, SATISFIABLE);
    let inside = portable.join("htmlspec/changes");
    std::fs::create_dir_all(&inside).expect("a directory inside the receiving project");

    let config = scratch.path().join("config");
    std::fs::create_dir_all(&config).expect("the registry's home exists");
    let (code, out, err) = run_with_registry(
        &inside,
        &config,
        &["store", "add", "portable", &portable.to_string_lossy()],
    );
    assert_eq!(code, 0, "{out}{err}");

    let (code, out, err) = run_with_registry(
        &inside,
        &config,
        &["--store", "portable", "run", "release", "--as", "refused"],
    );
    assert_eq!(
        code, 1,
        "a carried runbook started on a machine that does not satisfy it:\n{out}{err}"
    );
    assert!(
        !portable
            .join("htmlspec/runbooks/release/runs/refused.html")
            .exists(),
        "a refused start created a record on the receiving machine"
    );
    assert!(
        !portable.join("htmlspec/runbooks/release/runs").exists(),
        "a refused start created a records directory the copy did not arrive with"
    );
}

/// A carried runbook with no version is refused, and the refusal is the migration.
///
/// The case an operator actually meets: a runbook copied from a project that predates the obligation.
/// Refusing is right, and the report has to say what to write.
#[test]
fn a_carried_runbook_with_no_version_names_the_migration() {
    let scratch = Scratch::new("carried-unversioned");
    let (portable, target) = a_receiving_layout(&scratch, SATISFIABLE);
    std::fs::remove_file(portable.join("htmlspec/runbooks/release/VERSION")).expect("removed");

    let config = scratch.path().join("config");
    std::fs::create_dir_all(&config).expect("the registry's home exists");
    let (code, out, err) = run_with_registry(
        &target,
        &config,
        &["store", "add", "portable", &portable.to_string_lossy()],
    );
    assert_eq!(code, 0, "{out}{err}");

    for arguments in [
        vec!["--store", "portable", "preflight", "release"],
        vec!["--store", "portable", "run", "release", "--as", "first"],
    ] {
        let (code, out, err) = run_with_registry(&target, &config, &arguments);
        assert_eq!(code, 2, "{arguments:?} did not refuse:\n{out}{err}");
        assert!(
            err.contains("VERSION") && err.contains("CHANGELOG.md"),
            "{arguments:?} does not say what to write:\n{err}"
        );
    }
    assert!(
        !portable.join("htmlspec/runbooks/release/runs").exists(),
        "a refused start created a records directory"
    );
}

/// Runs the binary with the store registry pointed at a scratch directory.
///
/// **Every store test must use this**, for the reason `tests/archive.rs` gives: the registry lives
/// under `XDG_CONFIG_HOME`, so a test that ran with the ambient environment would register entries in
/// the config directory of whoever ran the suite and read whatever they had already registered.
fn run_with_registry(directory: &Path, config: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .env("XDG_CONFIG_HOME", config)
        .env("HOME", config)
        .args(arguments)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    support::held(arguments, &stdout);
    (
        output.status.code().expect("the process exited normally"),
        stdout,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}
