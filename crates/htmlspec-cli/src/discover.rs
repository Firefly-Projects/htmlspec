//! Finding the documents a command operates on.
//!
//! Two modes, both required by the specs: whole-project when given no path, and a
//! single document when given one. A directory argument means that directory rather
//! than the project, which falls out of the same walk.
//!
//! Order is sorted by path, always. A command that reported documents in filesystem
//! order would produce a different report on two machines from the same source, and
//! the deterministic-ordering requirement exists to prevent exactly that.

use std::path::{Path, PathBuf};

use crate::project::ProjectRoot;

/// Directories never walked into.
///
/// `assets` holds the vendored runtime, which is third-party JavaScript and not a
/// document. The rest are build and tool output. Skipping them by name is crude and
/// correct: a document does not live in `target/`.
const SKIPPED: &[&str] = &[".git", ".jj", ASSETS, "node_modules", "result", "target"];

/// The directory an asset store occupies, relative to the directory holding it.
///
/// Named here rather than spelled twice because [`documents`] skips it and [`stores`]
/// looks for it, and the two must agree about which directory that is.
const ASSETS: &str = htmlspec_assets::store::DIRECTORY;

/// Why documents could not be discovered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    /// The given path does not exist.
    NotFound {
        /// The path as given.
        path: PathBuf,
    },
    /// A directory could not be read.
    Unreadable {
        /// The path that could not be read.
        path: PathBuf,
        /// What the filesystem said.
        reason: String,
    },
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { path } => write!(f, "no such file or directory: {}", path.display()),
            Self::Unreadable { path, reason } => {
                write!(f, "cannot read {}: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for DiscoveryError {}

/// Finds the documents at `root`.
///
/// A file path yields that one file, whatever its extension: a contributor who names
/// a file explicitly means that file, and silently skipping it for having the wrong
/// suffix would be the tool second-guessing an explicit instruction. A directory
/// yields every `.html` file beneath it, in sorted order, except a change's evidence.
///
/// **A change's evidence is not a document of the project**, as artifact-kinds' "A change's
/// evidence is kept in the change and is not one of its documents" states. So where `root` lies
/// in a project, the walk does not descend into any directory
/// [`ProjectRoot::is_change_evidence`] identifies. Before this, `validate` reported every carried
/// record that declared an earlier contract as a failure, `format` rewrote all of them, and
/// promotion refused a migration whose only fault was its evidence.
///
/// **The exclusion is here, not in each caller**, because toolchain-packaging requires it to be
/// defined once and reached by every operation that walks. The project is resolved from `root`
/// rather than passed in: a parameter would make `None` a valid argument, and a caller passing it
/// would get the old walk with nothing to show the difference. Resolution is from the walk's own
/// root, so a walk of another project's directory excludes that project's evidence.
///
/// **The root itself is never excluded**, only what the walk comes across beneath it. Naming a
/// change's evidence directory, or a directory inside it, walks it in full: the exclusion is of
/// what a walk finds, never of what a contributor asked for. Finding a record is not permission to
/// write it, and `format`'s writing mode refuses one: see [`crate::commands::format`].
pub fn documents(root: &Path) -> Result<Vec<PathBuf>, DiscoveryError> {
    if !root.exists() {
        return Err(DiscoveryError::NotFound {
            path: root.to_owned(),
        });
    }
    if root.is_file() {
        return Ok(vec![root.to_owned()]);
    }

    let project = ProjectRoot::resolve(root).ok();
    let mut found = Vec::new();
    walk(root, project.as_ref(), &mut found)?;
    // Sorted, so two machines report the same documents in the same order.
    found.sort();
    Ok(found)
}

/// Finds every directory beneath `root` that holds an asset store.
///
/// The returned paths are the *parents* of each `assets/` directory, which is what
/// [`htmlspec_assets::Store::discover`] takes.
///
/// Unlike [`documents`], this descends into `assets` — it has to, because a store
/// nested inside another directory's store is exactly the arrangement worth reporting.
/// An unreadable directory is skipped rather than failing the run: the caller is a
/// report, and a permission error on one subtree should not hide the duplication found
/// in the rest.
pub fn stores(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if root.is_dir() {
        // Normalized first, so a report reads `<root>/htmlspec/...` rather than
        // `<root>/./htmlspec/...` when the root came in as the current directory.
        let root = root.components().collect::<PathBuf>();
        collect_stores(&root, &mut found);
    }
    // Sorted, so two machines report the same stores in the same order.
    found.sort();
    found
}

fn collect_stores(directory: &Path, found: &mut Vec<PathBuf>) {
    if directory.join(ASSETS).is_dir() {
        found.push(directory.to_owned());
    }

    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Build and tool output are skipped, but `assets` is not: a second store is
        // usually *inside* one of these directories, and skipping the name would skip
        // the finding.
        if name.starts_with('.') || matches!(name.as_ref(), "node_modules" | "result" | "target") {
            continue;
        }
        collect_stores(&path, found);
    }
}

fn walk(
    directory: &Path,
    project: Option<&ProjectRoot>,
    found: &mut Vec<PathBuf>,
) -> Result<(), DiscoveryError> {
    let entries = std::fs::read_dir(directory).map_err(|e| DiscoveryError::Unreadable {
        path: directory.to_owned(),
        reason: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| DiscoveryError::Unreadable {
            path: directory.to_owned(),
            reason: e.to_string(),
        })?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if path.is_dir() {
            // A dot-directory is tool state, not project content.
            if SKIPPED.contains(&name.as_ref()) || name.starts_with('.') {
                continue;
            }
            // Asked of each child and never of the root, which is what keeps a named path
            // walked in full.
            if project.is_some_and(|project| project.is_change_evidence(&path)) {
                continue;
            }
            walk(&path, project, found)?;
        } else if path.extension().is_some_and(|e| e == "html") {
            found.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary directory that removes itself.
    ///
    /// Hand-rolled rather than a dependency, because the need is one directory in
    /// one test module and a `tempfile` dependency for that is more surface than the
    /// eight lines it replaces.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            // The process id keeps concurrent test binaries apart; the label keeps
            // tests within one binary apart. Neither needs randomness.
            let path =
                std::env::temp_dir().join(format!("htmlspec-{label}-{}", std::process::id()));
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
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_missing_path_is_reported_rather_than_returning_nothing() {
        let error = documents(Path::new("/nonexistent/htmlspec/path")).expect_err("not found");
        assert!(matches!(error, DiscoveryError::NotFound { .. }));
        assert!(error.to_string().contains("no such file or directory"));
    }

    #[test]
    fn a_directory_yields_every_document_beneath_it_in_sorted_order() {
        let scratch = Scratch::new("discover-sorted");
        scratch.write("b.html", "");
        scratch.write("a.html", "");
        scratch.write("nested/c.html", "");
        scratch.write("notes.md", "");

        let found = documents(&scratch.0).expect("the directory is walked");
        let names: Vec<String> = found
            .iter()
            .map(|p| {
                p.strip_prefix(&scratch.0)
                    .expect("under the scratch root")
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(names, ["a.html", "b.html", "nested/c.html"]);
    }

    /// The vendored runtime and build output are not documents.
    #[test]
    fn skipped_directories_are_not_walked() {
        let scratch = Scratch::new("discover-skipped");
        scratch.write("real.html", "");
        for skipped in SKIPPED {
            scratch.write(&format!("{skipped}/decoy.html"), "");
        }
        scratch.write(".hidden/decoy.html", "");

        let found = documents(&scratch.0).expect("the directory is walked");
        assert_eq!(found.len(), 1, "{found:#?}");
        assert!(found[0].ends_with("real.html"));
    }

    /// An explicitly named file is used whatever its extension, because the
    /// contributor said so.
    #[test]
    fn an_explicit_file_is_used_whatever_its_extension() {
        let scratch = Scratch::new("discover-explicit");
        let path = scratch.write("document.htmlspec", "");
        assert_eq!(documents(&path).expect("the file is found"), vec![path]);
    }

    /// An empty project yields no documents and is not an error: there is nothing
    /// wrong with a project that has none yet.
    #[test]
    fn an_empty_directory_yields_no_documents_and_no_error() {
        let scratch = Scratch::new("discover-empty");
        assert!(documents(&scratch.0).expect("the walk succeeds").is_empty());
    }

    /// The names of `found` relative to `root`, with `/` separators.
    fn relative(found: &[PathBuf], root: &Path) -> Vec<String> {
        found
            .iter()
            .map(|p| {
                p.strip_prefix(root)
                    .expect("under the root")
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    /// A project's walk passes over every change's evidence, active and archived, at any depth,
    /// and keeps everything else — including a capability that happens to be named `evidence`.
    #[test]
    fn a_projects_walk_passes_over_a_changes_evidence() {
        let scratch = Scratch::new("discover-evidence");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        scratch.write("htmlspec/changes/a-change/proposal.html", "");
        scratch.write("htmlspec/changes/a-change/evidence/a-record.html", "");
        scratch.write(
            "htmlspec/changes/a-change/evidence/transcripts/a-session.html",
            "",
        );
        scratch.write("htmlspec/changes/archive/2026-01-01-old/design.html", "");
        scratch.write(
            "htmlspec/changes/archive/2026-01-01-old/evidence/a-record.html",
            "",
        );
        scratch.write("htmlspec/specs/evidence/spec.html", "");

        // From the project's directory, which is what a command with no path walks, and from the
        // repository root, which is what a contributor naming `.` walks.
        for root in [project.htmlspec(), scratch.0.clone()] {
            let found = documents(&root).expect("the project is walked");
            let prefix = if root == scratch.0 { "htmlspec/" } else { "" };
            assert_eq!(
                relative(&found, &root),
                [
                    format!("{prefix}changes/a-change/proposal.html"),
                    format!("{prefix}changes/archive/2026-01-01-old/design.html"),
                    format!("{prefix}specs/evidence/spec.html"),
                ],
                "walking {}",
                root.display()
            );
        }
    }

    /// Naming a change's evidence walks it in full, nested records included. The exclusion is of
    /// what a walk comes across, and the root is what the contributor asked for.
    #[test]
    fn a_root_inside_a_changes_evidence_is_walked_in_full() {
        let scratch = Scratch::new("discover-evidence-root");
        ProjectRoot::establish(&scratch.0).expect("established");
        let evidence = scratch.0.join("htmlspec/changes/a-change/evidence");
        scratch.write("htmlspec/changes/a-change/evidence/a-record.html", "");
        scratch.write(
            "htmlspec/changes/a-change/evidence/transcripts/a-session.html",
            "",
        );

        let found = documents(&evidence).expect("the evidence is walked");
        assert_eq!(
            relative(&found, &evidence),
            ["a-record.html", "transcripts/a-session.html"]
        );

        let transcripts = evidence.join("transcripts");
        let found = documents(&transcripts).expect("the transcripts are walked");
        assert_eq!(relative(&found, &transcripts), ["a-session.html"]);
    }

    /// Outside any project there are no changes, so a directory called `evidence` is walked like
    /// any other. The conformance fixtures keep the behavior they have.
    #[test]
    fn outside_a_project_an_evidence_directory_is_walked() {
        let scratch = Scratch::new("discover-evidence-outside");
        scratch.write("a-change/evidence/a-record.html", "");

        let found = documents(&scratch.0).expect("the directory is walked");
        assert_eq!(
            relative(&found, &scratch.0),
            ["a-change/evidence/a-record.html"]
        );
    }
}
