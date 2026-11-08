//! `cargo xtask set-version X.Y.Z`: move the workspace's version and every internal pin together,
//! then relock.
//!
//! # Why one command
//!
//! The six crates release together, so each `htmlspec-*` entry in `[workspace.dependencies]` pins
//! the others to exactly the workspace's version, `version = "=X.Y.Z"`. Cargo cannot read the
//! workspace version into a dependency entry, so those pins are strings, and a pin can only be right
//! if it moves with the version. Raising the version under `[workspace.package]` alone and relocking
//! fails: cargo 1.90 exits 101 with "failed to select a version for the requirement
//! `htmlspec-parse = "=0.32.0"`", and every stage of `scripts/check` that builds fails the same way
//! before `cargo xtask check-release` can name the stale pin. So the version, the pins, and the
//! lockfiles move in one command, and the release runbook's version step runs it.
//!
//! # What it does
//!
//! It writes the version under `[workspace.package]` and in every internal pin as one edit of the
//! text of `Cargo.toml`, the narrow edit design decision 5 keeps, because the crate that would make
//! it structural, a TOML editor, is not in the lockfile. Then it reads the result back through
//! `cargo metadata` and requires every member and every pin to name the new version, so the check
//! that follows the edit is structural. Then it runs `cargo update --workspace --offline`, which
//! rewrites the workspace's own entries in `Cargo.lock` and nothing else, and does the same for the
//! private workspace's `publish/Cargo.lock` where that exists, because the private workspace reaches
//! the crates' exact pins through the tools crate. Offline, because a version step has no reason to
//! reach the network.
//!
//! When a step fails, every file it may have written is put back as it was, the lockfiles beside
//! the manifest. Each write replaces its file whole, by a rename, so no file is left cut short. The
//! files are not one transaction: a run interrupted between the manifest's write and the relock, by
//! a signal or a crash, leaves the new version in the manifest and the old one in the lockfile.
//! Running the tool again with the same version repairs that, because a run with the version the
//! workspace already declares repairs stale pins and relocks, and changes nothing else.

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicUsize, Ordering};

use htmlspec_cli::version::ReleaseVersion;

use crate::report::{Exit, Report};
use crate::workspace::{CrateName, MetadataFault, PackageVersion, Requirement, Workspace};

/// The prefix every crate this workspace publishes is named with. An entry of
/// `[workspace.dependencies]` with it is one of the crates that release together.
const INTERNAL_CRATE: &str = "htmlspec-";

/// The manifest, and where the private workspace's lockfile is, from the root.
const MANIFEST: &str = "Cargo.toml";
const LOCKFILE: &str = "Cargo.lock";
const PRIVATE_MANIFEST: &str = "publish/Cargo.toml";
const PRIVATE_LOCKFILE: &str = "publish/Cargo.lock";

/// Why the manifest's text is not one this tool edits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestFault {
    /// `[workspace.package]` holds no `version = "..."` line, or more than one.
    PackageVersions { found: usize },
}

impl fmt::Display for ManifestFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageVersions { found } => write!(
                formatter,
                "expected one version under [workspace.package], found {found}"
            ),
        }
    }
}

/// The manifest's text with the version and every internal pin set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edited {
    pub text: String,
    /// The pins set, by the crate each names, in the manifest's order.
    pub pinned: Vec<String>,
}

/// The name of a table a header line opens, `[name]` or `[[name]]`, with a comment after it or
/// not.
fn header(line: &str) -> Option<&str> {
    let line = line.trim();
    let line = match line.find('#') {
        Some(comment) if line[..comment].trim_end().ends_with(']') => line[..comment].trim_end(),
        _ => line,
    };
    let inner = line.strip_prefix('[')?.strip_suffix(']')?;
    let inner = match inner.strip_prefix('[') {
        Some(array) => array.strip_suffix(']')?,
        None => inner,
    };
    let inner = inner.trim();
    (!inner.is_empty() && !inner.contains(['[', ']'])).then_some(inner)
}

/// Whether `byte` belongs to a word, as a regular expression's `\b` reads one.
fn word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Where the first `version = "..."` of `line` is, `version` being a whole word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VersionField {
    /// The byte its key begins at.
    key: usize,
    /// The byte its value begins at, just after the opening quote.
    open: usize,
    /// The closing quote's byte.
    close: usize,
}

/// The first `version = "..."` in `line`.
fn version_field(line: &str) -> Option<VersionField> {
    let bytes = line.as_bytes();
    let mut from = 0;
    while let Some(found) = line[from..].find("version") {
        let key = from + found;
        let after = key + "version".len();
        from = after;
        if key > 0 && word(bytes[key - 1]) {
            continue;
        }
        let Some(rest) = line[after..].trim_start().strip_prefix('=') else {
            continue;
        };
        let Some(value) = rest.trim_start().strip_prefix('"') else {
            continue;
        };
        let open = line.len() - value.len();
        let Some(close) = value.find('"') else {
            continue;
        };
        return Some(VersionField {
            key,
            open,
            close: open + close,
        });
    }
    None
}

/// `line`, an entry of `[workspace.dependencies]`, if it names an internal crate in an inline
/// table: the crate's name, and the byte just after the table's `{`.
fn internal_entry(line: &str) -> Option<(&str, usize)> {
    let key = line.trim_start();
    if !key.starts_with(INTERNAL_CRATE) {
        return None;
    }
    let length = key
        .find(|character: char| !(character.is_ascii_alphanumeric() || "_-".contains(character)))
        .unwrap_or(key.len());
    let name = &key[..length];
    if name.len() == INTERNAL_CRATE.len() {
        return None;
    }
    let after = key[length..].trim_start().strip_prefix('=')?;
    let brace = after.trim_start().strip_prefix('{')?;
    Some((name, line.len() - brace.len()))
}

/// The manifest with `version` under `[workspace.package]` and in every internal pin, and the pins
/// it set.
///
/// Refuses a manifest with no `version` under `[workspace.package]`, or with more than one, because
/// either is a manifest whose version this tool cannot name. An internal entry with no version is
/// given one at the head of its table.
pub fn set_version(manifest: &str, version: ReleaseVersion) -> Result<Edited, ManifestFault> {
    let mut text = String::with_capacity(manifest.len() + 64);
    let mut section: Option<&str> = None;
    let mut package_versions = 0;
    let mut pinned = Vec::new();
    let pin = format!("version = \"={version}\"");
    for line in manifest.split_inclusive('\n') {
        if let Some(name) = header(line) {
            section = Some(name);
            text.push_str(line);
            continue;
        }
        match section {
            // The whole line is the field, and nothing but space follows its value.
            Some("workspace.package") => match version_field(line) {
                Some(field)
                    if field.key == line.len() - line.trim_start().len()
                        && line[field.close + 1..].trim().is_empty() =>
                {
                    text.push_str(&line[..field.open]);
                    text.push_str(&version.to_string());
                    text.push_str(&line[field.close..]);
                    package_versions += 1;
                }
                _ => text.push_str(line),
            },
            Some("workspace.dependencies") => match internal_entry(line) {
                Some((name, brace)) => {
                    pinned.push(name.to_owned());
                    match version_field(line) {
                        Some(field) => {
                            text.push_str(&line[..field.key]);
                            text.push_str(&pin);
                            text.push_str(&line[field.close + 1..]);
                        }
                        None => {
                            text.push_str(&line[..brace]);
                            text.push_str(&format!(" {pin},"));
                            text.push_str(&line[brace..]);
                        }
                    }
                }
                None => text.push_str(line),
            },
            _ => text.push_str(line),
        }
    }
    if package_versions != 1 {
        return Err(ManifestFault::PackageVersions {
            found: package_versions,
        });
    }
    Ok(Edited { text, pinned })
}

/// What the manifest names, read back through `cargo metadata`, that is not the new version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Disagreement {
    Member {
        name: CrateName,
        version: PackageVersion,
    },
    Pin {
        dependent: CrateName,
        dependency: CrateName,
        requirement: Requirement,
    },
}

impl fmt::Display for Disagreement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Member { name, version } => write!(formatter, "{name} is at {version}"),
            Self::Pin {
                dependent,
                dependency,
                requirement,
            } => write!(
                formatter,
                "{dependent} depends on {dependency} at \"{requirement}\""
            ),
        }
    }
}

/// Every member and every pin a published crate has on another member that does not name
/// `version`, in the workspace as `cargo metadata` read it after the edit.
pub fn disagreements(workspace: &Workspace, version: ReleaseVersion) -> Vec<Disagreement> {
    let names: Vec<&CrateName> = workspace
        .members
        .iter()
        .map(|member| &member.name)
        .collect();
    let mut found = Vec::new();
    for member in &workspace.members {
        if ReleaseVersion::parse(member.version.as_str()) != Ok(version) {
            found.push(Disagreement::Member {
                name: member.name.clone(),
                version: member.version.clone(),
            });
        }
        if !member.publishable {
            continue;
        }
        for dependency in &member.dependencies {
            if dependency.reaches_the_registry
                && names.contains(&&dependency.name)
                && !dependency.requirement.pins_exactly(version)
            {
                found.push(Disagreement::Pin {
                    dependent: member.name.clone(),
                    dependency: dependency.name.clone(),
                    requirement: dependency.requirement.clone(),
                });
            }
        }
    }
    found
}

/// The two things set-version asks of cargo, so a test can pass one that fails.
pub trait Cargo {
    /// The workspace rooted at `root`, as `cargo metadata` reads it.
    fn metadata(&self, root: &Path) -> Result<Workspace, MetadataFault>;
    /// `cargo update --workspace --offline` for the workspace whose manifest is `manifest`.
    fn relock(&self, manifest: &Path) -> Result<(), Relock>;
}

/// Why a relock failed: cargo could not be started, or exited unsuccessfully.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Relock {
    NotRun(std::io::ErrorKind),
    Exited(Option<i32>),
}

impl fmt::Display for Relock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRun(kind) => write!(formatter, "could not run: {kind}"),
            Self::Exited(Some(code)) => write!(formatter, "exited {code}"),
            Self::Exited(None) => formatter.write_str("was stopped by a signal"),
        }
    }
}

/// The cargo that runs this tool.
pub struct RealCargo;

impl Cargo for RealCargo {
    fn metadata(&self, root: &Path) -> Result<Workspace, MetadataFault> {
        Workspace::read(&crate::workspace::cargo(), root)
    }

    fn relock(&self, manifest: &Path) -> Result<(), Relock> {
        // cargo's own report of a failed resolution goes to standard error as it happens, which is
        // where a contributor reads why.
        let status = Command::new(crate::workspace::cargo())
            .args(["update", "--workspace", "--offline", "--manifest-path"])
            .arg(manifest)
            .status()
            .map_err(|error| Relock::NotRun(error.kind()))?;
        if status.success() {
            Ok(())
        } else {
            Err(Relock::Exited(status.code()))
        }
    }
}

/// Why the run stopped after it wrote the manifest. Each one restores what it wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    ReadBack(MetadataFault),
    Disagrees(Vec<Disagreement>),
    Relock {
        lockfile: &'static str,
        relock: Relock,
    },
    Write {
        path: &'static str,
        kind: std::io::ErrorKind,
    },
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadBack(fault) => {
                write!(formatter, "the edited manifest does not read: {fault}")
            }
            Self::Disagrees(found) => write!(
                formatter,
                "the edited manifest does not name the version everywhere: {}",
                found
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Relock { lockfile, relock } => {
                let manifest = if *lockfile == LOCKFILE {
                    String::new()
                } else {
                    format!(" --manifest-path {PRIVATE_MANIFEST}")
                };
                write!(
                    formatter,
                    "cargo update --workspace --offline{manifest} {relock}"
                )
            }
            Self::Write { path, kind } => write!(formatter, "cannot write {path}: {kind}"),
        }
    }
}

/// One file the run may write, and its bytes before it began, so a failure puts it back.
struct Saved {
    path: &'static str,
    bytes: Vec<u8>,
}

/// Tells apart the temporary files one process writes.
static WRITTEN: AtomicUsize = AtomicUsize::new(0);

/// Replaces the file at `path` with `bytes` whole: written beside it under a name of its own,
/// flushed, and renamed over it, so an interruption leaves the old file or the new one and never
/// a file cut short.
fn replace(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temporary = path.with_file_name(format!(
        ".{name}.set-version-{}-{}",
        std::process::id(),
        WRITTEN.fetch_add(1, Ordering::Relaxed)
    ));
    let written = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    written
}

/// The run's steps after the edit: write, read back, check, relock each lockfile.
fn apply(
    root: &Path,
    edited: &Edited,
    version: ReleaseVersion,
    private: bool,
    cargo: &dyn Cargo,
) -> Result<(), Failure> {
    replace(&root.join(MANIFEST), edited.text.as_bytes()).map_err(|error| Failure::Write {
        path: MANIFEST,
        kind: error.kind(),
    })?;
    let workspace = cargo.metadata(root).map_err(Failure::ReadBack)?;
    let found = disagreements(&workspace, version);
    if !found.is_empty() {
        return Err(Failure::Disagrees(found));
    }
    cargo
        .relock(&root.join(MANIFEST))
        .map_err(|relock| Failure::Relock {
            lockfile: LOCKFILE,
            relock,
        })?;
    if private {
        cargo
            .relock(&root.join(PRIVATE_MANIFEST))
            .map_err(|relock| Failure::Relock {
                lockfile: PRIVATE_LOCKFILE,
                relock,
            })?;
    }
    Ok(())
}

/// Sets `version` in the workspace rooted at `root`, through `cargo`, and says how it ended.
pub fn set(root: &Path, version: ReleaseVersion, cargo: &dyn Cargo) -> Report {
    let manifest = root.join(MANIFEST);
    let before = match std::fs::read_to_string(&manifest) {
        Ok(text) => text,
        Err(error) => {
            return Report::could_not_ask(&format!(
                "set-version: cannot read {}: {}",
                manifest.display(),
                error.kind()
            ))
        }
    };
    let edited = match set_version(&before, version) {
        Ok(edited) => edited,
        Err(fault) => {
            return Report::could_not_ask(&format!("set-version: {}: {fault}", manifest.display()))
        }
    };
    let private = root.join(PRIVATE_LOCKFILE).is_file() && root.join(PRIVATE_MANIFEST).is_file();
    let mut saved = vec![Saved {
        path: MANIFEST,
        bytes: before.into_bytes(),
    }];
    for path in [LOCKFILE, PRIVATE_LOCKFILE] {
        if path == PRIVATE_LOCKFILE && !private {
            continue;
        }
        match std::fs::read(root.join(path)) {
            Ok(bytes) => saved.push(Saved { path, bytes }),
            Err(error) => {
                return Report::could_not_ask(&format!(
                    "set-version: cannot read {}: {}",
                    root.join(path).display(),
                    error.kind()
                ))
            }
        }
    }
    match apply(root, &edited, version, private, cargo) {
        Ok(()) => {
            let mut out = vec![
                format!(
                    "version {version}, and {} pin(s): {}",
                    edited.pinned.len(),
                    edited.pinned.join(", ")
                ),
                format!("{LOCKFILE} relocked offline"),
            ];
            if private {
                out.push(format!("{PRIVATE_LOCKFILE} relocked offline"));
            }
            Report {
                exit: Exit::Clean,
                out,
                err: Vec::new(),
            }
        }
        Err(failure) => {
            let mut err = vec![format!("set-version: {failure}")];
            let mut restored = Vec::new();
            for file in &saved {
                match replace(&root.join(file.path), &file.bytes) {
                    Ok(()) => restored.push(file.path),
                    Err(error) => err.push(format!(
                        "set-version: {} could not be restored: {}. Restore it from Git",
                        file.path,
                        error.kind()
                    )),
                }
            }
            let restored = match restored.as_slice() {
                [] => None,
                [one] => Some(format!("{one} is restored as it was")),
                [many @ .., last] => Some(format!(
                    "{}{} and {last} are restored as they were",
                    many.join(", "),
                    if many.len() > 1 { "," } else { "" }
                )),
            };
            err.extend(restored.map(|said| format!("set-version: {said}")));
            Report {
                exit: Exit::Faults,
                out: Vec::new(),
                err,
            }
        }
    }
}

/// Sets `version` in the workspace the current directory belongs to.
pub fn run(version: ReleaseVersion) -> ExitCode {
    let root: PathBuf = match Workspace::here() {
        Ok(workspace) => workspace.root,
        Err(fault) => return Report::could_not_ask(&format!("set-version: {fault}")).print(),
    };
    set(&root, version, &RealCargo).print()
}

#[cfg(test)]
mod tests;
