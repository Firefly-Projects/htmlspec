//! The request-verb spellings, driven through the built binary.
//!
//! toolchain-packaging's "The command surface names its operations as OpenSpec names them" gives
//! every operation a second spelling, a request verb and an object, and requires it to run the
//! operation its OpenSpec spelling runs, with the same arguments, output, and exit status. This file
//! holds what the unit tests in `main.rs` and `operation.rs` cannot reach:
//!
//! - both spellings of every operation, run with the same arguments in two copies of one project,
//!   print the same thing, exit the same way, and leave the same tree;
//! - every operation spelled with `get` leaves a project byte for byte as it was, and so does every
//!   option that withholds an operation's writes, under the verb spelling;
//! - help lists both spellings, and an object's help shows its operation's options.
//!
//! **The two copies stand at one location, one after the other**, rather than side by side. The
//! operations print absolute paths, and the store registry records one, so two copies at two
//! locations would differ in every line that names a file. A copy is made from a pristine tree, run,
//! read, and removed before the next.
//!
//! The table of spellings below states them a second time, on purpose: it is what the program is
//! checked against. The help tests hold it to what the program offers, in both directions.

mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const SUCCESS: i32 = 0;
const DOCUMENT_VIOLATION: i32 = 1;
const OPERATIONAL_FAILURE: i32 = 2;

/// Every operation's two spellings: the OpenSpec words, the verb, and the object.
const SPELLINGS: &[(&[&str], &str, &str)] = &[
    (&["validate"], "get", "validity"),
    (&["list"], "get", "list"),
    (&["status"], "get", "status"),
    (&["sync"], "get", "sync"),
    (&["show"], "get", "show"),
    (&["assets"], "get", "assets"),
    (&["preflight"], "get", "preflight"),
    (&["instructions"], "get", "instructions"),
    (&["store", "list"], "get", "registrations"),
    (&["migrate", "survey"], "get", "survey"),
    (&["new", "change"], "post", "change"),
    (&["new", "document"], "post", "document"),
    (&["store", "add"], "post", "registration"),
    (&["run"], "post", "run"),
    (&["migrate", "promote"], "post", "promotion"),
    (&["init"], "put", "project"),
    (&["format"], "put", "format"),
    (&["archive"], "patch", "archive"),
    (&["store", "remove"], "delete", "registration"),
];

/// The five verbs, in the order help lists them.
const VERBS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

/// The request-verb words for an OpenSpec spelling.
fn verb_spelling(openspec: &[&str]) -> [&'static str; 2] {
    let Some((_, verb, object)) = SPELLINGS.iter().find(|(words, _, _)| *words == openspec) else {
        panic!(
            "no request-verb spelling is listed for `{}`",
            openspec.join(" ")
        );
    };
    [verb, object]
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// Runs the binary from `directory`, with the registry and home directory under `config`.
///
/// `HOME` as well as `XDG_CONFIG_HOME`, because `preflight` reads the home directory and a test
/// that let it read the real one would answer differently on two machines.
fn run_in(directory: &Path, config: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .env("XDG_CONFIG_HOME", config)
        .env("HOME", config)
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

/// One entry of a tree: a directory, or a file and its bytes.
///
/// Directories are entries of their own, because an operation that creates or removes an empty one
/// has written, and a map of files alone would not see it.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Entry {
    Directory,
    File(Vec<u8>),
}

type Tree = BTreeMap<PathBuf, Entry>;

/// Every entry under `root`, by its path relative to `root`.
fn snapshot(root: &Path) -> Tree {
    let mut tree = Tree::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the directory reads") {
            let path = entry.expect("the entry reads").path();
            let relative = path
                .strip_prefix(root)
                .expect("beneath the root")
                .to_owned();
            if path.is_dir() {
                tree.insert(relative, Entry::Directory);
                pending.push(path);
            } else {
                tree.insert(
                    relative,
                    Entry::File(std::fs::read(&path).expect("the file reads")),
                );
            }
        }
    }
    tree
}

/// The paths at which two trees differ, each marked with what the second did to it.
fn differences(before: &Tree, after: &Tree) -> Vec<String> {
    let mut found = Vec::new();
    for (path, entry) in before {
        match after.get(path) {
            None => found.push(format!("removed {}", path.display())),
            Some(other) if other != entry => found.push(format!("changed {}", path.display())),
            Some(_) => {}
        }
    }
    for path in after.keys() {
        if !before.contains_key(path) {
            found.push(format!("created {}", path.display()));
        }
    }
    found
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the copy's directory is created");
    for entry in std::fs::read_dir(from).expect("the directory reads") {
        let path = entry.expect("the entry reads").path();
        let target = to.join(path.file_name().expect("an entry has a name"));
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("the file is copied");
        }
    }
}

/// What one invocation printed, how it exited, and the tree it left.
#[derive(Debug)]
struct Ran {
    code: i32,
    stdout: String,
    stderr: String,
    tree: Tree,
}

/// Where an invocation stands.
#[derive(Clone, Copy, Debug)]
enum Area {
    /// An HTMLSpec project with a change, an accepted specification, a runbook, and a document
    /// that is not canonical.
    Project,
    /// A small OpenSpec project, for `migrate survey`.
    Source,
    /// A staged project, its plan, and its survey payload, for `migrate promote`.
    Migration,
    /// An empty directory, for `init` where no project is.
    Fresh,
}

impl Area {
    fn directory(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Source => "source",
            Self::Migration => "migration",
            Self::Fresh => "fresh",
        }
    }
}

/// A pristine tree, and the location every copy of it is run at.
///
/// The tree is built at `run/` and then moved to `pristine/`, so the absolute paths it records,
/// the store registration among them, name the location each copy stands at.
struct Fixture {
    scratch: PathBuf,
    area: Area,
}

impl Fixture {
    fn new(label: &str, area: Area) -> Self {
        let scratch = fresh_directory(
            &std::env::temp_dir().join(format!("htmlspec-verbs-{label}-{}", std::process::id())),
        );
        let fixture = Self { scratch, area };
        let run = fixture.run_root();
        std::fs::create_dir_all(run.join("config")).expect("the config directory");
        std::fs::create_dir_all(run.join(area.directory())).expect("the area's directory");
        match area {
            Area::Project => fixture.build_project(),
            Area::Source => fixture.build_source(),
            Area::Migration => fixture.build_migration(),
            Area::Fresh => {}
        }
        std::fs::rename(&run, fixture.pristine()).expect("the pristine tree is set aside");
        fixture
    }

    fn run_root(&self) -> PathBuf {
        self.scratch.join("run")
    }

    fn pristine(&self) -> PathBuf {
        self.scratch.join("pristine")
    }

    fn pristine_tree(&self) -> Tree {
        snapshot(&self.pristine())
    }

    /// Runs the binary in the area while the fixture is being built, asserting it succeeds.
    fn build_step(&self, arguments: &[&str]) {
        let run = self.run_root();
        let (code, out, err) = run_in(
            &run.join(self.area.directory()),
            &run.join("config"),
            arguments,
        );
        assert_eq!(code, SUCCESS, "{arguments:?} failed:\n{out}{err}");
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.run_root().join(self.area.directory()).join(relative);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("parent");
        std::fs::write(path, contents).expect("the file is written");
    }

    fn edit(&self, relative: &str, edit: impl Fn(String) -> String) {
        let path = self.run_root().join(self.area.directory()).join(relative);
        let before = std::fs::read_to_string(&path).expect("the file reads");
        let after = edit(before.clone());
        assert_ne!(before, after, "the edit to {relative} is stale");
        std::fs::write(path, after).expect("the file is written");
    }

    /// A document from a template, with its example content made the author's own.
    ///
    /// `review/example-content` reports an example identifier that survives, and archival refuses
    /// over one, which would make every case about the template rather than about the spelling.
    ///
    /// Not every kind holds example content, a proposal among them, so this rewrite may change
    /// nothing; the edits that must change something go through [`Self::edit`].
    fn real_document(&self, kind: &str, relative: &str, subject: &str) {
        self.build_step(&["new", "document", kind, relative, "--subject", subject]);
        let path = self.run_root().join(self.area.directory()).join(relative);
        let text = std::fs::read_to_string(&path).expect("the document reads");
        let real = text.replace("-example", "-real").replace(
            "Replace this paragraph with what",
            "This document states what",
        );
        std::fs::write(path, real).expect("the document is written");
    }

    fn build_project(&self) {
        self.build_step(&["init", "--tools", "none", "--name", "demo"]);
        self.build_step(&["new", "change", "demo", "--subject", "A change under test"]);

        // The accepted specification, and the change's, which modifies its requirement by a word.
        let accepted = "htmlspec/specs/surface/spec.html";
        self.real_document("specification", accepted, "The command surface");
        self.edit(accepted, |text| {
            text.replace("data-change=\"added\"", "data-change=\"unchanged\"")
        });
        let stated = "htmlspec/changes/demo/specification.html";
        self.real_document("specification", stated, "The command surface");
        self.edit(stated, |text| {
            text.replace(
                "id=\"req-real\" data-change=\"added\"",
                "id=\"req-real\" data-change=\"modified\"",
            )
            .replace(
                "state one obligation per requirement",
                "state exactly one obligation per requirement",
            )
        });

        // A runbook, for `run` and `preflight`, versioned so that archival can publish it.
        for (kind, file) in [
            ("specification", "specification.html"),
            ("tasks", "tasks.html"),
        ] {
            let relative = format!("htmlspec/runbooks/release/{file}");
            self.real_document(kind, &relative, "Releasing the project");
            self.edit(&relative, |text| {
                text.replace("data-change=\"added\"", "data-change=\"unchanged\"")
            });
        }
        self.write("htmlspec/runbooks/release/VERSION", "0.1.0\n");
        self.write(
            "htmlspec/runbooks/release/CHANGELOG.md",
            "# Changelog\n\n## 0.1.0\n\nA release of this procedure.\n",
        );

        // The declared version, past the initial one, so that archival's version gate passes.
        self.edit("spec.nix", |text| text.replace("\"0.0.0\"", "\"0.7.0\""));

        // A document that is not canonical: blank lines `format` removes.
        let loose = "htmlspec/specs/loose/spec.html";
        self.real_document("proposal", loose, "A loose document");
        self.edit(loose, |text| text.replacen("<main", "\n\n<main", 1));

        // One registration, naming the project where every copy of it stands.
        let project = self.run_root().join("project");
        self.build_step(&["store", "add", "demo-store", &project.to_string_lossy()]);
    }

    fn build_source(&self) {
        self.write(
            "openspec/specs/surface/spec.md",
            "# surface Specification\n\n## Purpose\nThe command surface.\n\n## Requirements\n\
             ### Requirement: A name\nThe system SHALL name it.\n\n#### Scenario: Named\n\
             - **WHEN** it runs\n- **THEN** it is named\n",
        );
        self.write(
            "openspec/changes/archive/2026-01-01-first/proposal.md",
            "## Why\n\nBecause.\n",
        );
    }

    fn build_migration(&self) {
        std::fs::create_dir_all(self.run_root().join("migration/staging")).expect("staging");
        std::fs::create_dir_all(self.run_root().join("migration/repository")).expect("destination");
        let staging = self.run_root().join("migration/staging");
        let config = self.run_root().join("config");
        for arguments in [
            &["init", "--tools", "none", "--name", "staged"][..],
            &[
                "new",
                "document",
                "specification",
                "htmlspec/specs/capability-0/spec.html",
                "--subject",
                "Capability 0",
            ],
        ] {
            let (code, out, err) = run_in(&staging, &config, arguments);
            assert_eq!(code, SUCCESS, "{arguments:?} failed:\n{out}{err}");
        }
        self.write(
            "staging/htmlspec/changes/archive/2026-08-10-html-spec/VERSION",
            "0.1.0\n",
        );
        self.edit("staging/spec.nix", |text| {
            text.replace("\"0.0.0\"", "\"0.1.0\"")
        });
        self.write(
            "staging/CHANGELOG.md",
            "# Changelog\n\n## 0.1.0\n\nThe baseline release.\n",
        );
        let entities = [
            (
                "openspec/specs/capability-0/spec.md",
                "htmlspec/specs/capability-0/spec.html",
            ),
            (
                "openspec/changes/archive/2026-08-10-html-spec/VERSION",
                "htmlspec/changes/archive/2026-08-10-html-spec/VERSION",
            ),
        ];
        self.write(
            "survey.json",
            &serde_json::json!({
                "entities": entities
                    .iter()
                    .map(|(source, _)| serde_json::json!({ "path": source, "kind": "x" }))
                    .collect::<Vec<_>>(),
            })
            .to_string(),
        );
        let items: String = entities
            .iter()
            .enumerate()
            .map(|(index, (source, destination))| {
                format!(
                    "<li class=\"task\" id=\"task-entity-{index}\" data-change=\"added\" \
                     data-state=\"todo\"><h3>Entity {index}</h3>\
                     <p data-field=\"note\">source: {source}</p>\
                     <p data-field=\"note\">destination: {destination}</p>\
                     <p data-field=\"note\">disposition: converted</p></li>"
                )
            })
            .collect();
        self.write(
            "plan.html",
            &format!(
                "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
                 <meta content=\"0.1.0\" name=\"htmlspec-version\">\
                 <meta content=\"tasks\" name=\"htmlspec-kind\">\
                 <title>Migration plan</title></head><body><main>{items}</main></body></html>"
            ),
        );
    }

    /// Leaves the pristine project holding an archival interrupted before its move: the two files
    /// it staged in the active change, and the journal naming them.
    ///
    /// The journal names the files where every copy stands, under `run/`, which is where recovery
    /// will look for them.
    fn interrupt_an_archival(&self) {
        let pristine = self.pristine().join("project/htmlspec");
        let at_run = self.run_root().join("project/htmlspec");
        let staged = [
            ("runbooks.json", "{\"runbooks\":[]}\n"),
            ("VERSION", "0.7.0\n"),
        ];
        for (file, contents) in staged {
            std::fs::write(pristine.join("changes/demo").join(file), contents).expect("staged");
        }
        let journal = serde_json::json!({
            "format": "htmlspec-archive-journal/v1",
            "change": at_run.join("changes/demo"),
            "target": at_run.join("changes/archive/2026-01-01-demo"),
            "writes": staged
                .iter()
                .map(|(file, contents)| serde_json::json!({
                    "path": at_run.join("changes/demo").join(file),
                    "digest": format!("sha256:{}", sha256(contents.as_bytes())),
                }))
                .collect::<Vec<_>>(),
        });
        std::fs::write(
            pristine.join(".archive-journal.json"),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&journal).expect("serializes")
            ),
        )
        .expect("the journal is written");
    }

    /// Runs `arguments` in a fresh copy of the pristine tree, and returns what it did.
    fn ran(&self, arguments: &[&str]) -> Ran {
        let run = self.run_root();
        let _ = std::fs::remove_dir_all(&run);
        copy_tree(&self.pristine(), &run);
        let (code, stdout, stderr) = run_in(
            &run.join(self.area.directory()),
            &run.join("config"),
            arguments,
        );
        let tree = snapshot(&run);
        std::fs::remove_dir_all(&run).expect("the copy is removed");
        Ran {
            code,
            stdout,
            stderr,
            tree,
        }
    }

    /// Runs the OpenSpec spelling and the request-verb spelling of one operation, each in its own
    /// copy, and asserts that they agree in everything the requirement names. Returns the run of
    /// the OpenSpec spelling, for the caller to say what the operation should have done.
    fn both_spellings(&self, openspec: &[&str], arguments: &[&str]) -> Ran {
        let named: Vec<&str> = openspec.iter().chain(arguments).copied().collect();
        let verb: Vec<&str> = verb_spelling(openspec)
            .into_iter()
            .chain(arguments.iter().copied())
            .collect();
        let by_name = self.ran(&named);
        let by_verb = self.ran(&verb);
        let shown = |ran: &Ran| {
            format!(
                "exit {}\nstdout:\n{}\nstderr:\n{}",
                ran.code, ran.stdout, ran.stderr
            )
        };
        assert!(
            by_name.code == by_verb.code
                && by_name.stdout == by_verb.stdout
                && by_name.stderr == by_verb.stderr,
            "`htmlspec {}` and `htmlspec {}` differ.\n--- the OpenSpec spelling:\n{}\n--- the \
             request-verb spelling:\n{}",
            named.join(" "),
            verb.join(" "),
            shown(&by_name),
            shown(&by_verb),
        );
        let differ = differences(&by_name.tree, &by_verb.tree);
        assert!(
            differ.is_empty(),
            "`htmlspec {}` and `htmlspec {}` left different trees:\n  {}",
            named.join(" "),
            verb.join(" "),
            differ.join("\n  ")
        );
        // What the program prints names the OpenSpec spelling, whichever spelling ran it.
        for stream in [&by_name.stdout, &by_name.stderr] {
            if let Some(spelling) = a_request_verb_spelling_in(stream) {
                panic!(
                    "`htmlspec {}` names the request-verb spelling `{spelling}` in what it \
                     prints:\n{stream}",
                    named.join(" ")
                );
            }
        }
        by_name
    }

    /// What a run wrote: the paths at which the tree it left differs from the pristine one.
    fn writes(&self, ran: &Ran) -> Vec<String> {
        differences(&self.pristine_tree(), &ran.tree)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.scratch);
    }
}

/// An empty directory at `path`, by its canonical path.
///
/// Canonical, because the program canonicalizes what it registers, and a path through a symbolic
/// link (`/var` on macOS) would otherwise be printed two ways.
fn fresh_directory(path: &Path) -> PathBuf {
    let _ = std::fs::remove_dir_all(path);
    std::fs::create_dir_all(path).expect("the scratch directory is created");
    path.canonicalize().expect("the scratch directory resolves")
}

/// A sha256 digest in hexadecimal, as archival writes one into its journal.
fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The first request-verb spelling `text` names as a command, if any.
///
/// A spelling is named as a command when the program's name precedes it, or a backtick opens it.
/// The words alone are not enough, because prose uses them: "nowhere to put project context".
fn a_request_verb_spelling_in(text: &str) -> Option<String> {
    for verb in VERBS {
        let invocation = format!("htmlspec {verb} ");
        if text.contains(&invocation) {
            return Some(invocation.trim_end().to_owned());
        }
    }
    SPELLINGS.iter().find_map(|(_, verb, object)| {
        let quoted = format!("`{verb} {object}");
        text.contains(&quoted).then(|| format!("{verb} {object}"))
    })
}

// -----------------------------------------------------------------------------
// Task 2.2: both spellings of every operation agree
// -----------------------------------------------------------------------------

#[test]
fn validate_and_get_validity_agree() {
    let fixture = Fixture::new("validate", Area::Project);
    for arguments in [
        &[][..],
        &["--strict"],
        &["--json"],
        &["htmlspec/specs/surface/spec.html"],
    ] {
        let ran = fixture.both_spellings(&["validate"], arguments);
        assert_eq!(
            ran.code, SUCCESS,
            "{arguments:?}:\n{}{}",
            ran.stdout, ran.stderr
        );
    }
}

#[test]
fn format_and_put_format_agree() {
    let fixture = Fixture::new("format", Area::Project);
    let ran = fixture.both_spellings(&["format"], &[]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert_eq!(
        fixture.writes(&ran),
        vec!["changed project/htmlspec/specs/loose/spec.html"],
        "the case does not rewrite the one document that is not canonical"
    );
    for arguments in [&["--check"][..], &["--check", "--json"]] {
        let ran = fixture.both_spellings(&["format"], arguments);
        assert_eq!(ran.code, DOCUMENT_VIOLATION, "{}{}", ran.stdout, ran.stderr);
    }
    let ran = fixture.both_spellings(&["format"], &["htmlspec/specs/surface/spec.html", "--json"]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
}

#[test]
fn assets_and_get_assets_agree() {
    let fixture = Fixture::new("assets", Area::Project);
    for arguments in [&[][..], &["--json"]] {
        let ran = fixture.both_spellings(&["assets"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
}

#[test]
fn archive_and_patch_archive_agree() {
    let fixture = Fixture::new("archive", Area::Project);
    let ran = fixture.both_spellings(&["archive"], &["demo", "--yes"]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert!(
        fixture
            .writes(&ran)
            .contains(&String::from("removed project/htmlspec/changes/demo")),
        "the case does not archive the change: {:?}",
        fixture.writes(&ran)
    );
    for arguments in [&["demo", "--check"][..], &["demo", "--check", "--json"]] {
        let ran = fixture.both_spellings(&["archive"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
    // A change that does not exist is refused the same way under both.
    let ran = fixture.both_spellings(&["archive"], &["absent", "--check"]);
    assert_ne!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
}

#[test]
fn init_and_put_project_agree() {
    let fixture = Fixture::new("init", Area::Project);
    let ran = fixture.both_spellings(&["init"], &["--tools", "none"]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    let ran = fixture.both_spellings(&["init"], &["--tools", "claude", "--json"]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert!(
        !fixture.writes(&ran).is_empty(),
        "the case installs no instructions"
    );
    // The withdrawn spelling is refused, and the refusal names `new` under both spellings.
    let ran = fixture.both_spellings(&["init"], &["doc.html"]);
    assert_ne!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert!(ran.stderr.contains("new"), "{}", ran.stderr);

    let fresh = Fixture::new("init-fresh", Area::Fresh);
    for arguments in [
        &["--tools", "none", "--name", "fresh"][..],
        &["--tools", "none", "--json"],
    ] {
        let ran = fresh.both_spellings(&["init"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
        assert!(
            ran.tree.contains_key(Path::new("fresh/spec.nix")),
            "the case establishes no project"
        );
    }
}

#[test]
fn list_and_get_list_agree() {
    let fixture = Fixture::new("list", Area::Project);
    for arguments in [
        &[][..],
        &["--specs"],
        &["--runbooks"],
        &["--json"],
        &["--store", "demo-store"],
    ] {
        let ran = fixture.both_spellings(&["list"], arguments);
        assert_eq!(
            ran.code, SUCCESS,
            "{arguments:?}:\n{}{}",
            ran.stdout, ran.stderr
        );
    }
}

#[test]
fn run_and_post_run_agree() {
    let fixture = Fixture::new("run", Area::Project);
    let ran = fixture.both_spellings(&["run"], &["release", "--as", "first"]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert!(
        ran.tree.contains_key(Path::new(
            "project/htmlspec/runbooks/release/runs/first.html"
        )),
        "the case starts no run"
    );
    for arguments in [
        &["release", "--check"][..],
        &["release", "--check", "--json"],
    ] {
        let ran = fixture.both_spellings(&["run"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
    // The two refusals `main` writes itself, which name the OpenSpec spelling under both.
    for arguments in [&["release"][..], &["release", "--as", "first", "--check"]] {
        let ran = fixture.both_spellings(&["run"], arguments);
        assert_eq!(
            ran.code, OPERATIONAL_FAILURE,
            "{}{}",
            ran.stdout, ran.stderr
        );
        assert!(ran.stderr.contains("htmlspec run"), "{}", ran.stderr);
    }
}

#[test]
fn preflight_and_get_preflight_agree() {
    let fixture = Fixture::new("preflight", Area::Project);
    for arguments in [&["release"][..], &["release", "--json"]] {
        let ran = fixture.both_spellings(&["preflight"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
}

#[test]
fn status_and_get_status_agree() {
    let fixture = Fixture::new("status", Area::Project);
    for arguments in [&["--change", "demo"][..], &["--change", "demo", "--json"]] {
        let ran = fixture.both_spellings(&["status"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
}

#[test]
fn sync_and_get_sync_agree() {
    let fixture = Fixture::new("sync", Area::Project);
    for arguments in [&["--change", "demo"][..], &["--change", "demo", "--json"]] {
        let ran = fixture.both_spellings(&["sync"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
}

#[test]
fn show_and_get_show_agree() {
    let fixture = Fixture::new("show", Area::Project);
    for arguments in [&["demo", "--diff"][..], &["demo", "--diff", "--json"]] {
        let ran = fixture.both_spellings(&["show"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
    // The bare form is refused, naming `htmlspec show <change> --diff` under both spellings.
    let ran = fixture.both_spellings(&["show"], &["demo"]);
    assert_eq!(
        ran.code, OPERATIONAL_FAILURE,
        "{}{}",
        ran.stdout, ran.stderr
    );
}

#[test]
fn store_list_and_get_registrations_agree() {
    let fixture = Fixture::new("store-list", Area::Project);
    for arguments in [&[][..], &["--json"]] {
        let ran = fixture.both_spellings(&["store", "list"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
        assert!(ran.stdout.contains("demo-store"), "{}", ran.stdout);
    }
}

#[test]
fn store_add_and_post_registration_agree() {
    let fixture = Fixture::new("store-add", Area::Project);
    let ran = fixture.both_spellings(&["store", "add"], &["second", "../project"]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert_eq!(
        fixture.writes(&ran),
        vec!["changed config/htmlspec/stores.json"],
        "the case registers nothing"
    );
    // A location that cannot be read is refused the same way. Not `../config`: the registry's own
    // directory, `config/htmlspec/`, makes the configuration directory resolve as a project.
    let ran = fixture.both_spellings(&["store", "add"], &["third", "../absent"]);
    assert_eq!(
        ran.code, OPERATIONAL_FAILURE,
        "{}{}",
        ran.stdout, ran.stderr
    );
}

#[test]
fn store_remove_and_delete_registration_agree() {
    let fixture = Fixture::new("store-remove", Area::Project);
    let ran = fixture.both_spellings(&["store", "remove"], &["demo-store"]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert_eq!(
        fixture.writes(&ran),
        vec!["changed config/htmlspec/stores.json"],
        "the case removes nothing, or removes more than the registration"
    );
    let ran = fixture.both_spellings(&["store", "remove"], &["absent"]);
    assert_eq!(
        ran.code, OPERATIONAL_FAILURE,
        "{}{}",
        ran.stdout, ran.stderr
    );
}

#[test]
fn instructions_and_get_instructions_agree() {
    let fixture = Fixture::new("instructions", Area::Project);
    for arguments in [&["apply"][..], &["apply", "--json"]] {
        let ran = fixture.both_spellings(&["instructions"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
    let ran = fixture.both_spellings(&["instructions"], &["no-such-stage"]);
    assert_ne!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
}

#[test]
fn new_change_and_post_change_agree() {
    let fixture = Fixture::new("new-change", Area::Project);
    for arguments in [
        &["another", "--subject", "Another change"][..],
        &["another", "--json"],
    ] {
        let ran = fixture.both_spellings(&["new", "change"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
        assert!(
            ran.tree
                .contains_key(Path::new("project/htmlspec/changes/another/proposal.html")),
            "the case creates no change"
        );
    }
    let ran = fixture.both_spellings(&["new", "change"], &["demo"]);
    assert_ne!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
}

#[test]
fn new_document_and_post_document_agree() {
    let fixture = Fixture::new("new-document", Area::Project);

    // Beneath the project's `htmlspec/`, the document references the project's store, so it
    // is the one file written.
    let ran = fixture.both_spellings(
        &["new", "document"],
        &["proposal", "htmlspec/changes/demo/notes.html"],
    );
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert_eq!(fixture.writes(&ran).len(), 1, "{:?}", fixture.writes(&ran));

    // In the project's root, outside `htmlspec/`, the project's store is not above the
    // document, so validation would never find it there. The document gets a store beside it
    // (resolve-a-new-documents-project-from-its-destination): the document, and nothing written
    // anywhere but that store.
    let ran = fixture.both_spellings(
        &["new", "document"],
        &["tasks", "notes.html", "--subject", "Notes", "--json"],
    );
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    let wrote = fixture.writes(&ran);
    assert!(
        wrote.iter().any(|w| w.ends_with("project/notes.html")),
        "{wrote:?}"
    );
    assert!(
        wrote.iter().any(|w| w.contains("project/assets/")),
        "no store was written beside the document: {wrote:?}"
    );
    assert!(
        wrote
            .iter()
            .all(|w| w.ends_with("project/notes.html") || w.contains("project/assets")),
        "something was written outside the document and the store beside it: {wrote:?}"
    );
}

#[test]
fn migrate_survey_and_get_survey_agree() {
    let fixture = Fixture::new("survey", Area::Source);
    for arguments in [&[][..], &["--json"], &["."]] {
        let ran = fixture.both_spellings(&["migrate", "survey"], arguments);
        assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    }
}

#[test]
fn migrate_promote_and_post_promotion_agree() {
    let fixture = Fixture::new("promote", Area::Migration);
    let plan = ["plan.html", "--survey", "survey.json"];
    let staged = ["--staged", "staging", "--into", "repository"];
    for arguments in [
        plan.to_vec(),
        [&plan[..], &["--json"]].concat(),
        [&plan[..], &staged[..], &["--check"]].concat(),
    ] {
        let ran = fixture.both_spellings(&["migrate", "promote"], &arguments);
        assert_eq!(
            ran.code, SUCCESS,
            "{arguments:?}:\n{}{}",
            ran.stdout, ran.stderr
        );
    }
    let ran = fixture.both_spellings(&["migrate", "promote"], &[&plan[..], &staged[..]].concat());
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert!(
        ran.tree
            .contains_key(Path::new("migration/repository/spec.nix")),
        "the case promotes nothing"
    );
}

// -----------------------------------------------------------------------------
// Task 2.3: a `get` operation writes nothing, and neither does a withholding option
// -----------------------------------------------------------------------------

/// Asserts that both spellings agree on `arguments`, and that neither wrote anything.
fn writes_nothing(fixture: &Fixture, openspec: &[&str], arguments: &[&str]) {
    let ran = fixture.both_spellings(openspec, arguments);
    let wrote = fixture.writes(&ran);
    assert!(
        wrote.is_empty(),
        "`htmlspec {} {}` wrote:\n  {}",
        verb_spelling(openspec).join(" "),
        arguments.join(" "),
        wrote.join("\n  ")
    );
}

/// The toolchain-packaging scenario "A get operation writes nothing", for every `get` operation and
/// the options each accepts.
///
/// Each invocation runs under both spellings, so a `get` spelling that a parse error kept from
/// running cannot pass by having written nothing: it would differ from the OpenSpec spelling.
#[test]
fn every_get_operation_leaves_the_project_as_it_was() {
    let project = Fixture::new("get-project", Area::Project);
    let invocations: &[(&[&str], &[&str])] = &[
        (&["validate"], &[]),
        (&["validate"], &["--strict", "--json"]),
        (&["validate"], &["htmlspec/changes/demo"]),
        (&["list"], &[]),
        (&["list"], &["--specs", "--json"]),
        (&["list"], &["--runbooks"]),
        (&["status"], &["--change", "demo", "--json"]),
        (&["sync"], &["--change", "demo"]),
        (&["sync"], &["--change", "demo", "--json"]),
        (&["show"], &["demo", "--diff"]),
        (&["show"], &["demo", "--diff", "--json"]),
        (&["show"], &["demo"]),
        (&["assets"], &[]),
        (&["assets"], &["--json"]),
        (&["preflight"], &["release"]),
        (&["preflight"], &["release", "--json"]),
        (&["instructions"], &["archive"]),
        (&["instructions"], &["sync", "--json"]),
        (&["store", "list"], &[]),
        (&["store", "list"], &["--json"]),
        (&["list"], &["--store", "demo-store"]),
    ];
    for (openspec, arguments) in invocations {
        writes_nothing(&project, openspec, arguments);
    }

    let source = Fixture::new("get-source", Area::Source);
    for arguments in [&[][..], &["--json"]] {
        writes_nothing(&source, &["migrate", "survey"], arguments);
    }

    // Every `get` operation the table lists is among those run above.
    let ran: Vec<&[&str]> = invocations
        .iter()
        .map(|(openspec, _)| *openspec)
        .chain([&["migrate", "survey"][..]])
        .collect();
    for (openspec, verb, object) in SPELLINGS {
        if *verb == "get" {
            assert!(
                ran.contains(openspec),
                "`get {object}` is never run by this test"
            );
        }
    }
}

/// Each option that withholds an operation's writes withholds them under the verb spelling too.
#[test]
fn a_withholding_option_writes_nothing_under_the_verb() {
    let project = Fixture::new("withheld-project", Area::Project);
    writes_nothing(&project, &["format"], &["--check"]);
    writes_nothing(&project, &["archive"], &["demo", "--check"]);
    writes_nothing(&project, &["run"], &["release", "--check"]);

    let migration = Fixture::new("withheld-migration", Area::Migration);
    writes_nothing(
        &migration,
        &["migrate", "promote"],
        &["plan.html", "--survey", "survey.json"],
    );
    writes_nothing(
        &migration,
        &["migrate", "promote"],
        &[
            "plan.html",
            "--survey",
            "survey.json",
            "--staged",
            "staging",
            "--into",
            "repository",
            "--check",
        ],
    );
}

/// `archive --check` settles an archival interrupted earlier before it reports, and `patch archive
/// --check` settles it the same way: the same report, and the same tree.
#[test]
fn both_spellings_of_the_archive_check_settle_an_interrupted_archival_alike() {
    let fixture = Fixture::new("interrupted", Area::Project);
    fixture.interrupt_an_archival();
    let ran = fixture.both_spellings(&["archive"], &["demo", "--check"]);
    assert_eq!(ran.code, SUCCESS, "{}{}", ran.stdout, ran.stderr);
    assert!(
        ran.stderr.contains("rolled back an interrupted archival"),
        "{}",
        ran.stderr
    );
    let mut settled = fixture.writes(&ran);
    settled.sort();
    assert_eq!(
        settled,
        vec![
            "removed project/htmlspec/.archive-journal.json",
            "removed project/htmlspec/changes/demo/VERSION",
            "removed project/htmlspec/changes/demo/runbooks.json",
        ],
        "the check did not settle the interrupted archival by rolling it back"
    );
}

// -----------------------------------------------------------------------------
// Task 2.4: help lists both spellings
// -----------------------------------------------------------------------------

/// The help a spelling prints with `--help`.
fn help(words: &[&str]) -> String {
    let scratch = std::env::temp_dir();
    let arguments: Vec<&str> = words.iter().copied().chain(["--help"]).collect();
    let (code, out, err) = run_in(
        &scratch,
        &scratch.join("htmlspec-verbs-no-config"),
        &arguments,
    );
    assert_eq!(
        code,
        SUCCESS,
        "`htmlspec {}` failed:\n{out}{err}",
        arguments.join(" ")
    );
    out
}

/// The entries of the section of `help` under `heading`: the first word of each indented line,
/// with the rest of the line.
fn section<'a>(help: &'a str, heading: &str) -> Vec<(&'a str, &'a str)> {
    help.lines()
        .skip_while(|line| *line != heading)
        .skip(1)
        .take_while(|line| line.starts_with("  ") || line.is_empty())
        .filter_map(|line| {
            let line = line.trim_start();
            let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
            (!word.is_empty()).then_some((word, rest.trim_start()))
        })
        .collect()
}

/// What `help` documents from its arguments onward: everything after the usage line.
fn options_of(help: &str) -> String {
    help.lines()
        .skip_while(|line| !line.starts_with("Usage:"))
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
}

/// `htmlspec --help` lists every operation by its OpenSpec spelling, as before, and then the five
/// verbs under their own heading, saying what each means and that they send nothing.
#[test]
fn the_top_level_help_lists_the_operations_and_then_the_verbs() {
    let top = help(&[]);
    let commands: Vec<&str> = section(&top, "Commands:")
        .iter()
        .map(|(word, _)| *word)
        .collect();
    for (openspec, _, _) in SPELLINGS {
        assert!(
            commands.contains(&openspec[0]),
            "the top-level help does not list `{}`:\n{top}",
            openspec[0]
        );
    }
    for verb in VERBS {
        assert!(
            !commands.contains(&verb),
            "`{verb}` is listed among the operations rather than under its own heading:\n{top}"
        );
    }
    let listed: Vec<(&str, &str)> = section(&top, "Request verbs:")
        .into_iter()
        .filter(|(word, _)| VERBS.contains(word))
        .collect();
    let verbs: Vec<&str> = listed.iter().map(|(word, _)| *word).collect();
    assert_eq!(
        verbs, VERBS,
        "the top-level help does not list the five verbs under their own heading:\n{top}"
    );
    // Each verb's line ends with its objects, every one of them and no other.
    for (verb, line) in listed {
        let mut objects: Vec<&str> = line
            .rsplit_once(": ")
            .map_or("", |(_, objects)| objects)
            .split(", ")
            .collect();
        objects.sort_unstable();
        let mut expected: Vec<&str> = SPELLINGS
            .iter()
            .filter(|(_, listed_verb, _)| *listed_verb == verb)
            .map(|(_, _, object)| *object)
            .collect();
        expected.sort_unstable();
        assert_eq!(
            objects, expected,
            "the top-level help lists other objects for `{verb}`:\n{top}"
        );
    }
    let commands_at = top.find("Commands:").unwrap_or(usize::MAX);
    let verbs_at = top.find("Request verbs:").unwrap_or(0);
    assert!(
        commands_at < verbs_at,
        "the verbs are not listed after the operations:\n{top}"
    );
    assert!(top.contains("send nothing"), "{top}");
}

/// Each verb's help lists exactly its objects, each with the operation it runs.
#[test]
fn each_verbs_help_lists_its_objects_with_the_operation_each_runs() {
    for verb in VERBS {
        let shown = help(&[verb]);
        let objects = section(&shown, "Commands:");
        let mut listed: Vec<&str> = objects
            .iter()
            .map(|(object, _)| *object)
            .filter(|object| *object != "help")
            .collect();
        listed.sort_unstable();
        let mut expected: Vec<&str> = SPELLINGS
            .iter()
            .filter(|(_, listed_verb, _)| *listed_verb == verb)
            .map(|(_, _, object)| *object)
            .collect();
        expected.sort_unstable();
        assert_eq!(
            listed, expected,
            "`htmlspec {verb} --help` lists other objects:\n{shown}"
        );
        for (openspec, _, object) in SPELLINGS.iter().filter(|(_, v, _)| *v == verb) {
            let runs = format!("`htmlspec {}`", openspec.join(" "));
            let line = objects
                .iter()
                .find(|(listed, _)| listed == object)
                .map_or("", |(_, rest)| *rest);
            assert!(
                line.contains(&runs),
                "`htmlspec {verb} --help` does not say that `{object}` runs {runs}:\n{shown}"
            );
        }
        assert!(shown.contains("send nothing"), "{shown}");
    }
}

/// An object's own help shows the arguments and options its operation's help shows.
#[test]
fn an_objects_help_shows_its_operations_options() {
    for (openspec, verb, object) in SPELLINGS {
        let by_name = options_of(&help(openspec));
        let by_verb = options_of(&help(&[verb, object]));
        assert_eq!(
            by_name,
            by_verb,
            "`htmlspec {verb} {object} --help` documents other options than `htmlspec {} --help`",
            openspec.join(" ")
        );
        assert!(
            !by_name.is_empty(),
            "no options were read from {openspec:?}"
        );
    }
}
