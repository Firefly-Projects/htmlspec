//! `cargo xtask crate-notices`: write the notices of the Rust crates the `htmlspec` binary is built
//! from, and `--check` them.
//!
//! # What it credits
//!
//! The binary links every Rust crate it depends on, and their licenses (MIT, Apache-2.0, and the
//! others [`License`] names) ask that their notices travel with a binary copy. So each channel that
//! ships the binary ships `crates/htmlspec-cli/CRATE-LICENSES` beside it, and this tool writes that
//! file.
//!
//! The credited set is the union, over the four targets a release builds, of two closures, leaving
//! out the workspace's own crates:
//!
//! - `htmlspec-cli`'s normal-dependency closure, procedural macros and their dependencies
//!   included, because a macro's output is compiled into the binary;
//! - the closure of each workspace crate's own build dependencies, because a workspace build
//!   script generates code that is compiled in: `htmlspec-schema`'s turns the schema into Rust
//!   types with `typify`, `schemars`, and `prettyplease`.
//!
//! A third-party crate's build dependencies are left out, because they mostly probe the compiler and
//! leave nothing of their own in the binary, and so is every development dependency. Each target is
//! read through `cargo metadata --filter-platform`, by [`Graph`], so a crate only another platform
//! builds is not credited, and a crate only one release target builds is.
//!
//! # Which text
//!
//! A crate's SPDX expression decides what it is credited under, as [`Expression::credit`] reads
//! it, and a crate that leaves no accepted choice is refused.
//!
//! The text comes from a file whose name states the license: `LICENSE-X` or `LICENSE.X`, ignoring
//! ASCII case and optionally ending in `.md` or `.txt`, where X is one of the license's
//! [`License::file_keys`]; an `UNLICENSE` file for the Unlicense; and `LICENSE`, `LICENCE`, or
//! `COPYING` only where the crate's expression names one license. Nothing else is matched, and no
//! text is guessed at by its contents. A crate whose text the rule cannot find needs a reviewed row
//! in `license-clarifications.tsv`, which [`clarifications`] reads, naming its exact version, so an
//! upgrade asks for the review again. A vendored standard text carries a copyright line: the row's
//! own, where a reviewer read one in the crate's repository, or otherwise one built from the crate's
//! authors, with every address removed.
//!
//! An Apache-2.0 crate's `NOTICE` file is reproduced, as that license's section 4(d) asks, and an
//! MPL-2.0 crate's entry says where its source form is, as that license's section 3.2 asks.
//!
//! The file's header names every credited license, and the formula renderer reads it, so the
//! Homebrew formula declares what the binary credits. The output is the same bytes for the same
//! lockfile and sources: no date, and every list sorted.
//!
//! # `--check`
//!
//! Compares the committed file with what the lockfile reaches, naming each crate added, removed, or
//! changed, and writes nothing. Then it holds the Homebrew formula's template to what the notices
//! credit and what `htmlspec-assets` declares it embeds, so a pull request on Linux alone meets a
//! template that drifted, and not only the macOS-gated formula job. That second check needs the
//! assets crate's `license`, which only a manifest states, so it is made here, where cargo reads the
//! workspace, and the renderer, which runs where no toolchain is installed, does not repeat it.
//!
//! Exit codes: 0 when the notices are written, or match; 1 when a crate, the table, the committed
//! file, or the template is at fault, naming each; and 2 when the workspace or a file the tool
//! needs cannot be read.

mod clarifications;
mod license;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clarifications::{Row, RowKey, Source, TableFault};
use license::{ChoiceFault, Expression, License};

use crate::facts::{Facts, MacosJobs, Target};
use crate::render_formula;
use crate::report::{Exit, Report};
use crate::workspace::graph::{Graph, PackageId};
use crate::workspace::{cargo, CrateName, DependencyKind, PackageVersion};

/// The crate whose normal-dependency closure is the binary.
const BINARY_CRATE: &str = "htmlspec-cli";

/// The crate whose `license` states what the binary embeds for documents to load.
const ASSETS_CRATE: &str = "htmlspec-assets";

/// The notices, the reviewed table, and the vendored texts, from the workspace's root.
pub const OUTPUT: &str = "crates/htmlspec-cli/CRATE-LICENSES";
const TABLE: &str = "crates/htmlspec-cli/license-clarifications.tsv";
const STANDARD_TEXTS: &str = "crates/htmlspec-cli/licenses";

/// Where an MPL-2.0 crate's source form is, and the copyright line's fallback names a crate.
fn crate_page(name: &CrateName, version: &PackageVersion) -> String {
    format!("https://crates.io/crates/{name}/{version}")
}

const RULE: &str =
    "--------------------------------------------------------------------------------";

/// The file's header, which says where it comes from.
const HEADER: &str = "\
Licenses of the Rust crates the htmlspec binary is built from

The crates listed below are the ones the htmlspec program is compiled from, for the four targets
a release builds, and every crate the workspace's code generation depends on. Their licenses ask
that their notices travel with a binary copy, so each one's text follows, as the crate ships it,
or as license-clarifications.tsv records a reviewed choice where a crate ships no file that names
its license. The libraries the binary embeds for documents to load, Mermaid and Vega among them,
have their own notices in THIRD-PARTY-LICENSES.

Generated by cargo xtask crate-notices from Cargo.lock. Change the tool or the table, never this
file: the suite regenerates it and fails on any difference.
";

/// Whether the tool writes the notices or compares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Write,
    Check,
}

/// A crate the binary is built from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crate {
    pub name: CrateName,
    pub version: PackageVersion,
    /// The manifest's `license`, as written, where it states one.
    pub expression: Option<String>,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub directory: PathBuf,
}

/// A crate with the expression it is credited by, as its manifest writes it, the licenses it is
/// credited under, and their texts, each with the label that says where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    krate: Crate,
    written: String,
    licenses: Vec<License>,
    texts: Vec<(String, String)>,
}

/// One reason the notices cannot be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoticeFault {
    /// A target's graph holds no binary crate among the workspace's members.
    NoBinaryCrate,
    /// The table cannot be read.
    Table(TableFault),
    /// A crate's expression credits no accepted license.
    Choice {
        name: CrateName,
        version: PackageVersion,
        fault: ChoiceFault,
    },
    /// A license text that is not UTF-8.
    NotText {
        name: CrateName,
        version: PackageVersion,
        file: String,
    },
    /// A license text that could not be read.
    Unreadable {
        name: CrateName,
        version: PackageVersion,
        file: String,
        kind: std::io::ErrorKind,
    },
    /// A row names a vendored text that is not vendored.
    NotVendored {
        name: CrateName,
        version: PackageVersion,
        source: String,
    },
    /// A row names a file the crate does not ship.
    NotShipped {
        name: CrateName,
        version: PackageVersion,
        source: String,
    },
    /// A crate ships no file whose name states a license it is credited under, and has no row.
    NoText {
        name: CrateName,
        version: PackageVersion,
        license: License,
    },
    /// A row for a crate, version, or license the lockfile no longer credits.
    StaleRow(RowKey),
}

impl fmt::Display for NoticeFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoBinaryCrate => write!(formatter, "the workspace holds no {BINARY_CRATE}"),
            Self::Table(fault) => fault.fmt(formatter),
            Self::Choice {
                name,
                version,
                fault,
            } => write!(formatter, "{name} {version} {fault}"),
            Self::NotText {
                name,
                version,
                file,
            } => write!(formatter, "{name} {version}: {file} is not UTF-8 text"),
            Self::Unreadable {
                name,
                version,
                file,
                kind,
            } => write!(formatter, "{name} {version}: {file} cannot be read: {kind}"),
            Self::NotVendored {
                name,
                version,
                source,
            } => write!(
                formatter,
                "{name} {version}: its row names {source}, which is not vendored"
            ),
            Self::NotShipped {
                name,
                version,
                source,
            } => write!(
                formatter,
                "{name} {version}: its row names {source}, which the crate does not ship"
            ),
            Self::NoText {
                name,
                version,
                license,
            } => write!(
                formatter,
                "{name} {version} is credited under {license} and ships no file whose name states \
                 it, and {} has no row for it",
                clarifications::TABLE
            ),
            Self::StaleRow(key) => write!(
                formatter,
                "{} has a row for {} {} under {}, which the lockfile no longer credits; review \
                 the crate again and change or remove the row",
                clarifications::TABLE,
                key.name,
                key.version,
                key.license
            ),
        }
    }
}

/// The crates the binary is built from, over every target's graph, sorted by name and version.
pub fn credited(graphs: &[Graph]) -> Result<Vec<Crate>, NoticeFault> {
    let mut found: BTreeMap<(String, String), Crate> = BTreeMap::new();
    for graph in graphs {
        let roots: Vec<PackageId> = graph
            .members()
            .filter(|(_, package)| package.name.as_str() == BINARY_CRATE)
            .map(|(id, _)| id.clone())
            .collect();
        if roots.is_empty() {
            return Err(NoticeFault::NoBinaryCrate);
        }
        let normal = graph.closure(roots, DependencyKind::Normal);
        let build_roots: Vec<PackageId> = normal
            .iter()
            .filter(|id| graph.is_member(id))
            .flat_map(|id| graph.edges(id))
            .filter(|edge| edge.kinds.contains(&DependencyKind::Build))
            .map(|edge| edge.to.clone())
            .collect();
        let built = graph.closure(build_roots, DependencyKind::Normal);
        for id in normal.union(&built).filter(|id| !graph.is_member(id)) {
            let Some(package) = graph.package(id) else {
                continue;
            };
            found.insert(
                (
                    package.name.as_str().to_owned(),
                    package.version.as_str().to_owned(),
                ),
                Crate {
                    name: package.name.clone(),
                    version: package.version.clone(),
                    expression: package.license.clone(),
                    authors: package.authors.clone(),
                    repository: package.repository.clone(),
                    directory: package.directory.clone(),
                },
            );
        }
    }
    Ok(found.into_values().collect())
}

/// The files directly in `directory`, by name, sorted, or none where it is not a directory.
fn file_names(directory: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .collect();
    names.sort();
    names
}

/// Whether `name` is one of `stems`, optionally ending in `.md` or `.txt`, ignoring ASCII case.
fn named(name: &str, stems: &[String]) -> bool {
    let lower = name.to_ascii_lowercase();
    let mut candidates = vec![lower.as_str()];
    for suffix in [".md", ".txt"] {
        if let Some(stem) = lower.strip_suffix(suffix) {
            candidates.push(stem);
        }
    }
    candidates.into_iter().any(|candidate| {
        stems
            .iter()
            .any(|stem| stem.eq_ignore_ascii_case(candidate))
    })
}

/// The crate's file whose name states `license`, by the file-name rule alone.
fn text_file(krate: &Crate, expression: &Expression, license: License) -> Option<PathBuf> {
    let names = file_names(&krate.directory);
    let mut rules: Vec<Vec<String>> = Vec::new();
    if !license.file_keys().is_empty() {
        rules.push(
            license
                .file_keys()
                .iter()
                .flat_map(|key| [format!("LICENSE-{key}"), format!("LICENSE.{key}")])
                .collect(),
        );
    }
    if license == License::Unlicense {
        rules.push(vec![String::from("UNLICENSE")]);
    }
    if expression.identifiers().len() == 1 {
        rules.push(["LICENSE", "LICENCE", "COPYING"].map(String::from).to_vec());
    }
    rules.iter().find_map(|stems| {
        names
            .iter()
            .find(|name| named(name, stems))
            .map(|name| krate.directory.join(name))
    })
}

/// An Apache-2.0 crate's `NOTICE` files, which that license asks to be reproduced.
fn notice_files(krate: &Crate, expression: &Expression) -> Vec<String> {
    if !expression.names(License::Apache2) {
        return Vec::new();
    }
    file_names(&krate.directory)
        .into_iter()
        .filter(|name| named(name, &[String::from("NOTICE")]))
        .collect()
}

/// The copyright line a vendored standard text carries, built from the crate's authors with every
/// address removed, or naming where its authors are named.
fn copyright_line(krate: &Crate) -> String {
    let authors: Vec<String> = krate
        .authors
        .iter()
        .map(|author| without_addresses(author).trim().to_owned())
        .filter(|author| !author.is_empty())
        .collect();
    if !authors.is_empty() {
        return format!("Copyright (c) {}", authors.join(", "));
    }
    let place = krate
        .repository
        .clone()
        .filter(|repository| !repository.is_empty())
        .unwrap_or_else(|| crate_page(&krate.name, &krate.version));
    format!(
        "Copyright the authors of {}, as {place} names them",
        krate.name
    )
}

/// `author` with each `<...>` removed, and the space before it.
fn without_addresses(author: &str) -> String {
    let mut kept = String::new();
    let mut rest = author;
    while let Some(open) = rest.find('<') {
        let Some(close) = rest[open..].find('>') else {
            break;
        };
        kept.push_str(rest[..open].trim_end());
        rest = &rest[open + close + 1..];
    }
    kept.push_str(rest);
    kept
}

/// A text as it is reproduced: line endings made `\n`, and one at the end.
fn read_text(path: &Path, krate: &Crate, file: &str) -> Result<String, NoticeFault> {
    let bytes = std::fs::read(path).map_err(|error| NoticeFault::Unreadable {
        name: krate.name.clone(),
        version: krate.version.clone(),
        file: file.to_owned(),
        kind: error.kind(),
    })?;
    let text = String::from_utf8(bytes).map_err(|_| NoticeFault::NotText {
        name: krate.name.clone(),
        version: krate.version.clone(),
        file: file.to_owned(),
    })?;
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");
    Ok(format!("{}\n", unified.trim_end_matches('\n')))
}

/// Each crate with its credited licenses and their texts, or every fault found.
fn entries(
    crates: Vec<Crate>,
    rows: &BTreeMap<RowKey, Row>,
    standard: &Path,
) -> Result<Vec<Entry>, Vec<NoticeFault>> {
    let mut faults = Vec::new();
    let mut found = Vec::new();
    let mut used: BTreeSet<RowKey> = BTreeSet::new();
    for krate in crates {
        let read = krate
            .expression
            .clone()
            .ok_or(ChoiceFault::NoLicense)
            .and_then(|written| {
                let expression = Expression::parse(&written)?;
                let licenses = expression.credit(&written)?;
                Ok((written, expression, licenses))
            });
        let (written, expression, licenses) = match read {
            Ok(read) => read,
            Err(fault) => {
                faults.push(NoticeFault::Choice {
                    name: krate.name.clone(),
                    version: krate.version.clone(),
                    fault,
                });
                continue;
            }
        };
        let mut texts = Vec::new();
        for license in licenses.iter().copied() {
            let key = RowKey {
                name: krate.name.clone(),
                version: krate.version.clone(),
                license,
            };
            if let Some(row) = rows.get(&key) {
                used.insert(key);
                for source in &row.sources {
                    match source {
                        Source::Vendored { written, within } => {
                            let path = standard.join(within);
                            if !path.is_file() {
                                faults.push(NoticeFault::NotVendored {
                                    name: krate.name.clone(),
                                    version: krate.version.clone(),
                                    source: written.clone(),
                                });
                                continue;
                            }
                            match read_text(&path, &krate, written) {
                                Ok(text) => texts.push((
                                    format!(
                                        "{license}: the standard text, vendored as {written}, \
                                         reviewed by {}",
                                        row.reviewer
                                    ),
                                    format!(
                                        "{}\n\n{text}",
                                        row.copyright
                                            .clone()
                                            .unwrap_or_else(|| copyright_line(&krate))
                                    ),
                                )),
                                Err(fault) => faults.push(fault),
                            }
                        }
                        Source::Shipped { written } => {
                            let path = krate.directory.join(written);
                            if !path.is_file() {
                                faults.push(NoticeFault::NotShipped {
                                    name: krate.name.clone(),
                                    version: krate.version.clone(),
                                    source: written.clone(),
                                });
                                continue;
                            }
                            match read_text(&path, &krate, written) {
                                Ok(text) => texts.push((
                                    format!(
                                        "{license}: {written}, as the crate ships it, reviewed by \
                                         {}",
                                        row.reviewer
                                    ),
                                    text,
                                )),
                                Err(fault) => faults.push(fault),
                            }
                        }
                    }
                }
                continue;
            }
            let Some(path) = text_file(&krate, &expression, license) else {
                faults.push(NoticeFault::NoText {
                    name: krate.name.clone(),
                    version: krate.version.clone(),
                    license,
                });
                continue;
            };
            let file = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            match read_text(&path, &krate, &file) {
                Ok(text) => texts.push((format!("{license}: {file}, as the crate ships it"), text)),
                Err(fault) => faults.push(fault),
            }
        }
        for file in notice_files(&krate, &expression) {
            match read_text(&krate.directory.join(&file), &krate, &file) {
                Ok(text) => texts.push((format!("NOTICE: {file}, as the crate ships it"), text)),
                Err(fault) => faults.push(fault),
            }
        }
        found.push(Entry {
            krate,
            written,
            licenses,
            texts,
        });
    }
    let mut stale: Vec<&RowKey> = rows.keys().filter(|key| !used.contains(*key)).collect();
    stale.sort_by_key(|key| key.sort_text());
    faults.extend(stale.into_iter().cloned().map(NoticeFault::StaleRow));
    if faults.is_empty() {
        Ok(found)
    } else {
        Err(faults)
    }
}

/// `licenses` joined by `AND`, as the file states them.
fn joined(licenses: &[License]) -> String {
    licenses
        .iter()
        .map(|license| license.spdx())
        .collect::<Vec<_>>()
        .join(" AND ")
}

/// The file.
fn render(found: &[Entry]) -> String {
    let every: BTreeSet<License> = found
        .iter()
        .flat_map(|entry| entry.licenses.iter().copied())
        .collect();
    let every: Vec<License> = every.into_iter().collect();
    let width = found
        .iter()
        .map(|entry| entry.krate.name.as_str().len())
        .chain([5])
        .max()
        .unwrap_or(5);
    let version_width = found
        .iter()
        .map(|entry| entry.krate.version.as_str().len())
        .chain([7])
        .max()
        .unwrap_or(7);
    let mut lines = vec![
        HEADER.to_owned(),
        format!("{}{}", render_formula::CREDITED_PREFIX, joined(&every)),
        String::new(),
        format!("Crates: {}", found.len()),
        String::new(),
        format!(
            "  {:width$}  {:version_width$}  License",
            "Crate", "Version"
        ),
    ];
    for entry in found {
        lines.push(format!(
            "  {:width$}  {:version_width$}  {}",
            entry.krate.name.as_str(),
            entry.krate.version.as_str(),
            joined(&entry.licenses)
        ));
    }
    lines.push(String::new());
    for entry in found {
        let krate = &entry.krate;
        lines.extend([
            RULE.to_owned(),
            String::new(),
            format!("{} {}", krate.name, krate.version),
            format!("License: {}", joined(&entry.licenses)),
        ]);
        if entry.written.trim() != joined(&entry.licenses) {
            lines.push(format!("Licensed by its authors as: {}", entry.written));
        }
        if entry
            .licenses
            .iter()
            .any(|license| license.asks_for_source_form())
        {
            lines.push(format!(
                "Source form: {}",
                crate_page(&krate.name, &krate.version)
            ));
        }
        for (label, text) in &entry.texts {
            lines.extend([
                String::new(),
                format!("--- {label} ---"),
                String::new(),
                text.trim_end_matches('\n').to_owned(),
            ]);
        }
        lines.push(String::new());
    }
    format!("{}\n", lines.join("\n").trim_end_matches('\n'))
}

/// One row of a notices file's crate table: the version, and the licenses it is credited under.
#[derive(Clone, Debug, PartialEq, Eq)]
struct TableRow {
    version: String,
    licenses: String,
}

/// The crate table of a notices file, by `NAME VERSION`.
fn table(text: &str) -> BTreeMap<String, TableRow> {
    let mut rows = BTreeMap::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("  Crate ") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.trim().is_empty() {
            break;
        }
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        if rest.starts_with(char::is_whitespace) {
            continue;
        }
        let Some((name, rest)) = rest.split_once(char::is_whitespace) else {
            continue;
        };
        let Some((version, licenses)) = rest.trim_start().split_once(char::is_whitespace) else {
            continue;
        };
        let licenses = licenses.trim();
        if licenses.is_empty() {
            continue;
        }
        rows.insert(
            format!("{name} {version}"),
            TableRow {
                version: version.to_owned(),
                licenses: licenses.to_owned(),
            },
        );
    }
    rows
}

/// Each crate added, removed, or changed between two notices files.
///
/// A crate can be credited at two versions at once, as when one dependency needs `rand` 0.8 and
/// another `rand` 0.9. So a version that appears is an upgrade, "now at", only where the same crate
/// also loses one; where every earlier version stays, it is a second version beside them.
pub fn differences(committed: &str, generated: &str) -> Vec<String> {
    let (before, after) = (table(committed), table(generated));
    let versions = |rows: &BTreeMap<String, TableRow>| {
        let mut found: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for key in rows.keys() {
            if let Some((name, version)) = key.split_once(' ') {
                found
                    .entry(name.to_owned())
                    .or_default()
                    .insert(version.to_owned());
            }
        }
        found
    };
    let (had, has) = (versions(&before), versions(&after));
    let names: BTreeSet<&String> = had.keys().chain(has.keys()).collect();
    let empty = BTreeSet::new();
    let credited_under = |key: &str| {
        after
            .get(key)
            .map(|row| row.licenses.clone())
            .unwrap_or_default()
    };
    let (mut added, mut removed) = (Vec::new(), Vec::new());
    for name in names {
        let old = had.get(name).unwrap_or(&empty);
        let new = has.get(name).unwrap_or(&empty);
        let gained: Vec<&String> = new.difference(old).collect();
        let lost: Vec<&String> = old.difference(new).collect();
        let kept: Vec<&str> = old.intersection(new).map(String::as_str).collect();
        for version in &gained {
            let key = format!("{name} {version}");
            let licenses = credited_under(&key);
            added.push(if !lost.is_empty() {
                format!("now at {key}, credited under {licenses}")
            } else if !kept.is_empty() {
                format!(
                    "added: {key} (beside {}), credited under {licenses}",
                    kept.join(", ")
                )
            } else {
                format!("added: {key}, credited under {licenses}")
            });
        }
        for version in &lost {
            let key = format!("{name} {version}");
            removed.push(if !gained.is_empty() {
                format!("no longer at {key}")
            } else if !kept.is_empty() {
                format!("removed: {key} (beside {}, which stays)", kept.join(", "))
            } else {
                format!("removed: {key}, which the lockfile no longer reaches")
            });
        }
    }
    let mut lines = added;
    lines.append(&mut removed);
    for (key, row) in &before {
        if let Some(now) = after.get(key) {
            if now != row {
                lines.push(format!(
                    "changed: {key} is now credited under {}, not {}",
                    now.licenses, row.licenses
                ));
            }
        }
    }
    let (committed_header, committed_rest) = split_header(committed);
    let (generated_header, generated_rest) = split_header(generated);
    if lines.is_empty() && committed_rest != generated_rest {
        lines.push(String::from(
            "the crates are the same, and a text differs: a crate's license file changed",
        ));
    }
    header_difference(committed_header, generated_header)
        .into_iter()
        .chain(lines)
        .collect()
}

/// A notices file's header, every line before its `Credited licenses:` line, and the rest from
/// that line on. A file with no such line is all header.
fn split_header(text: &str) -> (&str, &str) {
    let at = text
        .match_indices(render_formula::CREDITED_PREFIX)
        .find(|(index, _)| *index == 0 || text[..*index].ends_with('\n'))
        .map_or(text.len(), |(index, _)| index);
    text.split_at(at)
}

/// Where the committed header differs from the one the tool writes: the first line that differs,
/// as each has it.
fn header_difference(committed: &str, generated: &str) -> Option<String> {
    if committed == generated {
        return None;
    }
    let (had, has): (Vec<&str>, Vec<&str>) =
        (committed.lines().collect(), generated.lines().collect());
    let line = (0..had.len().max(has.len()))
        .find(|index| had.get(*index) != has.get(*index))
        .unwrap_or(0);
    let quoted = |found: Option<&&str>| {
        found.map_or_else(|| String::from("nothing"), |text| format!("{text:?}"))
    };
    Some(format!(
        "the header differs from the one the tool writes, first at its line {}: it reads {}, and \
         the tool writes {}",
        line + 1,
        quoted(had.get(line)),
        quoted(has.get(line))
    ))
}

/// The notices for every graph given, with the reviewed table's text and the vendored texts'
/// directory, or every fault found.
pub fn generate(
    graphs: &[Graph],
    table_text: &str,
    standard: &Path,
) -> Result<String, Vec<NoticeFault>> {
    let rows = clarifications::read(table_text).map_err(|fault| vec![NoticeFault::Table(fault)])?;
    let crates = credited(graphs).map_err(|fault| vec![fault])?;
    entries(crates, &rows, standard).map(|found| render(&found))
}

/// What the formula's template is held to: its text, and the license `htmlspec-assets` declares.
pub struct Formula<'a> {
    pub template: &'a str,
    pub assets_license: Option<&'a str>,
    pub targets: &'a [Target],
}

/// What `--check` reports: the committed notices against what the lockfile reaches, then the
/// formula's template against what the notices credit.
pub fn check(committed: &str, generated: &str, formula: &Formula<'_>) -> Report {
    if committed != generated {
        let mut err = vec![format!(
            "crate-notices: {OUTPUT} does not match what Cargo.lock reaches:"
        )];
        err.extend(
            differences(committed, generated)
                .into_iter()
                .map(|line| format!("  {line}")),
        );
        err.push(String::from(
            "crate-notices: run cargo xtask crate-notices to write it again, and read what changed",
        ));
        return Report {
            exit: Exit::Faults,
            out: Vec::new(),
            err,
        };
    }
    let counted = format!(
        "  ok  {OUTPUT} credits the {} crates the lockfile reaches",
        table(generated).len()
    );
    // The formula declares what the binary credits, and its own check runs only where the macOS
    // jobs do. Held here too, so a pull request on Linux alone meets a formula that drifted from
    // these notices, rather than the release that would ship it.
    let faults = render_formula::template_faults(
        formula.template,
        formula.assets_license,
        committed,
        formula.targets,
    );
    if faults.is_empty() {
        return Report {
            exit: Exit::Clean,
            out: vec![
                counted,
                String::from(
                    "  ok  the Homebrew formula's template declares what the binary credits",
                ),
            ],
            err: Vec::new(),
        };
    }
    let mut err = vec![counted];
    err.extend(
        faults
            .iter()
            .map(|fault| format!("crate-notices: the Homebrew formula's template: {fault}")),
    );
    Report {
        exit: Exit::Faults,
        out: Vec::new(),
        err,
    }
}

/// A fault that stops the tool before it can say anything of the notices, as the run ends.
fn could_not_ask(reason: &dyn fmt::Display) -> Report {
    Report::could_not_ask(&format!("crate-notices: {reason}"))
}

/// Writes or checks the notices of the workspace the current directory belongs to.
pub fn run(mode: Mode) -> ExitCode {
    report(mode).print()
}

fn report(mode: Mode) -> Report {
    let facts = match Facts::read() {
        Ok(facts) => facts,
        Err(fault) => return could_not_ask(&format!("xtask/release-facts.json {fault}")),
    };
    let targets = facts.targets(MacosJobs::Run);
    let directory = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(error) => {
            return could_not_ask(&format!(
                "cannot read the current directory: {}",
                error.kind()
            ))
        }
    };
    let mut graphs = Vec::new();
    for target in &targets {
        match Graph::read(&cargo(), &directory, *target) {
            Ok(graph) => graphs.push(graph),
            Err(fault) => return could_not_ask(&format!("for {target}: {fault}")),
        }
    }
    let Some(root) = graphs.first().map(|graph| graph.root.clone()) else {
        return could_not_ask(&"the release facts name no target");
    };
    let table_text = match std::fs::read_to_string(root.join(TABLE)) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return could_not_ask(&format!("cannot read {TABLE}: {}", error.kind())),
    };
    let generated = match generate(&graphs, &table_text, &root.join(STANDARD_TEXTS)) {
        Ok(text) => text,
        Err(faults) => {
            return Report {
                exit: Exit::Faults,
                out: Vec::new(),
                err: faults
                    .iter()
                    .map(|fault| format!("crate-notices: {fault}"))
                    .collect(),
            }
        }
    };
    let output = root.join(OUTPUT);
    match mode {
        Mode::Write => match std::fs::write(&output, &generated) {
            Ok(()) => Report {
                exit: Exit::Clean,
                out: vec![format!(
                    "wrote {OUTPUT}: {} crates, crediting {}",
                    table(&generated).len(),
                    render_formula::credited_licenses(&generated)
                        .unwrap_or_default()
                        .join(", ")
                )],
                err: Vec::new(),
            },
            Err(error) => could_not_ask(&format!("cannot write {OUTPUT}: {}", error.kind())),
        },
        Mode::Check => {
            let committed = match std::fs::read_to_string(&output) {
                Ok(text) => text,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(error) => {
                    return could_not_ask(&format!("cannot read {OUTPUT}: {}", error.kind()))
                }
            };
            let template = match std::fs::read_to_string(root.join(render_formula::TEMPLATE)) {
                Ok(text) => text,
                Err(error) => {
                    return could_not_ask(&format!(
                        "cannot read {}: {}",
                        render_formula::TEMPLATE,
                        error.kind()
                    ))
                }
            };
            let assets_license = graphs
                .first()
                .and_then(|graph| {
                    graph
                        .members()
                        .find(|(_, package)| package.name.as_str() == ASSETS_CRATE)
                })
                .and_then(|(_, package)| package.license.clone());
            check(
                &committed,
                &generated,
                &Formula {
                    template: &template,
                    assets_license: assets_license.as_deref(),
                    targets: &targets,
                },
            )
        }
    }
}
