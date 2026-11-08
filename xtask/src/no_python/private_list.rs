//! The private list, `publish/no-python`: the fourth permitted shell file, which never crosses.
//!
//! `publish/export` is one of the four shell files the repository may run, and it never crosses to
//! the public repository. The allow-list crosses, and every path it names must exist, so the
//! export's entry cannot be there: an exported tree holds no `publish/`. It is in this list instead,
//! which never crosses either and is read where it exists. An exported tree holds neither the list
//! nor the file it names, and in the private repository a list that is missing while the file is
//! present leaves the file a plain finding, so the stage fails rather than passing.
//!
//! The list once also pinned the Python the port had not removed yet, each program by its digest
//! and each line by its text, beside a public list of the same form. The port removed all of it, so
//! a permit is the only entry there is. Each is one line of fields separated by tabs, and a `#` line
//! and a blank line are ignored:
//!
//! ```text
//! shell<TAB>publish/export<TAB>REASON   the fourth permitted shell file, and no other
//! ```

use std::fmt;

use super::tracked::{PathFault, RepoPath};

/// Where the list is kept, from the repository's root.
pub const PATH: &str = "publish/no-python";

/// The one shell file the list may permit.
pub const PRIVATE_SHELL: &str = "publish/export";

/// A permitted shell file that never crosses, with why it stays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permit {
    pub path: RepoPath,
    pub reason: String,
}

/// The list, read: its one permit, where it holds one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct List {
    pub permit: Option<Permit>,
}

/// A line of the list that is not an entry it may hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListFault {
    pub line: usize,
    pub why: EntryFault,
}

/// What is wrong with a line of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryFault {
    /// Fewer than the three fields an entry has: its kind, its path, and its reason.
    TooFewFields,
    /// A `program` or `line` entry, which pinned the Python the port has removed.
    Debt(String),
    /// The first field is not `shell`.
    UnknownKind(String),
    /// The path, or the reason after it, is not UTF-8.
    NotUtf8,
    /// The path is not one path from the repository's root.
    Path { path: String, why: PathFault },
    /// The entry names another path than `publish/export`.
    NotTheExport(RepoPath),
    /// A second permit, which the one shell file cannot need.
    AlreadyPermitted { at: usize },
}

impl fmt::Display for EntryFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewFields => {
                f.write_str("an entry is shell, a tab, a path, a tab, and its reason")
            }
            Self::Debt(kind) => write!(
                f,
                "a {kind} entry pinned Python the port had not removed yet, and the port removed \
                 it all; this list permits {PRIVATE_SHELL} and holds nothing else"
            ),
            Self::UnknownKind(kind) => {
                write!(f, "{kind:?} is not an entry's kind: the one kind is shell")
            }
            Self::NotUtf8 => f.write_str("the entry is not UTF-8"),
            Self::Path { path, why } => write!(f, "the path {path:?}: {why}"),
            Self::NotTheExport(path) => write!(
                f,
                "{path}: this list permits one shell file, {PRIVATE_SHELL}, and no other"
            ),
            Self::AlreadyPermitted { at } => {
                write!(f, "{PRIVATE_SHELL} already has a shell entry at line {at}")
            }
        }
    }
}

impl fmt::Display for ListFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{PATH}:{}: {}", self.line, self.why)
    }
}

/// Reads the list, refusing every line that is not an entry it may hold.
pub fn parse(content: &[u8]) -> Result<List, Vec<ListFault>> {
    let mut list = List::default();
    let mut permitted_at = None;
    let mut faults = Vec::new();
    for (index, line) in content.split(|byte| *byte == b'\n').enumerate() {
        let number = index + 1;
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() || line.starts_with(b"#") {
            continue;
        }
        let refused = match (permit(line), permitted_at) {
            (Err(why), _) => Some(why),
            (Ok(_), Some(at)) => Some(EntryFault::AlreadyPermitted { at }),
            (Ok(permit), None) => {
                list.permit = Some(permit);
                permitted_at = Some(number);
                None
            }
        };
        if let Some(why) = refused {
            faults.push(ListFault { line: number, why });
        }
    }
    if faults.is_empty() {
        Ok(list)
    } else {
        Err(faults)
    }
}

fn permit(line: &[u8]) -> Result<Permit, EntryFault> {
    let mut fields = line.splitn(3, |byte| *byte == b'\t');
    let kind = fields.next().unwrap_or_default();
    let (Some(path), Some(reason)) = (fields.next(), fields.next()) else {
        return Err(EntryFault::TooFewFields);
    };
    match kind {
        b"shell" => {}
        b"program" | b"line" => {
            return Err(EntryFault::Debt(String::from_utf8_lossy(kind).into_owned()))
        }
        other => {
            return Err(EntryFault::UnknownKind(
                String::from_utf8_lossy(other).into_owned(),
            ))
        }
    }
    let path = std::str::from_utf8(path).map_err(|_| EntryFault::NotUtf8)?;
    let path = RepoPath::parse(path).map_err(|why| EntryFault::Path {
        path: path.to_owned(),
        why,
    })?;
    if path.as_str() != PRIVATE_SHELL {
        return Err(EntryFault::NotTheExport(path));
    }
    let reason = std::str::from_utf8(reason).map_err(|_| EntryFault::NotUtf8)?;
    Ok(Permit {
        path,
        reason: reason.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn faults(text: &str) -> Vec<String> {
        parse(text.as_bytes())
            .expect_err("the list is refused")
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn the_list_permits_the_export() {
        let list = parse(b"# a comment\n\nshell\tpublish/export\tthe export's launcher\n")
            .expect("the list reads");
        assert_eq!(
            list.permit,
            Some(Permit {
                path: RepoPath::parse("publish/export").expect("a path"),
                reason: String::from("the export's launcher"),
            })
        );
        assert_eq!(parse(b"# nothing\n"), Ok(List::default()));
    }

    /// The reviewer's seventh way around the ban, which the port's debt left open while it lasted:
    /// the private list permitting a fifth shell file, under `publish/` or anywhere else.
    #[test]
    fn the_list_permits_the_export_alone_and_once() {
        for path in ["publish/deploy", "scripts/tidy", ".githooks/post-commit"] {
            let said = faults(&format!("shell\t{path}\tx\n"));
            assert_eq!(
                said,
                [format!(
                    "publish/no-python:1: {path}: this list permits one shell file, \
                     publish/export, and no other"
                )]
            );
        }
        assert_eq!(
            faults("shell\tpublish/export\tx\nshell\tpublish/export\ty\n"),
            ["publish/no-python:2: publish/export already has a shell entry at line 1"]
        );
    }

    /// The debt is paid, so an entry that would pin Python is refused, naming why, rather than read.
    #[test]
    fn an_entry_that_pins_python_is_refused() {
        let digest = format!("sha256:{}", "0".repeat(64));
        let said = faults(&format!(
            "program\tpublish/scan\t{digest}\t12\nline\t.github/workflows/publish.yml\tpython3 x\n"
        ));
        assert_eq!(said.len(), 2, "{said:#?}");
        assert!(
            said[0].starts_with("publish/no-python:1: a program entry pinned Python"),
            "{said:#?}"
        );
        assert!(
            said[1].starts_with("publish/no-python:2: a line entry pinned Python"),
            "{said:#?}"
        );
    }

    #[test]
    fn a_malformed_entry_is_refused_by_its_line() {
        let said = faults("shell\tpublish/export\nallow\tpublish/export\tx\nshell\t/abs\tx\n");
        assert_eq!(
            said,
            [
                "publish/no-python:1: an entry is shell, a tab, a path, a tab, and its reason",
                "publish/no-python:2: \"allow\" is not an entry's kind: the one kind is shell",
                "publish/no-python:3: the path \"/abs\": it is absolute",
            ]
        );
    }
}
