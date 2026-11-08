//! The release check on scratch repositories: one that passes, and each question broken once.

use super::*;
use crate::scratch::Scratch;
use crate::workspace::{Dependency, DependencyKind, PackageVersion};

/// A fingerprint of the shape an inventory holds, distinct for each `seed`.
fn digest(seed: u8) -> String {
    format!("sha256:{}", format!("{seed:02x}").repeat(32))
}

/// An inventory holding one runbook, `release`, at `version`, whose specification has `seed`'s
/// fingerprint, and whose changelog heads `heading`.
fn inventory(version: &str, seed: u8, heading: &str) -> String {
    serde_json::json!({
        "format": "htmlspec-runbook-inventory/v1",
        "runbooks": [{
            "name": "release",
            "version": version,
            "specification": digest(seed),
            "procedure": digest(0xee),
            "changelog": format!("# Changelog\n\n## {heading}\n\nA release.\n"),
        }],
    })
    .to_string()
}

/// The baseline the scratch repositories share 0.1.0 under, as this repository does.
fn baseline() -> ProjectVersion {
    ProjectVersion::parse("0.1.0").expect("a version")
}

fn name(name: &str) -> CrateName {
    CrateName::parse(name).expect("a crate name")
}

fn dependency(on: &str, requirement: &str, kind: DependencyKind) -> Dependency {
    Dependency {
        name: name(on),
        kind,
        requirement: Requirement::parse(requirement),
        reaches_the_registry: !(kind == DependencyKind::Development && requirement == "*"),
    }
}

fn member(root: &Path, called: &str, publishable: bool, dependencies: Vec<Dependency>) -> Member {
    Member {
        name: name(called),
        version: PackageVersion::parse("0.2.0").expect("a version"),
        directory: root.join("crates").join(called),
        publishable,
        repository: None,
        dependencies,
    }
}

/// A repository whose record is coherent at 0.2.0: two changes share the 0.1.0 baseline, a third
/// records 0.2.0, the changelog heads both, a runbook is published at 0.1.0 from adoption at 0.2.0,
/// and two published crates carry the licence and pin each other, beside an unpublished harness
/// that names one by path alone.
fn repository(scratch: &Scratch) -> Workspace {
    let root = scratch.path();
    scratch.write("LICENSE", "The licence.\n");
    scratch.write("crates/one/LICENSE", "The licence.\n");
    scratch.write("crates/two/LICENSE", "The licence.\n");
    scratch.write(
        "CHANGELOG.md",
        "# Changelog\n\n## 0.2.0 — the second\n\nProse naming 9.9.9.\n\n## 0.1.0\n\nThe baseline.\n",
    );
    scratch.write("htmlspec/changes/archive/2026-01-01-a/VERSION", "0.1.0\n");
    scratch.write("htmlspec/changes/archive/2026-01-02-b/VERSION", "0.1.0\n");
    scratch.write("htmlspec/changes/archive/2026-01-03-c/VERSION", "0.2.0\n");
    scratch.write(
        "htmlspec/changes/archive/2026-01-03-c/runbooks.json",
        &inventory("0.1.0", 1, "0.1.0"),
    );
    scratch.write(
        "htmlspec/runbook-releases.json",
        r#"{"format": "htmlspec-runbook-adoption/v1", "version": "0.2.0"}"#,
    );
    scratch.write(
        "htmlspec/runbooks/release/specification.html",
        "<p>spec</p>\n",
    );
    scratch.write("htmlspec/runbooks/release/tasks.html", "<p>tasks</p>\n");
    scratch.write("htmlspec/runbooks/release/VERSION", "0.1.0\n");
    scratch.write(
        "htmlspec/runbooks/release/CHANGELOG.md",
        "# Changelog\n\n## 0.1.0\n\nFirst.\n",
    );
    Workspace {
        root: root.to_path_buf(),
        members: vec![
            member(root, "one", true, Vec::new()),
            member(
                root,
                "two",
                true,
                vec![
                    dependency("one", "=0.2.0", DependencyKind::Normal),
                    dependency("one", "=0.2.0", DependencyKind::Development),
                    dependency("serde", "^1.0", DependencyKind::Normal),
                ],
            ),
            member(
                root,
                "harness",
                false,
                vec![dependency("one", "*", DependencyKind::Normal)],
            ),
        ],
    }
}

/// The faults a check of `workspace` found, as it prints them, without the `!!! ` each begins with.
fn faults(workspace: &Workspace, tag: Option<&str>) -> Vec<String> {
    let report = check(workspace, baseline(), tag);
    assert_eq!(report.exit, Exit::Faults, "{report:?}");
    report
        .err
        .iter()
        .filter_map(|line| line.strip_prefix("!!! "))
        .map(str::to_owned)
        .collect()
}

/// One fault, whose words hold each of `holding`.
fn one_fault(workspace: &Workspace, tag: Option<&str>, holding: &[&str]) {
    let found = faults(workspace, tag);
    assert_eq!(found.len(), 1, "{found:#?}");
    for words in holding {
        assert!(
            found[0].contains(words),
            "{words:?} is not in {:?}",
            found[0]
        );
    }
}

#[test]
fn a_coherent_record_passes_and_says_what_it_counted() {
    let scratch = Scratch::new("release-coherent");
    let workspace = repository(&scratch);
    let report = check(&workspace, baseline(), Some("v0.2.0"));
    assert_eq!(
        report,
        Report {
            exit: Exit::Clean,
            out: vec![
                String::from(
                    "  ok  3 archived change(s) record a version, and 2 version(s) are in the \
                     changelog"
                ),
                String::from("  ok  declared version 0.2.0 has a changelog entry"),
                String::from("  ok  1 internal dependency pin(s) name =0.2.0"),
                String::from("  ok  2 published crate(s) carry the root LICENSE"),
                String::from("  ok  tag v0.2.0 names the declared version"),
            ],
            err: Vec::new(),
        }
    );
    assert_eq!(report.exit.code(), 0);
}

/// Faults go to standard error, each named, then a count, and the run exits 1.
#[test]
fn a_fault_is_named_on_standard_error_and_exits_one() {
    let scratch = Scratch::new("release-exit-one");
    let workspace = repository(&scratch);
    scratch.remove("htmlspec/changes/archive/2026-01-02-b/VERSION");
    let report = check(&workspace, baseline(), None);
    assert_eq!(report.exit.code(), 1);
    assert!(report.out.is_empty(), "{report:?}");
    assert_eq!(
        report.err,
        vec![
            String::from("release hygiene:"),
            String::from(
                "!!! 2026-01-02-b records no version. Add VERSION naming the version it shipped in, \
                 so the release it belongs to is recoverable"
            ),
            String::new(),
            String::from("1 fault(s) across 3 archived change(s), declared version 0.2.0"),
        ]
    );
}

/// A question the check cannot ask exits 2, naming why, and says nothing of the record.
#[test]
fn what_cannot_be_asked_exits_two() {
    let scratch = Scratch::new("release-exit-two");
    let workspace = repository(&scratch);

    let mut versions = workspace.clone();
    versions.members[1].version = PackageVersion::parse("0.3.0").expect("a version");
    let mut unpublished = workspace.clone();
    for member in &mut unpublished.members {
        member.publishable = false;
    }
    let mut prerelease = workspace.clone();
    for member in &mut prerelease.members {
        member.version = PackageVersion::parse("0.2.0-rc.1").expect("a version");
    }
    for (workspace, words) in [
        (&versions, "more than one version, 0.2.0 and 0.3.0"),
        (&unpublished, "publishes no crate"),
        (&prerelease, "no parseable version in"),
    ] {
        let report = check(workspace, baseline(), None);
        assert_eq!(report.exit, Exit::CouldNotAsk, "{report:?}");
        assert_eq!(report.exit.code(), 2);
        assert!(report.out.is_empty());
        assert_eq!(report.err.len(), 1, "{report:?}");
        assert!(report.err[0].contains(words), "{report:?}");
    }

    // An archive that is a file cannot be listed, which is not the same answer as no archive.
    let archive = scratch.path().join("htmlspec/changes/archive");
    std::fs::remove_dir_all(&archive).expect("the archive is removed");
    std::fs::write(&archive, "not a directory\n").expect("written");
    let report = check(&workspace, baseline(), None);
    assert_eq!(report.exit, Exit::CouldNotAsk);
    assert!(report.err[0].contains("cannot be read"), "{report:?}");

    std::fs::remove_file(&archive).expect("the file is removed");
    let report = check(&workspace, baseline(), None);
    assert_eq!(report.exit, Exit::CouldNotAsk);
    assert!(
        report.err[0].contains("no archived changes under"),
        "{report:?}"
    );

    let nowhere = Scratch::new("release-no-project");
    let mut elsewhere = workspace.clone();
    elsewhere.root = nowhere.path().to_path_buf();
    let report = check(&elsewhere, baseline(), None);
    assert_eq!(report.exit, Exit::CouldNotAsk);
    assert!(
        report.err[0].contains("so there is no project here"),
        "{report:?}"
    );
}

/// The baseline is the one the check is given, so a repository whose shared release is another
/// version is checked against its own.
#[test]
fn the_baseline_is_the_one_the_check_is_given() {
    let scratch = Scratch::new("release-baseline");
    let workspace = repository(&scratch);
    let below = ProjectVersion::parse("0.0.9").expect("a version");
    let report = check(&workspace, below, None);
    assert_eq!(report.exit, Exit::Faults, "{report:?}");
    assert!(
        report.err[1]
            .contains("2026-01-02-b records version 0.1.0, which 2026-01-01-a already records"),
        "{report:?}"
    );
}

/// Question 1, every archived change records a version: none at all, and one the one grammar does
/// not read, `0.1`, which the Python check read as a version.
#[test]
fn an_archived_change_that_records_no_version_fails() {
    let scratch = Scratch::new("release-q1");
    let workspace = repository(&scratch);
    scratch.write("htmlspec/changes/archive/2026-01-02-b/VERSION", "0.1\n");
    one_fault(
        &workspace,
        None,
        &[
            "2026-01-02-b records \"0.1\" in VERSION, which is not a version",
            "this has 2 component(s)",
        ],
    );
    scratch.remove("htmlspec/changes/archive/2026-01-02-b/VERSION");
    one_fault(&workspace, None, &["2026-01-02-b records no version"]);
}

/// Question 2, distinct versions above the baseline, while the baseline's 0.1.0 is shared.
#[test]
fn a_version_two_changes_record_above_the_baseline_fails() {
    let scratch = Scratch::new("release-q2");
    let workspace = repository(&scratch);
    scratch.write("htmlspec/changes/archive/2026-01-04-d/VERSION", "0.2.0\n");
    scratch.write(
        "htmlspec/changes/archive/2026-01-04-d/runbooks.json",
        &inventory("0.1.0", 1, "0.1.0"),
    );
    one_fault(
        &workspace,
        None,
        &["2026-01-04-d records version 0.2.0, which 2026-01-03-c already records"],
    );
}

/// Question 3, a changelog heading for every recorded version and the declared one, read through
/// the one reading of a heading.
#[test]
fn a_version_no_changelog_heading_names_fails() {
    let scratch = Scratch::new("release-q3");
    let workspace = repository(&scratch);
    // A heading that the Python check read as 0.2.0, and the one reading reads as nothing.
    scratch.write(
        "CHANGELOG.md",
        "# Changelog\n\n## 0.2.0.\n\nThe second.\n\n## 0.1.0\n\nThe baseline.\n",
    );
    one_fault(
        &workspace,
        None,
        &["no CHANGELOG.md heading names version 0.2.0"],
    );
    scratch.remove("CHANGELOG.md");
    let found = faults(&workspace, None);
    assert_eq!(found[0], "no CHANGELOG.md at the repository root. Every version the project ships records what changed in it");
    assert_eq!(found.len(), 3, "{found:#?}");
}

/// Question 4, for a runbook the project holds: a version the one grammar reads, a release, and a
/// heading for it.
#[test]
fn a_runbook_that_cannot_name_its_release_fails() {
    let scratch = Scratch::new("release-q4-held");
    let workspace = repository(&scratch);
    scratch.write("htmlspec/runbooks/release/VERSION", "0.1\n");
    one_fault(
        &workspace,
        None,
        &[
            "the runbook release at htmlspec/runbooks/release cannot be read as a release",
            "VERSION does not hold a version",
        ],
    );
    scratch.write("htmlspec/runbooks/release/VERSION", "0.2.0\n");
    one_fault(
        &workspace,
        None,
        &["the runbook release declares 0.2.0 and its CHANGELOG.md has no heading naming it"],
    );
    scratch.write("htmlspec/runbooks/release/VERSION", "0.0.0\n");
    scratch.write(
        "htmlspec/runbooks/release/CHANGELOG.md",
        "# Changelog\n\n## 0.0.0\n\nNot yet.\n",
    );
    one_fault(
        &workspace,
        None,
        &["declares 0.0.0, which is not a release"],
    );
}

/// Question 4, for the published history, read as archival reads it: an inventory entry at `0.1`,
/// which the Python reader accepted, and a snapshot that moves a runbook backwards.
#[test]
fn a_published_history_archival_would_refuse_fails() {
    let scratch = Scratch::new("release-q4-history");
    let workspace = repository(&scratch);
    scratch.write(
        "htmlspec/changes/archive/2026-01-03-c/runbooks.json",
        &inventory("0.1", 1, "0.1.0"),
    );
    one_fault(
        &workspace,
        None,
        &[
            "the archive 2026-01-03-c carries an unreadable runbooks.json",
            "release's version is not one",
        ],
    );

    scratch.write(
        "htmlspec/changes/archive/2026-01-03-c/runbooks.json",
        &inventory("0.2.0", 1, "0.2.0"),
    );
    scratch.write("htmlspec/changes/archive/2026-01-04-d/VERSION", "0.3.0\n");
    scratch.write(
        "htmlspec/changes/archive/2026-01-04-d/runbooks.json",
        &inventory("0.1.0", 2, "0.1.0"),
    );
    let found = faults(&workspace, None);
    assert!(
        found.iter().any(|fault| fault.starts_with(
            "2026-01-04-d: the runbook release declares \
             0.1.0, which is ordered before the 0.2.0 that 2026-01-03-c published"
        )),
        "{found:#?}"
    );
}

/// Question 5, every internal pin of a published crate exact: a stale one, and one naming none,
/// while an unpublished member's dependency by path alone is no pin.
#[test]
fn a_pin_that_is_not_the_declared_version_fails() {
    let scratch = Scratch::new("release-q5");
    let mut workspace = repository(&scratch);
    workspace.members[1].dependencies[0].requirement = Requirement::parse("=0.1.0");
    one_fault(
        &workspace,
        None,
        &[
            "one is pinned at \"=0.1.0\" where two depend(s) on it, and the workspace declares 0.2.0",
            "cargo xtask set-version",
        ],
    );
    workspace.members[1].dependencies[0].requirement = Requirement::parse("*");
    one_fault(
        &workspace,
        None,
        &["one names no version where two depend(s) on it. Add version = \"=0.2.0\""],
    );
}

/// Question 6, the licence in every published crate, byte for byte.
#[test]
fn a_published_crate_without_the_root_s_licence_fails() {
    let scratch = Scratch::new("release-q6");
    let workspace = repository(&scratch);
    scratch.write("crates/two/LICENSE", "Another licence.\n");
    one_fault(
        &workspace,
        None,
        &["crates/two/LICENSE differs from the root LICENSE"],
    );
    scratch.remove("crates/two/LICENSE");
    one_fault(&workspace, None, &["crates/two/LICENSE is missing"]);
    scratch.remove("LICENSE");
    one_fault(&workspace, None, &["no LICENSE at the repository root"]);
    // A root licence that is there and cannot be read is not one that is missing.
    std::fs::create_dir(scratch.path().join("LICENSE")).expect("made");
    one_fault(&workspace, None, &["the root LICENSE cannot be read"]);
}

/// Question 7, a tag naming exactly the declared version as a release writes it. `v00.2.0` passed
/// the Python check, which read its numbers and not its text.
#[test]
fn a_tag_that_does_not_name_the_declared_version_fails() {
    let scratch = Scratch::new("release-q7");
    let workspace = repository(&scratch);
    one_fault(
        &workspace,
        Some("v0.2.1"),
        &["tag v0.2.1 names 0.2.1, and Cargo.toml declares 0.2.0"],
    );
    for tag in [
        "0.2.0",
        "v0.2",
        "v0.2.0.0",
        "v00.2.0",
        "v0.2.0-rc.1",
        "release-0.2.0",
    ] {
        one_fault(
            &workspace,
            Some(tag),
            &[&format!("tag {tag} is not a release tag")],
        );
    }
}
