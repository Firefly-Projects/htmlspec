//! Creating a runbook, finding the runbooks a project holds, and moving documents onto the store's
//! current assets.
//!
//! The runbooks workstream of `ship-the-crates-notices-and-runbook-affordances`: its design's
//! decisions 7 (`new runbook`), 8 (`format --upgrade-assets`), and 18 (the runbooks are easy to find
//! from the commands that use them). Driven through the built binary, because what is under test is
//! files appearing where they should, a refusal's exit status, and what a person reads.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The exit codes the program contracts to, restated for the reason `cli.rs` restates them.
const SUCCESS: i32 = 0;
const DOCUMENT_VIOLATION: i32 = 1;
const OPERATIONAL_FAILURE: i32 = 2;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// Runs the binary from inside `directory`, holding any payload it prints to its schema.
fn run_in(directory: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    support::held(arguments, &stdout);
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
            std::env::temp_dir().join(format!("htmlspec-runbooks-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }

    fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.path(relative))
            .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
    }

    /// An initialized project, with no instructions installed.
    fn project(label: &str) -> Self {
        let scratch = Self::new(label);
        let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "none"]);
        assert_eq!(code, SUCCESS, "init failed:\n{out}{err}");
        scratch
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Every value `attribute` takes in `source`, in document order.
fn attribute_values<'s>(source: &'s str, attribute: &str) -> Vec<&'s str> {
    let opening = format!("{attribute}=\"");
    source
        .match_indices(&opening)
        .filter_map(|(at, _)| {
            let rest = &source[at + opening.len()..];
            rest.find('"').map(|end| &rest[..end])
        })
        .collect()
}

// -----------------------------------------------------------------------------
// `new runbook`
// -----------------------------------------------------------------------------

/// What `new runbook` writes is a runbook a run can start from at once: two documents that validate
/// and are canonical, a `VERSION` at 0.1.0 with its changelog heading, a preflight with zero checks,
/// and a listing that names it.
#[test]
fn new_runbook_writes_a_runbook_that_validates_and_can_start() {
    let scratch = Scratch::project("new-runbook");
    let (code, out, err) = run_in(
        &scratch.0,
        &[
            "new",
            "runbook",
            "deploy",
            "--subject",
            "Deploying the site",
        ],
    );
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        out.contains("created a runbook at 0.1.0") && out.contains("htmlspec run deploy --as"),
        "the report does not say what was made and how to run it:\n{out}"
    );

    let directory = "htmlspec/runbooks/deploy";
    assert_eq!(scratch.read(&format!("{directory}/VERSION")), "0.1.0\n");
    assert!(
        scratch
            .read(&format!("{directory}/CHANGELOG.md"))
            .contains("\n## 0.1.0\n"),
        "the changelog has no heading for the version"
    );
    for file in ["specification.html", "tasks.html"] {
        assert!(
            scratch
                .read(&format!("{directory}/{file}"))
                .contains("<title>Deploying the site</title>"),
            "{file} does not take the subject"
        );
    }

    for arguments in [
        &["validate", directory][..],
        &["format", "--check", directory],
        &["preflight", "deploy"],
    ] {
        let (code, out, err) = run_in(&scratch.0, arguments);
        assert_eq!(code, SUCCESS, "{arguments:?}:\n{out}{err}");
    }
    let (_, out, _) = run_in(&scratch.0, &["preflight", "deploy"]);
    assert!(
        out.contains("declares no machine check"),
        "preflight reports checks the template declared:\n{out}"
    );

    let (code, out, err) = run_in(&scratch.0, &["list", "--runbooks"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        out.lines().any(|row| row.starts_with("deploy ")
            && row.contains("0.1.0")
            && row.contains("Deploying the site")),
        "the listing does not name the runbook with its version and title:\n{out}"
    );

    // And a run starts from it: the procedure, its figure included, copies into a record.
    let (code, out, err) = run_in(&scratch.0, &["run", "deploy", "--as", "first"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let (code, out, err) = run_in(&scratch.0, &["validate", "htmlspec/runbooks/deploy/runs"]);
    assert_eq!(code, SUCCESS, "the record does not validate:\n{out}{err}");
}

/// Every unit `new runbook` writes is `unchanged` and every task `todo`.
///
/// runbook-authority's "The canonical procedure carries no completion state", which `new runbook`
/// is the second exception to beside starting a run. Validation does not catch a runbook written
/// `added`, so this is what does.
#[test]
fn new_runbook_writes_no_completion_state() {
    let scratch = Scratch::project("new-runbook-state");
    let (code, out, err) = run_in(&scratch.0, &["post", "runbook", "deploy"]);
    assert_eq!(
        code, SUCCESS,
        "the request-verb spelling fails:\n{out}{err}"
    );

    for file in ["specification.html", "tasks.html"] {
        let source = scratch.read(&format!("htmlspec/runbooks/deploy/{file}"));
        let changes = attribute_values(&source, "data-change");
        assert!(!changes.is_empty(), "{file} carries no unit");
        assert!(
            changes.iter().all(|change| *change == "unchanged"),
            "{file} carries a change state other than unchanged: {changes:?}"
        );
        let states = attribute_values(&source, "data-state");
        assert!(
            states.iter().all(|state| *state == "todo"),
            "{file} carries a completion state: {states:?}"
        );
    }
    let tasks = scratch.read("htmlspec/runbooks/deploy/tasks.html");
    assert_eq!(attribute_values(&tasks, "data-state").len(), 2, "two steps");
    assert!(
        tasks.contains("class=\"mermaid\""),
        "the tasks document draws no order"
    );
}

/// A fresh runbook still carries its templates' example content, and validation says so.
///
/// contract-authority's "Generated example content is non-normative and its survival is reported":
/// both scans read the runbook templates, so the example identifiers and the placeholder summaries
/// are reported in each document.
#[test]
fn a_fresh_runbook_reports_its_example_content() {
    let scratch = Scratch::project("new-runbook-examples");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "deploy"]);
    assert_eq!(code, SUCCESS, "{out}{err}");

    for (file, identifier) in [
        ("specification.html", "req-example"),
        ("tasks.html", "task-first-step-example"),
    ] {
        let (code, out, err) = run_in(
            &scratch.0,
            &["validate", &format!("htmlspec/runbooks/deploy/{file}")],
        );
        assert_eq!(code, SUCCESS, "{file}: {out}{err}");
        assert!(
            out.contains(&format!("review/example-content at {identifier}")),
            "{file}'s example identifier is not reported:\n{out}"
        );
        assert!(
            out.contains("review/example-content at document")
                && out.contains("summary is still the paragraph a template writes"),
            "{file}'s placeholder summary is not reported:\n{out}"
        );
    }
}

/// A name that is not a plain name is refused before anything is written.
#[test]
fn new_runbook_refuses_a_name_that_is_not_a_plain_name() {
    let scratch = Scratch::project("new-runbook-names");
    // Beside the shapes of a path: the names the project's layout already gives a meaning. A runbook
    // named `assets` would be taken for every other runbook's store, which PR #80's review measured.
    for given in [
        "",
        ".",
        "..",
        "a/b",
        "a\\b",
        "a..b",
        "a\u{1}b",
        "de\u{202E}ploy",
        "de\u{200B}ploy",
        "de\u{00A0}ploy",
        "assets",
        "target",
        "node_modules",
        "result",
        ".git",
        ".hidden",
    ] {
        let (code, out, err) = run_in(&scratch.0, &["new", "runbook", given]);
        assert_eq!(code, OPERATIONAL_FAILURE, "{given:?}: {out}{err}");
        assert!(
            err.contains("is a path rather than a runbook name")
                || err.contains("is not a name a new runbook can take"),
            "{given:?} was not refused as a name:\n{err}"
        );
    }
    let left: Vec<_> = std::fs::read_dir(scratch.path("htmlspec/runbooks"))
        .map(|entries| entries.flatten().map(|entry| entry.file_name()).collect())
        .unwrap_or_default();
    assert!(left.is_empty(), "a refused name left {left:?} behind");
}

/// A runbook named as 1.0 let a runbook be named still runs: the stricter rule is `new runbook`'s
/// alone, and toolchain-packaging's "The command surface is stable within a major version" keeps
/// every runbook 1.0 ran runnable.
#[test]
fn a_runbook_1_0_ran_still_runs_whatever_its_name() {
    let scratch = Scratch::project("names-1-0-ran");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "base"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    for name in ["result", "target", ".ops", "v1..2"] {
        copy_directory(
            &scratch.path("htmlspec/runbooks/base"),
            &scratch.path(&format!("htmlspec/runbooks/{name}")),
        );
        for arguments in [
            &["preflight", name][..],
            &["run", name, "--check"],
            &["run", name, "--as", "first"],
        ] {
            let (code, out, err) = run_in(&scratch.0, arguments);
            assert_eq!(code, SUCCESS, "{arguments:?}:\n{out}{err}");
        }
    }
}

/// A link already at the runbook's path, dangling or not, is refused and left where it is.
#[cfg(unix)]
#[test]
fn new_runbook_refuses_a_link_at_its_path_and_leaves_it() {
    let scratch = Scratch::project("new-runbook-links");
    let file = scratch.path("a-file");
    std::fs::write(&file, "a file").expect("the file");
    for (name, target) in [
        ("dangling", scratch.path("nowhere")),
        ("to-a-file", file.clone()),
    ] {
        let link = scratch.path(&format!("htmlspec/runbooks/{name}"));
        std::os::unix::fs::symlink(&target, &link).expect("the link");
        let (code, _, err) = run_in(&scratch.0, &["new", "runbook", name]);
        assert_eq!(code, OPERATIONAL_FAILURE, "{name}: {err}");
        assert!(err.contains("already exists"), "{name}: {err}");
        assert!(
            std::fs::symlink_metadata(&link)
                .expect("the link is still there")
                .file_type()
                .is_symlink(),
            "{name}: the link was replaced or removed"
        );
    }
    assert_eq!(std::fs::read_to_string(&file).expect("readable"), "a file");
}

/// Copies a directory's files and subdirectories.
fn copy_directory(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the copy's directory");
    for entry in std::fs::read_dir(from).expect("the directory reads") {
        let path = entry.expect("the entry reads").path();
        let target = to.join(path.file_name().expect("a name"));
        if path.is_dir() {
            copy_directory(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("the file is copied");
        }
    }
}

/// An existing runbook is never written into, and a runbook needs a project.
#[test]
fn new_runbook_refuses_an_existing_runbook_and_a_missing_project() {
    let scratch = Scratch::project("new-runbook-twice");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "deploy"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let before = scratch.read("htmlspec/runbooks/deploy/tasks.html");
    let (code, _, err) = run_in(&scratch.0, &["new", "runbook", "deploy"]);
    assert_eq!(code, OPERATIONAL_FAILURE, "{err}");
    assert!(err.contains("already exists"), "{err}");
    assert_eq!(scratch.read("htmlspec/runbooks/deploy/tasks.html"), before);

    let outside = Scratch::new("new-runbook-outside");
    let (code, _, err) = run_in(&outside.0, &["new", "runbook", "deploy"]);
    assert_eq!(code, OPERATIONAL_FAILURE, "{err}");
    assert!(
        !outside.path("htmlspec").exists(),
        "a runbook was written outside any project"
    );
}

// -----------------------------------------------------------------------------
// Finding the runbooks
// -----------------------------------------------------------------------------

/// `run` named with no runbook lists the runbooks with their versions and titles, says how to start
/// one, and exits 2, whatever else was passed.
#[test]
fn run_alone_lists_the_runbooks_and_exits_two() {
    let scratch = Scratch::project("run-alone");
    for name in ["deploy", "restore"] {
        let (code, out, err) = run_in(&scratch.0, &["new", "runbook", name]);
        assert_eq!(code, SUCCESS, "{out}{err}");
    }

    for arguments in [
        &["run"][..],
        &["run", "--as", "first"],
        &["run", "--check"],
        &["post", "run"],
    ] {
        let (code, out, err) = run_in(&scratch.0, arguments);
        assert_eq!(code, OPERATIONAL_FAILURE, "{arguments:?}: {out}{err}");
        assert!(out.is_empty(), "{arguments:?} wrote to stdout: {out}");
        assert!(
            err.contains("run needs a runbook")
                && err.contains("deploy")
                && err.contains("restore")
                && err.contains("0.1.0")
                && err.contains("htmlspec run <runbook> --as <record>"),
            "{arguments:?} does not list the runbooks and say how to start one:\n{err}"
        );
    }
    assert!(
        !scratch.path("htmlspec/runbooks/deploy/runs").exists(),
        "run with no runbook started one"
    );

    let empty = Scratch::project("run-alone-empty");
    let (code, _, err) = run_in(&empty.0, &["run"]);
    assert_eq!(code, OPERATIONAL_FAILURE, "{err}");
    assert!(
        err.contains("holds none") && err.contains("htmlspec new runbook <name>"),
        "an empty project's refusal does not say how to make one:\n{err}"
    );
}

/// The listing carries each runbook's version and title, and a runbook with no `VERSION` is listed
/// without one rather than with a version nobody released.
#[test]
fn the_listing_names_each_runbooks_version_and_title() {
    let scratch = Scratch::project("listing");
    let (code, out, err) = run_in(
        &scratch.0,
        &[
            "new",
            "runbook",
            "deploy",
            "--subject",
            "Deploying the site",
        ],
    );
    assert_eq!(code, SUCCESS, "{out}{err}");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "older"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    std::fs::remove_file(scratch.path("htmlspec/runbooks/older/VERSION")).expect("removed");

    let (code, out, err) = run_in(&scratch.0, &["--json", "list", "--runbooks"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
    let entry = |name: &str| {
        payload["runbooks"]
            .as_array()
            .expect("an array")
            .iter()
            .find(|entry| entry["name"] == name)
            .cloned()
            .unwrap_or_else(|| panic!("{name} is not listed: {payload}"))
    };
    let deploy = entry("deploy");
    assert_eq!(deploy["version"], "0.1.0", "{deploy}");
    assert_eq!(deploy["title"], "Deploying the site", "{deploy}");
    let older = entry("older");
    assert!(
        older.get("version").is_none(),
        "a runbook with no VERSION is listed with one: {older}"
    );
    assert_eq!(older["title"], "older", "{older}");

    let (code, out, err) = run_in(&scratch.0, &["list", "--runbooks"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        out.lines()
            .any(|row| row.starts_with("older ") && row.contains("no version")),
        "the human listing does not say the runbook has no version:\n{out}"
    );
}

/// `run --help` and `preflight --help` point to the listing.
#[test]
fn run_and_preflight_help_point_to_the_listing() {
    let scratch = Scratch::new("help");
    for command in ["run", "preflight"] {
        let (code, out, err) = run_in(&scratch.0, &[command, "--help"]);
        assert_eq!(code, SUCCESS, "{out}{err}");
        assert!(
            out.contains("htmlspec list --runbooks"),
            "{command} --help does not name the listing:\n{out}"
        );
    }
}

// -----------------------------------------------------------------------------
// `format --upgrade-assets`
// -----------------------------------------------------------------------------

/// The path from the document at `relative` to the project's store, which sits in `htmlspec/`.
fn store_prefix(relative: &str) -> String {
    let depth = relative.matches('/').count() - 1;
    format!("{}assets", "../".repeat(depth))
}

/// The reference the document at `relative` makes to the embedded asset named `name`.
fn current_reference(relative: &str, name: &str) -> String {
    let asset = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == name)
        .expect("an embedded asset");
    format!("{}/{}", store_prefix(relative), asset.path)
}

/// Stores an older revision of the stylesheet, and points the document at `relative` to it, as a
/// document written by an earlier release references it. Returns the older reference.
fn on_an_older_stylesheet(root: &Path, relative: &str) -> String {
    let bytes = b"/* an older revision of the stylesheet */";
    let stored =
        htmlspec_assets::address::stored_path(&htmlspec_assets::sha256_hex(bytes), "htmlspec.css");
    let path = root.join("htmlspec/assets").join(&stored);
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("the address");
    std::fs::write(&path, bytes).expect("the older revision is stored");
    let older = format!("{}/{stored}", store_prefix(relative));

    let document = root.join(relative);
    let source = std::fs::read_to_string(&document).expect("readable");
    let current = current_reference(relative, "htmlspec.css");
    assert!(
        source.contains(&current),
        "{relative} does not name {current}"
    );
    std::fs::write(&document, source.replace(&current, &older)).expect("rewritten");
    older
}

/// A document in the project at `root`, at `relative`, from the specification template.
fn a_document(root: &Path, relative: &str) {
    let (code, out, err) = run_in(
        root,
        &[
            "new",
            "document",
            "specification",
            relative,
            "--subject",
            "Earlier",
        ],
    );
    assert_eq!(code, SUCCESS, "new document failed:\n{out}{err}");
}

/// A document on an older stylesheet stays there without the request, moves onto the current one
/// with it, and a second request changes nothing.
#[test]
fn upgrade_assets_moves_a_document_onto_the_current_stylesheet_only_on_request() {
    let scratch = Scratch::project("upgrade");
    let relative = "htmlspec/specs/earlier.html";
    a_document(&scratch.0, relative);
    let older = on_an_older_stylesheet(&scratch.0, relative);

    let (code, out, err) = run_in(&scratch.0, &["format"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        scratch.read(relative).contains(&older),
        "plain format re-pointed a reference to stored bytes"
    );

    let before = scratch.read(relative);
    let (code, out, _) = run_in(&scratch.0, &["format", "--check", "--upgrade-assets"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}");
    assert!(
        out.contains("earlier.html") && out.contains("behind the store's current revision"),
        "check mode does not name the document that is behind:\n{out}"
    );
    assert_eq!(
        scratch.read(relative),
        before,
        "check mode wrote the document"
    );

    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let once = scratch.read(relative);
    assert!(!once.contains(&older), "the older reference survived");
    assert!(once.contains(&current_reference(relative, "htmlspec.css")));

    let (code, out, err) = run_in(&scratch.0, &["put", "format", "--upgrade-assets"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert_eq!(
        scratch.read(relative),
        once,
        "a second request changed the document"
    );
    for arguments in [
        &["format", "--check", "--upgrade-assets"][..],
        &["validate", "--strict", relative],
    ] {
        let (code, out, err) = run_in(&scratch.0, arguments);
        assert_eq!(code, SUCCESS, "{arguments:?}:\n{out}{err}");
    }
}

/// The request adds nothing a document does not load.
#[test]
fn upgrade_assets_adds_no_asset_a_document_does_not_load() {
    let scratch = Scratch::project("upgrade-adds-nothing");
    let relative = "htmlspec/specs/plain.html";
    a_document(&scratch.0, relative);
    let without: String = scratch
        .read(relative)
        .lines()
        .filter(|line| !line.contains("vega"))
        .map(|line| format!("{line}\n"))
        .collect();
    std::fs::write(scratch.path(relative), without).expect("rewritten");
    on_an_older_stylesheet(&scratch.0, relative);

    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        !scratch.read(relative).contains("vega"),
        "the request added a script the document did not load"
    );
}

/// Archived changes and run records are history, and the request leaves them as written, whether it
/// reaches them directly or through a symbolic link.
#[test]
fn upgrade_assets_leaves_history_as_written() {
    let scratch = Scratch::project("upgrade-history");
    let archived = "htmlspec/changes/archive/2026-01-01-earlier/specification.html";
    let record = "htmlspec/runbooks/release/runs/first.html";
    let living = "htmlspec/specs/earlier.html";
    for relative in [archived, record, living] {
        a_document(&scratch.0, relative);
        on_an_older_stylesheet(&scratch.0, relative);
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        scratch.path("htmlspec/changes/archive/2026-01-01-earlier"),
        scratch.path("htmlspec/specs/linked"),
    )
    .expect("the link is made");

    let archived_before = scratch.read(archived);
    let record_before = scratch.read(record);
    let living_before = scratch.read(living);
    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert_eq!(
        scratch.read(archived),
        archived_before,
        "an archived document changed"
    );
    assert_eq!(
        scratch.read(record),
        record_before,
        "a run's record changed"
    );
    assert_ne!(
        scratch.read(living),
        living_before,
        "the living document did not"
    );
    assert!(
        err.contains("2026-01-01-earlier") && err.contains("first.html") && err.contains("history"),
        "the documents left as written are not reported:\n{err}"
    );
}

/// A store missing the current bytes is given them from the copy compiled into the program: nothing
/// is fetched, and the document validates against the store beside it.
#[test]
fn upgrade_assets_fills_the_store_from_the_program() {
    let scratch = Scratch::project("upgrade-fills-store");
    let relative = "htmlspec/specs/earlier.html";
    a_document(&scratch.0, relative);
    on_an_older_stylesheet(&scratch.0, relative);
    let asset = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == "htmlspec.css")
        .expect("the stylesheet");
    let current = scratch.path("htmlspec/assets").join(asset.path);
    std::fs::remove_file(&current).expect("the current revision is removed from the store");

    // No network is reachable through these, and the operation needs none.
    let output = Command::new(binary())
        .current_dir(&scratch.0)
        .args(["format", "--upgrade-assets"])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("ALL_PROXY", "http://127.0.0.1:9")
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(SUCCESS),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read(&current).expect("the current revision is stored again"),
        asset.bytes,
        "the store does not hold the program's own bytes"
    );
    let (code, out, err) = run_in(&scratch.0, &["validate", "--strict", relative]);
    assert_eq!(code, SUCCESS, "{out}{err}");
}

// -----------------------------------------------------------------------------
// PR #80's review
// -----------------------------------------------------------------------------

/// A runbook that cannot validate leaves nothing behind: both documents are checked before its
/// directory exists, so a failure writes no file into it.
#[test]
fn new_runbook_writes_nothing_when_a_document_would_not_validate() {
    let scratch = Scratch::project("new-runbook-no-partial");
    // The current stylesheet's address holds other bytes, so every new document naming it fails
    // `asset/mismatch`, which is an error.
    let current = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == "htmlspec.css")
        .expect("the stylesheet");
    std::fs::write(
        scratch.path("htmlspec/assets").join(current.path),
        b"/* replaced */",
    )
    .expect("the store's bytes are replaced");

    let (code, _, err) = run_in(&scratch.0, &["new", "runbook", "deploy"]);
    assert_eq!(code, OPERATIONAL_FAILURE, "{err}");
    assert!(err.contains("nothing was written"), "{err}");
    assert!(
        !scratch.path("htmlspec/runbooks/deploy").exists(),
        "a runbook that would not validate left a directory behind"
    );
}

/// A project below the directory `format` is given is still judged as a project: its archived
/// changes and run records are left as written, under `--upgrade-assets` and `--fallback` alike.
/// The `--fallback` half was a gap before this change.
#[test]
fn format_given_a_directory_above_a_project_leaves_its_history_as_written() {
    let scratch = Scratch::new("format-from-above");
    let project = scratch.path("wrap/proj");
    std::fs::create_dir_all(&project).expect("the project's directory");
    let (code, out, err) = run_in(&project, &["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let archived = "htmlspec/changes/archive/2026-01-01-earlier/specification.html";
    let record = "htmlspec/runbooks/release/runs/first.html";
    let living = "htmlspec/specs/earlier.html";
    for relative in [archived, record, living] {
        a_document(&project, relative);
        on_an_older_stylesheet(&project, relative);
        let path = project.join(relative);
        let without = std::fs::read_to_string(&path)
            .expect("readable")
            .replace(&format!("    {}\n", htmlspec_assets::fallback::ELEMENT), "");
        std::fs::write(&path, without).expect("rewritten");
    }
    let read = |relative: &str| std::fs::read_to_string(project.join(relative)).expect("readable");
    let (archived_before, record_before, living_before) =
        (read(archived), read(record), read(living));

    for flag in ["--upgrade-assets", "--fallback"] {
        let (code, out, err) = run_in(&scratch.0, &["format", flag, "wrap"]);
        assert_eq!(code, SUCCESS, "{flag}: {out}{err}");
        assert_eq!(
            read(archived),
            archived_before,
            "{flag} changed an archived document"
        );
        assert_eq!(read(record), record_before, "{flag} changed a run's record");
    }
    let living_after = read(living);
    assert_ne!(
        living_after, living_before,
        "the living document did not change"
    );
    assert!(living_after.contains(htmlspec_assets::fallback::ELEMENT));
    assert!(living_after.contains(&current_reference(living, "htmlspec.css")));
}

/// A link to an archived document or a run's record, from a living directory, is judged where it
/// resolves, so neither request rewrites the history it points at.
#[cfg(unix)]
#[test]
fn a_link_to_history_is_judged_where_it_resolves() {
    let scratch = Scratch::project("format-file-links");
    let archived = "htmlspec/changes/archive/2026-01-01-earlier/specification.html";
    let record = "htmlspec/runbooks/release/runs/first.html";
    for relative in [archived, record] {
        a_document(&scratch.0, relative);
        on_an_older_stylesheet(&scratch.0, relative);
    }
    std::os::unix::fs::symlink(
        scratch.path(archived),
        scratch.path("htmlspec/specs/linked-archive.html"),
    )
    .expect("the link to the archive");
    std::os::unix::fs::symlink(
        scratch.path(record),
        scratch.path("htmlspec/specs/linked-record.html"),
    )
    .expect("the link to the record");
    let (archived_before, record_before) = (scratch.read(archived), scratch.read(record));

    for flag in ["--upgrade-assets", "--fallback"] {
        let (code, out, err) = run_in(&scratch.0, &["format", flag, "htmlspec/specs"]);
        assert_eq!(code, SUCCESS, "{flag}: {out}{err}");
        assert_eq!(
            scratch.read(archived),
            archived_before,
            "{flag} rewrote an archived document through a link"
        );
        assert_eq!(
            scratch.read(record),
            record_before,
            "{flag} rewrote a run's record through a link"
        );
        assert!(err.contains("history"), "{flag} did not say why:\n{err}");
    }
}

/// `--check --upgrade-assets` names a canonical document that is behind as behind, and only that.
#[test]
fn check_names_a_canonical_document_that_is_behind_as_behind_only() {
    let scratch = Scratch::project("upgrade-check-message");
    let relative = "htmlspec/specs/earlier.html";
    a_document(&scratch.0, relative);
    on_an_older_stylesheet(&scratch.0, relative);

    let (code, out, _) = run_in(&scratch.0, &["format", "--check", "--upgrade-assets"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}");
    assert!(out.contains("behind the store's current revision"), "{out}");
    assert!(
        !out.contains("not canonically formatted"),
        "a canonical document was reported as not canonical:\n{out}"
    );
}

/// The store holding other bytes at the current address stops the move: the reference is left as
/// written, and the report says why.
#[test]
fn upgrade_assets_does_not_move_a_reference_onto_replaced_bytes() {
    let scratch = Scratch::project("upgrade-onto-replaced");
    let relative = "htmlspec/specs/earlier.html";
    a_document(&scratch.0, relative);
    let older = on_an_older_stylesheet(&scratch.0, relative);
    let current = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == "htmlspec.css")
        .expect("the stylesheet");
    std::fs::write(
        scratch.path("htmlspec/assets").join(current.path),
        b"/* replaced */",
    )
    .expect("the store's bytes are replaced");

    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets", relative]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        scratch.read(relative).contains(&older),
        "the reference moved onto replaced bytes"
    );
    assert!(err.contains("left as written"), "{err}");
}

/// A title holding a control character reaches the terminal escaped, from the listing and from a
/// bare `run`, and the payload carries it as it is.
#[test]
fn a_runbook_title_reaches_the_terminal_escaped() {
    let scratch = Scratch::project("listing-escaped");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "deploy"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let tasks = scratch.path("htmlspec/runbooks/deploy/tasks.html");
    let source = std::fs::read_to_string(&tasks).expect("readable");
    std::fs::write(
        &tasks,
        source.replace(
            "<title>deploy</title>",
            "<title>deploy &#27;[31mred &#x202E;txt.exe</title>",
        ),
    )
    .expect("rewritten");

    let (code, out, _) = run_in(&scratch.0, &["list", "--runbooks"]);
    assert_eq!(code, SUCCESS, "{out}");
    let (_, _, err) = run_in(&scratch.0, &["run"]);
    for printed in [&out, &err] {
        assert!(
            !printed.contains('\u{1b}') && !printed.contains('\u{202e}'),
            "an escape sequence or a bidi override reached the terminal: {printed:?}"
        );
        assert!(
            printed.contains("deploy \\u{1b}[31mred \\u{202e}txt.exe"),
            "{printed}"
        );
    }
    let (_, json, _) = run_in(&scratch.0, &["--json", "list", "--runbooks"]);
    let payload: serde_json::Value = serde_json::from_str(&json).expect("the payload parses");
    assert_eq!(
        payload["runbooks"][0]["title"],
        "deploy \u{1b}[31mred \u{202e}txt.exe"
    );
}

// -----------------------------------------------------------------------------
// The upgrade names each runbook it rewrites
// -----------------------------------------------------------------------------

/// A runbook named `name`, both of its documents on an older stylesheet.
fn a_runbook_on_an_older_stylesheet(root: &Path, name: &str) {
    let (code, out, err) = run_in(root, &["new", "runbook", name]);
    assert_eq!(code, SUCCESS, "new runbook failed:\n{out}{err}");
    for document in ["specification.html", "tasks.html"] {
        on_an_older_stylesheet(root, &format!("htmlspec/runbooks/{name}/{document}"));
    }
}

/// A change named `name`, ready to archive with the project declaring `version`. Its specification
/// states one decision and nothing else, so archival has no requirement to merge and no example
/// content to refuse.
fn a_change_ready_to_archive(root: &Path, name: &str, version: &str) {
    let (code, out, err) = run_in(root, &["new", "change", name, "--subject", name]);
    assert_eq!(code, SUCCESS, "new change failed:\n{out}{err}");
    let relative = format!("htmlspec/changes/{name}/specification.html");
    a_document(root, &relative);
    let path = root.join(&relative);
    let source = std::fs::read_to_string(&path).expect("readable");
    let opening = "<main class=\"hs-main\" id=\"hs-content\">";
    let start = source.find(opening).expect("a main") + opening.len();
    let end = source.find("</main>").expect("a main's end");
    let decision = format!(
        "\n<section id=\"hs-decisions\"><h2>Decisions</h2>\
         <section class=\"decision\" id=\"decision-{name}\" data-change=\"added\" \
         data-status=\"accepted\"><h3>Archive {name} <span class=\"hs-id\">decision-{name}</span></h3>\
         <p>The change {name} is archived.</p>\
         <p data-field=\"rationale\">Archival publishes the project's runbooks.</p></section></section>\n"
    );
    let written = format!("{}{decision}{}", &source[..start], &source[end..])
        .lines()
        .filter(|line| {
            ![
                "hs-requirements",
                "hs-risks",
                "hs-evidence",
                "hs-verification",
            ]
            .iter()
            .any(|section| line.contains(&format!("href=\"#{section}\"")))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            "Replace this paragraph with what this document is for. It projects into the \
             document's summary, so the argument the change rests on lives in the model rather \
             than in markup the model cannot address.",
            "A change archived to publish the project's runbooks.",
        );
    std::fs::write(&path, format!("{written}\n")).expect("written");
    let (code, out, err) = run_in(root, &["format", &relative]);
    assert_eq!(code, SUCCESS, "format failed:\n{out}{err}");

    let declaration = root.join("spec.nix");
    let text = std::fs::read_to_string(&declaration).expect("readable");
    let current = text
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("version = \"")
                .and_then(|rest| rest.strip_suffix("\";"))
        })
        .expect("a declared version")
        .to_owned();
    std::fs::write(
        &declaration,
        text.replace(
            &format!("version = \"{current}\""),
            &format!("version = \"{version}\""),
        ),
    )
    .expect("written");
}

/// Archives a change named `name` at project `version`, which publishes every runbook the project
/// holds.
fn archive_a_change(root: &Path, name: &str, version: &str) {
    a_change_ready_to_archive(root, name, version);
    let (code, out, err) = run_in(root, &["archive", name, "--yes"]);
    assert_eq!(code, SUCCESS, "archival failed:\n{out}{err}");
}

/// A runbook released at its current version and then rewritten by the upgrade is named, in both
/// modes, as needing a greater version, and archival agrees: it refuses until the version moves,
/// and its refusal names the upgrade.
#[test]
fn upgrading_a_released_runbook_names_it_and_says_its_version_must_advance() {
    let scratch = Scratch::project("upgrade-released-runbook");
    a_runbook_on_an_older_stylesheet(&scratch.0, "demo");
    archive_a_change(&scratch.0, "first", "0.1.0");

    let tasks = "htmlspec/runbooks/demo/tasks.html";
    let before = scratch.read(tasks);
    let (code, out, err) = run_in(&scratch.0, &["format", "--check", "--upgrade-assets"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");
    assert!(
        out.contains("the runbook demo would change with its documents")
            && out.contains("0.1.0 is the version")
            && out.contains("-first already released")
            && out.contains("htmlspec/runbooks/demo/VERSION"),
        "check mode does not say the released runbook needs a greater version:\n{out}"
    );
    assert_eq!(scratch.read(tasks), before, "check mode wrote the runbook");
    assert_eq!(
        out.matches("the runbook demo").count(),
        1,
        "the runbook is named once per run, not once per document:\n{out}"
    );

    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    assert_eq!(
        code, SUCCESS,
        "the advice changed the exit status:\n{out}{err}"
    );
    assert!(
        out.contains("the runbook demo changed with its documents")
            && out.contains("already released"),
        "writing does not say the released runbook needs a greater version:\n{out}"
    );

    a_change_ready_to_archive(&scratch.0, "second", "0.2.0");
    let (code, out, err) = run_in(&scratch.0, &["archive", "second", "--check"]);
    assert_ne!(
        code, SUCCESS,
        "archival accepted a rewritten runbook at a released version:\n{out}{err}"
    );
    assert!(
        err.contains("demo")
            && err.contains("0.1.0")
            && err.contains("htmlspec format --upgrade-assets"),
        "the refusal does not name the upgrade as the likely cause:\n{err}"
    );

    // The advice followed: a greater version with its heading, and archival accepts the runbook.
    std::fs::write(scratch.path("htmlspec/runbooks/demo/VERSION"), "0.1.1\n").expect("written");
    let changelog = scratch.path("htmlspec/runbooks/demo/CHANGELOG.md");
    let text = std::fs::read_to_string(&changelog).expect("readable");
    std::fs::write(
        &changelog,
        text.replacen(
            "## 0.1.0",
            "## 0.1.1\n\nMoves onto the current stylesheet.\n\n## 0.1.0",
            1,
        ),
    )
    .expect("written");
    let (code, out, err) = run_in(&scratch.0, &["archive", "second", "--check"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
}

/// A runbook whose version no archive has released is named as releasing at that version, and the
/// next archival publishes it as the upgrade left it.
#[test]
fn upgrading_an_unreleased_runbook_says_the_next_archival_releases_it() {
    let scratch = Scratch::project("upgrade-unreleased-runbook");
    a_runbook_on_an_older_stylesheet(&scratch.0, "demo");

    let (code, out, err) = run_in(&scratch.0, &["format", "--check", "--upgrade-assets"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");
    assert!(
        out.contains("the runbook demo would change with its documents")
            && out.contains("Its version, 0.1.0, is not released yet"),
        "check mode does not say an unreleased runbook releases at its version:\n{out}"
    );
    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        out.contains("the runbook demo changed with its documents")
            && out.contains("not released yet"),
        "{out}"
    );
    archive_a_change(&scratch.0, "first", "0.1.0");
}

/// Only a runbook whose own documents the upgrade rewrote is named: not a run's record, which is
/// history and never rewritten, not a document outside every runbook, and not a runbook already on
/// the current assets.
#[test]
fn the_upgrade_names_only_the_runbooks_it_rewrites() {
    let scratch = Scratch::project("upgrade-names-only-rewritten");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "current"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let record = "htmlspec/runbooks/current/runs/first.html";
    a_document(&scratch.0, record);
    let kept = on_an_older_stylesheet(&scratch.0, record);
    let relative = "htmlspec/specs/earlier.html";
    a_document(&scratch.0, relative);
    let moved = on_an_older_stylesheet(&scratch.0, relative);

    for arguments in [
        &["format", "--check", "--upgrade-assets"][..],
        &["format", "--upgrade-assets"][..],
    ] {
        let (_, out, err) = run_in(&scratch.0, arguments);
        assert!(
            !out.contains("the runbook"),
            "{arguments:?} named a runbook it did not rewrite:\n{out}{err}"
        );
    }
    assert!(
        !scratch.read(relative).contains(&moved),
        "the document outside every runbook was not upgraded"
    );
    assert!(
        scratch.read(record).contains(&kept),
        "the run's record was rewritten"
    );
}

/// Takes the fallback stylesheet out of the document at `relative`, as a runbook written before the
/// fallback existed carries none.
fn without_the_fallback(root: &Path, relative: &str) {
    let path = root.join(relative);
    let written = std::fs::read_to_string(&path).expect("readable");
    let without = written.replace(&format!("    {}\n", htmlspec_assets::fallback::ELEMENT), "");
    assert_ne!(
        without, written,
        "{relative} carried no fallback to take out"
    );
    std::fs::write(&path, without).expect("rewritten");
}

/// A released runbook the fallback is added to is named as `--upgrade-assets` names one, in both
/// modes, and archival's refusal names the fallback. A runbook that already carries the current
/// fallback, and a second request that changes nothing, name no runbook.
#[test]
fn adding_the_fallback_to_a_released_runbook_names_it() {
    let scratch = Scratch::project("fallback-released-runbook");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    for document in ["specification.html", "tasks.html"] {
        without_the_fallback(&scratch.0, &format!("htmlspec/runbooks/demo/{document}"));
    }
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "current"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    archive_a_change(&scratch.0, "first", "0.1.0");

    let tasks = "htmlspec/runbooks/demo/tasks.html";
    let before = scratch.read(tasks);
    let (code, out, err) = run_in(&scratch.0, &["format", "--check", "--fallback"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");
    assert!(
        out.contains("the runbook demo would change with its documents")
            && out.contains("0.1.0 is the version")
            && out.contains("htmlspec/runbooks/demo/VERSION"),
        "check mode does not say the released runbook needs a greater version:\n{out}"
    );
    assert_eq!(
        out.matches("the runbook demo").count(),
        1,
        "the runbook is named once per run, not once per document:\n{out}"
    );
    assert!(
        !out.contains("the runbook current"),
        "a runbook already carrying the fallback is named:\n{out}"
    );
    assert_eq!(scratch.read(tasks), before, "check mode wrote the runbook");

    let (code, out, err) = run_in(&scratch.0, &["format", "--fallback"]);
    assert_eq!(
        code, SUCCESS,
        "the advice changed the exit status:\n{out}{err}"
    );
    assert!(
        out.contains("the runbook demo changed with its documents")
            && out.contains("already released")
            && !out.contains("the runbook current"),
        "writing does not name exactly the runbook it gave the fallback:\n{out}"
    );

    // Nothing left to change, so nothing to name.
    for arguments in [
        &["format", "--check", "--fallback"][..],
        &["format", "--fallback"][..],
    ] {
        let (code, out, err) = run_in(&scratch.0, arguments);
        assert_eq!(code, SUCCESS, "{arguments:?}:\n{out}{err}");
        assert!(
            !out.contains("the runbook"),
            "{arguments:?} named a runbook it did not change:\n{out}{err}"
        );
    }

    a_change_ready_to_archive(&scratch.0, "second", "0.2.0");
    let (code, out, err) = run_in(&scratch.0, &["archive", "second", "--check"]);
    assert_ne!(
        code, SUCCESS,
        "archival accepted a rewritten runbook at a released version:\n{out}{err}"
    );
    assert!(
        err.contains("demo") && err.contains("htmlspec format --fallback"),
        "the refusal does not name the fallback as a likely cause:\n{err}"
    );
}

/// A released runbook already raised to an unreleased version, with its heading, is told what that
/// heading says: that it now carries the fallback, and, given both requests, that it also moved onto
/// the current assets. Without the heading it is told to add one saying so.
#[test]
fn the_heading_advice_says_which_requests_rewrote_the_runbook() {
    let scratch = Scratch::project("fallback-unreleased-runbook");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    archive_a_change(&scratch.0, "first", "0.1.0");
    std::fs::write(scratch.path("htmlspec/runbooks/demo/VERSION"), "0.1.1\n").expect("raised");
    without_the_fallback(&scratch.0, "htmlspec/runbooks/demo/specification.html");

    let (code, out, err) = run_in(&scratch.0, &["format", "--check", "--fallback"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");
    assert!(
        out.contains(
            "has no heading for 0.1.1: give it one, saying that it now carries the \
             current fallback stylesheet"
        ),
        "the advice does not ask for a heading saying what changed:\n{out}"
    );

    let changelog = scratch.path("htmlspec/runbooks/demo/CHANGELOG.md");
    let text = std::fs::read_to_string(&changelog).expect("readable");
    std::fs::write(
        &changelog,
        text.replacen("## 0.1.0", "## 0.1.1\n\nThe fallback.\n\n## 0.1.0", 1),
    )
    .expect("written");

    let (code, out, err) = run_in(&scratch.0, &["format", "--check", "--fallback"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");
    assert!(
        out.contains("not released yet")
            && out.contains("that it now carries the current fallback stylesheet")
            && !out.contains("current assets"),
        "the advice does not say the fallback alone changed it:\n{out}"
    );

    on_an_older_stylesheet(&scratch.0, "htmlspec/runbooks/demo/tasks.html");
    let (code, out, err) = run_in(
        &scratch.0,
        &["format", "--check", "--fallback", "--upgrade-assets"],
    );
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");
    assert!(
        out.contains(
            "that it moved onto the current assets and now carries the current fallback stylesheet"
        ),
        "the advice does not say both requests changed it:\n{out}"
    );
    assert_eq!(out.matches("the runbook demo").count(), 1, "{out}");
}

// -----------------------------------------------------------------------------
// The upgrade names a runbook only for documents it settled, and finds each as archival does
// -----------------------------------------------------------------------------

/// The runbook `name`'s procedure on an older stylesheet, with markup the formatter refuses: a
/// stray end tag, which the parser can only recover from.
fn a_refused_procedure(root: &Path, name: &str) {
    let relative = format!("htmlspec/runbooks/{name}/tasks.html");
    on_an_older_stylesheet(root, &relative);
    let path = root.join(&relative);
    let source = std::fs::read_to_string(&path).expect("readable");
    let refused = source.replacen("</main>", "</span></main>", 1);
    assert_ne!(refused, source, "the procedure has no main to spoil");
    std::fs::write(&path, refused).expect("written");
}

/// A runbook whose only rewritten document the formatter refuses is named in neither mode: nothing
/// of it is written, so nothing of it changed.
#[test]
fn a_refused_document_names_no_runbook() {
    let scratch = Scratch::project("upgrade-refused-runbook");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    a_refused_procedure(&scratch.0, "demo");
    let before = scratch.read("htmlspec/runbooks/demo/tasks.html");

    for arguments in [
        &["format", "--check", "--upgrade-assets"][..],
        &["format", "--upgrade-assets"][..],
    ] {
        let (code, out, err) = run_in(&scratch.0, arguments);
        assert_eq!(code, OPERATIONAL_FAILURE, "{arguments:?}:\n{out}{err}");
        assert!(
            err.contains("refusing to format"),
            "{arguments:?} did not refuse the document:\n{err}"
        );
        assert!(
            !out.contains("the runbook demo"),
            "{arguments:?} named a runbook whose document it refused:\n{out}"
        );
    }
    assert_eq!(
        scratch.read("htmlspec/runbooks/demo/tasks.html"),
        before,
        "the refused document was written"
    );
}

/// A runbook whose only rewritten document cannot be written is named when checking, which writes
/// nothing, and not when writing fails.
#[cfg(unix)]
#[test]
fn an_unwritable_document_names_no_runbook_when_writing() {
    use std::os::unix::fs::PermissionsExt;

    let scratch = Scratch::project("upgrade-unwritable-runbook");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let tasks = scratch.path("htmlspec/runbooks/demo/tasks.html");
    on_an_older_stylesheet(&scratch.0, "htmlspec/runbooks/demo/tasks.html");
    std::fs::set_permissions(&tasks, std::fs::Permissions::from_mode(0o444)).expect("read-only");

    let (code, out, err) = run_in(&scratch.0, &["format", "--check", "--upgrade-assets"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");
    assert!(
        out.contains("the runbook demo would change"),
        "checking does not name the runbook:\n{out}"
    );
    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    std::fs::set_permissions(&tasks, std::fs::Permissions::from_mode(0o644)).expect("writable");
    assert_eq!(code, OPERATIONAL_FAILURE, "{out}{err}");
    assert!(err.contains("cannot write"), "{err}");
    assert!(
        !out.contains("the runbook demo"),
        "writing named a runbook whose document it could not write:\n{out}"
    );
}

/// A file in a runbook's directory other than its specification and procedure is no part of the
/// release archival compares, so upgrading it names no runbook.
#[test]
fn another_document_in_a_runbook_folder_names_no_runbook() {
    let scratch = Scratch::project("upgrade-runbook-notes");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    let notes = "htmlspec/runbooks/demo/notes.html";
    a_document(&scratch.0, notes);
    let older = on_an_older_stylesheet(&scratch.0, notes);

    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        !scratch.read(notes).contains(&older),
        "the notes were not upgraded"
    );
    assert!(
        !out.contains("the runbook demo"),
        "a document outside the release named its runbook:\n{out}"
    );
}

/// A runbook reached through a symbolic link is the runbook archival publishes under the link's
/// name, so the upgrade names it by that name, once, however the walk reaches its documents.
#[cfg(unix)]
#[test]
fn a_linked_runbook_is_named_by_its_name() {
    let scratch = Scratch::project("upgrade-linked-runbook");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    std::fs::create_dir_all(scratch.path("htmlspec/kept")).expect("a directory");
    std::fs::rename(
        scratch.path("htmlspec/runbooks/demo"),
        scratch.path("htmlspec/kept/demo"),
    )
    .expect("moved");
    std::os::unix::fs::symlink("../kept/demo", scratch.path("htmlspec/runbooks/demo"))
        .expect("linked");
    on_an_older_stylesheet(&scratch.0, "htmlspec/kept/demo/tasks.html");

    let (code, out, err) = run_in(&scratch.0, &["format", "--check", "--upgrade-assets"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");
    assert_eq!(
        out.matches("the runbook demo would change").count(),
        1,
        "the linked runbook is not named once:\n{out}"
    );
    assert!(
        !out.contains("the runbook kept"),
        "the link's target directory is named as a runbook:\n{out}"
    );
}

/// A runbook at a version nothing released, whose changelog has no heading for it, is told to add
/// one, because archival refuses it without one, as archival then does.
#[test]
fn an_unreleased_runbook_without_its_heading_is_told_to_add_one() {
    let scratch = Scratch::project("upgrade-unheaded-runbook");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    on_an_older_stylesheet(&scratch.0, "htmlspec/runbooks/demo/tasks.html");
    let changelog = scratch.path("htmlspec/runbooks/demo/CHANGELOG.md");
    let text = std::fs::read_to_string(&changelog).expect("readable");
    let unheaded = text.replacen("## 0.1.0", "## First", 1);
    assert_ne!(unheaded, text, "the changelog had no heading to take out");
    std::fs::write(&changelog, unheaded).expect("written");

    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        out.contains("has no heading for 0.1.0") && !out.contains("releases it as it now stands"),
        "the line promises a release archival refuses:\n{out}"
    );
    a_change_ready_to_archive(&scratch.0, "first", "0.1.0");
    let (code, out, err) = run_in(&scratch.0, &["archive", "first", "--check"]);
    assert_ne!(
        code, SUCCESS,
        "archival accepted a release without its heading:\n{out}{err}"
    );
}

/// A first release is not told what its heading says moved: the heading is the first release's.
#[test]
fn a_first_release_is_not_told_what_its_heading_says() {
    let scratch = Scratch::project("upgrade-first-release-heading");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    on_an_older_stylesheet(&scratch.0, "htmlspec/runbooks/demo/tasks.html");

    let (code, out, err) = run_in(&scratch.0, &["format", "--upgrade-assets"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        out.contains("releases it as it now stands") && !out.contains("heading for 0.1.0 that"),
        "a never-published runbook is told what its first heading says:\n{out}"
    );
}

/// A runbook directory named with an escape sequence reaches the terminal with it named, in the
/// formatter's lines and in archival's refusal, and never as the sequence itself.
#[test]
fn a_runbook_name_with_an_escape_reaches_the_report_shown() {
    const NAME: &str = "de\u{1b}[31mmo";
    let scratch = Scratch::project("upgrade-escaped-runbook");
    let (code, out, err) = run_in(&scratch.0, &["new", "runbook", "demo"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    std::fs::rename(
        scratch.path("htmlspec/runbooks/demo"),
        scratch.path(&format!("htmlspec/runbooks/{NAME}")),
    )
    .expect("renamed");
    on_an_older_stylesheet(&scratch.0, &format!("htmlspec/runbooks/{NAME}/tasks.html"));
    archive_a_change(&scratch.0, "first", "0.1.0");

    for arguments in [
        &["format", "--check", "--upgrade-assets"][..],
        &["format", "--upgrade-assets"][..],
    ] {
        let (_, out, err) = run_in(&scratch.0, arguments);
        assert!(
            out.contains("the runbook de<U+001B>[31mmo"),
            "{arguments:?} does not name the runbook shown:\n{out}"
        );
        assert!(
            !out.contains('\u{1b}') && !err.contains('\u{1b}'),
            "{arguments:?} wrote an escape sequence to the terminal:\n{out}{err}"
        );
    }
    a_change_ready_to_archive(&scratch.0, "second", "0.2.0");
    let (code, out, err) = run_in(&scratch.0, &["archive", "second", "--check"]);
    assert_ne!(code, SUCCESS, "{out}{err}");
    assert!(
        err.contains("de<U+001B>[31mmo") && !err.contains('\u{1b}'),
        "archival's refusal wrote the runbook's name raw:\n{err}"
    );
}
