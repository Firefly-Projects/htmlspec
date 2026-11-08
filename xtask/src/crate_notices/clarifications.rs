//! `license-clarifications.tsv`: the reviewed choices for crates whose license text the file-name
//! rule cannot find, read into typed rows.
//!
//! Lines that are blank or begin with `#` are comments. The first other line is the header, and
//! every line after it is a row of tab-separated cells: the crate, its exact version, the license it
//! is credited under, the sources of the text, who reviewed it, and, optionally, a copyright line.
//! A row names an exact version, so an upgrade asks for the review again, and its license must be
//! one this project accepts, since a row for any other could never be used.
//!
//! A source is a file the crate ships, or a standard text vendored under `licenses/` beside the
//! table, and a row joins several with ` + `. Never an absolute path, a path that climbs out with
//! `..`, or one written with a backslash: those would read a file the review never saw.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use super::license::License;
use crate::workspace::{CrateName, PackageVersion};

/// The table's file name, as its faults name it.
pub const TABLE: &str = "license-clarifications.tsv";

/// The header's cells, in order.
pub const COLUMNS: [&str; 6] = [
    "crate",
    "version",
    "license",
    "source",
    "reviewer",
    "copyright",
];

/// The cells a row must fill. `copyright` may be left empty, or left off the end of the row.
const REQUIRED_CELLS: usize = 5;

/// Where the vendored standard texts are named from.
const VENDORED: &str = "licenses/";

/// Where one part of a row's text comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// A file the crate ships, at this path within the crate.
    Shipped { written: String },
    /// A standard text vendored under `licenses/`, at this path within that directory.
    Vendored { written: String, within: PathBuf },
}

/// A reviewed row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub sources: Vec<Source>,
    pub reviewer: String,
    /// The copyright line a vendored text carries, where the reviewer read one.
    pub copyright: Option<String>,
}

/// Which crate, at which version, under which license, a row is for.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RowKey {
    pub name: CrateName,
    pub version: PackageVersion,
    pub license: License,
}

impl RowKey {
    /// The order the Python table's keys sorted in, by their text: name, version, and license.
    pub fn sort_text(&self) -> (String, String, &'static str) {
        (
            self.name.as_str().to_owned(),
            self.version.as_str().to_owned(),
            self.license.spdx(),
        )
    }
}

/// Why the table cannot be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableFault {
    /// The first line that is not a comment is not the header.
    Header { line: usize },
    /// A row with too few or too many cells, or a required one left empty.
    Cells { line: usize },
    /// A crate name no crate can have.
    Name { line: usize, name: String },
    /// A version that is not one word.
    Version { line: usize, version: String },
    /// A license this project does not accept, which no crate is credited under.
    License { line: usize, license: String },
    /// A source that is not a path within the crate or under `licenses/`.
    Source {
        line: usize,
        source: String,
        name: String,
    },
    /// A copyright line on a row whose sources name no vendored text.
    Copyright { line: usize, name: String },
    /// Two rows for one crate, version, and license.
    Twice(RowKey),
}

impl fmt::Display for TableFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header { line } => write!(
                formatter,
                "{TABLE} line {line} should be the header {:?}",
                COLUMNS.join("\t")
            ),
            Self::Cells { line } => write!(
                formatter,
                "{TABLE} line {line} needs tab-separated cells {}, each filled, and may add {}",
                COLUMNS[..REQUIRED_CELLS].join(", "),
                COLUMNS[REQUIRED_CELLS]
            ),
            Self::Name { line, name } => write!(
                formatter,
                "{TABLE} line {line} names the crate {name:?}, which no crate can be named"
            ),
            Self::Version { line, version } => write!(
                formatter,
                "{TABLE} line {line} names the version {version:?}, which is not one word"
            ),
            Self::License { line, license } => write!(
                formatter,
                "{TABLE} line {line} names the license {license:?}, which is none this project \
                 accepts, so no crate is credited under it: {}",
                License::accepted()
            ),
            Self::Source { line, source, name } => write!(
                formatter,
                "{TABLE} line {line} names the source {source:?} for {name}, which is not a path \
                 within the crate or under licenses/: a source is a file the crate ships or a \
                 vendored standard text, never a file elsewhere"
            ),
            Self::Copyright { line, name } => write!(
                formatter,
                "{TABLE} line {line} gives {name} a copyright line, which only a vendored \
                 standard text carries, and its source names none"
            ),
            Self::Twice(key) => write!(
                formatter,
                "{TABLE} names {} {} {} twice",
                key.name, key.version, key.license
            ),
        }
    }
}

/// Whether `source` is a relative path that stays within the directory it is read from.
fn stays_within(source: &str) -> bool {
    !source.is_empty()
        && !source.contains('\\')
        && Path::new(source)
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

/// One row's sources, or the source that is not one.
fn sources(cell: &str) -> Result<Vec<Source>, String> {
    cell.split(" + ")
        .map(str::trim)
        .map(|part| {
            if !stays_within(part) {
                return Err(part.to_owned());
            }
            Ok(match part.strip_prefix(VENDORED) {
                Some(within) => Source::Vendored {
                    written: part.to_owned(),
                    within: PathBuf::from(within),
                },
                None => Source::Shipped {
                    written: part.to_owned(),
                },
            })
        })
        .collect()
}

/// The table's rows, by the crate, version, and license each is for.
pub fn read(text: &str) -> Result<BTreeMap<RowKey, Row>, TableFault> {
    let mut rows = BTreeMap::new();
    let mut header_seen = false;
    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cells: Vec<&str> = line.split('\t').collect();
        if !header_seen {
            if cells != COLUMNS {
                return Err(TableFault::Header { line: line_number });
            }
            header_seen = true;
            continue;
        }
        let cells: Vec<&str> = cells.into_iter().map(str::trim).collect();
        let [name, version, license, source, reviewer, rest @ ..] = cells.as_slice() else {
            return Err(TableFault::Cells { line: line_number });
        };
        if rest.len() > 1
            || [name, version, license, source, reviewer]
                .iter()
                .any(|cell| cell.is_empty())
        {
            return Err(TableFault::Cells { line: line_number });
        }
        let copyright = rest
            .first()
            .filter(|cell| !cell.is_empty())
            .map(|cell| (*cell).to_owned());
        let crate_name = CrateName::parse(name).ok_or_else(|| TableFault::Name {
            line: line_number,
            name: (*name).to_owned(),
        })?;
        let crate_version = PackageVersion::parse(version).ok_or_else(|| TableFault::Version {
            line: line_number,
            version: (*version).to_owned(),
        })?;
        let accepted = License::from_spdx(license).ok_or_else(|| TableFault::License {
            line: line_number,
            license: (*license).to_owned(),
        })?;
        let sources = sources(source).map_err(|source| TableFault::Source {
            line: line_number,
            source,
            name: (*name).to_owned(),
        })?;
        if copyright.is_some()
            && !sources
                .iter()
                .any(|source| matches!(source, Source::Vendored { .. }))
        {
            return Err(TableFault::Copyright {
                line: line_number,
                name: (*name).to_owned(),
            });
        }
        let key = RowKey {
            name: crate_name,
            version: crate_version,
            license: accepted,
        };
        if rows.contains_key(&key) {
            return Err(TableFault::Twice(key));
        }
        rows.insert(
            key,
            Row {
                sources,
                reviewer: (*reviewer).to_owned(),
                copyright,
            },
        );
    }
    Ok(rows)
}
