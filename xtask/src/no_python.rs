//! `cargo xtask no-python`, the `no-python` stage of `scripts/check`.
//!
//! It reads the repository's tracked files and fails on any Python the repository holds or runs,
//! and on any program but the four permitted shell files, outside two excuses. The allow-list,
//! [`allowed::ALLOW_LIST`], names each path that stays, exactly, with its reason, and
//! [`allowed::ALLOWED_LINES`] the one line of the flake that asks whether the suite's sandbox
//! supplies an interpreter. The private list, [`private_list`], permits `publish/export`, the
//! fourth permitted shell file, which never crosses to the public repository. Nothing else excuses
//! a finding: the debt that pinned the Python the port had not removed yet is paid, and its list
//! and its reader are gone. Each finding names its path, its line where it has one, and the rule it
//! broke, and the stage ends by naming the requirement this change adds to toolchain-packaging at
//! its archival, "The repository holds and runs no Python".
//!
//! The six matchers are in [`matchers`], where the tracked files come from is in [`tracked`], and
//! [`judge`] sets one against the other.

mod allowed;
mod judge;
mod matchers;
mod private_list;
mod tracked;

use std::path::Path;
use std::process::ExitCode;

use private_list::List;
use tracked::{Entry, FileSet};

/// The requirement the stage enforces.
const REQUIREMENT: &str = "toolchain-packaging's \"The repository holds and runs no Python\"";

/// Runs the stage from the current directory, with `HTMLSPEC_SOURCE_TREE` as the process has it.
pub fn run() -> ExitCode {
    let directory = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(error) => {
            eprintln!("no-python: cannot read the current directory: {error}");
            return ExitCode::from(2);
        }
    };
    let report = check(std::env::var_os(tracked::SOURCE_TREE), &directory);
    for line in &report.out {
        println!("{line}");
    }
    for line in &report.err {
        eprintln!("{line}");
    }
    ExitCode::from(report.status.code())
}

/// How the stage ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Passed,
    /// Python, or a program, outside the lists, or a list that no longer matches the tree.
    Failed,
    /// The tracked files or a list could not be read, so nothing was judged.
    Unread,
}

impl Status {
    fn code(self) -> u8 {
        match self {
            Self::Passed => 0,
            Self::Failed => 1,
            Self::Unread => 2,
        }
    }
}

/// What the stage prints, and how it ends.
struct Report {
    status: Status,
    out: Vec<String>,
    err: Vec<String>,
}

impl Report {
    fn unread(lines: Vec<String>) -> Self {
        Self {
            status: Status::Unread,
            out: Vec::new(),
            err: lines,
        }
    }
}

fn check(source_tree: Option<std::ffi::OsString>, directory: &Path) -> Report {
    let files = match tracked::read(source_tree, directory) {
        Ok(files) => files,
        Err(fault) => return Report::unread(vec![format!("no-python: {fault}")]),
    };
    let private = match read_private_list(&files) {
        Ok(private) => private,
        Err(faults) => {
            let mut lines = vec![format!(
                "no-python: {} holds lines that are not entries it may hold:",
                private_list::PATH
            )];
            lines.extend(faults.iter().map(|fault| format!("  {fault}")));
            return Report::unread(lines);
        }
    };
    let verdict = judge::judge(&files, private.as_ref());
    let read = format!(
        "no-python: read {} tracked files from {}",
        files.count(),
        files.origin()
    );
    if verdict.faults.is_empty() {
        let permit = match private.as_ref().and_then(|list| list.permit.as_ref()) {
            Some(permit) => format!(
                "{}'s permit of {} excused {}",
                private_list::PATH,
                permit.path,
                counted(verdict.permitted, "finding", "findings")
            ),
            None if private.is_some() => format!("{} permits nothing", private_list::PATH),
            None => format!("no {} was read", private_list::PATH),
        };
        return Report {
            status: Status::Passed,
            out: vec![
                read,
                format!(
                    "no-python: passed. The allow-list's {} and {} excused {}, and {permit}.",
                    counted(allowed::ALLOW_LIST.len(), "path", "paths"),
                    counted(allowed::ALLOWED_LINES.len(), "line", "lines"),
                    counted(verdict.allowed, "finding", "findings"),
                ),
            ],
            err: Vec::new(),
        };
    }
    let mut err = Vec::new();
    let (found, stale): (Vec<_>, Vec<_>) =
        verdict.faults.iter().partition(|fault| fault.is_finding());
    if !found.is_empty() {
        err.push(format!(
            "Python outside the allow-list, which {REQUIREMENT} forbids:"
        ));
        err.extend(found.iter().map(|fault| format!("  {fault}")));
    }
    if !stale.is_empty() {
        err.push(String::from(
            "The allow-list and the private list no longer match the tracked files:",
        ));
        err.extend(stale.iter().map(|fault| format!("  {fault}")));
    }
    Report {
        status: Status::Failed,
        out: vec![read],
        err,
    }
}

/// `count` and the noun, singular or plural as the count needs.
fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// The private list, read where the tracked files hold it. A list that does not exist is not an
/// error: an exported tree holds no private list, and the file it would permit is absent too.
fn read_private_list(files: &FileSet) -> Result<Option<List>, Vec<private_list::ListFault>> {
    match files.get(private_list::PATH) {
        Some(Entry::File { content, .. }) => private_list::parse(content).map(Some),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::allowed::{Allowance, ALLOWED_LINES, ALLOW_LIST};
    use super::tracked::tests::Scratch;
    use super::*;

    /// Each allowed line's file, holding its line.
    fn write_allowed_lines(scratch: &Scratch) {
        for line in ALLOWED_LINES {
            scratch.write(
                line.path,
                &format!("{{\n  check = ''\n{}\n  '';\n}}\n", line.text),
            );
        }
    }

    /// A Git repository holding every allow-listed path and line, as this one does.
    fn repository(label: &str) -> Scratch {
        let scratch = Scratch::new(label);
        scratch.git(&["init", "-q"]);
        for entry in ALLOW_LIST {
            match entry.allowance {
                Allowance::PermittedShell { .. } => {
                    scratch.write(entry.path, "#!/usr/bin/env bash\n");
                    scratch.git(&["add", "--chmod=+x", entry.path]);
                }
                _ => {
                    scratch.write(entry.path, "#!/usr/bin/env python3\n");
                    scratch.git(&["add", entry.path]);
                }
            }
        }
        write_allowed_lines(&scratch);
        for line in ALLOWED_LINES {
            scratch.git(&["add", line.path]);
        }
        scratch
    }

    #[test]
    fn a_tree_holding_only_the_allow_list_passes() {
        let scratch = repository("passes");
        let report = check(None, &scratch.0);
        assert_eq!(report.status, Status::Passed, "{:#?}", report.err);
        assert!(
            report.out[0].contains("from Git's index of"),
            "{:#?}",
            report.out
        );
    }

    /// The proposal's account of a contributor's first try: untracked, the file is not read; staged,
    /// it fails the stage, named, with the requirement.
    #[test]
    fn an_untracked_python_file_passes_until_it_is_staged() {
        let scratch = repository("staged");
        scratch.write("scripts/tidy.py", "print()\n");
        assert_eq!(check(None, &scratch.0).status, Status::Passed);
        scratch.git(&["add", "scripts/tidy.py"]);
        let report = check(None, &scratch.0);
        assert_eq!(report.status, Status::Failed);
        assert_eq!(
            report.err,
            [
                format!("Python outside the allow-list, which {REQUIREMENT} forbids:"),
                String::from("  scripts/tidy.py: a Python file name"),
                String::from(
                    "  scripts/tidy.py: a file under scripts/, which holds scripts/check and \
                     scripts/install-hooks alone"
                ),
            ]
        );
    }

    #[test]
    fn a_private_list_with_a_line_that_is_no_entry_stops_the_stage() {
        let scratch = repository("bad-list");
        scratch.write("publish/no-python", "program\tpublish/tool\tsha256:00\t3\n");
        scratch.git(&["add", "publish/no-python"]);
        let report = check(None, &scratch.0);
        assert_eq!(report.status, Status::Unread);
        assert!(
            report.err[1].starts_with("  publish/no-python:1: a program entry pinned Python"),
            "{:#?}",
            report.err
        );
    }

    /// The debt is paid, so a list of it beside the tools is no excuse: it is a tracked file like
    /// any other, and the Python it would pin is a finding.
    #[test]
    fn a_debt_list_put_back_excuses_nothing() {
        let scratch = repository("debt-back");
        scratch.write("tools/tidy.py", "print()\n");
        scratch.write(
            "xtask/python-debt",
            "program\ttools/tidy.py\tsha256:00\t1\n",
        );
        scratch.git(&["add", "tools/tidy.py", "xtask/python-debt"]);
        let report = check(None, &scratch.0);
        assert_eq!(report.status, Status::Failed);
        assert_eq!(report.err[1], "  tools/tidy.py: a Python file name");
    }

    /// In the private repository, the private list permits the export's launcher, and the report
    /// says so; without the list the launcher is a program like any other.
    #[test]
    fn the_private_list_permits_the_launcher_and_the_report_says_so() {
        let scratch = repository("private");
        scratch.write("publish/export", "#!/usr/bin/env bash\nexec cargo run\n");
        scratch.git(&["add", "--chmod=+x", "publish/export"]);
        let report = check(None, &scratch.0);
        assert_eq!(report.status, Status::Failed);

        scratch.write(
            "publish/no-python",
            "shell\tpublish/export\tthe export's launcher\n",
        );
        scratch.git(&["add", "publish/no-python"]);
        let report = check(None, &scratch.0);
        assert_eq!(report.status, Status::Passed, "{:#?}", report.err);
        assert_eq!(
            report.out[1],
            "no-python: passed. The allow-list's 7 paths and 1 line excused 19 findings, and \
             publish/no-python's permit of publish/export excused 2 findings."
        );
    }

    #[test]
    fn nowhere_to_read_tracked_files_from_is_not_a_pass() {
        let scratch = Scratch::new("unread");
        let report = check(None, &scratch.0);
        assert_eq!(report.status, Status::Unread);
        assert!(
            report.err[0].contains("cannot tell tracked files"),
            "{:#?}",
            report.err
        );
    }

    /// Under the source tree the flake names, the stage reads every file of the tree, the file's
    /// own mode says whether it is executable, and the report says where it read.
    #[test]
    fn a_source_tree_is_read_and_named() {
        let scratch = Scratch::new("stage-source-tree");
        for entry in ALLOW_LIST {
            if let Allowance::PermittedShell { .. } = entry.allowance {
                scratch.write(entry.path, "#!/usr/bin/env bash\n");
                scratch.mark_executable(entry.path);
            } else {
                scratch.write(entry.path, "#!/usr/bin/env python3\n");
            }
        }
        write_allowed_lines(&scratch);
        let tree = || Some(scratch.0.clone().into_os_string());
        let report = check(tree(), Path::new("/"));
        assert_eq!(report.status, Status::Passed, "{:#?}", report.err);
        assert!(
            report.out[0].contains("the source tree HTMLSPEC_SOURCE_TREE names"),
            "{:#?}",
            report.out
        );

        scratch.write(
            ".github/workflows/check.yml",
            "steps:\n  - run: python3 x\n",
        );
        let report = check(tree(), Path::new("/"));
        assert_eq!(report.status, Status::Failed);
        assert_eq!(
            report.err[1],
            "  .github/workflows/check.yml:2: names python3"
        );
    }
}
