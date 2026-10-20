//! Task 4.4: the project-level asset fixtures.
//!
//! A conformance fixture in `conformance.rs` is a *document*, and the harness there
//! asserts what the validator reports about it. `asset/duplicate` has no document to
//! attach to — its subject is a project, and the same document is conforming or not
//! depending only on what sits in the directory around it. So its fixtures are
//! *projects*: directory trees built here, checked through the public API, and asserted
//! against a declared expectation.
//!
//! Built rather than committed, which is the one place this suite departs from the
//! document fixtures. A committed project fixture would mean committing duplicate asset
//! bytes on purpose, and `self-assets` in `scripts/check` runs over the whole repository
//! — so a fixture pinning duplication would be found by the check it exists to test, and
//! the suite would fail on its own fixtures. Each project is therefore constructed in a
//! temporary directory and removed when the test ends.
//!
//! Everything drives the public API only, exactly as `conformance.rs` does: the store is
//! discovered through [`htmlspec_assets::Store`] and judged through
//! [`htmlspec_validate::rules::topology`], with no reach into a private module.

use std::path::{Path, PathBuf};

use htmlspec_assets::{address, sha256_hex, Store};
use htmlspec_validate::rules::topology;

/// A project fixture: a tree of asset files, and what the rule must report about it.
struct ProjectFixture {
    /// Why this fixture exists, in one line.
    note: &'static str,
    /// How many duplicate sets the rule must report.
    duplicate_sets: usize,
    /// Every location holding bytes, as `(path relative to assets/, bytes)`.
    ///
    /// A path containing no `/` and no address prefix is an unaddressed file, which is
    /// where real duplication lives. `addressed` below writes the addressed form.
    files: &'static [(&'static str, &'static [u8])],
    /// Names whose bytes are stored at their own address, rather than at a literal path.
    addressed: &'static [(&'static str, &'static [u8])],
}

const MERMAID: &[u8] = b"the mermaid library, one revision";
const RUNTIME: &[u8] = b"the shared enhancement script";
const STYLESHEET_NEW: &[u8] = b"--hs-measure: 36rem;\n";
const STYLESHEET_OLD: &[u8] = b"--hs-measure: 34rem;\n";

const FIXTURES: &[(&str, ProjectFixture)] = &[
    (
        "one-store-serving-many-documents",
        ProjectFixture {
            note: "the arrangement the project wants: one addressed copy of each asset, \
                   shared by every document",
            duplicate_sets: 0,
            files: &[],
            addressed: &[("mermaid.min.js", MERMAID), ("htmlspec.js", RUNTIME)],
        },
    ),
    (
        "distinct-revisions-are-not-duplicates",
        ProjectFixture {
            note: "two revisions of one stylesheet whose bytes differ; content \
                   addressing makes coexistence the default rather than a conflict",
            duplicate_sets: 0,
            files: &[],
            addressed: &[
                ("htmlspec.css", STYLESHEET_NEW),
                ("htmlspec.css", STYLESHEET_OLD),
            ],
        },
    ),
    (
        "duplicate-copies-of-one-library",
        ProjectFixture {
            note: "the same library bytes at an address and in a legacy vendor tree, \
                   which is the duplication this project currently has",
            duplicate_sets: 1,
            files: &[("vendor/mermaid.min.js", MERMAID)],
            addressed: &[("mermaid.min.js", MERMAID)],
        },
    ),
    (
        "a-copy-per-document",
        ProjectFixture {
            note: "each document carrying its own copy of the runtime, which violates \
                   the shared-runtime obligation the store exists to keep",
            duplicate_sets: 1,
            files: &[
                ("proposal/htmlspec.js", RUNTIME),
                ("design/htmlspec.js", RUNTIME),
                ("review/htmlspec.js", RUNTIME),
            ],
            addressed: &[("htmlspec.js", RUNTIME)],
        },
    ),
    (
        "two-libraries-each-duplicated",
        ProjectFixture {
            note: "two separate duplications are two separate reports, so a contributor \
                   fixing one is not told the project is now clean",
            duplicate_sets: 2,
            files: &[
                ("vendor/mermaid.min.js", MERMAID),
                ("vendor/htmlspec.js", RUNTIME),
            ],
            addressed: &[("mermaid.min.js", MERMAID), ("htmlspec.js", RUNTIME)],
        },
    ),
];

/// A scratch project, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "htmlspec-topology-fixture-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("assets")).expect("the scratch store is created");
        Self(path)
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.0.join("assets").join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory");
        std::fs::write(&path, bytes).expect("the file is written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Builds one fixture's project and returns its store.
fn build(name: &str, fixture: &ProjectFixture) -> (Scratch, Store) {
    let scratch = Scratch::new(name);
    for (relative, bytes) in fixture.files {
        scratch.write(relative, bytes);
    }
    for (asset_name, bytes) in fixture.addressed {
        scratch.write(&address::stored_path(&sha256_hex(bytes), asset_name), bytes);
    }
    let store = Store::discover(&scratch.0);
    (scratch, store)
}

/// Every fixture reports exactly the number of duplicate sets it declares.
#[test]
fn every_project_fixture_reports_the_duplication_it_declares() {
    let mut failures = Vec::new();
    for (name, fixture) in FIXTURES {
        let (_scratch, store) = build(name, fixture);
        let found = topology::duplicates(&store);
        if found.len() != fixture.duplicate_sets {
            failures.push(format!(
                "{name} ({}) declares {} duplicate set(s) and reported {}:\n{}",
                fixture.note,
                fixture.duplicate_sets,
                found.len(),
                found
                    .iter()
                    .map(|d| format!("  {d}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// A report names every location, so a contributor can tell which copy to remove.
///
/// The load-bearing half of the requirement. A report naming one of three copies would
/// satisfy the count above and still leave the reader guessing.
#[test]
fn a_duplication_report_names_every_location_holding_the_bytes() {
    let (_scratch, store) = build(
        "a-copy-per-document",
        &FIXTURES
            .iter()
            .find(|(name, _)| *name == "a-copy-per-document")
            .expect("the fixture exists")
            .1,
    );
    let found = topology::duplicates(&store);
    assert_eq!(found.len(), 1, "{found:#?}");

    // One addressed copy plus three per-document copies.
    for expected in [
        "proposal/htmlspec.js",
        "design/htmlspec.js",
        "review/htmlspec.js",
    ] {
        assert!(
            found[0].message.contains(expected),
            "the report omits {expected}: {}",
            found[0].message
        );
    }
    assert!(
        found[0].message.contains("4 locations"),
        "the report does not count every copy: {}",
        found[0].message
    );
}

/// Duplication fails the project, and cannot be reduced to a warning.
#[test]
fn a_duplication_report_is_a_non_downgradable_error() {
    let (_scratch, store) = build(
        "duplicate-copies-of-one-library",
        &FIXTURES
            .iter()
            .find(|(name, _)| *name == "duplicate-copies-of-one-library")
            .expect("the fixture exists")
            .1,
    );
    let found = topology::duplicates(&store);
    assert_eq!(found.len(), 1);
    assert!(found[0].is_error(), "duplication did not fail the project");
    assert_eq!(found[0].rule.to_string(), "asset/duplicate");
    assert!(
        !found[0].rule.namespace().is_downgradable(),
        "the asset namespace accepted a downgrade"
    );
}

/// A directory with no asset store has nothing to duplicate.
#[test]
fn a_directory_with_no_store_is_not_reported() {
    let path =
        std::env::temp_dir().join(format!("htmlspec-topology-nostore-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("the directory");

    let store = Store::discover(&path);
    assert!(!store.is_present(), "no assets directory was created");
    assert!(topology::duplicates(&store).is_empty());

    let _ = std::fs::remove_dir_all(&path);
}

/// Every fixture states why it exists, and the set covers both verdicts.
///
/// A suite of only-failing fixtures would not show the rule leaves a correct project
/// alone, which is how an over-strict check ships.
#[test]
fn the_fixture_set_is_well_formed_and_covers_both_verdicts() {
    let clean = FIXTURES
        .iter()
        .filter(|(_, f)| f.duplicate_sets == 0)
        .count();
    let failing = FIXTURES
        .iter()
        .filter(|(_, f)| f.duplicate_sets > 0)
        .count();
    assert!(clean >= 2, "only {clean} conforming project fixture(s)");
    assert!(failing >= 2, "only {failing} violating project fixture(s)");

    for (name, fixture) in FIXTURES {
        assert!(
            !fixture.note.trim().is_empty(),
            "{name} records no reason for existing"
        );
        assert!(
            !fixture.files.is_empty() || !fixture.addressed.is_empty(),
            "{name} builds an empty project, so it asserts nothing"
        );
    }
}

/// This project's own topology, reported rather than asserted clean.
///
/// The migration is deliberately additive, so the old layout still sits beside the
/// addressed one and this reports duplication until group 7 removes it. `scripts/check`
/// is where that becomes a failure — asserting cleanliness here would duplicate the
/// gate and fail the whole test suite rather than one named step, which makes the
/// signal harder to read, not stronger.
///
/// The store read is the toolchain's source copy, which sat at the repository root until
/// `ship-through-four-channels` moved it into `crates/htmlspec-assets/assets/`, so a crate
/// built from crates.io holds it. That crate's directory is where it is discovered now.
#[test]
fn this_projects_topology_is_reported() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../crates/htmlspec-assets");
    let store = Store::discover(&root);
    assert!(store.is_present(), "this project has an asset store");

    let found = topology::duplicates(&store);
    println!(
        "this project reports {} duplicate set(s):\n{}",
        found.len(),
        found
            .iter()
            .map(|d| format!("  {d}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
