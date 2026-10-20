//! The staged migration, and the move that puts it in place.
//!
//! A migration assembles a whole project somewhere else and hands it over in one step. This
//! module holds the checks that make that safe and the move itself.
//!
//! ## Why the staging location is outside the destination
//!
//! A project holds exactly one asset store, and `htmlspec assets` exists to report a second one.
//! A staging tree beneath the destination puts a second store inside it, so the check that would
//! report the duplication fires on the migration's own working directory — and a contributor
//! reading that report cannot tell it from a real defect.
//!
//! ## Why the staged tree is a complete project rather than a bag of files
//!
//! Every conformance check resolves a project first. [`crate::project::ProjectRoot::resolve`]
//! ascends looking for a child `htmlspec/` directory, and a staging area it cannot resolve cannot
//! be validated — which would push every check past promotion and make promotion the step that
//! discovers faults. So staging holds a declaration, the three places a project keeps documents,
//! and one asset store, and [`Staged::resolve`] is the proof.
//!
//! ## Why the paths mirror the destination exactly
//!
//! Each document references the project's asset store by a path relative to its own directory:
//! `htmlspec/specs/x.html` reaches `../assets/` and `htmlspec/changes/y/proposal.html` reaches
//! `../../assets/`. Stage a document at a different depth and promotion has to rewrite every
//! reference it moves — a second transformation applied *after* the checks that would have caught
//! a mistake in the rewriting. So the staged tree holds the relative paths the destination will,
//! and promotion is a move.
//!
//! ## Why the move is whole-or-none, and how
//!
//! `project-migration` requires the destination to hold either the whole migrated project or none
//! of it. Two things make that true rather than hoped for. Promotion refuses while the destination
//! already holds the project directory or the declaration, so neither is ever overwritten. And a
//! failure part-way through moves back what was already moved, so a full disk on the third entry
//! leaves the destination as it was.
//!
//! The release record is the one exception, and it is not a weakening. A migration promotes into
//! the repository the source project already lives in, and such a repository normally has a root
//! `CHANGELOG.md` already — so refusing on it refused the ordinary case. The requirement is that
//! the record be *carried*, which a destination already holding it satisfies, and the destination's
//! own copy is left alone rather than replaced with the source's.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::project::{ProjectRoot, DECLARATION, DIRECTORY};

/// The project's release record, which a migration carries.
///
/// `toolchain-packaging` requires recorded versions to be distinct, with a stated exception: where
/// several changes were archived under one version before the obligation existed, the changelog
/// carries one baseline entry naming them. So the changelog is not decoration a migration can drop
/// — a migrated project without it fails its own release check on arrival, reporting a version
/// collision it cannot explain.
pub const RELEASE_RECORD: &str = "CHANGELOG.md";

/// What a staged project places in the destination, in the order promotion moves them.
///
/// The project directory first, because it is one rename and it is the entry whose absence makes
/// the destination unmistakably un-migrated. Generated agent instructions are not here: the
/// destination's own bootstrap operation writes those from its compiled workflow set, and
/// converted ones would name commands the destination program does not provide.
pub const PLACED: &[&str] = &[DIRECTORY, DECLARATION, RELEASE_RECORD];

/// What promotion refuses to place over, out of [`PLACED`].
///
/// **Not [`RELEASE_RECORD`], and that omission is the whole point.** The ordinary migration
/// promotes into the repository the source project already lives in, and such a repository almost
/// always has a root `CHANGELOG.md` already — this one does. Refusing on it made promotion refuse
/// the ordinary case with no way past, while the requirement that the release record be *carried*
/// is satisfied by the destination already holding it. Two rules of this change deadlocked, and a
/// review of the pull request found it.
///
/// The project directory and the declaration stay here. A destination already holding either is
/// already an HTMLSpec project, and promoting a second one over it is a merge rather than the
/// single placement the contract requires.
const REFUSED_OVER: &[&str] = &[DIRECTORY, DECLARATION];

/// Why a staged tree is not ready to promote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fault {
    /// The staging location is inside the destination project.
    InsideTheDestination {
        /// The staging location.
        staged: PathBuf,
        /// The destination it lies within.
        destination: PathBuf,
    },
    /// The staged tree does not resolve as a project.
    NotAProject {
        /// The staging location.
        staged: PathBuf,
    },
    /// The staged tree is missing a place a project keeps documents.
    Incomplete {
        /// Every absent path, relative to the staging location.
        absent: Vec<String>,
    },
    /// The staged tree holds more than one asset store.
    SeveralStores {
        /// Where each store is, relative to the staging location.
        stores: Vec<String>,
    },
    /// The destination already holds something promotion would place.
    DestinationOccupied {
        /// Every occupied entry, relative to the destination.
        occupied: Vec<String>,
    },
    /// An archived change's directory name encodes a version.
    VersionInAnArchiveName {
        /// Every offending directory name.
        names: Vec<String>,
    },
    /// The declaration's version is not the greatest version the archives record.
    DeclarationDisagrees {
        /// What the declaration says.
        declared: String,
        /// The greatest version the migrated archives record.
        greatest: String,
    },
    /// Several archived changes share a version and the release record does not explain it.
    UnexplainedSharedVersion {
        /// The shared version.
        version: String,
        /// Every change recording it, sorted.
        changes: Vec<String>,
    },
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsideTheDestination {
                staged,
                destination,
            } => write!(
                f,
                "the staging location {} is inside the destination project {}. A project holds \
                 exactly one asset store, so a staging tree within the destination puts a second \
                 one there — which is the condition `htmlspec assets` exists to report, firing on \
                 the migration's own working directory",
                staged.display(),
                destination.display()
            ),
            Self::NotAProject { staged } => write!(
                f,
                "{} does not resolve as a project, so the conformance checks cannot run against \
                 it. A staging area that is not a resolvable project pushes every check past \
                 promotion and makes promotion the step that discovers faults",
                staged.display()
            ),
            Self::Incomplete { absent } => write!(
                f,
                "the staged project is missing {}. It must be a complete project so the \
                 project's own checks run against it exactly as they will after promotion",
                absent.join(", ")
            ),
            Self::SeveralStores { stores } => write!(
                f,
                "the staged project holds {} asset stores ({}). A project holds exactly one, and \
                 promoting several would move the defect `htmlspec assets` reports into the \
                 destination",
                stores.len(),
                stores.join(", ")
            ),
            Self::DestinationOccupied { occupied } => write!(
                f,
                "the destination already holds {}. Promotion places the whole migrated project or \
                 none of it, so it does not overwrite and does not merge",
                occupied.join(", ")
            ),
            Self::VersionInAnArchiveName { names } => {
                write!(
                f,
                "{} archived {} a version in {} directory name ({}). An archived change is named \
                 for its date followed by the change's name, and the archival gate reads the \
                 version each change records rather than deriving one from a name — so a version \
                 in the name is both a naming violation and a version the gate never reads",
                names.len(),
                if names.len() == 1 { "change encodes" } else { "changes encode" },
                if names.len() == 1 { "its" } else { "their" },
                names.join(", ")
            )
            }
            Self::DeclarationDisagrees { declared, greatest } => write!(
                f,
                "the staged project declares version {declared} and its archives record {greatest} \
                 as the greatest. The declared version must equal the greatest migrated version, \
                 so that the next archival is required to advance past the project's real history \
                 rather than past a version it has already passed"
            ),
            Self::UnexplainedSharedVersion { version, changes } => write!(
                f,
                "{} archived changes record version {version} ({}) and the release record has no \
                 entry naming it. Recorded versions must be distinct, with one exception: a \
                 baseline entry naming the changes that shipped together before that obligation \
                 existed. Carry the source's entry rather than inventing distinct versions, which \
                 would record releases that never existed — a migrated project without it fails \
                 its own release check on arrival, reporting a collision it cannot explain",
                changes.len(),
                changes.join(", ")
            ),
        }
    }
}

impl std::error::Error for Fault {}

/// A staged project, known to be complete and outside its destination.
///
/// Construction is the proof, as with [`ProjectRoot`]: a value of this type exists only where
/// every structural check passed, so the promotion below does not re-check and cannot be handed a
/// directory that is not ready.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Staged {
    /// The staging location, which is the migrated project's root.
    root: PathBuf,
    /// The destination the project will be promoted into.
    destination: PathBuf,
}

impl Staged {
    /// Checks the staged tree at `staged` against the destination at `destination`.
    ///
    /// Every fault, together. A staged tree missing two directories and holding two stores reports
    /// all of it in one pass, because repairing a staging fault one report at a time is the same
    /// waste the check-last rule exists to prevent one stage later.
    pub fn resolve(staged: &Path, destination: &Path) -> Result<Self, Vec<Fault>> {
        let staged_root = absolute(staged);
        let destination_root = absolute(destination);
        let mut faults = Vec::new();

        // First, because every other check reads the staged tree and this one says the tree is in
        // the wrong place entirely. A staging directory inside the destination is a mistake about
        // the operation rather than about the content.
        if staged_root.starts_with(&destination_root) {
            faults.push(Fault::InsideTheDestination {
                staged: staged_root.clone(),
                destination: destination_root.clone(),
            });
        }

        let project = match ProjectRoot::resolve(&staged_root) {
            // Resolution ascends, so a staging directory *inside* a project resolves to the
            // enclosing one. Only a root that resolves to itself is the staged project.
            Ok(project) if project.path() == staged_root => Some(project),
            _ => {
                faults.push(Fault::NotAProject {
                    staged: staged_root.clone(),
                });
                None
            }
        };

        if let Some(project) = &project {
            let mut absent = Vec::new();
            for (path, label) in [
                (project.declaration(), DECLARATION),
                (project.specs(), "htmlspec/specs"),
                (project.changes(), "htmlspec/changes"),
                (project.archive(), "htmlspec/changes/archive"),
                (project.assets(), "htmlspec/assets"),
            ] {
                if !path.exists() {
                    absent.push(String::from(label));
                }
            }
            if !absent.is_empty() {
                faults.push(Fault::Incomplete { absent });
            }

            let stores = crate::discover::stores(&staged_root);
            if stores.len() > 1 {
                faults.push(Fault::SeveralStores {
                    stores: stores
                        .iter()
                        .map(|store| relative(&staged_root, store))
                        .collect(),
                });
            }

            faults.extend(version_faults(project, &destination_root));
        }

        let occupied: Vec<String> = REFUSED_OVER
            .iter()
            .filter(|entry| destination_root.join(entry).exists())
            .map(|entry| (*entry).to_owned())
            .collect();
        if !occupied.is_empty() {
            faults.push(Fault::DestinationOccupied { occupied });
        }

        if faults.is_empty() {
            Ok(Self {
                root: staged_root,
                destination: destination_root,
            })
        } else {
            Err(faults)
        }
    }

    /// The staging location.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The destination the project promotes into.
    pub fn destination(&self) -> &Path {
        &self.destination
    }

    /// Moves the staged project into the destination.
    ///
    /// **Whole or none.** [`Self::resolve`] already refused while the destination held the project
    /// directory or the declaration, so neither is overwritten. What remains is a failure part-way
    /// through, and this moves back what it already moved rather than leaving a half-migrated
    /// destination behind — which is the state a contributor cannot recover from by reading a
    /// report.
    ///
    /// The release record is the one entry the destination may already hold, and this leaves that
    /// copy alone. The requirement is that the record be *carried*, and a destination already
    /// holding it satisfies that. Overwriting it would replace a repository's own history with a
    /// copy of the source's, which is a destructive edit nobody asked for by running a migration.
    pub fn promote(&self) -> std::io::Result<Vec<String>> {
        let mut moved: Vec<String> = Vec::new();

        for entry in PLACED {
            let from = self.root.join(entry);
            if !from.exists() {
                // Not every entry is owed. A source project with no release record stages none,
                // and the declaration and project directory are already required by `resolve`.
                continue;
            }
            let to = self.destination.join(entry);
            if to.exists() {
                // Reachable only for the release record, since `resolve` refused on the other two.
                // The destination's own copy wins.
                continue;
            }
            if let Err(error) = rename_or_copy(&from, &to) {
                // Back out, so the destination is as it was. The unwind's own errors are
                // deliberately not reported over the one that caused it: the first error is the
                // fault a contributor acts on, and a rollback failure would bury it.
                for placed in moved.iter().rev() {
                    let _ = rename_or_copy(&self.destination.join(placed), &self.root.join(placed));
                }
                return Err(error);
            }
            moved.push((*entry).to_owned());
        }

        Ok(moved)
    }
}

/// The version faults a staged project carries.
///
/// Three, and the first two are one requirement read from opposite ends. `project-bootstrap` —
/// "Archival requires the project version to advance" — states that the last archived version is
/// read from what each change records and **not** derived from a directory's name, and
/// `toolchain-packaging` — "An archived change is named for its date and refuses to collide" —
/// states that the name is the date followed by the change's name. So a version in an archive
/// directory name violates the naming rule *and* records the version where the gate does not read
/// it. A migration composing `<date>-<version>-<name>` would produce a project passing every other
/// check whose next archival passes without advancing.
///
/// The third is the collision a migrated project inherits. `toolchain-packaging` — "An archived
/// change records the version it shipped in" — requires recorded versions to be distinct, with one
/// exception: a baseline changelog entry naming the changes that shipped together before that
/// obligation existed. **Seven of this repository's sixteen archived changes record `0.1.0`**, so a
/// migration of it inherits that collision, and a migrated project whose release record lost the
/// entry fails its own release check on arrival — reporting a collision it cannot explain.
///
/// **That third check reads the record that will survive promotion, which is not always the staged
/// one.** Promotion leaves a destination's existing `CHANGELOG.md` alone, so a check against the
/// staged copy passed while the surviving copy explained nothing — the migrated project then failed
/// its own release check on arrival, which is exactly the outcome the fault exists to prevent. A
/// review of the pull request found it. Both records are read, and an entry in either satisfies the
/// check: the staged one is what arrives where the destination has none, and the destination's is
/// what survives where it has one.
fn version_faults(project: &ProjectRoot, destination: &Path) -> Vec<Fault> {
    let mut faults = Vec::new();

    let mut named = Vec::new();
    let mut greatest: Option<crate::version::Version> = None;
    let mut by_version: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(project.archive()) {
        for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if encodes_a_version(&name) {
                named.push(name.clone());
            }
            let recorded =
                std::fs::read_to_string(entry.path().join(crate::project::RECORDED_VERSION))
                    .ok()
                    .and_then(|text| crate::version::Version::parse(text.trim()).ok());
            if let Some(version) = recorded {
                by_version
                    .entry(version.to_string())
                    .or_default()
                    .push(name);
                greatest = Some(match greatest {
                    Some(held) if held >= version => held,
                    _ => version,
                });
            }
        }
    }
    if !named.is_empty() {
        named.sort();
        faults.push(Fault::VersionInAnArchiveName { names: named });
    }

    // A shared version is carried honestly rather than resolved, so what is checked is that the
    // release record explains it. Presence of an entry, not the prose: the accepted requirement
    // says the check "reads the entry's presence and applies no rule to the entry's prose", and a
    // migration is not the place to start judging changelog wording.
    // The record that survives, whichever it is. Where the destination holds one, promotion keeps
    // it and discards the staged copy, so an entry in the staged copy alone explains nothing.
    let surviving = destination.join(RELEASE_RECORD);
    let mut entries = release_record_versions(&project.path().join(RELEASE_RECORD));
    if surviving.is_file() {
        entries = release_record_versions(&surviving);
    }
    for (version, mut changes) in by_version {
        if changes.len() < 2 || entries.contains(&version) {
            continue;
        }
        changes.sort();
        faults.push(Fault::UnexplainedSharedVersion { version, changes });
    }

    // Compared only where the archives record something. A migrated project with no archived
    // change has no history to declare, and requiring the declaration to equal an absent value
    // would refuse a young source project for being young.
    if let (Some(greatest), Ok(declaration)) =
        (greatest, crate::declaration::read(&project.declaration()))
    {
        if declaration.version != greatest {
            faults.push(Fault::DeclarationDisagrees {
                declared: declaration.version.to_string(),
                greatest: greatest.to_string(),
            });
        }
    }

    faults
}

/// Every version a release record's headings name.
///
/// **Headings only, and parsed rather than matched as text.** A substring test on the whole file
/// was the first implementation and it was wrong twice over: `0.1.1` is a substring of an
/// unrelated `0.1.10` entry, and any prose containing those digits satisfied it — so a real
/// unexplained collision passed the gate silently. A review of the pull request found it.
///
/// This is the rule `scripts/check-release` already applies to the same file, which is what keeps
/// the two from disagreeing about whether a version has an entry.
fn release_record_versions(path: &Path) -> BTreeSet<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    text.lines()
        .filter(|line| line.starts_with('#'))
        .flat_map(dotted_numbers)
        .collect()
}

/// The dotted numbers in one line, as whole tokens.
///
/// Split on anything that is not a digit or a dot, so `## 0.11.0 — arriving from OpenSpec` yields
/// `0.11.0` and nothing else. A token is kept only where it holds a dot and parses as a version,
/// which is what stops a bare year or a section number from reading as one.
fn dotted_numbers(line: &str) -> Vec<String> {
    line.split(|byte: char| !byte.is_ascii_digit() && byte != '.')
        .filter(|token| token.contains('.'))
        .filter(|token| crate::version::Version::parse(token).is_ok())
        .map(str::to_owned)
        .collect()
}

/// Whether an archive directory name encodes a version after its date.
///
/// A name is `YYYY-MM-DD-<name>`, so this asks whether the component right after the date is a
/// dotted number. `2026-08-10-0.1.0-html-spec` matches and `2026-08-10-html-spec` does not.
fn encodes_a_version(name: &str) -> bool {
    if !crate::commands::leads_with_a_date(name) {
        return false;
    }
    name["YYYY-MM-DD-".len()..]
        .split('-')
        .next()
        .is_some_and(|first| {
            !first.is_empty()
                && first.split('.').count() > 1
                && first
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        })
}

/// Moves `from` to `to`, falling back to a copy across filesystems.
///
/// `rename` is one operation and cannot leave a partly-moved directory, so it is what makes the
/// promotion of the project directory whole-or-none on its own. It fails with `CrossesDevices`
/// when the staging location is on another filesystem — a temporary directory very often is — so
/// the fallback copies and then removes.
fn rename_or_copy(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match std::fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(error) if crosses_devices(&error) => copy_then_remove(from, to),
        Err(error) => Err(error),
    }
}

/// The cross-device fallback: copy the entry, then remove the original.
///
/// **A failure here cleans up after itself, and it has to.** `copy_tree` writes into the destination
/// before anything is removed from staging, so an error part-way leaves the destination holding a
/// partial entry. The caller's rollback cannot help: [`Staged::promote`] unwinds the entries it
/// recorded as moved, and this entry is not one of them yet — the record is written after this
/// returns. So a full disk on the copy left a destination holding half a project while promotion
/// reported it unchanged and said a retry was safe. A review of the pull request found it.
///
/// Named as its own function rather than inlined, because that is what makes the recovery testable:
/// `EXDEV` needs two real filesystems to provoke, so a test driving `rename_or_copy` cannot reach
/// this branch at all.
fn copy_then_remove(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Err(error) = copy_tree(from, to) {
        // Best effort, and the copy's own error is what propagates. A cleanup failure reported over
        // the cause would send a contributor to the wrong problem.
        let _ = remove(to);
        return Err(error);
    }
    if let Err(error) = remove(from) {
        // The copy succeeded and the staged original could not be removed. Removing the
        // destination's new copy keeps the two trees from both holding the entry, which is the state
        // that makes a retry trip the occupied-destination refusal.
        let _ = remove(to);
        return Err(error);
    }
    Ok(())
}

/// Removes `path`, whether it is a file or a directory.
fn remove(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else if path.exists() {
        std::fs::remove_file(path)
    } else {
        Ok(())
    }
}

/// Whether a rename failed because the two paths are on different filesystems.
///
/// Matched on the raw code rather than on `ErrorKind::CrossesDevices`, which is unstable. `EXDEV`
/// is 18 on both Linux and macOS.
fn crosses_devices(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(18)
}

/// Copies `from` to `to`, recursing through directories.
fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(from, to).map(|_| ())
    }
}

/// `path` relative to `root`, as a display string.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Absolutizes and normalizes a path, dropping `.` segments.
fn absolute(path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_owned()
    } else {
        match std::env::current_dir() {
            Ok(current) => current.join(path),
            Err(_) => path.to_owned(),
        }
    };
    joined.components().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary directory that removes itself.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-staging-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            let path = path.canonicalize().expect("the scratch path resolves");
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

        fn directory(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            std::fs::create_dir_all(&path).expect("the directory is created");
            path
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A complete staged project at `at`, whose archive records `versions`.
    fn stage(scratch: &Scratch, at: &str, versions: &[(&str, &str)]) -> PathBuf {
        let root = scratch.directory(at);
        for directory in [
            "htmlspec/specs",
            "htmlspec/changes/archive",
            "htmlspec/assets",
        ] {
            std::fs::create_dir_all(root.join(directory)).expect("the directory is created");
        }
        let greatest = versions
            .iter()
            .filter_map(|(_, version)| crate::version::Version::parse(version).ok())
            .max()
            .map(|version| version.to_string())
            .unwrap_or_else(|| String::from("0.0.0"));
        std::fs::write(
            root.join(DECLARATION),
            format!(
                "{{\n  name = \"staged\";\n  version = \"{greatest}\";\n  htmlspec = null;\n}}\n"
            ),
        )
        .expect("the declaration is written");
        for (name, version) in versions {
            let change = root.join("htmlspec/changes/archive").join(name);
            std::fs::create_dir_all(&change).expect("the archived change is created");
            std::fs::write(change.join("VERSION"), format!("{version}\n"))
                .expect("the version is recorded");
        }
        root
    }

    /// A complete staged tree outside its destination resolves.
    #[test]
    fn a_complete_staged_project_outside_the_destination_resolves() {
        let scratch = Scratch::new("complete");
        let staged = stage(&scratch, "staging", &[("2026-08-10-html-spec", "0.1.0")]);
        let destination = scratch.directory("repository");

        let resolved = Staged::resolve(&staged, &destination).expect("the staged tree resolves");
        assert_eq!(resolved.root(), staged);
        assert_eq!(resolved.destination(), destination);
    }

    /// The staged tree resolves as a project, which is what every conformance check needs.
    #[test]
    fn the_staged_tree_resolves_as_a_project() {
        let scratch = Scratch::new("resolves");
        let staged = stage(&scratch, "staging", &[]);
        assert_eq!(
            ProjectRoot::resolve(&staged)
                .expect("the staged tree is a project")
                .path(),
            staged
        );
    }

    /// A staging location inside the destination is refused, because it would put a second asset
    /// store in a project that holds exactly one.
    #[test]
    fn a_staging_location_inside_the_destination_is_refused() {
        let scratch = Scratch::new("inside");
        let destination = scratch.directory("repository");
        let staged = stage(&scratch, "repository/.htmlspec-migration", &[]);

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        assert!(
            faults
                .iter()
                .any(|fault| matches!(fault, Fault::InsideTheDestination { .. })),
            "{faults:?}"
        );
        // And the report says why rather than only that.
        let rendered = faults[0].to_string();
        assert!(rendered.contains("one asset store"), "{rendered}");
    }

    /// A staged tree missing the places a project keeps documents is refused, with **every**
    /// absence named.
    #[test]
    fn an_incomplete_staged_tree_names_every_absence() {
        let scratch = Scratch::new("incomplete");
        let staged = scratch.directory("staging");
        std::fs::create_dir_all(staged.join("htmlspec")).expect("the project directory");
        let destination = scratch.directory("repository");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        let Some(Fault::Incomplete { absent }) = faults
            .iter()
            .find(|fault| matches!(fault, Fault::Incomplete { .. }))
        else {
            panic!("no incompleteness was reported: {faults:?}");
        };
        for wanted in [
            "spec.nix",
            "htmlspec/specs",
            "htmlspec/changes",
            "htmlspec/changes/archive",
            "htmlspec/assets",
        ] {
            assert!(
                absent.iter().any(|entry| entry == wanted),
                "{wanted} is absent and was not reported: {absent:?}"
            );
        }
    }

    /// A staged tree holding two asset stores is refused, because promoting it would move the
    /// defect `htmlspec assets` reports into the destination.
    #[test]
    fn a_staged_tree_with_two_asset_stores_is_refused() {
        let scratch = Scratch::new("two-stores");
        let staged = stage(&scratch, "staging", &[]);
        scratch.write(
            "staging/htmlspec/assets/sha256-aaaaaaaaaaaaaaaa/x.css",
            "a\n",
        );
        scratch.write(
            "staging/htmlspec/changes/one/assets/sha256-bbbbbbbbbbbbbbbb/x.css",
            "b\n",
        );
        let destination = scratch.directory("repository");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        assert!(
            faults
                .iter()
                .any(|fault| matches!(fault, Fault::SeveralStores { .. })),
            "{faults:?}"
        );
    }

    /// A version in an archive directory name is refused. Two accepted requirements forbid it: the
    /// gate reads a recorded version rather than deriving one from a name, and an archived
    /// directory is named for its date followed by the change's name.
    #[test]
    fn a_version_in_an_archive_directory_name_is_refused() {
        let scratch = Scratch::new("named-version");
        let staged = stage(
            &scratch,
            "staging",
            &[("2026-08-10-0.1.0-html-spec", "0.1.0")],
        );
        let destination = scratch.directory("repository");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        let Some(Fault::VersionInAnArchiveName { names }) = faults
            .iter()
            .find(|fault| matches!(fault, Fault::VersionInAnArchiveName { .. }))
        else {
            panic!("the composed name was accepted: {faults:?}");
        };
        assert_eq!(names, &["2026-08-10-0.1.0-html-spec"]);
        // The report names both halves of the fault, because a contributor who fixes only the
        // naming still has a version the gate does not read.
        let rendered = faults
            .iter()
            .find(|fault| matches!(fault, Fault::VersionInAnArchiveName { .. }))
            .expect("present")
            .to_string();
        assert!(rendered.contains("date followed by"), "{rendered}");
        assert!(rendered.contains("records"), "{rendered}");
    }

    /// `<date>-<name>` is accepted, including a name whose own first component holds a dot that is
    /// not a version.
    #[test]
    fn a_date_and_name_archive_is_accepted() {
        assert!(!encodes_a_version("2026-08-10-html-spec"));
        assert!(!encodes_a_version("2026-09-06-distinguish-a-requirement"));
        assert!(encodes_a_version("2026-08-10-0.1.0-html-spec"));
        assert!(encodes_a_version("2026-08-10-0.10.0-x"));
        // A name that is not dated at all is not this check's business — the archival naming rule
        // reports that, and reporting it twice in two vocabularies helps nobody.
        assert!(!encodes_a_version("html-spec"));
        // A single number is a name, not a version: `2026-08-10-2-more-fixes` is a change name.
        assert!(!encodes_a_version("2026-08-10-2-more-fixes"));
    }

    /// The declaration must equal the greatest version the archives record, so the next archival
    /// is required to advance past the project's real history.
    #[test]
    fn a_declaration_below_the_greatest_archived_version_is_refused() {
        let scratch = Scratch::new("declaration-behind");
        let staged = stage(
            &scratch,
            "staging",
            &[("2026-08-10-a", "0.9.0"), ("2026-08-11-b", "0.10.0")],
        );
        // `stage` wrote the greatest, which is `0.10.0` compared numerically. Lower it, which is
        // what a migration comparing version *text* would have produced.
        std::fs::write(
            staged.join(DECLARATION),
            "{\n  name = \"staged\";\n  version = \"0.9.0\";\n  htmlspec = null;\n}\n",
        )
        .expect("the declaration is rewritten");
        let destination = scratch.directory("repository");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        let Some(Fault::DeclarationDisagrees { declared, greatest }) = faults
            .iter()
            .find(|fault| matches!(fault, Fault::DeclarationDisagrees { .. }))
        else {
            panic!("the disagreement was accepted: {faults:?}");
        };
        assert_eq!(declared, "0.9.0");
        assert_eq!(greatest, "0.10.0", "the greatest was compared as text");
    }

    /// A shared version with no release-record entry is refused, with every change named.
    ///
    /// This repository's own case, and it is not an edge one: **seven of its sixteen archived
    /// changes record `0.1.0`**, because they were archived before the distinctness obligation
    /// existed. `toolchain-packaging` — "An archived change records the version it shipped in" —
    /// provides for exactly that with a baseline changelog entry naming them, and
    /// `scripts/check-release` holds `BASELINE = (0, 1, 0)` for it.
    ///
    /// So a migration of this repository inherits the collision. A migrated project whose release
    /// record lost the entry fails its own release check the moment it arrives, with a diagnostic
    /// about version distinctness that names seven changes and explains none of them — which is
    /// why this is caught before promotion rather than after.
    #[test]
    fn a_shared_version_with_no_release_record_entry_is_refused() {
        let scratch = Scratch::new("shared-unexplained");
        let staged = stage(
            &scratch,
            "staging",
            &[
                ("2026-08-10-html-spec", "0.1.0"),
                ("2026-08-13-content-address-runtime-assets", "0.1.0"),
                ("2026-08-14-separate-artifact-kinds", "0.1.0"),
                ("2026-08-20-later", "0.2.0"),
            ],
        );
        // A release record naming only the later version, which is the state a migration that
        // carried the changelog's recent entries and dropped its baseline would produce.
        std::fs::write(
            staged.join(RELEASE_RECORD),
            "# Changelog\n\n## 0.2.0\n\nThe later release.\n",
        )
        .expect("the release record is written");
        let destination = scratch.directory("repository");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        let Some(Fault::UnexplainedSharedVersion { version, changes }) = faults
            .iter()
            .find(|fault| matches!(fault, Fault::UnexplainedSharedVersion { .. }))
        else {
            panic!("the unexplained collision was accepted: {faults:?}");
        };
        assert_eq!(version, "0.1.0");
        // Every change, because the repair is one changelog entry naming all of them and a
        // contributor told only the first has to find the rest.
        assert_eq!(changes.len(), 3, "{changes:?}");
        assert!(changes.contains(&String::from("2026-08-10-html-spec")));

        // And the report says the repair rather than only the fault: carry the source's entry,
        // never invent distinct versions.
        let rendered = faults
            .iter()
            .find(|fault| matches!(fault, Fault::UnexplainedSharedVersion { .. }))
            .expect("present")
            .to_string();
        assert!(rendered.contains("never existed"), "{rendered}");
    }

    /// A destination that already holds a release record is promoted into, and its copy survives.
    ///
    /// **The ordinary case, and promotion refused it.** A migration promotes into the repository
    /// the source project already lives in, and such a repository normally has a root
    /// `CHANGELOG.md` — this one does. Refusing on it made stage 7 report "the destination already
    /// holds CHANGELOG.md" for a correctly staged migration, with nothing in the workflow saying
    /// how to get past it. A review of the pull request found that, and this pins the repair.
    ///
    /// The destination's own copy wins. The requirement is that the release record be *carried*,
    /// which a destination already holding it satisfies, and replacing a repository's history with
    /// the source's is a destructive edit nobody asked for by running a migration.
    #[test]
    fn a_destination_holding_a_release_record_is_promoted_into() {
        let scratch = Scratch::new("occupied-changelog");
        let staged = stage(&scratch, "staging", &[("2026-08-10-html-spec", "0.1.0")]);
        std::fs::write(
            staged.join(RELEASE_RECORD),
            "# Changelog\n\n## 0.1.0\n\nThe source's own record.\n",
        )
        .expect("the staged record is written");

        let destination = scratch.directory("repository");
        let theirs = "# Changelog\n\n## 0.1.0\n\nThe destination's record, which predates this.\n";
        std::fs::write(destination.join(RELEASE_RECORD), theirs).expect("their record");

        let resolved = Staged::resolve(&staged, &destination).expect("the ordinary case resolves");
        let moved = resolved.promote().expect("promoted");

        // The record is not moved, because the destination already holds one.
        assert_eq!(moved, vec!["htmlspec", "spec.nix"]);
        assert_eq!(
            std::fs::read_to_string(destination.join(RELEASE_RECORD)).expect("readable"),
            theirs,
            "the destination's own release record was overwritten"
        );
        // And the project arrived, which is what the refusal used to prevent.
        assert!(destination.join(DIRECTORY).is_dir());
        assert!(destination.join(DECLARATION).is_file());
    }

    /// A destination already holding the project directory or the declaration is still refused.
    ///
    /// The other side of the repair above. Relaxing the release record must not relax these: a
    /// destination holding either is already an HTMLSpec project, and placing a second one over it
    /// is a merge rather than the single placement the contract requires.
    #[test]
    fn a_destination_holding_the_project_or_the_declaration_is_still_refused() {
        for occupant in [DIRECTORY, DECLARATION] {
            let scratch = Scratch::new(&format!("occupied-{occupant}"));
            let staged = stage(&scratch, "staging", &[]);
            let destination = scratch.directory("repository");
            if occupant == DIRECTORY {
                std::fs::create_dir_all(destination.join(DIRECTORY)).expect("their project");
            } else {
                std::fs::write(destination.join(DECLARATION), "{}\n").expect("their declaration");
            }

            let faults = Staged::resolve(&staged, &destination).expect_err("refused");
            let Some(Fault::DestinationOccupied { occupied }) = faults
                .iter()
                .find(|fault| matches!(fault, Fault::DestinationOccupied { .. }))
            else {
                panic!("{occupant} in the destination was accepted: {faults:?}");
            };
            assert_eq!(occupied, &[occupant.to_owned()]);
        }
    }

    /// A version an unrelated entry merely contains does not satisfy the shared-version check.
    ///
    /// **The first implementation was a substring test on the whole changelog**, and it was wrong
    /// twice over: `0.1.1` is a substring of `0.1.10`, and any prose holding those digits satisfied
    /// it — so a real unexplained collision passed the gate silently. A review of the pull request
    /// found it, and this is the case that proves the repair.
    #[test]
    fn a_version_another_entry_merely_contains_does_not_explain_it() {
        let scratch = Scratch::new("substring");
        let staged = stage(
            &scratch,
            "staging",
            &[("2026-08-10-a", "0.1.1"), ("2026-08-13-b", "0.1.1")],
        );
        // `0.1.10` holds `0.1.1` as a substring and is a different release. The prose names the
        // digits too, which the old test also accepted.
        std::fs::write(
            staged.join(RELEASE_RECORD),
            "# Changelog\n\n## 0.1.10\n\nUnrelated. Supersedes the 0.1.1 line entirely.\n",
        )
        .expect("the record is written");
        // The declaration must agree with the archives, so the shared-version fault is the only
        // one this asserts on.
        std::fs::write(
            staged.join(DECLARATION),
            "{\n  name = \"staged\";\n  version = \"0.1.1\";\n  htmlspec = null;\n}\n",
        )
        .expect("the declaration is written");
        let destination = scratch.directory("repository");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        assert!(
            faults
                .iter()
                .any(|fault| matches!(fault, Fault::UnexplainedSharedVersion { .. })),
            "a substring satisfied the release record: {faults:?}"
        );
    }

    /// Versions are read from the record's headings, parsed rather than matched as text.
    ///
    /// The rule `scripts/check-release` already applies to the same file, which is what keeps the
    /// two from disagreeing about whether a version has an entry.
    #[test]
    fn release_record_versions_come_from_headings() {
        let scratch = Scratch::new("record-headings");
        let path = scratch.write(
            RELEASE_RECORD,
            "# Changelog\n\n## 0.11.0 — arriving from OpenSpec\n\nProse naming 0.9.9 in passing.\n\
             \n### 1.2.3 a nested heading\n\n## not-a-version\n\n## 2026 alone\n",
        );
        let found = release_record_versions(&path);
        // The prose version is absent, and a bare year is not a version.
        assert!(found.contains("0.11.0"), "{found:?}");
        assert!(found.contains("1.2.3"), "{found:?}");
        assert!(
            !found.contains("0.9.9"),
            "prose was read as an entry: {found:?}"
        );
        assert!(
            !found.contains("2026"),
            "a bare year read as a version: {found:?}"
        );
        assert_eq!(found.len(), 2, "{found:?}");
    }

    /// A shared version explained only by the staged record, which promotion discards, is refused.
    ///
    /// **The check has to read the record that survives.** Promotion leaves a destination's existing
    /// `CHANGELOG.md` alone, so a baseline entry present only in the staged copy explains nothing
    /// after the move. The check read the staged copy, passed, and the promoted project then failed
    /// its own release check on arrival — reporting a collision it cannot explain, which is verbatim
    /// the outcome this fault exists to prevent. A review of the pull request found it, and I
    /// reproduced it end to end before fixing it.
    #[test]
    fn a_shared_version_explained_only_by_the_discarded_record_is_refused() {
        let scratch = Scratch::new("staged-record-discarded");
        let staged = stage(
            &scratch,
            "staging",
            &[("2026-01-01-a", "0.1.0"), ("2026-01-02-b", "0.1.0")],
        );
        // The staged record explains the collision, which is what the workflow instructs.
        std::fs::write(
            staged.join(RELEASE_RECORD),
            "# Changelog\n\n## 0.1.0\n\nBaseline covering a and b.\n",
        )
        .expect("the staged record");

        // The destination already holds a record with no entry for the shared version. Promotion
        // keeps this one, so it is the one the check must read.
        let destination = scratch.directory("repository");
        std::fs::write(
            destination.join(RELEASE_RECORD),
            "# Changelog\n\n## 2.0.0\n\nThe destination history.\n",
        )
        .expect("their record");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        let Some(Fault::UnexplainedSharedVersion { version, .. }) = faults
            .iter()
            .find(|fault| matches!(fault, Fault::UnexplainedSharedVersion { .. }))
        else {
            panic!("the discarded explanation was accepted: {faults:?}");
        };
        assert_eq!(version, "0.1.0");
    }

    /// A shared version the destination's own record explains is accepted.
    ///
    /// The other side of the repair above. Reading the surviving record must not mean refusing a
    /// destination that already explains the collision — that is the ordinary case for a repository
    /// migrating its own history, and refusing it would be the deadlock this change already fixed
    /// once.
    #[test]
    fn a_shared_version_the_destinations_record_explains_is_accepted() {
        let scratch = Scratch::new("destination-record-explains");
        let staged = stage(
            &scratch,
            "staging",
            &[("2026-01-01-a", "0.1.0"), ("2026-01-02-b", "0.1.0")],
        );
        // The staged record says nothing about it, and the destination's does.
        std::fs::write(staged.join(RELEASE_RECORD), "# Changelog\n").expect("the staged record");
        let destination = scratch.directory("repository");
        std::fs::write(
            destination.join(RELEASE_RECORD),
            "# Changelog\n\n## 0.1.0\n\nBaseline covering a and b.\n",
        )
        .expect("their record");

        assert!(
            Staged::resolve(&staged, &destination).is_ok(),
            "a destination that explains the collision was refused"
        );
    }

    /// A shared version the release record explains is accepted, carried rather than resolved.
    ///
    /// The other half of the requirement, and the half that matters most: inventing distinct
    /// versions to satisfy the check would record releases nobody installed, which the contract
    /// calls worse than recording the shared version honestly.
    #[test]
    fn a_shared_version_the_release_record_explains_is_carried() {
        let scratch = Scratch::new("shared-explained");
        let staged = stage(
            &scratch,
            "staging",
            &[("2026-08-10-a", "0.1.0"), ("2026-08-13-b", "0.1.0")],
        );
        std::fs::write(
            staged.join(RELEASE_RECORD),
            "# Changelog\n\n## 0.1.0\n\nThe baseline release, covering 2026-08-10-a and \
             2026-08-13-b, which were archived together before recorded versions were required \
             to be distinct.\n",
        )
        .expect("the release record is written");
        let destination = scratch.directory("repository");

        assert!(
            Staged::resolve(&staged, &destination).is_ok(),
            "an explained collision was refused"
        );
    }

    /// A project with no archived change declares whatever it declares. Requiring the declaration
    /// to equal an absent value would refuse a young source project for being young.
    #[test]
    fn a_staged_project_with_no_archived_change_declares_freely() {
        let scratch = Scratch::new("no-archive");
        let staged = stage(&scratch, "staging", &[]);
        let destination = scratch.directory("repository");
        assert!(Staged::resolve(&staged, &destination).is_ok());
    }

    /// Promotion refuses while the destination already holds anything it would place, so nothing
    /// is overwritten and nothing is merged.
    #[test]
    fn a_destination_already_holding_a_project_is_refused() {
        let scratch = Scratch::new("occupied");
        let staged = stage(&scratch, "staging", &[]);
        let destination = scratch.directory("repository");
        std::fs::create_dir_all(destination.join("htmlspec/specs")).expect("an existing project");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        let Some(Fault::DestinationOccupied { occupied }) = faults
            .iter()
            .find(|fault| matches!(fault, Fault::DestinationOccupied { .. }))
        else {
            panic!("the occupied destination was accepted: {faults:?}");
        };
        assert_eq!(occupied, &["htmlspec"]);
    }

    /// Every fault is reported together. A staged tree with three faults reported one at a time
    /// costs three runs, which is the waste the check-last rule exists to prevent.
    #[test]
    fn every_staging_fault_is_reported_together() {
        let scratch = Scratch::new("all-faults");
        let destination = scratch.directory("repository");
        std::fs::create_dir_all(destination.join("htmlspec")).expect("an existing project");
        let staged = scratch.directory("repository/staging");
        std::fs::create_dir_all(staged.join("htmlspec")).expect("a partial project");

        let faults = Staged::resolve(&staged, &destination).expect_err("refused");
        assert!(
            faults.len() >= 3,
            "only {} fault(s) were reported: {faults:?}",
            faults.len()
        );
    }

    /// Promotion places the whole project, and the staging location is left holding none of it.
    #[test]
    fn promotion_places_the_whole_project() {
        let scratch = Scratch::new("promote");
        let staged = stage(&scratch, "staging", &[("2026-08-10-html-spec", "0.1.0")]);
        std::fs::write(staged.join(RELEASE_RECORD), "# Changelog\n\n## 0.1.0\n")
            .expect("the release record is written");
        let destination = scratch.directory("repository");

        let resolved = Staged::resolve(&staged, &destination).expect("resolved");
        let moved = resolved.promote().expect("promoted");
        assert_eq!(moved, vec!["htmlspec", "spec.nix", "CHANGELOG.md"]);

        for entry in PLACED {
            assert!(
                destination.join(entry).exists(),
                "{entry} did not arrive in the destination"
            );
            assert!(
                !staged.join(entry).exists(),
                "{entry} is still in the staging location"
            );
        }
        // And the promoted tree is a project the toolchain resolves, which is the point of
        // mirroring the destination's paths in staging.
        assert_eq!(
            ProjectRoot::resolve(&destination)
                .expect("the promoted tree is a project")
                .path(),
            destination
        );
        assert!(destination
            .join("htmlspec/changes/archive/2026-08-10-html-spec/VERSION")
            .exists());
    }

    /// A staged project with no release record promotes what it has. A source project that never
    /// kept a changelog does not gain one from a migration.
    #[test]
    fn a_staged_project_with_no_release_record_promotes_what_it_has() {
        let scratch = Scratch::new("no-changelog");
        let staged = stage(&scratch, "staging", &[]);
        let destination = scratch.directory("repository");
        let moved = Staged::resolve(&staged, &destination)
            .expect("resolved")
            .promote()
            .expect("promoted");
        assert_eq!(moved, vec!["htmlspec", "spec.nix"]);
        assert!(!destination.join(RELEASE_RECORD).exists());
    }

    /// A failure part-way through the move leaves the destination as it was.
    ///
    /// Provoked by making the destination's `spec.nix` an existing **directory**, which the second
    /// move cannot replace. The project directory has already arrived at that point, so a
    /// promotion without a rollback would leave a destination holding a migrated `htmlspec/` and
    /// no declaration — a project the toolchain resolves and cannot read the version of.
    #[test]
    fn a_failed_move_leaves_the_destination_unchanged() {
        let scratch = Scratch::new("rollback");
        let staged = stage(&scratch, "staging", &[("2026-08-10-html-spec", "0.1.0")]);
        let destination = scratch.directory("repository");

        let resolved = Staged::resolve(&staged, &destination).expect("resolved");
        // Made read-only *after* the check, so `resolve` did not see it. That is what makes this a
        // mid-flight failure rather than a precondition: the project directory has already arrived
        // when the declaration's move fails.
        //
        // An occupied target used to be the provocation here, and it stopped failing once
        // promotion learned to skip one. That is the correct behavior for the release record, so
        // the test moved to a cause promotion cannot skip.
        let mut permissions = std::fs::metadata(&destination)
            .expect("the destination is readable")
            .permissions();
        let restore = permissions.clone();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            permissions.set_mode(0o500);
        }
        #[cfg(not(unix))]
        permissions.set_readonly(true);
        std::fs::set_permissions(&destination, permissions).expect("the destination is locked");

        let outcome = resolved.promote();
        // Restored before any assertion, so a failing assertion does not leave the scratch
        // directory unremovable and take the next run's setup down with it.
        std::fs::set_permissions(&destination, restore).expect("the destination is unlocked");

        let error = outcome.expect_err("the move fails");
        assert!(error.raw_os_error().is_some(), "{error}");

        // The destination holds none of the migrated project.
        assert!(
            !destination.join(DIRECTORY).exists(),
            "the project directory stayed in the destination after a failed move"
        );
        // And the staged tree is whole again, so the migration can be retried.
        assert!(staged.join(DIRECTORY).is_dir());
        assert!(staged.join(DECLARATION).is_file());
        assert!(staged
            .join("htmlspec/changes/archive/2026-08-10-html-spec/VERSION")
            .exists());
    }

    /// A copy that fails part-way leaves no partial entry behind.
    ///
    /// **The branch most likely to run, and the one whose failure was unrecoverable.** `rename`
    /// fails with `EXDEV` when staging and destination are on different filesystems, which a staging
    /// directory under the system temporary directory usually is. The fallback copies and then
    /// removes — so it writes into the destination *before* anything leaves staging.
    ///
    /// A failure part-way could not be rolled back by the caller: `promote` unwinds the entries it
    /// recorded as moved, and the entry being copied is not recorded until the copy returns. So a
    /// full disk on the copy left the destination holding a partial project while promotion reported
    /// it unchanged and said it was safe to run again. It was not — the retry then tripped the
    /// occupied-destination refusal. A review of the pull request found it.
    ///
    /// **This drives `copy_then_remove` rather than `rename_or_copy`, and that is deliberate.** Two
    /// earlier versions of this test tried to reach the fallback through `rename_or_copy` and could
    /// not: locking the destination's parent fails in `create_dir_all` before `rename` runs, and an
    /// occupied destination makes `rename` fail with `ENOTEMPTY` rather than `EXDEV`, so
    /// `crosses_devices` is false and the fallback is skipped. `EXDEV` needs two real filesystems.
    /// Naming the fallback as its own function is what makes its recovery testable at all.
    #[test]
    fn a_failed_copy_leaves_no_partial_entry_in_the_destination() {
        let scratch = Scratch::new("copy-cleanup");
        let from = scratch.directory("staged-entry");
        std::fs::write(from.join("a.html"), "content").expect("a staged file");
        std::fs::create_dir_all(from.join("nested")).expect("a staged directory");
        std::fs::write(from.join("nested/b.html"), "more").expect("a nested file");

        // A destination whose nested path is occupied by a file where the copy needs a directory.
        // `copy_tree` creates the top level, copies `a.html`, and then fails on `nested` — so it
        // dies after writing, which is the state the caller cannot roll back.
        let to = scratch.0.join("destination-entry");
        std::fs::create_dir_all(&to).expect("the destination entry");
        std::fs::write(to.join("nested"), "a file where a directory must go").expect("obstruction");

        let outcome = copy_then_remove(&from, &to);
        assert!(outcome.is_err(), "the copy succeeded: {outcome:?}");

        // The whole entry is gone rather than left holding what the copy managed to write. Without
        // the cleanup, `a.html` sits in the destination and `promote` reports it unchanged.
        assert!(
            !to.exists(),
            "the destination kept a partial entry: {:?}",
            std::fs::read_dir(&to).map(|d| d.flatten().map(|e| e.file_name()).collect::<Vec<_>>())
        );
        assert!(from.join("a.html").is_file(), "the staged tree lost a file");
        assert!(from.join("nested/b.html").is_file());
    }

    /// The staged tree's paths are the ones the destination will hold, so promotion rewrites no
    /// asset reference.
    ///
    /// Asserted on the reference text rather than on the paths, because the claim is about what a
    /// document says: a specification one level below the project root reaches `../assets/`, and a
    /// change's document two levels below reaches `../../assets/`. Both are true in staging and
    /// after the move, which is what makes promotion a move.
    #[test]
    fn asset_references_resolve_unchanged_across_the_move() {
        let scratch = Scratch::new("references");
        let staged = stage(&scratch, "staging", &[]);
        let documents = [
            ("htmlspec/specs/project-bootstrap/spec.html", "../../assets"),
            ("htmlspec/specs/one.html", "../assets"),
            ("htmlspec/changes/m/proposal.html", "../../assets"),
            (
                "htmlspec/changes/archive/2026-08-10-html-spec/specification.html",
                "../../../assets",
            ),
        ];
        for (relative, prefix) in documents {
            let path = staged.join(relative);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory");
            std::fs::write(&path, format!("<link href=\"{prefix}/htmlspec.css\">\n"))
                .expect("the document is written");
            // The reference reaches the staged store, which is the property the depth encodes.
            assert_eq!(
                resolve_reference(&path, prefix),
                staged.join("htmlspec/assets"),
                "{relative} references {prefix} and that does not reach the store in staging"
            );
        }
        let destination = scratch.directory("repository");
        Staged::resolve(&staged, &destination)
            .expect("resolved")
            .promote()
            .expect("promoted");

        for (relative, prefix) in documents {
            let path = destination.join(relative);
            let text = std::fs::read_to_string(&path).expect("the promoted document is readable");
            assert!(
                text.contains(&format!("{prefix}/htmlspec.css")),
                "{relative}'s reference was rewritten: {text}"
            );
            // And the same unrewritten reference reaches the promoted store, computed from the
            // document's own directory exactly as it was in staging. That is what "promotion is a
            // move" means: identical text, identical resolution, one level up.
            assert_eq!(
                resolve_reference(&path, prefix),
                destination.join("htmlspec/assets"),
                "{relative}'s reference does not reach the project's store after promotion"
            );
        }
    }

    /// Where `reference` points, read from the directory holding `document`.
    ///
    /// `Path::components()` keeps a `..` rather than resolving it, so this pops instead. Written
    /// out because the whole claim of the test above is about what a relative reference resolves
    /// to, and a comparison that left `..` in both sides would hold for any depth at all.
    fn resolve_reference(document: &Path, reference: &str) -> PathBuf {
        let mut resolved = document
            .parent()
            .expect("the document has a parent")
            .to_owned();
        for part in reference.split('/') {
            match part {
                ".." => {
                    resolved.pop();
                }
                "." | "" => {}
                name => resolved.push(name),
            }
        }
        resolved
    }
}
