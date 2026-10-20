//! Archival's checking half, and the parser it must never grow.
//!
//! The archival operation is split deliberately: an agent produces a change's
//! specification from the completed change, and the program checks what the agent
//! wrote. This file drives the checking half through the built binary, and then
//! asserts the negative — that nothing here reads a proposal or a design and emits
//! requirements.
//!
//! The negative is the load-bearing test. A specification produced by an extractor
//! would satisfy every check while misstating what the change established, and it
//! would look authoritative doing it. That failure is invisible to a validator,
//! because the output is well-formed; the only thing that catches it is a test
//! asserting the extractor does not exist.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The binary under test, built by cargo before the test runs.
fn binary() -> PathBuf {
    // `CARGO_BIN_EXE_<name>` is set for an integration test in the same package.
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// A temporary directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-archive-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs the binary from inside `directory`.
///
/// `new` and `archive` resolve their project by walking upward from the working
/// directory, so a test that drives them has to say where it is standing. Setting the
/// child's directory rather than the test process's keeps these tests safe to run in
/// parallel.
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

/// Runs the binary from inside `directory`, with the store registry pointed at `config`.
///
/// **Every store test must use this.** The registry lives under `XDG_CONFIG_HOME`, so a test
/// that ran the binary with the ambient environment would register entries in the config
/// directory of whoever ran the suite, and would read whatever they had already registered.
/// A test that writes to a developer's home directory is a test that fails differently on
/// two machines.
fn run_with_registry(directory: &Path, config: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .env("XDG_CONFIG_HOME", config)
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

/// Runs `archive` against a change, from inside its project.
///
/// The version gate resolves the project from the working directory, and it requires the
/// declared version to be ordered after the last archived one. A test whose subject is the
/// specification advances the version first, so the gate is satisfied and the specification
/// is what the assertion is actually about. `the_version_gate_*` tests cover the gate
/// itself.
fn archive_in(change: &Path) -> (i32, String, String) {
    let project = change
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("a change sits three levels below its project");
    advance_version(project);
    run_in(project, &["archive", &change.to_string_lossy(), "--check"])
}

/// Runs `archive` for real, answering its offer to proceed in advance.
///
/// The fixtures carry an example task and a requirement no accepted specification holds, so
/// archival has findings to report and would ask. `--yes` is how the operation is driven where
/// nothing can answer, which is exactly a test process.
fn archive_performing_in(change: &Path) -> (i32, String, String) {
    let project = change
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("a change sits three levels below its project");
    advance_version(project);
    run_in(project, &["archive", &change.to_string_lossy(), "--yes"])
}

/// Raises the declared version past the initial one.
fn advance_version(project: &Path) {
    let path = project.join("spec.nix");
    let declaration = std::fs::read_to_string(&path).expect("the declaration is readable");
    std::fs::write(&path, declaration.replace("0.0.0", "0.1.0")).expect("written");
}

/// A project, bootstrapped in the scratch directory.
fn a_project(scratch: &Scratch) -> PathBuf {
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "init failed:\n{out}{err}");
    scratch.0.clone()
}

/// A change directory with its proposal, design, and tasks documents.
fn a_change(scratch: &Scratch, name: &str) -> PathBuf {
    let project = a_project(scratch);
    let (code, out, err) = run_in(
        &project,
        &["new", "change", name, "--subject", "A change under test"],
    );
    assert_eq!(code, 0, "new change failed:\n{out}{err}");
    project.join("htmlspec").join("changes").join(name)
}

/// Writes a specification into `directory`, as an agent would at archival.
fn write_specification(directory: &Path, body: &str) -> PathBuf {
    let path = directory.join("specification.html");
    let source = format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n\
         \x20   <meta charset=\"utf-8\">\n\
         \x20   <meta content=\"0.1.0\" name=\"htmlspec-version\">\n\
         \x20   <meta content=\"specification\" name=\"htmlspec-kind\">\n\
         \x20   <title>A change under test: the contract</title>\n\
         \x20 </head>\n  <body>\n    <main>\n{body}\n    </main>\n  </body>\n</html>\n"
    );
    std::fs::write(&path, source).expect("the specification is written");
    path
}

/// Writes a specification whose masthead carries `summary`, with `body` as its content.
///
/// [`write_specification`] writes no `header` at all, so every fixture using it projects an empty
/// summary — which is what keeps the tests about identifiers from also being tests about the
/// masthead. Widening that helper would silently add a second subject to a dozen existing
/// assertions, so the summary tests build their own.
fn write_specification_with_summary(directory: &Path, summary: &str, body: &str) -> PathBuf {
    let path = directory.join("specification.html");
    let source = format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n\
         \x20   <meta charset=\"utf-8\">\n\
         \x20   <meta content=\"0.1.0\" name=\"htmlspec-version\">\n\
         \x20   <meta content=\"specification\" name=\"htmlspec-kind\">\n\
         \x20   <title>A change under test: the contract</title>\n\
         \x20 </head>\n  <body>\n\
         \x20   <header class=\"hs-masthead\">\n\
         \x20     <h1>A change under test: the contract</h1>\n\
         \x20     <p>{summary}</p>\n\
         \x20   </header>\n    <main>\n{body}\n    </main>\n  </body>\n</html>\n"
    );
    std::fs::write(&path, source).expect("the specification is written");
    path
}

/// The `specification` template's own placeholder summary, read from the template.
///
/// Read rather than written here, so this test cannot pass against a derivation that has stopped
/// finding anything: the fixture and the expectation would go empty together, and an empty summary
/// is not refused at all — so the assertion that archival *does* refuse still fails.
fn the_specification_placeholder() -> String {
    let rendered = htmlspec_assets::render_kind("specification", "A change under test", "assets")
        .expect("the specification template renders");
    let outcome = htmlspec_parse::parse_document(&rendered);
    let projection = htmlspec_parse::project(&outcome.document);
    let summary = projection.model.summary.join(" ");
    assert!(
        htmlspec_assets::is_example_placeholder(&summary),
        "a freshly rendered specification's summary is not its template's placeholder, so this \
         test's subject does not exist: {summary:?}"
    );
    summary
}

/// A requirement and the verification that names it, so nothing is reported.
const VERIFIED_REQUIREMENT: &str = "      <section class=\"requirement\" id=\"req-established\">\n\
     \x20       <h2>What the change established</h2>\n\
     \x20       <p>The system SHALL do the thing the change set out to do.</p>\n\
     \x20       <article class=\"scenario\" id=\"scenario-established\">\n\
     \x20         <h3>It holds</h3>\n\
     \x20         <dl><dt>When</dt><dd>It runs.</dd><dt>Then</dt><dd>It works.</dd></dl>\n\
     \x20       </article>\n\
     \x20     </section>\n\
     \x20     <section class=\"verification\" id=\"verify-established\" \
     data-outcome=\"passed\" data-refs=\"req-established\">\n\
     \x20       <h2>Checked</h2>\n\
     \x20       <p data-field=\"method\">Ran the suite.</p>\n\
     \x20     </section>";

/// [`VERIFIED_REQUIREMENT`], renamed: stated under a new heading with a `renamed-from` paragraph
/// naming the one it replaced, and `data-change="modified"`, as a change that renames and rewrites
/// it states it.
fn renamed_requirement() -> String {
    let renamed = VERIFIED_REQUIREMENT
        .replace(
            "<section class=\"requirement\" id=\"req-established\">",
            "<section class=\"requirement\" id=\"req-established\" data-change=\"modified\">",
        )
        .replace(
            "<h2>What the change established</h2>\n",
            "<h2>What the change now establishes</h2>\n\
             \x20       <p data-field=\"renamed-from\">What the change established</p>\n",
        );
    assert_ne!(renamed, VERIFIED_REQUIREMENT, "the rename edit is stale");
    renamed
}

/// Marks every task in a change's tasks document done or dropped, so the one finding a test
/// makes is the one it is about.
fn finish_every_task(change: &Path) {
    let path = change.join("tasks.html");
    let tasks = std::fs::read_to_string(&path).expect("the tasks document reads");
    let finished = tasks.replace("data-state=\"todo\"", "data-state=\"done\"");
    assert_ne!(
        finished, tasks,
        "the tasks document had no open task to finish"
    );
    std::fs::write(&path, finished).expect("the tasks document is written");
}

/// The toolchain-packaging scenario "A change whose only unapplied work is a rename".
///
/// Every identifier the change states is held, so before renames were compared this change was
/// reported as synced: `sync` said "applied in full" and archival said nothing. Task 1.1 recorded
/// that as the defect. Both operations are asserted, human and structured, and then the rename is
/// applied to show the finding clears for the reason it was raised.
#[test]
fn a_change_whose_only_unapplied_work_is_a_rename_has_not_been_synced() {
    let scratch = Scratch::new("rename-only");
    let directory = a_change(&scratch, "a-change");
    finish_every_task(&directory);
    let specs = scratch.0.join("htmlspec").join("specs");
    write_specification(&specs, VERIFIED_REQUIREMENT);
    write_specification(&directory, &renamed_requirement());

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(
        code, 0,
        "an untaken rename is a finding, not a failure:\n{out}{err}"
    );
    assert!(
        out.contains("0 of 1"),
        "the rename was counted as reached:\n{out}"
    );
    assert!(
        !out.contains("applied in full"),
        "the change was reported as applied:\n{out}"
    );
    for expected in [
        "renamed  req-established",
        "\"What the change established\"",
        "\"What the change now establishes\"",
    ] {
        assert!(
            out.contains(expected),
            "sync does not name {expected}:\n{out}"
        );
    }

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change", "--json"]);
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(payload["reached"], serde_json::json!([]), "{payload}");
    assert_eq!(payload["absent"], serde_json::json!([]), "{payload}");
    assert_eq!(
        payload["untakenRenames"],
        serde_json::json!([{
            "id": "req-established",
            "heading": "What the change now establishes",
            "acceptedHeading": "What the change established",
        }]),
        "{payload}"
    );

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("this change has not been synced"),
        "archival called a change with an untaken rename synced:\n{out}"
    );
    for expected in [
        "req-established",
        "\"What the change established\"",
        "\"What the change now establishes\"",
        "htmlspec sync --change a-change",
    ] {
        assert!(
            err.contains(expected),
            "archival does not name {expected}:\n{err}"
        );
    }

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check", "--json"],
    );
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(payload["needsSync"], true, "{payload}");
    assert_eq!(
        payload["unmergedRequirements"],
        serde_json::json!([]),
        "{payload}"
    );
    assert_eq!(
        payload["untakenRenames"][0]["id"], "req-established",
        "{payload}"
    );

    // The rename applied as the sync instructions say: the new heading, the same identifier, and
    // no renamed-from paragraph in the accepted specification.
    write_specification(
        &specs,
        &VERIFIED_REQUIREMENT.replace(
            "<h2>What the change established</h2>",
            "<h2>What the change now establishes</h2>",
        ),
    );
    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("applied in full"), "{out}");
    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !out.contains("has not been synced") && !err.contains("has not been synced"),
        "a taken rename was still reported:\n{out}{err}"
    );
}

/// [`VERIFIED_REQUIREMENT`] as a change modifying it states it, with one word added to its
/// statement.
fn reworded_requirement() -> String {
    let reworded = VERIFIED_REQUIREMENT
        .replace(
            "<section class=\"requirement\" id=\"req-established\">",
            "<section class=\"requirement\" id=\"req-established\" data-change=\"modified\">",
        )
        .replace("set out to do.</p>", "set out to do, quickly.</p>");
    assert_ne!(reworded, VERIFIED_REQUIREMENT, "the reword edit is stale");
    reworded
}

/// A requirement a change withdraws, with its one scenario, in `state` (`""` for none).
fn withdrawn_requirement(state: &str) -> String {
    let marked = if state.is_empty() {
        String::new()
    } else {
        format!(" data-change=\"{state}\"")
    };
    format!(
        "      <section class=\"requirement\" id=\"req-withdrawn\"{marked}>\n\
         \x20       <h2>What the project no longer requires</h2>\n\
         \x20       <p>The system SHALL do the thing nobody needs now.</p>\n\
         \x20       <article class=\"scenario\" id=\"scenario-withdrawn\"{marked}>\n\
         \x20         <h3>It held</h3>\n\
         \x20         <dl><dt>When</dt><dd>It ran.</dd><dt>Then</dt><dd>It worked.</dd></dl>\n\
         \x20       </article>\n\
         \x20     </section>"
    )
}

/// The toolchain-packaging scenario "A change whose only unapplied work is a word".
///
/// Every identifier the change states is held, so before content was compared this change was
/// reported as synced: `sync` said "applied in full" and archival said "ready". Both operations
/// are asserted, human and structured, and then the word is merged to show the finding clears for
/// the reason it was raised.
#[test]
fn a_change_whose_only_unapplied_work_is_a_word_has_not_been_synced() {
    let scratch = Scratch::new("word-only");
    let directory = a_change(&scratch, "a-change");
    finish_every_task(&directory);
    let specs = scratch.0.join("htmlspec").join("specs");
    write_specification(&specs, VERIFIED_REQUIREMENT);
    write_specification(&directory, &reworded_requirement());

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(
        code, 0,
        "untaken content is a finding, not a failure:\n{out}{err}"
    );
    assert!(
        out.contains("0 of 1"),
        "the word was counted as reached:\n{out}"
    );
    assert!(
        !out.contains("applied in full"),
        "the change was reported as applied:\n{out}"
    );
    assert!(
        out.contains("differs  req-established (statements from paragraph 1)"),
        "sync does not say where the content differs:\n{out}"
    );
    assert!(
        !out.contains("quickly"),
        "sync carries the change's text, which reads as merge text:\n{out}"
    );

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change", "--json"]);
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(payload["reached"], serde_json::json!([]), "{payload}");
    assert_eq!(payload["absent"], serde_json::json!([]), "{payload}");
    assert_eq!(
        payload["untakenContent"],
        serde_json::json!([{
            "id": "req-established",
            "firstDifferingStatement": 1,
            "scenariosNotHeld": [],
            "scenariosNotKept": [],
        }]),
        "{payload}"
    );
    assert_eq!(
        payload["untakenRemovals"],
        serde_json::json!([]),
        "{payload}"
    );

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("this change has not been synced"),
        "archival called a change with untaken content synced:\n{out}"
    );
    for expected in ["req-established", "htmlspec sync --change a-change"] {
        assert!(
            err.contains(expected),
            "archival does not name {expected}:\n{err}"
        );
    }

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check", "--json"],
    );
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(payload["needsSync"], true, "{payload}");
    assert_eq!(
        payload["unmergedRequirements"],
        serde_json::json!([]),
        "{payload}"
    );
    assert_eq!(
        payload["untakenContent"][0]["id"], "req-established",
        "{payload}"
    );

    // The word merged as the change states it.
    write_specification(
        &specs,
        &VERIFIED_REQUIREMENT.replace("set out to do.</p>", "set out to do, quickly.</p>"),
    );
    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("applied in full"), "{out}");
    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !out.contains("has not been synced") && !err.contains("has not been synced"),
        "merged content was still reported:\n{out}{err}"
    );
}

/// The toolchain-packaging scenario "A scenario changed inside a requirement marked unchanged".
///
/// Found by the code review of PR #42: the requirement carries no `data-change`, so it reads as
/// carried forward, and a scenario it adds was never compared. `sync` said "applied in full" and
/// archival said "ready" with the scenario unmerged.
#[test]
fn a_scenario_added_inside_an_unchanged_requirement_has_not_been_synced() {
    let scratch = Scratch::new("scenario-in-unchanged");
    let directory = a_change(&scratch, "a-change");
    finish_every_task(&directory);
    let specs = scratch.0.join("htmlspec").join("specs");
    write_specification(&specs, VERIFIED_REQUIREMENT);
    let with_a_scenario = VERIFIED_REQUIREMENT.replace(
        "      </article>\n",
        "      </article>\n\
         \x20       <article class=\"scenario\" id=\"scenario-added\" data-change=\"added\">\n\
         \x20         <h3>It holds again</h3>\n\
         \x20         <dl><dt>When</dt><dd>It runs twice.</dd><dt>Then</dt><dd>It still works.</dd></dl>\n\
         \x20       </article>\n",
    );
    assert_ne!(
        with_a_scenario, VERIFIED_REQUIREMENT,
        "the scenario edit is stale"
    );
    write_specification(&directory, &with_a_scenario);

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !out.contains("applied in full"),
        "an unmerged scenario inside an unchanged requirement was called applied:\n{out}"
    );
    assert!(
        out.contains("differs  req-established (not held: scenario-added)"),
        "sync does not name the scenario the accepted requirement lacks:\n{out}"
    );

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("this change has not been synced"),
        "archival called the change synced:\n{out}"
    );
}

/// The toolchain-packaging scenarios "A removal the accepted specifications have taken" and "A
/// withdrawal the merge applied".
///
/// Before withdrawals were judged by their change state, a correct merge of one was reported as a
/// requirement in no accepted specification, and archival said the change had not been synced.
#[test]
fn a_withdrawal_the_merge_applied_has_reached_the_contract() {
    let scratch = Scratch::new("withdrawal-applied");
    let directory = a_change(&scratch, "a-change");
    finish_every_task(&directory);
    let specs = scratch.0.join("htmlspec").join("specs");
    write_specification(&specs, VERIFIED_REQUIREMENT);
    write_specification(&directory, &withdrawn_requirement("removed"));

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change", "--json"]);
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(
        payload["reached"],
        serde_json::json!(["req-withdrawn"]),
        "{payload}"
    );
    assert_eq!(payload["absent"], serde_json::json!([]), "{payload}");
    assert_eq!(
        payload["untakenRemovals"],
        serde_json::json!([]),
        "{payload}"
    );

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("applied in full"), "{out}");

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !out.contains("has not been synced") && !err.contains("has not been synced"),
        "archival reported an applied withdrawal as unsynced:\n{out}{err}"
    );
    assert!(
        !err.contains("req-withdrawn"),
        "archival reported an applied withdrawal at all:\n{err}"
    );
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check", "--json"],
    );
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(payload["needsSync"], false, "{payload}");
    assert_eq!(
        payload["unmergedRequirements"],
        serde_json::json!([]),
        "{payload}"
    );
}

/// The toolchain-packaging scenarios "A removal the accepted specifications have not taken" and "A
/// change whose only unapplied work is a withdrawal".
#[test]
fn a_withdrawal_the_merge_did_not_apply_has_not_been_synced() {
    let scratch = Scratch::new("withdrawal-unapplied");
    let directory = a_change(&scratch, "a-change");
    finish_every_task(&directory);
    let specs = scratch.0.join("htmlspec").join("specs");
    write_specification(&specs, &withdrawn_requirement(""));
    write_specification(&directory, &withdrawn_requirement("removed"));

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("0 of 1"),
        "an unapplied withdrawal was reached:\n{out}"
    );
    assert!(
        out.contains("held     req-withdrawn"),
        "sync does not name the withdrawal still held:\n{out}"
    );
    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change", "--json"]);
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(payload["reached"], serde_json::json!([]), "{payload}");
    assert_eq!(
        payload["untakenRemovals"],
        serde_json::json!(["req-withdrawn"]),
        "{payload}"
    );

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("this change has not been synced"),
        "archival called an unapplied withdrawal synced:\n{out}"
    );
    for expected in ["req-withdrawn", "htmlspec sync --change a-change"] {
        assert!(
            err.contains(expected),
            "archival does not name {expected}:\n{err}"
        );
    }
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check", "--json"],
    );
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(payload["needsSync"], true, "{payload}");
    assert_eq!(
        payload["untakenRemovals"],
        serde_json::json!(["req-withdrawn"]),
        "{payload}"
    );
}

/// The toolchain-packaging scenario "Every reported set is worked".
///
/// The keys are read from a payload `sync` emits rather than listed here, because a list in this
/// file would be a second statement of the payload, and it would drift the first time a set was
/// added. Every key whose value is an array is a set, and each must appear in backticks in the
/// instructions an agent reads, or the agent is shown work it is not told how to do.
#[test]
fn every_set_the_sync_payload_carries_is_named_in_its_instructions() {
    let scratch = Scratch::new("payload-named");
    let directory = a_change(&scratch, "a-change");
    let specs = scratch.0.join("htmlspec").join("specs");
    write_specification(&specs, VERIFIED_REQUIREMENT);
    write_specification(&directory, &renamed_requirement());

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change", "--json"]);
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    let sets: Vec<&String> = payload
        .as_object()
        .expect("the payload is an object")
        .iter()
        .filter(|(_, value)| value.is_array())
        .map(|(key, _)| key)
        .collect();
    assert!(
        sets.len() >= 4,
        "the payload carries fewer sets than it did when this was written: {payload}"
    );

    let instructions = htmlspec_assets::instructions::workflow_for("sync")
        .expect("the sync workflow ships")
        .body;
    let unnamed: Vec<&&String> = sets
        .iter()
        .filter(|key| !instructions.contains(&format!("`{key}`")))
        .collect();
    assert!(
        unnamed.is_empty(),
        "the sync payload carries {unnamed:?}, and its instructions never name them"
    );
}

#[test]
fn a_valid_specification_lets_archival_proceed() {
    let scratch = Scratch::new("valid");
    let directory = a_change(&scratch, "a-change");
    write_specification(&directory, VERIFIED_REQUIREMENT);

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("ready"), "{out}");
}

/// The gate: archival does not proceed while the specification reports an error.
#[test]
fn an_invalid_specification_stops_archival() {
    let scratch = Scratch::new("invalid");
    let directory = a_change(&scratch, "a-change");
    // A dangling reference, which is an error rather than a warning. Only the
    // verification's target moves, so the requirement it named still exists under its
    // own identifier and the reference resolves to nothing.
    write_specification(
        &directory,
        &VERIFIED_REQUIREMENT.replace("data-refs=\"req-established\"", "data-refs=\"req-absent\""),
    );

    let (code, out, err) = archive_in(&directory);
    assert_ne!(code, 0, "archival proceeded with an invalid specification");
    assert!(out.contains("refs/unresolved"), "{out}");
    assert!(err.contains("does not proceed"), "{err}");
}

/// A task unit in the specification stops archival, which is the one placement rule
/// reaching the operation that matters most for it.
#[test]
fn a_specification_carrying_a_task_stops_archival() {
    let scratch = Scratch::new("task");
    let directory = a_change(&scratch, "a-change");
    write_specification(
        &directory,
        &format!(
            "{VERIFIED_REQUIREMENT}\n      <section class=\"task\" id=\"task-done\" \
             data-state=\"done\"><h2>The work</h2></section>"
        ),
    );

    let (code, out, _) = archive_in(&directory);
    assert_ne!(code, 0);
    assert!(out.contains("kind/task-in-specification"), "{out}");
}

/// Task 3.8: the severity split, asserted at both operations against one document.
///
/// This is the whole shape of the example-content rule. `htmlspec validate` accepts the
/// document, because a document carries example content the instant it is created and
/// `document-template` requires that result to be valid. `htmlspec archive` refuses, because
/// archiving `req-example` would put an obligation nobody wrote into the accepted contract.
///
/// The two assertions are in one test on purpose. Written apart, each would pass while the
/// split was broken in the direction the other one covers.
#[test]
fn example_content_warns_at_validation_and_refuses_at_archival() {
    let scratch = Scratch::new("example-content");
    let directory = a_change(&scratch, "a-change");
    // The half-replaced case: this requirement's prose is the change's own and its identifier
    // is still the template's.
    let path = write_specification(
        &directory,
        &VERIFIED_REQUIREMENT
            .replace("req-established", "req-example")
            .replace("scenario-established", "scenario-example")
            .replace("verify-established", "verify-example"),
    );

    // Validation accepts it, and says so as a warning.
    let (validate_code, validate_out, _) =
        run_in(&scratch.0, &["validate", &path.to_string_lossy()]);
    assert_eq!(
        validate_code, 0,
        "validation failed a document the program itself writes:\n{validate_out}"
    );
    assert!(
        validate_out.contains("review/example-content"),
        "validation did not report the example content at all:\n{validate_out}"
    );
    assert!(
        validate_out.contains("warning: review/example-content"),
        "the report is not a warning:\n{validate_out}"
    );

    // And it survives `--strict`, which is why the rule is in `review/`. A promotable report
    // would fail a freshly created document here.
    let (strict_code, strict_out, _) = run_in(
        &scratch.0,
        &["validate", &path.to_string_lossy(), "--strict"],
    );
    assert_eq!(
        strict_code, 0,
        "--strict promoted the example-content warning:\n{strict_out}"
    );

    // Archival refuses, and names every identifier it found.
    let (archive_code, archive_out, archive_err) = archive_in(&directory);
    assert_ne!(
        archive_code, 0,
        "archival proceeded with example content in the specification:\n{archive_out}"
    );
    assert!(
        archive_err.contains("does not proceed while the specification carries"),
        "{archive_err}"
    );
    for id in ["req-example", "scenario-example", "verify-example"] {
        assert!(
            archive_err.contains(id),
            "archival did not name {id}:\n{archive_err}"
        );
    }
}

/// Replacing the identifiers clears the refusal, so the check is about the identifier and not
/// about the document's shape.
#[test]
fn archival_proceeds_once_the_example_identifiers_are_replaced() {
    let scratch = Scratch::new("example-replaced");
    let directory = a_change(&scratch, "a-change");
    write_specification(&directory, VERIFIED_REQUIREMENT);

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    // The refusal is absent. `a_change` writes a tasks document from the template, so
    // `task-example` still appears in the open-task note — which is a different report about a
    // different document, and asserting on the word alone would confuse the two.
    assert!(
        !err.contains("does not proceed while the specification carries"),
        "a specification carrying no example identifier was refused:\n{err}"
    );
    assert!(
        !out.contains("review/example-content"),
        "the specification was reported for example content it does not carry:\n{out}"
    );
}

/// The comparison refuses on example content too, and refuses *before* it reports a merge.
///
/// Archival is not the only operation that puts a change's contract into the accepted
/// specifications. `sync` compares the two and names every requirement no accepted
/// specification holds, directing an agent to merge each one — so on a specification straight
/// from the template that direction reads "merge `req-example`", and an agent that complies has
/// put an obligation nobody wrote into the contract with every check green. The refusal arriving
/// *instead of* that instruction is the property, which is why the absence of the note is
/// asserted rather than only the presence of the refusal.
#[test]
fn the_comparison_refuses_example_content_instead_of_reporting_a_merge() {
    let scratch = Scratch::new("sync-example-content");
    let directory = a_change(&scratch, "a-change");
    write_specification(
        &directory,
        &VERIFIED_REQUIREMENT
            .replace("req-established", "req-example")
            .replace("scenario-established", "scenario-example")
            .replace("verify-established", "verify-example"),
    );

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_ne!(
        code, 0,
        "the comparison ran against a specification carrying example content:\n{out}{err}"
    );
    assert!(
        err.contains("does not run while the specification carries"),
        "{err}"
    );
    for id in ["req-example", "scenario-example", "verify-example"] {
        assert!(err.contains(id), "the refusal did not name {id}:\n{err}");
    }
    // The instruction an agent would have followed. Its absence is the point of the gate.
    assert!(
        !out.contains("Merge each into the specification"),
        "the comparison told an agent to merge the template's example content:\n{out}"
    );
    assert!(
        !out.contains("absent"),
        "the comparison reported example content as a requirement to merge:\n{out}"
    );
}

/// Replacing the identifiers clears the comparison's refusal as well as archival's.
///
/// Both gates read one function, so this is what shows the shared refusal is about the
/// identifier rather than about the document.
#[test]
fn the_comparison_proceeds_once_the_example_identifiers_are_replaced() {
    let scratch = Scratch::new("sync-example-replaced");
    let directory = a_change(&scratch, "a-change");
    write_specification(&directory, VERIFIED_REQUIREMENT);

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !err.contains("does not run while the specification carries"),
        "a specification carrying no example identifier was refused:\n{err}"
    );
    // And it does the work it exists for: `req-established` is stated and no accepted
    // specification holds it, so the note a real change needs is the one that arrives.
    assert!(
        out.contains("req-established"),
        "the comparison named no requirement:\n{out}"
    );
}

/// Task 5.3: archival refuses a specification whose only fault is its summary.
///
/// **The case that is invisible today.** Every unit is the author's own, so no example identifier
/// remains for archival's existing refusal to find, and validation reported the document accepted
/// with zero diagnostics before this change. Left to itself it archives, and "Replace this
/// paragraph" enters the project's permanent record — which has already happened once in this
/// repository, at commit `4a89958`.
///
/// Validation still accepts it, which is the severity split holding at both operations against one
/// document. Written as one test for the reason
/// `example_content_warns_at_validation_and_refuses_at_archival` gives: apart, each would pass
/// while the split was broken in the direction the other covers.
#[test]
fn a_placeholder_summary_warns_at_validation_and_refuses_at_archival() {
    let scratch = Scratch::new("placeholder-summary");
    let directory = a_change(&scratch, "a-change");
    let path = write_specification_with_summary(
        &directory,
        &the_specification_placeholder(),
        VERIFIED_REQUIREMENT,
    );

    // Validation accepts it, as a warning.
    let (validate_code, validate_out, _) =
        run_in(&scratch.0, &["validate", &path.to_string_lossy()]);
    assert_eq!(
        validate_code, 0,
        "validation failed a document whose summary the program itself wrote:\n{validate_out}"
    );
    assert!(
        validate_out.contains("warning: review/example-content"),
        "validation did not report the placeholder summary as a warning:\n{validate_out}"
    );

    // And it survives `--strict`, since `htmlspec new document` writes this summary every time.
    let (strict_code, strict_out, _) = run_in(
        &scratch.0,
        &["validate", &path.to_string_lossy(), "--strict"],
    );
    assert_eq!(
        strict_code, 0,
        "--strict promoted the placeholder-summary warning:\n{strict_out}"
    );

    // Archival refuses.
    let (archive_code, archive_out, archive_err) = archive_in(&directory);
    assert_ne!(
        archive_code, 0,
        "archival proceeded with a placeholder summary in the specification:\n{archive_out}"
    );
    assert!(
        archive_err.contains("does not proceed while the specification carries"),
        "{archive_err}"
    );
    // Task 5.2: the summary is named as a summary, not listed among unit identifiers.
    assert!(
        archive_err.contains("the masthead summary"),
        "the refusal does not name the summary distinctly:\n{archive_err}"
    );
    // And the reason is the summary's rather than the requirement's, since no example
    // requirement is present to state one about.
    assert!(
        archive_err.contains("states nothing about what the document is for"),
        "the refusal gives the example-requirement reason for a document that carries none, so a \
         contributor who checked would find the reason wrong:\n{archive_err}"
    );
    assert!(
        !archive_err.contains("An example requirement states no obligation"),
        "the refusal names an obligation nobody wrote against a document whose requirements are \
         all the author's own:\n{archive_err}"
    );
}

/// Task 5.4: the comparison refuses the same specification, before reporting a merge.
///
/// The report this replaces names every requirement no accepted specification holds and directs an
/// agent to merge each one. Deciding which accepted specification a requirement belongs under is
/// what an agent reads a summary to do — that is what the summary is for — so a placeholder there
/// is a merge directed by a document with the deciding text missing. The absence of the
/// instruction is asserted, not only the presence of the refusal.
#[test]
fn the_comparison_refuses_a_placeholder_summary_instead_of_reporting_a_merge() {
    let scratch = Scratch::new("sync-placeholder-summary");
    let directory = a_change(&scratch, "a-change");
    write_specification_with_summary(
        &directory,
        &the_specification_placeholder(),
        VERIFIED_REQUIREMENT,
    );

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_ne!(
        code, 0,
        "the comparison ran against a specification with a placeholder summary:\n{out}{err}"
    );
    assert!(
        err.contains("does not run while the specification carries"),
        "{err}"
    );
    assert!(
        err.contains("the masthead summary"),
        "the refusal does not name the summary distinctly:\n{err}"
    );
    // The instruction an agent would have followed. Its absence is the point of the gate.
    assert!(
        !out.contains("Merge each into the specification"),
        "the comparison told an agent to merge out of a specification that does not say what it \
         is for:\n{out}"
    );
    assert!(
        !out.contains("req-established"),
        "the comparison reported a requirement to merge before refusing:\n{out}"
    );
}

/// Replacing the masthead paragraph clears both refusals, so the check is about the summary's text
/// rather than about the document having a header at all.
///
/// The mirror of `archival_proceeds_once_the_example_identifiers_are_replaced`, and it is what
/// shows the gate is not simply "a specification with a masthead is refused". Both operations are
/// asserted, because both read one function and a test of either alone would not show the shared
/// refusal is shared.
#[test]
fn both_operations_proceed_once_the_masthead_is_the_authors_own() {
    let scratch = Scratch::new("summary-replaced");
    let directory = a_change(&scratch, "a-change");
    write_specification_with_summary(
        &directory,
        "This change reports a specification whose masthead is still the template's words, \
         which validated with no diagnostics before it.",
        VERIFIED_REQUIREMENT,
    );

    let (sync_code, sync_out, sync_err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(sync_code, 0, "{sync_out}{sync_err}");
    assert!(
        !sync_err.contains("does not run while the specification carries"),
        "a specification whose summary is the author's own was refused:\n{sync_err}"
    );
    assert!(
        sync_out.contains("req-established"),
        "the comparison named no requirement:\n{sync_out}"
    );

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !err.contains("does not proceed while the specification carries"),
        "a specification whose summary is the author's own was refused:\n{err}"
    );
}

/// A summary quoting the placeholder inside the author's own statement does not refuse either.
///
/// The design's central decision, asserted at the operation where it costs the most. A substring
/// search here would refuse a specification whose author explained the rule in its masthead — and
/// this project's own changes do that, so the refusal would land on the change that introduced it.
#[test]
fn a_specification_quoting_the_placeholder_is_not_refused() {
    let scratch = Scratch::new("summary-quoted");
    let directory = a_change(&scratch, "a-change");
    write_specification_with_summary(
        &directory,
        &format!(
            "Unlike a document whose whole summary reads {} this one states the contract the \
             change established.",
            the_specification_placeholder()
        ),
        VERIFIED_REQUIREMENT,
    );

    let (code, out, err) = archive_in(&directory);
    assert_eq!(
        code, 0,
        "a specification quoting the placeholder inside the author's own statement was refused, \
         which would refuse the change that introduced this rule:\n{out}{err}"
    );
}

/// An unverified requirement is a warning, so archival still proceeds.
///
/// This is the `review/*` placement holding at the operation it was decided for.
/// Verification arrives as work completes, and a specification whose verification is
/// not yet recorded is in progress rather than wrong.
#[test]
fn an_unverified_requirement_does_not_stop_archival() {
    let scratch = Scratch::new("unverified");
    let directory = a_change(&scratch, "a-change");
    write_specification(
        &directory,
        "      <section class=\"requirement\" id=\"req-unverified\">\n\
         \x20       <h2>Not yet checked</h2>\n\
         \x20       <p>The system SHALL do the thing.</p>\n\
         \x20       <article class=\"scenario\" id=\"scenario-unverified\">\n\
         \x20         <h3>It holds</h3>\n\
         \x20         <dl><dt>When</dt><dd>It runs.</dd><dt>Then</dt><dd>It works.</dd></dl>\n\
         \x20       </article>\n\
         \x20     </section>",
    );

    let (code, out, err) = archive_in(&directory);
    assert_eq!(code, 0, "an advisory warning gated archival:\n{out}{err}");
    assert!(out.contains("review/unverified-requirement"), "{out}");
    assert!(out.contains("ready"), "{out}");
}

/// A change with no specification is refused with the reason.
#[test]
fn a_change_with_no_specification_is_refused() {
    let scratch = Scratch::new("absent");
    let directory = a_change(&scratch, "a-change");

    let (code, _, err) = archive_in(&directory);
    assert_ne!(code, 0);
    assert!(err.contains("has no specification.html"), "{err}");
    // The message says whose job producing it is, because the answer is not the
    // program's and a contributor reading only this error would assume it was.
    assert!(err.contains("agent"), "{err}");
}

/// A document declaring another kind is not the change's specification.
#[test]
fn a_specification_declaring_another_kind_is_refused() {
    let scratch = Scratch::new("wrong-kind");
    let directory = a_change(&scratch, "a-change");
    let path = write_specification(&directory, VERIFIED_REQUIREMENT);
    let source = std::fs::read_to_string(&path).expect("readable");
    std::fs::write(
        &path,
        source.replace(
            r#"content="specification" name="htmlspec-kind""#,
            r#"content="design" name="htmlspec-kind""#,
        ),
    )
    .expect("written");

    let (code, _, err) = archive_in(&directory);
    assert_ne!(code, 0);
    assert!(err.contains("declares the design kind"), "{err}");
}

/// Every document survives archival, which is the reason for keeping the stages
/// separate at all.
#[test]
fn the_changes_documents_survive_the_check() {
    let scratch = Scratch::new("survive");
    let directory = a_change(&scratch, "a-change");
    write_specification(&directory, VERIFIED_REQUIREMENT);

    let (code, out, err) = archive_performing_in(&directory);
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");

    // The change moved, so the documents are looked for where it went. Keeping every document
    // is the whole reason the stages are separate: a reader opening an archived change must
    // find the argument, the reasoning, the work, and the contract.
    let archived = the_only_archived_change(&scratch.0);
    for name in [
        "proposal.html",
        "design.html",
        "tasks.html",
        "specification.html",
    ] {
        assert!(
            archived.join(name).exists(),
            "{name} did not survive archival"
        );
    }
    assert!(
        !directory.exists(),
        "the change is still among the active changes"
    );
}

/// The one directory in the project's archive.
fn the_only_archived_change(project: &Path) -> PathBuf {
    let archive = project.join("htmlspec").join("changes").join("archive");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&archive)
        .expect("the archive is readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "expected one archived change in {archive:?}"
    );
    entries.pop().expect("one entry")
}

/// Task 9.5: the archival operation ships no parser that reads a proposal or a
/// design and emits requirements.
///
/// A claim about absence, so it is asserted two ways. First behaviorally: a change
/// whose proposal and design are full of prose that *describes* obligations produces
/// no specification at all, and the command says whose job that is. An extractor
/// would have written one.
///
/// The upstream kinds carry no imposed structure, so there is nothing to parse
/// reliably — deciding which paragraph of a design states an obligation is reading
/// rather than parsing. An extractor that guessed would emit a specification that
/// passes every check and misstates its source, which is worse than producing
/// nothing because it looks authoritative.
#[test]
fn the_archive_operation_ships_no_requirement_extractor() {
    let scratch = Scratch::new("no-extractor");
    let directory = a_change(&scratch, "a-change");

    // A proposal and a design whose prose is full of obligation language. An
    // extractor pointed at these would have plenty to work with.
    for name in ["proposal.html", "design.html"] {
        let path = directory.join(name);
        let source = std::fs::read_to_string(&path).expect("readable");
        let rewritten = source.replace(
            "</main>",
            "<h2>Obligations</h2>\
             <p>The system SHALL persist the reader's choice of theme.</p>\
             <p>Every text and background pair MUST meet a contrast ratio of 4.5 to 1.</p>\
             <p>The application MUST follow the operating system when no choice is recorded.</p>\
             </main>",
        );
        std::fs::write(&path, rewritten).expect("written");
    }

    let (code, out, err) = archive_in(&directory);

    // No specification was derived from that prose. The command refuses and names
    // whose work it is.
    assert_ne!(
        code, 0,
        "archival succeeded with no specification, which means something produced one \
         from the proposal and design:\n{out}{err}"
    );
    assert!(
        !directory.join("specification.html").exists(),
        "a specification was written from a proposal and a design. Those kinds carry no \
         imposed structure, so anything that produced this guessed at which prose states \
         an obligation — and a wrong guess passes every check while misstating the \
         contract"
    );
    assert!(err.contains("has no specification.html"), "{err}");
}

/// The second half of task 9.5, as a source constraint: the archival command reads
/// one file.
///
/// The behavioral test above passes for a command that reads every document and
/// happens not to write one yet. This asserts the stronger property at the layer
/// where an extractor would have to arrive: `archive_check` resolves the
/// specification by name, and never opens a proposal or a design.
#[test]
fn the_archive_check_reads_only_the_specification() {
    let source =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands.rs"))
            .expect("the command source is readable");

    let body = source
        .split_once("pub fn archive<")
        .and_then(|(_, rest)| rest.split_once("\n/// "))
        .map(|(body, _)| body)
        .expect("archive is defined and something follows it");

    // It resolves exactly one document, by name.
    assert!(
        body.contains("directory.join(CHANGE_SPECIFICATION)"),
        "archive no longer resolves the specification by name"
    );

    // And nothing in the archival path reads either upstream kind. This is the load-bearing
    // assertion and it widened when archival grew: the operation now reads the tasks document
    // and the accepted specifications too, so counting reads no longer distinguishes an
    // extractor from ordinary work. Naming a proposal or a design still does. `CHANGE_DOCUMENTS`
    // lists both by name, but only to report which documents are absent, so the check is on the
    // functions that make up archival rather than on the file.
    for function in [
        "pub fn archive<",
        "fn findings_for",
        "fn open_tasks",
        // Renamed from `unmerged_requirements` when `sync` became a second caller of the
        // same comparison. The assertion is unchanged: whatever the function is called, it
        // must not read an upstream kind.
        "fn merge_state",
        // The content comparison `merge_state` calls, and the function under it. Each reads the
        // projections it is handed and nothing else, and this keeps it that way.
        "fn untaken_content",
        "fn content_difference",
        // The index of the accepted contract and the comparison over it, taken out of
        // `merge_state` so that `show --diff` reads the same ones.
        "fn accepted_requirements",
        "fn compare_content",
        "fn perform<",
    ] {
        let region = source
            .split_once(function)
            .and_then(|(_, rest)| rest.split_once("\n/// "))
            .map(|(body, _)| body)
            .unwrap_or_else(|| panic!("{function} is defined and something follows it"));
        for forbidden in ["proposal.html", "design.html"] {
            assert!(
                !region.contains(forbidden),
                "{function} names {forbidden}. The upstream kinds have no imposed \
                 structure, so a command that opened one would be guessing at what its \
                 prose obliges"
            );
        }
    }
}

/// The view beside it: `show --diff` reads a change's specification and the accepted ones, and
/// writes nothing and reads neither upstream kind.
///
/// toolchain-packaging's "The command surface shows a modified requirement beside the accepted
/// one" says the view reads and never writes, because a view that wrote would be a merge under
/// another name. `tests/show.rs` holds every run of the view to that by its effect on disk; this
/// holds the source to it, so a write added on a path no test reaches is caught as well.
#[test]
fn the_view_reads_only_the_specifications() {
    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/show.rs"))
        .expect("the view's source is readable");
    // The view's code, without its tests and without comments, which may name what it never does.
    let code: String = source
        .split("#[cfg(test)]")
        .next()
        .expect("the source has a first part")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "fs::write",
        "create_dir",
        "fs::rename",
        "fs::remove",
        "File::create",
        "OpenOptions",
        "proposal.html",
        "design.html",
    ] {
        assert!(
            !code.contains(forbidden),
            "show.rs names {forbidden}. The view reads a change's specification and the accepted \
             specifications, and writes nothing"
        );
    }
}

// -----------------------------------------------------------------------------
// The version gate
// -----------------------------------------------------------------------------

/// The change plus a valid specification, ready to archive.
fn a_change_ready_to_archive(scratch: &Scratch, name: &str) -> PathBuf {
    let directory = a_change(scratch, name);
    write_specification(&directory, VERIFIED_REQUIREMENT);
    directory
}

/// An archived change, recording `version` where one is given.
///
/// The version goes in a `VERSION` file, which is where `toolchain-packaging` requires an
/// archived change to record it and where `scripts/check-release` reads it. A fixture that
/// encoded the version in the directory name instead is what let the gate's defect survive:
/// the suite built names in a shape nothing else in the project ever wrote, so the gate passed
/// against fixtures and compared against `0.0.0` in every real project.
fn an_archived_change(project: &Path, name: &str, version: Option<&str>) -> PathBuf {
    let directory = project
        .join("htmlspec")
        .join("changes")
        .join("archive")
        .join(name);
    std::fs::create_dir_all(&directory).expect("the archived change is created");
    if let Some(version) = version {
        std::fs::write(directory.join("VERSION"), format!("{version}\n")).expect("written");
    }
    directory
}

/// The gate reads the version an archived change records, not its directory name.
///
/// This is the regression test for a gate that never fired. `last_archived_version` scanned
/// directory names for a dotted triple, and no archive directory in any project carries one —
/// archival did not create archive directories at all. So the scan always found nothing and
/// every declaration was compared against `0.0.0`.
///
/// The fixture is deliberately named the way real archives are named: a date and a change
/// name, with the version only in the file.
#[test]
fn the_version_gate_reads_the_recorded_version_not_the_directory_name() {
    let scratch = Scratch::new("gate-recorded");
    let directory = a_change_ready_to_archive(&scratch, "recorded");
    an_archived_change(
        &scratch.0,
        "2026-08-24-report-in-the-contributors-terms",
        Some("0.5.0"),
    );

    // 0.1.0 is ordered before the recorded 0.5.0. Under the directory-name scan this passed,
    // because the name carries no version and the previous version resolved to 0.0.0.
    advance_version(&scratch.0);
    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "the gate compared against 0.0.0 again: {err}");
    assert!(err.contains("0.5.0"), "{err}");
    assert!(err.contains("ordered *before*"), "{err}");
}

/// An archived change recording no version does not lower the answer.
///
/// This feeds a `max`, so a change with no `VERSION` — one archived before the convention
/// existed — must be skipped rather than treated as `0.0.0`. Reporting it belongs to
/// `scripts/check-release`, which names the change; refusing here would block an archival over
/// a file belonging to some other change.
#[test]
fn an_archived_change_recording_no_version_does_not_lower_the_gate() {
    let scratch = Scratch::new("gate-unrecorded");
    let directory = a_change_ready_to_archive(&scratch, "unrecorded");
    an_archived_change(&scratch.0, "2026-08-24-recorded", Some("0.5.0"));
    an_archived_change(&scratch.0, "2026-08-25-recorded-nothing", None);
    an_archived_change(
        &scratch.0,
        "2026-08-26-recorded-nonsense",
        Some("not-a-version"),
    );

    // The maximum is still 0.5.0, so 0.6.0 is accepted and 0.5.0 is not.
    let path = scratch.0.join("spec.nix");
    let declaration = std::fs::read_to_string(&path).expect("readable");
    std::fs::write(&path, declaration.replace("0.0.0", "0.5.0")).expect("written");
    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("still 0.5.0"), "{err}");

    let declaration = std::fs::read_to_string(&path).expect("readable");
    std::fs::write(&path, declaration.replace("0.5.0", "0.6.0")).expect("written");
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");
    assert!(err.contains("follows 0.5.0"), "{err}");
}

/// Archival refuses while the version has not moved. With nothing archived the previous
/// version is the initial `0.0.0` that `init` wrote, so this is also the first-archival
/// case — and it behaves exactly like any later one.
#[test]
fn the_version_gate_refuses_a_version_that_has_not_moved() {
    let scratch = Scratch::new("gate-unchanged");
    let directory = a_change_ready_to_archive(&scratch, "unchanged");

    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("still 0.0.0"), "{err}");
    assert!(err.contains("compares and never edits"), "{err}");
}

/// The first archival is not a special case. Bootstrapping writes `0.0.0`, so the very
/// first change must advance past it exactly as the thousandth advances past its
/// predecessor — there is no absent previous version for the gate to wave through.
#[test]
fn a_projects_first_archival_is_an_ordinary_comparison() {
    let scratch = Scratch::new("gate-first");
    let directory = a_change_ready_to_archive(&scratch, "first");

    // Nothing has ever been archived here.
    let archive = scratch.0.join("htmlspec").join("changes").join("archive");
    assert_eq!(
        std::fs::read_dir(&archive)
            .expect("the archive exists")
            .count(),
        0,
        "the archive is not empty, so this is not the first-archival case"
    );

    // Refused at 0.0.0, on the same grounds any later archival is refused.
    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("still 0.0.0"), "{err}");

    // And accepted once advanced. Checked rather than performed: the subject is the gate.
    advance_version(&scratch.0);
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");
}

/// A version ordered *before* the last archived one is a different problem from one that
/// has not moved, and it is reported as such. A difference test would accept it.
#[test]
fn the_version_gate_refuses_a_version_that_went_backwards() {
    let scratch = Scratch::new("gate-backwards");
    let directory = a_change_ready_to_archive(&scratch, "backwards");

    // An archive entry recording a later version than the one declared.
    an_archived_change(&scratch.0, "2026-08-14-something-earlier", Some("0.9.0"));

    // Declared 0.1.0, which is ordered before the archived 0.9.0.
    advance_version(&scratch.0);
    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("ordered *before*"), "{err}");
    assert!(err.contains("0.9.0"), "{err}");
}

/// A version the program cannot order is reported rather than accepted, because a gate
/// that cannot compare its inputs is not a gate.
#[test]
fn the_version_gate_refuses_an_unorderable_version() {
    let scratch = Scratch::new("gate-unorderable");
    let directory = a_change_ready_to_archive(&scratch, "unorderable");

    let path = scratch.0.join("spec.nix");
    let declaration = std::fs::read_to_string(&path).expect("readable");
    std::fs::write(&path, declaration.replace("\"0.0.0\"", "\"tuesday\"")).expect("written");

    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("cannot be ordered"), "{err}");
}

/// The declaration must exist, and its absence names the operation that writes one rather
/// than passing the gate by default.
#[test]
fn the_version_gate_refuses_an_absent_declaration() {
    let scratch = Scratch::new("gate-absent");
    let directory = a_change_ready_to_archive(&scratch, "absent");
    std::fs::remove_file(scratch.0.join("spec.nix")).expect("removed");

    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("htmlspec init"), "{err}");
}

/// Task 9.5: `archive` compares and never edits. Snapshotted across both a refusal and a
/// success, because a convenience bump added later would pass a test that only checked one.
#[test]
fn the_archive_check_does_not_write_the_declaration() {
    let scratch = Scratch::new("gate-no-write");
    let directory = a_change_ready_to_archive(&scratch, "no-write");
    let path = scratch.0.join("spec.nix");

    // A refusal leaves it alone.
    let before = std::fs::read_to_string(&path).expect("readable");
    let (code, _, _) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2);
    assert_eq!(
        std::fs::read_to_string(&path).expect("readable"),
        before,
        "a refused archival edited the declaration"
    );

    // And so does a success — which now moves the change, writes a VERSION file, and still
    // must not touch the declaration. That is exactly the obligation the withdrawal preserved:
    // archival writes to the project, and advancing the version remains the contributor's act.
    advance_version(&scratch.0);
    let advanced = std::fs::read_to_string(&path).expect("readable");
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");
    assert_eq!(
        std::fs::read_to_string(&path).expect("readable"),
        advanced,
        "a successful archival advanced the version itself, which is the contributor's work"
    );
}

// Archival that performs
// -----------------------------------------------------------------------------

/// Every path and its bytes beneath `root`, sorted.
///
/// Atomicity is the claim that a refused archival leaves the project as it found it, and the
/// only way to check that claim is to compare the whole tree rather than the one file the
/// operation was expected to touch. An operation that moved a change and moved it back would
/// pass a narrower assertion.
fn tree(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(directory: &Path, base: &Path, into: &mut Vec<(String, Vec<u8>)>) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, into);
            } else if let Ok(bytes) = std::fs::read(&path) {
                let relative = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .into_owned();
                into.push((relative, bytes));
            }
        }
    }
    let mut found = Vec::new();
    walk(root, root, &mut found);
    found.sort();
    found
}

/// A refused archival changes nothing at all.
#[test]
fn a_refused_archival_leaves_the_project_untouched() {
    let scratch = Scratch::new("perform-refused");
    let directory = a_change(&scratch, "refused");
    // No specification, so the gate refuses.
    let before = tree(&scratch.0);

    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    assert_eq!(
        tree(&scratch.0),
        before,
        "a refused archival modified the project"
    );
}

/// Findings with nothing able to answer them stop the operation rather than archiving over
/// them silently. This is the case an agent loop hits.
#[test]
fn findings_with_no_one_to_ask_stop_the_archival() {
    let scratch = Scratch::new("perform-noask");
    let directory = a_change_ready_to_archive(&scratch, "noask");
    advance_version(&scratch.0);
    let before = tree(&scratch.0);

    let (code, _, err) = run_in(&scratch.0, &["archive", &directory.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("--yes"),
        "the report does not name the way through: {err}"
    );
    assert_eq!(
        tree(&scratch.0),
        before,
        "a stopped archival modified the project"
    );
}

/// The change moves, is named for the date, and records the version it shipped in.
#[test]
fn a_completed_archival_moves_the_change_and_records_its_version() {
    let scratch = Scratch::new("perform-moves");
    let directory = a_change_ready_to_archive(&scratch, "moves");

    let (code, out, err) = archive_performing_in(&directory);
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");

    let archived = the_only_archived_change(&scratch.0);
    let name = archived
        .file_name()
        .expect("a name")
        .to_string_lossy()
        .into_owned();
    assert!(name.ends_with("-moves"), "the archived name is {name}");
    assert!(
        name.len() > "moves".len() + 1 && name.as_bytes()[4] == b'-',
        "the archived name carries no date: {name}"
    );
    assert_eq!(
        std::fs::read_to_string(archived.join("VERSION"))
            .expect("the version is recorded")
            .trim(),
        "0.1.0"
    );
    assert!(
        !directory.exists(),
        "the change is still among the active changes"
    );
}

/// toolchain-packaging's "Archival produces the specification and updates the accepted contract",
/// for the references a change's documents carry.
///
/// The first change this repository archived with `htmlspec archive` came out of it with every
/// document referencing `../../assets/`, one level short of the store from the archive, and a link to
/// an accepted requirement that led nowhere. Each kind of reference a change carries is here: the
/// store, the accepted specifications, the change's own files, a page outside the project, an
/// author's quotation of a reference, and a record in the change's evidence.
#[test]
fn archival_keeps_every_reference_reaching_what_it_reached() {
    let scratch = Scratch::new("perform-references");
    let directory = a_change_ready_to_archive(&scratch, "references");

    let proposal = directory.join("proposal.html");
    let source = std::fs::read_to_string(&proposal).expect("the proposal reads");
    let (opening, rest) = source
        .split_once("</h2>")
        .expect("the proposal has a section heading");
    let links = "<p><a href=\"../../specs/\">The accepted specifications</a>, \
                 <a href=\"evidence/a-record.html\">a record</a>, \
                 <a href=\"tasks.html#task-example\">a task</a>, \
                 <a href=\"https://example.com/\">a page elsewhere</a>, and a quotation: \
                 <code>href=\"../../assets/quoted.css\"</code>.</p>";
    std::fs::write(&proposal, format!("{opening}</h2>{links}{rest}")).expect("written");
    let (code, out, err) = run_in(&scratch.0, &["format", &directory.to_string_lossy()]);
    assert_eq!(code, 0, "the change does not format:\n{out}{err}");

    // A record carrying the store reference the documents carry, written after formatting so
    // nothing but archival could touch it.
    let before = std::fs::read_to_string(&proposal).expect("the proposal reads");
    let stylesheet = before
        .split("href=\"")
        .find(|value| value.starts_with("../../assets/sha256-"))
        .and_then(|value| value.split('"').next())
        .expect("the proposal references the store's stylesheet")
        .to_owned();
    let record = directory.join("evidence/a-record.html");
    std::fs::create_dir_all(record.parent().expect("the evidence")).expect("created");
    let record_bytes = format!(
        "<!DOCTYPE html>\n<html lang=\"en\"><head><title>A record</title>\
         <link href=\"{stylesheet}\" rel=\"stylesheet\"></head><body><p>As it was.</p></body></html>\n"
    );
    std::fs::write(&record, &record_bytes).expect("written");
    let others: Vec<(&str, String)> = ["design.html", "tasks.html", "specification.html"]
        .into_iter()
        .map(|file| {
            (
                file,
                std::fs::read_to_string(directory.join(file)).expect("readable"),
            )
        })
        .collect();

    let (code, out, err) = archive_performing_in(&directory);
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");
    let archived = the_only_archived_change(&scratch.0);

    // Only the references that left the change climb one level more. Every other byte, including
    // the quotation, the links within the change, and the link elsewhere, is as it was.
    let climbed = |text: &str| {
        text.replace("\"../../assets/sha256-", "\"../../../assets/sha256-")
            .replace("href=\"../../specs/\"", "href=\"../../../specs/\"")
    };
    assert_eq!(
        std::fs::read_to_string(archived.join("proposal.html")).expect("readable"),
        climbed(&before),
        "the archived proposal's references do not reach what they reached"
    );
    for (file, text) in &others {
        assert_eq!(
            std::fs::read_to_string(archived.join(file)).expect("readable"),
            climbed(text),
            "the archived {file}'s references do not reach what they reached"
        );
    }
    assert_eq!(
        std::fs::read_to_string(archived.join("evidence/a-record.html")).expect("readable"),
        record_bytes,
        "archival rewrote a change's evidence"
    );

    let (code, out, err) = run_in(&scratch.0, &["validate", &archived.to_string_lossy()]);
    assert_eq!(
        code, 0,
        "the archived change does not validate:\n{out}{err}"
    );
    let (code, out, err) = run_in(
        &scratch.0,
        &["format", "--check", &archived.to_string_lossy()],
    );
    assert_eq!(code, 0, "the archived change is not canonical:\n{out}{err}");
}

/// A change already named with a date does not get a second one.
#[test]
fn an_archived_change_carries_one_date() {
    let scratch = Scratch::new("perform-dated");
    let directory = a_change_ready_to_archive(&scratch, "2026-08-29-already-dated");

    let (code, out, err) = archive_performing_in(&directory);
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");

    let archived = the_only_archived_change(&scratch.0);
    assert_eq!(
        archived.file_name().expect("a name").to_string_lossy(),
        "2026-08-29-already-dated",
        "archival stacked a second date onto a name that had one"
    );
}

/// An occupied target is refused rather than suffixed, merged, or overwritten.
#[test]
fn an_occupied_archive_target_is_refused() {
    let scratch = Scratch::new("perform-collision");
    let directory = a_change_ready_to_archive(&scratch, "2026-08-29-collides");
    // The target name is the change's own, because it already carries a date.
    let occupied = an_archived_change(&scratch.0, "2026-08-29-collides", Some("0.0.1"));
    std::fs::write(occupied.join("MARKER"), b"the existing archive").expect("written");
    // Past the archived 0.0.1, so the collision is what refuses rather than the version gate.
    advance_version(&scratch.0);
    let before = tree(&scratch.0);

    let (code, _, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("already exists"), "{err}");
    assert_eq!(
        tree(&scratch.0),
        before,
        "a refused collision modified the project"
    );
}

/// `--check` reports readiness, exits zero, and changes nothing.
#[test]
fn check_mode_reports_readiness_without_archiving() {
    let scratch = Scratch::new("check-ready");
    let directory = a_change_ready_to_archive(&scratch, "ready");
    advance_version(&scratch.0);
    let before = tree(&scratch.0);

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");
    // The fixture carries an example task and a requirement no accepted specification holds,
    // so the honest answer is that archival would ask rather than that the change is ready.
    assert!(out.contains("ready with questions"), "{out}");
    assert_eq!(tree(&scratch.0), before, "--check modified the project");
}

/// `--check` fails exactly where archival refuses, so it works as a gate.
#[test]
fn check_mode_fails_where_archival_refuses() {
    let scratch = Scratch::new("check-refused");
    let directory = a_change(&scratch, "refused");
    // A dangling reference, which is an error rather than a warning.
    write_specification(
        &directory,
        &VERIFIED_REQUIREMENT.replace("data-refs=\"req-established\"", "data-refs=\"req-absent\""),
    );
    advance_version(&scratch.0);
    let before = tree(&scratch.0);

    let (code, _, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_ne!(code, 0, "--check accepted a change archival would refuse");
    assert_eq!(tree(&scratch.0), before, "--check modified the project");
    let _ = err;
}

/// The structured payload states whether the change moved.
#[test]
fn the_structured_payload_states_whether_the_change_moved() {
    let scratch = Scratch::new("perform-json");
    let directory = a_change_ready_to_archive(&scratch, "json");
    advance_version(&scratch.0);

    // Checking: not moved, and the findings are in the payload.
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check", "--json"],
    );
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(parsed["moved"], serde_json::Value::Bool(false));
    assert!(
        parsed["unmergedRequirements"]
            .as_array()
            .is_some_and(|found| found.iter().any(|id| id == "req-established")),
        "the payload does not name the unmerged requirement: {parsed}"
    );

    // Performing: moved, with the destination named.
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes", "--json"],
    );
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    assert_eq!(parsed["moved"], serde_json::Value::Bool(true));
    assert!(
        parsed["archivedTo"]
            .as_str()
            .is_some_and(|to| to.ends_with("-json")),
        "the payload does not name where the change went: {parsed}"
    );
}

// list and status
// -----------------------------------------------------------------------------

/// The archive lives beneath `changes/`, so an enumeration that listed directories would
/// report it as a change named `archive`.
#[test]
fn listing_changes_excludes_the_archive() {
    let scratch = Scratch::new("list-archive");
    let _ = a_change(&scratch, "still-active");
    an_archived_change(&scratch.0, "2026-08-01-long-done", Some("0.1.0"));

    let (code, out, err) = run_in(&scratch.0, &["list"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("still-active"), "{out}");
    assert!(
        !out.contains("archive"),
        "the archive was listed as a change: {out}"
    );
    assert!(
        !out.contains("long-done"),
        "an archived change was listed as active: {out}"
    );
}

/// `status` and `archive` report the same count for one change.
///
/// Two counts of one thing drift. A contributor who runs `status`, reads that two tasks are
/// open, and then sees archival report three has been told the project disagrees with itself,
/// and has no way to tell which number to believe.
#[test]
fn status_and_archival_agree_about_open_tasks() {
    let scratch = Scratch::new("status-agree");
    let directory = a_change_ready_to_archive(&scratch, "agree");
    advance_version(&scratch.0);

    let (code, out, err) = run_in(&scratch.0, &["status", "--change", "agree", "--json"]);
    assert_eq!(code, 0, "{err}");
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    let from_status = parsed["tasks"]["openTasks"]
        .as_array()
        .expect("openTasks is an array")
        .iter()
        .filter_map(|task| task.as_str().map(str::to_owned))
        .collect::<Vec<String>>();

    let (code, _, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_eq!(code, 0, "{err}");
    for task in &from_status {
        assert!(
            err.contains(task.as_str()),
            "archival did not name {task}, which status reported open: {err}"
        );
    }
    assert!(
        err.contains(&format!("{} task(s)", from_status.len())),
        "archival reported a different count from status ({}): {err}",
        from_status.len()
    );
}

/// `status` names each artifact and says which are absent. A change mid-implementation has no
/// specification, and that is the ordinary state rather than a fault.
#[test]
fn status_reports_an_absent_specification() {
    let scratch = Scratch::new("status-absent");
    let _ = a_change(&scratch, "mid-flight");

    let (code, out, err) = run_in(&scratch.0, &["status", "--change", "mid-flight", "--json"]);
    assert_eq!(code, 0, "{err}");
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
    let artifacts = parsed["artifacts"].as_array().expect("an array");
    let specification = artifacts
        .iter()
        .find(|artifact| artifact["name"] == "specification.html")
        .expect("the specification is named");
    assert_eq!(specification["present"], serde_json::Value::Bool(false));
}

/// One active change is selected and announced. Several are listed rather than guessed between.
#[test]
fn archival_with_no_argument_selects_or_asks() {
    let scratch = Scratch::new("select");
    let only = a_change_ready_to_archive(&scratch, "the-only-one");
    advance_version(&scratch.0);

    let (code, _, err) = run_in(&scratch.0, &["archive", "--check"]);
    assert_eq!(code, 0, "{err}");
    assert!(err.contains("the-only-one"), "{err}");
    assert!(err.contains("the only active change"), "{err}");
    assert!(only.exists());

    // A second change makes the answer ambiguous, and ambiguity is reported rather than
    // resolved: archival moves a directory and rewrites a contract.
    let (code, out, err) = run_in(
        &scratch.0,
        &["new", "change", "a-second", "--subject", "Another"],
    );
    assert_eq!(code, 0, "{out}{err}");
    let (code, _, err) = run_in(&scratch.0, &["archive", "--check"]);
    assert_ne!(code, 0, "archival guessed between two changes");
    assert!(err.contains("several active changes"), "{err}");
    assert!(err.contains("a-second"), "{err}");
}

// instructions
// -----------------------------------------------------------------------------

/// The emitted instructions are the installed instructions.
///
/// One source, checked rather than intended. `init` writes these into a tool's directory with
/// per-tool frontmatter around them, and the body is what both sides share — so an installed
/// file must contain, verbatim, what the command emits. Two copies of one text drift, and this
/// is what stops them.
#[test]
fn emitted_instructions_are_the_installed_instructions() {
    let scratch = Scratch::new("instructions");
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "claude"]);
    assert_eq!(code, 0, "{out}{err}");

    // Driven from the table rather than from a list written here, so a workflow added later is
    // covered without this test being remembered. The skill directory is its own field and is
    // not derivable from the name: `new` installs as `htmlspec-new-change`.
    for workflow in htmlspec_assets::instructions::WORKFLOWS {
        let stage = workflow.name;
        let (code, emitted, err) = run_in(&scratch.0, &["instructions", stage]);
        assert_eq!(code, 0, "instructions {stage} failed: {err}");
        assert!(!emitted.trim().is_empty(), "{stage} emitted nothing");

        let installed = scratch
            .0
            .join(".claude")
            .join("skills")
            .join(workflow.directory)
            .join("SKILL.md");
        let installed = std::fs::read_to_string(&installed)
            .unwrap_or_else(|error| panic!("{}: {error}", installed.display()));
        assert!(
            installed.contains(emitted.trim_end()),
            "the installed instructions for {stage} do not contain what the command emits"
        );
    }
}

/// An unknown stage names the stages that exist, rather than reporting only that this one
/// does not. A contributor who mistyped needs the list they were choosing from.
#[test]
fn an_unknown_stage_names_the_stages_that_exist() {
    let scratch = Scratch::new("instructions-unknown");
    let _ = a_project(&scratch);

    let (code, _, err) = run_in(&scratch.0, &["instructions", "archivve"]);
    assert_ne!(code, 0);
    assert!(err.contains("archive"), "{err}");
    assert!(err.contains("explore"), "{err}");
}

// store
// -----------------------------------------------------------------------------

/// A registered project is acted on instead of the one containing the working directory.
#[test]
fn a_registered_store_is_acted_on_instead_of_the_local_project() {
    let scratch = Scratch::new("store-acts");
    let config = scratch.0.join("config");

    // Two projects: one we stand in, one we register.
    let here = scratch.0.join("here");
    std::fs::create_dir_all(&here).expect("created");
    let (code, out, err) = run_with_registry(&here, &config, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, err) = run_with_registry(
        &here,
        &config,
        &["new", "change", "local-change", "--subject", "Local"],
    );
    assert_eq!(code, 0, "{out}{err}");

    let elsewhere = scratch.0.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("created");
    let (code, out, err) = run_with_registry(&elsewhere, &config, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, err) = run_with_registry(
        &elsewhere,
        &config,
        &["new", "change", "remote-change", "--subject", "Remote"],
    );
    assert_eq!(code, 0, "{out}{err}");

    let (code, _, err) = run_with_registry(
        &here,
        &config,
        &["store", "add", "far", &elsewhere.to_string_lossy()],
    );
    assert_eq!(code, 0, "{err}");

    // Standing in `here`, unscoped lists the local change.
    let (code, out, err) = run_with_registry(&here, &config, &["list"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("local-change"), "{out}");

    // And scoped lists the registered one instead.
    let (code, out, err) = run_with_registry(&here, &config, &["list", "--store", "far"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("remote-change"), "{out}");
    assert!(
        !out.contains("local-change"),
        "the selection was ignored and the local project was listed: {out}"
    );
}

/// A registration whose project is gone is reported, and nothing falls back.
///
/// This is the failure worth a test: falling back means acting on a different project than the
/// one named, and it would look exactly like success.
#[test]
fn a_store_whose_project_is_gone_does_not_fall_back() {
    let scratch = Scratch::new("store-gone");
    let config = scratch.0.join("config");

    let here = scratch.0.join("here");
    std::fs::create_dir_all(&here).expect("created");
    let (code, out, err) = run_with_registry(&here, &config, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, err) = run_with_registry(
        &here,
        &config,
        &["new", "change", "local-change", "--subject", "Local"],
    );
    assert_eq!(code, 0, "{out}{err}");

    let elsewhere = scratch.0.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("created");
    let (code, _, err) = run_with_registry(&elsewhere, &config, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "{err}");
    let (code, _, err) = run_with_registry(
        &here,
        &config,
        &["store", "add", "far", &elsewhere.to_string_lossy()],
    );
    assert_eq!(code, 0, "{err}");

    // The registered project goes away.
    std::fs::remove_dir_all(&elsewhere).expect("removed");

    let (code, out, err) = run_with_registry(&here, &config, &["list", "--store", "far"]);
    assert_ne!(code, 0, "an unresolvable store was treated as success");
    assert!(err.contains("no longer holds a project"), "{err}");
    assert!(
        !out.contains("local-change"),
        "an unresolvable store fell back to the local project: {out}"
    );
}

/// An unregistered identifier names what is registered.
#[test]
fn an_unregistered_store_names_what_is_registered() {
    let scratch = Scratch::new("store-unknown");
    let config = scratch.0.join("config");
    let here = scratch.0.join("here");
    std::fs::create_dir_all(&here).expect("created");
    let (code, _, err) = run_with_registry(&here, &config, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "{err}");
    let (code, _, err) = run_with_registry(
        &here,
        &config,
        &["store", "add", "known", &here.to_string_lossy()],
    );
    assert_eq!(code, 0, "{err}");

    let (code, _, err) = run_with_registry(&here, &config, &["list", "--store", "mistyped"]);
    assert_ne!(code, 0);
    assert!(err.contains("known"), "{err}");
}

/// Removing a registration leaves the project it named alone.
#[test]
fn removing_a_registration_leaves_the_project() {
    let scratch = Scratch::new("store-remove");
    let config = scratch.0.join("config");
    let here = scratch.0.join("here");
    std::fs::create_dir_all(&here).expect("created");
    let (code, _, err) = run_with_registry(&here, &config, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "{err}");
    let (code, _, err) = run_with_registry(
        &here,
        &config,
        &["store", "add", "gone-soon", &here.to_string_lossy()],
    );
    assert_eq!(code, 0, "{err}");

    let (code, _, err) = run_with_registry(&here, &config, &["store", "remove", "gone-soon"]);
    assert_eq!(code, 0, "{err}");
    assert!(
        here.join("htmlspec").is_dir(),
        "removing a registration removed the project"
    );

    let (code, out, err) = run_with_registry(&here, &config, &["store", "list"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("no stores registered"), "{out}");
}

/// **Archival does not destroy a runbook, driven through the real binary.**
///
/// The regression test for a measured data loss. Before the guard, `htmlspec archive
/// htmlspec/runbooks/release --yes` printed `archived to
/// .../htmlspec/changes/archive/<date>-release at version 0.1.0` and left `htmlspec/runbooks/`
/// empty. `runbook-authority` states that a procedure run repeatedly is not a change and does not
/// archive; nothing checked it, which is the shape `project-bootstrap` names.
///
/// **The full clearing shape is what lets this test fail.** Three unrelated gates refuse before
/// archival moves anything — the version gate, the absent-`specification.html` gate, and the
/// template example-content gate. A runbook missing any of them is refused for a reason that has
/// nothing to do with runbooks, so the test would pass against the defect. This builds both
/// documents, replaces the example identifiers, and advances the version, then asserts the exit
/// is non-zero, the refusal names the runbook, and **the documents are still on disk**.
///
/// Through the binary rather than through the resolver, because the thing being prevented is a
/// filesystem move: an assertion that the files survive has to be made against a real run.
#[test]
fn archival_refuses_a_runbook_and_leaves_it_on_disk() {
    let scratch = Scratch::new("runbook-not-archived");
    let project = a_project(&scratch);
    let runbook = project.join("htmlspec").join("runbooks").join("release");
    std::fs::create_dir_all(&runbook).expect("the runbook directory exists");

    // Both documents, from the real templates through the real command, with the example
    // identifiers replaced so the example-content gate has nothing to say.
    for (kind, file) in [
        ("specification", "specification.html"),
        ("tasks", "tasks.html"),
    ] {
        let path = runbook.join(file);
        let (code, out, err) = run_in(
            &project,
            &[
                "new",
                "document",
                kind,
                &path.to_string_lossy(),
                "--subject",
                "Releasing the project",
            ],
        );
        assert_eq!(code, 0, "new document failed:\n{out}{err}");
        let document = std::fs::read_to_string(&path).expect("the document reads");
        // The example identifiers *and* the masthead placeholder. **Four gates, not three:**
        // `review/example-content` fires on the summary as well as on unit identifiers, and
        // archival refuses on either. Discovered by running this test against the pre-fix code
        // and reading why it failed — it was refused for the placeholder, so it would have
        // passed against the defect for the wrong reason.
        let document = document.replace("-example", "-real").replace(
            "Replace this paragraph with what",
            "This document states what",
        );
        std::fs::write(&path, document).expect("rewritten");
    }
    // And the version gate satisfied, so it is not what refuses.
    advance_version(&project);

    let (code, out, err) = run_in(&project, &["archive", &runbook.to_string_lossy(), "--yes"]);

    assert_ne!(code, 0, "archival accepted a runbook:\n{out}{err}");
    assert!(
        err.contains("is a runbook, not a change"),
        "the refusal does not name what the directory is:\n{out}{err}"
    );
    assert!(
        !out.contains("archived to"),
        "archival moved the runbook:\n{out}"
    );
    // The assertion the whole test exists for. The three gates above would each produce a
    // non-zero exit, so exit status alone does not distinguish a refusal from a near miss.
    for file in ["specification.html", "tasks.html"] {
        assert!(
            runbook.join(file).is_file(),
            "{file} is gone: archival destroyed the runbook it was asked about"
        );
    }
    assert!(
        std::fs::read_dir(project.join("htmlspec").join("changes").join("archive"))
            .expect("the archive is readable")
            .flatten()
            .next()
            .is_none(),
        "something was archived"
    );
}

/// **A `htmlspec/` inside a runbook does not make the runbook archivable.**
///
/// Bypass A, measured on `d8523bb`: the guard resolved its own project from the argument, and
/// `ProjectRoot::resolve` returns the first ancestor holding a child `htmlspec/` — *including the
/// directory itself*. So the runbook resolved as its own project, `runbooks()` became
/// `<runbook>/htmlspec/runbooks`, which does not exist, and the guard read that as "not a runbook".
/// `archived to .../changes/archive/2026-09-18-release`, `runbooks/` empty.
///
/// A nested `htmlspec/` is not contrived: a runbook that documents how to bootstrap a project, or
/// that ships an example project as a fixture, has one.
#[test]
fn a_nested_project_directory_does_not_make_a_runbook_archivable() {
    let scratch = Scratch::new("runbook-nested-project");
    let (project, runbook) = a_project_with_a_complete_runbook(&scratch);
    std::fs::create_dir_all(runbook.join("htmlspec")).expect("the nested directory exists");

    assert_the_runbook_survived(&project, &runbook, "a nested htmlspec/ directory");
}

/// **A `.git` inside a runbook does not make the runbook archivable.**
///
/// Bypass B, measured on `d8523bb`. `ProjectRoot::resolve` stops at a `.git` directory — the
/// repository boundary — so with one inside the runbook it found no project at all, and the guard's
/// `else` arm returned `Ok(())`. Same destruction as bypass A by a different route.
///
/// Also not contrived: a submodule, or a repository a runbook vendors as an example.
#[test]
fn a_nested_repository_does_not_make_a_runbook_archivable() {
    let scratch = Scratch::new("runbook-nested-repository");
    let (project, runbook) = a_project_with_a_complete_runbook(&scratch);
    std::fs::create_dir_all(runbook.join(".git")).expect("the nested repository exists");

    assert_the_runbook_survived(&project, &runbook, "a nested .git directory");
}

/// **An unreadable `runbooks/` refuses rather than proceeding.**
///
/// The guard exists to prevent an irreversible `rename`, so a condition it cannot interpret must
/// not read as permission. A broken symlink is the reachable form: `Path::exists` follows symlinks
/// and reports `false` for a broken one, so the first build of this fix read it as "no runbooks
/// directory" and archived a change. Found by probing rather than by review, which is why the check
/// is now `symlink_metadata`.
///
/// The subject is a **real change**, not a runbook: the point is that an unreadable `runbooks/`
/// stops archival of anything, because the guard can no longer answer its question.
#[test]
fn an_unreadable_runbooks_directory_refuses_rather_than_proceeding() {
    let scratch = Scratch::new("runbook-unreadable");
    let project = a_project(&scratch);
    let change = project.join("htmlspec").join("changes").join("a-change");
    std::fs::create_dir_all(&change).expect("the change directory exists");
    let runbooks = project.join("htmlspec").join("runbooks");
    std::fs::remove_dir_all(&runbooks).expect("the runbooks directory is removed");
    std::os::unix::fs::symlink("/nonexistent-target", &runbooks).expect("the symlink is made");
    advance_version(&project);

    let (code, out, err) = run_in(&project, &["archive", &change.to_string_lossy(), "--yes"]);

    assert_ne!(
        code, 0,
        "archival proceeded past a check it could not make:\n{out}{err}"
    );
    assert!(
        err.contains("Refusing rather than proceeding"),
        "the refusal does not say it is failing closed:\n{out}{err}"
    );
    assert!(
        !out.contains("archived to"),
        "the directory was moved:\n{out}"
    );
}

/// **A name that is a path is refused, so it cannot create a change inside `runbooks/`.**
///
/// Measured on `d8523bb`: `htmlspec new change ../runbooks/sneaky` created a change directory
/// inside the runbooks directory. It then listed as a runbook, was invisible to `htmlspec list`,
/// and every change-taking operation refused it — a directory nothing could act on.
///
/// **The guard is what made it unrecoverable**, and that is worth stating plainly: before the
/// runbook guard existed the same typo produced a directory that was odd and still archivable,
/// verified by building `88759b6^` and archiving one. So the fix for the data loss converted a
/// recoverable mistake into a wedge, and the repair belongs on the constructor rather than here.
#[test]
fn a_change_name_that_is_a_path_is_refused() {
    let scratch = Scratch::new("runbook-wedge");
    let project = a_project(&scratch);

    let (code, out, err) = run_in(&project, &["new", "change", "../runbooks/sneaky"]);

    assert_ne!(code, 0, "a path was accepted as a change name:\n{out}{err}");
    assert!(
        err.contains("is a path rather than a change name"),
        "the refusal does not name the problem:\n{out}{err}"
    );
    assert!(
        !project
            .join("htmlspec")
            .join("runbooks")
            .join("sneaky")
            .exists(),
        "a change directory was created inside the runbooks directory"
    );
    // And the ordinary name still works, so the check refuses paths rather than names.
    let (code, out, err) = run_in(&project, &["new", "change", "a-real-change"]);
    assert_eq!(code, 0, "a plain name was refused:\n{out}{err}");
}

/// A project holding a complete runbook: two documents, real content, version advanced.
///
/// The clearing shape, in one helper. Four unrelated gates refuse before archival moves anything —
/// the version gate, the absent-`specification.html` gate, and both halves of
/// `review/example-content` — so a runbook missing any of them is refused for a reason that has
/// nothing to do with runbooks, and a bypass test would pass against the defect.
fn a_project_with_a_complete_runbook(scratch: &Scratch) -> (PathBuf, PathBuf) {
    let project = a_project(scratch);
    let runbook = project.join("htmlspec").join("runbooks").join("release");
    std::fs::create_dir_all(&runbook).expect("the runbook directory exists");
    for (kind, file) in [
        ("specification", "specification.html"),
        ("tasks", "tasks.html"),
    ] {
        let path = runbook.join(file);
        let (code, out, err) = run_in(
            &project,
            &[
                "new",
                "document",
                kind,
                &path.to_string_lossy(),
                "--subject",
                "Releasing the project",
            ],
        );
        assert_eq!(code, 0, "new document failed:\n{out}{err}");
        let document = std::fs::read_to_string(&path).expect("the document reads");
        let document = document.replace("-example", "-real").replace(
            "Replace this paragraph with what",
            "This document states what",
        );
        std::fs::write(&path, document).expect("rewritten");
    }
    advance_version(&project);
    (project, runbook)
}

/// Asserts archival refused the runbook and left both its documents on disk.
///
/// The surviving files are the assertion that matters. Exit status alone does not distinguish a
/// refusal from a near miss, and the four clearing gates each produce a non-zero exit.
fn assert_the_runbook_survived(project: &Path, runbook: &Path, despite: &str) {
    let (code, out, err) = run_in(project, &["archive", &runbook.to_string_lossy(), "--yes"]);

    assert_ne!(
        code, 0,
        "archival accepted a runbook with {despite}:\n{out}{err}"
    );
    assert!(
        err.contains("is a runbook, not a change"),
        "the refusal does not name what the directory is, with {despite}:\n{out}{err}"
    );
    assert!(
        !out.contains("archived to"),
        "archival moved a runbook with {despite}:\n{out}"
    );
    for file in ["specification.html", "tasks.html"] {
        assert!(
            runbook.join(file).is_file(),
            "{file} is gone: {despite} let archival destroy the runbook"
        );
    }
}

// -----------------------------------------------------------------------------
// Archival publishes a runbook's release, and refuses where the release does not follow.
//
// The subject of every test here is a *file*: an inventory in the archive, a marker at the planning
// root, and the active change still being where it was when archival refused. An exit status alone
// does not distinguish a refusal from a near miss, which is the reason this file drives the binary.
// -----------------------------------------------------------------------------

/// A runbook the project holds, with release metadata and two canonical documents.
///
/// Built through `new document` rather than written by hand, so a template change reaches these tests
/// instead of being shadowed by a sample — the discipline the rest of this file uses.
fn a_runbook(project: &Path, name: &str, version: &str) -> PathBuf {
    let runbook = project.join("htmlspec").join("runbooks").join(name);
    std::fs::create_dir_all(&runbook).expect("the runbook directory exists");
    for (kind, file) in [
        ("specification", "specification.html"),
        ("tasks", "tasks.html"),
    ] {
        let path = runbook.join(file);
        let (code, out, err) = run_in(
            project,
            &[
                "new",
                "document",
                kind,
                &path.to_string_lossy(),
                "--subject",
                "A procedure worth repeating",
            ],
        );
        assert_eq!(code, 0, "new document failed:\n{out}{err}");
        let document = std::fs::read_to_string(&path).expect("the document reads");
        // The example identifiers a template writes are replaced, because `review/example-content`
        // reports a surviving one and archival refuses over it — which would make every test below
        // about the template rather than about releases.
        let document = document
            .replace("-example", "-real")
            .replace(
                "Replace this paragraph with what",
                "This document states what",
            )
            .replace("data-change=\"added\"", "data-change=\"unchanged\"");
        std::fs::write(&path, document).expect("rewritten");
    }
    std::fs::write(runbook.join("VERSION"), format!("{version}\n")).expect("written");
    std::fs::write(
        runbook.join("CHANGELOG.md"),
        format!("# Changelog\n\n## {version}\n\nA release of this procedure.\n"),
    )
    .expect("written");
    runbook
}

/// The published inventory an archive carries, as this program writes it.
fn a_published_inventory(archive: &Path, name: &str, version: &str, fill: &str) {
    let digest = format!(
        "sha256:{}",
        fill.repeat(64).chars().take(64).collect::<String>()
    );
    let inventory = serde_json::json!({
        "format": "htmlspec-runbook-inventory/v1",
        "runbooks": [{
            "name": name,
            "version": version,
            "specification": digest,
            "procedure": digest,
            "changelog": format!("# Changelog\n\n## {version}\n\nA release.\n"),
        }],
    });
    std::fs::write(
        archive.join("runbooks.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&inventory).expect("serializes")
        ),
    )
    .expect("written");
}

/// Archival publishes an inventory beside the change it archives, and adopts publication once.
///
/// **Both files are the assertion, and so is the marker's version.** The marker names the first
/// project version subject to inventories and is never advanced afterwards, so a second archival must
/// leave it alone — a marker that moved would turn recorded history into a legacy exemption.
#[test]
fn archival_publishes_a_runbook_inventory_and_adopts_publication_once() {
    let scratch = Scratch::new("publishes");
    let directory = a_change_ready_to_archive(&scratch, "publishes");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 0, "archival refused:\n{out}{err}");

    let archives = scratch
        .0
        .join("htmlspec/changes/archive")
        .read_dir()
        .expect("the archive exists")
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    assert_eq!(archives.len(), 1, "{archives:?}");
    let inventory = archives[0].join("runbooks.json");
    assert!(
        inventory.is_file(),
        "the archive carries no inventory, so the release it published is not recorded"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&inventory).expect("readable"))
            .expect("the inventory is JSON");
    assert_eq!(
        parsed["format"],
        serde_json::json!("htmlspec-runbook-inventory/v1"),
        "{parsed}"
    );
    assert_eq!(
        parsed["runbooks"][0]["name"],
        serde_json::json!("release"),
        "{parsed}"
    );
    assert_eq!(
        parsed["runbooks"][0]["version"],
        serde_json::json!("0.1.0"),
        "{parsed}"
    );
    // Two digests, and they are of the canonical documents rather than of anything a run wrote.
    for field in ["specification", "procedure"] {
        let digest = parsed["runbooks"][0][field].as_str().expect("a digest");
        assert!(
            digest.starts_with("sha256:") && digest.len() == 71,
            "{field} is not a whole sha256 digest: {digest}"
        );
    }

    // Adoption, recorded at the project version this archival shipped.
    let marker = scratch.0.join("htmlspec/runbook-releases.json");
    assert!(
        marker.is_file(),
        "the first inventory was published without the marker that says when the obligation began"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&marker).expect("readable"))
            .expect("the marker is JSON");
    assert_eq!(parsed["version"], serde_json::json!("0.7.0"), "{parsed}");

    // A second archival publishes an inventory and leaves the marker where it was.
    let second = a_change_ready_to_archive(&scratch, "second");
    advance_the_version(&scratch.0, "0.8.0");
    let (code, out, err) = run_in(&scratch.0, &["archive", &second.to_string_lossy(), "--yes"]);
    assert_eq!(code, 0, "the second archival refused:\n{out}{err}");
    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&marker).expect("readable"))
            .expect("the marker is JSON");
    assert_eq!(
        parsed["version"],
        serde_json::json!("0.7.0"),
        "the adoption marker advanced on a later release, so the beginning of the history moved: \
         {parsed}"
    );
}

/// A runbook whose canonical content changed without its version changing refuses archival.
///
/// The scenario `project-bootstrap` names, and the refusal happens **before** anything moves: the
/// change is still active and no archive holds it.
#[test]
fn a_changed_procedure_without_a_version_increment_refuses_archival() {
    let scratch = Scratch::new("no-bump");
    let directory = a_change_ready_to_archive(&scratch, "no-bump");
    let runbook = a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    // Published at 0.1.0 with different canonical content, which is what makes this a reuse rather
    // than a first release.
    let archive = an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));
    a_published_inventory(&archive, "release", "0.1.0", "0");
    std::fs::write(
        scratch.0.join("htmlspec/runbook-releases.json"),
        "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"0.6.0\"}\n",
    )
    .expect("written");

    // The procedure moves; the version does not.
    let procedure = runbook.join("tasks.html");
    let source = std::fs::read_to_string(&procedure).expect("readable");
    std::fs::write(
        &procedure,
        source.replace("This document states what", "A changed procedure"),
    )
    .expect("rewritten");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 1, "archival published a reused version:\n{out}{err}");
    assert!(
        err.contains("release") && err.contains("0.1.0"),
        "the refusal does not name the runbook and the version: {err}"
    );
    assert!(
        err.contains("2026-01-01-earlier"),
        "the refusal does not name the archive that already published that version: {err}"
    );
    // Nothing moved, and nothing was published.
    assert!(
        directory.is_dir(),
        "a refused archival moved the change out of the active changes"
    );
    assert!(
        !directory.join("runbooks.json").exists(),
        "a refused archival left a staged inventory in the active change"
    );
    assert!(
        !scratch
            .0
            .join("htmlspec/changes/archive")
            .join(
                std::fs::read_dir(scratch.0.join("htmlspec/changes/archive"))
                    .expect("readable")
                    .count()
                    .to_string()
            )
            .exists(),
        "an archive appeared for a refused archival"
    );
    assert_eq!(
        std::fs::read_dir(scratch.0.join("htmlspec/changes/archive"))
            .expect("readable")
            .count(),
        1,
        "a refused archival created a second archive beside the fixture's"
    );

    // Readiness-only archival reports the same refusal and writes nothing, which is what makes it
    // usable as a gate.
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_eq!(code, 1, "--check did not report the refusal:\n{out}{err}");
    assert!(directory.is_dir(), "--check moved something");
}

/// A runbook that advances its version with a heading for it publishes.
#[test]
fn a_greater_version_with_a_heading_publishes() {
    let scratch = Scratch::new("bumped");
    let directory = a_change_ready_to_archive(&scratch, "bumped");
    let runbook = a_runbook(&scratch.0, "release", "0.2.0");
    advance_the_version(&scratch.0, "0.7.0");

    let archive = an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));
    a_published_inventory(&archive, "release", "0.1.0", "0");
    std::fs::write(
        scratch.0.join("htmlspec/runbook-releases.json"),
        "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"0.6.0\"}\n",
    )
    .expect("written");
    std::fs::write(
        runbook.join("CHANGELOG.md"),
        "# Changelog\n\n## 0.2.0\n\nA changed procedure.\n\n## 0.1.0\n\nThe first.\n",
    )
    .expect("written");
    let procedure = runbook.join("tasks.html");
    let source = std::fs::read_to_string(&procedure).expect("readable");
    std::fs::write(
        &procedure,
        source.replace("This document states what", "A changed procedure"),
    )
    .expect("rewritten");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(
        code, 0,
        "a properly bumped version was refused:\n{out}{err}"
    );
    assert!(
        out.contains("1 runbook release"),
        "the report does not say what was published:\n{out}"
    );
}

/// A downgrade is refused even where the bytes are identical.
///
/// The gate is an ordering and not a difference test, and this is the case that separates them: a
/// difference test would accept it, because nothing differs.
#[test]
fn a_runbook_version_that_goes_backwards_refuses_archival() {
    let scratch = Scratch::new("downgrade");
    let directory = a_change_ready_to_archive(&scratch, "downgrade");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let archive = an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));
    // Published at 0.2.0 with content this fixture cannot reproduce byte for byte, which is not the
    // point: a lower version is refused whatever the content is.
    a_published_inventory(&archive, "release", "0.2.0", "0");
    std::fs::write(
        scratch.0.join("htmlspec/runbook-releases.json"),
        "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"0.6.0\"}\n",
    )
    .expect("written");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 1, "a downgrade was published:\n{out}{err}");
    assert!(
        err.contains("ordered before"),
        "the refusal does not say the version went backwards: {err}"
    );
    assert!(directory.is_dir(), "a refused archival moved the change");
}

/// A runbook with no version at all refuses archival, naming what to write.
#[test]
fn a_runbook_with_no_version_refuses_archival() {
    let scratch = Scratch::new("unversioned-runbook");
    let directory = a_change_ready_to_archive(&scratch, "unversioned-runbook");
    let runbook = a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");
    std::fs::remove_file(runbook.join("VERSION")).expect("removed");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_ne!(code, 0, "an unversioned runbook was published:\n{out}{err}");
    assert!(
        err.contains("VERSION"),
        "the refusal does not name the file to write: {err}"
    );
    assert!(directory.is_dir(), "a refused archival moved the change");
}

/// A malformed published snapshot is reported rather than treated as an absent predecessor.
///
/// The failure this prevents is the quiet one: a defective inventory skipped would make the next
/// release look like a first one, and a first release is allowed to declare any version above 0.0.0.
#[test]
fn a_malformed_prior_snapshot_is_reported_and_not_skipped() {
    let scratch = Scratch::new("malformed-snapshot");
    let directory = a_change_ready_to_archive(&scratch, "malformed-snapshot");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let archive = an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));
    std::fs::write(
        archive.join("runbooks.json"),
        "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[{\"name\":\"release\"}]}\n",
    )
    .expect("written");
    std::fs::write(
        scratch.0.join("htmlspec/runbook-releases.json"),
        "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"0.6.0\"}\n",
    )
    .expect("written");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_ne!(code, 0, "a defective snapshot was skipped:\n{out}{err}");
    assert!(
        err.contains("2026-01-01-earlier"),
        "the refusal does not name the archive holding the defective snapshot: {err}"
    );
    assert!(directory.is_dir(), "a refused archival moved the change");
}

/// An archive at or after adoption with no inventory is a missing record, and a legacy one is not.
///
/// Both halves, because the marker is what tells them apart. Without it every archive missing an
/// inventory would be either a fault or exempt, and the wrong one of those is a history nobody can
/// trust.
#[test]
fn a_missing_inventory_after_adoption_is_refused_and_before_it_is_not() {
    // After adoption: refused.
    let scratch = Scratch::new("missing-after");
    let directory = a_change_ready_to_archive(&scratch, "missing-after");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");
    an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));
    std::fs::write(
        scratch.0.join("htmlspec/runbook-releases.json"),
        "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"0.6.0\"}\n",
    )
    .expect("written");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_ne!(
        code, 0,
        "an archive inside the adoption window had no inventory:\n{out}{err}"
    );
    assert!(
        err.contains("2026-01-01-earlier") && err.contains("runbooks.json"),
        "the refusal does not name the archive and the missing file: {err}"
    );

    // Before adoption: old history stays valid without invented records.
    let scratch = Scratch::new("missing-before");
    let directory = a_change_ready_to_archive(&scratch, "missing-before");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");
    an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(
        code, 0,
        "an archive predating adoption was refused for having no inventory:\n{out}{err}"
    );
}

/// An inventory with no adoption marker is refused, and so is a marker naming no archive.
///
/// The pair is what makes the history have a beginning: either half alone leaves a reader unable to
/// say from which release a missing inventory means something is wrong.
#[test]
fn an_inventory_and_its_marker_are_each_refused_without_the_other() {
    let scratch = Scratch::new("inventory-without-marker");
    let directory = a_change_ready_to_archive(&scratch, "inventory-without-marker");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");
    let archive = an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));
    a_published_inventory(&archive, "release", "0.1.0", "0");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_ne!(
        code, 0,
        "an inventory with no marker was accepted:\n{out}{err}"
    );
    assert!(
        err.contains("runbook-releases.json"),
        "the refusal does not name the missing marker: {err}"
    );

    let scratch = Scratch::new("marker-without-archive");
    let directory = a_change_ready_to_archive(&scratch, "marker-without-archive");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");
    an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));
    std::fs::write(
        scratch.0.join("htmlspec/runbook-releases.json"),
        // Names a project version no archive records.
        "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"0.4.0\"}\n",
    )
    .expect("written");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_ne!(
        code, 0,
        "a marker naming no archive was accepted:\n{out}{err}"
    );
    assert!(
        err.contains("0.4.0"),
        "the refusal does not name the version nothing records: {err}"
    );
}

/// A run's record does not move a fingerprint, so adding one owes no version increment.
///
/// `project-bootstrap` states it and the reason is worth holding in a test: a runbook exists to be
/// run, and a gate that fired on every run would be a gate an author learns to bump past without
/// reading.
#[test]
fn adding_a_run_record_does_not_require_a_version_increment() {
    let scratch = Scratch::new("record-only");
    let directory = a_change_ready_to_archive(&scratch, "record-only");
    let runbook = a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    // Publish the runbook as it stands, so the comparison has a predecessor with the same content.
    let archive = an_archived_change(&scratch.0, "2026-01-01-earlier", Some("0.6.0"));
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 0, "the first archival refused:\n{out}{err}");
    let published = std::fs::read_to_string(archive.join("runbooks.json"));
    let _ = published;

    // A run starts, which writes a record and touches neither canonical document.
    let (code, out, err) = run_in(&scratch.0, &["run", "release", "--as", "0-7-0"]);
    assert_eq!(code, 0, "the run did not start:\n{out}{err}");
    assert!(
        runbook.join("runs/0-7-0.html").is_file(),
        "the run left no record, so this test is not about a record"
    );

    let second = a_change_ready_to_archive(&scratch, "second");
    advance_the_version(&scratch.0, "0.8.0");
    let (code, out, err) = run_in(&scratch.0, &["archive", &second.to_string_lossy(), "--yes"]);
    assert_eq!(
        code, 0,
        "a new run record required a version increment:\n{out}{err}"
    );
}

/// The archived directory's own inventory is the one that ships, and the active change keeps none.
#[test]
fn a_staged_inventory_travels_with_the_change_and_is_not_left_behind() {
    let scratch = Scratch::new("staged");
    let directory = a_change_ready_to_archive(&scratch, "staged");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !directory.exists(),
        "the active change is still there after archival"
    );
    let archives = scratch
        .0
        .join("htmlspec/changes/archive")
        .read_dir()
        .expect("the archive exists")
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    assert!(
        archives[0].join("runbooks.json").is_file(),
        "the inventory did not travel with the change"
    );
    // And the runbook itself is untouched: publication records a release, it does not edit one.
    assert!(
        scratch
            .0
            .join("htmlspec/runbooks/release/VERSION")
            .is_file(),
        "the runbook lost its version"
    );
}

/// Advances the project's declared version, which archival requires before it compares anything else.
fn advance_the_version(project: &Path, version: &str) {
    let path = project.join("spec.nix");
    let declaration = std::fs::read_to_string(&path).expect("the declaration reads");
    let advanced = declaration
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("version =") {
                format!("  version = \"{version}\";")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, format!("{advanced}\n")).expect("written");
}

/// Readiness-only archival reports the same refusals as publication, and writes nothing at all.
///
/// `project-bootstrap` requires it, and the reason is that `--check` is what a continuous-integration
/// job runs: a gate that reported readiness and then refused to perform is a gate nobody can plan
/// around. Every refusal the publication path makes is asserted here too, on one fixture, because the
/// two paths share a comparison and the test that pins that is the one that runs both.
#[test]
fn readiness_only_archival_reports_the_same_refusals_and_writes_nothing() {
    // Each case is a way one runbook's release metadata can be wrong, named so a failure says which
    // one it was.
    // Each case is one way a runbook's release metadata can be wrong, named so a failure says which.
    type Break = fn(&Path);
    let cases: Vec<(&str, Break)> = vec![
        ("no-version", |runbook: &Path| {
            std::fs::remove_file(runbook.join("VERSION")).expect("removed");
        }),
        ("no-heading", |runbook: &Path| {
            std::fs::write(
                runbook.join("CHANGELOG.md"),
                "# Changelog\n\n## 0.9.0\n\nAnother.\n",
            )
            .expect("written");
        }),
        ("malformed-version", |runbook: &Path| {
            std::fs::write(runbook.join("VERSION"), "one\n").expect("written");
        }),
    ];
    for (label, break_it) in cases {
        let scratch = Scratch::new(label);
        let directory = a_change_ready_to_archive(&scratch, label);
        let runbook = a_runbook(&scratch.0, "release", "0.1.0");
        advance_the_version(&scratch.0, "0.7.0");
        break_it(&runbook);

        let before = walk(&scratch.0);
        let (check_code, check_out, check_err) = run_in(
            &scratch.0,
            &["archive", &directory.to_string_lossy(), "--check"],
        );
        assert_ne!(check_code, 0, "{label}: --check reported readiness");
        assert_eq!(
            before,
            walk(&scratch.0),
            "{label}: --check wrote something\nstdout:\n{check_out}\nstderr:\n{check_err}"
        );

        let (perform_code, perform_out, perform_err) = run_in(
            &scratch.0,
            &["archive", &directory.to_string_lossy(), "--yes"],
        );
        assert_ne!(perform_code, 0, "{label}: publication proceeded");
        assert_eq!(
            check_code, perform_code,
            "{label}: the two paths disagree about the exit class\n--check:\n{check_out}{check_err}\
             \nperform:\n{perform_out}{perform_err}"
        );
        assert_eq!(
            before,
            walk(&scratch.0),
            "{label}: a refused publication changed the project"
        );
    }
}

/// An interrupted publication is recovered before another one is accepted, and the recovery does not
/// fabricate a release.
///
/// **The journal is the mechanism, and this is the test that makes it a mechanism rather than a
/// claim.** A crash between staging the inventory and moving the change leaves files in an active
/// change directory that belong to an archive. The next archival has to notice them and settle them:
/// completing would publish a release nobody compared, so the recovery removes what the interrupted
/// transaction owned and starts again.
#[test]
fn an_interrupted_publication_is_settled_before_the_next_one() {
    let scratch = Scratch::new("interrupted");
    let directory = a_change_ready_to_archive(&scratch, "interrupted");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    // What a crash after staging and before the move leaves behind: an inventory and a recorded
    // version inside a change that is still active.
    std::fs::write(
        directory.join("runbooks.json"),
        "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[]}\n",
    )
    .expect("written");
    std::fs::write(directory.join("VERSION"), "0.7.0\n").expect("written");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    // The active change is not a published release, and a `VERSION` inside one is not a fact this
    // program wrote: archival records the version in the archive. Refusing is the recovery that
    // cannot go wrong, and the report names both files.
    assert_ne!(
        code, 0,
        "a change holding an interrupted publication was archived as though nothing had happened:\n\
         {out}{err}"
    );
    assert!(
        err.contains("runbooks.json") || err.contains("VERSION"),
        "the refusal does not name what the interrupted publication left behind: {err}"
    );
    assert!(
        directory.is_dir(),
        "the recovery moved a change it could not account for"
    );

    // Clearing the leftovers is all it takes, and the retry publishes normally rather than
    // fabricating a release for the interrupted one.
    std::fs::remove_file(directory.join("runbooks.json")).expect("removed");
    std::fs::remove_file(directory.join("VERSION")).expect("removed");
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 0, "the retry after cleanup refused:\n{out}{err}");
    let archives = scratch
        .0
        .join("htmlspec/changes/archive")
        .read_dir()
        .expect("the archive exists")
        .flatten()
        .count();
    assert_eq!(archives, 1, "the retry published two archives, or none");
}

/// Every file beneath `root` with its length, for a before-and-after comparison.
fn walk(root: &Path) -> Vec<(PathBuf, u64)> {
    let mut found = Vec::new();
    collect_files(root, &mut found);
    found.sort();
    found
}

fn collect_files(directory: &Path, into: &mut Vec<(PathBuf, u64)>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, into);
        } else {
            let length = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            into.push((path, length));
        }
    }
}

// -----------------------------------------------------------------------------
// An interrupted publication.
//
// The journal is what makes recovery possible rather than a guess, and these tests drive the two
// sides of the commit point: an interruption before the move rolls back, and one after it completes.
// A third case — a file the transaction owned that somebody has since edited — is refused rather than
// settled either way, because neither answer would be right.
// -----------------------------------------------------------------------------

/// Writes the journal an interrupted archival would have left, naming `writes` as its own.
fn a_journal(project: &Path, change: &Path, target: &Path, writes: &[(&Path, &str)]) {
    let entries: Vec<serde_json::Value> = writes
        .iter()
        .map(|(path, document)| {
            serde_json::json!({
                "path": path,
                "digest": format!("sha256:{}", sha256(document.as_bytes())),
            })
        })
        .collect();
    let journal = serde_json::json!({
        "format": "htmlspec-archive-journal/v1",
        "change": change,
        "target": target,
        "writes": entries,
    });
    std::fs::write(
        project.join("htmlspec/.archive-journal.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&journal).expect("serializes")
        ),
    )
    .expect("the journal is written");
}

/// A sha256 digest in hexadecimal, as the program writes one into its journal.
///
/// **Computed in the test rather than by shelling out**, and the sandbox is why: the build this suite
/// runs in has no `shasum` and no `sha256sum`, so a helper that reached for one produced an empty
/// digest, every journal entry mismatched, and three tests passed locally by asserting about a
/// recovery that had silently never recognised its own files. The digest algorithm is not what these
/// tests are about — whether recovery can tell an intended file from an edited one is — so the value
/// is computed the only way that is available everywhere.
fn sha256(bytes: &[u8]) -> String {
    // sha2 is already a workspace dependency, used by the asset store for the same question.
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// An interruption before the move rolls back: the change stays active and the staged files go.
#[test]
fn an_interruption_before_the_move_rolls_back() {
    let scratch = Scratch::new("rollback");
    let directory = a_change_ready_to_archive(&scratch, "rollback");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    // What a crash between staging and moving leaves: both files in an active change, and a journal
    // naming them.
    let inventory = directory.join("runbooks.json");
    let recorded = directory.join("VERSION");
    std::fs::write(&inventory, "{\"runbooks\":[]}\n").expect("written");
    std::fs::write(&recorded, "0.7.0\n").expect("written");
    let target = scratch
        .0
        .join("htmlspec/changes/archive/2026-01-01-rollback");
    a_journal(
        &scratch.0,
        &directory,
        &target,
        &[(&inventory, "{\"runbooks\":[]}\n"), (&recorded, "0.7.0\n")],
    );

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_eq!(
        code, 0,
        "recovery refused a transaction it could have rolled back:\n{out}{err}"
    );
    assert!(
        err.contains("rolled back"),
        "the recovery did not say what it did:\n{err}"
    );
    assert!(
        directory.is_dir(),
        "the rollback moved a change that was never archived"
    );
    assert!(
        !inventory.exists() && !recorded.exists(),
        "the rollback left staged files in an active change, where nothing else puts them"
    );
    assert!(
        !scratch.0.join("htmlspec/.archive-journal.json").exists(),
        "the journal outlived the transaction it described"
    );
    assert!(
        !target.exists(),
        "the rollback created the archive the transaction never reached"
    );
}

/// An interruption after the move completes rather than undoing an archive that exists.
#[test]
fn an_interruption_after_the_move_completes() {
    let scratch = Scratch::new("complete");
    let directory = a_change_ready_to_archive(&scratch, "complete");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    // The change is already archived, holding the files the transaction staged — and the active
    // change directory is gone, because the move that carried them is the commit point this case sits
    // past. Leaving both in place is a different state, and `a_transaction_that_cannot_say_where_
    // the_change_is_refuses` is the test for it.
    let target = scratch
        .0
        .join("htmlspec/changes/archive/2026-01-01-complete");
    std::fs::create_dir_all(&target).expect("the archive exists");
    std::fs::rename(&directory, target.join("moved")).expect("the change moved");
    let inventory = target.join("runbooks.json");
    let recorded = target.join("VERSION");
    let contents = "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[]}\n";
    std::fs::write(&inventory, contents).expect("written");
    std::fs::write(&recorded, "0.7.0\n").expect("written");
    a_journal(
        &scratch.0,
        &directory,
        &target,
        &[(inventory.as_path(), contents), (&recorded, "0.7.0\n")],
    );
    // The marker was published too, and it has to be: an inventory at the planning root with no
    // marker beside it is a history with no beginning, which the next archival refuses on its own
    // terms and would confound what this test is about.
    std::fs::write(
        scratch.0.join("htmlspec/runbook-releases.json"),
        "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"0.7.0\"}\n",
    )
    .expect("written");

    // Another change to archive, so the operation has something to do after recovery. Recovery is
    // what is under test, and it runs before archival is accepted.
    let next = a_change_ready_to_archive(&scratch, "next");
    advance_the_version(&scratch.0, "0.8.0");
    let (code, out, err) = run_in(&scratch.0, &["archive", &next.to_string_lossy(), "--check"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        err.contains("recovered"),
        "the recovery did not say what it found:\n{err}"
    );
    assert!(
        inventory.is_file() && recorded.is_file(),
        "completing the transaction removed what it had published"
    );
    assert!(
        !scratch.0.join("htmlspec/.archive-journal.json").exists(),
        "the journal outlived the transaction"
    );
}

/// Re-running the interrupted archival itself settles it, although its change has already moved.
///
/// Before this held, archival looked the change up before recovering, so the re-run a contributor
/// reaches for first — the same command, by name, or `--check` — failed with "no change named …"
/// and recovery never ran.
#[test]
fn re_running_the_interrupted_archival_settles_it() {
    for arguments in [
        &["archive", "rerun"][..],
        &["archive", "rerun", "--check"][..],
        &["archive", "--check"][..],
    ] {
        let scratch = Scratch::new("rerun");
        let directory = a_change_ready_to_archive(&scratch, "rerun");
        a_runbook(&scratch.0, "release", "0.1.0");
        advance_the_version(&scratch.0, "0.7.0");

        let target = scratch.0.join("htmlspec/changes/archive/2026-01-01-rerun");
        std::fs::create_dir_all(target.parent().expect("has a parent")).expect("archive exists");
        std::fs::rename(&directory, &target).expect("the change moved");
        let recorded = target.join("VERSION");
        std::fs::write(&recorded, "0.7.0\n").expect("written");
        a_journal(&scratch.0, &directory, &target, &[(&recorded, "0.7.0\n")]);

        let (code, out, err) = run_in(&scratch.0, arguments);
        assert_eq!(
            code, 0,
            "{arguments:?} did not settle the archival it re-ran:\n{out}{err}"
        );
        assert!(
            err.contains("recovered"),
            "{arguments:?} did not say what it settled:\n{err}"
        );
        assert!(
            !scratch.0.join("htmlspec/.archive-journal.json").exists(),
            "{arguments:?} left the journal behind"
        );
        assert!(
            target.is_dir() && recorded.is_file(),
            "{arguments:?} disturbed the archive it completed"
        );
    }
}

/// A file the transaction owned, edited since, is refused rather than settled either way.
///
/// Completing would publish content nobody compared; rolling back would destroy somebody's work. The
/// report has to say which file and why neither answer is available.
#[test]
fn a_conflicting_edit_to_an_owned_file_is_refused() {
    let scratch = Scratch::new("conflict");
    let directory = a_change_ready_to_archive(&scratch, "conflict");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let recorded = directory.join("VERSION");
    std::fs::write(&recorded, "0.7.0\n").expect("written");
    let target = scratch
        .0
        .join("htmlspec/changes/archive/2026-01-01-conflict");
    // The journal names a digest the file no longer has.
    a_journal(&scratch.0, &directory, &target, &[(&recorded, "0.7.0\n")]);
    std::fs::write(&recorded, "9.9.9\n").expect("edited since");

    for arguments in [
        vec!["archive", &*directory.to_string_lossy(), "--check"],
        vec!["archive", &*directory.to_string_lossy(), "--yes"],
    ] {
        let (code, out, err) = run_in(&scratch.0, &arguments);
        assert_eq!(
            code, 2,
            "{arguments:?} settled a transaction whose owned file had been edited:\n{out}{err}"
        );
        assert!(
            err.contains("does not") && err.contains("hold what the transaction intended"),
            "{arguments:?} does not say why it will not choose:\n{err}"
        );
        assert!(
            std::fs::read_to_string(&recorded).expect("readable") == "9.9.9\n",
            "{arguments:?} overwrote somebody's edit"
        );
        assert!(
            scratch.0.join("htmlspec/.archive-journal.json").exists(),
            "{arguments:?} cleared a journal it could not act on, so the state is now unaccounted for"
        );
    }
}

/// A journal that cannot be read is reported as the journal, and left where it is.
///
/// Before this held, an unparseable journal was treated as no journal at all: its staged files
/// stayed in the active change, and the next refusal blamed them instead of the journal.
#[test]
fn an_unreadable_journal_is_reported_rather_than_ignored() {
    let scratch = Scratch::new("unreadable-journal");
    let directory = a_change_ready_to_archive(&scratch, "unreadable-journal");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");
    let journal = scratch.0.join("htmlspec/.archive-journal.json");

    for (label, contents) in [
        (
            "truncated",
            String::from("{\"format\": \"htmlspec-archive-"),
        ),
        (
            "relative",
            serde_json::json!({
                "format": "htmlspec-archive-journal/v1",
                "change": "htmlspec/changes/unreadable-journal",
                "target": scratch.0.join("htmlspec/changes/archive/2026-01-01-unreadable-journal"),
                "writes": [],
            })
            .to_string(),
        ),
    ] {
        std::fs::write(&journal, &contents).expect("written");
        let (code, out, err) = run_in(
            &scratch.0,
            &["archive", &directory.to_string_lossy(), "--check"],
        );
        assert_eq!(
            code, 2,
            "{label}: an unreadable journal was ignored:\n{out}{err}"
        );
        assert!(
            err.contains(".archive-journal.json") && err.contains("cannot be acted on"),
            "{label}: the report does not name the journal:\n{err}"
        );
        assert_eq!(
            std::fs::read_to_string(&journal).expect("the journal is still there"),
            contents,
            "{label}: a journal nothing could read was changed or removed"
        );
        assert!(directory.is_dir(), "{label}: the change moved");
    }
}

/// A change named by a relative path archives, and the journal it wrote never outlives it.
///
/// Recovery reads the journal from wherever the next command runs, so the paths in it are absolute
/// whatever the change was called on the command line.
#[test]
fn a_change_named_by_a_relative_path_archives() {
    let scratch = Scratch::new("relative-change");
    a_change_ready_to_archive(&scratch, "relative-change");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", "htmlspec/changes/relative-change", "--yes"],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !scratch.0.join("htmlspec/changes/relative-change").exists(),
        "the change did not move"
    );
    assert!(
        !scratch.0.join("htmlspec/.archive-journal.json").exists(),
        "the journal outlived the transaction"
    );
}

/// Two directories both holding the change is a state nothing can interpret, and is refused as such.
#[test]
fn a_transaction_that_cannot_say_where_the_change_is_refuses() {
    let scratch = Scratch::new("ambiguous");
    let directory = a_change_ready_to_archive(&scratch, "ambiguous");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let target = scratch
        .0
        .join("htmlspec/changes/archive/2026-01-01-ambiguous");
    std::fs::create_dir_all(&target).expect("the archive exists");
    a_journal(&scratch.0, &directory, &target, &[]);

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains("both") && err.contains("will not choose"),
        "the report does not say why it stopped:\n{err}"
    );
    assert!(
        err.contains("look at both"),
        "the report does not say what to do about it:\n{err}"
    );
    assert!(
        directory.is_dir() && target.is_dir(),
        "the refusal moved or removed one of the two"
    );
}

// -----------------------------------------------------------------------------
// The documents archival rewrites.
//
// Archival re-expresses the references in a change's documents, and does it inside the transaction
// the journal describes: each rewritten document is staged in the change as `.<name>.archiving` and
// takes its document's place only after the move. These tests drive an interruption on each side of
// the move, and the refusals that keep a staged document nobody compared from being published.
// -----------------------------------------------------------------------------

/// Writes the journal an interrupted archival would have left, naming `writes` as its own and
/// `replacements` as the documents it rewrites, each with its rewritten bytes.
fn a_journal_replacing(
    project: &Path,
    change: &Path,
    target: &Path,
    writes: &[(&Path, &str)],
    replacements: &[(&str, &str)],
) {
    let entries: Vec<serde_json::Value> = writes
        .iter()
        .map(|(path, document)| {
            serde_json::json!({
                "path": path,
                "digest": format!("sha256:{}", sha256(document.as_bytes())),
            })
        })
        .collect();
    let replaced: Vec<serde_json::Value> = replacements
        .iter()
        .map(|(document, rewritten)| {
            serde_json::json!({
                "document": document,
                "digest": format!("sha256:{}", sha256(rewritten.as_bytes())),
            })
        })
        .collect();
    let journal = serde_json::json!({
        "format": "htmlspec-archive-journal/v2",
        "change": change,
        "target": target,
        "writes": entries,
        "replacements": replaced,
    });
    std::fs::write(
        project.join("htmlspec/.archive-journal.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&journal).expect("serializes")
        ),
    )
    .expect("the journal is written");
}

/// A document's bytes as archival rewrites them for the archive.
fn climbed(document: &str) -> String {
    let rewritten = document.replace("\"../../assets/", "\"../../../assets/");
    assert_ne!(rewritten, document, "the document references no store");
    rewritten
}

/// An interruption before the move removes the staged documents, and leaves each document as it was.
#[test]
fn an_interruption_before_the_move_removes_the_staged_documents() {
    let scratch = Scratch::new("rollback-documents");
    let directory = a_change_ready_to_archive(&scratch, "rollback-documents");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let proposal = directory.join("proposal.html");
    let original = std::fs::read_to_string(&proposal).expect("readable");
    let staged = directory.join(".proposal.html.archiving");
    std::fs::write(&staged, climbed(&original)).expect("written");
    let recorded = directory.join("VERSION");
    std::fs::write(&recorded, "0.7.0\n").expect("written");
    let target = scratch
        .0
        .join("htmlspec/changes/archive/2026-01-01-rollback-documents");
    a_journal_replacing(
        &scratch.0,
        &directory,
        &target,
        &[(&recorded, "0.7.0\n")],
        &[("proposal.html", &climbed(&original))],
    );

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_eq!(
        code, 0,
        "recovery refused a rollback it could make:\n{out}{err}"
    );
    assert!(err.contains("rolled back"), "{err}");
    assert!(
        !staged.exists() && !recorded.exists(),
        "the rollback left staged files in an active change"
    );
    assert_eq!(
        std::fs::read_to_string(&proposal).expect("readable"),
        original,
        "the rollback changed a document the transaction had not yet replaced"
    );
    assert!(
        !scratch.0.join("htmlspec/.archive-journal.json").exists(),
        "the journal outlived the transaction"
    );
}

/// An interruption after the move puts each staged document in place, including after one was
/// already replaced.
#[test]
fn an_interruption_after_the_move_puts_the_staged_documents_in_place() {
    let scratch = Scratch::new("complete-documents");
    let directory = a_change_ready_to_archive(&scratch, "complete-documents");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");
    let proposal = std::fs::read_to_string(directory.join("proposal.html")).expect("readable");
    let design = std::fs::read_to_string(directory.join("design.html")).expect("readable");

    // Past the commit point, part-way through the replacements: the design is in place and the
    // proposal is still staged.
    let target = scratch
        .0
        .join("htmlspec/changes/archive/2026-01-01-complete-documents");
    std::fs::create_dir_all(target.parent().expect("the archive")).expect("created");
    std::fs::rename(&directory, &target).expect("the change moved");
    std::fs::write(target.join(".proposal.html.archiving"), climbed(&proposal)).expect("written");
    std::fs::write(target.join("design.html"), climbed(&design)).expect("written");
    std::fs::write(target.join("VERSION"), "0.7.0\n").expect("written");
    a_journal_replacing(
        &scratch.0,
        &directory,
        &target,
        &[(&directory.join("VERSION"), "0.7.0\n")],
        &[
            ("design.html", &climbed(&design)),
            ("proposal.html", &climbed(&proposal)),
        ],
    );

    let (code, out, err) = run_in(&scratch.0, &["archive", "--check"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(err.contains("recovered"), "{err}");
    assert_eq!(
        std::fs::read_to_string(target.join("proposal.html")).expect("readable"),
        climbed(&proposal),
        "completing the transaction left the proposal pointing one level short"
    );
    assert_eq!(
        std::fs::read_to_string(target.join("design.html")).expect("readable"),
        climbed(&design),
        "completing the transaction disturbed a document already in place"
    );
    assert!(
        !target.join(".proposal.html.archiving").exists(),
        "completing the transaction left a staged document in the archive"
    );
    assert!(
        !scratch.0.join("htmlspec/.archive-journal.json").exists(),
        "the journal outlived the transaction"
    );
}

/// An archival killed while it wrote a staged document is rolled back, and the next one proceeds.
///
/// The code review of pull request #58 found that a staged copy was written by opening it and then
/// writing it, so a run killed part-way left a copy holding part of the rewrite. Its digest is not
/// the journal's, and recovery refused it as somebody's edit, so the contributor had to delete a file
/// archival owns, on the word of a report that misdescribed it. Before the move, a staged copy is a
/// file only archival writes, derived from a document still in place, and removing it loses nothing.
#[test]
fn an_interruption_while_a_document_was_staged_rolls_back() {
    let scratch = Scratch::new("torn-staged");
    let directory = a_change_ready_to_archive(&scratch, "torn-staged");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let proposal = directory.join("proposal.html");
    let original = std::fs::read_to_string(&proposal).expect("readable");
    let rewrite = climbed(&original);
    let staged = directory.join(".proposal.html.archiving");
    let torn = rewrite
        .as_bytes()
        .get(..rewrite.len() / 2)
        .expect("half the rewrite");
    std::fs::write(&staged, torn).expect("written");
    let recorded = directory.join("VERSION");
    std::fs::write(&recorded, "0.7.0\n").expect("written");
    let target = scratch
        .0
        .join("htmlspec/changes/archive/2026-01-01-torn-staged");
    a_journal_replacing(
        &scratch.0,
        &directory,
        &target,
        &[(&recorded, "0.7.0\n")],
        &[("proposal.html", &rewrite)],
    );

    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--check"],
    );
    assert_eq!(
        code, 0,
        "recovery refused a staged copy archival was writing when it stopped:\n{out}{err}"
    );
    assert!(err.contains("rolled back"), "{err}");
    assert!(
        !staged.exists() && !recorded.exists(),
        "the rollback left staged files in an active change"
    );
    assert_eq!(
        std::fs::read_to_string(&proposal).expect("readable"),
        original,
        "the rollback changed the document"
    );
    assert!(!scratch.0.join("htmlspec/.archive-journal.json").exists());

    // Nothing was left for the contributor to remove by hand: the archival runs.
    let (code, out, err) = run_in(
        &scratch.0,
        &["archive", &directory.to_string_lossy(), "--yes"],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(
        std::fs::read_to_string(the_only_archived_change(&scratch.0).join("proposal.html"))
            .expect("readable"),
        rewrite
    );
}

/// After the move, a staged document edited since it was staged is refused rather than published or
/// removed.
///
/// Past the commit point the staged copy is about to take its document's place, so whether it holds
/// what the transaction intended decides what the archive publishes. Before the move nothing is
/// published, and the copy is removed whatever it holds
/// (`an_interruption_while_a_document_was_staged_rolls_back`).
#[test]
fn an_edited_staged_document_is_refused_after_the_move() {
    let scratch = Scratch::new("edited-staged");
    let directory = a_change_ready_to_archive(&scratch, "edited-staged");
    a_runbook(&scratch.0, "release", "0.1.0");
    advance_the_version(&scratch.0, "0.7.0");

    let original = std::fs::read_to_string(directory.join("proposal.html")).expect("readable");
    let target = scratch
        .0
        .join("htmlspec/changes/archive/2026-01-01-edited-staged");
    std::fs::create_dir_all(target.parent().expect("the archive")).expect("created");
    std::fs::rename(&directory, &target).expect("the change moved");
    let staged = target.join(".proposal.html.archiving");
    std::fs::write(&staged, "somebody's edit\n").expect("written");
    a_journal_replacing(
        &scratch.0,
        &directory,
        &target,
        &[],
        &[("proposal.html", &climbed(&original))],
    );

    let (code, out, err) = run_in(&scratch.0, &["archive", "--check"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains(".proposal.html.archiving")
            && err.contains("hold what the transaction intended"),
        "the report does not name the staged document and why it will not choose:\n{err}"
    );
    assert_eq!(
        std::fs::read_to_string(&staged).expect("readable"),
        "somebody's edit\n",
        "the refusal overwrote or removed somebody's edit"
    );
    assert_eq!(
        std::fs::read_to_string(target.join("proposal.html")).expect("readable"),
        original,
        "the refusal published the edited copy"
    );
    assert!(scratch.0.join("htmlspec/.archive-journal.json").exists());
}

/// A change that already holds a staged document is refused, as one holding a staged inventory is.
///
/// Both a document this archival rewrites and one it does not: the specification here carries no
/// relative reference, so nothing would be staged over its leftover, and without the refusal it
/// would be carried into the archive.
#[test]
fn a_leftover_staged_document_is_refused() {
    for document in ["design.html", "specification.html"] {
        let scratch = Scratch::new("leftover-staged");
        let directory = a_change_ready_to_archive(&scratch, "leftover-staged");
        let leftover = format!(".{document}.archiving");
        std::fs::write(directory.join(&leftover), "left behind\n").expect("written");
        advance_version(&scratch.0);
        let before = tree(&scratch.0);

        let (code, out, err) = run_in(
            &scratch.0,
            &["archive", &directory.to_string_lossy(), "--yes"],
        );
        assert_eq!(code, 2, "{document}: {out}{err}");
        assert!(
            err.contains(&leftover) && err.contains("interrupted archival"),
            "{document}: the refusal does not name the staged document and how it got there:\n{err}"
        );
        assert_eq!(
            tree(&scratch.0),
            before,
            "{document}: the refusal modified the project"
        );
    }
}
