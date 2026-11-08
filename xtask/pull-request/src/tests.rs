//! Each title rule, the changelog rule, the diff on a scratch Git repository, and how a run ends.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

/// Tells apart the directories one process's tests make, which run in parallel.
static MADE: AtomicUsize = AtomicUsize::new(0);

/// A temporary directory that removes itself when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "htmlspec-pull-request-{label}-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// Writes `content` at `path`, beneath the directory, making its parents.
    fn write(&self, path: &str, content: &str) {
        let file = self.0.join(path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).expect("the parent directory is created");
        }
        std::fs::write(&file, content).expect("the file is written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The command line the guard's binary and the tools' subcommand share.
#[test]
fn the_command_line_reads_each_job() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        arguments: Arguments,
    }
    let parsed = |line: &[&str]| {
        Cli::try_parse_from(std::iter::once("check-pull-request").chain(line.iter().copied()))
            .map(|cli| cli.arguments.job())
    };
    assert_eq!(
        parsed(&["--title", "Port the export"]).ok(),
        Some(Job::Whole {
            title: "Port the export".to_owned(),
            changed: Changed::FromGit {
                base: "origin/main".to_owned()
            },
            labels_json: "[]".to_owned(),
        })
    );
    assert_eq!(
        parsed(&["--title", "Port the export", "--changed", "-"]).ok(),
        Some(Job::Whole {
            title: "Port the export".to_owned(),
            changed: Changed::Listed(Source::StandardInput),
            labels_json: "[]".to_owned(),
        })
    );
    assert_eq!(
        parsed(&["--title", "Port the export", "--title-only"]).ok(),
        Some(Job::TitleOnly {
            title: "Port the export".to_owned()
        })
    );
    for refused in [
        &["--title-only"][..],
        &["--title", "x", "--title-only", "--base", "main"],
        &["--title", "x", "--title-only", "--changed", "-"],
        &["--title", "x", "--title-only", "--labels-json", "[]"],
    ] {
        assert_eq!(
            parsed(refused).err().map(|error| error.exit_code()),
            Some(2),
            "{refused:?}"
        );
    }
}

/// The faults' kinds, without what they carry, for the rules' tests.
fn kinds(title: &str) -> Vec<&'static str> {
    title_faults(title)
        .iter()
        .map(|fault| match fault {
            TitleFault::Empty => "empty",
            TitleFault::WorkInProgress { .. } => "work in progress",
            TitleFault::Conventional { .. } => "conventional",
            TitleFault::TooLong { .. } => "too long",
            TitleFault::TooShort { .. } => "too short",
            TitleFault::OpensWithoutAWord { .. } => "opens without a word",
            TitleFault::Lowercase { .. } => "lowercase",
            TitleFault::Reports { .. } => "reports",
            TitleFault::EndsWithAPeriod => "period",
            TitleFault::Numbered { .. } => "numbered",
        })
        .collect()
}

/// Subjects from this project's history, which the rules exist to keep reading as they do.
#[test]
fn a_title_that_reads_as_the_history_reads_passes() {
    for title in [
        "Finish the archival, and fix the gate that never fired",
        "Pass a contributor's arguments through the invocation",
        "Add readme markdown",
        "Archive distinguish-command-names-from-skill-names",
        "Port the release workflow's tools to Rust, and build them once",
        "Export htmlspec 1.3.1 from 0123456789ab",
        "Begin htmlspec with its README and license",
        "  Trimmed at both ends, as the guard reads it  ",
        "Wipe the cache before the suite runs",
        "Drafting is not a marker of work in progress",
    ] {
        assert_eq!(title_faults(title), Vec::new(), "{title:?}");
    }
}

#[test]
fn an_empty_title_is_refused_and_nothing_else_is_said() {
    for title in ["", "   ", "\t\n", "\u{1c}\u{3000}"] {
        assert_eq!(title_faults(title), vec![TitleFault::Empty], "{title:?}");
    }
}

#[test]
fn a_work_in_progress_marker_is_refused_in_any_case() {
    for title in [
        "WIP: port the export onto its capabilities",
        "Wip port the export onto its capabilities",
        "[wip] Port the export onto its capabilities",
        "Draft: port the export onto its capabilities",
        "[DRAFT] Port the export onto its capabilities",
        "W\u{131}p port the export onto its capabilities",
    ] {
        assert!(kinds(title).contains(&"work in progress"), "{title:?}");
    }
    for title in [
        "Wipe the cache before the suite runs",
        "Drafted plans stay where they are",
        "Wip_tool is not a marker either",
    ] {
        assert!(!kinds(title).contains(&"work in progress"), "{title:?}");
    }
}

#[test]
fn a_conventional_commits_prefix_is_refused() {
    for title in [
        "feat: port the export onto its capabilities",
        "fix(parser)!: read a heading's version once",
        "chore(deps): move the lockfile forward",
        "refactor!:\tthe whole thing at once",
    ] {
        assert!(kinds(title).contains(&"conventional"), "{title:?}");
    }
    for title in [
        "Fix: the gate that never fired again",
        "feat:no space after the colon here",
        "fix(unclosed: the scope is never closed",
        "Read cargo metadata (once): then stop",
    ] {
        assert!(!kinds(title).contains(&"conventional"), "{title:?}");
    }
}

#[test]
fn a_title_over_seventy_two_characters_is_refused() {
    let at_the_limit = format!("Port {}", "x".repeat(MAX_TITLE - 5));
    assert_eq!(at_the_limit.chars().count(), MAX_TITLE);
    assert_eq!(title_faults(&at_the_limit), Vec::new());
    let over = format!("{at_the_limit}y");
    assert_eq!(
        title_faults(&over),
        vec![TitleFault::TooLong { length: 73 }]
    );
    // Characters, not bytes: seventy-two letters of two bytes each pass.
    let wide = format!("Port {}", "é".repeat(MAX_TITLE - 5));
    assert_eq!(title_faults(&wide), Vec::new());
}

#[test]
fn a_title_under_sixteen_characters_or_of_one_word_is_refused() {
    assert_eq!(
        title_faults("Dark mode"),
        vec![TitleFault::TooShort {
            length: 9,
            words: 2
        }]
    );
    assert_eq!(
        title_faults("Supercalifragilistic"),
        vec![TitleFault::TooShort {
            length: 20,
            words: 1
        }]
    );
    assert_eq!(title_faults("Add the fifteen"), kinds_of_too_short(15, 3));
    assert_eq!(
        title_faults("Add the sixteen."),
        vec![TitleFault::EndsWithAPeriod]
    );
}

fn kinds_of_too_short(length: usize, words: usize) -> Vec<TitleFault> {
    vec![TitleFault::TooShort { length, words }]
}

#[test]
fn a_title_opens_with_a_capital_letter() {
    assert_eq!(
        title_faults("#3 is the issue this closes, at last"),
        vec![TitleFault::OpensWithoutAWord { first: '#' }]
    );
    assert_eq!(
        title_faults("port the export onto its capabilities"),
        vec![TitleFault::Lowercase {
            title: "port the export onto its capabilities".to_owned()
        }]
    );
    assert_eq!(title_faults("Éclair the parser, with care"), Vec::new());
}

#[test]
fn an_opener_that_reports_rather_than_instructs_is_refused() {
    for (title, first) in [
        ("Fixed the parser that read two grammars", "Fixed"),
        ("Updates: the docs, and the guard again", "Updates:"),
        ("Adding, then removing, the second reader", "Adding,"),
        ("RENAMED every script to its subcommand", "RENAMED"),
    ] {
        assert_eq!(
            title_faults(title),
            vec![TitleFault::Reports {
                first: first.to_owned()
            }],
            "{title:?}"
        );
    }
    assert_eq!(
        title_faults("Fix the parser that read two grammars"),
        Vec::new()
    );
}

#[test]
fn a_title_ending_with_a_period_is_refused() {
    assert_eq!(
        title_faults("Port the export onto its capabilities."),
        vec![TitleFault::EndsWithAPeriod]
    );
}

#[test]
fn a_title_naming_a_pull_request_number_is_refused() {
    let title = "Port the export onto its capabilities (#103)";
    assert_eq!(
        title_faults(title),
        vec![TitleFault::Numbered {
            title: title.to_owned()
        }]
    );
    for title in [
        "Port the export onto its capabilities (#)",
        "Port the export onto its capabilities (103)",
        "Port the export (#103) onto its capabilities",
    ] {
        assert!(!kinds(title).contains(&"numbered"), "{title:?}");
    }
}

/// One run reports every fault, in the order the rules are listed.
#[test]
fn every_fault_is_reported_in_one_run() {
    assert_eq!(
        kinds("fixed: wip."),
        vec![
            "conventional",
            "too short",
            "lowercase",
            "reports",
            "period"
        ]
    );
    assert_eq!(
        kinds("wip"),
        vec!["work in progress", "too short", "lowercase"]
    );
}

/// The words a fault says, which a contributor reads and the export's refusal repeats.
#[test]
fn each_fault_says_what_the_script_said() {
    let said = |title: &str| -> Vec<String> {
        title_faults(title)
            .iter()
            .map(ToString::to_string)
            .collect()
    };
    assert_eq!(
        said(&format!(
            "Export htmlspec 1.2.3-{} from 0123456789ab",
            "x".repeat(40)
        )),
        vec![
            "the title is 80 characters, over 72. It becomes a commit subject, which is read in \
             one line or not at all"
        ]
    );
    assert_eq!(
        said("fix: it's"),
        vec![
            "the title carries a Conventional Commits prefix: \"fix: it's\". This project's \
             history is imperative prose — see `git log --format=%s`",
            "the title is 9 character(s) in 2 word(s), under 16. Say what the change does, not \
             what area it touches",
            "the title starts lowercase: \"fix: it's\". A commit subject is a sentence, not a \
             branch name",
        ]
    );
    assert_eq!(
        said("Fixed the parser that read two grammars"),
        vec![
            "the title opens with 'Fixed', which reports rather than instructs. This project's \
             subjects are imperative: 'Fix the gate', not 'Fixed the gate'"
        ]
    );
    assert_eq!(
        said("'Quoted' is how this title opens"),
        vec!["the title starts with \"'\". A commit subject opens with a word"]
    );
}

/// The quoting the script's messages used, Python's `repr` of a string.
#[test]
fn a_string_is_quoted_as_the_script_quoted_it() {
    assert_eq!(quoted("plain"), "'plain'");
    assert_eq!(quoted("it's"), "\"it's\"");
    assert_eq!(quoted("both ' and \""), "'both \\' and \"'");
    assert_eq!(quoted("a\tb\nc\\d\re"), "'a\\tb\\nc\\\\d\\re'");
    assert_eq!(
        quoted("\u{0}\u{7f}\u{a0}\u{200b}\u{2028}\u{e000}é—"),
        "'\\x00\\x7f\\xa0\\u200b\\u2028\\ue000é—'"
    );
    assert_eq!(quoted("\u{e0001}"), "'\\U000e0001'");
}

#[test]
fn the_changelog_rule_asks_for_its_file_or_the_label() {
    let changed =
        |paths: &[&str]| -> Vec<String> { paths.iter().map(|p| (*p).to_owned()).collect() };
    let no_labels: Vec<String> = Vec::new();
    let skip = vec![SKIP_LABEL.to_owned()];
    assert_eq!(
        changelog_faults(&changed(&["CHANGELOG.md", "xtask/src/lib.rs"]), &no_labels),
        Vec::new()
    );
    assert_eq!(
        changelog_faults(&changed(&["xtask/src/lib.rs"]), &no_labels),
        vec![ChangelogFault::Untouched]
    );
    assert_eq!(
        changelog_faults(&changed(&["docs/CHANGELOG.md"]), &no_labels),
        vec![ChangelogFault::Untouched]
    );
    assert_eq!(
        changelog_faults(&[], &no_labels),
        vec![ChangelogFault::NothingChanged]
    );
    assert_eq!(
        changelog_faults(&changed(&["xtask/src/lib.rs"]), &skip),
        Vec::new()
    );
    assert_eq!(changelog_faults(&[], &skip), Vec::new());
    assert_eq!(
        ChangelogFault::Untouched.to_string(),
        "CHANGELOG.md is untouched. Record what this change does for someone reading the project \
         a year from now, or label the pull request 'skip-changelog' to state that there is \
         nothing to record"
    );
}

#[test]
fn a_list_of_changed_files_is_one_path_a_line() {
    assert_eq!(
        listed("CHANGELOG.md\n\n  xtask/src/lib.rs  \r\nREADME.md"),
        vec!["CHANGELOG.md", "xtask/src/lib.rs", "README.md"]
    );
    assert_eq!("-".parse::<Source>(), Ok(Source::StandardInput));
    assert_eq!(
        "changed.txt".parse::<Source>(),
        Ok(Source::File(PathBuf::from("changed.txt")))
    );
}

/// A scratch repository with a `main` and a branch off it.
struct Repository(Scratch);

impl Repository {
    fn new() -> Self {
        let repository = Self(Scratch::new("pull-request"));
        repository.git(&["init", "-q", "-b", "main"]);
        repository.0.write("README.md", "# A project\n");
        repository.0.write("CHANGELOG.md", "# Changelog\n");
        repository.commit("Begin the project with its README");
        repository
    }

    fn git(&self, arguments: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(self.0.path())
            .args(arguments)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .status()
            .expect("git starts");
        assert!(status.success(), "git {arguments:?} failed");
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }
}

/// The diff is three-dot: what the branch changed since it left `main`, and not what `main` did
/// after.
#[test]
fn the_changed_files_are_what_the_branch_changed_since_it_left_its_base() {
    let repository = Repository::new();
    repository.git(&["switch", "-q", "-c", "a-change"]);
    repository.0.write("src/lib.rs", "// the change\n");
    repository.commit("Make the change");
    repository.git(&["switch", "-q", "main"]);
    repository
        .0
        .write("CHANGELOG.md", "# Changelog\n\n## Moved on\n");
    repository.0.write("main-only.txt", "main moved on\n");
    repository.commit("Move main on");
    repository.git(&["switch", "-q", "a-change"]);
    let changed = changed_from_git("main", repository.0.path()).expect("git answers");
    assert_eq!(changed, vec!["src/lib.rs"]);
    assert_eq!(
        changelog_faults(&changed, &[]),
        vec![ChangelogFault::Untouched],
        "main's changelog edit is not the branch's"
    );
    repository
        .0
        .write("CHANGELOG.md", "# Changelog\n\n## The change\n");
    repository.commit("Record the change");
    let changed = changed_from_git("main", repository.0.path()).expect("git answers");
    assert_eq!(changed, vec!["CHANGELOG.md", "src/lib.rs"]);
    assert_eq!(changelog_faults(&changed, &[]), Vec::new());
}

#[test]
fn a_base_git_does_not_hold_cannot_be_diffed() {
    let repository = Repository::new();
    assert_eq!(changed_from_git("origin/main", repository.0.path()), None);
    let report = report(
        &Job::Whole {
            title: "Port the export onto its capabilities".to_owned(),
            changed: Changed::FromGit {
                base: "origin/main".to_owned(),
            },
            labels_json: "[]".to_owned(),
        },
        |changed| match changed {
            Changed::FromGit { base } => changed_from_git(base, repository.0.path())
                .ok_or_else(|| CannotAsk::NoDiff { base: base.clone() }),
            Changed::Listed(_) => unreachable!("the job asks Git"),
        },
    );
    assert_eq!(report.exit, Exit::CouldNotAsk);
    assert_eq!(
        report.err,
        vec!["cannot diff against origin/main. Fetch it, or pass --changed with the file list"]
    );
}

fn whole(title: &str, labels_json: &str) -> Job {
    Job::Whole {
        title: title.to_owned(),
        changed: Changed::Listed(Source::StandardInput),
        labels_json: labels_json.to_owned(),
    }
}

/// Each ending's exit code, and every line it prints.
#[test]
fn each_ending_has_its_exit_code_and_its_lines() {
    let listing = |paths: &[&str]| {
        let paths: Vec<String> = paths.iter().map(|path| (*path).to_owned()).collect();
        move |_: &Changed| Ok(paths)
    };
    let clean = report(
        &whole("  Port the export onto its capabilities ", "[]"),
        listing(&["CHANGELOG.md", "publish/export"]),
    );
    assert_eq!(clean.exit, Exit::Clean);
    assert_eq!(
        clean.out,
        vec![
            "  ok  the title reads as a commit subject: Port the export onto its capabilities",
            "  ok  2 changed file(s) record the change",
        ]
    );
    let excused = report(
        &whole(
            "Port the export onto its capabilities",
            "[\"skip-changelog\"]",
        ),
        listing(&["publish/export"]),
    );
    assert_eq!(excused.exit, Exit::Clean);
    assert_eq!(
        excused.out[1],
        "  ok  1 changed file(s) record the change, skip-changelog label present"
    );
    let faults = report(&whole("fixed it", "[]"), listing(&["publish/export"]));
    assert_eq!(faults.exit, Exit::Faults);
    assert!(faults.out.is_empty());
    assert_eq!(
        faults.err.first().map(String::as_str),
        Some("pull request:")
    );
    assert_eq!(faults.err.len(), 1 + 4 + 2, "{:?}", faults.err);
    assert!(faults.err[1..5].iter().all(|line| line.starts_with("!!! ")));
    assert_eq!(faults.err[5], "");
    assert_eq!(faults.err[6], "4 fault(s) across 1 changed file(s)");
    for given in ["{}", "[1]", "not json", "[\"a\", null]"] {
        let refused = report(
            &whole("Port the export onto its capabilities", given),
            |_| unreachable!("the labels are read first"),
        );
        assert_eq!(refused.exit, Exit::CouldNotAsk, "{given}");
        assert_eq!(
            refused.err,
            vec![format!(
                "--labels-json is not a JSON array of strings: {}",
                quoted(given)
            )]
        );
    }
    let unreadable = report(
        &whole("Port the export onto its capabilities", "[]"),
        |_| {
            Err(CannotAsk::Unreadable {
                source: Source::File(PathBuf::from("changed.txt")),
                kind: std::io::ErrorKind::NotFound,
            })
        },
    );
    assert_eq!(unreadable.exit, Exit::CouldNotAsk);
    assert_eq!(
        unreadable.err,
        vec!["cannot read --changed changed.txt: entity not found"]
    );
}

/// The mode the export runs from the tree it exports: the title, and nothing of a diff.
#[test]
fn the_title_alone_is_checked_without_a_diff() {
    let taken = report(
        &Job::TitleOnly {
            title: "Export htmlspec 1.3.1 from 0123456789ab".to_owned(),
        },
        |_| unreachable!("no diff is read"),
    );
    assert_eq!(taken.exit, Exit::Clean);
    assert_eq!(
        taken.out,
        vec!["  ok  the title reads as a commit subject: Export htmlspec 1.3.1 from 0123456789ab"]
    );
    let long = format!("Export htmlspec 1.2.3-{} from 0123456789ab", "x".repeat(40));
    let refused = report(&Job::TitleOnly { title: long }, |_| {
        unreachable!("no diff is read")
    });
    assert_eq!(refused.exit, Exit::Faults);
    assert_eq!(
        refused.err,
        vec![
            "pull request:".to_owned(),
            "!!! the title is 80 characters, over 72. It becomes a commit subject, which is read \
             in one line or not at all"
                .to_owned(),
            String::new(),
            "1 fault(s) in the title".to_owned(),
        ]
    );
}
