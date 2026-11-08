//! The verdict: every finding the allow-list and the private list do not excuse, and every entry of
//! either that no longer matches the tree.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::allowed::{permitted_shell_excuses, Allowance, AllowedLine, ALLOWED_LINES, ALLOW_LIST};
use super::matchers::{findings, rules_handed, Context, Finding};
use super::private_list::{self, List};
use super::tracked::{Entry, FileSet, RepoPath};

/// One reason the stage fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A finding nothing excuses.
    Unexcused(Finding),
    /// An allow-listed path the tracked files do not hold.
    AllowListedAbsent {
        path: &'static str,
        allowance: Allowance,
    },
    /// An allowed line its file no longer holds, or a file it names that is gone.
    AllowedLineAbsent(AllowedLine),
    /// The private list's permit of a path the tracked files do not hold as a file.
    PermittedAbsent { path: RepoPath },
}

impl Fault {
    fn path(&self) -> &str {
        match self {
            Self::Unexcused(finding) => finding.path().as_str(),
            Self::AllowListedAbsent { path, .. } => path,
            Self::AllowedLineAbsent(line) => line.path,
            Self::PermittedAbsent { path } => path.as_str(),
        }
    }

    /// Whether the fault is Python, or a program, outside the lists, rather than a list that no
    /// longer matches the tree.
    pub fn is_finding(&self) -> bool {
        matches!(self, Self::Unexcused(_))
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexcused(finding) => write!(f, "{finding}"),
            Self::AllowListedAbsent { path, allowance } => write!(
                f,
                "{path}: allow-listed, as {}, and absent; remove its entry from \
                 xtask/src/no_python/allowed.rs",
                allowance.reason()
            ),
            Self::AllowedLineAbsent(line) => write!(
                f,
                "{}: allowed one line, {}, and the file no longer holds it: {:?}; change its \
                 entry in xtask/src/no_python/allowed.rs with the line",
                line.path, line.reason, line.text
            ),
            Self::PermittedAbsent { path } => write!(
                f,
                "{path}: {} permits it, and the tracked files hold no such file; remove its entry",
                private_list::PATH
            ),
        }
    }
}

/// What the stage found.
#[derive(Debug, Default)]
pub struct Verdict {
    pub faults: Vec<Fault>,
    /// Findings the allow-list's paths and lines excused.
    pub allowed: usize,
    /// Findings the private list's permit excused.
    pub permitted: usize,
}

/// Judges every tracked file against the allow-list and the private list, where it was read.
pub fn judge(files: &FileSet, private: Option<&List>) -> Verdict {
    judge_against(files, private, &ALLOWED_LINES)
}

/// [`judge`], with the allowed lines given, so a test can allow a line on which a matcher finds
/// more than the name the allowance excuses.
fn judge_against(
    files: &FileSet,
    private: Option<&List>,
    allowed_lines: &[AllowedLine],
) -> Verdict {
    let mut verdict = Verdict::default();
    let allowances: BTreeMap<&str, Allowance> = ALLOW_LIST
        .iter()
        .map(|entry| (entry.path, entry.allowance))
        .collect();
    for entry in ALLOW_LIST {
        if files.get(entry.path).is_none() {
            verdict.faults.push(Fault::AllowListedAbsent {
                path: entry.path,
                allowance: entry.allowance,
            });
        }
    }

    // The private list's one permit, where the list was read and the file it names is tracked.
    let mut shells: BTreeSet<&RepoPath> = BTreeSet::new();
    if let Some(permit) = private.and_then(|list| list.permit.as_ref()) {
        if matches!(files.get(permit.path.as_str()), Some(Entry::File { .. })) {
            shells.insert(&permit.path);
        } else {
            verdict.faults.push(Fault::PermittedAbsent {
                path: permit.path.clone(),
            });
        }
    }

    // The shell files the repository may run: the allow-list's, and the private list's where it
    // was read. Each is read line by line whatever its first line, and must be bash.
    let permitted: Vec<&str> = ALLOW_LIST
        .iter()
        .filter(|entry| matches!(entry.allowance, Allowance::PermittedShell { .. }))
        .map(|entry| entry.path)
        .chain(shells.iter().map(|path| path.as_str()))
        .collect();

    // The files handed to `make -f` or `just --justfile`, read as files that run. A file read so
    // may hand on another, so this runs until no new one is found; each round adds one or ends.
    let mut rules: BTreeSet<String> = BTreeSet::new();
    loop {
        let context = Context {
            permitted: &permitted,
            files,
            rules: &rules,
        };
        let handed: BTreeSet<String> = files
            .iter()
            .flat_map(|(path, entry)| rules_handed(path, entry, &context))
            .filter(|path| !rules.contains(path))
            .collect();
        if handed.is_empty() {
            break;
        }
        rules.extend(handed);
    }
    let context = Context {
        permitted: &permitted,
        files,
        rules: &rules,
    };

    // Each allowed line excuses one line of its file, and is struck off when it has.
    let mut lines_unmet: Vec<AllowedLine> = allowed_lines.to_vec();
    for (path, entry) in files.iter() {
        for finding in findings(path, entry, &context) {
            let excused = match &finding {
                Finding::File { rule, .. } => {
                    if allowances
                        .get(path.as_str())
                        .is_some_and(|allowance| allowance.excuses(rule))
                    {
                        verdict.allowed += 1;
                        true
                    } else if shells.contains(path) && permitted_shell_excuses(rule) {
                        verdict.permitted += 1;
                        true
                    } else {
                        false
                    }
                }
                Finding::Line { text, rule, .. } => {
                    match lines_unmet.iter().position(|line| {
                        line.path == path.as_str()
                            && line.text.as_bytes() == text.as_slice()
                            && line.excuses(rule)
                    }) {
                        Some(index) => {
                            lines_unmet.swap_remove(index);
                            verdict.allowed += 1;
                            true
                        }
                        None => false,
                    }
                }
            };
            if !excused {
                verdict.faults.push(Fault::Unexcused(finding));
            }
        }
    }
    verdict
        .faults
        .extend(lines_unmet.into_iter().map(Fault::AllowedLineAbsent));
    verdict
        .faults
        .sort_by(|a, b| (!a.is_finding(), a.path()).cmp(&(!b.is_finding(), b.path())));
    verdict
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::private_list::parse;
    use super::super::tracked::Origin;
    use super::*;

    const PROGRAM: &str = "#!/usr/bin/env python3\nprint('a tool')\n";

    /// The flake as the allowed line needs it: a build phase that asks for the interpreter.
    fn flake_asking() -> String {
        format!(
            "{{\n  buildPhase = ''\n{}\n    exit 1\n  fi\n  '';\n}}\n",
            ALLOWED_LINES[0].text
        )
    }

    /// A tree holding every allow-listed path, as the repository does: the data files as Python
    /// that nothing runs, the shell files as executable bash, and the flake with its allowed line.
    fn tree(extra: &[(&str, bool, &str)]) -> FileSet {
        let mut files = BTreeMap::new();
        for entry in ALLOW_LIST {
            let shell = matches!(entry.allowance, Allowance::PermittedShell { .. });
            let content = if shell {
                "#!/usr/bin/env bash\n"
            } else {
                PROGRAM
            };
            files.insert(
                RepoPath::parse(entry.path).expect("a path"),
                Entry::File {
                    executable: shell,
                    content: content.as_bytes().to_vec(),
                },
            );
        }
        files.insert(
            RepoPath::parse("flake.nix").expect("a path"),
            Entry::File {
                executable: false,
                content: flake_asking().into_bytes(),
            },
        );
        for (path, executable, content) in extra {
            files.insert(
                RepoPath::parse(path).expect("a path"),
                Entry::File {
                    executable: *executable,
                    content: content.as_bytes().to_vec(),
                },
            );
        }
        FileSet::new(Origin::Index(PathBuf::from("/repository")), files)
    }

    fn private(text: &str) -> List {
        parse(text.as_bytes()).expect("the list reads")
    }

    fn faults(verdict: &Verdict) -> Vec<String> {
        verdict.faults.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn the_allow_list_alone_passes_a_tree_that_holds_it() {
        let verdict = judge(
            &tree(&[("README.md", false, "python3 -m http.server\n")]),
            None,
        );
        assert_eq!(faults(&verdict), Vec::<String>::new());
        // Four data files are each a name and a first line; three shell files a first line and a
        // mode; and the flake's one line.
        assert_eq!(verdict.allowed, 4 * 3 + 3 * 2 + 1);
        assert_eq!(verdict.permitted, 0);
    }

    #[test]
    fn a_python_file_outside_the_lists_is_named() {
        let verdict = judge(&tree(&[("scripts/tidy.py", false, "print()\n")]), None);
        assert_eq!(
            faults(&verdict),
            [
                "scripts/tidy.py: a Python file name",
                "scripts/tidy.py: a file under scripts/, which holds scripts/check and scripts/install-hooks alone",
            ]
        );
    }

    #[test]
    fn an_allow_listed_path_that_is_gone_fails_and_says_so() {
        let mut files = BTreeMap::new();
        for (path, entry) in tree(&[]).iter() {
            if !path.as_str().ends_with("prove-sequence.py") {
                files.insert(path.clone(), entry.clone());
            }
        }
        let verdict = judge(&FileSet::new(Origin::Index(PathBuf::new()), files), None);
        let said = faults(&verdict);
        // In path order: the fixture's copy, then the archive's own.
        assert_eq!(said.len(), 2, "{said:#?}");
        assert!(
            said[0].contains("allow-listed, as a survey fixture"),
            "{said:#?}"
        );
        assert!(
            said[1].contains("allow-listed, as archived evidence"),
            "{said:#?}"
        );
    }

    #[test]
    fn a_data_file_made_executable_is_a_program() {
        let path = ALLOW_LIST[0].path;
        let verdict = judge(&tree(&[(path, true, PROGRAM)]), None);
        assert_eq!(
            faults(&verdict),
            [format!(
                "{path}: carries the executable bit, and it is none of the four permitted shell files"
            )]
        );
    }

    #[test]
    fn a_shell_file_outside_the_four_is_refused_whatever_its_language() {
        let verdict = judge(
            &tree(&[("scripts/tidy", true, "#!/usr/bin/env bash\n")]),
            None,
        );
        assert_eq!(
            faults(&verdict),
            [
                "scripts/tidy: a file under scripts/, which holds scripts/check and scripts/install-hooks alone",
                "scripts/tidy:1: begins with #!, and it is none of the four permitted shell files",
                "scripts/tidy: carries the executable bit, and it is none of the four permitted shell files",
            ]
        );
    }

    #[test]
    fn a_permitted_shell_file_is_still_read_for_an_interpreter() {
        let verdict = judge(
            &tree(&[("scripts/check", true, "#!/usr/bin/env bash\npython3 x\n")]),
            None,
        );
        assert_eq!(faults(&verdict), ["scripts/check:2: names python3"]);
    }

    /// The flake's one allowed line is excused at exactly its text, once, and nowhere else.
    #[test]
    fn the_allowed_line_is_excused_once_and_only_as_it_stands() {
        let asking = ALLOWED_LINES[0].text;

        // A second copy of the line, and the line with one byte changed, are each a finding.
        let twice = flake_asking().replace(asking, &format!("{asking}\n{asking}"));
        assert_eq!(
            faults(&judge(&tree(&[("flake.nix", false, &twice)]), None)),
            ["flake.nix:4: names python3"]
        );
        let changed = flake_asking().replace(asking, &asking.replace(" || ", " && "));
        let said = faults(&judge(&tree(&[("flake.nix", false, &changed)]), None));
        assert_eq!(said.len(), 2, "{said:#?}");
        assert_eq!(said[0], "flake.nix:3: names python3");
        assert!(
            said[1].starts_with("flake.nix: allowed one line, the flake's question whether"),
            "{said:#?}"
        );

        // The same text in another file that runs is a finding there, and the flake's is unmet.
        let elsewhere = format!("#!/usr/bin/env bash\n{asking}\n  exit 1\nfi\n");
        let said = faults(&judge(
            &tree(&[
                ("flake.nix", false, "{ }\n"),
                ("scripts/check", true, &elsewhere),
            ]),
            None,
        ));
        assert_eq!(said.len(), 2, "{said:#?}");
        assert_eq!(said[0], "scripts/check:2: names python3");
        assert!(said[1].contains("the file no longer holds it"), "{said:#?}");
    }

    /// Another finding on an allowed line's exact text is not excused: the allowance takes the name,
    /// and a handoff on the same line, to a tracked file none of the four, is still refused. The
    /// flake's own line hands nothing off, so the test allows a line that does.
    #[test]
    fn another_finding_on_an_allowed_line_is_not_excused() {
        let text = "    bash tools/run python3";
        let allowed = [AllowedLine {
            path: "flake.nix",
            text,
            reason: "a line the test allows",
        }];
        let flake = format!("{{\n  buildPhase = ''\n{text}\n  '';\n}}\n");
        let files = tree(&[
            ("flake.nix", false, &flake),
            ("tools/run", false, "echo run\n"),
        ]);
        let verdict = judge_against(&files, None, &allowed);
        assert_eq!(
            faults(&verdict),
            ["flake.nix:3: runs tools/run as a script, and it is none of the four permitted shell files"]
        );
        // Without the allowance the name is refused too, so the allowance took that one finding.
        let said = faults(&judge_against(&files, None, &[]));
        assert_eq!(
            said,
            [
                "flake.nix:3: names python3",
                "flake.nix:3: runs tools/run as a script, and it is none of the four permitted shell files",
            ]
        );

        // An allowed text that names nothing and hands a file off excuses nothing, because the
        // handoff is no name: the finding stands, and the allowance is unmet.
        let handoff = "    bash tools/run";
        let allowed = [AllowedLine {
            path: "flake.nix",
            text: handoff,
            reason: "a line the test allows",
        }];
        let flake = format!("{{\n  buildPhase = ''\n{handoff}\n  '';\n}}\n");
        let files = tree(&[
            ("flake.nix", false, &flake),
            ("tools/run", false, "echo run\n"),
        ]);
        let said = faults(&judge_against(&files, None, &allowed));
        assert_eq!(said.len(), 2, "{said:#?}");
        assert_eq!(
            said[0],
            "flake.nix:3: runs tools/run as a script, and it is none of the four permitted shell files"
        );
        assert!(said[1].contains("the file no longer holds it"), "{said:#?}");
    }

    /// The line runs nothing but the question, so a handoff written on it is still a finding: the
    /// allowance excuses the names it asks about, and no other rule.
    #[test]
    fn the_allowed_line_excuses_a_name_and_nothing_else() {
        let line = ALLOWED_LINES[0];
        assert!(line.excuses(&super::super::matchers::LineRule::Names {
            word: String::from("python3")
        }));
        assert!(
            !line.excuses(&super::super::matchers::LineRule::RunsScript {
                target: String::from("tools/x")
            })
        );
    }

    /// The private list permits its shell file as a program and nothing more, and a missing private
    /// list leaves the file it would have permitted as plain findings.
    #[test]
    fn the_private_list_is_read_where_it_exists() {
        let launcher = (
            "publish/export",
            true,
            "#!/usr/bin/env bash\nexec cargo run\n",
        );
        let permit = private("shell\tpublish/export\tthe export's launcher\n");
        let verdict = judge(&tree(&[launcher]), Some(&permit));
        assert_eq!(faults(&verdict), Vec::<String>::new());
        assert_eq!(verdict.permitted, 2, "its first line and its mode");

        // Permitted as a program, and still read for an interpreter like the other three.
        let naming = (
            "publish/export",
            true,
            "#!/usr/bin/env bash\nexec python3 x\n",
        );
        assert_eq!(
            faults(&judge(&tree(&[naming]), Some(&permit))),
            ["publish/export:2: names python3"]
        );

        let said = faults(&judge(&tree(&[launcher]), None));
        assert_eq!(
            said,
            [
                "publish/export:1: begins with #!, and it is none of the four permitted shell files",
                "publish/export: carries the executable bit, and it is none of the four permitted shell files",
            ]
        );
    }

    #[test]
    fn a_permit_for_a_path_that_is_gone_fails() {
        let permit = private("shell\tpublish/export\tthe export's launcher\n");
        assert_eq!(
            faults(&judge(&tree(&[]), Some(&permit))),
            ["publish/export: publish/no-python permits it, and the tracked files hold no such file; remove its entry"]
        );
    }
}
