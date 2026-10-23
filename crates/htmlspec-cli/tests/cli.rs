//! Task 7.8: the CLI's contract, verified by running the real binary.
//!
//! The unit tests in `commands.rs` drive the command functions directly, which is
//! faster and covers more cases. This file exists for what those cannot reach: the
//! actual process exit code, the actual stream separation, and the actual argument
//! parsing. A command function that returned the right [`Outcome`] while `main`
//! exited with the wrong code would pass every unit test in the crate.
//!
//! Each exit-code path is covered once, and the structured-output test asserts both
//! halves of the requirement together: stdout parses cleanly *and* progress is
//! present on stderr. Asserting either alone would miss the failure mode, which is
//! progress leaking into the payload.

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The exit codes the program contracts to.
///
/// Restated here rather than imported, deliberately. These are a public contract that
/// other programs depend on, and a test that read them from the source could not
/// notice them changing. A failure here means the contract moved.
const SUCCESS: i32 = 0;
const DOCUMENT_VIOLATION: i32 = 1;
const OPERATIONAL_FAILURE: i32 = 2;

/// The binary under test, as Cargo built it.
fn binary() -> PathBuf {
    // `CARGO_BIN_EXE_<name>` is set by Cargo for an integration test of a crate with
    // a binary target, and it points at the binary this test was built alongside.
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// Runs the binary and returns its exit code, stdout, and stderr.
fn run(args: &[&str]) -> (i32, String, String) {
    let output: Output = Command::new(binary())
        .args(args)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    support::held(args, &stdout);
    (
        output.status.code().expect("the process exited normally"),
        stdout,
        String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    )
}

/// Runs the binary from inside `directory`.
///
/// `init` and `new` resolve their project by walking upward from the working directory,
/// so a test driving them says where it stands. Setting the child's directory rather
/// than the test process's keeps these safe to run in parallel.
fn run_in(directory: &Path, args: &[&str]) -> (i32, String, String) {
    let output: Output = Command::new(binary())
        .current_dir(directory)
        .args(args)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    support::held(args, &stdout);
    (
        output.status.code().expect("the process exited normally"),
        stdout,
        String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    )
}

/// A temporary directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-cli-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.0.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("the parent exists");
        }
        std::fs::write(&path, contents).expect("the file is written");
        path
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A document that validates.
///
/// The `specification` shape rather than `init`'s default, because [`invalid`]
/// breaks a reference and a proposal carries none: `init` defaults to a proposal,
/// which is prose with no unit scaffolding at all.
fn valid() -> String {
    htmlspec_assets::render_kind("specification", "A subject", "assets")
        .expect("the specification template exists")
}

/// A document with a dangling reference: a violation, not an operational failure.
fn invalid() -> String {
    valid().replace(r#"data-refs="req-example""#, r#"data-refs="req-absent""#)
}

fn as_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

// -----------------------------------------------------------------------------
// Exit codes
// -----------------------------------------------------------------------------

#[test]
fn clean_validation_exits_zero() {
    let scratch = Scratch::new("exit-clean");
    let path = scratch.write("doc.html", &valid());
    let (code, _, _) = run(&["validate", &as_str(&path)]);
    assert_eq!(code, SUCCESS);
}

#[test]
fn an_invalid_document_exits_with_the_violation_code() {
    let scratch = Scratch::new("exit-invalid");
    let path = scratch.write("doc.html", &invalid());
    let (code, out, _) = run(&["validate", &as_str(&path)]);
    assert_eq!(code, DOCUMENT_VIOLATION);
    assert!(out.contains("refs/unresolved"), "{out}");
}

/// The distinction the specs specifically require: an unreadable file exits with a
/// code distinct from a document violation.
#[test]
fn a_missing_file_exits_with_the_operational_failure_code() {
    let (code, out, err) = run(&["validate", "/nonexistent/htmlspec/doc.html"]);
    assert_eq!(code, OPERATIONAL_FAILURE);
    assert_ne!(code, DOCUMENT_VIOLATION);
    assert!(out.is_empty(), "a failure reached stdout: {out}");
    assert!(err.contains("no such file or directory"), "{err}");
}

/// An invalid invocation is an operational failure, not a document violation:
/// nothing was determined about any document.
#[test]
fn an_unknown_subcommand_exits_with_the_operational_failure_code() {
    let (code, out, err) = run(&["lint"]);
    assert_eq!(code, OPERATIONAL_FAILURE);
    assert!(out.is_empty(), "{out}");
    // The error lists what is available, so a contributor is not left guessing.
    for subcommand in ["validate", "format", "init"] {
        assert!(err.contains(subcommand), "{err} omits {subcommand}");
    }
}

/// A document too ambiguous to format is refused, which is operational rather than a
/// violation: the formatter could not run, so it reports nothing about the document's
/// conformance.
#[test]
fn an_ambiguous_document_exits_with_the_operational_failure_code() {
    let scratch = Scratch::new("exit-ambiguous");
    let ambiguous = "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
         <body><p><strong>text</p></strong></body></html>\n";
    let path = scratch.write("doc.html", ambiguous);

    let (code, _, err) = run(&["format", &as_str(&path)]);
    assert_eq!(code, OPERATIONAL_FAILURE);
    assert!(err.contains("refusing to format"), "{err}");
    assert_eq!(
        std::fs::read_to_string(&path).expect("readable"),
        ambiguous,
        "the file was modified despite the refusal"
    );
}

/// A continuous-integration job that validates the project fails when any document
/// is invalid, which is the whole point of the exit code.
#[test]
fn a_project_run_fails_when_any_document_is_invalid() {
    let scratch = Scratch::new("exit-project");
    scratch.write("good.html", &valid());
    scratch.write("bad.html", &invalid());
    let (code, _, _) = run(&["validate", &as_str(&scratch.0)]);
    assert_eq!(code, DOCUMENT_VIOLATION);
}

// -----------------------------------------------------------------------------
// Output routing
// -----------------------------------------------------------------------------

/// The requirement, both halves at once: standard output parses cleanly *and*
/// progress output is present. Either alone would miss progress leaking into the
/// payload.
#[test]
fn structured_stdout_parses_cleanly_with_progress_present() {
    let scratch = Scratch::new("routing-structured");
    scratch.write("good.html", &valid());
    scratch.write("bad.html", &invalid());

    let (code, out, err) = run(&["validate", "--json", &as_str(&scratch.0)]);
    assert_eq!(code, DOCUMENT_VIOLATION);

    // Nothing but the payload on stdout.
    let parsed: serde_json::Value =
        serde_json::from_str(&out).unwrap_or_else(|e| panic!("stdout does not parse: {e}\n{out}"));
    assert_eq!(parsed["accepted"], serde_json::json!(false));
    assert_eq!(parsed["documents"], serde_json::json!(2));

    let results = parsed["results"].as_array().expect("results is an array");
    assert_eq!(results.len(), 2);
    // Each entry carries the stable rule identifier and a location, which is what an
    // agent acts on.
    let diagnostics: Vec<&serde_json::Value> = results
        .iter()
        .filter_map(|r| r["report"]["diagnostics"].as_array())
        .flatten()
        .collect();
    assert!(
        diagnostics.iter().any(|d| d["rule"] == "refs/unresolved"),
        "no stable rule identifier in the payload: {out}"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d["unitId"].is_string() && d["severity"].is_string()),
        "a diagnostic carries no location or severity: {out}"
    );

    // And progress is present, on standard error.
    assert!(err.contains("validating"), "no progress output: {err}");
    assert!(
        err.contains("good.html") && err.contains("bad.html"),
        "{err}"
    );
}

/// Human-readable output is the default and goes to standard output.
#[test]
fn the_default_output_is_human_readable_on_standard_output() {
    let scratch = Scratch::new("routing-human");
    let path = scratch.write("doc.html", &invalid());
    let (_, out, _) = run(&["validate", &as_str(&path)]);
    assert!(
        serde_json::from_str::<serde_json::Value>(&out).is_err(),
        "the default output is JSON: {out}"
    );
    assert!(out.contains("error:"), "{out}");
    assert!(out.contains("validation failed"), "{out}");
}

/// An operational failure in structured mode leaves stdout empty rather than
/// half-written, so a parser sees nothing rather than malformed input.
#[test]
fn an_operational_failure_leaves_structured_stdout_empty() {
    let (code, out, err) = run(&["validate", "--json", "/nonexistent/htmlspec/doc.html"]);
    assert_eq!(code, OPERATIONAL_FAILURE);
    assert!(out.is_empty(), "stdout was not empty: {out}");
    assert!(!err.is_empty());
}

// -----------------------------------------------------------------------------
// Format
// -----------------------------------------------------------------------------

#[test]
fn check_mode_exits_nonzero_and_writes_nothing() {
    let scratch = Scratch::new("format-check");
    let unformatted = "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>T</title></head><body><p>Body.</p></body></html>\n";
    let path = scratch.write("doc.html", unformatted);

    let (code, out, _) = run(&["format", "--check", &as_str(&path)]);
    assert_eq!(code, DOCUMENT_VIOLATION);
    assert!(out.contains("not canonically formatted"), "{out}");
    assert!(out.contains("doc.html"), "the document is not named: {out}");
    assert_eq!(
        std::fs::read_to_string(&path).expect("readable"),
        unformatted,
        "check mode modified the file"
    );
}

#[test]
fn write_mode_formats_and_is_then_idempotent() {
    let scratch = Scratch::new("format-write");
    let path = scratch.write(
        "doc.html",
        "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>T</title></head><body><p>Body.</p></body></html>\n",
    );

    let (code, _, _) = run(&["format", &as_str(&path)]);
    assert_eq!(code, SUCCESS);
    let once = std::fs::read_to_string(&path).expect("readable");
    assert!(
        once.starts_with("<!DOCTYPE html>\n<html lang=\"en\">\n  <head>"),
        "{once}"
    );

    // A second run changes nothing, and check mode now passes.
    let (code, _, _) = run(&["format", &as_str(&path)]);
    assert_eq!(code, SUCCESS);
    assert_eq!(std::fs::read_to_string(&path).expect("readable"), once);
    assert_eq!(run(&["format", "--check", &as_str(&path)]).0, SUCCESS);
}

// -----------------------------------------------------------------------------
// The fallback stylesheet
// -----------------------------------------------------------------------------

/// A project holding documents written before the fallback existed, at `paths` beneath it.
///
/// Each is created by `new document` and then has its fallback taken out, which is exactly the
/// document an earlier release wrote: the fallback is the only line the template gained.
fn a_project_with_documents_lacking_the_fallback(scratch: &Scratch, paths: &[&str]) {
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "init failed:\n{out}{err}");
    for relative in paths {
        let kind = if relative.ends_with("tasks.html") || relative.contains("/runs/") {
            "tasks"
        } else {
            "specification"
        };
        let (code, out, err) = run_in(
            &scratch.0,
            &["new", "document", kind, relative, "--subject", "Earlier"],
        );
        assert_eq!(code, SUCCESS, "new document failed:\n{out}{err}");
        let path = scratch.path(relative);
        let written = std::fs::read_to_string(&path).expect("readable");
        let without = written.replace(&format!("    {}\n", htmlspec_assets::fallback::ELEMENT), "");
        assert_ne!(
            without, written,
            "the new document carried no fallback to take out"
        );
        std::fs::write(&path, without).expect("rewritten");
    }
}

fn carries_the_fallback(scratch: &Scratch, relative: &str) -> usize {
    std::fs::read_to_string(scratch.path(relative))
        .expect("readable")
        .matches(htmlspec_assets::fallback::ELEMENT)
        .count()
}

/// Plain formatting never adds the fallback, so upgrading changes no document a contributor did
/// not ask to change, and no runbook's fingerprint.
#[test]
fn plain_format_leaves_a_document_without_the_fallback_without_one() {
    let scratch = Scratch::new("fallback-plain");
    a_project_with_documents_lacking_the_fallback(&scratch, &["htmlspec/specs/earlier.html"]);

    let (code, out, err) = run_in(&scratch.0, &["format"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert_eq!(
        carries_the_fallback(&scratch, "htmlspec/specs/earlier.html"),
        0
    );
    let (code, out, err) = run_in(&scratch.0, &["format", "--check"]);
    assert_eq!(
        code, SUCCESS,
        "a document without the fallback is canonical:\n{out}{err}"
    );
}

/// The request adds it, says so, and a second request changes nothing.
#[test]
fn format_fallback_adds_it_once_and_is_then_idempotent() {
    let scratch = Scratch::new("fallback-add");
    a_project_with_documents_lacking_the_fallback(&scratch, &["htmlspec/specs/earlier.html"]);

    let (code, out, err) = run_in(&scratch.0, &["format", "--fallback"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(err.contains("gains the fallback stylesheet"), "{err}");
    assert_eq!(
        carries_the_fallback(&scratch, "htmlspec/specs/earlier.html"),
        1
    );
    let once = std::fs::read_to_string(scratch.path("htmlspec/specs/earlier.html")).expect("read");

    let (code, out, err) = run_in(&scratch.0, &["format", "--fallback"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let twice = std::fs::read_to_string(scratch.path("htmlspec/specs/earlier.html")).expect("read");
    assert_eq!(twice, once, "a second request changed the document");
    for arguments in [
        vec!["format", "--check", "--fallback"],
        vec!["format", "--check"],
        vec!["validate", "--strict", "htmlspec/specs/earlier.html"],
    ] {
        let (code, out, err) = run_in(&scratch.0, &arguments);
        assert_eq!(
            code, SUCCESS,
            "{arguments:?} fails after the request:\n{out}{err}"
        );
    }
}

/// With `--check`, the request reports a document that lacks the fallback and writes nothing, so a
/// project that wants every document to carry one can gate on it.
#[test]
fn format_check_fallback_reports_a_document_without_it_and_writes_nothing() {
    let scratch = Scratch::new("fallback-check");
    a_project_with_documents_lacking_the_fallback(&scratch, &["htmlspec/specs/earlier.html"]);
    let before =
        std::fs::read_to_string(scratch.path("htmlspec/specs/earlier.html")).expect("read");

    let (code, out, _) = run_in(&scratch.0, &["format", "--check", "--fallback"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}");
    assert!(
        out.contains("earlier.html"),
        "the document is not named: {out}"
    );
    assert_eq!(
        std::fs::read_to_string(scratch.path("htmlspec/specs/earlier.html")).expect("read"),
        before,
        "check mode wrote the document"
    );
}

/// A fallback that differs from the current one is replaced on request, and only on request.
#[test]
fn a_stale_fallback_is_replaced_only_on_request() {
    let scratch = Scratch::new("fallback-stale");
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let (code, out, err) = run_in(
        &scratch.0,
        &[
            "new",
            "document",
            "specification",
            "htmlspec/specs/stale.html",
            "--subject",
            "S",
        ],
    );
    assert_eq!(code, SUCCESS, "{out}{err}");
    let path = scratch.path("htmlspec/specs/stale.html");
    let current = std::fs::read_to_string(&path).expect("read");
    let stale = current.replace("line-height: 1.55;", "line-height: 1.5;");
    std::fs::write(&path, &stale).expect("written");

    let (code, out, err) = run_in(&scratch.0, &["format"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        stale,
        "plain format refreshed it"
    );

    let (code, out, err) = run_in(&scratch.0, &["format", "--fallback"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(err.contains("replaced by the current one"), "{err}");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), current);
}

/// Archived changes and run records are history, and the request leaves them as written.
#[test]
fn format_fallback_leaves_history_as_written() {
    let scratch = Scratch::new("fallback-history");
    let archived = "htmlspec/changes/archive/2026-01-01-earlier/specification.html";
    let record = "htmlspec/runbooks/release/runs/first.html";
    let living = "htmlspec/specs/earlier.html";
    a_project_with_documents_lacking_the_fallback(&scratch, &[archived, record, living]);

    let (code, out, err) = run_in(&scratch.0, &["format", "--fallback"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert_eq!(
        carries_the_fallback(&scratch, archived),
        0,
        "an archived document changed"
    );
    assert_eq!(
        carries_the_fallback(&scratch, record),
        0,
        "a run's record changed"
    );
    assert_eq!(
        carries_the_fallback(&scratch, living),
        1,
        "the living document did not"
    );
    assert!(
        err.contains("2026-01-01-earlier") && err.contains("first.html") && err.contains("history"),
        "the skipped documents are not reported:\n{err}"
    );
}

/// The request-verb spelling takes the request too, as it takes every argument of `format`.
#[test]
fn put_format_takes_the_fallback_request() {
    let scratch = Scratch::new("fallback-verb");
    a_project_with_documents_lacking_the_fallback(&scratch, &["htmlspec/specs/earlier.html"]);
    let (code, out, err) = run_in(&scratch.0, &["put", "format", "--fallback"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert_eq!(
        carries_the_fallback(&scratch, "htmlspec/specs/earlier.html"),
        1
    );
}

// -----------------------------------------------------------------------------
// Bootstrap
// -----------------------------------------------------------------------------

/// The first-run experience, which is the defect this change exists to fix. Bare `init`
/// used to fail with "the following required arguments were not provided: <PATH>".
#[test]
fn bare_init_establishes_a_project_and_names_the_next_command() {
    let scratch = Scratch::new("init-bare");
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "claude"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");

    for expected in [
        "htmlspec/specs",
        "htmlspec/changes/archive",
        "htmlspec/assets",
    ] {
        assert!(
            scratch.path(expected).is_dir(),
            "{expected} was not created:\n{out}"
        );
    }
    assert!(scratch.path("spec.nix").is_file(), "no declaration:\n{out}");
    // The closing message names what to type, spelled for the tool selected.
    assert!(out.contains("htmlspec new change"), "{out}");

    // And the workflow it names is the lifecycle's entry point, read from the table rather
    // than written here. A test pinning a literal spelling would have to be rewritten by
    // every change that touches the workflow set, and it would still not assert the
    // property that matters: a first run is told where to start, not where to finish.
    let entry = htmlspec_assets::instructions::entry_point().expect("the table has one");
    assert!(
        out.contains(&format!("/htmlspec:{}", entry.name)),
        "the closing message does not name the entry point `{}`:\n{out}",
        entry.name
    );
    assert!(
        !out.contains("/htmlspec:archive"),
        "a fresh bootstrap named archival, which is work the contributor has not \
         started:\n{out}"
    );
}

/// Task 5.5: a re-run refreshes program-owned output and leaves the contributor's work
/// alone. This is what makes `init` the way a project picks up newer instructions.
#[test]
fn a_re_run_refreshes_instructions_and_preserves_everything_else() {
    let scratch = Scratch::new("init-rerun");
    assert_eq!(
        run_in(&scratch.0, &["init", "--tools", "claude"]).0,
        SUCCESS
    );
    assert_eq!(run_in(&scratch.0, &["new", "change", "mine"]).0, SUCCESS);

    // A document the contributor edited, and a declaration they bumped.
    let proposal = scratch.path("htmlspec/changes/mine/proposal.html");
    let edited = std::fs::read_to_string(&proposal)
        .expect("readable")
        .replace("</body>", "<p>My own words.</p></body>");
    std::fs::write(&proposal, &edited).expect("written");
    let declaration = scratch.path("spec.nix");
    let bumped = std::fs::read_to_string(&declaration)
        .expect("readable")
        .replace("0.0.0", "1.4.0");
    std::fs::write(&declaration, &bumped).expect("written");

    let before = store_size(&scratch.path("htmlspec/assets"));

    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "claude"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");

    // The contributor's document and declaration survive byte-identical.
    assert_eq!(
        std::fs::read_to_string(&proposal).expect("readable"),
        edited,
        "a re-run modified a document under changes/"
    );
    assert_eq!(
        std::fs::read_to_string(&declaration).expect("readable"),
        bumped,
        "a re-run reverted the declared version"
    );
    // The skill is present and refreshed rather than duplicated.
    assert!(scratch
        .path(".claude/skills/htmlspec-archive-change/SKILL.md")
        .is_file());
    // And the store did not grow.
    assert_eq!(
        before,
        store_size(&scratch.path("htmlspec/assets")),
        "a re-run changed the number of stored assets"
    );
}

/// How many files a store holds, counted recursively.
fn store_size(store: &Path) -> usize {
    fn count(directory: &Path) -> usize {
        std::fs::read_dir(directory)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| {
                        let path = entry.path();
                        if path.is_dir() {
                            count(&path)
                        } else {
                            1
                        }
                    })
                    .sum()
            })
            .unwrap_or(0)
    }
    count(store)
}

/// Task 7.7: the non-interactive path completes without blocking.
///
/// The property that makes `init` safe in continuous integration. A prompt appearing here
/// would hang the run forever, so this asserts against a wall clock rather than trusting
/// that the interactivity test is right: `Command::output` gives the child no terminal, and
/// the run must finish regardless.
#[test]
fn init_does_not_block_when_no_terminal_is_attached() {
    let scratch = Scratch::new("init-no-tty");
    // A detected tool, so the path that would prompt is the one taken.
    std::fs::create_dir_all(scratch.path(".claude")).expect("created");

    let start = std::time::Instant::now();
    // No `--tools`, which is the case that prompts when a terminal is present.
    let (code, out, err) = run_in(&scratch.0, &["init"]);
    let elapsed = start.elapsed();

    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(
        elapsed < std::time::Duration::from_secs(30),
        "init took {elapsed:?}, which suggests it waited for input"
    );
    // It used what it detected rather than asking.
    assert!(err.contains("detected: Claude Code"), "{err}");
    assert!(scratch
        .path(".claude/skills/htmlspec-archive-change/SKILL.md")
        .is_file());
}

/// Structured output suppresses the prompt and keeps stdout parseable.
#[test]
fn init_with_structured_output_does_not_prompt() {
    let scratch = Scratch::new("init-json");
    std::fs::create_dir_all(scratch.path(".claude")).expect("created");
    let (code, _, err) = run_in(&scratch.0, &["--json", "init"]);
    assert_eq!(code, SUCCESS, "{err}");
    assert!(scratch.path("htmlspec").is_dir());
}

/// A misspelled tool is rejected before anything is written.
#[test]
fn an_unknown_tool_is_rejected_before_a_project_is_created() {
    let scratch = Scratch::new("init-bad-tool");
    let (code, _, err) = run_in(&scratch.0, &["init", "--tools", "emacs"]);
    assert_eq!(code, OPERATIONAL_FAILURE);
    assert!(err.contains("not a supported tool"), "{err}");
    assert!(
        !scratch.path("htmlspec").exists(),
        "a project was created despite the rejected selection"
    );
}

/// `--tools none` is a legitimate answer: the project is established and no instructions
/// are installed, and the output does not name a command no tool answers to.
#[test]
fn no_tools_establishes_the_project_and_names_no_tool_command() {
    let scratch = Scratch::new("init-no-tools");
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(scratch.path("htmlspec").is_dir());
    assert!(out.contains("htmlspec new change"), "{out}");
    // No tool-specific spelling for any workflow, checked across the whole table rather
    // than against one name: with no tool selected there is nothing that answers to any of
    // them.
    for workflow in htmlspec_assets::instructions::WORKFLOWS {
        assert!(
            !out.contains(&format!("/htmlspec:{}", workflow.name)),
            "a tool-specific command was named with no tool selected: {}\n{out}",
            workflow.name
        );
    }
}

/// The measured defect, end to end.
///
/// A change directory copied out of its project keeps references that climb three levels for
/// a store that is no longer there. Strict validation reported **zero diagnostics** and
/// accepted it — 21 dead references across three documents. This asserts the opposite.
#[test]
fn strict_validation_reports_a_change_copied_out_of_its_project() {
    let scratch = Scratch::new("orphan");
    assert_eq!(run_in(&scratch.0, &["init", "--tools", "none"]).0, SUCCESS);
    assert_eq!(
        run_in(&scratch.0, &["new", "change", "dark-mode"]).0,
        SUCCESS
    );

    // In place, inside its project, it validates.
    let inside = scratch.path("htmlspec/changes/dark-mode");
    let (code, out, err) = run(&["validate", &as_str(&inside)]);
    assert_eq!(
        code, SUCCESS,
        "the change does not validate in place:\nstdout: {out}\nstderr: {err}"
    );

    // Copied out, every asset reference is dead.
    let orphan = scratch.path("orphan/dark-mode");
    std::fs::create_dir_all(&orphan).expect("created");
    for file in ["proposal.html", "design.html", "tasks.html"] {
        std::fs::copy(inside.join(file), orphan.join(file)).expect("copied");
    }

    let (code, out, _) = run(&["validate", &as_str(&orphan), "--strict"]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "an orphaned change was accepted with every asset reference dead:\n{out}"
    );
    assert!(
        out.contains("asset/unresolved"),
        "the report does not name the rule:\n{out}"
    );
    assert!(
        out.contains("reaches above this document"),
        "the report does not say the document was separated from its store:\n{out}"
    );
}

/// document-validation's "An asset reference must match the bytes it names", for a reference whose
/// path misses the store.
///
/// A change moved by hand one level deeper, as archival moved one before it re-expressed references:
/// every document still references `../../assets/`, which from the archive is
/// `htmlspec/changes/assets/`, a directory that does not exist. `htmlspec 0.29.1` accepted it,
/// because the address and file name at the end of each path are stored in the project's store.
#[test]
fn a_reference_whose_path_misses_the_store_is_unresolved() {
    let scratch = Scratch::new("misplaced");
    assert_eq!(run_in(&scratch.0, &["init", "--tools", "none"]).0, SUCCESS);
    assert_eq!(run_in(&scratch.0, &["new", "change", "moved"]).0, SUCCESS);
    let archived = scratch.path("htmlspec/changes/archive/2026-01-01-moved");
    std::fs::create_dir_all(archived.parent().expect("the archive")).expect("created");
    std::fs::rename(scratch.path("htmlspec/changes/moved"), &archived).expect("moved by hand");

    let (code, out, err) = run(&["validate", &as_str(&archived)]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "a change whose references reach no store was accepted:\nstdout: {out}\nstderr: {err}"
    );
    assert!(out.contains("asset/unresolved"), "{out}");
    assert!(
        out.contains("leads to ../../assets/") && out.contains("store is at ../../../assets/"),
        "the report does not name where the path leads and where the store is:\n{out}"
    );

    // Formatting does not move a reference: deterministic-formatting's "Formatting preserves
    // meaning" leaves one naming stored bytes as written, and this one does name them.
    let before = std::fs::read(archived.join("proposal.html")).expect("readable");
    let (code, out, err) = run(&["format", &as_str(&archived)]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert_eq!(
        std::fs::read(archived.join("proposal.html")).expect("readable"),
        before,
        "formatting moved a misplaced reference"
    );

    // Pointed at the store, the same references are accepted.
    for file in ["proposal.html", "design.html", "tasks.html"] {
        let path = archived.join(file);
        let source = std::fs::read_to_string(&path).expect("readable");
        std::fs::write(
            &path,
            source.replace("\"../../assets/", "\"../../../assets/"),
        )
        .expect("written");
    }
    let (code, out, err) = run(&["validate", &as_str(&archived)]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
}

/// A reference leads to the store when the directory it names, from the document, is the store's,
/// however it is spelled.
///
/// The code review of pull request #58 found that the path check compared the reference's path with
/// the store's as text, so `../../../htmlspec/assets/`, which climbs out of `htmlspec/` and back in
/// and which a browser loads from the store, was reported as leading elsewhere. A reference that
/// climbs past the project and back in by the directory's own name lands there too.
#[test]
fn a_reference_that_climbs_out_and_back_in_reaches_the_store() {
    let scratch = Scratch::new("climbs");
    assert_eq!(run_in(&scratch.0, &["init", "--tools", "none"]).0, SUCCESS);
    assert_eq!(run_in(&scratch.0, &["new", "change", "climbs"]).0, SUCCESS);
    let change = scratch.path("htmlspec/changes/climbs");
    let checkout = scratch
        .0
        .file_name()
        .expect("the scratch directory has a name")
        .to_string_lossy()
        .into_owned();

    for spelled in [
        String::from("../../../htmlspec/assets/"),
        format!("../../../../{checkout}/htmlspec/assets/"),
    ] {
        let path = change.join("proposal.html");
        let original = std::fs::read_to_string(&path).expect("readable");
        std::fs::write(
            &path,
            original.replace("\"../../assets/", &format!("\"{spelled}")),
        )
        .expect("written");
        let (code, out, err) = run(&["validate", &as_str(&path)]);
        assert_eq!(
            code, SUCCESS,
            "{spelled} reaches the store and was rejected:\nstdout: {out}\nstderr: {err}"
        );
        std::fs::write(&path, original).expect("restored");
    }
}

/// Whether a reference reaches the store is read from its path alone, so the verdict is the same on
/// every machine.
///
/// `../../ASSETS/` loads the store's bytes on a filesystem that ignores case, as macOS's does by
/// default, and loads nothing on one that does not, as a Linux machine running the project's checks
/// does. A verdict that depended on which of the two validated the document would pass a document
/// on a contributor's machine and fail it in continuous integration, so it is rejected on both.
#[test]
fn a_reference_spelled_in_another_case_does_not_reach_the_store() {
    let scratch = Scratch::new("case");
    assert_eq!(run_in(&scratch.0, &["init", "--tools", "none"]).0, SUCCESS);
    assert_eq!(run_in(&scratch.0, &["new", "change", "case"]).0, SUCCESS);
    let path = scratch.path("htmlspec/changes/case/proposal.html");
    let original = std::fs::read_to_string(&path).expect("readable");
    std::fs::write(
        &path,
        original.replace("\"../../assets/", "\"../../ASSETS/"),
    )
    .expect("written");

    let (code, out, err) = run(&["validate", &as_str(&path)]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "a reference whose verdict depends on the filesystem was accepted:\nstdout: {out}\nstderr: {err}"
    );
    assert!(out.contains("leads to ../../ASSETS/"), "{out}");
}

// -----------------------------------------------------------------------------
// Assets
// -----------------------------------------------------------------------------

/// The measured defect, as an assertion.
///
/// Two changes each holding their own copy of the vendored runtime is 6 MB twice, and
/// the previous `assets` reported `no asset store under .` and exited zero because it
/// checked only the directory it was handed. A project-wide check sees the duplication
/// because it descends, and duplication *between* two stores is invisible to either one
/// alone: each addressed copy sits by itself at its own address.
#[test]
fn the_asset_check_reports_duplication_between_two_stores() {
    let scratch = Scratch::new("assets-two-stores");

    // Two stores, holding the same bytes at the same address, two directories down.
    // A real project, because cross-store duplication is a statement about one project.
    assert_eq!(run_in(&scratch.0, &["init", "--tools", "none"]).0, SUCCESS);

    let bytes = b"shared runtime bytes";
    let digest = htmlspec_assets::sha256_hex(bytes);
    let address = format!("sha256-{}", &digest[..16]);
    for change in ["one", "another"] {
        let path = scratch.path(&format!(
            "htmlspec/changes/{change}/assets/{address}/runtime.js"
        ));
        std::fs::create_dir_all(path.parent().expect("the address directory")).expect("created");
        std::fs::write(&path, bytes).expect("the asset is written");
    }

    let (code, out, err) = run(&["assets", &as_str(&scratch.0)]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "the duplication was not reported\nstdout: {out}\nstderr: {err}"
    );
    // Both locations are named, so a contributor can tell which copy to remove.
    for change in ["one", "another"] {
        assert!(
            out.contains(change),
            "the report does not name the {change} store:\n{out}"
        );
    }
}

/// One store holding each asset once is the arrangement the project wants, and it
/// passes. Written alongside the failing case so the rule is pinned in both directions.
#[test]
fn the_asset_check_accepts_one_store_per_project() {
    let scratch = Scratch::new("assets-one-store");
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let (code, out, err) = run_in(&scratch.0, &["new", "change", "only"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");

    let (code, out, err) = run(&["assets", &as_str(&scratch.0)]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    // The partition is stated whenever a project is in view, including when there is nothing
    // outside it: document-validation requires the report to say which stores were whose.
    assert!(
        err.contains("1 store(s) within the project's htmlspec/")
            && err.contains("No store outside it was found"),
        "the report does not state the partition:\n{err}"
    );
}

/// Two changes in one project share one store, which is the arrangement the whole
/// change exists to produce. Measured before as 6 MB per change.
#[test]
fn two_changes_in_a_project_share_one_store() {
    let scratch = Scratch::new("assets-shared-store");
    assert_eq!(run_in(&scratch.0, &["init", "--tools", "none"]).0, SUCCESS);
    for name in ["one", "another"] {
        let (code, out, err) = run_in(&scratch.0, &["new", "change", name]);
        assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
        assert!(
            !scratch
                .path(&format!("htmlspec/changes/{name}/assets"))
                .exists(),
            "the {name} change established its own store"
        );
    }
    // One store, and the project-wide check is clean.
    let (code, out, err) = run(&["assets", &as_str(&scratch.0)]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
}

/// Copies every file under `from` to the same relative path under `to`.
fn copy_tree(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("the source is readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            std::fs::create_dir_all(&target).expect("the directory is created");
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("the file is copied");
        }
    }
}

/// A project established by `init`, whose one store is `htmlspec/assets/`.
fn a_project(label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(scratch.path("htmlspec/assets").is_dir());
    scratch
}

/// The document-validation scenario "A project beside a fixture that carries its own store".
///
/// A conformance fixture is a project of its own and carries its own bytes on purpose. This
/// repository holds four such stores, and creating a project beside them reported nine
/// duplicates — the check comparing a repository rather than a project.
#[test]
fn a_project_beside_a_fixture_store_is_not_a_duplicate_of_it() {
    let scratch = a_project("assets-beside-fixture");
    let fixture = scratch.path("fixtures/conformance/a-fixture");
    std::fs::create_dir_all(fixture.join("assets")).expect("the fixture store is created");
    copy_tree(&scratch.path("htmlspec/assets"), &fixture.join("assets"));

    let (code, out, err) = run(&["assets", &as_str(&scratch.0)]);
    assert_eq!(
        code, SUCCESS,
        "a fixture's store was compared with the project's\nstdout: {out}\nstderr: {err}"
    );
    assert!(
        err.contains("checked on its own") && err.contains("fixtures/conformance/a-fixture/assets"),
        "the report does not name the fixture's store as checked on its own:\n{err}"
    );
}

/// The document-validation scenario "A project beside a toolchain's source copy of the runtime".
///
/// This repository's `crates/htmlspec-assets/assets/` is the toolchain's source copy, which the
/// build embeds and `init` copies from, so a project here holds the same bytes by construction.
/// The scratch repository below keeps such a copy at its root, where this repository kept its own
/// until `ship-through-four-channels`: a source copy outside `htmlspec/` is not the project's,
/// wherever it sits.
#[test]
fn a_project_beside_a_toolchain_source_store_is_not_a_duplicate_of_it() {
    let scratch = a_project("assets-beside-toolchain");
    std::fs::create_dir_all(scratch.path("assets")).expect("the root store is created");
    copy_tree(&scratch.path("htmlspec/assets"), &scratch.path("assets"));

    let (code, out, err) = run(&["assets", &as_str(&scratch.0)]);
    assert_eq!(
        code, SUCCESS,
        "the root store was compared with the project's\nstdout: {out}\nstderr: {err}"
    );

    let (code, out, err) = run(&["assets", &as_str(&scratch.0), "--json"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
    assert_eq!(payload["stores"], 2, "{payload}");
    assert_eq!(payload["accepted"], true, "{payload}");
    let names = |key: &str| -> Vec<String> {
        payload[key]
            .as_array()
            .unwrap_or_else(|| panic!("the payload has no {key} list: {payload}"))
            .iter()
            .map(|path| path.as_str().expect("a path").to_owned())
            .collect()
    };
    assert!(
        names("projectStores")
            .iter()
            .all(|p| p.ends_with("htmlspec/assets"))
            && names("projectStores").len() == 1,
        "{payload}"
    );
    assert_eq!(names("otherStores").len(), 1, "{payload}");
}

/// The document-validation scenario "Two stores inside the project": the partition narrows the
/// comparison to the project, and inside it nothing is excused.
#[test]
fn two_stores_inside_the_project_are_still_compared() {
    let scratch = a_project("assets-two-inside");
    let beside = scratch.path("htmlspec/changes/a-change/assets");
    std::fs::create_dir_all(&beside).expect("the change's store is created");
    copy_tree(&scratch.path("htmlspec/assets"), &beside);

    let (code, out, err) = run(&["assets", &as_str(&scratch.0)]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "a second store inside the project was not reported\nstdout: {out}\nstderr: {err}"
    );
    assert!(out.contains("htmlspec/changes/a-change/assets"), "{out}");
    assert!(out.contains("htmlspec/assets/"), "{out}");
}

/// The document-validation scenario "A store outside the project that duplicates within itself".
///
/// Outside the project is not outside the check. A store holding the same bytes twice is
/// reported wherever it lives; it is only exempt from being compared with the project's.
#[test]
fn an_outside_store_that_duplicates_within_itself_is_reported() {
    let scratch = a_project("assets-outside-within");
    let bytes = b"a vendored file stored twice in one store";
    let digest = htmlspec_assets::sha256_hex(bytes);
    let address = format!("sha256-{}", &digest[..16]);
    for name in ["first.js", "second.js"] {
        let path = scratch.path(&format!("vendor/assets/{address}/{name}"));
        std::fs::create_dir_all(path.parent().expect("the address directory")).expect("created");
        std::fs::write(&path, bytes).expect("written");
    }

    let (code, out, err) = run(&["assets", &as_str(&scratch.0)]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "a store duplicating within itself was excused for being outside the project\n\
         stdout: {out}\nstderr: {err}"
    );
    assert!(
        out.contains("first.js") && out.contains("second.js"),
        "the report does not name both locations:\n{out}"
    );
}

// -----------------------------------------------------------------------------
// Evidence
// -----------------------------------------------------------------------------

/// A document kept as a record: the contract version it declared when it was written, and the
/// stylesheet it was written against, whose bytes the project no longer stores.
///
/// This is the shape of this repository's own HTML evidence — the migration's plan, its collapse,
/// and four documents converted by hand in August — each of which declares `0.1.0` and names a
/// stylesheet revision that has since been replaced. The two addresses are the ones those records
/// carry. It also carries a dangling reference, so it fails outside any project too, where there is
/// no store for a stylesheet to be missing from. As a document it fails `validate` and is rewritten
/// by `format`, which is what makes it a useful subject: a walk that reaches it is visible.
fn a_record() -> String {
    let mut document = invalid();
    let declared = document
        .split("<meta content=\"")
        .find_map(|rest| rest.split_once("\" name=\"htmlspec-version\""))
        .map(|(version, _)| version.to_owned())
        .expect("the template declares its contract version");
    document = document.replace(
        &format!("<meta content=\"{declared}\" name=\"htmlspec-version\""),
        "<meta content=\"0.1.0\" name=\"htmlspec-version\"",
    );
    for (name, superseded) in [
        ("htmlspec.css", "sha256-e4f5de7cecc4f691/htmlspec.css"),
        (
            "htmlspec-print.css",
            "sha256-0350261bb5666167/htmlspec-print.css",
        ),
    ] {
        let current = htmlspec_assets::all_assets()
            .into_iter()
            .find(|asset| asset.name() == name)
            .unwrap_or_else(|| panic!("{name} is embedded"));
        let replaced = document.replace(current.path, superseded);
        assert_ne!(replaced, document, "the template does not reference {name}");
        document = replaced;
    }
    document
}

/// A project holding one change created by the program, whose documents conform, and a record in
/// that change's evidence at `evidence_path` beneath the change.
fn a_project_with_evidence(label: &str, evidence_path: &str) -> (Scratch, PathBuf) {
    let scratch = a_project(label);
    let (code, out, err) = run_in(&scratch.0, &["new", "change", "a-change"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let record = scratch.write(
        &format!("htmlspec/changes/a-change/evidence/{evidence_path}"),
        &a_record(),
    );
    (scratch, record)
}

/// The record really is non-conforming, so a walk that passes over it proves something.
///
/// Without this, every test below would pass against a record that happened to validate, and
/// "excluded" would be indistinguishable from "checked and clean".
#[test]
fn a_record_does_not_conform_where_it_sits() {
    let (_scratch, record) = a_project_with_evidence("evidence-baseline", "a-record.html");
    let (code, out, err) = run(&["validate", &as_str(&record)]);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(out.contains("asset/unresolved"), "{out}");
}

/// toolchain-packaging's "Validating a project that holds HTML evidence".
#[test]
fn validating_a_project_passes_over_a_changes_evidence() {
    let (scratch, _record) = a_project_with_evidence("evidence-validate", "a-record.html");

    let (code, out, err) = run_in(&scratch.0, &["validate"]);
    assert_eq!(
        code, SUCCESS,
        "a change's evidence was validated as a document\nstdout: {out}\nstderr: {err}"
    );
    assert!(
        !out.contains("a-record.html") && !err.contains("a-record.html"),
        "the report names the evidence file:\nstdout: {out}\nstderr: {err}"
    );
    // The change's own documents were still found, so the pass is not an empty walk.
    assert!(err.contains("a-change/proposal.html"), "{err}");
}

/// toolchain-packaging's "Formatting leaves evidence untouched".
///
/// The measured defect: `format` repointed every carried record's stylesheet at the current store,
/// destroying the byte-identity evidence exists for.
#[test]
fn formatting_a_project_leaves_a_changes_evidence_byte_identical() {
    let (scratch, record) = a_project_with_evidence("evidence-format", "a-record.html");
    let before = std::fs::read(&record).expect("the record is readable");

    let (code, out, err) = run_in(&scratch.0, &["format"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert_eq!(
        std::fs::read(&record).expect("the record is readable"),
        before,
        "format rewrote a change's evidence"
    );

    let (code, out, err) = run_in(&scratch.0, &["format", "--check"]);
    assert_eq!(
        code, SUCCESS,
        "format --check judged a change's evidence\nstdout: {out}\nstderr: {err}"
    );
}

/// toolchain-packaging's "Naming an evidence file checks it". A file means that file.
#[test]
fn naming_an_evidence_file_checks_it() {
    let (scratch, record) = a_project_with_evidence("evidence-named-file", "a-record.html");

    let (code, out, err) = run_in(&scratch.0, &["validate", &as_str(&record)]);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(out.contains("a-record.html"), "{out}");
}

/// toolchain-packaging's "Naming the evidence directory checks what it holds".
///
/// The exclusion is of what a walk comes across, never of the walk's own root, so a contributor
/// who asks about the records gets the ordinary answer.
#[test]
fn naming_the_evidence_directory_checks_what_it_holds() {
    let (scratch, _record) =
        a_project_with_evidence("evidence-named-directory", "transcripts/a-record.html");

    for named in [
        "htmlspec/changes/a-change/evidence",
        "htmlspec/changes/a-change/evidence/transcripts",
    ] {
        let (code, out, err) = run_in(&scratch.0, &["validate", named]);
        assert_eq!(
            code, DOCUMENT_VIOLATION,
            "naming {named} did not check it\nstdout: {out}\nstderr: {err}"
        );
        assert!(out.contains("a-record.html"), "{named}: {out}");
    }
}

/// toolchain-packaging's "Formatting a named record is refused".
///
/// artifact-kinds says no operation rewrites a file beneath a change's evidence, and a named path
/// is walked as named. Checking a named record is the ordinary answer; writing one is refused
/// before anything is written. A review of the pull request found the first version rewrote a named
/// evidence directory in full, which is the scenario this reproduces.
#[test]
fn formatting_a_named_record_is_refused_and_writes_nothing() {
    let (scratch, record) =
        a_project_with_evidence("evidence-format-named", "transcripts/a-record.html");
    let before = std::fs::read(&record).expect("the record is readable");

    for named in [
        as_str(&record),
        String::from("htmlspec/changes/a-change/evidence"),
        String::from("htmlspec/changes/a-change/evidence/transcripts"),
    ] {
        let (code, out, err) = run_in(&scratch.0, &["format", &named]);
        assert_eq!(
            code, OPERATIONAL_FAILURE,
            "format wrote to {named}\nstdout: {out}\nstderr: {err}"
        );
        assert!(
            err.contains("refusing to write") && err.contains("a-record.html"),
            "the refusal does not name the record for {named}:\n{err}"
        );
        assert!(err.contains("nothing was written"), "{err}");
        assert_eq!(
            std::fs::read(&record).expect("the record is readable"),
            before,
            "format rewrote a named record at {named}"
        );

        // Checking is not writing: the named record is still judged, and still left alone.
        let (code, out, err) = run_in(&scratch.0, &["format", "--check", &named]);
        assert_eq!(
            code, DOCUMENT_VIOLATION,
            "format --check did not judge {named}\nstdout: {out}\nstderr: {err}"
        );
        assert_eq!(std::fs::read(&record).expect("readable"), before);
    }

    // A named change directory holds documents and evidence; the walk passes over the evidence,
    // so there is nothing to refuse and the change's documents are formatted as usual.
    let (code, out, err) = run_in(&scratch.0, &["format", "htmlspec/changes/a-change"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert_eq!(std::fs::read(&record).expect("readable"), before);
}

/// A project nested in `wrap/proj` of a fresh scratch directory, holding one change created by the
/// program and a non-conforming record in that change's evidence.
fn a_nested_project_with_evidence(label: &str) -> (Scratch, PathBuf) {
    let scratch = Scratch::new(label);
    let project = scratch.path("wrap/proj");
    std::fs::create_dir_all(&project).expect("the project's directory");
    let (code, out, err) = run_in(&project, &["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let (code, out, err) = run_in(&project, &["new", "change", "a-change"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let record = scratch.write(
        "wrap/proj/htmlspec/changes/a-change/evidence/a-record.html",
        &a_record(),
    );
    (scratch, record)
}

/// toolchain-packaging's "An operation over a directory collects documents and not a change's
/// evidence", from a directory *above* the project.
///
/// Until 1.1.0 the walk resolved its project from its own root alone, so a root above a project
/// resolved none and walked into that project's evidence: `validate wrap` reported the record,
/// `format --check wrap` judged it, and `format wrap` met it and refused the whole run. The walk now
/// takes the project of each directory it enters, so the three behave as they do from inside.
#[test]
fn from_above_a_project_a_walk_passes_over_its_evidence() {
    let (scratch, record) = a_nested_project_with_evidence("evidence-from-above");
    let before = std::fs::read(&record).expect("the record is readable");

    let (code, out, err) = run_in(&scratch.0, &["validate", "wrap"]);
    assert_eq!(
        code, SUCCESS,
        "validate from above walked into the evidence\nstdout: {out}\nstderr: {err}"
    );
    assert!(
        !out.contains("a-record.html") && !err.contains("a-record.html"),
        "the report names the evidence file:\nstdout: {out}\nstderr: {err}"
    );
    assert!(err.contains("a-change/proposal.html"), "{err}");

    let (code, out, err) = run_in(&scratch.0, &["format", "--check", "wrap"]);
    assert_eq!(
        code, SUCCESS,
        "format --check from above judged the evidence\nstdout: {out}\nstderr: {err}"
    );

    let (code, out, err) = run_in(&scratch.0, &["format", "wrap"]);
    assert_eq!(
        code, SUCCESS,
        "format from above refused or failed\nstdout: {out}\nstderr: {err}"
    );
    assert!(!err.contains("refusing to write"), "{err}");
    assert_eq!(
        std::fs::read(&record).expect("the record is readable"),
        before,
        "format from above rewrote a change's evidence"
    );
}

/// The same walk through a link: a directory outside the project holding a link to it walks the
/// project's documents and passes over its evidence, as walking the project itself does.
#[cfg(unix)]
#[test]
fn through_a_link_to_a_project_a_walk_passes_over_its_evidence() {
    let (scratch, record) = a_nested_project_with_evidence("evidence-through-a-link");
    let before = std::fs::read(&record).expect("the record is readable");
    std::fs::create_dir_all(scratch.path("via")).expect("the linking directory");
    std::os::unix::fs::symlink(scratch.path("wrap/proj"), scratch.path("via/proj"))
        .expect("the link to the project");

    let (code, out, err) = run_in(&scratch.0, &["validate", "via"]);
    assert_eq!(
        code, SUCCESS,
        "validate through a link walked into the evidence\nstdout: {out}\nstderr: {err}"
    );
    assert!(!out.contains("a-record.html"), "{out}");
    assert!(err.contains("a-change/proposal.html"), "{err}");

    let (code, out, err) = run_in(&scratch.0, &["format", "via"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert_eq!(
        std::fs::read(&record).expect("the record is readable"),
        before,
        "format through a link rewrote a change's evidence"
    );
}

/// toolchain-packaging's "Outside a project every file is a document", in both of its settings.
///
/// Outside any project there are no changes, so nothing is a change's evidence. And a directory
/// outside `htmlspec/` in a repository that holds a project is outside every change too: after this
/// repository's own migration, `fixtures/` is walked from inside a project, and a fixture directory
/// that happens to be called `evidence` must still be checked.
#[test]
fn outside_a_project_an_evidence_directory_is_walked() {
    let loose = Scratch::new("evidence-outside");
    loose.write("evidence/a-record.html", &a_record());
    let (code, out, err) = run(&["validate", &as_str(&loose.0)]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "a directory outside any project skipped its evidence\nstdout: {out}\nstderr: {err}"
    );

    let project = a_project("evidence-beside-project");
    project.write("fixtures/a-fixture/evidence/a-record.html", &a_record());
    let (code, out, err) = run_in(&project.0, &["validate", "fixtures"]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "a directory outside htmlspec/ skipped its evidence\nstdout: {out}\nstderr: {err}"
    );
}

/// artifact-kinds' "Evidence nested below the evidence directory". Every file beneath it, at any
/// depth — the flat-glob fault that under-counted this repository's evidence by four, in the survey.
#[test]
fn evidence_nested_below_the_evidence_directory_is_evidence() {
    let (scratch, _record) =
        a_project_with_evidence("evidence-nested", "transcripts/a-session.html");

    let (code, out, err) = run_in(&scratch.0, &["validate"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
}

/// artifact-kinds' "An archived change keeps its evidence".
#[test]
fn an_archived_changes_evidence_is_evidence() {
    let scratch = a_project("evidence-archived");
    scratch.write(
        "htmlspec/changes/archive/2026-01-01-a-change/evidence/a-record.html",
        &a_record(),
    );

    let (code, out, err) = run_in(&scratch.0, &["validate"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let (code, out, err) = run_in(&scratch.0, &["format", "--check"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
}

/// artifact-kinds' "A capability that happens to be named evidence".
///
/// The exclusion is positional. A name test — skip every directory called `evidence`, the way
/// `target` is skipped — is one line, and it would stop validating this specification with nothing
/// to say so.
#[test]
fn a_capability_named_evidence_is_still_validated() {
    let scratch = a_project("evidence-capability");
    scratch.write("htmlspec/specs/evidence/spec.html", &a_record());

    let (code, out, err) = run_in(&scratch.0, &["validate"]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "a specification at specs/evidence/ was skipped\nstdout: {out}\nstderr: {err}"
    );
    assert!(out.contains("specs/evidence/spec.html"), "{out}");
}

/// A directory named `evidence` directly inside the archive belongs to no change, because the
/// archive is not a change. It stays walked.
#[test]
fn an_evidence_directory_directly_in_the_archive_is_walked() {
    let scratch = a_project("evidence-in-archive");
    scratch.write(
        "htmlspec/changes/archive/evidence/a-record.html",
        &a_record(),
    );

    let (code, out, err) = run_in(&scratch.0, &["validate"]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "archive/evidence/ was taken for a change's evidence\nstdout: {out}\nstderr: {err}"
    );
}

// -----------------------------------------------------------------------------
// Init
// -----------------------------------------------------------------------------

/// The scenario the specs state: a new document passes validation immediately, and
/// the assets are established as part of initialization.
#[test]
fn new_document_creates_a_document_that_validates_immediately() {
    let scratch = Scratch::new("init");
    let path = scratch.path("proposal.html");

    let (code, out, _) = run(&[
        "new",
        "document",
        "proposal",
        &as_str(&path),
        "--subject",
        "A new proposal",
    ]);
    assert_eq!(code, SUCCESS, "{out}");
    assert!(path.exists(), "no document was written");

    // Validated by the program itself, in a separate invocation, which is the
    // promise a contributor actually relies on.
    assert_eq!(run(&["validate", &as_str(&path)]).0, SUCCESS);
    // And it is already canonical, so `format --check` passes on a fresh document.
    assert_eq!(run(&["format", "--check", &as_str(&path)]).0, SUCCESS);

    // The shared assets are present, so the document renders offline with no
    // further step.
    for asset in htmlspec_assets::all_assets() {
        assert!(
            scratch.path("assets").join(asset.path).exists(),
            "init did not establish {}",
            asset.path
        );
    }
}

#[test]
fn new_document_refuses_to_overwrite_and_exits_with_an_operational_failure() {
    let scratch = Scratch::new("new-overwrite");
    let path = scratch.write("doc.html", "original");
    let (code, _, err) = run(&["new", "document", "proposal", &as_str(&path)]);
    assert_eq!(code, OPERATIONAL_FAILURE);
    assert!(err.contains("already exists"), "{err}");
    assert_eq!(
        std::fs::read_to_string(&path).expect("readable"),
        "original"
    );
}

// -----------------------------------------------------------------------------
// Which store a new document references
//
// document-template's "A project can be initialized with a conforming document": the
// new document "passes validation immediately", and initialization does not copy a
// runtime asset the project already stores. Validation looks for a document's store in
// the document's own directory and above it, so the store a new document references is
// decided by where the document will live. These are the four arrangements of working
// directory and destination that resolve-a-new-documents-project-from-its-destination
// measured, and before it three of them went wrong.
// -----------------------------------------------------------------------------

/// A scratch directory holding a bootstrapped project at `project/` and a sibling
/// directory `elsewhere/` that belongs to no project.
fn a_project_and_somewhere_else(label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    std::fs::create_dir_all(scratch.path("project")).expect("the project directory");
    std::fs::create_dir_all(scratch.path("elsewhere")).expect("the other directory");
    let (code, out, err) = run_in(&scratch.path("project"), &["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    scratch
}

/// Creates a proposal at `destination`, running from `from`, and asserts what every
/// arrangement owes: the command succeeds, and the program accepts what it wrote.
fn create_a_document_that_validates(from: &Path, destination: &Path) {
    let (code, out, err) = run_in(
        from,
        &[
            "new",
            "document",
            "proposal",
            &as_str(destination),
            "--subject",
            "A subject",
        ],
    );
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let (code, out, err) = run(&["validate", &as_str(destination)]);
    assert_eq!(
        code, SUCCESS,
        "the new document does not validate\nstdout: {out}\nstderr: {err}"
    );
}

/// The project's asset check passes: each asset is stored once, in one store.
fn the_project_stores_each_asset_once(project: &Path) {
    let (code, out, err) = run(&["assets", &as_str(project)]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
}

/// Run from inside a project, written outside it: the document gets a store beside it,
/// because the project's store is not above it. This is the arrangement that failed the
/// day this repository became a project.
#[test]
fn a_document_written_outside_the_project_it_is_created_from_gets_its_own_store() {
    let scratch = a_project_and_somewhere_else("new-doc-out-of-project");
    let destination = scratch.path("elsewhere/a/proposal.html");
    create_a_document_that_validates(&scratch.path("project"), &destination);
    assert!(
        scratch.path("elsewhere/a/assets").is_dir(),
        "no store beside the document"
    );
    the_project_stores_each_asset_once(&scratch.path("project"));
}

/// Run from inside a project, written into its root but outside `htmlspec/`: the
/// project's store is not above the document, so it gets a store beside it.
#[test]
fn a_document_written_into_a_projects_root_outside_its_directory_gets_its_own_store() {
    let scratch = a_project_and_somewhere_else("new-doc-project-root");
    let destination = scratch.path("project/docs/proposal.html");
    create_a_document_that_validates(&scratch.path("project"), &destination);
    assert!(
        scratch.path("project/docs/assets").is_dir(),
        "no store beside the document"
    );
    the_project_stores_each_asset_once(&scratch.path("project"));
}

/// Run from outside a project, written beneath its `htmlspec/`: the document references
/// the project's store, and no second store is written.
#[test]
fn a_document_written_into_a_project_from_outside_references_the_projects_store() {
    let scratch = a_project_and_somewhere_else("new-doc-into-project");
    let destination = scratch.path("project/htmlspec/changes/c/proposal.html");
    create_a_document_that_validates(&scratch.path("elsewhere"), &destination);
    assert!(
        !scratch.path("project/htmlspec/changes/c/assets").exists(),
        "a second store was written beside the document"
    );
    let written = std::fs::read_to_string(&destination).expect("the document is readable");
    assert!(
        written.contains("../../assets/"),
        "the project's store is not referenced:\n{written}"
    );
    the_project_stores_each_asset_once(&scratch.path("project"));
}

/// Run from outside every project, written outside every project: the document gets a
/// store beside it. The one arrangement that worked before, pinned so it stays working.
#[test]
fn a_document_written_outside_every_project_gets_its_own_store() {
    let scratch = a_project_and_somewhere_else("new-doc-no-project");
    let destination = scratch.path("elsewhere/d/proposal.html");
    create_a_document_that_validates(&scratch.path("elsewhere"), &destination);
    assert!(
        scratch.path("elsewhere/d/assets").is_dir(),
        "no store beside the document"
    );
}

/// A project whose store is not established yet: a document beneath its `htmlspec/`
/// establishes the store there, as "Initializing into a project without assets" requires,
/// and not beside the document.
#[test]
fn a_document_written_into_a_project_with_no_store_establishes_the_projects_store() {
    // The project and the other directory are siblings, so the command runs outside the
    // project and only the destination can find it.
    let scratch = Scratch::new("new-doc-storeless-project");
    std::fs::create_dir_all(scratch.path("project/htmlspec/changes")).expect("a storeless project");
    std::fs::create_dir_all(scratch.path("elsewhere")).expect("the other directory");
    let destination = scratch.path("project/htmlspec/changes/first/proposal.html");
    create_a_document_that_validates(&scratch.path("elsewhere"), &destination);
    for asset in htmlspec_assets::all_assets() {
        assert!(
            scratch
                .path("project/htmlspec/assets")
                .join(asset.path)
                .exists(),
            "the project's store does not hold {}",
            asset.path
        );
    }
    assert!(
        !scratch
            .path("project/htmlspec/changes/first/assets")
            .exists(),
        "a store was written beside the document instead of in the project"
    );
}

/// `new change` is unchanged: its subject is a project, so project-bootstrap's "Commands
/// resolve the project from any directory beneath it" has it search from the directory it
/// runs in. Run from a subdirectory, the change lands in the project.
#[test]
fn new_change_from_a_subdirectory_lands_in_the_project() {
    let scratch = Scratch::new("new-change-subdirectory");
    assert_eq!(run_in(&scratch.0, &["init", "--tools", "none"]).0, SUCCESS);
    let (code, out, err) = run_in(
        &scratch.path("htmlspec/specs"),
        &["new", "change", "from-below"],
    );
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(
        scratch
            .path("htmlspec/changes/from-below/proposal.html")
            .exists(),
        "the change did not land in the project's changes"
    );
    assert!(
        !scratch.path("htmlspec/specs/htmlspec").exists(),
        "a second project was created"
    );
}

// -----------------------------------------------------------------------------
// Help and version
// -----------------------------------------------------------------------------

#[test]
fn help_lists_every_subcommand_and_exits_zero() {
    let (code, out, _) = run(&["--help"]);
    assert_eq!(code, SUCCESS);
    for subcommand in ["validate", "format", "init"] {
        assert!(out.contains(subcommand), "{out}");
    }
    assert!(out.contains("--json"), "{out}");
}

/// The version identifies the revision the binary was built from, so a diagnostic
/// can be traced to a specific build.
#[test]
fn the_version_identifies_the_build() {
    let (code, out, _) = run(&["--version"]);
    assert_eq!(code, SUCCESS);
    assert!(out.contains(env!("CARGO_PKG_VERSION")), "{out}");
    // Either a revision or an explicit statement that it is unknown; never silence,
    // which would leave a reader unable to tell an old binary from an untraceable one.
    assert!(
        out.contains('(') && out.contains(')'),
        "the version does not report a revision: {out}"
    );
}

/// `-v` and `-V` print exactly what `--version` prints, so a person's first guess at the flag
/// works.
#[test]
fn every_spelling_of_the_version_flag_prints_the_version() {
    let (code, expected, _) = run(&["--version"]);
    assert_eq!(code, SUCCESS);
    for spelling in ["-v", "-V"] {
        let (code, out, err) = run(&[spelling]);
        assert_eq!(code, SUCCESS, "`htmlspec {spelling}` failed: {err}");
        assert_eq!(
            out, expected,
            "`htmlspec {spelling}` printed something else"
        );
    }
}

/// Every subcommand's help is reachable and documents its own options.
#[test]
fn each_subcommands_help_is_reachable() {
    for (subcommand, options) in [
        ("validate", vec!["--strict", "--json"]),
        ("format", vec!["--check", "--json"]),
        ("init", vec!["--tools", "--root", "--name", "--json"]),
        ("new", vec!["change", "document"]),
    ] {
        let (code, out, _) = run(&[subcommand, "--help"]);
        assert_eq!(code, SUCCESS, "{subcommand} --help failed");
        for option in options {
            assert!(
                out.contains(option),
                "{subcommand} --help omits {option}:\n{out}"
            );
        }
    }
}

/// The subcommand and option spellings are the contract, not an implementer's choice.
///
/// Every plausible adopter arrives from OpenSpec, so an operation OpenSpec also provides
/// carries OpenSpec's name for it. This is asserted rather than intended because the
/// workflow instructions this program installs invoke these commands **by name**: a
/// subcommand renamed to something equally reasonable — `changes` for `list`, `ls` for
/// `list` — leaves every installed instruction naming a command that does not exist, and
/// nothing else in the suite would notice.
///
/// `format` and `assets` are here to pin the other half of the rule: OpenSpec has no
/// counterpart for either, so they take this project's own names and must not be renamed
/// toward OpenSpec's vocabulary either.
#[test]
fn the_command_surface_uses_openspecs_names() {
    let (code, out, err) = run(&["--help"]);
    assert_eq!(code, SUCCESS, "{err}");

    // Parsed out of the command column rather than searched for in the whole help text.
    // `contains("instructions")` passes on the description "Print the workflow instructions
    // for a stage" even after the subcommand is renamed, which would make this test agree
    // with a surface that had already broken every installed instruction.
    let offered: Vec<&str> = out
        .lines()
        .skip_while(|line| !line.starts_with("Commands:"))
        .skip(1)
        .take_while(|line| line.starts_with("  ") || line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert!(
        offered.contains(&"validate"),
        "no command column was parsed out of the help: {out}"
    );

    // Named by OpenSpec, and spelled as OpenSpec spells them.
    for shared in [
        "validate",
        "archive",
        "init",
        "new",
        "list",
        "status",
        "sync",
        "show",
        "instructions",
        "store",
    ] {
        assert!(
            offered.contains(&shared),
            "the surface does not offer `{shared}`, which is OpenSpec's name for an \
             operation this program provides. Offered: {offered:?}"
        );
    }

    // No OpenSpec counterpart, so these keep this project's names.
    for own in ["format", "assets"] {
        assert!(
            offered.contains(&own),
            "the surface no longer offers `{own}`. Offered: {offered:?}"
        );
    }

    // The selectors carry OpenSpec's spellings too. Checked against each subcommand's own
    // help, because a global flag and a per-subcommand flag read the same way here and the
    // contract fixes the spelling rather than the scope.
    for (subcommand, option) in [
        ("list", "--specs"),
        ("status", "--change"),
        ("list", "--json"),
        ("list", "--store"),
        ("show", "--diff"),
    ] {
        let (code, help, err) = run(&[subcommand, "--help"]);
        assert_eq!(code, SUCCESS, "{err}");
        assert!(
            help.contains(option),
            "`{subcommand}` does not offer `{option}`, which is OpenSpec's spelling"
        );
    }
}

// -----------------------------------------------------------------------------
// A reference across a change's documents
//
// document-schema's "Cross-references address units by identifier": a `data-refs` entry in one of
// a change's documents resolves in that document, then in the change's specification, then in the
// accepted specifications. Each project here is built with `init` and `new`, as a contributor
// builds one.
// -----------------------------------------------------------------------------

/// A project holding one change, `a-change`, created by the program.
fn a_project_with_a_change(label: &str) -> Scratch {
    let scratch = a_project(label);
    let (code, out, err) = run_in(&scratch.0, &["new", "change", "a-change"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    scratch
}

/// A specification at `relative`, written by `new document`, with every example identifier the
/// template writes renamed to end in `suffix`, so `req-example` becomes `req-{suffix}`.
///
/// Where `accepted`, every `data-change` is removed, as an accepted specification states its units.
fn a_specification(scratch: &Scratch, relative: &str, suffix: &str, accepted: bool) -> PathBuf {
    let (code, out, err) = run_in(
        &scratch.0,
        &[
            "new",
            "document",
            "specification",
            relative,
            "--subject",
            "A specification",
        ],
    );
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let path = scratch.path(relative);
    let mut text = std::fs::read_to_string(&path)
        .expect("the specification reads")
        .replace("-example", &format!("-{suffix}"));
    if accepted {
        text = text.replace(" data-change=\"added\"", "");
    }
    std::fs::write(&path, text).expect("the specification is written");
    path
}

/// Gives the unit `id` in the document at `path` the references `refs`.
fn refer(path: &Path, id: &str, refs: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let opening = format!("id=\"{id}\"");
    assert_eq!(
        text.matches(&opening).count(),
        1,
        "{id} is not one unit in {path:?}"
    );
    let referred = text.replacen(&opening, &format!("{opening} data-refs=\"{refs}\""), 1);
    std::fs::write(path, referred).expect("the document is written");
}

/// Applies `edit` to the document at `path`, asserting that it changed something.
fn edit(path: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    assert!(text.contains(from), "{from:?} is not in {path:?}");
    std::fs::write(path, text.replacen(from, to, 1)).expect("the document is written");
}

const TASKS: &str = "htmlspec/changes/a-change/tasks.html";
const CHANGE_SPECIFICATION: &str = "htmlspec/changes/a-change/specification.html";
const ACCEPTED: &str = "htmlspec/specs/cap/spec.html";

/// Validates the one document at `relative`, from the project root.
fn validate_one(scratch: &Scratch, relative: &str) -> (i32, String, String) {
    run_in(&scratch.0, &["validate", relative])
}

/// The document-schema scenario "A task references a requirement in its change's specification",
/// which is the tasks template's own advice followed.
#[test]
fn a_task_naming_its_change_specifications_requirement_resolves() {
    let scratch = a_project_with_a_change("refs-change-specification");
    a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    refer(&scratch.path(TASKS), "task-example", "req-x");

    let (code, out, err) = validate_one(&scratch, TASKS);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(!out.contains("refs/"), "{out}");
}

/// The document-schema scenario "A task references a requirement only the accepted contract holds".
#[test]
fn a_task_naming_an_accepted_requirement_resolves() {
    let scratch = a_project_with_a_change("refs-accepted");
    a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    a_specification(&scratch, ACCEPTED, "y", true);
    refer(&scratch.path(TASKS), "task-example", "req-y");

    let (code, out, err) = validate_one(&scratch, TASKS);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(!out.contains("refs/"), "{out}");
}

/// The document-validation scenario "A design references its change's specification".
#[test]
fn a_decision_in_the_design_naming_its_change_specification_resolves() {
    let scratch = a_project_with_a_change("refs-design");
    a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    let design = scratch.path("htmlspec/changes/a-change/design.html");
    edit(
        &design,
        "</main>",
        "<section class=\"decision\" id=\"decision-in-the-design\" data-change=\"added\" \
         data-status=\"proposed\" data-refs=\"req-x\"><h3>A decision \
         <span class=\"hs-id\">decision-in-the-design</span></h3><p>The design decides this.</p>\
         <p data-field=\"rationale\">Because the requirement asks it.</p></section></main>",
    );

    let (code, out, err) = validate_one(&scratch, "htmlspec/changes/a-change/design.html");
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(!out.contains("refs/"), "{out}");
}

/// The document-schema scenario "Accepted specifications reference one another".
#[test]
fn an_accepted_specification_naming_anothers_requirement_resolves() {
    let scratch = a_project("refs-accepted-to-accepted");
    a_specification(&scratch, "htmlspec/specs/a/spec.html", "x", true);
    let b = a_specification(&scratch, "htmlspec/specs/b/spec.html", "y", true);
    edit(
        &b,
        "id=\"evidence-y\" data-refs=\"req-y\"",
        "id=\"evidence-y\" data-refs=\"req-x\"",
    );

    let (code, out, err) = validate_one(&scratch, "htmlspec/specs/b/spec.html");
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(!out.contains("refs/"), "{out}");
}

/// Moves `a-change` into the archive as `archived`, re-expressing its documents' references as
/// archival does.
///
/// Each of the change's references out of it climbs one level more, because the archive is one level
/// deeper. A fixture renamed without that carries references reaching `htmlspec/changes/assets/`,
/// which validation rejects as `asset/unresolved`, and these tests are about what an archived
/// change's references reach by identifier, not about where its store is.
fn archive_by_hand(scratch: &Scratch, archived: &str) {
    let destination = scratch.path(&format!("htmlspec/changes/archive/{archived}"));
    std::fs::create_dir_all(destination.parent().expect("the archive"))
        .expect("the archive exists");
    std::fs::rename(scratch.path("htmlspec/changes/a-change"), &destination)
        .expect("the change moves into the archive");
    for entry in std::fs::read_dir(&destination).expect("the archived change reads") {
        let path = entry.expect("an entry").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "html")
        {
            let source = std::fs::read_to_string(&path).expect("a document");
            std::fs::write(&path, source.replace("\"../../", "\"../../../")).expect("written");
        }
    }
}

/// design decision 6: an archived change reaches what an active one does.
#[test]
fn an_archived_changes_task_naming_its_own_specification_resolves() {
    let scratch = a_project_with_a_change("refs-archived");
    a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    refer(&scratch.path(TASKS), "task-example", "req-x");
    archive_by_hand(&scratch, "2026-10-03-a-change");

    let (code, out, err) = validate_one(
        &scratch,
        "htmlspec/changes/archive/2026-10-03-a-change/tasks.html",
    );
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(!out.contains("refs/"), "{out}");
}

/// The document-schema scenario "A change withdraws a requirement the accepted contract still
/// holds": the change's specification is reached first, and it withdraws the requirement.
#[test]
fn a_task_naming_a_requirement_its_change_withdraws_is_a_removed_target() {
    let scratch = a_project_with_a_change("refs-withdrawn");
    a_specification(&scratch, ACCEPTED, "y", true);
    let specification = a_specification(&scratch, CHANGE_SPECIFICATION, "y", false);
    edit(
        &specification,
        "id=\"req-y\" data-change=\"added\"",
        "id=\"req-y\" data-change=\"removed\"",
    );
    refer(&scratch.path(TASKS), "task-example", "req-y");

    let (code, out, err) = validate_one(&scratch, TASKS);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(out.contains("refs/removed-target"), "{out}");
    assert!(
        !out.contains("refs/unresolved"),
        "the withdrawn requirement was reported as unresolved: {out}"
    );
}

/// The document-validation scenario "A reference resolved by the change's specification", and
/// toolchain-packaging's "Validating one document": the specification is read for what it
/// declares, and nothing is reported about it.
#[test]
fn validating_one_document_reports_nothing_about_the_documents_it_reaches() {
    let scratch = a_project_with_a_change("refs-one-document");
    let specification = a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    edit(
        &specification,
        "<section class=\"requirement\" id=\"req-x\"",
        "<section class=\"requirement\" id=\"req-without-a-scenario\" data-change=\"added\">\
         <h3>A requirement with no scenario</h3><p>The system SHALL do one more thing.</p>\
         </section><section class=\"requirement\" id=\"req-x\"",
    );
    let (code, _, _) = validate_one(&scratch, CHANGE_SPECIFICATION);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "the specification is meant to be invalid"
    );
    refer(&scratch.path(TASKS), "task-example", "req-x");

    let (code, out, err) = validate_one(&scratch, TASKS);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(
        !out.contains("specification.html"),
        "validating the tasks document reported on the specification: {out}"
    );
}

/// The document-validation scenario "A reachable document that cannot be read".
#[test]
fn a_specification_that_needs_recovery_is_named_as_unread() {
    let scratch = a_project_with_a_change("refs-unread");
    let specification = a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    edit(
        &specification,
        "<p>Replace this paragraph",
        "<p><strong>Replace this paragraph",
    );
    refer(&scratch.path(TASKS), "task-example", "req-x");

    let (code, out, err) = validate_one(&scratch, TASKS);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    let unresolved = out
        .lines()
        .find(|line| line.contains("refs/unresolved"))
        .unwrap_or_else(|| panic!("no refs/unresolved: {out}"));
    assert!(
        unresolved.contains("specification.html"),
        "the report does not name the specification that could not be read: {unresolved}"
    );
}

/// The document-validation scenario "A change with no specification yet".
#[test]
fn a_change_with_no_specification_reaches_the_accepted_contract() {
    let scratch = a_project_with_a_change("refs-no-specification");
    a_specification(&scratch, ACCEPTED, "y", true);
    refer(&scratch.path(TASKS), "task-example", "req-y");

    let (code, out, err) = validate_one(&scratch, TASKS);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    // Nothing about the missing specification: not named, and not called unread. The template's
    // own example-content warning mentions a change's specification in general, so the check is on
    // the file and on the unread report, not on the word.
    assert!(!out.contains("refs/"), "{out}");
    assert!(
        !out.contains("specification.html") && !out.contains("could not be read"),
        "the missing specification was reported: {out}"
    );
}

// The four below pass before this change and after it: what a reference does not reach.

/// The document-schema scenario "A reference to another change".
#[test]
fn a_task_naming_another_changes_requirement_is_unresolved() {
    let scratch = a_project_with_a_change("refs-another-change");
    let (code, out, err) = run_in(&scratch.0, &["new", "change", "b-change"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    a_specification(
        &scratch,
        "htmlspec/changes/b-change/specification.html",
        "x",
        false,
    );
    refer(&scratch.path(TASKS), "task-example", "req-x");

    let (code, out, err) = validate_one(&scratch, TASKS);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(out.contains("refs/unresolved"), "{out}");
}

/// The document-schema scenario "A fragment link to a unit in another document".
#[test]
fn a_fragment_link_to_the_change_specification_is_unresolved_and_broken() {
    let scratch = a_project_with_a_change("refs-fragment");
    a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    edit(
        &scratch.path(TASKS),
        "<p data-field=\"note\">",
        "<p data-field=\"note\">Serves <a href=\"#req-x\">the requirement</a>. ",
    );

    let (code, out, err) = validate_one(&scratch, TASKS);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(out.contains("refs/unresolved"), "{out}");
    assert!(out.contains("refs/broken-link"), "{out}");
}

/// The document-schema scenario "A document that reaches only itself", for both cases it names.
#[test]
fn a_loose_document_reaches_only_itself() {
    // Outside any project, beside a document that declares the identifier.
    let loose = Scratch::new("refs-loose");
    loose.write("other.html", &valid().replace("-example", "-elsewhere"));
    let alone = loose.write(
        "alone.html",
        &valid().replace(
            "id=\"evidence-example\" data-change=\"added\" data-refs=\"req-example\"",
            "id=\"evidence-example\" data-change=\"added\" data-refs=\"req-elsewhere\"",
        ),
    );
    let (code, out, err) = run(&["validate", &as_str(&alone)]);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(out.contains("refs/unresolved"), "{out}");

    // Beneath a change's evidence, named on the command line, beside the change's specification.
    let scratch = a_project_with_a_change("refs-evidence");
    a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    let record = scratch.write(
        "htmlspec/changes/a-change/evidence/record.html",
        &valid().replace(
            "id=\"evidence-example\" data-change=\"added\" data-refs=\"req-example\"",
            "id=\"evidence-example\" data-change=\"added\" data-refs=\"req-x\"",
        ),
    );
    let (code, out, err) = run(&["validate", &as_str(&record)]);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(out.contains("refs/unresolved"), "{out}");
}

/// The document-schema scenario "A runbook's document names an accepted requirement".
#[test]
fn a_runbooks_task_naming_an_accepted_requirement_is_unresolved() {
    let scratch = a_project("refs-runbook");
    a_specification(&scratch, ACCEPTED, "y", true);
    let tasks = "htmlspec/runbooks/release/tasks.html";
    let (code, out, err) = run_in(
        &scratch.0,
        &["new", "document", "tasks", tasks, "--subject", "A runbook"],
    );
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    refer(&scratch.path(tasks), "task-example", "req-y");

    let (code, out, err) = validate_one(&scratch, tasks);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(out.contains("refs/unresolved"), "{out}");
}

/// A document named by its bare file name, from its own directory, is placed where it sits.
///
/// Found by the code review of PR #43. `Path::parent` of a bare file name is the empty path, which
/// the placement read as no directory at all, so `validate tasks.html` inside a change was loose and
/// `validate ./tasks.html` was not: the result depended on how the path was written.
#[test]
fn a_document_named_by_its_bare_file_name_is_placed_where_it_sits() {
    let scratch = a_project_with_a_change("refs-bare-name");
    a_specification(&scratch, CHANGE_SPECIFICATION, "x", false);
    refer(&scratch.path(TASKS), "task-example", "req-x");
    let change = scratch.path("htmlspec/changes/a-change");
    for spelled in ["tasks.html", "./tasks.html"] {
        let (code, out, err) = run_in(&change, &["validate", spelled]);
        assert_eq!(code, SUCCESS, "{spelled}\nstdout: {out}\nstderr: {err}");
        assert!(!out.contains("refs/"), "{spelled}: {out}");
    }

    // An accepted specification named the same way still reaches the others.
    a_specification(&scratch, "htmlspec/specs/a/spec.html", "a", true);
    let b = a_specification(&scratch, "htmlspec/specs/b/spec.html", "b", true);
    edit(
        &b,
        "id=\"evidence-b\" data-refs=\"req-b\"",
        "id=\"evidence-b\" data-refs=\"req-a\"",
    );
    let (code, out, err) = run_in(
        &scratch.path("htmlspec/specs/b"),
        &["validate", "spec.html"],
    );
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(!out.contains("refs/"), "{out}");
}

// -----------------------------------------------------------------------------
// Change markup in a change's specification
//
// document-validation's "Change markup in or around a semantic unit is rejected" permits `ins` and
// `del` in a modified unit of a change's specification, active or archived, and nowhere else. Where
// a document sits is derived from its path, so these tests put one document's bytes in each place
// and check the verdict end to end.
// -----------------------------------------------------------------------------

/// The units of the specification the measurement wrote: two requirements marked `modified`, one
/// with `sync` inserted into a list and one with a word deleted beside an inserted phrase, each with
/// a scenario carried forward and a verification naming it.
const MARKED_UNITS: &str = r#"
        <section id="hs-requirements">
          <h2>Requirements</h2>
          <section class="requirement" id="req-a-single-command-exposes-validate-format-and-init" data-change="modified">
            <h3>The command surface names its operations as OpenSpec names them <span class="hs-id">req-a-single-command-exposes-validate-format-and-init</span></h3>
            <p><strong>Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it.</strong> This holds for the subcommand and for the options that select what it acts on.</p>
            <p>The spellings are therefore fixed rather than left to an implementer: <code>validate</code>, <code>archive</code>, <code>init</code>, <code>new</code>, <code>list</code>, <code>status</code>, <ins><code>sync</code>, </ins><code>instructions</code>, and <code>store</code> name the operations OpenSpec also provides. <code>format</code> and <code>assets</code> name operations it does not.</p>
            <article class="scenario" id="scenario-the-spellings-are-found" data-change="unchanged">
              <h4>The spellings are found</h4>
              <dl>
                <dt>When</dt>
                <dd>A contributor runs the operation.</dd>
                <dt>Then</dt>
                <dd>It is found under the spelling the requirement fixes.</dd>
              </dl>
            </article>
          </section>
          <section class="requirement" id="req-a-project-has-a-declared-root-holding-one-asset-store" data-change="modified">
            <h3>A project has a declared root holding one asset store <span class="hs-id">req-a-project-has-a-declared-root-holding-one-asset-store</span></h3>
            <p>An HTMLSpec project SHALL be identified by a directory containing an <code>htmlspec/</code> directory. That directory SHALL hold the project's accepted specifications, its active changes, <del>and </del>its archive, <ins>and its runbooks, </ins>each in its own place.</p>
            <article class="scenario" id="scenario-the-places-are-found" data-change="unchanged">
              <h4>The places are found</h4>
              <dl>
                <dt>When</dt>
                <dd>A contributor runs the operation.</dd>
                <dt>Then</dt>
                <dd>It is found under the spelling the requirement fixes.</dd>
              </dl>
            </article>
          </section>
        </section>
        <section id="hs-verification">
          <h2>Verification</h2>
          <section class="verification" id="verify-the-spellings" data-change="added" data-outcome="not-run" data-refs="req-a-single-command-exposes-validate-format-and-init">
            <h3>Checked by reading the command list <span class="hs-id">verify-the-spellings</span></h3>
            <p data-field="method">List the commands and compare each with the fixed spellings.</p>
          </section>
          <section class="verification" id="verify-the-places" data-change="added" data-outcome="not-run" data-refs="req-a-project-has-a-declared-root-holding-one-asset-store">
            <h3>Checked by reading the command list <span class="hs-id">verify-the-places</span></h3>
            <p data-field="method">List the commands and compare each with the fixed spellings.</p>
          </section>
        </section>
      "#;

/// A specification at `relative`, written by `new document` and given the measurement's units,
/// summary, and contents list, as `the-measurement-2026-09-30.md`'s appendix A writes it.
fn a_marked_specification(scratch: &Scratch, relative: &str) -> PathBuf {
    let (code, out, err) = run_in(
        &scratch.0,
        &[
            "new",
            "document",
            "specification",
            relative,
            "--subject",
            "measure-marked-words: specification",
        ],
    );
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let path = scratch.path(relative);
    let text = std::fs::read_to_string(&path).expect("the specification reads");

    let open = "<main class=\"hs-main\" id=\"hs-content\">";
    let start = text.find(open).expect("the template has its main") + open.len();
    let end = text.find("</main>").expect("the main closes");
    let mut text = format!("{}{MARKED_UNITS}{}", &text[..start], &text[end..]);

    let placeholder = "<p>Replace this paragraph with what this document is for.";
    let start = text
        .find(placeholder)
        .expect("the template has its summary");
    let end = start + text[start..].find("</p>").expect("the summary closes") + "</p>".len();
    text.replace_range(
        start..end,
        "<p>A scratch specification for measuring how change markup projects today.</p>",
    );

    let text: String = text
        .split_inclusive('\n')
        .filter(|line| {
            !["#hs-decisions", "#hs-risks", "#hs-evidence"]
                .iter()
                .any(|section| line.contains(&format!("<li><a href=\"{section}\">")))
        })
        .collect();
    std::fs::write(&path, text).expect("the specification is written");
    path
}

/// Whether a run reported `schema/change-markup`.
fn reports_change_markup(out: &str, err: &str) -> bool {
    out.contains("schema/change-markup") || err.contains("schema/change-markup")
}

/// The scenario "A word and a paragraph deleted from a modified requirement", end to end: the
/// specification of an active change validates, with nothing reported for its marks.
#[test]
fn marks_in_an_active_changes_specification_are_accepted() {
    let scratch = a_project_with_a_change("marks-active");
    a_marked_specification(&scratch, CHANGE_SPECIFICATION);

    let (code, out, err) = validate_one(&scratch, CHANGE_SPECIFICATION);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(!reports_change_markup(&out, &err), "{out}");
}

/// The scenario "An archived change keeps its marks": an archived change's specification is still
/// a change's specification.
#[test]
fn marks_in_an_archived_changes_specification_are_accepted() {
    let scratch = a_project_with_a_change("marks-archived");
    a_marked_specification(&scratch, CHANGE_SPECIFICATION);
    archive_by_hand(&scratch, "2026-10-05-a-change");

    let (code, out, err) = validate_one(
        &scratch,
        "htmlspec/changes/archive/2026-10-05-a-change/specification.html",
    );
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    assert!(!reports_change_markup(&out, &err), "{out}");
}

/// `sync` validates the change's specification before comparing it, and its marks do not refuse
/// the comparison.
#[test]
fn sync_compares_a_specification_carrying_marks() {
    let scratch = a_project_with_a_change("marks-sync");
    a_marked_specification(&scratch, CHANGE_SPECIFICATION);

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_ne!(code, OPERATIONAL_FAILURE, "stdout: {out}\nstderr: {err}");
    assert!(
        !reports_change_markup(&out, &err),
        "stdout: {out}\nstderr: {err}"
    );
    assert!(
        !err.contains("does not run while the specification reports an error"),
        "stdout: {out}\nstderr: {err}"
    );
}

/// `sync` refuses a specification that does not validate, with the code for a document violation.
///
/// toolchain-packaging's "The command surface reports whether a change reached the accepted
/// contract" requires the refusal. `show-what-a-modification-changed` moved it into the loading
/// that `sync` and `show --diff` share, and its mutation testing found that no test of `sync` held
/// it: dropping the refusal failed only the view's own test.
#[test]
fn sync_refuses_a_specification_that_does_not_validate() {
    let scratch = a_project_with_a_change("sync-invalid");
    let path = a_marked_specification(&scratch, CHANGE_SPECIFICATION);
    edit(
        &path,
        "data-change=\"modified\"",
        "data-change=\"sideways\"",
    );

    let (code, out, err) = run_in(&scratch.0, &["sync", "--change", "a-change"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(
        err.contains("does not run while the specification reports an error"),
        "stdout: {out}\nstderr: {err}"
    );
    assert!(
        !out.contains("reached the accepted specifications"),
        "{out}"
    );
}

/// Archival's gate validates the change's specification, and its marks do not refuse the change.
#[test]
fn archival_does_not_refuse_a_specification_for_its_marks() {
    let scratch = a_project_with_a_change("marks-archive");
    a_marked_specification(&scratch, CHANGE_SPECIFICATION);
    // Archival compares the declared version with the last archived one before it reads the
    // specification, so the version is advanced as a contributor advances it.
    let declaration = scratch.path("spec.nix");
    let declared = std::fs::read_to_string(&declaration).expect("the declaration reads");
    std::fs::write(&declaration, declared.replace("0.0.0", "0.1.0"))
        .expect("the declaration is written");

    let (_code, out, err) = run_in(&scratch.0, &["archive", "a-change", "--check"]);
    assert!(
        out.contains("specification.html"),
        "archival stopped before it read the specification\nstdout: {out}\nstderr: {err}"
    );
    assert!(
        !reports_change_markup(&out, &err),
        "stdout: {out}\nstderr: {err}"
    );
    assert!(
        !err.contains("does not proceed while the specification reports an error"),
        "stdout: {out}\nstderr: {err}"
    );
}

/// The scenario "Markup left in the accepted contract": the same bytes as an accepted
/// specification are refused, one error for each of the three elements.
#[test]
fn marks_in_an_accepted_specification_are_refused() {
    let scratch = a_project_with_a_change("marks-accepted");
    a_marked_specification(&scratch, ACCEPTED);

    let (code, out, err) = validate_one(&scratch, ACCEPTED);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert_eq!(
        out.matches("error: schema/change-markup").count(),
        3,
        "{out}"
    );
}

/// The same bytes as a change's proposal are refused: only its specification restates accepted
/// text.
#[test]
fn marks_in_a_changes_proposal_are_refused() {
    let scratch = a_project_with_a_change("marks-proposal");
    let specification = a_marked_specification(&scratch, CHANGE_SPECIFICATION);
    let proposal = "htmlspec/changes/a-change/proposal.html";
    std::fs::copy(&specification, scratch.path(proposal)).expect("the bytes are copied");
    std::fs::remove_file(&specification).expect("the specification is removed");

    let (code, out, err) = validate_one(&scratch, proposal);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(reports_change_markup(&out, &err), "{out}");
}

/// The same bytes beneath a change's evidence, named on the command line, are refused: a file
/// there is placed loose.
#[test]
fn marks_beneath_a_changes_evidence_are_refused() {
    let scratch = a_project_with_a_change("marks-evidence");
    let specification = a_marked_specification(&scratch, CHANGE_SPECIFICATION);
    let evidence = "htmlspec/changes/a-change/evidence/specification.html";
    std::fs::create_dir_all(scratch.path("htmlspec/changes/a-change/evidence"))
        .expect("the evidence directory exists");
    std::fs::copy(&specification, scratch.path(evidence)).expect("the bytes are copied");

    let (code, out, err) = validate_one(&scratch, evidence);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(reports_change_markup(&out, &err), "{out}");
}

/// The same bytes outside any project are refused: a document there sits in no change.
#[test]
fn marks_outside_any_project_are_refused() {
    let scratch = a_project_with_a_change("marks-loose");
    let specification = a_marked_specification(&scratch, CHANGE_SPECIFICATION);
    let elsewhere = Scratch::new("marks-loose-elsewhere");
    let loose = elsewhere.path("specification.html");
    std::fs::copy(&specification, &loose).expect("the bytes are copied");

    let (code, out, err) = run_in(&elsewhere.0, &["validate", "specification.html"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(reports_change_markup(&out, &err), "{out}");
}
