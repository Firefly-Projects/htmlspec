//! What a runbook's release is, and whether the next one may be published.
//!
//! A runbook that travels to another machine arrives without the archive that would say where it
//! came from, so its release has to be recorded somewhere that travels with it: a `VERSION` and a
//! `CHANGELOG.md` beside its documents. That declaration is only as good as the gate that compares
//! it, and this module is the gate. `runbook-authority` states the obligation and
//! `project-bootstrap` states when it is checked — at source publication, which in this program is
//! archival.
//!
//! # What is fingerprinted
//!
//! Exactly two documents: the runbook's `specification.html` and `tasks.html`, after canonical
//! formatting. **Records are deliberately outside the fingerprint**, and so is changelog prose. A
//! run is history rather than procedure, and requiring a version increment for every run would make
//! the gate fire on the one thing a runbook exists to accumulate. An asset reference updated inside
//! either document *does* move the fingerprint, because the bytes of the canonical document moved —
//! which makes an editorial edit require a release too. That cost is accepted deliberately: a rule
//! that tried to tell an editorial edit from a substantive one would be a judgment, and
//! `artifact-kinds` reserves judgment for an agent.
//!
//! # Two readers, one format
//!
//! The inventory and the adoption marker are read here and by `scripts/check-release`, which cannot
//! call into this crate because it runs where no build has happened. The two share their fixtures
//! under `fixtures/release/` rather than sharing code, and `the_shared_fixtures_say_what_both_readers
//! _read` is the test that keeps them from diverging: a version-ordering rule stated twice is a rule
//! that disagrees with itself the first time one side changes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::terminal::shown;
use crate::version::{ProjectVersion, RunbookVersion};

/// The file a runbook declares its release version in.
///
/// The same name an archived change records its project version in, and the same reason: a version
/// is a fact about a directory as a whole, so it lives in a file beside the documents rather than
/// inside one of them.
pub const VERSION_FILE: &str = "VERSION";

/// The file a runbook records its releases in.
pub const CHANGELOG_FILE: &str = "CHANGELOG.md";

/// The inventory one archive carries, naming every runbook it published.
pub const INVENTORY_FILE: &str = "runbooks.json";

/// The adoption marker at the planning root, naming the first project version subject to
/// inventories.
///
/// Beside the planning root's other whole-project facts rather than inside an archive, because the
/// question it answers — "from which release does a missing inventory mean something is wrong?" —
/// is about the history as a whole and not about one archive.
pub const ADOPTION_FILE: &str = "runbook-releases.json";

/// The format string an inventory carries.
pub const INVENTORY_FORMAT: &str = "htmlspec-runbook-inventory/v1";

/// The format string an adoption marker carries.
pub const ADOPTION_FORMAT: &str = "htmlspec-runbook-adoption/v1";

/// The digest algorithm a fingerprint names.
///
/// Spelled rather than read from [`htmlspec_assets::address`], because a fingerprint is a whole-digest
/// value in a JSON file and the asset store's truncated address form is a different thing with a
/// different use. Sharing the constant would couple two formats that happen to hash the same way.
const DIGEST_ALGORITHM: &str = "sha256";

/// A content fingerprint of one canonical document.
///
/// A distinct type from a `String` because the two documents' digests sit beside each other in an
/// inventory entry, and a swapped pair would be invisible in a `String`-typed field while being a
/// different fact: it would report a changed specification as a changed procedure.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContentDigest(String);

/// Why a string could not be a content digest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DigestFault {
    /// The algorithm is not named, or not one this program writes.
    Algorithm,
    /// The digest is not lowercase hexadecimal of the length this program writes.
    Hex,
}

impl std::fmt::Display for DigestFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Algorithm => "a digest is `sha256:` followed by its hexadecimal value",
            Self::Hex => "a digest's value is 64 lowercase hexadecimal characters",
        })
    }
}

impl ContentDigest {
    /// The digest of these bytes.
    pub fn of(bytes: &[u8]) -> Self {
        Self(format!(
            "{DIGEST_ALGORITHM}:{}",
            htmlspec_assets::sha256_hex(bytes)
        ))
    }

    /// Reads a digest, refusing one this program could not have written.
    ///
    /// Refused rather than accepted as an opaque string, because the comparison this type exists for
    /// is equality and a digest in another form would compare unequal to the same content — which is
    /// the failure that would look like a changed document.
    pub fn parse(text: &str) -> Result<Self, DigestFault> {
        let Some(value) = text.trim().strip_prefix(&format!("{DIGEST_ALGORITHM}:")) else {
            return Err(DigestFault::Algorithm);
        };
        let hex = value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit());
        if !hex || value.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return Err(DigestFault::Hex);
        }
        Ok(Self(text.trim().to_owned()))
    }

    /// The digest as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ContentDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One runbook's released content, as a snapshot or as the working copy.
///
/// The same type for both, and that is the point: [`compare_release`] compares a published snapshot
/// against what is on disk now, and two types would need a conversion that could get a field wrong.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunbookRelease {
    /// The runbook's identity, which is its directory name.
    ///
    /// `runbook-authority` already requires the name to be the directory name, and
    /// `project-bootstrap` makes it the comparison's identity: a renamed runbook begins a new
    /// lineage rather than continuing the old one under a new spelling.
    pub name: String,
    /// The version it declares.
    pub version: RunbookVersion,
    /// The specification's fingerprint.
    pub specification: ContentDigest,
    /// The procedure's fingerprint.
    pub procedure: ContentDigest,
    /// The changelog, as a snapshot.
    ///
    /// Carried in full rather than reduced to its headings, because a deleted or renamed runbook
    /// must not make a past release unverifiable: the headings a release had are a fact about the
    /// release, and the runbook's own file is not there to be asked once it is gone.
    pub changelog: String,
}

impl RunbookRelease {
    /// The changelog headings naming a released version, in the order they appear.
    ///
    /// Read as headings rather than parsed as prose, which is what `scripts/check-release` already
    /// does for the project's own changelog and what `runbook-authority` asks of an author: "one
    /// heading per runbook release". A version appearing in body prose is not a release heading, and
    /// counting it would let an entry satisfy the gate without being one.
    pub fn headings(&self) -> Vec<RunbookVersion> {
        changelog_headings(&self.changelog)
    }

    /// Whether the changelog has a heading for the version this release declares.
    pub fn declares_its_own_release(&self) -> bool {
        self.headings().contains(&self.version)
    }

    /// Whether the canonical content differs from `other`'s.
    ///
    /// Both documents, because a change to either is a change to what a run does or what it owes.
    pub fn content_differs_from(&self, other: &Self) -> bool {
        self.specification != other.specification || self.procedure != other.procedure
    }
}

/// Every version a changelog's headings name, in order.
///
/// A heading is a line beginning `#`, and a version in it is any dotted numeric run — the same rule
/// `scripts/check-release` applies to the project changelog, so the two cannot disagree about what a
/// heading is. Repeats are kept: [`duplicate_headings`] asks about them and an order-sensitive caller
/// should see what the file holds.
pub fn changelog_headings(changelog: &str) -> Vec<RunbookVersion> {
    let mut found = Vec::new();
    for line in changelog.lines() {
        if !line.starts_with('#') {
            continue;
        }
        for candidate in dotted_numbers(line) {
            if let Ok(version) = RunbookVersion::parse(candidate) {
                found.push(version);
            }
        }
    }
    found
}

/// Every dotted numeric run in a line.
///
/// Written out rather than pulled from a regular-expression crate: the workspace already depends on
/// one for the schema, and a second use for a three-line scan is more surface than the scan.
fn dotted_numbers(line: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    let mut start = None;
    for (index, byte) in bytes.iter().enumerate() {
        let part = byte.is_ascii_digit() || *byte == b'.';
        match (start, part) {
            (None, true) => start = Some(index),
            (Some(_), false) => {
                found.push(&line[start.expect("a start")..index]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(start) = start {
        found.push(&line[start..]);
    }
    found
        .into_iter()
        // A run has to be three components to be a version, and `RunbookVersion::parse` says so; a
        // run of `1.2.3.4` is not one, and neither is `...`.
        .filter(|run| run.matches('.').count() == 2)
        .collect()
}

/// The headings a changelog repeats.
///
/// `runbook-authority` requires one heading per release, so a version named twice is a changelog
/// that cannot say which entry is the release. Reported rather than deduplicated, because silently
/// keeping one would hide the ambiguity from the author who has to resolve it.
pub fn duplicate_headings(changelog: &str) -> Vec<RunbookVersion> {
    let mut seen: BTreeMap<RunbookVersion, usize> = BTreeMap::new();
    for heading in changelog_headings(changelog) {
        *seen.entry(heading).or_insert(0) += 1;
    }
    seen.into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(version, _)| version)
        .collect()
}

/// One runbook's entry in an archive's inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryEntry {
    /// The released runbook.
    pub release: RunbookRelease,
}

/// The runbooks one archive published, as a versioned typed inventory.
///
/// **An empty inventory is explicit rather than absent.** `project-bootstrap` requires that "each
/// later archive SHALL explicitly record its runbook inventory, including an empty inventory, so a
/// missing inventory cannot masquerade as old history" — an absent file and a file saying "there were
/// no runbooks" are different facts, and only the second one is a record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inventory {
    entries: Vec<InventoryEntry>,
}

impl Inventory {
    /// An inventory holding these releases.
    ///
    /// Refuses a repeated name, because two entries for one runbook make the predecessor question
    /// unanswerable: which of the two is the release this archive published?
    pub fn of(releases: Vec<RunbookRelease>) -> Result<Self, InventoryFaults> {
        let mut faults = Vec::new();
        for (index, release) in releases.iter().enumerate() {
            if releases[..index]
                .iter()
                .any(|earlier| earlier.name == release.name)
            {
                faults.push(InventoryFault::DuplicateName {
                    name: release.name.clone(),
                });
            }
        }
        if faults.is_empty() {
            Ok(Self {
                entries: releases
                    .into_iter()
                    .map(|release| InventoryEntry { release })
                    .collect(),
            })
        } else {
            Err(InventoryFaults(faults))
        }
    }

    /// The releases this inventory records, in the order they were written.
    pub fn releases(&self) -> Vec<&RunbookRelease> {
        self.entries.iter().map(|entry| &entry.release).collect()
    }

    /// How many runbooks this inventory records.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether this inventory records no runbook.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Reads an inventory, refusing one this program could not have written.
    pub fn parse(text: &str) -> Result<Self, InventoryFaults> {
        let value: serde_json::Value = serde_json::from_str(text).map_err(|error| {
            InventoryFaults(vec![InventoryFault::Malformed {
                problem: format!("it is not JSON: {error}"),
            }])
        })?;
        let Some(object) = value.as_object() else {
            return Err(InventoryFaults(vec![InventoryFault::Malformed {
                problem: String::from("an inventory is one JSON object"),
            }]));
        };
        let format = object.get("format").and_then(|value| value.as_str());
        if format != Some(INVENTORY_FORMAT) {
            return Err(InventoryFaults(vec![InventoryFault::UnknownFormat {
                found: format.map(str::to_owned),
                wanted: INVENTORY_FORMAT,
            }]));
        }
        for field in object.keys() {
            if !["format", "runbooks"].contains(&field.as_str()) {
                return Err(InventoryFaults(vec![InventoryFault::Malformed {
                    problem: format!(
                        "`{field}` is not a field an inventory has, and an unknown field is \
                         refused rather than ignored: it is a fact this program would not read"
                    ),
                }]));
            }
        }
        let entries = match object.get("runbooks") {
            None => {
                return Err(InventoryFaults(vec![InventoryFault::Malformed {
                    problem: String::from(
                        "an inventory states its runbooks, and an inventory of none states an \
                         empty list rather than omitting it",
                    ),
                }]))
            }
            Some(value) => match value.as_array() {
                Some(entries) => entries,
                None => {
                    return Err(InventoryFaults(vec![InventoryFault::Malformed {
                        problem: String::from("`runbooks` is a list"),
                    }]))
                }
            },
        };

        let mut releases = Vec::with_capacity(entries.len());
        let mut faults = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            match release_from(index, entry) {
                Ok(release) => releases.push(release),
                Err(found) => faults.extend(found.0),
            }
        }
        if !faults.is_empty() {
            return Err(InventoryFaults(faults));
        }
        Self::of(releases)
    }

    /// This inventory as the file an archive carries.
    ///
    /// Pretty-printed and key-ordered, so two archives holding the same releases hold the same
    /// bytes: an inventory is committed, and a diff that moves because a map iterated differently
    /// is a diff nobody can read.
    pub fn render(&self) -> String {
        let entries: Vec<serde_json::Value> = self
            .entries
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "name": entry.release.name,
                    "version": entry.release.version.to_string(),
                    "specification": entry.release.specification.as_str(),
                    "procedure": entry.release.procedure.as_str(),
                    "changelog": entry.release.changelog,
                })
            })
            .collect();
        let document = serde_json::json!({
            "format": INVENTORY_FORMAT,
            "runbooks": entries,
        });
        // Serialization of a value this function built cannot fail: every field is a string, an
        // array, or an object. Reported rather than unwrapped because an empty file is a visible
        // wrong answer where a panic in archival is not.
        serde_json::to_string_pretty(&document)
            .map(|rendered| format!("{rendered}\n"))
            .unwrap_or_else(|_| {
                format!("{{\n  \"format\": \"{INVENTORY_FORMAT}\",\n  \"runbooks\": []\n}}\n")
            })
    }
}

/// One inventory entry, read.
fn release_from(
    index: usize,
    value: &serde_json::Value,
) -> Result<RunbookRelease, InventoryFaults> {
    let at =
        |problem: String| InventoryFaults(vec![InventoryFault::MalformedEntry { index, problem }]);
    let Some(object) = value.as_object() else {
        return Err(at(String::from("an entry is one JSON object")));
    };
    for field in object.keys() {
        if !["name", "version", "specification", "procedure", "changelog"].contains(&field.as_str())
        {
            return Err(at(format!(
                "`{field}` is not a field an entry has, and an unknown field is refused rather \
                 than ignored"
            )));
        }
    }
    let name = object
        .get("name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| at(String::from("an entry names its runbook")))?;
    let version = object
        .get("version")
        .and_then(|value| value.as_str())
        .ok_or_else(|| at(format!("{name} states a version")))?;
    let version = RunbookVersion::parse(version)
        .map_err(|error| at(format!("{name}'s version is not one: {error}")))?;
    let specification = digest_of(object, "specification", name).map_err(at)?;
    let procedure = digest_of(object, "procedure", name).map_err(at)?;
    let changelog = object
        .get("changelog")
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            at(format!(
                "{name} carries its changelog as a snapshot, so a past release stays verifiable \
                 after the runbook is deleted or renamed"
            ))
        })?;
    Ok(RunbookRelease {
        name: name.to_owned(),
        version,
        specification,
        procedure,
        changelog: changelog.to_owned(),
    })
}

/// One entry's digest field, read.
fn digest_of(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    name: &str,
) -> Result<ContentDigest, String> {
    let text = object
        .get(field)
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("{name} fingerprints its {field} document"))?;
    ContentDigest::parse(text)
        .map_err(|fault| format!("{name}'s {field} fingerprint is not one: {fault}"))
}

/// Why an inventory could not be read or built.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InventoryFault {
    /// The file is not an inventory this program writes.
    Malformed {
        /// What is wrong with it.
        problem: String,
    },
    /// One entry is not readable.
    MalformedEntry {
        /// Which entry, by position.
        index: usize,
        /// What is wrong with it.
        problem: String,
    },
    /// The format is not one this program reads.
    UnknownFormat {
        /// What it says, if it says anything.
        found: Option<String>,
        /// What this program writes.
        wanted: &'static str,
    },
    /// Two entries name one runbook.
    DuplicateName {
        /// The name both entries carry.
        name: String,
    },
}

impl std::fmt::Display for InventoryFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed { problem } => {
                write!(f, "the runbook inventory is not readable: {problem}")
            }
            Self::MalformedEntry { index, problem } => {
                write!(
                    f,
                    "runbook inventory entry {index} is not readable: {problem}"
                )
            }
            Self::UnknownFormat { found, wanted } => match found {
                Some(found) => write!(
                    f,
                    "the runbook inventory declares the format {found:?}, and this program reads \
                     {wanted:?}"
                ),
                None => write!(
                    f,
                    "the runbook inventory declares no format, and this program reads {wanted:?}"
                ),
            },
            Self::DuplicateName { name } => write!(
                f,
                "the runbook inventory records {name:?} twice, so which of the two this archive \
                 published is not a question the file can answer"
            ),
        }
    }
}

/// Every fault an inventory carries. Nonempty wherever it is returned as an error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryFaults(Vec<InventoryFault>);

impl std::fmt::Display for InventoryFaults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, fault) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str("\n")?;
            }
            write!(f, "{fault}")?;
        }
        Ok(())
    }
}

impl std::error::Error for InventoryFaults {}

/// The first project version subject to runbook inventories.
///
/// **Never advanced on a later release**, and that is what makes it a marker rather than a cursor:
/// it answers "from where does a missing inventory mean something is wrong", and the answer is fixed
/// at adoption. Deleting the first inventory would otherwise turn recorded history into a legacy
/// exemption, which is the failure `project-bootstrap` names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Adoption {
    /// The project version adoption happened at.
    pub version: ProjectVersion,
}

/// Why an adoption marker could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdoptionFault {
    /// The file is not a marker this program writes.
    Malformed {
        /// What is wrong with it.
        problem: String,
    },
    /// The format is not one this program reads.
    UnknownFormat {
        /// What it says, if it says anything.
        found: Option<String>,
        /// What this program writes.
        wanted: &'static str,
    },
}

impl std::fmt::Display for AdoptionFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed { problem } => {
                write!(f, "the runbook adoption marker is not readable: {problem}")
            }
            Self::UnknownFormat { found, wanted } => match found {
                Some(found) => write!(
                    f,
                    "the runbook adoption marker declares the format {found:?}, and this program \
                     reads {wanted:?}"
                ),
                None => write!(
                    f,
                    "the runbook adoption marker declares no format, and this program reads \
                     {wanted:?}"
                ),
            },
        }
    }
}

impl Adoption {
    /// Reads a marker.
    pub fn parse(text: &str) -> Result<Self, AdoptionFault> {
        let malformed = |problem: String| AdoptionFault::Malformed { problem };
        let value: serde_json::Value = serde_json::from_str(text)
            .map_err(|error| malformed(format!("it is not JSON: {error}")))?;
        let object = value
            .as_object()
            .ok_or_else(|| malformed(String::from("a marker is one JSON object")))?;
        let format = object.get("format").and_then(|value| value.as_str());
        if format != Some(ADOPTION_FORMAT) {
            return Err(AdoptionFault::UnknownFormat {
                found: format.map(str::to_owned),
                wanted: ADOPTION_FORMAT,
            });
        }
        for field in object.keys() {
            if !["format", "version"].contains(&field.as_str()) {
                return Err(malformed(format!(
                    "`{field}` is not a field a marker has, and an unknown field is refused \
                     rather than ignored"
                )));
            }
        }
        let version = object
            .get("version")
            .and_then(|value| value.as_str())
            .ok_or_else(|| malformed(String::from("a marker names the version adopted at")))?;
        let version = ProjectVersion::parse(version)
            .map_err(|error| malformed(format!("its version is not one: {error}")))?;
        Ok(Self { version })
    }

    /// This marker as the file a planning root carries.
    pub fn render(self) -> String {
        let document = serde_json::json!({
            "format": ADOPTION_FORMAT,
            "version": self.version.to_string(),
        });
        serde_json::to_string_pretty(&document)
            .map(|rendered| format!("{rendered}\n"))
            .unwrap_or_default()
    }
}

/// One published snapshot, with the archive that published it and the project version it shipped in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    /// The project version whose archive carries this snapshot.
    pub project: ProjectVersion,
    /// The archive directory it was read from, for a report.
    pub archive: String,
    /// What was published.
    pub release: RunbookRelease,
}

/// Why a release cannot be published.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseFault {
    /// A first release at or below the version nothing may be released at.
    UnreleasedVersion {
        /// The runbook.
        name: String,
        /// The version it declares.
        version: RunbookVersion,
    },
    /// Changed canonical content at a version that did not advance.
    ChangedWithoutABump {
        /// The runbook.
        name: String,
        /// The version both releases declare.
        version: RunbookVersion,
        /// The archive that published that version.
        published_in: String,
    },
    /// A version ordered before the one already published.
    Downgrade {
        /// The runbook.
        name: String,
        /// The version it declares now.
        current: RunbookVersion,
        /// The version already published.
        published: RunbookVersion,
        /// The archive that published it.
        published_in: String,
    },
    /// No changelog heading for the declared version.
    NoHeading {
        /// The runbook.
        name: String,
        /// The version with no heading.
        version: RunbookVersion,
    },
    /// A changelog heading naming one version twice.
    RepeatedHeading {
        /// The runbook.
        name: String,
        /// The version named twice.
        version: RunbookVersion,
    },
    /// An archive at or after adoption carries no inventory.
    MissingInventory {
        /// The archive directory.
        archive: String,
        /// The project version it records.
        project: ProjectVersion,
        /// The version adoption happened at.
        adopted_at: ProjectVersion,
    },
    /// An inventory exists with no adoption marker to say when the obligation began.
    InventoryWithoutAdoption {
        /// The archive carrying the inventory.
        archive: String,
    },
    /// The adoption marker names a project version no archive records.
    AdoptionNamesNoArchive {
        /// The version it names.
        version: ProjectVersion,
    },
    /// An archive's inventory could not be read.
    ///
    /// Reported rather than treated as an absent predecessor: a defective snapshot is a fact about
    /// the history, and skipping it would make the next release look like a first one.
    UnreadableInventory {
        /// The archive directory.
        archive: String,
        /// What is wrong with it.
        problem: String,
    },
    /// A runbook the project holds could not be read as a release.
    ///
    /// Its own variant rather than an unreadable inventory, because the fault is in the runbook
    /// being published and the remedy is in its directory: a missing `VERSION`, a missing
    /// changelog, or a document that will not read or format canonically.
    UnreadableRunbook {
        /// The runbook.
        name: String,
        /// Its directory, for a report.
        directory: String,
        /// What is wrong with it.
        problem: String,
    },
    /// The adoption marker exists and could not be read or parsed.
    ///
    /// Its own variant because the marker is not an archive's inventory: it lives in the planning
    /// root, and an author told an archive's inventory was unreadable would look in the wrong place.
    UnreadableAdoption {
        /// The marker's path.
        marker: String,
        /// What is wrong with it.
        problem: String,
    },
}

impl std::fmt::Display for ReleaseFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // A runbook's name and an archive's are their directories' names, so either can hold
            // an escape sequence a terminal would act on. Each reaches the report shown.
            Self::UnreleasedVersion { name, version } => {
                let name = shown(name);
                write!(
                    f,
                    "the runbook {name} declares {version}, which is not a release: a runbook's \
                     first published version is above 0.0.0, because 0.0.0 is the version a \
                     runbook carries before it has been released at all. Raise {} and give the new \
                     version a changelog heading",
                    VERSION_FILE
                )
            }
            Self::ChangedWithoutABump {
                name,
                version,
                published_in,
            } => {
                let (name, published_in) = (shown(name), shown(published_in));
                write!(
                    f,
                    "the runbook {name} declares {version}, which {published_in} already published \
                     with different canonical content. A version identifies one released \
                     procedure, so a changed one needs a greater version — and choosing the \
                     increment is the author's, not this program's. Asset references `htmlspec \
                     format --upgrade-assets` moved, a fallback stylesheet `htmlspec format \
                     --fallback` added or replaced, and a reference plain `htmlspec format` filled \
                     because the store lacked the bytes it named, are changes too, and the \
                     likeliest ones where nobody edited the runbook"
                )
            }
            Self::Downgrade {
                name,
                current,
                published,
                published_in,
            } => {
                let (name, published_in) = (shown(name), shown(published_in));
                write!(
                    f,
                    "the runbook {name} declares {current}, which is ordered before the \
                     {published} that {published_in} published. Publishing it would move the \
                     runbook backwards, so a reader on another machine could not tell which of the \
                     two a copy came from"
                )
            }
            Self::NoHeading { name, version } => {
                let name = shown(name);
                write!(
                    f,
                    "the runbook {name} declares {version} and its {CHANGELOG_FILE} has no heading \
                     naming it. A carried runbook identifies its release by the pair, so a version \
                     nothing headings is one a reader cannot find"
                )
            }
            Self::RepeatedHeading { name, version } => {
                let name = shown(name);
                write!(
                    f,
                    "the runbook {name}'s {CHANGELOG_FILE} heads {version} twice, so it cannot say \
                     which entry is the release. One heading per release"
                )
            }
            Self::MissingInventory {
                archive,
                project,
                adopted_at,
            } => write!(
                f,
                "the archive {archive} records project version {project}, which is at or after the \
                 {adopted_at} adoption of runbook publication, and it carries no {INVENTORY_FILE}. \
                 A missing inventory after adoption is a missing record rather than old history"
            ),
            Self::InventoryWithoutAdoption { archive } => write!(
                f,
                "the archive {archive} carries an {INVENTORY_FILE} and the planning root carries no \
                 {ADOPTION_FILE}. The marker is what says from which release a missing inventory \
                 means something is wrong, so an inventory without it leaves the history with no \
                 beginning"
            ),
            Self::AdoptionNamesNoArchive { version } => write!(
                f,
                "the {ADOPTION_FILE} marker names project version {version} and no archive records \
                 it, so the first published inventory cannot be found. Either the marker or the \
                 archive is wrong about when adoption happened"
            ),
            Self::UnreadableInventory { archive, problem } => write!(
                f,
                "the archive {archive} carries an unreadable {INVENTORY_FILE}: {problem}. It is \
                 reported rather than treated as an absent predecessor, because a defective \
                 snapshot skipped would make the next release look like a first one"
            ),
            Self::UnreadableRunbook {
                name,
                directory,
                problem,
            } => {
                let (name, directory) = (shown(name), shown(directory));
                write!(
                    f,
                    "the runbook {name} at {directory} cannot be read as a release: {problem}. A \
                     runbook that cannot be read is refused rather than left out, because an \
                     inventory that silently omitted it would publish a history missing a runbook \
                     the project holds"
                )
            }
            Self::UnreadableAdoption { marker, problem } => write!(
                f,
                "the {ADOPTION_FILE} marker at {marker} cannot be read: {problem}. The marker says \
                 from which release a missing inventory is a fault, so a history with an unreadable \
                 one has no beginning this operation can trust"
            ),
        }
    }
}

/// Every reason a release cannot be published. Nonempty wherever it is returned as an error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseFaults(Vec<ReleaseFault>);

impl ReleaseFaults {
    /// One fault, as a refusal.
    pub fn one(fault: ReleaseFault) -> Self {
        Self(vec![fault])
    }

    /// The faults found.
    pub fn faults(&self) -> &[ReleaseFault] {
        &self.0
    }
}

impl std::fmt::Display for ReleaseFaults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, fault) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str("\n")?;
            }
            write!(f, "{fault}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ReleaseFaults {}

/// The last release published for one runbook name, or the fact that there is none.
///
/// A sum type rather than an `Option`, because "no predecessor" and "a predecessor this could not
/// read" lead to different refusals: the first makes this a first release and the second stops the
/// comparison until somebody fixes the record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviousRelease {
    /// Nothing has been published under this name.
    None,
    /// The most recent snapshot, ordered by recorded project version.
    Snapshot(Snapshot),
}

/// Evidence that one runbook's release may be published.
///
/// Carries the compared values rather than a boolean, so a caller staging an inventory has the entry
/// it needs and cannot accidentally stage something it did not compare.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishableRelease {
    /// What may be published.
    pub release: RunbookRelease,
}

/// Whether `current` may be published after `previous`.
///
/// Total over its inputs and pure: no filesystem, no clock, and every refusal is a named fault. The
/// rules are `project-bootstrap`'s, and the ordering of the checks matters only for which fault an
/// author sees first — a malformed version is reported before a comparison that would need it.
///
/// **A repeated snapshot of unchanged content at one version is not a duplicate release.** That case
/// is legal rather than a fault, and it is the reason this function compares content before it
/// compares versions: an archive published twice for an unrelated reason must not be refused for
/// having said the same thing about an unchanged runbook.
pub fn compare_release(
    previous: PreviousRelease,
    current: RunbookRelease,
) -> Result<PublishableRelease, ReleaseFaults> {
    let mut faults = Vec::new();

    // The changelog is checked before the comparison, because a heading is required of every
    // release and an author told only "your version did not advance" would fix the version and
    // re-run into a second refusal.
    for repeated in duplicate_headings(&current.changelog) {
        faults.push(ReleaseFault::RepeatedHeading {
            name: current.name.clone(),
            version: repeated,
        });
    }
    if !current.declares_its_own_release() {
        faults.push(ReleaseFault::NoHeading {
            name: current.name.clone(),
            version: current.version,
        });
    }

    match previous {
        PreviousRelease::None => {
            if !current.version.is_released() {
                faults.push(ReleaseFault::UnreleasedVersion {
                    name: current.name.clone(),
                    version: current.version,
                });
            }
        }
        PreviousRelease::Snapshot(published) => {
            let changed = current.content_differs_from(&published.release);
            match current.version.cmp(&published.release.version) {
                std::cmp::Ordering::Less => faults.push(ReleaseFault::Downgrade {
                    name: current.name.clone(),
                    current: current.version,
                    published: published.release.version,
                    published_in: published.archive.clone(),
                }),
                // Same version: legal for unchanged content, which is one release snapshotted
                // twice, and a refusal for changed content, which is one version identifying two
                // procedures.
                std::cmp::Ordering::Equal if changed => {
                    faults.push(ReleaseFault::ChangedWithoutABump {
                        name: current.name.clone(),
                        version: current.version,
                        published_in: published.archive.clone(),
                    })
                }
                std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {}
            }
        }
    }

    if faults.is_empty() {
        Ok(PublishableRelease { release: current })
    } else {
        Err(ReleaseFaults(faults))
    }
}

/// What a runbook's version needs before the next archival, once its canonical content differs
/// from what was last published under its name, or would.
///
/// The version half of [`compare_release`], and its heading check, for a caller that knows the
/// content changed but has no release to compare: `htmlspec format --upgrade-assets` in check mode
/// writes nothing to fingerprint. Each variant carries what the caller reports, so it cannot name a
/// version it did not compare.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VersionAfterAChange {
    /// An archive released a version at or above this one, so this one must exceed it.
    MustExceed {
        /// The version last published.
        published: RunbookVersion,
        /// The archive that published it.
        published_in: String,
    },
    /// Nothing is published and the version is 0.0.0, which is not a release.
    NeedsAFirstRelease,
    /// The version is above anything published, but the changelog has no heading for it.
    NeedsAHeading(RunbookVersion),
    /// Nothing is published, and the next archival releases the runbook at this version, which its
    /// changelog heads, as its first release.
    FirstRelease(RunbookVersion),
    /// An earlier version is published, and the next archival releases the runbook at this greater
    /// one, which its changelog heads.
    Advances(RunbookVersion),
}

/// What `version` needs after a change, given what was last published under the runbook's name and
/// the runbook's changelog.
///
/// Total and pure, and agrees with [`compare_release`] for changed content whose changelog heads no
/// version twice: it answers [`VersionAfterAChange::FirstRelease`] or
/// [`VersionAfterAChange::Advances`] exactly when that comparison raises no fault.
pub fn version_after_a_change(
    previous: &PreviousRelease,
    version: RunbookVersion,
    changelog: &str,
) -> VersionAfterAChange {
    match previous {
        PreviousRelease::Snapshot(published) if version <= published.release.version => {
            VersionAfterAChange::MustExceed {
                published: published.release.version,
                published_in: published.archive.clone(),
            }
        }
        PreviousRelease::None if !version.is_released() => VersionAfterAChange::NeedsAFirstRelease,
        _ if !changelog_headings(changelog).contains(&version) => {
            VersionAfterAChange::NeedsAHeading(version)
        }
        PreviousRelease::None => VersionAfterAChange::FirstRelease(version),
        PreviousRelease::Snapshot(_) => VersionAfterAChange::Advances(version),
    }
}

/// One runbook's released content, read from where it lives.
///
/// Fingerprints the canonical bytes, which is what makes the comparison about the document a reader
/// opens rather than about how it happened to be formatted: `htmlspec format` is part of the suite,
/// so a whitespace-only difference between two releases is a difference nobody chose.
///
/// An absent document is a fault rather than an empty fingerprint. A runbook is two documents, and
/// one that cannot be read cannot be compared — fingerprinting an empty string would report it as
/// changed, which is a different claim and a wrong one where the file is merely unreadable.
pub fn release_of(directory: &Path, name: &str) -> Result<RunbookRelease, ReleaseFaults> {
    let unreadable = |problem: String| {
        ReleaseFaults::one(ReleaseFault::UnreadableRunbook {
            name: name.to_owned(),
            directory: directory.display().to_string(),
            problem,
        })
    };
    let version = read_version(directory).map_err(unreadable)?;
    let changelog = std::fs::read_to_string(directory.join(CHANGELOG_FILE))
        .map_err(|error| unreadable(format!("its {CHANGELOG_FILE} could not be read: {error}")))?;
    let specification = fingerprint(directory, crate::runs::CONTRACT).map_err(unreadable)?;
    let procedure = fingerprint(directory, crate::runs::PROCEDURE).map_err(unreadable)?;
    Ok(RunbookRelease {
        name: name.to_owned(),
        version,
        specification,
        procedure,
        changelog,
    })
}

/// The canonical fingerprint of one document in a runbook's directory.
fn fingerprint(directory: &Path, document: &str) -> Result<ContentDigest, String> {
    let path = directory.join(document);
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("its {document} could not be read: {error}"))?;
    // Formatted before it is hashed, so two copies of one document that differ only in formatting
    // fingerprint alike. The formatter refuses only ambiguity, and a document it refuses is one the
    // validator already reported — so the refusal is carried rather than smoothed over.
    let canonical = htmlspec_format::format(&source)
        .map_err(|error| format!("its {document} cannot be formatted canonically: {error}"))?;
    Ok(ContentDigest::of(canonical.as_bytes()))
}

/// The release version a runbook declares.
///
/// Shared by publication, by preflight, and by starting a run, because all three refuse a runbook
/// that cannot say which release it is and none of them may disagree about where the answer lives.
pub fn read_version(directory: &Path) -> Result<RunbookVersion, String> {
    let path = directory.join(VERSION_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "it has no {VERSION_FILE}. A runbook carried to another machine identifies its \
                 release by that file and its {CHANGELOG_FILE}, because the archive that would \
                 otherwise say where it came from did not travel with it. Write the version this \
                 runbook is released at — three dot-separated numbers above 0.0.0 — and give it a \
                 heading in {CHANGELOG_FILE}"
            ))
        }
        Err(error) => return Err(format!("its {VERSION_FILE} could not be read: {error}")),
    };
    RunbookVersion::parse(&text).map_err(|error| {
        format!(
            "its {VERSION_FILE} does not hold a version: {error}. A runbook's release version is \
             three nonnegative dot-separated integers"
        )
    })
}

/// The published history of one planning root, ordered by recorded project version.
///
/// **Ordered by version rather than by archive directory name**, which `project-bootstrap` requires
/// and which is what makes a renamed archive directory leave the predecessor unchanged: a name is a
/// label a contributor may fix, and a history that moved when somebody tidied a directory name would
/// be a history nobody can rely on.
///
/// Fails closed. An archive whose version cannot be read and an inventory that cannot be parsed are
/// both refusals rather than omissions, because either one silently skipped would change which
/// snapshot this returns as the predecessor.
pub fn history(
    archive: &Path,
    adoption: &Path,
) -> Result<(Option<Adoption>, Vec<Snapshot>), ReleaseFaults> {
    let marker = match std::fs::read_to_string(adoption) {
        Ok(text) => Some(Adoption::parse(&text).map_err(|fault| {
            ReleaseFaults::one(ReleaseFault::UnreadableAdoption {
                marker: adoption.display().to_string(),
                problem: fault.to_string(),
            })
        })?),
        // An absent marker is the ordinary state before adoption, and every archive holding an
        // inventory without one is reported below rather than here: the fault is in the pair, and
        // naming the archive that carries the inventory is the report an author can act on.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(ReleaseFaults::one(ReleaseFault::UnreadableAdoption {
                marker: adoption.display().to_string(),
                problem: format!("it could not be read: {error}"),
            }))
        }
    };

    let mut faults = Vec::new();
    let mut snapshots: Vec<Snapshot> = Vec::new();
    let Ok(entries) = std::fs::read_dir(archive) else {
        // No archive directory is a project that has never archived, which is the same silence
        // `last_archived_version` keeps: nothing published, nothing to compare against.
        return Ok((marker, snapshots));
    };

    for entry in entries.flatten() {
        let directory = entry.path();
        if !directory.is_dir() {
            continue;
        }
        let name = directory
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let recorded = directory.join(crate::project::RECORDED_VERSION);
        let project = match std::fs::read_to_string(&recorded) {
            Ok(text) => match ProjectVersion::parse(&text) {
                Ok(version) => version,
                // **Skipped rather than refused, and this is a decision the design did not make.**
                // An archive whose recorded version will not parse cannot be placed in the ordering,
                // so it contributes no snapshot. Refusing every later archival over it was the
                // alternative, and it is worse: the fault is in a file belonging to some *other*
                // change, and a gate blocking the project on that has no route back through this
                // operation. `last_archived_version` already draws the line here for the project
                // version gate, for the stated reason that reporting a malformed `VERSION` belongs to
                // `scripts/check-release` — which names the change carrying it, and which asks about
                // every archive's version independently of this reader.
                Err(_) => continue,
            },
            // An archive predating the recorded-version convention is old history, and
            // `project-bootstrap` leaves archives predating adoption valid "without invented
            // history". Skipped rather than refused: it cannot be placed in the ordering, and
            // refusing every archival over an archive from before the convention would wedge the
            // project.
            Err(_) => continue,
        };

        let inventory_path = directory.join(INVENTORY_FILE);
        let text = match std::fs::read_to_string(&inventory_path) {
            Ok(text) => Some(text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                faults.push(ReleaseFault::UnreadableInventory {
                    archive: name.clone(),
                    problem: format!("it could not be read: {error}"),
                });
                continue;
            }
        };
        match (marker, text.as_ref()) {
            (None, Some(_)) => faults.push(ReleaseFault::InventoryWithoutAdoption {
                archive: name.clone(),
            }),
            (Some(adopted), None) if project >= adopted.version => {
                faults.push(ReleaseFault::MissingInventory {
                    archive: name.clone(),
                    project,
                    adopted_at: adopted.version,
                })
            }
            // A marker and an inventory together are the ordinary state after adoption, and no
            // inventory anywhere is the ordinary state before it.
            (Some(_), Some(_)) | (Some(_), None) | (None, None) => {}
        }
        let Some(text) = text else { continue };
        match Inventory::parse(&text) {
            Ok(inventory) => {
                snapshots.extend(inventory.releases().into_iter().map(|release| Snapshot {
                    project,
                    archive: name.clone(),
                    release: release.clone(),
                }))
            }
            Err(found) => faults.push(ReleaseFault::UnreadableInventory {
                archive: name.clone(),
                problem: found.to_string(),
            }),
        }
    }

    // Ordered by the recorded project version, with the directory name breaking a tie so the
    // ordering is total: two archives recording one version is already a fault
    // `scripts/check-release` reports, and an unstable sort would make the predecessor depend on
    // the order the filesystem happened to list them in.
    snapshots.sort_by(|left, right| {
        left.project
            .cmp(&right.project)
            .then_with(|| left.archive.cmp(&right.archive))
    });

    if !faults.is_empty() {
        return Err(ReleaseFaults(faults));
    }
    if let Some(adopted) = marker {
        // The marker has to name an archive that exists, or the history has no beginning anybody can
        // point at. Checked after the walk, which is what knows the recorded versions.
        let named = snapshots
            .iter()
            .any(|snapshot| snapshot.project == adopted.version)
            || records_version(archive, adopted.version);
        if !named {
            return Err(ReleaseFaults::one(ReleaseFault::AdoptionNamesNoArchive {
                version: adopted.version,
            }));
        }
    }
    Ok((marker, snapshots))
}

/// Whether any archive in `archive` records `version`.
///
/// Separate from the snapshot walk because the adoption archive may legitimately hold an empty
/// inventory — a project adopting publication with no runbook yet — and an empty inventory
/// contributes no snapshot while still being the archive the marker names.
fn records_version(archive: &Path, version: ProjectVersion) -> bool {
    let Ok(entries) = std::fs::read_dir(archive) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let recorded = entry.path().join(crate::project::RECORDED_VERSION);
        std::fs::read_to_string(recorded)
            .ok()
            .and_then(|text| ProjectVersion::parse(&text).ok())
            .is_some_and(|found| found == version)
    })
}

/// The last snapshot published for one runbook name.
///
/// Takes the ordered history rather than reading it, so the ordering rule is stated once and a
/// caller that already walked the archive does not walk it again.
pub fn predecessor(history: &[Snapshot], name: &str) -> PreviousRelease {
    match history
        .iter()
        .filter(|snapshot| snapshot.release.name == name)
        .next_back()
    {
        Some(snapshot) => PreviousRelease::Snapshot(snapshot.clone()),
        None => PreviousRelease::None,
    }
}

/// Every runbook a planning root currently holds, with the release each declares.
///
/// The names are the directory names, which is the identity `project-bootstrap` fixes for the
/// comparison. A directory holding neither canonical document is not a runbook and is skipped: the
/// project's own enumeration applies the same test, and a directory somebody left half-made should
/// not refuse an archival.
pub fn present(runbooks: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(runbooks) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.join(crate::runs::CONTRACT).is_file()
                || path.join(crate::runs::PROCEDURE).is_file()
        })
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().into_owned();
            Some((name, path))
        })
        .collect();
    found.sort_by(|left, right| left.0.cmp(&right.0));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixtures both readers share, at the repository root.
    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/release")
            .canonicalize()
            .expect("the shared release fixtures are present")
    }

    fn digest(fill: u8) -> ContentDigest {
        ContentDigest(format!(
            "{DIGEST_ALGORITHM}:{}",
            std::iter::repeat_n(format!("{fill:02x}", fill = fill), 32)
                .take(32)
                .collect::<String>()
        ))
    }

    fn version(text: &str) -> RunbookVersion {
        RunbookVersion::parse(text).expect("a version")
    }

    fn release(name: &str, declared: &str, fill: u8, changelog: &str) -> RunbookRelease {
        RunbookRelease {
            name: name.to_owned(),
            version: version(declared),
            specification: digest(fill),
            procedure: digest(fill.wrapping_add(1)),
            changelog: changelog.to_owned(),
        }
    }

    fn snapshot(archive: &str, project: &str, release: RunbookRelease) -> Snapshot {
        Snapshot {
            project: ProjectVersion::parse(project).expect("a version"),
            archive: archive.to_owned(),
            release,
        }
    }

    /// A fingerprint is a whole digest, and identical bytes fingerprint alike.
    #[test]
    fn a_digest_is_the_algorithm_and_its_value() {
        let one = ContentDigest::of(b"the same bytes");
        let same = ContentDigest::of(b"the same bytes");
        let other = ContentDigest::of(b"different bytes");
        assert_eq!(one, same);
        assert_ne!(one, other);
        assert!(one.as_str().starts_with("sha256:"), "{one}");
        assert_eq!(
            one.as_str().len(),
            "sha256:".len() + 64,
            "the digest is not a whole sha256 value: {one}"
        );

        // And it round-trips, which is what an inventory entry read back has to do.
        assert_eq!(ContentDigest::parse(one.as_str()).expect("parses"), one);
    }

    /// A digest in another form is refused, because it would compare unequal to the same content.
    #[test]
    fn a_digest_of_another_shape_is_refused() {
        for (text, fault) in [
            ("not-a-digest", DigestFault::Algorithm),
            (
                "md5:0123456789abcdef0123456789abcdef",
                DigestFault::Algorithm,
            ),
            ("sha256:tooshort", DigestFault::Hex),
            (
                "sha256:0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF",
                DigestFault::Hex,
            ),
            (
                "sha256:zzzzzz56789abcdef0123456789abcdef0123456789abcdef0123456789abcd",
                DigestFault::Hex,
            ),
        ] {
            assert_eq!(
                ContentDigest::parse(text).expect_err("accepted"),
                fault,
                "{text}"
            );
        }
    }

    /// A changelog's headings are its releases, and prose naming a version is not one.
    #[test]
    fn a_heading_names_a_release_and_prose_does_not() {
        let changelog = "# Changelog\n\n\
             ## 0.2.0\n\nA second release.\n\n\
             ## 0.1.0\n\nThe first release, which fixed 1.2.3 of nothing in particular.\n\n\
             ## 0.1.0\n\nA repeated heading.\n";
        assert_eq!(
            changelog_headings(changelog)
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["0.2.0", "0.1.0", "0.1.0"],
            "prose was counted as a release, or a heading was missed"
        );
        assert_eq!(
            duplicate_headings(changelog)
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["0.1.0"]
        );
        assert!(
            duplicate_headings("# Changelog\n\n## 0.1.0\n\nOne release.\n").is_empty(),
            "a single heading was reported as a repeat"
        );
    }

    /// A version-shaped run that is not three components is not a heading.
    #[test]
    fn a_dotted_run_that_is_not_a_version_is_not_a_heading() {
        for changelog in [
            "## 1.2\n",
            "## 1.2.3.4\n",
            "## release\n",
            "no heading here 1.2.3\n",
        ] {
            assert!(
                changelog_headings(changelog).is_empty(),
                "{changelog:?} produced a heading"
            );
        }
    }

    /// An inventory round-trips, and an empty one is a record rather than an omission.
    #[test]
    fn an_inventory_round_trips_including_the_empty_case() {
        let full = Inventory::of(vec![
            release(
                "release",
                "0.2.0",
                1,
                "# Changelog\n\n## 0.2.0\n\nSecond.\n",
            ),
            release("cookies", "0.1.0", 3, "# Changelog\n\n## 0.1.0\n\nFirst.\n"),
        ])
        .expect("an inventory");
        let rendered = full.render();
        assert_eq!(
            Inventory::parse(&rendered).expect("parses"),
            full,
            "an inventory did not survive its own rendering:\n{rendered}"
        );
        assert!(rendered.contains(INVENTORY_FORMAT), "{rendered}");
        // Pretty-printed and stable, so a committed inventory diffs as a change in meaning.
        assert_eq!(
            Inventory::parse(&rendered).expect("parses").render(),
            rendered
        );

        let empty = Inventory::of(Vec::new()).expect("an inventory of none");
        let rendered = empty.render();
        assert!(
            rendered.contains("\"runbooks\": []"),
            "an empty inventory does not state that it is empty:\n{rendered}"
        );
        assert_eq!(Inventory::parse(&rendered).expect("parses"), empty);
        assert!(empty.is_empty(), "an inventory of none reported a length");
    }

    /// Two entries naming one runbook are refused, and the name is in the report.
    #[test]
    fn an_inventory_recording_one_runbook_twice_is_refused() {
        let faults = Inventory::of(vec![
            release("release", "0.1.0", 1, "## 0.1.0\n"),
            release("release", "0.2.0", 2, "## 0.2.0\n"),
        ])
        .expect_err("a repeated name");
        assert!(
            faults.to_string().contains("release"),
            "the report does not name the runbook: {faults}"
        );
        assert!(faults.to_string().contains("twice"), "{faults}");
    }

    /// A malformed inventory is refused naming what is wrong, in every shape it can be wrong in.
    #[test]
    fn an_inventory_of_another_shape_is_refused() {
        for (text, wanted) in [
            ("not json", "not JSON"),
            ("[]", "one JSON object"),
            ("{}", "no format"),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v2\",\"runbooks\":[]}",
                "htmlspec-runbook-inventory/v2",
            ),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v1\"}",
                "states its runbooks",
            ),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":{}}",
                "`runbooks` is a list",
            ),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[],\"extra\":1}",
                "`extra` is not a field",
            ),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[{\"name\":\"r\"}]}",
                "states a version",
            ),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[\
                 {\"name\":\"r\",\"version\":\"1.0\",\"specification\":\"sha256:x\",\
                 \"procedure\":\"sha256:x\",\"changelog\":\"\"}]}",
                "version is not one",
            ),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[\
                 {\"name\":\"r\",\"version\":\"1.0.0\",\"specification\":\"nope\",\
                 \"procedure\":\"sha256:x\",\"changelog\":\"\"}]}",
                "fingerprint is not one",
            ),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[\
                 {\"name\":\"r\",\"version\":\"1.0.0\",\"specification\":\"sha256:abababababababababababababababababababababababababababababababab\",\
                 \"procedure\":\"sha256:abababababababababababababababababababababababababababababababab\"}]}",
                "carries its changelog",
            ),
            (
                "{\"format\":\"htmlspec-runbook-inventory/v1\",\"runbooks\":[\
                 {\"name\":\"r\",\"version\":\"1.0.0\",\"specification\":\"sha256:x\",\
                 \"procedure\":\"sha256:x\",\"changelog\":\"\",\"extra\":1}]}",
                "`extra` is not a field an entry has",
            ),
        ] {
            let faults = Inventory::parse(text).expect_err("accepted");
            assert!(
                faults.to_string().contains(wanted),
                "{text:?} reported {faults} and not {wanted:?}"
            );
        }
    }

    /// An adoption marker round-trips and refuses the shapes it cannot be.
    #[test]
    fn an_adoption_marker_round_trips_and_refuses_another_shape() {
        let marker = Adoption {
            version: ProjectVersion::parse("0.15.0").expect("a version"),
        };
        let rendered = marker.render();
        assert_eq!(Adoption::parse(&rendered).expect("parses"), marker);
        assert!(rendered.contains(ADOPTION_FORMAT), "{rendered}");

        for (text, wanted) in [
            ("nope", "not JSON"),
            ("[]", "one JSON object"),
            ("{}", "no format"),
            (
                "{\"format\":\"htmlspec-runbook-adoption/v2\",\"version\":\"0.15.0\"}",
                "htmlspec-runbook-adoption/v2",
            ),
            (
                "{\"format\":\"htmlspec-runbook-adoption/v1\"}",
                "names the version",
            ),
            (
                "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"15\"}",
                "not one",
            ),
            (
                "{\"format\":\"htmlspec-runbook-adoption/v1\",\"version\":\"0.15.0\",\"x\":1}",
                "`x` is not a field",
            ),
        ] {
            let fault = Adoption::parse(text).expect_err("accepted");
            assert!(
                fault.to_string().contains(wanted),
                "{text:?} reported {fault} and not {wanted:?}"
            );
        }
    }

    /// A first release above 0.0.0 with its heading publishes.
    #[test]
    fn a_first_release_publishes() {
        let published = compare_release(
            PreviousRelease::None,
            release("release", "0.1.0", 1, "# Changelog\n\n## 0.1.0\n\nFirst.\n"),
        )
        .expect("publishable");
        assert_eq!(published.release.version, version("0.1.0"));
    }

    /// A first release at 0.0.0 does not, and the report says what to write instead.
    #[test]
    fn a_first_release_at_the_unreleased_version_is_refused() {
        let faults = compare_release(
            PreviousRelease::None,
            release(
                "release",
                "0.0.0",
                1,
                "# Changelog\n\n## 0.0.0\n\nAdoption.\n",
            ),
        )
        .expect_err("0.0.0 published");
        let text = faults.to_string();
        assert!(
            text.contains("not a release") && text.contains(VERSION_FILE),
            "{text}"
        );
    }

    /// Unchanged content at one version is the same release snapshotted twice, not a duplicate.
    #[test]
    fn unchanged_content_at_one_version_is_one_release() {
        let published = release("release", "0.1.0", 1, "# Changelog\n\n## 0.1.0\n\nFirst.\n");
        let again = compare_release(
            PreviousRelease::Snapshot(snapshot("2026-01-01-one", "0.15.0", published.clone())),
            published.clone(),
        )
        .expect("a repeated snapshot publishes");
        assert_eq!(again.release, published);
    }

    /// Unchanged content may advance its version, with a heading for the new one.
    #[test]
    fn unchanged_content_may_still_advance_its_version() {
        let published = release("release", "0.1.0", 1, "# Changelog\n\n## 0.1.0\n\nFirst.\n");
        let advanced = release(
            "release",
            "0.2.0",
            1,
            "# Changelog\n\n## 0.2.0\n\nEditorial.\n\n## 0.1.0\n\nFirst.\n",
        );
        assert!(
            compare_release(
                PreviousRelease::Snapshot(snapshot("2026-01-01-one", "0.15.0", published)),
                advanced
            )
            .is_ok(),
            "a version-only release was refused"
        );
    }

    /// Changed content at one version is refused, naming the archive that published it.
    #[test]
    fn changed_content_without_a_bump_is_refused_naming_the_archive() {
        let published = release("release", "0.1.0", 1, "# Changelog\n\n## 0.1.0\n\nFirst.\n");
        let changed = release("release", "0.1.0", 2, "# Changelog\n\n## 0.1.0\n\nFirst.\n");
        let faults = compare_release(
            PreviousRelease::Snapshot(snapshot("2026-01-01-one", "0.15.0", published)),
            changed,
        )
        .expect_err("one version identified two procedures");
        let text = faults.to_string();
        assert!(text.contains("2026-01-01-one"), "{text}");
        assert!(
            text.contains("one released procedure"),
            "the report does not say why a version cannot be reused: {text}"
        );
        assert!(
            text.contains("htmlspec format --upgrade-assets")
                && text.contains("htmlspec format --fallback"),
            "the report does not name the asset upgrade and the fallback, the likeliest unedited \
             causes: {text}"
        );
    }

    /// What a changed runbook's version needs is what archival decides for it: a release exactly
    /// where the comparison of changed content raises no fault.
    #[test]
    fn what_a_changed_version_needs_agrees_with_archival() {
        let published = release("release", "0.1.0", 1, "# Changelog\n\n## 0.1.0\n\nFirst.\n");
        let after =
            || PreviousRelease::Snapshot(snapshot("2026-01-01-one", "0.15.0", published.clone()));
        let must_exceed = || VersionAfterAChange::MustExceed {
            published: version("0.1.0"),
            published_in: "2026-01-01-one".to_owned(),
        };
        for (previous, declared, headed, wanted) in [
            (
                PreviousRelease::None,
                "0.0.0",
                true,
                VersionAfterAChange::NeedsAFirstRelease,
            ),
            (
                PreviousRelease::None,
                "0.1.0",
                true,
                VersionAfterAChange::FirstRelease(version("0.1.0")),
            ),
            (
                PreviousRelease::None,
                "0.1.0",
                false,
                VersionAfterAChange::NeedsAHeading(version("0.1.0")),
            ),
            (after(), "0.0.9", true, must_exceed()),
            (after(), "0.1.0", true, must_exceed()),
            (after(), "0.1.0", false, must_exceed()),
            (
                after(),
                "0.1.1",
                true,
                VersionAfterAChange::Advances(version("0.1.1")),
            ),
            (
                after(),
                "0.1.1",
                false,
                VersionAfterAChange::NeedsAHeading(version("0.1.1")),
            ),
        ] {
            // Changed content, with one heading for the declared version or none, so only the
            // version and its heading decide.
            let changelog = if headed {
                format!("# Changelog\n\n## {declared}\n\nChanged.\n")
            } else {
                "# Changelog\n\nChanged.\n".to_owned()
            };
            let needs = version_after_a_change(&previous, version(declared), &changelog);
            assert_eq!(
                needs, wanted,
                "{declared}, headed {headed}, after {previous:?}"
            );
            let verdict = compare_release(previous, release("release", declared, 2, &changelog));
            assert_eq!(
                matches!(
                    needs,
                    VersionAfterAChange::FirstRelease(_) | VersionAfterAChange::Advances(_)
                ),
                verdict.is_ok(),
                "{declared}, headed {headed}: the advice {needs:?} and archival's {verdict:?} \
                 disagree"
            );
        }
    }

    /// A changed specification alone moves the release, and so does a changed procedure alone.
    #[test]
    fn either_document_changing_requires_a_greater_version() {
        let published = release("release", "0.1.0", 1, "# Changelog\n\n## 0.1.0\n\nFirst.\n");

        let specification_only = RunbookRelease {
            specification: digest(9),
            ..published.clone()
        };
        assert!(
            compare_release(
                PreviousRelease::Snapshot(snapshot("a", "0.15.0", published.clone())),
                specification_only
            )
            .is_err(),
            "a changed specification published at one version"
        );

        let procedure_only = RunbookRelease {
            procedure: digest(9),
            ..published.clone()
        };
        assert!(
            compare_release(
                PreviousRelease::Snapshot(snapshot("a", "0.15.0", published)),
                procedure_only
            )
            .is_err(),
            "a changed procedure published at one version"
        );
    }

    /// A downgrade is refused even where the bytes are identical.
    ///
    /// The ordering is the gate rather than a difference test, and this is the case that separates
    /// them: a difference test would accept it, because the content did not differ.
    #[test]
    fn a_downgrade_is_refused_even_with_identical_content() {
        let published = release(
            "release",
            "0.2.0",
            1,
            "# Changelog\n\n## 0.2.0\n\nSecond.\n",
        );
        let downgraded = release(
            "release",
            "0.1.0",
            1,
            "# Changelog\n\n## 0.2.0\n\nSecond.\n\n## 0.1.0\n\nFirst.\n",
        );
        let faults = compare_release(
            PreviousRelease::Snapshot(snapshot("2026-02-02-two", "0.16.0", published)),
            downgraded,
        )
        .expect_err("a downgrade published");
        let text = faults.to_string();
        assert!(text.contains("ordered before"), "{text}");
        assert!(text.contains("0.2.0") && text.contains("0.1.0"), "{text}");
        assert!(text.contains("2026-02-02-two"), "{text}");
    }

    /// A missing heading for the declared version is refused, and so is a repeated one.
    #[test]
    fn a_release_its_changelog_does_not_head_is_refused() {
        let faults = compare_release(
            PreviousRelease::None,
            release(
                "release",
                "0.1.0",
                1,
                "# Changelog\n\n## 0.0.9\n\nSomething else.\n",
            ),
        )
        .expect_err("an unheaded release");
        let text = faults.to_string();
        assert!(
            text.contains(CHANGELOG_FILE) && text.contains("no heading"),
            "{text}"
        );

        let faults = compare_release(
            PreviousRelease::None,
            release(
                "release",
                "0.1.0",
                1,
                "# Changelog\n\n## 0.1.0\n\nOne.\n\n## 0.1.0\n\nTwice.\n",
            ),
        )
        .expect_err("a twice-headed release");
        assert!(faults.to_string().contains("twice"), "{}", faults);
    }

    /// Every fault about one release is reported at once, so an author fixes them in one pass.
    ///
    /// **Two rather than three, and the reason is worth stating: a downgrade and a reused version
    /// are mutually exclusive answers about one version.** A version ordered before the published
    /// one cannot also be equal to it, so the case that reports two independent faults is changed
    /// content at the published version beside a changelog that does not head it — which is also the
    /// mistake an author actually makes.
    #[test]
    fn every_fault_about_one_release_is_reported_together() {
        let published = release(
            "release",
            "0.2.0",
            1,
            "# Changelog\n\n## 0.2.0\n\nSecond.\n",
        );
        let faults = compare_release(
            PreviousRelease::Snapshot(snapshot("2026-01-01-one", "0.15.0", published)),
            release(
                "release",
                "0.2.0",
                2,
                "# Changelog\n\n## 0.9.0\n\nNothing.\n",
            ),
        )
        .expect_err("refused");
        assert_eq!(
            faults.faults().len(),
            2,
            "changed content and a missing heading were not both reported: {faults}"
        );
        let text = faults.to_string();
        assert!(
            text.contains("ChangedWithoutABump".to_lowercase().as_str())
                || text.contains("already published"),
            "{text}"
        );
        assert!(text.contains("no heading"), "{text}");

        // And the ordering case reports its own single fault rather than being swallowed by it.
        let faults = compare_release(
            PreviousRelease::Snapshot(snapshot(
                "2026-02-02-two",
                "0.16.0",
                release("release", "0.2.0", 1, "## 0.2.0\n"),
            )),
            release("release", "0.1.0", 1, "## 0.1.0\n"),
        )
        .expect_err("refused");
        assert_eq!(faults.faults().len(), 1, "{faults}");
    }

    /// Two runbooks may each be at one version, because identity is per runbook.
    #[test]
    fn two_runbooks_may_share_a_version() {
        for name in ["release", "cookies"] {
            assert!(
                compare_release(
                    PreviousRelease::None,
                    release(name, "0.1.0", 1, "# Changelog\n\n## 0.1.0\n\nFirst.\n")
                )
                .is_ok(),
                "{name} was refused for a version another runbook also declares"
            );
        }
    }

    /// A renamed runbook begins a new lineage, and the old name's snapshots stay where they were.
    #[test]
    fn a_renamed_runbook_begins_a_new_lineage() {
        let history = vec![
            snapshot(
                "2026-01-01-one",
                "0.15.0",
                release("release", "0.3.0", 1, "## 0.3.0\n"),
            ),
            snapshot(
                "2026-02-02-two",
                "0.16.0",
                release("shipping", "0.1.0", 1, "## 0.1.0\n"),
            ),
        ];
        match predecessor(&history, "shipping") {
            PreviousRelease::None => panic!("the new name found no predecessor at all"),
            PreviousRelease::Snapshot(found) => {
                assert_eq!(found.release.version, version("0.1.0"));
            }
        }
        // And the new name's first release is judged as a first release, not against `release`'s
        // 0.3.0 — which is what "renaming intentionally starts a new lineage" means.
        assert!(compare_release(
            predecessor(&history, "shipping"),
            release("shipping", "0.1.0", 1, "## 0.1.0\n")
        )
        .is_ok());
        assert_eq!(
            predecessor(&history, "gone"),
            PreviousRelease::None,
            "a name nothing published found a predecessor"
        );
        // The former name's history is still there, so a deleted runbook does not take its releases
        // with it.
        match predecessor(&history, "release") {
            PreviousRelease::Snapshot(found) => assert_eq!(found.release.version, version("0.3.0")),
            PreviousRelease::None => panic!("a published runbook lost its history"),
        }
    }

    /// The predecessor is the one with the greatest recorded project version, whatever the
    /// directories are called.
    #[test]
    fn the_predecessor_is_ordered_by_recorded_version_not_by_name() {
        let release_one = release("release", "0.1.0", 1, "## 0.1.0\n");
        let release_two = release("release", "0.2.0", 2, "## 0.2.0\n");
        // Named so that a lexical ordering would pick the wrong one: `zzz` sorts after `aaa` but
        // records the earlier project version.
        let history = vec![
            snapshot("zzz-early", "0.15.0", release_one),
            snapshot("aaa-later", "0.16.0", release_two.clone()),
        ];
        let mut ordered = history.clone();
        ordered.sort_by(|left, right| {
            left.project
                .cmp(&right.project)
                .then_with(|| left.archive.cmp(&right.archive))
        });
        match predecessor(&ordered, "release") {
            PreviousRelease::Snapshot(found) => {
                assert_eq!(
                    found.release, release_two,
                    "a directory name chose the predecessor"
                );
                assert_eq!(found.archive, "aaa-later");
            }
            PreviousRelease::None => panic!("no predecessor was found"),
        }
        // And a rename of an archive directory leaves the answer alone, which is the property
        // `project-bootstrap` states: recorded versions determine the predecessor.
        let renamed: Vec<Snapshot> = ordered
            .iter()
            .map(|entry| Snapshot {
                archive: format!("renamed-{}", entry.archive),
                ..entry.clone()
            })
            .collect();
        match predecessor(&renamed, "release") {
            PreviousRelease::Snapshot(found) => assert_eq!(found.release, release_two),
            PreviousRelease::None => panic!("renaming an archive lost the history"),
        }
    }

    /// A fresh scratch directory for a test that reads files, removed and recreated per call.
    fn scratch(label: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("htmlspec-release-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("scratch created");
        directory
    }

    /// An unreadable runbook is reported as the runbook, not as an archive's inventory.
    ///
    /// Before this had its own variant, a runbook missing its `VERSION` was reported as "an
    /// unreadable runbooks.json", which sent the author to an archive that was not at fault.
    #[test]
    fn an_unreadable_runbook_names_the_runbook_and_not_an_inventory() {
        let directory = scratch("unreadable-runbook");
        let faults = release_of(&directory, "release").expect_err("read as a release");
        match faults.faults() {
            [ReleaseFault::UnreadableRunbook { name, problem, .. }] => {
                assert_eq!(name, "release");
                assert!(problem.contains(VERSION_FILE), "{problem}");
            }
            other => panic!("an unversioned runbook was reported as {other:?}"),
        }
        let report = faults.to_string();
        assert!(!report.contains(INVENTORY_FILE), "{report}");

        // Each later read names its own file, in the same variant.
        std::fs::write(directory.join(VERSION_FILE), "0.1.0\n").expect("written");
        let report = release_of(&directory, "release")
            .expect_err("read as a release")
            .to_string();
        assert!(report.contains(CHANGELOG_FILE), "{report}");
        assert!(!report.contains(INVENTORY_FILE), "{report}");
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// An unparseable adoption marker is reported as the marker, not as an archive's inventory.
    #[test]
    fn an_unreadable_adoption_marker_names_the_marker_and_not_an_inventory() {
        let directory = scratch("unreadable-adoption");
        let marker = directory.join(ADOPTION_FILE);
        std::fs::write(&marker, "not a marker\n").expect("written");
        let faults = history(&directory.join("archive"), &marker).expect_err("read as a history");
        match faults.faults() {
            [ReleaseFault::UnreadableAdoption { marker: named, .. }] => {
                assert_eq!(named, &marker.display().to_string());
            }
            other => panic!("an unparseable marker was reported as {other:?}"),
        }
        let report = faults.to_string();
        assert!(report.contains(ADOPTION_FILE), "{report}");
        assert!(!report.contains(INVENTORY_FILE), "{report}");
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// Ten is ordered after nine, which is the whole reason versions are parsed rather than
    /// compared as text.
    #[test]
    fn the_ordering_is_numeric() {
        let history = vec![
            snapshot("a", "0.9.0", release("release", "0.1.0", 1, "## 0.1.0\n")),
            snapshot("b", "0.10.0", release("release", "0.2.0", 2, "## 0.2.0\n")),
        ];
        let mut ordered = history;
        ordered.sort_by(|left, right| left.project.cmp(&right.project));
        assert_eq!(ordered[1].archive, "b", "0.10.0 was ordered before 0.9.0");
    }

    /// The shared fixtures are what both readers read, and this reader agrees with them.
    ///
    /// **The test that keeps two implementations of one format honest.** `scripts/check-release`
    /// reads the same files in Python, because it runs where no build has happened; a fixture set
    /// only one side reads would let the two disagree about version ordering or about what a legacy
    /// archive is, and the disagreement would surface as a suite that passes locally and fails in
    /// continuous integration.
    #[test]
    fn the_shared_fixtures_say_what_both_readers_read() {
        let cases = std::fs::read_to_string(fixtures().join("cases.json"))
            .expect("the shared fixture manifest is readable");
        let cases: serde_json::Value = serde_json::from_str(&cases).expect("the manifest is JSON");

        for case in cases["inventories"].as_array().expect("inventory cases") {
            let file = case["file"].as_str().expect("a file name");
            let text = std::fs::read_to_string(fixtures().join(file)).unwrap_or_else(|error| {
                panic!("{file} is missing from the shared fixtures: {error}")
            });
            match Inventory::parse(&text) {
                Ok(inventory) => {
                    assert!(
                        case["parses"].as_bool() == Some(true),
                        "{file} parsed here and the shared manifest says it should not"
                    );
                    let names: Vec<String> = inventory
                        .releases()
                        .iter()
                        .map(|release| release.name.clone())
                        .collect();
                    let expected: Vec<String> = case["runbooks"]
                        .as_array()
                        .expect("a runbook list")
                        .iter()
                        .filter_map(|name| name.as_str().map(str::to_owned))
                        .collect();
                    assert_eq!(names, expected, "{file}");
                }
                Err(faults) => {
                    assert!(
                        case["parses"].as_bool() == Some(false),
                        "{file} was refused here and the shared manifest says it parses: {faults}"
                    );
                }
            }
        }

        for case in cases["adoptions"].as_array().expect("adoption cases") {
            let file = case["file"].as_str().expect("a file name");
            let text = std::fs::read_to_string(fixtures().join(file)).unwrap_or_else(|error| {
                panic!("{file} is missing from the shared fixtures: {error}")
            });
            match Adoption::parse(&text) {
                Ok(marker) => {
                    assert!(
                        case["parses"].as_bool() == Some(true),
                        "{file} parsed here and the shared manifest says it should not"
                    );
                    assert_eq!(
                        marker.version.to_string(),
                        case["version"].as_str().expect("a version"),
                        "{file}"
                    );
                }
                Err(fault) => {
                    assert!(
                        case["parses"].as_bool() == Some(false),
                        "{file} was refused here and the shared manifest says it parses: {fault}"
                    );
                }
            }
        }

        for case in cases["comparisons"].as_array().expect("comparison cases") {
            let name = case["name"].as_str().expect("a case name");
            let current = release_from_json(&case["current"]).expect("a current release");
            let previous = match case["previous"].as_object() {
                None => PreviousRelease::None,
                Some(previous) => PreviousRelease::Snapshot(Snapshot {
                    project: ProjectVersion::parse(
                        previous["project"].as_str().expect("a version"),
                    )
                    .expect("a project version"),
                    archive: previous["archive"].as_str().expect("an archive").to_owned(),
                    release: release_from_json(&previous["release"]).expect("a published release"),
                }),
            };
            let expected = case["decision"].as_str().expect("a decision");
            match compare_release(previous, current) {
                Ok(_) => assert_eq!(expected, "publish", "{name} published and should not have"),
                Err(faults) => assert_eq!(
                    expected, "refuse",
                    "{name} was refused and should have published: {faults}"
                ),
            }
        }
    }

    /// One fixture release, read the way the manifest spells it.
    fn release_from_json(value: &serde_json::Value) -> Result<RunbookRelease, String> {
        Ok(RunbookRelease {
            name: value["name"].as_str().ok_or("a name")?.to_owned(),
            version: RunbookVersion::parse(value["version"].as_str().ok_or("a version")?)
                .map_err(|error| error.to_string())?,
            specification: ContentDigest::parse(value["specification"].as_str().ok_or("a digest")?)
                .map_err(|fault| fault.to_string())?,
            procedure: ContentDigest::parse(value["procedure"].as_str().ok_or("a digest")?)
                .map_err(|fault| fault.to_string())?,
            changelog: value["changelog"].as_str().ok_or("a changelog")?.to_owned(),
        })
    }
}
