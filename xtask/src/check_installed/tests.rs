//! The comparison on scratch trees, and the check on an `init` that writes what each test says.

use super::*;
use crate::scratch::Scratch;

/// What `init` writes, in this repository's shape: a command, a skill, and a prompt.
const WRITTEN: [(&str, &str); 6] = [
    (
        ".claude/commands/htmlspec/apply.md",
        "apply, generatedBy: htmlspec 1.3.1\n",
    ),
    (
        ".claude/skills/htmlspec-apply-change/SKILL.md",
        "the skill\n",
    ),
    (
        ".codex/skills/htmlspec-apply-change/SKILL.md",
        "the skill\n",
    ),
    (".pi/prompts/htmlspec-apply.md", "the prompt\n"),
    (".pi/skills/htmlspec-apply-change/SKILL.md", "the skill\n"),
    (
        "htmlspec/README.md",
        "a new project's skeleton, a byproduct\n",
    ),
];

/// An `init` that writes [`WRITTEN`], with `changed` written instead where it names a file, or
/// fails saying `failing`.
struct FakeInit {
    changed: Option<(&'static str, &'static str)>,
    failing: Option<&'static str>,
}

impl Init for FakeInit {
    fn init(&self, _root: &Path, scratch: &Path) -> Result<(), NotRun> {
        if let Some(stderr) = self.failing {
            return Err(NotRun::Failed {
                stderr: stderr.to_owned(),
            });
        }
        for (path, content) in WRITTEN {
            let content = match self.changed {
                Some((changed, instead)) if changed == path => instead,
                _ => content,
            };
            let file = scratch.join(path);
            std::fs::create_dir_all(file.parent().expect("a parent")).expect("made");
            std::fs::write(file, content).expect("written");
        }
        Ok(())
    }
}

const WRITES: FakeInit = FakeInit {
    changed: None,
    failing: None,
};

/// A repository whose committed files are what [`WRITES`] writes, beside another tool's skill.
fn committed(scratch: &Scratch) {
    for (path, content) in &WRITTEN[..5] {
        scratch.write(path, content);
    }
    scratch.write(
        ".claude/skills/openspec-apply/SKILL.md",
        "another tool's skill\n",
    );
    scratch.write("htmlspec/README.md", "this repository's own project\n");
}

#[test]
fn committed_files_that_match_what_init_writes_pass() {
    let (repository, scratch) = (
        Scratch::new("installed-root"),
        Scratch::new("installed-run"),
    );
    committed(&repository);
    let report = check(repository.path(), scratch.path(), &WRITES);
    assert_eq!(
        report,
        Report {
            exit: Exit::Clean,
            out: vec![String::from(
                "  ok  5 installed instruction file(s) match what `init` writes"
            )],
            err: Vec::new(),
        }
    );
    assert_eq!(report.exit.code(), 0);
}

/// The defect the check exists for: a version stamp edited by hand, which no run of `init` wrote.
#[test]
fn a_stamp_edited_by_hand_fails_naming_the_file() {
    let (repository, scratch) = (
        Scratch::new("installed-root"),
        Scratch::new("installed-run"),
    );
    committed(&repository);
    repository.write(
        ".claude/commands/htmlspec/apply.md",
        "apply, generatedBy: htmlspec 9.9.9\n",
    );
    let report = check(repository.path(), scratch.path(), &WRITES);
    assert_eq!(report.exit, Exit::Faults);
    assert_eq!(report.exit.code(), 1);
    assert_eq!(
        report.out[0],
        "  FAIL  .claude/commands/htmlspec/apply.md: differs from what `init` writes"
    );
    assert_eq!(report.out.len(), 7, "{report:?}");
    assert_eq!(report.out[4], "  htmlspec init --tools claude,codex,pi");
}

/// A file `init` writes and nothing commits, one committed and no longer written, and a skill
/// whose body drifted, each named; another tool's skill is not judged.
#[test]
fn every_kind_of_drift_is_named() {
    let (repository, scratch) = (
        Scratch::new("installed-root"),
        Scratch::new("installed-run"),
    );
    committed(&repository);
    repository.remove(".pi/prompts/htmlspec-apply.md");
    repository.write(
        ".pi/prompts/htmlspec-sync.md",
        "a prompt init stopped writing\n",
    );
    let drifted = FakeInit {
        changed: Some((
            ".codex/skills/htmlspec-apply-change/SKILL.md",
            "a new body\n",
        )),
        failing: None,
    };
    std::fs::remove_dir_all(scratch.path()).expect("removed");
    std::fs::create_dir(scratch.path()).expect("made");
    assert_eq!(
        ask(repository.path(), scratch.path(), &drifted),
        Ok(vec![
            Drift::Differs {
                path: PathBuf::from(".codex/skills/htmlspec-apply-change/SKILL.md")
            },
            Drift::NotCommitted {
                path: PathBuf::from(".pi/prompts/htmlspec-apply.md")
            },
        ])
    );
    // The committed prompt was copied into the scratch directory before `init` ran, as `init`
    // refreshes what it finds, so it reads as written: the copy is what `init` would leave.
    assert!(scratch
        .path()
        .join(".pi/prompts/htmlspec-sync.md")
        .is_file());
    let fresh = Scratch::new("installed-fresh");
    WRITES
        .init(repository.path(), fresh.path())
        .expect("written");
    assert_eq!(
        compare(repository.path(), fresh.path()),
        vec![
            Drift::NotCommitted {
                path: PathBuf::from(".pi/prompts/htmlspec-apply.md")
            },
            Drift::NoLongerWritten {
                path: PathBuf::from(".pi/prompts/htmlspec-sync.md")
            },
        ]
    );
}

/// A directory the table names and `init` does not write is a stale table, and says so.
#[test]
fn a_directory_init_does_not_write_is_named() {
    let (repository, fresh) = (
        Scratch::new("installed-root"),
        Scratch::new("installed-fresh"),
    );
    committed(&repository);
    WRITES
        .init(repository.path(), fresh.path())
        .expect("written");
    std::fs::remove_dir_all(fresh.path().join(".pi/skills")).expect("removed");
    assert_eq!(
        compare(repository.path(), fresh.path()),
        vec![Drift::NotWritten {
            directory: PathBuf::from(".pi/skills")
        }]
    );
}

/// A binary that cannot be run exits 2, saying what it said.
#[test]
fn a_binary_that_cannot_run_exits_two() {
    let (repository, scratch) = (
        Scratch::new("installed-root"),
        Scratch::new("installed-run"),
    );
    committed(&repository);
    let failing = FakeInit {
        changed: None,
        failing: Some("error: could not compile"),
    };
    let report = check(repository.path(), scratch.path(), &failing);
    assert_eq!(report.exit, Exit::CouldNotAsk);
    assert_eq!(report.exit.code(), 2);
    assert_eq!(
        report.out,
        vec![String::from(
            "  the binary could not be run: error: could not compile"
        )]
    );
}
