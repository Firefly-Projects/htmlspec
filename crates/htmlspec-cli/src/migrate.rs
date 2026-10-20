//! Enumerating an OpenSpec project, which is the only reading the program does during a
//! migration.
//!
//! A migration has exactly three exact parts and this is the first: OpenSpec's layout is
//! fixed, so listing what a project holds involves no judgment. What each of those files
//! *argues*, and which of them deserves to become a diagram rather than a paragraph, is
//! reading — and `project-migration`'s "Conversion of a document's meaning is agent-driven"
//! forbids a program from deciding it. So this module counts and names, and stops there.
//!
//! **The enumeration is derived from what the source holds, not from a list of kinds we
//! expect.** That is the requirement's own wording, and it exists because of a measured
//! failure: the design's mapping table had six rows, and this repository's own `openspec/`
//! holds four entity kinds none of those rows named — HTML documents from an earlier hand
//! conversion, per-change configuration files, delta sections recording a rename the
//! destination has no state for, and evidence nested one directory deeper than the format
//! describes. Fifteen files. A survey built from the table would have reported full coverage
//! over a set that omitted them, and the coverage gate compares the plan against *this*
//! enumeration, so those fifteen would have been lost with every check passing.
//!
//! Hence [`Survey::of`] walks directories rather than matching known names, reads
//! `config.yaml`'s live keys rather than looking for the three the design mentioned, and
//! records delta section headings as it finds them. An entity kind nobody anticipated still
//! reaches the plan, where a person can see it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The directory an OpenSpec project occupies within its repository.
pub const DIRECTORY: &str = "openspec";

/// Where an OpenSpec project keeps its accepted capability specifications.
const SPECS: &str = "specs";

/// Where an OpenSpec project keeps its changes.
const CHANGES: &str = "changes";

/// Where an OpenSpec project keeps its archived changes, beneath [`CHANGES`].
const ARCHIVE: &str = "archive";

/// The file each capability's specification is named.
const SPEC_FILE: &str = "spec.md";

/// The project's configuration file.
const CONFIG: &str = "config.yaml";

/// A change's per-change configuration file.
///
/// Not in any published description of the layout, and present in only some of this
/// repository's changes. Enumerated because it is there, which is the whole discipline this
/// module is built on.
const CHANGE_CONFIG: &str = ".openspec.yaml";

/// The file an archived change records the version it shipped in.
///
/// The same filename the destination uses, which is why the version row of the mapping is the
/// identity. `project-bootstrap`'s "Archival requires the project version to advance" requires
/// the version to be read from what a change records and **not** derived from a directory
/// name, so migration carries this file rather than composing its contents into the archive's
/// name.
const VERSION_FILE: &str = "VERSION";

/// Where a change keeps its evidence.
const EVIDENCE: &str = "evidence";

/// Where a change may keep documents already converted to the destination format.
const DOCUMENTS: &str = "documents";

/// The prose documents a change carries, in the order a reader meets them.
const CHANGE_DOCUMENTS: &[&str] = &["proposal.md", "design.md", "tasks.md"];

/// The inventory an archive carries of the runbooks it published.
///
/// A whole-directory fact about an archive, in a file beside its documents, for the reason
/// [`VERSION_FILE`] is one — and enumerated as its own kind for the reason `Unrecognized` is not the
/// answer here: a plan told this file is unrecognized is a plan asked to decide what a release record
/// becomes, and the destination has a place for it.
const RUNBOOK_INVENTORY: &str = "runbooks.json";

/// The planning root's marker naming the first release subject to runbook inventories.
const RUNBOOK_ADOPTION: &str = "runbook-releases.json";

/// Why a path could not be surveyed.
///
/// Structured rather than a string because the report has to name what was expected. A
/// contributor who pointed the survey at the wrong directory needs to know which directory it
/// looked in and what it wanted to find, and "not an OpenSpec project" alone leaves them
/// guessing at both.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotAnOpenSpecProject {
    /// The path as given.
    pub path: PathBuf,
    /// Where the survey looked for the project directory.
    pub looked_for: PathBuf,
}

impl std::fmt::Display for NotAnOpenSpecProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is not an OpenSpec project: no {DIRECTORY}/{SPECS} or {DIRECTORY}/{CHANGES} \
             directory at {}. A migration reads an OpenSpec project, so point it at the \
             repository holding one",
            self.path.display(),
            self.looked_for.display()
        )
    }
}

impl std::error::Error for NotAnOpenSpecProject {}

/// What a source entity is, as the survey found it.
///
/// One variant per kind of thing a plan has to give a disposition to. The four variants after
/// `Evidence` are the ones the design's mapping table had no row for; they are named
/// individually rather than folded into an `Other` bucket, because a plan whose unmapped
/// entities all read `other` tells a contributor nothing about what they are being asked to
/// decide.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    /// The project's configuration file.
    Configuration,
    /// An accepted capability specification.
    AcceptedSpecification,
    /// A change's proposal, design, or tasks document.
    ChangeDocument,
    /// A change's delta specification for one capability.
    DeltaSpecification,
    /// A change's recorded version.
    RecordedVersion,
    /// A file recording a transcript, a measurement, or a complaint.
    Evidence,
    /// A document already in the destination format, from an earlier conversion.
    ConvertedDocument,
    /// A change's own configuration file.
    ChangeConfiguration,
    /// An archive's inventory of the runbooks it published.
    ReleaseInventory,
    /// The planning root's marker naming when runbook publication was adopted.
    AdoptionMarker,
    /// A file the survey found and has no kind for.
    ///
    /// Deliberately reachable. A source project can hold anything, and the requirement is that
    /// the plan account for every file — so an unrecognized one is enumerated under its own
    /// kind rather than skipped.
    Unrecognized,
}

impl Kind {
    /// The label this kind carries in a payload and a report.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Configuration => "configuration",
            Self::AcceptedSpecification => "accepted-specification",
            Self::ChangeDocument => "change-document",
            Self::DeltaSpecification => "delta-specification",
            Self::RecordedVersion => "recorded-version",
            Self::Evidence => "evidence",
            Self::ConvertedDocument => "converted-document",
            Self::ChangeConfiguration => "change-configuration",
            Self::ReleaseInventory => "release-inventory",
            Self::AdoptionMarker => "adoption-marker",
            Self::Unrecognized => "unrecognized",
        }
    }
}

/// One thing in the source project that the plan owes a disposition for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    /// The path, relative to the repository root holding the project.
    ///
    /// Relative so that two surveys of the same project in different directories produce the
    /// same payload. The coverage gate compares a plan against a payload, and an absolute path
    /// would make that comparison depend on where the migration happened to run.
    pub path: PathBuf,
    /// What the survey takes this to be.
    pub kind: Kind,
    /// The change this belongs to, where it belongs to one.
    pub change: Option<String>,
}

/// What a project's configuration holds.
///
/// The live keys, read from the file. The design named three optional fields; the file in this
/// repository documents a fourth in a comment, and a project in the wild may set a fifth. So
/// this records the keys that are *set*, which is also what lets a plan distinguish a field
/// that was empty in the source from content that was dropped — the distinction
/// `project-migration`'s plan requirement demands.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Configuration {
    /// The keys set in the file, in the order a reader meets them.
    pub keys: Vec<String>,
}

/// What a change's own `.openspec.yaml` holds, as far as any stage of a migration needs it.
///
/// project-migration's "A source change's own configuration is surveyed, and its predecessor becomes
/// its declaration". The keys are read as the project's configuration's are, so a key no stage has a
/// name for still reaches the plan.
///
/// **The value of `follows:` is read, and the value of no other key.** `follows:` becomes the
/// migrated change's `FOLLOWS`, and the stage that writes it carries it from here rather than reading
/// the file again, so the value it writes is the one the plan was checked against. No other value has
/// a reader. `created:` is the case in point: the project records no creation date, because no
/// command depends on one, so its key is reported for the plan to account for and its date crosses
/// only inside the carried file. A field nothing consumes is what project-bootstrap's "A project
/// declares its name, version, and expected toolchain" calls a declaration that "states nothing that
/// can be wrong".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeConfiguration {
    /// Every key the file sets, in the order a reader meets them.
    pub keys: Vec<String>,
    /// `follows:`, as the author wrote it, where the file states one.
    ///
    /// `None` where the file sets no `follows:`, or sets it with no value. Carried as text rather
    /// than resolved: whether it names a change is what `FOLLOWS`'s reader reports, as archived,
    /// active, or unmatched.
    pub follows: Option<String>,
}

/// One archived or active change, as the survey found it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    /// The change's directory name.
    pub name: String,
    /// Whether it is archived.
    pub archived: bool,
    /// The version it records, if it records one.
    ///
    /// `None` means the file is absent or unreadable. A migration must not invent a version
    /// here: `project-migration` requires the plan to record that the source carried none,
    /// which is a different fact from a version that was dropped.
    pub version: Option<String>,
    /// The delta section headings its specifications use, deduplicated and sorted.
    ///
    /// Recorded because one of them — a rename — is carried as a field rather than a change
    /// state: a `renamed-from` paragraph on the requirement headed with the `TO:` heading, which
    /// keeps the identifier its `FROM:` heading resolved to. The plan has to account for each one,
    /// and a section nobody reported is a rename the migrated project silently loses.
    pub delta_sections: Vec<String>,
    /// The change's own configuration file, read.
    ///
    /// `None` means the change holds no `.openspec.yaml`, or holds one that could not be read, as
    /// [`Change::version`] reads its file. That is a different fact from a file that states no
    /// predecessor, which is `Some` with no `follows`, and the plan carries the difference.
    pub configuration: Option<ChangeConfiguration>,
}

/// Everything a source project holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Survey {
    /// The repository root holding the project.
    pub root: PathBuf,
    /// Every entity, sorted by path.
    ///
    /// Sorted rather than in filesystem order, and the reason is that a person reads this. A
    /// coverage failure is diagnosed by diffing two payloads, and filesystem order differs
    /// between machines, so an unsorted payload would report spurious differences and bury the
    /// real one.
    pub entities: Vec<Entity>,
    /// The changes, sorted by name.
    pub changes: Vec<Change>,
    /// The project's configuration.
    pub configuration: Configuration,
}

impl Survey {
    /// Enumerates the OpenSpec project at `path`.
    ///
    /// Writes nothing, reads only to enumerate, and depends on no earlier stage having run.
    pub fn of(path: &Path) -> Result<Self, NotAnOpenSpecProject> {
        let root = absolute(path);
        let project = root.join(DIRECTORY);
        // Either directory is enough to identify a project. Requiring both would refuse a
        // project that has capabilities and no changes yet, which is a legitimate thing for a
        // young project to be.
        if !project.join(SPECS).is_dir() && !project.join(CHANGES).is_dir() {
            return Err(NotAnOpenSpecProject {
                path: path.to_owned(),
                looked_for: project,
            });
        }

        let mut entities = Vec::new();
        let mut changes = Vec::new();

        let configuration = read_configuration(&project.join(CONFIG));

        let changes_directory = project.join(CHANGES);
        let archive_directory = changes_directory.join(ARCHIVE);

        // Every change directory, so the walk below can tell a file inside one from a file loose
        // at the level above it. Collected before the walk rather than tested during it, because
        // the two loops classify by different rules and a shared predicate is what keeps them
        // from disagreeing about where a change begins.
        let change_directories: Vec<PathBuf> = directories(&changes_directory)
            .into_iter()
            .filter(|directory| directory.file_name().is_some_and(|name| name != ARCHIVE))
            .chain(directories(&archive_directory))
            .collect();

        // **Everything the project directory holds, then classified — rather than three known
        // places enumerated.** The requirement is that the enumeration "SHALL be derived from what
        // the source holds rather than from a list of the entity kinds migration expects", and the
        // earlier form violated it: it named `config.yaml`, walked `specs/`, and walked each change
        // directory, so a file sitting loose in `openspec/`, `openspec/changes/`, or
        // `openspec/changes/archive/` was invisible.
        //
        // That is not hypothetical. OpenSpec itself writes `openspec/project.md` and
        // `openspec/AGENTS.md`, and this change's own proposal names the second as an entity it
        // deliberately does not migrate — which still has to reach the plan as a disposition. A
        // review of the pull request found it, and this repository could not: its `openspec/`
        // happens to hold only `config.yaml` at the top level, so
        // `the_survey_enumerates_every_file_the_project_holds` compared two walks that agreed
        // about a project with no loose files.
        for file in walk(&project) {
            let change = change_directories
                .iter()
                .find(|directory| file.starts_with(directory));
            let kind = match change {
                Some(directory) => classify(directory, &file),
                None => classify_outside_a_change(&project, &file),
            };
            entities.push(Entity {
                path: relative(&root, &file),
                kind,
                change: change.and_then(|directory| {
                    directory
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                }),
            });
        }

        for (directory, archived) in directories(&changes_directory)
            .into_iter()
            .filter(|directory| directory.file_name().is_some_and(|name| name != ARCHIVE))
            .map(|directory| (directory, false))
            .chain(
                directories(&archive_directory)
                    .into_iter()
                    .map(|directory| (directory, true)),
            )
        {
            let Some(name) = directory
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
            else {
                continue;
            };

            // The delta sections this change uses. The walk above already enumerated the files,
            // so this reads them rather than pushing entities a second time — two loops both
            // pushing was the shape that let a file be counted twice or not at all.
            let mut sections = BTreeMap::new();
            for file in walk(&directory) {
                if classify(&directory, &file) == Kind::DeltaSpecification {
                    for section in delta_sections(&file) {
                        sections.insert(section, ());
                    }
                }
            }

            changes.push(Change {
                version: std::fs::read_to_string(directory.join(VERSION_FILE))
                    .ok()
                    .map(|text| text.trim().to_owned()),
                delta_sections: sections.into_keys().collect(),
                configuration: read_change_configuration(&directory.join(CHANGE_CONFIG)),
                archived,
                name,
            });
        }

        entities.sort_by(|left, right| left.path.cmp(&right.path));
        changes.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(Self {
            root,
            entities,
            changes,
            configuration,
        })
    }

    /// How many entities of each kind the survey found.
    pub fn counts(&self) -> BTreeMap<&'static str, usize> {
        let mut counts = BTreeMap::new();
        for entity in &self.entities {
            *counts.entry(entity.kind.label()).or_insert(0) += 1;
        }
        counts
    }

    /// The greatest version among the changes that record one.
    ///
    /// What a migrated project's declaration is set to, so that its next archival is required
    /// to advance past its real history rather than past an initial value. Compared
    /// component-wise rather than as text, because `0.10.0` is ordered after `0.9.0` and
    /// string comparison says otherwise — the defect a migration would carry into the version
    /// gate.
    pub fn greatest_version(&self) -> Option<String> {
        self.changes
            .iter()
            .filter_map(|change| change.version.as_deref())
            .filter_map(|version| Some((components(version)?, version)))
            .max()
            .map(|(_, version)| version.to_owned())
    }

    /// The versions recorded by more than one change.
    ///
    /// A migrated project inherits these, and the destination requires recorded versions to be
    /// distinct — with a release-record entry naming the changes that shipped together. So
    /// migration reports them rather than resolving them: inventing distinct versions would
    /// record releases that never existed.
    pub fn shared_versions(&self) -> BTreeMap<String, Vec<String>> {
        let mut by_version: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for change in &self.changes {
            if let Some(version) = &change.version {
                by_version
                    .entry(version.clone())
                    .or_default()
                    .push(change.name.clone());
            }
        }
        by_version.retain(|_, changes| changes.len() > 1);
        by_version
    }

    /// The structured payload the coverage gate consumes.
    pub fn payload(&self) -> serde_json::Value {
        serde_json::json!({
            "root": self.root.to_string_lossy(),
            "counts": self.counts(),
            "greatestVersion": self.greatest_version(),
            "sharedVersions": self.shared_versions(),
            "configuration": { "keys": self.configuration.keys },
            "changes": self
                .changes
                .iter()
                .map(|change| serde_json::json!({
                    "name": change.name,
                    "archived": change.archived,
                    // Present-and-null rather than omitted, because "this change recorded no
                    // version" is a fact the plan has to carry. Omitting the field would make
                    // it indistinguishable from a survey that did not look.
                    "version": change.version,
                    "deltaSections": change.delta_sections,
                    // Present-and-null for a change with no file, for the reason `version` gives,
                    // and `follows` present-and-null for a file that states no predecessor.
                    "configuration": change.configuration.as_ref().map(|configuration| {
                        serde_json::json!({
                            "keys": configuration.keys,
                            "follows": configuration.follows,
                        })
                    }),
                }))
                .collect::<Vec<_>>(),
            "entities": self
                .entities
                .iter()
                .map(|entity| serde_json::json!({
                    "path": entity.path.to_string_lossy(),
                    "kind": entity.kind.label(),
                    "change": entity.change,
                }))
                .collect::<Vec<_>>(),
        })
    }
}

/// What kind of entity `file` is, where it belongs to no change.
///
/// Three cases and a fallback. The project's own configuration, an accepted capability
/// specification, and anything else — which is `Unrecognized` rather than skipped, because a file
/// the enumeration omits is a file the coverage gate cannot report, and it goes missing with every
/// check passing.
///
/// `openspec/project.md` and `openspec/AGENTS.md` land here, and both are files OpenSpec itself
/// writes. Neither has a kind of its own: giving them one would be this module deciding what they
/// become, which belongs to the plan. `Unrecognized` is the honest answer and it reaches a
/// contributor as a line to decide about.
fn classify_outside_a_change(project: &Path, file: &Path) -> Kind {
    let relative = file.strip_prefix(project).unwrap_or(file);
    let mut components = relative.components();
    let first = components
        .next()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default();
    let nested = components.next().is_some();

    match first.as_str() {
        CONFIG if !nested => Kind::Configuration,
        RUNBOOK_ADOPTION if !nested => Kind::AdoptionMarker,
        SPECS if nested => {
            if file.file_name().is_some_and(|name| name == SPEC_FILE) {
                Kind::AcceptedSpecification
            } else {
                Kind::Unrecognized
            }
        }
        _ => Kind::Unrecognized,
    }
}

/// What kind of entity `file` is, within `change`.
fn classify(change: &Path, file: &Path) -> Kind {
    let relative = file.strip_prefix(change).unwrap_or(file);
    let mut components = relative.components();
    let first = components
        .next()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default();
    // Whether anything follows the first component, which distinguishes `evidence/x.md` from a
    // file named `evidence`.
    let nested = components.next().is_some();

    match first.as_str() {
        // Nested rather than direct, so `evidence/transcripts/x.md` is evidence too. A flat
        // match here is exactly the bug that under-counted this repository's evidence by four.
        EVIDENCE if nested => Kind::Evidence,
        DOCUMENTS if nested => Kind::ConvertedDocument,
        SPECS if nested => {
            if file.file_name().is_some_and(|name| name == SPEC_FILE) {
                Kind::DeltaSpecification
            } else {
                Kind::Unrecognized
            }
        }
        VERSION_FILE => Kind::RecordedVersion,
        RUNBOOK_INVENTORY if !nested => Kind::ReleaseInventory,
        CHANGE_CONFIG => Kind::ChangeConfiguration,
        name if CHANGE_DOCUMENTS.contains(&name) => Kind::ChangeDocument,
        _ => Kind::Unrecognized,
    }
}

/// The `## ` section headings a delta specification uses.
///
/// Read as text rather than parsed, because the survey does not interpret what it finds. A
/// heading naming a change state the destination cannot express is a fact about the source;
/// what to do about it belongs to the plan.
fn delta_sections(file: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(|heading| heading.trim().to_owned())
        .collect()
}

/// The keys a configuration file sets, each with the raw text after its colon.
///
/// A key is a line beginning at column zero and holding a colon. Comment lines are skipped,
/// which is what makes a commented-out example an absent field rather than a set one — the
/// distinction between "there was nothing there" and "there was something and it is gone".
///
/// One rule for both configuration files, the project's and each change's, so the survey cannot
/// read a key in one that it would miss in the other.
fn settings(text: &str) -> impl Iterator<Item = (&str, &str)> {
    // A byte-order mark is not part of the first key: read with it, a `follows:` on the first line
    // is a key named something else, and the predecessor is lost without a word.
    text.strip_prefix('\u{feff}')
        .unwrap_or(text)
        .lines()
        .filter(|line| !line.starts_with([' ', '\t', '#']) && !line.trim().is_empty())
        .filter_map(|line| line.split_once(':'))
        .map(|(key, rest)| (key.trim(), rest))
}

/// A setting's value, from the text after its colon, as a YAML reader gives a scalar.
///
/// A quoted value is the text between its quotes, so a `#` inside them is part of it. Otherwise the
/// value ends where a comment begins, at a `#` after whitespace of either kind or at the start, and
/// the rest is trimmed. An empty value is `None`, because YAML reads it as null. Nothing else is
/// interpreted: no escape inside quotes, and no multi-line form, because no stage reads one.
fn value_of(rest: &str) -> Option<String> {
    let trimmed = rest.trim_start();
    let quoted = ['"', '\''].into_iter().find_map(|quote| {
        let inner = trimmed.strip_prefix(quote)?;
        inner.find(quote).map(|end| &inner[..end])
    });
    let value = quoted.unwrap_or_else(|| {
        let comment = trimmed
            .char_indices()
            .find(|&(at, character)| {
                character == '#' && (at == 0 || trimmed[..at].ends_with(char::is_whitespace))
            })
            .map_or(trimmed.len(), |(at, _)| at);
        trimmed[..comment].trim()
    });
    (!value.is_empty()).then(|| value.to_owned())
}

/// The keys the project's configuration file sets.
fn read_configuration(path: &Path) -> Configuration {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Configuration::default();
    };
    Configuration {
        keys: settings(&text).map(|(key, _)| key.to_owned()).collect(),
    }
}

/// A change's own configuration file, or `None` where the change holds none.
///
/// Read with lossy decoding, so a file that exists is read whatever its bytes, and only a file the
/// survey cannot open at all reads as absent. The first `follows:` is the one carried, as the first
/// key is the one a reader meets.
fn read_change_configuration(path: &Path) -> Option<ChangeConfiguration> {
    let bytes = std::fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let mut keys = Vec::new();
    let mut follows = None;
    for (key, rest) in settings(&text) {
        if key == "follows" && follows.is_none() {
            follows = value_of(rest);
        }
        keys.push(key.to_owned());
    }
    Some(ChangeConfiguration { keys, follows })
}

/// Every file beneath `root`, sorted, descending through directories.
fn walk(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(root, &mut found);
    found.sort();
    found
}

/// Accumulates the files beneath `directory` into `found`.
fn collect(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
        } else {
            found.push(path);
        }
    }
}

/// The immediate subdirectories of `directory`, sorted.
fn directories(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    found.sort();
    found
}

/// A version's numeric components, for ordering.
fn components(version: &str) -> Option<Vec<u64>> {
    version
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect()
}

/// `path` relative to `root`, or `path` where it is not beneath it.
fn relative(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_owned()
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
    ///
    /// Hand-rolled to match `project.rs` and `discover.rs`, which do the same for the same
    /// reason: a dependency for one directory in one test module is more surface than the lines
    /// it replaces.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-migrate-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            // Canonicalized because the survey absolutizes, and on macOS the temporary
            // directory is a symlink — an uncanonicalized expectation compares `/var/...`
            // against `/private/var/...` and fails for the wrong reason.
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
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A minimal source project: one capability, one archived change with everything on it.
    fn source(label: &str) -> Scratch {
        let scratch = Scratch::new(label);
        scratch.write("openspec/config.yaml", "schema: spec-driven\n");
        scratch.write(
            "openspec/specs/theme-selection/spec.md",
            "## Purpose\n\nThemes.\n\n### Requirement: A theme is selectable\n",
        );
        let change = "openspec/changes/archive/2026-08-04-add-dark-mode";
        scratch.write(&format!("{change}/proposal.md"), "## Why\n\nDark.\n");
        scratch.write(&format!("{change}/design.md"), "## Context\n\nDark.\n");
        scratch.write(&format!("{change}/tasks.md"), "- [x] 1.1 Do it\n");
        scratch.write(
            &format!("{change}/specs/theme-selection/spec.md"),
            "## ADDED Requirements\n\n### Requirement: A theme is selectable\n",
        );
        scratch.write(&format!("{change}/evidence/measured.md"), "42\n");
        scratch.write(&format!("{change}/VERSION"), "0.2.0\n");
        // The change's own configuration, as OpenSpec writes it and as this repository's archive
        // holds fifteen of: the workflow, the day the change was started, and its predecessor.
        scratch.write(
            &format!("{change}/{CHANGE_CONFIG}"),
            "schema: spec-driven\ncreated: 2026-08-02\nfollows: establish-themes\n",
        );
        // The two files runbook publication adds to a project: an inventory inside each archive, and
        // one marker at the planning root. Enumerated as their own kinds rather than left to
        // `Unrecognized`, because a plan told a release record is unrecognized is a plan asked to
        // decide something the destination already has a place for.
        scratch.write(
            &format!("{change}/{RUNBOOK_INVENTORY}"),
            "{\"format\": \"htmlspec-runbook-inventory/v1\", \"runbooks\": []}\n",
        );
        scratch.write(
            &format!("openspec/{RUNBOOK_ADOPTION}"),
            "{\"format\": \"htmlspec-runbook-adoption/v1\", \"version\": \"0.2.0\"}\n",
        );
        scratch
    }

    /// The refusal names what it looked for, so a contributor who pointed at the wrong
    /// directory does not have to guess which directory or which layout.
    #[test]
    fn a_path_that_is_not_an_openspec_project_is_reported_as_such() {
        let scratch = Scratch::new("not-a-project");
        let error = Survey::of(&scratch.0).expect_err("no project");
        let rendered = error.to_string();
        assert!(
            rendered.contains("is not an OpenSpec project"),
            "the report does not say what is wrong: {rendered}"
        );
        assert!(
            rendered.contains("openspec/specs") && rendered.contains("openspec/changes"),
            "the report does not name what was expected: {rendered}"
        );
    }

    /// A project with capabilities and no changes yet is still a project. Requiring both
    /// directories would refuse a young project for being young.
    #[test]
    fn a_project_with_no_changes_is_still_a_project() {
        let scratch = Scratch::new("specs-only");
        scratch.write("openspec/specs/a/spec.md", "## Purpose\n");
        assert!(Survey::of(&scratch.0).is_ok());
    }

    #[test]
    fn every_entity_kind_is_enumerated_with_its_change() {
        let scratch = source("kinds");
        let survey = Survey::of(&scratch.0).expect("surveyed");
        let counts = survey.counts();

        assert_eq!(counts.get("configuration"), Some(&1));
        assert_eq!(counts.get("accepted-specification"), Some(&1));
        assert_eq!(counts.get("change-document"), Some(&3));
        assert_eq!(counts.get("delta-specification"), Some(&1));
        assert_eq!(counts.get("evidence"), Some(&1));
        assert_eq!(counts.get("recorded-version"), Some(&1));
        assert_eq!(counts.get("release-inventory"), Some(&1));
        assert_eq!(counts.get("adoption-marker"), Some(&1));
        assert_eq!(counts.get("change-configuration"), Some(&1));
        assert_eq!(survey.entities.len(), 11);

        // Every entity inside a change names it, so a plan's per-entity line can say which
        // change a contributor is being asked to decide about.
        for entity in &survey.entities {
            let inside_a_change = entity.path.starts_with("openspec/changes");
            assert_eq!(
                entity.change.is_some(),
                inside_a_change,
                "{} attributes its change wrongly",
                entity.path.display()
            );
        }

        // The marker is a fact about the project's history rather than about one change, so it is
        // enumerated at the planning root and attributed to no change.
        let marker = survey
            .entities
            .iter()
            .find(|entity| entity.kind == Kind::AdoptionMarker)
            .expect("the marker is enumerated");
        assert_eq!(
            marker.path.to_string_lossy(),
            "openspec/runbook-releases.json"
        );
        assert_eq!(marker.change, None);
    }

    /// The survey writes nothing. Asserted by comparing the tree before and after, because a
    /// survey that created a cache or a lock file would satisfy every other test here.
    #[test]
    fn the_survey_writes_nothing() {
        let scratch = source("writes-nothing");
        let before = walk(&scratch.0);
        let _ = Survey::of(&scratch.0).expect("surveyed");
        let after = walk(&scratch.0);
        assert_eq!(before, after, "the survey changed the source tree");
    }

    /// Two runs enumerate the same entities in the same order. Order matters because a
    /// coverage failure is diagnosed by a person diffing two payloads, and filesystem order
    /// differs between machines.
    #[test]
    fn the_survey_is_repeatable_in_the_same_order() {
        let scratch = source("repeatable");
        let first = Survey::of(&scratch.0).expect("surveyed");
        let second = Survey::of(&scratch.0).expect("surveyed");
        assert_eq!(first, second);
        assert_eq!(first.payload(), second.payload());

        let mut sorted = first.entities.clone();
        sorted.sort_by(|left, right| left.path.cmp(&right.path));
        assert_eq!(first.entities, sorted, "the entities are not path-ordered");
    }

    /// An absent version is reported as absent rather than defaulted. The plan has to record
    /// that the source carried none, which is a different fact from a version that was
    /// dropped — and a default would make the two indistinguishable.
    #[test]
    fn a_change_recording_no_version_reports_none() {
        let scratch = Scratch::new("no-version");
        scratch.write(
            "openspec/changes/archive/2026-08-04-undated/proposal.md",
            "## Why\n",
        );
        let survey = Survey::of(&scratch.0).expect("surveyed");

        let change = &survey.changes[0];
        assert_eq!(change.version, None);
        assert_eq!(survey.greatest_version(), None);
        // And the payload carries the field as null rather than omitting it, so a reader
        // cannot mistake "recorded none" for "did not look".
        let payload = survey.payload();
        assert!(payload["changes"][0]["version"].is_null());
        assert!(
            payload["changes"][0].get("version").is_some(),
            "the version field is omitted rather than null: {payload}"
        );
    }

    /// Versions are ordered numerically. `0.10.0` follows `0.9.0`, and a survey comparing
    /// version text would set a migrated declaration to `0.9.0` — leaving the next archival
    /// able to pass without advancing past the project's real history.
    #[test]
    fn the_greatest_version_is_ordered_numerically_rather_than_as_text() {
        let scratch = Scratch::new("version-order");
        for (name, version) in [("a", "0.9.0"), ("b", "0.10.0"), ("c", "0.2.0")] {
            scratch.write(
                &format!("openspec/changes/archive/2026-08-04-{name}/VERSION"),
                version,
            );
        }
        let survey = Survey::of(&scratch.0).expect("surveyed");
        assert_eq!(survey.greatest_version().as_deref(), Some("0.10.0"));
    }

    /// Several changes recording one version is reported rather than resolved. The destination
    /// requires recorded versions to be distinct and provides for this case with a release
    /// record naming the changes that shipped together, so inventing distinct versions here
    /// would record releases nobody installed.
    #[test]
    fn versions_shared_by_several_changes_are_reported() {
        let scratch = Scratch::new("shared-versions");
        for name in ["a", "b", "c"] {
            scratch.write(
                &format!("openspec/changes/archive/2026-08-04-{name}/VERSION"),
                if name == "c" { "0.2.0" } else { "0.1.0" },
            );
        }
        let survey = Survey::of(&scratch.0).expect("surveyed");
        let shared = survey.shared_versions();
        assert_eq!(shared.len(), 1, "{shared:?}");
        assert_eq!(
            shared.get("0.1.0").map(Vec::len),
            Some(2),
            "the shared version does not name both changes: {shared:?}"
        );
    }

    /// Evidence nested below the directory the format names is still evidence. This is the
    /// defect that under-counted this repository's evidence by four: a flat match on
    /// `evidence/*` finds nine files in a change that holds thirteen.
    #[test]
    fn evidence_nested_deeper_than_the_format_describes_is_enumerated() {
        let scratch = Scratch::new("nested-evidence");
        let change = "openspec/changes/archive/2026-08-04-a";
        scratch.write(&format!("{change}/evidence/flat.md"), "1\n");
        scratch.write(&format!("{change}/evidence/transcripts/nested.md"), "2\n");
        scratch.write(
            &format!("{change}/evidence/transcripts/deeper/still.md"),
            "3\n",
        );
        let survey = Survey::of(&scratch.0).expect("surveyed");
        assert_eq!(survey.counts().get("evidence"), Some(&3));
    }

    /// A document already in the destination format is enumerated as its own kind, so the plan
    /// gives it a disposition. Being in the right format is not being conforming, and these
    /// carry a superseded contract version and references that resolve to nothing.
    #[test]
    fn documents_already_in_the_destination_format_are_enumerated() {
        let scratch = Scratch::new("converted");
        scratch.write(
            "openspec/changes/archive/2026-08-04-a/documents/spec.html",
            "<!DOCTYPE html>\n",
        );
        let survey = Survey::of(&scratch.0).expect("surveyed");
        assert_eq!(survey.counts().get("converted-document"), Some(&1));
    }

    /// A file sitting loose at any level of the project is enumerated.
    ///
    /// **The three levels the survey used to skip entirely.** It named `config.yaml`, walked
    /// `specs/`, and walked each change directory — so a file directly in `openspec/`,
    /// `openspec/changes/`, or `openspec/changes/archive/` was invisible. That is not a contrived
    /// arrangement: OpenSpec itself writes `openspec/project.md` and `openspec/AGENTS.md`, and this
    /// change's own proposal names the second as an entity it deliberately does not migrate — which
    /// still has to reach the plan as a disposition.
    ///
    /// Two failures followed from the omission. The coverage gate could not report the files, so
    /// they went missing with every check passing. And an agent who *did* write a plan line for one
    /// had it reported as naming a source the survey never found, which reads as "the plan was
    /// written against a different survey" and points at deleting the correct line.
    ///
    /// A review of the pull request found this, and this repository could not: its `openspec/` holds
    /// only `config.yaml` at the top level, so the two-walk comparison agreed about a project with
    /// no loose files.
    #[test]
    fn a_file_loose_at_any_level_of_the_project_is_enumerated() {
        let scratch = source("loose");
        // One per level the old enumeration skipped, plus the two OpenSpec writes itself.
        let loose = [
            "openspec/project.md",
            "openspec/AGENTS.md",
            "openspec/changes/NOTES.md",
            "openspec/changes/archive/README.md",
        ];
        for path in loose {
            scratch.write(path, "content nobody planned a mapping for\n");
        }
        let survey = Survey::of(&scratch.0).expect("surveyed");

        let enumerated: Vec<String> = survey
            .entities
            .iter()
            .map(|entity| entity.path.to_string_lossy().replace('\\', "/"))
            .collect();
        for path in loose {
            assert!(
                enumerated.iter().any(|found| found == path),
                "{path} is on disk and the survey did not enumerate it: {enumerated:?}"
            );
        }
        // And each reaches the plan as a kind a contributor decides about rather than as nothing.
        assert_eq!(
            survey.counts().get("unrecognized"),
            Some(&4),
            "{:?}",
            survey.counts()
        );
        // The files inside a change still attribute to it, and the loose ones do not.
        for entity in &survey.entities {
            let inside =
                entity.path.starts_with("openspec/changes") && entity.path.components().count() > 4;
            assert_eq!(
                entity.change.is_some(),
                inside,
                "{} attributes its change wrongly",
                entity.path.display()
            );
        }
    }

    /// The survey's entity list equals an independent walk of the project directory.
    ///
    /// The property the coverage gate rests on, asserted against a constructed project rather than
    /// only against this repository. The repository's own version of this test could not catch the
    /// loose-file omission, because the repository has no loose files — so the same property is
    /// checked here against a project that does.
    #[test]
    fn the_enumeration_equals_an_independent_walk() {
        let scratch = source("independent-walk");
        for path in [
            "openspec/project.md",
            "openspec/changes/NOTES.md",
            "openspec/changes/archive/README.md",
            "openspec/specs/theme-selection/diagram.svg",
        ] {
            scratch.write(path, "x\n");
        }
        let survey = Survey::of(&scratch.0).expect("surveyed");

        let mut surveyed: Vec<PathBuf> = survey
            .entities
            .iter()
            .map(|entity| scratch.0.join(&entity.path))
            .collect();
        surveyed.sort();
        let independently = walk(&scratch.0.join(DIRECTORY));
        assert_eq!(
            surveyed, independently,
            "the survey and an independent walk disagree about what the project holds"
        );
    }

    /// A file no mapping was written for is enumerated rather than skipped, because the
    /// coverage check compares a plan against this enumeration — so a file missing here is a
    /// file the check cannot report, and it goes missing with every check passing.
    #[test]
    fn a_file_no_mapping_expects_is_still_enumerated() {
        let scratch = source("unrecognized");
        scratch.write(
            "openspec/changes/archive/2026-08-04-add-dark-mode/NOTES.txt",
            "a file nobody planned for\n",
        );
        scratch.write("openspec/specs/theme-selection/diagram.svg", "<svg/>\n");
        let survey = Survey::of(&scratch.0).expect("surveyed");
        assert_eq!(
            survey.counts().get("unrecognized"),
            Some(&2),
            "{:?}",
            survey.counts()
        );
    }

    /// The delta sections a change uses are recorded, including one the destination's
    /// change-state vocabulary has no value for. The survey reports it as a fact and decides
    /// nothing: what to do about a rename belongs to the plan.
    #[test]
    fn delta_section_headings_are_recorded_including_a_rename() {
        let scratch = Scratch::new("sections");
        scratch.write(
            "openspec/changes/archive/2026-08-04-a/specs/x/spec.md",
            "## RENAMED Requirements\n\n- FROM: `### Requirement: Old`\n- TO: `### Requirement: New`\n\n## MODIFIED Requirements\n",
        );
        let survey = Survey::of(&scratch.0).expect("surveyed");
        assert_eq!(
            survey.changes[0].delta_sections,
            vec![
                "MODIFIED Requirements".to_owned(),
                "RENAMED Requirements".to_owned()
            ]
        );
    }

    /// A commented-out example is an absent field, not a set one. This is what lets a plan
    /// distinguish "there was nothing there" from "there was something and it is gone", and
    /// this repository's own configuration is the case: one live key and three commented
    /// example blocks.
    #[test]
    fn a_commented_configuration_field_is_absent_rather_than_set() {
        let scratch = Scratch::new("config-keys");
        scratch.write(
            "openspec/config.yaml",
            "schema: spec-driven\n\n# context: |\n#   Tech stack: Rust\n#   rules:\n\nversion: 2\n",
        );
        scratch.write("openspec/specs/a/spec.md", "## Purpose\n");
        let survey = Survey::of(&scratch.0).expect("surveyed");
        assert_eq!(
            survey.configuration.keys,
            vec!["schema".to_owned(), "version".to_owned()],
            "a commented field was read as set, or a nested one was read as top-level"
        );
    }

    /// Entity paths are relative to the repository root, so two surveys of one project run
    /// from different directories produce the same payload. The coverage gate compares a plan
    /// against a payload, and absolute paths would make that comparison depend on where the
    /// migration happened to run.
    #[test]
    fn entity_paths_are_relative_to_the_repository_root() {
        let scratch = source("relative");
        let survey = Survey::of(&scratch.0).expect("surveyed");
        for entity in &survey.entities {
            assert!(
                entity.path.is_relative(),
                "{} is absolute",
                entity.path.display()
            );
            assert!(
                entity.path.starts_with(DIRECTORY),
                "{} is not beneath the project directory",
                entity.path.display()
            );
        }
    }

    /// An active change is distinguished from an archived one, and the archive directory
    /// itself is not mistaken for a change.
    #[test]
    fn active_and_archived_changes_are_distinguished() {
        let scratch = source("active");
        scratch.write("openspec/changes/add-light-mode/proposal.md", "## Why\n");
        let survey = Survey::of(&scratch.0).expect("surveyed");

        assert_eq!(survey.changes.len(), 2, "{:?}", survey.changes);
        let active: Vec<&str> = survey
            .changes
            .iter()
            .filter(|change| !change.archived)
            .map(|change| change.name.as_str())
            .collect();
        assert_eq!(active, vec!["add-light-mode"]);
        assert!(
            !survey.changes.iter().any(|change| change.name == ARCHIVE),
            "the archive directory was counted as a change"
        );
    }

    /// The archived change every configuration test below surveys.
    const CONFIGURED: &str = "openspec/changes/archive/2026-09-26-carry-a-runbook";

    /// A source with one archived change whose `.openspec.yaml` holds `contents`, surveyed.
    fn configured(label: &str, contents: &str) -> (Scratch, Survey) {
        let scratch = Scratch::new(label);
        scratch.write(&format!("{CONFIGURED}/proposal.md"), "## Why\n");
        scratch.write(&format!("{CONFIGURED}/{CHANGE_CONFIG}"), contents);
        let survey = Survey::of(&scratch.0).expect("surveyed");
        (scratch, survey)
    }

    /// The one change's configuration, as the survey found it.
    fn configuration_of(survey: &Survey) -> Option<&ChangeConfiguration> {
        survey
            .changes
            .iter()
            .find(|change| change.name == "2026-09-26-carry-a-runbook")
            .expect("the change is surveyed")
            .configuration
            .as_ref()
    }

    /// Every key the file sets is reported, including one no stage has a name for.
    ///
    /// project-migration's configuration requirement: the survey reads the keys the file sets
    /// rather than the keys migration knows the name of, so the plan can account for each one. A
    /// survey reading only `schema`, `created`, and `follows` would drop `skip_specs`, and the
    /// plan would never be asked about it.
    #[test]
    fn a_change_configuration_reports_every_key_it_sets() {
        let (_scratch, survey) = configured(
            "change-keys",
            "schema: spec-driven\ncreated: 2026-09-17\n# follows: an-example\nskip_specs: true\n  \
             nested: not a key\n",
        );
        let configuration = configuration_of(&survey).expect("the change has a configuration");
        assert_eq!(
            configuration.keys,
            vec![
                "schema".to_owned(),
                "created".to_owned(),
                "skip_specs".to_owned()
            ],
            "a set key was missed, or a commented or nested line was read as a key"
        );
    }

    /// `follows:` is carried exactly as the author wrote it, so `FOLLOWS` can hold it unchanged.
    ///
    /// Not normalized and not resolved: whether it names a change is what `FOLLOWS`'s reader
    /// reports, and a survey that lowercased or trimmed it into a name would be inferring.
    #[test]
    fn a_predecessor_is_read_as_the_author_wrote_it() {
        let (_scratch, survey) = configured(
            "predecessor",
            "schema: spec-driven\ncreated: 2026-09-17\nfollows: log-a-runbook-run\n",
        );
        let configuration = configuration_of(&survey).expect("the change has a configuration");
        assert_eq!(configuration.follows.as_deref(), Some("log-a-runbook-run"));

        let (_scratch, survey) = configured("predecessor-as-written", "follows: Log-A-Run_2\n");
        let configuration = configuration_of(&survey).expect("the change has a configuration");
        assert_eq!(
            configuration.follows.as_deref(),
            Some("Log-A-Run_2"),
            "the value was rewritten rather than carried"
        );
    }

    /// `follows:` is read as a YAML reader reads a scalar, in the cases the plain rule misread.
    ///
    /// Found by the code review of PR #52. `migrate.md` has the plan copy the survey's value and
    /// never re-read the file, so a misreading here becomes the migrated change's `FOLLOWS`, and no
    /// later step compares it with the file: a comment after a tab was carried as part of the name,
    /// a quoted value was cut at a ` #` inside its quotes, and a byte-order mark before the first
    /// key hid the key.
    #[test]
    fn a_predecessor_is_read_as_a_yaml_reader_reads_it() {
        for (label, text, expected) in [
            (
                "space-comment",
                "follows: log-a-run # why\n",
                Some("log-a-run"),
            ),
            (
                "tab-comment",
                "follows: log-a-run\t# why\n",
                Some("log-a-run"),
            ),
            (
                "double-quoted",
                "follows: \"log-a-run\"\n",
                Some("log-a-run"),
            ),
            (
                "single-quoted",
                "follows: 'log-a-run' # why\n",
                Some("log-a-run"),
            ),
            ("quoted-hash", "follows: \"a # b\"\n", Some("a # b")),
            (
                "byte-order-mark",
                "\u{feff}follows: log-a-run\n",
                Some("log-a-run"),
            ),
            ("only-a-comment", "follows: # nothing yet\n", None),
            ("empty", "follows:\n", None),
        ] {
            let (_scratch, survey) = configured(&format!("yaml-{label}"), text);
            let configuration = configuration_of(&survey).expect("the change has a configuration");
            assert_eq!(
                configuration.follows.as_deref(),
                expected,
                "{label}: {text:?}"
            );
            assert_eq!(
                configuration.keys,
                vec!["follows".to_owned()],
                "{label}: {text:?}"
            );
        }
    }

    /// `created:` is reported among the keys, and its value appears nowhere in the survey.
    ///
    /// The project records no creation date, because no command depends on one, so a date in the
    /// payload would be a field nothing reads. Its key still reaches the plan, which accounts for
    /// it; the date itself crosses only inside the carried file.
    #[test]
    fn a_creation_date_is_reported_as_a_key_and_its_value_is_not_read() {
        let (_scratch, survey) = configured(
            "created",
            "schema: spec-driven\ncreated: 2026-09-17\nfollows: log-a-runbook-run\n",
        );
        let configuration = configuration_of(&survey).expect("the change has a configuration");
        assert!(
            configuration.keys.iter().any(|key| key == "created"),
            "the creation date's key was not reported: {:?}",
            configuration.keys
        );

        let payload = survey.payload();
        let change = &payload["changes"][0];
        assert!(
            !change.to_string().contains("2026-09-17"),
            "the creation date's value reached the payload: {change}"
        );
        let mut fields: Vec<&str> = change["configuration"]
            .as_object()
            .expect("the configuration is an object")
            .keys()
            .map(String::as_str)
            .collect();
        fields.sort_unstable();
        assert_eq!(
            fields,
            vec!["follows", "keys"],
            "the configuration carries a field beyond its keys and its predecessor"
        );
    }

    /// A change with no `.openspec.yaml` reports none, which is not a file that states nothing.
    #[test]
    fn a_change_with_no_configuration_file_reports_none() {
        let scratch = Scratch::new("no-change-config");
        scratch.write(&format!("{CONFIGURED}/proposal.md"), "## Why\n");
        let survey = Survey::of(&scratch.0).expect("surveyed");
        assert_eq!(configuration_of(&survey), None);
    }

    /// A file that sets no `follows:` states no predecessor, and none is inferred.
    #[test]
    fn a_file_that_states_no_predecessor_states_none() {
        let (_scratch, survey) = configured(
            "no-predecessor",
            "schema: spec-driven\ncreated: 2026-08-04\n",
        );
        let configuration = configuration_of(&survey).expect("the file is there");
        assert_eq!(configuration.follows, None);
        assert_eq!(
            configuration.keys,
            vec!["schema".to_owned(), "created".to_owned()]
        );
    }

    /// `follows:` with nothing after it names no change. YAML reads an empty value as null, so
    /// the file sets the key and states no predecessor.
    #[test]
    fn a_predecessor_key_with_no_value_states_none() {
        let (_scratch, survey) = configured("empty-predecessor", "follows:\nschema: spec-driven\n");
        let configuration = configuration_of(&survey).expect("the file is there");
        assert_eq!(configuration.follows, None);
        assert_eq!(
            configuration.keys,
            vec!["follows".to_owned(), "schema".to_owned()]
        );
    }

    /// A value is the text after the first colon, less a trailing ` #` comment and one pair of
    /// matching quotes, which is how a YAML reader would give it.
    #[test]
    fn a_quoted_value_and_a_trailing_comment_are_read_as_the_value() {
        for (label, line) in [
            ("double", "follows: \"log-a-runbook-run\"\n"),
            ("single", "follows: 'log-a-runbook-run'\n"),
            (
                "comment",
                "follows: log-a-runbook-run # the change before\n",
            ),
            (
                "both",
                "follows:   \"log-a-runbook-run\"   # the change before\n",
            ),
        ] {
            let (_scratch, survey) = configured(&format!("quoted-{label}"), line);
            let configuration = configuration_of(&survey).expect("the file is there");
            assert_eq!(
                configuration.follows.as_deref(),
                Some("log-a-runbook-run"),
                "{line:?} was not read as its value"
            );
        }
        // One pair only, and only a matching one: anything else is the author's text.
        let (_scratch, survey) = configured("quoted-unmatched", "follows: \"log-a-run'\n");
        let configuration = configuration_of(&survey).expect("the file is there");
        assert_eq!(configuration.follows.as_deref(), Some("\"log-a-run'"));
    }

    /// The payload carries `configuration: null` for a change with no file, rather than omitting
    /// it, so "this change has no configuration" cannot be mistaken for a survey that did not look.
    #[test]
    fn the_payload_carries_null_for_a_change_with_no_configuration() {
        let scratch = source("payload-configuration");
        scratch.write("openspec/changes/add-light-mode/proposal.md", "## Why\n");
        let payload = Survey::of(&scratch.0).expect("surveyed").payload();
        let changes = payload["changes"].as_array().expect("an array");

        let unconfigured = changes
            .iter()
            .find(|change| change["name"] == "add-light-mode")
            .expect("the active change is surveyed");
        assert!(
            unconfigured
                .as_object()
                .is_some_and(|change| change.contains_key("configuration")),
            "the configuration field is omitted rather than null: {unconfigured}"
        );
        assert!(unconfigured["configuration"].is_null(), "{unconfigured}");

        let configured = changes
            .iter()
            .find(|change| change["name"] == "2026-08-04-add-dark-mode")
            .expect("the archived change is surveyed");
        assert_eq!(
            configured["configuration"],
            serde_json::json!({
                "keys": ["schema", "created", "follows"],
                "follows": "establish-themes",
            })
        );

        // A file that states no predecessor carries the field as null too, for the same reason.
        let (_scratch, survey) = configured_payload_without_follows();
        assert!(
            survey["changes"][0]["configuration"]
                .as_object()
                .is_some_and(|configuration| configuration.contains_key("follows")),
            "{survey}"
        );
        assert!(survey["changes"][0]["configuration"]["follows"].is_null());
    }

    /// The payload of a source whose one change's file states no predecessor.
    fn configured_payload_without_follows() -> (Scratch, serde_json::Value) {
        let (scratch, survey) = configured("payload-no-follows", "schema: spec-driven\n");
        let payload = survey.payload();
        (scratch, payload)
    }
}
