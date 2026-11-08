//! The allow-list: the exact paths that may stay although a matcher finds them, and why, and the
//! one exact line that may name the interpreter.
//!
//! Each entry is one path, never a prefix, so a new evidence script beside an archived one fails
//! like any other file. An entry whose path the tracked files do not hold fails the stage, so the
//! list cannot outlive what it names.
//!
//! The fourth permitted shell file, `publish/export`, never crosses to the public repository, so
//! its entry is not here: the private list beside it, `publish/no-python`, holds it.
//!
//! [`ALLOWED_LINES`] names one line, by its path and its exact text: the question `flake.nix`
//! states once and the suite's derivation and the development shell's check each ask, whether an
//! interpreter resolves, failing the build that asks if one does. To ask, it must name what it
//! asks for, and a name in a file that runs is a finding, so the line is allowed by its text, which
//! runs nothing but the question. Any other line, and that line changed by one byte or written
//! twice, is a finding as before, and the entry fails the stage when the file no longer holds the
//! line.

use super::matchers::{FileRule, LineRule};

/// Why an allow-listed path stays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Allowance {
    /// An archived change's evidence, which its archived specification cites by this id. Data:
    /// nothing runs it.
    ArchivedEvidence { cited_as: &'static str },
    /// A copy of archived evidence in the migration fixture, which this test needs on disk. Data:
    /// nothing runs it.
    SurveyFixture { needed_by: &'static str },
    /// One of the four shell files the repository may run.
    PermittedShell { runs: &'static str },
}

impl Allowance {
    /// Whether this allowance excuses a whole-file finding. A data file stays as the Python it is
    /// and may begin with `#!`, and never carries the executable bit, because the allow-list admits
    /// only files that nothing runs. A permitted shell file may be a program, and nothing more: it
    /// is read for an interpreter's name like any other shell script.
    pub fn excuses(&self, rule: &FileRule) -> bool {
        match self {
            Self::ArchivedEvidence { .. } | Self::SurveyFixture { .. } => matches!(
                rule,
                FileRule::PythonFileName
                    | FileRule::PythonShebang { .. }
                    | FileRule::BeginsWithShebang
            ),
            Self::PermittedShell { .. } => permitted_shell_excuses(rule),
        }
    }

    /// Why the path stays, in the words the stage's report uses.
    pub fn reason(&self) -> String {
        match self {
            Self::ArchivedEvidence { cited_as } => {
                format!("archived evidence its archived specification cites as {cited_as}")
            }
            Self::SurveyFixture { needed_by } => format!("a survey fixture that {needed_by} needs"),
            Self::PermittedShell { runs } => format!("a permitted shell file, {runs}"),
        }
    }
}

/// What a permitted shell file is excused: being a program, by its first line and its mode.
pub fn permitted_shell_excuses(rule: &FileRule) -> bool {
    matches!(rule, FileRule::BeginsWithShebang | FileRule::ExecutableBit)
}

/// One allow-listed path.
#[derive(Clone, Copy, Debug)]
pub struct Allowed {
    pub path: &'static str,
    pub allowance: Allowance,
}

/// `every_extension_the_project_holds_reaches_the_survey`, which surveys the migration fixture.
const SURVEY: &str =
    "every_extension_the_project_holds_reaches_the_survey, in crates/htmlspec-cli/tests/migrate.rs";

/// The paths that stay, each exactly.
pub const ALLOW_LIST: [Allowed; 7] = [
    Allowed {
        path: "htmlspec/changes/archive/2026-09-02-carry-task-progress-into-the-listing/evidence/prove-listing.py",
        allowance: Allowance::ArchivedEvidence {
            cited_as: "evidence-prove-listing",
        },
    },
    Allowed {
        path: "htmlspec/changes/archive/2026-09-02-state-a-changes-place-in-a-sequence/evidence/prove-sequence.py",
        allowance: Allowance::ArchivedEvidence {
            cited_as: "evidence-prove-sequence",
        },
    },
    Allowed {
        path: "fixtures/migration-source/openspec/changes/archive/2026-09-02-carry-task-progress-into-the-listing/evidence/prove-listing.py",
        allowance: Allowance::SurveyFixture { needed_by: SURVEY },
    },
    Allowed {
        path: "fixtures/migration-source/openspec/changes/archive/2026-09-02-state-a-changes-place-in-a-sequence/evidence/prove-sequence.py",
        allowance: Allowance::SurveyFixture { needed_by: SURVEY },
    },
    Allowed {
        path: "scripts/check",
        allowance: Allowance::PermittedShell {
            runs: "the check suite, as one command",
        },
    },
    Allowed {
        path: "scripts/install-hooks",
        allowance: Allowance::PermittedShell {
            runs: "the installer that points core.hooksPath at .githooks",
        },
    },
    Allowed {
        path: ".githooks/pre-push",
        allowance: Allowance::PermittedShell {
            runs: "the hook that refuses a push to main or to the public repository",
        },
    },
];

/// One line of a file that runs, allowed to name the interpreter by its exact text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllowedLine {
    pub path: &'static str,
    /// The line's exact bytes, its indentation included and its line ending left out.
    pub text: &'static str,
    /// Why it stays, in the words the stage's report uses.
    pub reason: &'static str,
}

impl AllowedLine {
    /// Whether the line excuses a finding of this rule: a name it asks about, and nothing else, so a
    /// handoff or a declaration on the same text would still be a finding.
    pub fn excuses(&self, rule: &LineRule) -> bool {
        matches!(rule, LineRule::Names { .. })
    }
}

/// The lines that stay, each by its exact text, each excusing one line of its file.
pub const ALLOWED_LINES: [AllowedLine; 1] = [AllowedLine {
    path: "flake.nix",
    text: "          if command -v python3 || command -v python; then",
    reason:
        "the flake's question whether the suite's sandbox or the development shell supplies an \
             interpreter, which fails the build that asks it if one does",
}];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::super::tracked::RepoPath;
    use super::*;

    /// The allowed line asks and runs nothing: each command on it is `command -v`, which looks a
    /// name up and never starts it, or the shell's own `if` and `then`.
    #[test]
    fn the_allowed_line_runs_nothing_but_the_question() {
        for line in ALLOWED_LINES {
            assert!(RepoPath::parse(line.path).is_ok(), "{}", line.path);
            let question = line
                .text
                .trim_start()
                .strip_prefix("if ")
                .and_then(|rest| rest.strip_suffix("; then"))
                .expect("an if whose condition is the question");
            for asked in question.split(" || ") {
                let mut words = asked.split(' ');
                assert_eq!(
                    (words.next(), words.next(), words.clone().count()),
                    (Some("command"), Some("-v"), 1),
                    "{asked:?} is not one name looked up"
                );
            }
        }
    }

    #[test]
    fn every_entry_is_one_exact_path_named_once() {
        let mut seen = BTreeSet::new();
        for entry in ALLOW_LIST {
            assert!(RepoPath::parse(entry.path).is_ok(), "{}", entry.path);
            assert!(seen.insert(entry.path), "{} is listed twice", entry.path);
        }
    }

    /// Three shell files cross to the public repository; the fourth is in the private list.
    #[test]
    fn the_list_names_four_data_files_and_three_shell_files() {
        let shells = ALLOW_LIST
            .iter()
            .filter(|entry| matches!(entry.allowance, Allowance::PermittedShell { .. }))
            .count();
        assert_eq!((ALLOW_LIST.len() - shells, shells), (4, 3));
    }

    #[test]
    fn a_data_file_is_never_a_program_and_a_shell_file_is_nothing_else() {
        let data = Allowance::ArchivedEvidence { cited_as: "x" };
        assert!(data.excuses(&FileRule::PythonFileName));
        assert!(data.excuses(&FileRule::BeginsWithShebang));
        assert!(!data.excuses(&FileRule::ExecutableBit));
        let shell = Allowance::PermittedShell { runs: "x" };
        assert!(shell.excuses(&FileRule::BeginsWithShebang));
        assert!(shell.excuses(&FileRule::ExecutableBit));
        assert!(!shell.excuses(&FileRule::PythonFileName));
        assert!(!shell.excuses(&FileRule::PythonShebang {
            interpreter: String::from("python3")
        }));
    }
}
