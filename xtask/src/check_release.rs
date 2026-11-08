//! The release check: whether this repository's release record is coherent, and with `--tag`,
//! whether a tag names the version the crates declare.
//!
//! # Why it asks about the archive rather than the working change
//!
//! A stage of `scripts/check` passes or fails, with no advisory level between. "Fail if the version
//! has not advanced" would be red from a change's first commit to its last, because a change under
//! way has not advanced the version yet: the patch-versus-minor decision is best made once the
//! work's shape is known. So it asks four questions of the archive, which is a settled record:
//!
//!  1. Every archived change records a version.
//!  2. Every recorded version above the baseline is distinct.
//!  3. Every recorded version has a changelog entry, and so does the version the crates declare.
//!  4. Every published runbook release is accounted for, and every runbook the project holds can
//!     name the release it is at.
//!
//! A runbook whose canonical documents changed while its version did not is not a fault here:
//! unarchived work can still be choosing its release number. Archival is where the increment
//! becomes obligatory, and `htmlspec archive --check` reports it before then. The cost, accepted
//! deliberately: a change under way with no changelog entry passes.
//!
//! Beside the archive it asks two questions of the crates the workspace publishes, and with
//! `--tag` a third, which the release workflow asks before it builds anything the release ships:
//!
//!  5. Every internal dependency of a published crate pins exactly the declared version.
//!  6. Every published crate carries a copy of the root `LICENSE`, byte for byte.
//!  7. The tag names exactly the declared version.
//!
//! Each has a right answer at every moment. `cargo xtask set-version` moves the version and every
//! pin in one edit, so a pin that differs is a stale string, never work in progress, and a crate
//! published to crates.io holds its own directory and nothing else, so it carries its own licence.
//!
//! # One reader
//!
//! Every format this check reads is read through the program's library, the code archival runs: an
//! archived change's version as a `ProjectVersion`, a changelog's headings by
//! `runbook_release::changelog_versions`, the archive and the runbook inventory by
//! `runbook_release::history`, and a runbook's release by `runbook_release::compare_release`. So
//! this check and archival cannot disagree about any of them, which is what a second reader with
//! shared fixtures used to be tested for, and failed at: the Python reader read a runbook version
//! of `0.1` as one, where archival refuses the inventory that holds it. The workspace is read
//! through `cargo metadata`, the version the crates declare and a tag's as a [`ReleaseVersion`],
//! and the baseline from `xtask/release-facts.json`.
//!
//! Exit codes: 0 clean, 1 faults found, 2 could not ask.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use htmlspec_cli::project::{NotAProject, ProjectRoot, RECORDED_VERSION};
use htmlspec_cli::runbook_release::{self, ReleaseFault, CHANGELOG_FILE};
use htmlspec_cli::version::{ProjectVersion, ReleaseVersion, VersionError};

use crate::facts::{Facts, FactsFault};
use crate::report::{Exit, Report};
use crate::workspace::{
    CrateName, Member, MetadataFault, Requirement, UndeclaredVersion, Workspace,
};

/// The project's changelog, at the repository's root.
const CHANGELOG: &str = "CHANGELOG.md";

/// The licence text every published crate carries a copy of, named as the root's file is.
const LICENSE: &str = "LICENSE";

/// What one archived change records as the version it shipped in.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Recorded {
    Version(ProjectVersion),
    Missing,
    Unreadable(std::io::ErrorKind),
    NotAVersion { text: String, reason: VersionError },
}

/// One archived change, by its directory's name.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Archived {
    name: String,
    recorded: Recorded,
}

/// One way the release record is wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    NoChangelog,
    UnreadableChangelog(std::io::ErrorKind),
    RecordsNoVersion {
        change: String,
    },
    UnreadableVersion {
        change: String,
        kind: std::io::ErrorKind,
    },
    RecordsNotAVersion {
        change: String,
        text: String,
        reason: VersionError,
    },
    SharedVersion {
        change: String,
        version: ProjectVersion,
        first: String,
    },
    NoChangelogEntry {
        version: ProjectVersion,
    },
    /// A runbook the project holds cannot name its release, or the published history cannot be
    /// read, in the words archival uses.
    Runbook(ReleaseFault),
    /// A published snapshot that could not have followed the one before it.
    Published {
        archive: String,
        fault: ReleaseFault,
    },
    PinNamesNoVersion {
        dependency: CrateName,
        dependents: Vec<CrateName>,
        declared: ReleaseVersion,
    },
    StalePin {
        dependency: CrateName,
        requirement: Requirement,
        dependents: Vec<CrateName>,
        declared: ReleaseVersion,
    },
    NoRootLicense,
    RootLicenseUnreadable(std::io::ErrorKind),
    LicenseMissing {
        copy: PathBuf,
    },
    LicenseUnreadable {
        copy: PathBuf,
        kind: std::io::ErrorKind,
    },
    LicenseDiffers {
        copy: PathBuf,
    },
    NotAReleaseTag {
        tag: String,
    },
    TagNamesAnother {
        tag: String,
        named: ReleaseVersion,
        declared: ReleaseVersion,
    },
}

/// `names`, joined by commas.
fn listed(names: &[CrateName]) -> String {
    names
        .iter()
        .map(CrateName::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoChangelog => write!(
                formatter,
                "no {CHANGELOG} at the repository root. Every version the project ships records \
                 what changed in it"
            ),
            Self::UnreadableChangelog(kind) => write!(
                formatter,
                "{CHANGELOG} at the repository root cannot be read: {kind}"
            ),
            Self::RecordsNoVersion { change } => write!(
                formatter,
                "{change} records no version. Add {RECORDED_VERSION} naming the version it shipped \
                 in, so the release it belongs to is recoverable"
            ),
            Self::UnreadableVersion { change, kind } => write!(
                formatter,
                "{change}'s {RECORDED_VERSION} cannot be read: {kind}"
            ),
            Self::RecordsNotAVersion {
                change,
                text,
                reason,
            } => write!(
                formatter,
                "{change} records {text:?} in {RECORDED_VERSION}, which is not a version: {reason}. \
                 Archival reads it as no version at all, so the release it belongs to is not \
                 recoverable"
            ),
            Self::SharedVersion {
                change,
                version,
                first,
            } => write!(
                formatter,
                "{change} records version {version}, which {first} already records. A version \
                 names the change that produced it"
            ),
            Self::NoChangelogEntry { version } => write!(
                formatter,
                "no {CHANGELOG} heading names version {version}. A shipped version records what \
                 changed in it"
            ),
            Self::Runbook(fault) => fault.fmt(formatter),
            Self::Published { archive, fault } => write!(formatter, "{archive}: {fault}"),
            Self::PinNamesNoVersion {
                dependency,
                dependents,
                declared,
            } => write!(
                formatter,
                "{dependency} names no version where {} depend(s) on it. Add version = \
                 \"={declared}\" beside its path, because the crates release together and pin one \
                 another exactly",
                listed(dependents)
            ),
            Self::StalePin {
                dependency,
                requirement,
                dependents,
                declared,
            } => write!(
                formatter,
                "{dependency} is pinned at \"{requirement}\" where {} depend(s) on it, and the \
                 workspace declares {declared}. Run cargo xtask set-version, which moves the \
                 version and every pin together",
                listed(dependents)
            ),
            Self::RootLicenseUnreadable(kind) => write!(
                formatter,
                "the root {LICENSE} cannot be read: {kind}, so no crate's copy can be compared"
            ),
            Self::NoRootLicense => write!(
                formatter,
                "no {LICENSE} at the repository root, so no crate's copy can be compared"
            ),
            Self::LicenseMissing { copy } => write!(
                formatter,
                "{} is missing. A published crate holds only its own directory, so it carries a \
                 copy of the root {LICENSE}",
                copy.display()
            ),
            Self::LicenseUnreadable { copy, kind } => {
                write!(formatter, "{} cannot be read: {kind}", copy.display())
            }
            Self::LicenseDiffers { copy } => write!(
                formatter,
                "{} differs from the root {LICENSE}. Copy the root's over it, so every crate is \
                 published under the license the project declares",
                copy.display()
            ),
            Self::NotAReleaseTag { tag } => write!(
                formatter,
                "tag {tag} is not a release tag. A release tag is v and the version, as in v1.2.3"
            ),
            Self::TagNamesAnother {
                tag,
                named,
                declared,
            } => write!(
                formatter,
                "tag {tag} names {named}, and Cargo.toml declares {declared}. A release is tagged \
                 at the commit that declares its version"
            ),
        }
    }
}

/// Why the check could not ask anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CannotAsk {
    Metadata(MetadataFault),
    Undeclared(UndeclaredVersion),
    NoProject(NotAProject),
    NoArchive {
        archive: PathBuf,
    },
    UnreadableArchive {
        archive: PathBuf,
        kind: std::io::ErrorKind,
    },
    Facts(FactsFault),
}

impl fmt::Display for CannotAsk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metadata(fault) => fault.fmt(formatter),
            Self::Undeclared(fault) => fault.fmt(formatter),
            Self::NoProject(fault) => fault.fmt(formatter),
            Self::NoArchive { archive } => {
                write!(formatter, "no archived changes under {}", archive.display())
            }
            Self::UnreadableArchive { archive, kind } => write!(
                formatter,
                "the archive {} cannot be read: {kind}",
                archive.display()
            ),
            Self::Facts(fault) => write!(formatter, "xtask/release-facts.json {fault}"),
        }
    }
}

/// What the check found, when it could ask.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Answers {
    declared: ReleaseVersion,
    archived: usize,
    in_the_changelog: usize,
    pinned: usize,
    licensed: usize,
    tag: Option<String>,
    faults: Vec<Fault>,
}

impl Answers {
    fn report(self) -> Report {
        let declared = self.declared;
        if self.faults.is_empty() {
            let mut out = vec![
                format!(
                    "  ok  {} archived change(s) record a version, and {} version(s) are in the \
                     changelog",
                    self.archived, self.in_the_changelog
                ),
                format!("  ok  declared version {declared} has a changelog entry"),
                format!(
                    "  ok  {} internal dependency pin(s) name ={declared}",
                    self.pinned
                ),
                format!(
                    "  ok  {} published crate(s) carry the root {LICENSE}",
                    self.licensed
                ),
            ];
            if let Some(tag) = self.tag {
                out.push(format!("  ok  tag {tag} names the declared version"));
            }
            return Report {
                exit: Exit::Clean,
                out,
                err: Vec::new(),
            };
        }
        let mut err = vec![String::from("release hygiene:")];
        err.extend(self.faults.iter().map(|fault| format!("!!! {fault}")));
        err.push(String::new());
        err.push(format!(
            "{} fault(s) across {} archived change(s), declared version {declared}",
            self.faults.len(),
            self.archived
        ));
        Report {
            exit: Exit::Faults,
            out: Vec::new(),
            err,
        }
    }
}

/// The publishable members, the crates a release publishes.
fn published(workspace: &Workspace) -> impl Iterator<Item = &Member> {
    workspace.members.iter().filter(|member| member.publishable)
}

/// Every archived change, by name, and the version it records: none where the archive does not
/// exist, and a refusal where it exists and cannot be read, which is not the same answer.
fn archived(archive: &Path) -> Result<Vec<Archived>, CannotAsk> {
    let unreadable = |error: std::io::Error| CannotAsk::UnreadableArchive {
        archive: archive.to_path_buf(),
        kind: error.kind(),
    };
    let entries = match std::fs::read_dir(archive) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(unreadable(error)),
    };
    let entries = entries.collect::<Result<Vec<_>, _>>().map_err(unreadable)?;
    let mut found: Vec<Archived> = entries
        .into_iter()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let recorded = match std::fs::read_to_string(entry.path().join(RECORDED_VERSION)) {
                Ok(text) => match ProjectVersion::parse(&text) {
                    Ok(version) => Recorded::Version(version),
                    Err(reason) => Recorded::NotAVersion {
                        text: text.trim_end_matches(['\n', '\r']).to_owned(),
                        reason,
                    },
                },
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Recorded::Missing,
                Err(error) => Recorded::Unreadable(error.kind()),
            };
            Archived { name, recorded }
        })
        .collect();
    found.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(found)
}

/// Questions 1 to 3: each change records a version, the versions above the baseline are distinct,
/// and each version, the declared one among them, has a changelog entry. Returns how many distinct
/// versions the changelog was asked for.
fn archive_faults(
    archives: &[Archived],
    changelog: &BTreeSet<ProjectVersion>,
    baseline: ProjectVersion,
    declared: ReleaseVersion,
    faults: &mut Vec<Fault>,
) -> usize {
    for archived in archives {
        let change = archived.name.clone();
        match &archived.recorded {
            Recorded::Version(_) => {}
            Recorded::Missing => faults.push(Fault::RecordsNoVersion { change }),
            Recorded::Unreadable(kind) => faults.push(Fault::UnreadableVersion {
                change,
                kind: *kind,
            }),
            Recorded::NotAVersion { text, reason } => faults.push(Fault::RecordsNotAVersion {
                change,
                text: text.clone(),
                reason: reason.clone(),
            }),
        }
    }

    // At or below the baseline several changes share one release, which is recorded as the truth.
    let mut seen: BTreeMap<ProjectVersion, &str> = BTreeMap::new();
    for archived in archives {
        let Recorded::Version(version) = archived.recorded else {
            continue;
        };
        if version <= baseline {
            continue;
        }
        match seen.get(&version) {
            Some(first) => faults.push(Fault::SharedVersion {
                change: archived.name.clone(),
                version,
                first: (*first).to_owned(),
            }),
            None => {
                seen.insert(version, &archived.name);
            }
        }
    }

    let mut wanted: BTreeSet<ProjectVersion> = archives
        .iter()
        .filter_map(|archived| match archived.recorded {
            Recorded::Version(version) => Some(version),
            _ => None,
        })
        .collect();
    wanted.insert(declared.project());
    for version in &wanted {
        if !changelog.contains(version) {
            faults.push(Fault::NoChangelogEntry { version: *version });
        }
    }
    wanted.len()
}

/// Question 4: every runbook the project holds names a release its changelog heads, and every
/// published snapshot could have followed the one before it under its name.
fn runbook_faults(root: &Path, project: &ProjectRoot) -> Vec<Fault> {
    let mut faults = Vec::new();
    for (name, directory) in runbook_release::present(&project.runbooks()) {
        let shown = directory
            .strip_prefix(root)
            .unwrap_or(&directory)
            .display()
            .to_string();
        let unreadable = |problem: String| {
            Fault::Runbook(ReleaseFault::UnreadableRunbook {
                name: name.clone(),
                directory: shown.clone(),
                problem,
            })
        };
        let version = match runbook_release::read_version(&directory) {
            Ok(version) => version,
            Err(problem) => {
                faults.push(unreadable(problem));
                continue;
            }
        };
        if !version.is_released() {
            faults.push(Fault::Runbook(ReleaseFault::UnreleasedVersion {
                name: name.clone(),
                version,
            }));
        }
        match std::fs::read_to_string(directory.join(CHANGELOG_FILE)) {
            Ok(changelog) => {
                if !runbook_release::changelog_headings(&changelog).contains(&version) {
                    faults.push(Fault::Runbook(ReleaseFault::NoHeading {
                        name: name.clone(),
                        version,
                    }));
                }
            }
            Err(error) => faults.push(unreadable(format!(
                "its {CHANGELOG_FILE} could not be read: {error}"
            ))),
        }
    }

    match runbook_release::history(&project.archive(), &project.runbook_releases()) {
        Err(found) => faults.extend(found.faults().iter().cloned().map(Fault::Runbook)),
        Ok((_, snapshots)) => {
            for (index, snapshot) in snapshots.iter().enumerate() {
                let previous =
                    runbook_release::predecessor(&snapshots[..index], &snapshot.release.name);
                if let Err(found) =
                    runbook_release::compare_release(previous, snapshot.release.clone())
                {
                    faults.extend(
                        found
                            .faults()
                            .iter()
                            .cloned()
                            .map(|fault| Fault::Published {
                                archive: snapshot.archive.clone(),
                                fault,
                            }),
                    );
                }
            }
        }
    }
    faults
}

/// Question 5: every dependency a published crate has on another member, which its published
/// manifest names, pins exactly the declared version. Returns how many crates are pinned.
fn pin_faults(workspace: &Workspace, declared: ReleaseVersion, faults: &mut Vec<Fault>) -> usize {
    let members: BTreeSet<&CrateName> = workspace
        .members
        .iter()
        .map(|member| &member.name)
        .collect();
    let mut pins: BTreeMap<(&CrateName, &Requirement), BTreeSet<&CrateName>> = BTreeMap::new();
    for member in published(workspace) {
        for dependency in &member.dependencies {
            if dependency.reaches_the_registry && members.contains(&dependency.name) {
                pins.entry((&dependency.name, &dependency.requirement))
                    .or_default()
                    .insert(&member.name);
            }
        }
    }
    let pinned: BTreeSet<&CrateName> = pins.keys().map(|(name, _)| *name).collect();
    for ((dependency, requirement), dependents) in pins {
        if requirement.pins_exactly(declared) {
            continue;
        }
        let dependency = dependency.clone();
        let dependents: Vec<CrateName> = dependents.into_iter().cloned().collect();
        faults.push(if requirement.names_none() {
            Fault::PinNamesNoVersion {
                dependency,
                dependents,
                declared,
            }
        } else {
            Fault::StalePin {
                dependency,
                requirement: requirement.clone(),
                dependents,
                declared,
            }
        });
    }
    pinned.len()
}

/// Question 6: every published crate carries the root's licence text. Returns how many crates
/// were asked about.
fn license_faults(workspace: &Workspace, faults: &mut Vec<Fault>) -> usize {
    let mut crates: Vec<&Member> = published(workspace).collect();
    crates.sort_by(|left, right| left.directory.cmp(&right.directory));
    let original = match std::fs::read(workspace.root.join(LICENSE)) {
        Ok(original) => original,
        Err(error) => {
            faults.push(if error.kind() == std::io::ErrorKind::NotFound {
                Fault::NoRootLicense
            } else {
                Fault::RootLicenseUnreadable(error.kind())
            });
            return crates.len();
        }
    };
    for member in &crates {
        let path = member.directory.join(LICENSE);
        let copy = path
            .strip_prefix(&workspace.root)
            .unwrap_or(&path)
            .to_path_buf();
        match std::fs::read(&path) {
            Ok(text) if text == original => {}
            Ok(_) => faults.push(Fault::LicenseDiffers { copy }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                faults.push(Fault::LicenseMissing { copy })
            }
            Err(error) => faults.push(Fault::LicenseUnreadable {
                copy,
                kind: error.kind(),
            }),
        }
    }
    crates.len()
}

/// Question 7: the tag is `v` and exactly the declared version, as a release writes it.
fn tag_fault(tag: &str, declared: ReleaseVersion) -> Option<Fault> {
    let Some(named) = tag
        .strip_prefix('v')
        .and_then(|version| ReleaseVersion::parse(version).ok())
    else {
        return Some(Fault::NotAReleaseTag {
            tag: tag.to_owned(),
        });
    };
    (named != declared).then(|| Fault::TagNamesAnother {
        tag: tag.to_owned(),
        named,
        declared,
    })
}

/// Asks every question of the repository `workspace` is rooted at, and of its crates, exempting
/// from distinctness the versions at or below `baseline`.
fn ask(
    workspace: &Workspace,
    baseline: ProjectVersion,
    tag: Option<&str>,
) -> Result<Answers, CannotAsk> {
    let declared = workspace.release_version().map_err(CannotAsk::Undeclared)?;
    let root = &workspace.root;
    let project = ProjectRoot::resolve(root).map_err(CannotAsk::NoProject)?;
    let archive = project.archive();
    let archives = archived(&archive)?;
    if archives.is_empty() {
        return Err(CannotAsk::NoArchive { archive });
    }

    let mut faults = Vec::new();
    let changelog: BTreeSet<ProjectVersion> = match std::fs::read_to_string(root.join(CHANGELOG)) {
        Ok(text) => runbook_release::changelog_versions(&text)
            .into_iter()
            .collect(),
        Err(error) => {
            faults.push(if error.kind() == std::io::ErrorKind::NotFound {
                Fault::NoChangelog
            } else {
                Fault::UnreadableChangelog(error.kind())
            });
            BTreeSet::new()
        }
    };
    let in_the_changelog = archive_faults(&archives, &changelog, baseline, declared, &mut faults);
    faults.extend(runbook_faults(root, &project));
    let pinned = pin_faults(workspace, declared, &mut faults);
    let licensed = license_faults(workspace, &mut faults);
    if let Some(tag) = tag {
        faults.extend(tag_fault(tag, declared));
    }
    Ok(Answers {
        declared,
        archived: archives.len(),
        in_the_changelog,
        pinned,
        licensed,
        tag: tag.map(str::to_owned),
        faults,
    })
}

/// The check of the repository `workspace` is rooted at, with `baseline` the version at or below
/// which its archived changes may share one, as it ends.
pub fn check(workspace: &Workspace, baseline: ProjectVersion, tag: Option<&str>) -> Report {
    match ask(workspace, baseline, tag) {
        Ok(answers) => answers.report(),
        Err(cannot) => Report::could_not_ask(&cannot),
    }
}

/// Runs the check on the workspace the current directory belongs to, read through the cargo that
/// runs this tool, with the baseline `xtask/release-facts.json` states.
pub fn run(tag: Option<&str>) -> ExitCode {
    let baseline = match Facts::read() {
        Ok(facts) => facts.shared_release_baseline(),
        Err(fault) => return Report::could_not_ask(&CannotAsk::Facts(fault)).print(),
    };
    match crate::workspace::Workspace::here() {
        Ok(workspace) => check(&workspace, baseline, tag).print(),
        Err(fault) => Report::could_not_ask(&CannotAsk::Metadata(fault)).print(),
    }
}

#[cfg(test)]
mod tests;
