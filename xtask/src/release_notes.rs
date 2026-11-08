//! `cargo xtask release-notes VERSION`: print one version's entry from `CHANGELOG.md`, as the
//! notes of its GitHub release.
//!
//! The entry is the text under the `##` heading that names the version, up to the next `##` or `#`
//! heading. The heading itself is left out, because the release's title names the version. A
//! version no heading names, or one two headings name, is refused, and so is an empty entry,
//! because the notes would then be empty or a guess, and a release's notes are what a person reads
//! before installing it.
//!
//! Which versions a heading names is the one reading of a changelog's headings,
//! `runbook_release::heading_versions`, which the release check and archival read through too. So a
//! version this finds an entry for is a version the release check finds a heading for.
//!
//! It reads the changelog in the directory it runs in, the repository's root, and needs no cargo,
//! because the release workflow's release job, which runs it from a built binary, installs no
//! toolchain.
//!
//! Exit codes: 0 with the notes on standard output, 1 when the changelog holds no entry to give,
//! and 2 when the changelog cannot be read or the version is not a release's.

use std::fmt;
use std::process::ExitCode;

use htmlspec_cli::runbook_release::heading_versions;
use htmlspec_cli::version::{ProjectVersion, ReleaseVersion};

use crate::report::{Exit, Report};

/// The project's changelog, in the directory the tool runs in.
const CHANGELOG: &str = "CHANGELOG.md";

/// Why the changelog holds no notes for the version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoNotes {
    NoHeading {
        version: ReleaseVersion,
    },
    Headings {
        count: usize,
        version: ReleaseVersion,
    },
    Empty {
        version: ReleaseVersion,
    },
}

impl fmt::Display for NoNotes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeading { version } => {
                write!(formatter, "no {CHANGELOG} heading names {version}")
            }
            Self::Headings { count, version } => {
                write!(formatter, "{count} {CHANGELOG} headings name {version}")
            }
            Self::Empty { version } => {
                write!(formatter, "the {CHANGELOG} entry for {version} is empty")
            }
        }
    }
}

/// Whether `line` begins with `marker` and then a space of any kind.
fn opens_with(line: &str, marker: &str) -> bool {
    line.strip_prefix(marker)
        .and_then(|rest| rest.chars().next())
        .is_some_and(char::is_whitespace)
}

/// Whether `line` heads a version's entry: `##` and a space.
fn entry_heading(line: &str) -> bool {
    opens_with(line, "##")
}

/// Whether `line` ends an entry: another entry's heading, or the changelog's own `#` heading.
fn ends_an_entry(line: &str) -> bool {
    entry_heading(line) || opens_with(line, "#")
}

/// The notes for `version` in `changelog`: its entry, without its heading or the blank lines
/// around it, ending in one newline.
pub fn notes(changelog: &str, version: ReleaseVersion) -> Result<String, NoNotes> {
    let lines: Vec<&str> = changelog.lines().collect();
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            entry_heading(line)
                && heading_versions::<ProjectVersion>(line).contains(&version.project())
        })
        .map(|(index, _)| index)
        .collect();
    let start = match starts.as_slice() {
        [] => return Err(NoNotes::NoHeading { version }),
        [start] => *start,
        many => {
            return Err(NoNotes::Headings {
                count: many.len(),
                version,
            })
        }
    };
    let body: Vec<&str> = lines[start + 1..]
        .iter()
        .take_while(|line| !ends_an_entry(line))
        .copied()
        .collect();
    let joined = body.join("\n");
    let notes = joined.trim_matches('\n');
    if notes.trim().is_empty() {
        return Err(NoNotes::Empty { version });
    }
    Ok(format!("{notes}\n"))
}

/// The notes for `version` in `changelog`, read from where it is, as the run ends.
pub fn report(changelog: std::io::Result<String>, version: ReleaseVersion) -> Report {
    let text = match changelog {
        Ok(text) => text,
        Err(error) => {
            return Report::could_not_ask(&format!(
                "release-notes: cannot read {CHANGELOG}: {}",
                error.kind()
            ))
        }
    };
    match notes(&text, version) {
        Ok(notes) => Report {
            exit: Exit::Clean,
            out: notes.lines().map(str::to_owned).collect(),
            err: Vec::new(),
        },
        Err(none) => Report {
            exit: Exit::Faults,
            out: Vec::new(),
            err: vec![format!("release-notes: {none}")],
        },
    }
}

/// Prints the notes for `version` from the changelog in the current directory.
pub fn run(version: ReleaseVersion) -> ExitCode {
    report(std::fs::read_to_string(CHANGELOG), version).print()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(text: &str) -> ReleaseVersion {
        ReleaseVersion::parse(text).expect("a release version")
    }

    const CHANGELOG_TEXT: &str = "# Changelog\n\nWhat changed.\n\n\
        ## 1.3.1 — keep Python out\n\nThe patch.\n\n- **One.** A point.\n\n### A sub-heading\n\nMore.\n\n\
        ## 1.3.0 — see how far\n\nThe minor, after 1.3.1's prose names it.\n\n\
        ## 01.2.0\n\nWritten with a leading zero.\n\n\
        ## 1.1.0\n\n   \n\n\
        ## 1.0.0\n\nThe first.\n## 1.0.0 again\n\nTwice.\n";

    #[test]
    fn the_entry_is_the_text_under_its_heading_up_to_the_next() {
        assert_eq!(
            notes(CHANGELOG_TEXT, version("1.3.1")),
            Ok(String::from(
                "The patch.\n\n- **One.** A point.\n\n### A sub-heading\n\nMore.\n"
            ))
        );
        assert_eq!(
            notes(CHANGELOG_TEXT, version("1.3.0")),
            Ok(String::from("The minor, after 1.3.1's prose names it.\n"))
        );
    }

    /// The one reading of a heading reads `01.2.0` as 1.2.0, as the release check always did. The
    /// Python release notes compared a heading's text with their argument, so they found none.
    #[test]
    fn a_heading_names_its_version_as_the_one_reading_reads_it() {
        assert_eq!(
            notes(CHANGELOG_TEXT, version("1.2.0")),
            Ok(String::from("Written with a leading zero.\n"))
        );
    }

    #[test]
    fn a_version_with_no_entry_one_with_two_or_an_empty_one_is_refused() {
        assert_eq!(
            notes(CHANGELOG_TEXT, version("0.9.0")),
            Err(NoNotes::NoHeading {
                version: version("0.9.0")
            })
        );
        assert_eq!(
            notes(CHANGELOG_TEXT, version("1.0.0")),
            Err(NoNotes::Headings {
                count: 2,
                version: version("1.0.0")
            })
        );
        assert_eq!(
            notes(CHANGELOG_TEXT, version("1.1.0")),
            Err(NoNotes::Empty {
                version: version("1.1.0")
            })
        );
    }

    /// The notes on standard output, exit 0; a refusal on standard error, exit 1; a changelog that
    /// cannot be read, exit 2.
    #[test]
    fn each_ending_has_its_exit_code() {
        let printed = report(Ok(String::from(CHANGELOG_TEXT)), version("1.3.0"));
        assert_eq!(
            printed,
            Report {
                exit: Exit::Clean,
                out: vec![String::from("The minor, after 1.3.1's prose names it.")],
                err: Vec::new(),
            }
        );
        assert_eq!(printed.exit.code(), 0);

        let refused = report(Ok(String::from(CHANGELOG_TEXT)), version("0.9.0"));
        assert_eq!(
            refused,
            Report {
                exit: Exit::Faults,
                out: Vec::new(),
                err: vec![String::from(
                    "release-notes: no CHANGELOG.md heading names 0.9.0"
                )],
            }
        );
        assert_eq!(refused.exit.code(), 1);

        let unreadable = report(
            Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
            version("1.3.0"),
        );
        assert_eq!(unreadable.exit.code(), 2);
        assert_eq!(
            unreadable.err,
            vec![String::from(
                "release-notes: cannot read CHANGELOG.md: entity not found"
            )]
        );
    }
}
