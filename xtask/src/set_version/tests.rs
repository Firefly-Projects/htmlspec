//! The edit, the release runbook's two scenarios on a scratch workspace with the real cargo, and
//! the restore on a cargo that fails.

use std::cell::RefCell;

use super::*;
use crate::scratch::Scratch;

fn version(text: &str) -> ReleaseVersion {
    ReleaseVersion::parse(text).expect("a release version")
}

/// The manifest's shape: the version, a pin with a version, a pin without one, another table's
/// `version`, and a dependency that is not internal.
const MANIFEST_TEXT: &str = "\
[workspace]
members = [\"crates/*\"]

[workspace.package]
version = \"0.32.0\"
edition = \"2021\"

[workspace.dependencies]
serde = { version = \"1.0\", features = [\"derive\"] }
htmlspec-parse = { version = \"=0.32.0\", path = \"crates/htmlspec-parse\" }
htmlspec-format = { path = \"crates/htmlspec-format\" }

[workspace.lints.rust]
version = \"not a version\"
";

#[test]
fn the_version_and_every_internal_pin_move_together() {
    let edited = set_version(MANIFEST_TEXT, version("0.32.1")).expect("edited");
    assert_eq!(
        edited.text,
        MANIFEST_TEXT
            .replace("version = \"0.32.0\"", "version = \"0.32.1\"")
            .replace("\"=0.32.0\"", "\"=0.32.1\"")
            .replace(
                "htmlspec-format = { path",
                "htmlspec-format = { version = \"=0.32.1\", path"
            )
    );
    assert_eq!(edited.pinned, vec!["htmlspec-parse", "htmlspec-format"]);
}

/// The same version again repairs a stale pin and changes nothing else.
#[test]
fn the_version_already_declared_repairs_a_stale_pin() {
    let stale = MANIFEST_TEXT.replace("\"=0.32.0\"", "\"=0.31.0\"");
    let edited = set_version(&stale, version("0.32.0")).expect("edited");
    assert_eq!(
        edited.text,
        MANIFEST_TEXT.replace(
            "htmlspec-format = { path",
            "htmlspec-format = { version = \"=0.32.0\", path"
        )
    );
}

#[test]
fn a_manifest_with_no_version_or_two_is_refused() {
    let none = MANIFEST_TEXT.replace("version = \"0.32.0\"\n", "");
    assert_eq!(
        set_version(&none, version("0.32.1")),
        Err(ManifestFault::PackageVersions { found: 0 })
    );
    let two = MANIFEST_TEXT.replace(
        "edition = \"2021\"",
        "edition = \"2021\"\nversion = \"0.1.0\"",
    );
    assert_eq!(
        set_version(&two, version("0.32.1")),
        Err(ManifestFault::PackageVersions { found: 2 })
    );
}

/// This repository's own manifest: its version and its five pins, and not one other byte.
#[test]
fn this_workspace_s_manifest_moves_its_version_and_five_pins() {
    let manifest = include_str!("../../../Cargo.toml");
    let edited = set_version(manifest, version("9.8.7")).expect("edited");
    assert_eq!(edited.pinned.len(), 5, "{:?}", edited.pinned);
    let before: Vec<&str> = manifest.lines().collect();
    let after: Vec<&str> = edited.text.lines().collect();
    assert_eq!(before.len(), after.len());
    let changed: Vec<(&str, &str)> = before
        .into_iter()
        .zip(after)
        .filter(|(one, other)| one != other)
        .collect();
    assert_eq!(changed.len(), 6, "{changed:#?}");
    assert!(
        changed[0].1.starts_with("version = \"9.8.7\""),
        "{changed:#?}"
    );
    for (_, line) in &changed[1..] {
        assert!(line.contains("version = \"=9.8.7\""), "{line}");
    }
}

/// A scratch workspace of two internal crates at `at`, the second depending on the first through
/// `[workspace.dependencies]`, and its lockfile, which cargo writes offline: neither has a
/// dependency outside the workspace.
fn scratch_workspace(scratch: &Scratch, at: &str) {
    scratch.write(
        "Cargo.toml",
        &format!(
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/*\"]\n\n[workspace.package]\n\
             version = \"{at}\"\nedition = \"2021\"\n\n[workspace.dependencies]\n\
             htmlspec-a = {{ version = \"={at}\", path = \"crates/htmlspec-a\" }}\n"
        ),
    );
    for (name, dependencies) in [
        ("htmlspec-a", ""),
        (
            "htmlspec-b",
            "\n[dependencies]\nhtmlspec-a = { workspace = true }\n",
        ),
    ] {
        scratch.write(
            &format!("crates/{name}/Cargo.toml"),
            &format!(
                "[package]\nname = \"{name}\"\nversion.workspace = true\n\
                 edition.workspace = true\n{dependencies}"
            ),
        );
        scratch.write(&format!("crates/{name}/src/lib.rs"), "");
    }
    let status = Command::new(crate::workspace::cargo())
        .args(["generate-lockfile", "--offline", "--quiet"])
        .current_dir(scratch.path())
        .status()
        .expect("cargo starts");
    assert!(status.success(), "the scratch lockfile was not written");
}

/// The release runbook's "The version raised by hand, its pins left behind": raised under
/// `[workspace.package]` alone, the relock fails.
#[test]
fn a_version_raised_by_hand_fails_the_relock() {
    let scratch = Scratch::new("set-version-by-hand");
    scratch_workspace(&scratch, "0.32.0");
    let manifest = scratch.read("Cargo.toml");
    scratch.write(
        "Cargo.toml",
        &manifest.replace("version = \"0.32.0\"", "version = \"0.32.1\""),
    );
    assert_eq!(
        RealCargo.relock(&scratch.path().join(MANIFEST)),
        Err(Relock::Exited(Some(101)))
    );
}

/// The same scenario's other half: the tool writes the pin and relocks, and the lockfile changes in
/// the workspace's own entries alone.
#[test]
fn the_tool_writes_every_pin_and_relocks() {
    let scratch = Scratch::new("set-version-relocks");
    scratch_workspace(&scratch, "0.32.0");
    let lock_before = scratch.read("Cargo.lock");
    let report = set(scratch.path(), version("0.32.1"), &RealCargo);
    assert_eq!(
        report,
        Report {
            exit: Exit::Clean,
            out: vec![
                String::from("version 0.32.1, and 1 pin(s): htmlspec-a"),
                String::from("Cargo.lock relocked offline"),
            ],
            err: Vec::new(),
        }
    );
    assert!(scratch
        .read("Cargo.toml")
        .contains("htmlspec-a = { version = \"=0.32.1\""));
    assert_eq!(
        scratch.read("Cargo.lock"),
        lock_before.replace("version = \"0.32.0\"", "version = \"0.32.1\"")
    );
}

/// A cargo that reads the workspace as `workspace` says, and fails the relock of the lockfile
/// `failing` names after scribbling on it, as a cargo that died part way might.
struct FailingCargo {
    workspace: Result<Workspace, MetadataFault>,
    failing: Option<&'static str>,
    relocked: RefCell<Vec<PathBuf>>,
}

impl Cargo for FailingCargo {
    fn metadata(&self, _root: &Path) -> Result<Workspace, MetadataFault> {
        self.workspace.clone()
    }

    fn relock(&self, manifest: &Path) -> Result<(), Relock> {
        self.relocked.borrow_mut().push(manifest.to_path_buf());
        let lockfile = manifest.with_file_name(LOCKFILE);
        std::fs::write(&lockfile, "scribbled").expect("the lockfile is written");
        match self.failing {
            Some(failing) if lockfile.ends_with(failing) => Err(Relock::Exited(Some(101))),
            _ => Ok(()),
        }
    }
}

/// The workspace `cargo metadata` would read after the edit, at `at`.
fn read_back(at: &str) -> Workspace {
    Workspace::parse(&crate::workspace::tests::metadata(
        &[crate::workspace::tests::Sketch {
            name: "htmlspec-a",
            publish: true,
            dependencies: &[],
        }],
        at,
        "https://github.com/example/example",
    ))
    .expect("the sketch reads")
}

/// A failed step puts back every file the run may have written, and exits 1 saying so.
#[test]
fn a_failure_after_the_edit_restores_every_file() {
    let scratch = Scratch::new("set-version-restores");
    scratch.write("Cargo.toml", MANIFEST_TEXT);
    scratch.write("Cargo.lock", "the root lockfile\n");
    scratch.write("publish/Cargo.toml", "[workspace]\n");
    scratch.write("publish/Cargo.lock", "the private lockfile\n");

    for (cargo, words) in [
        (
            FailingCargo {
                workspace: Ok(read_back("0.32.1")),
                failing: Some(PRIVATE_LOCKFILE),
                relocked: RefCell::new(Vec::new()),
            },
            "cargo update --workspace --offline --manifest-path publish/Cargo.toml exited 101",
        ),
        (
            FailingCargo {
                workspace: Ok(read_back("0.32.1")),
                failing: Some(LOCKFILE),
                relocked: RefCell::new(Vec::new()),
            },
            "cargo update --workspace --offline exited 101",
        ),
        (
            FailingCargo {
                workspace: Ok(read_back("0.32.0")),
                failing: None,
                relocked: RefCell::new(Vec::new()),
            },
            "the edited manifest does not name the version everywhere: htmlspec-a is at 0.32.0",
        ),
        (
            FailingCargo {
                workspace: Err(MetadataFault::Failed {
                    stderr: String::from("bad manifest"),
                }),
                failing: None,
                relocked: RefCell::new(Vec::new()),
            },
            "the edited manifest does not read: cargo metadata failed: bad manifest",
        ),
    ] {
        let report = set(scratch.path(), version("0.32.1"), &cargo);
        assert_eq!(report.exit, Exit::Faults, "{report:?}");
        assert_eq!(report.exit.code(), 1);
        assert_eq!(report.err[0], format!("set-version: {words}"), "{report:?}");
        assert_eq!(
            report.err.last().map(String::as_str),
            Some("set-version: Cargo.toml, Cargo.lock, and publish/Cargo.lock are restored as they were")
        );
        assert_eq!(scratch.read("Cargo.toml"), MANIFEST_TEXT);
        assert_eq!(scratch.read("Cargo.lock"), "the root lockfile\n");
        assert_eq!(scratch.read("publish/Cargo.lock"), "the private lockfile\n");
    }
}

/// A file is replaced whole, through a temporary file beside it that does not outlive the
/// replacement, and a replacement that cannot be made leaves the file as it was.
#[test]
fn a_file_is_replaced_whole_and_leaves_nothing_beside_it() {
    let scratch = Scratch::new("set-version-replace");
    scratch.write("Cargo.toml", "before\n");
    replace(&scratch.path().join("Cargo.toml"), b"after\n").expect("replaced");
    assert_eq!(scratch.read("Cargo.toml"), "after\n");
    let names: Vec<String> = std::fs::read_dir(scratch.path())
        .expect("listed")
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec![String::from("Cargo.toml")]);

    // A directory where the file would go cannot be renamed over.
    std::fs::create_dir(scratch.path().join("Cargo.lock")).expect("made");
    assert!(replace(&scratch.path().join("Cargo.lock"), b"lock\n").is_err());
    let mut names: Vec<String> = std::fs::read_dir(scratch.path())
        .expect("listed")
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![String::from("Cargo.lock"), String::from("Cargo.toml")]
    );
}

/// Where the private workspace exists, its lockfile is relocked after the root's.
#[test]
fn the_private_lockfile_is_relocked_where_it_exists() {
    let scratch = Scratch::new("set-version-private");
    scratch.write("Cargo.toml", MANIFEST_TEXT);
    scratch.write("Cargo.lock", "the root lockfile\n");
    let cargo = FailingCargo {
        workspace: Ok(read_back("0.32.1")),
        failing: None,
        relocked: RefCell::new(Vec::new()),
    };
    let report = set(scratch.path(), version("0.32.1"), &cargo);
    assert_eq!(report.exit, Exit::Clean, "{report:?}");
    assert_eq!(cargo.relocked.borrow().len(), 1);

    scratch.write("publish/Cargo.toml", "[workspace]\n");
    scratch.write("publish/Cargo.lock", "the private lockfile\n");
    cargo.relocked.borrow_mut().clear();
    let report = set(scratch.path(), version("0.32.1"), &cargo);
    assert_eq!(report.exit, Exit::Clean, "{report:?}");
    assert_eq!(
        report.out.last().map(String::as_str),
        Some("publish/Cargo.lock relocked offline")
    );
    assert_eq!(
        *cargo.relocked.borrow(),
        vec![
            scratch.path().join("Cargo.toml"),
            scratch.path().join("publish/Cargo.toml")
        ]
    );
}

/// A manifest the tool cannot read or edit exits 2, and writes nothing.
#[test]
fn a_manifest_it_cannot_edit_exits_two() {
    let scratch = Scratch::new("set-version-cannot");
    let cargo = FailingCargo {
        workspace: Ok(read_back("0.32.1")),
        failing: None,
        relocked: RefCell::new(Vec::new()),
    };
    let report = set(scratch.path(), version("0.32.1"), &cargo);
    assert_eq!(report.exit, Exit::CouldNotAsk);
    assert_eq!(report.exit.code(), 2);
    assert!(
        report.err[0].starts_with("set-version: cannot read"),
        "{report:?}"
    );

    let none = MANIFEST_TEXT.replace("version = \"0.32.0\"\n", "");
    scratch.write("Cargo.toml", &none);
    let report = set(scratch.path(), version("0.32.1"), &cargo);
    assert_eq!(report.exit, Exit::CouldNotAsk);
    assert!(
        report.err[0].ends_with("expected one version under [workspace.package], found 0"),
        "{report:?}"
    );
    assert_eq!(scratch.read("Cargo.toml"), none);
    assert!(cargo.relocked.borrow().is_empty());
}
