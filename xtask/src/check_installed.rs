//! `cargo xtask check-installed`: whether this repository's installed instruction files are what
//! the binary writes.
//!
//! # Why this exists
//!
//! The workflow bodies are compiled into the binary, so an agent reads the copy `htmlspec init`
//! wrote into `.claude/`, `.codex/`, and `.pi/`, not the file under
//! `crates/htmlspec-assets/instructions/`. Those are two artifacts, and nothing compared them until
//! this check did. They drifted for four releases: `sync` shipped in 0.7.0 and existed in the
//! binary and in no file an agent could read until 0.9.0, and two `.pi` files were once edited by
//! hand to a version stamp no run of `init` could have written. `project-bootstrap` states the
//! general form, that a rule nothing checks is not a rule, and this is that shape applied to the
//! files this project asks other projects to trust.
//!
//! # How it asks
//!
//! By running `init` into a scratch directory and comparing what it writes with what is committed.
//! A body-by-body comparison would need this tool to know how each tool's file is assembled, its
//! frontmatter, its field order, its name prefixes, and that knowledge is `install.rs`'s. A copy of
//! it here would pass while the real writer did something else, which is the defect in miniature.
//!
//! `--tools claude,codex,pi` rather than `all`: `all` also installs for tools this repository does
//! not use, writing `.cursor/`, and appends the bodies to the hand-authored `AGENTS.md`.
//!
//! Exit codes: 0 clean, 1 drift found, 2 could not ask.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::report::{Exit, Report};
use crate::workspace::Workspace;

/// The tools this repository installs for. Not `all`, as the module says.
const TOOLS: &str = "claude,codex,pi";

/// The prefix marking an entry as this project's, in a directory shared with another tool.
const PREFIX: &str = "htmlspec";

/// Whose entries a directory `init` writes into holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Holds {
    /// Namespaced by its own path, so every entry is this project's and carries no prefix.
    OursAlone,
    /// Shared with other tools' entries, which this check must not judge, so only the entries
    /// named with [`PREFIX`] are compared. `.claude/skills` held OpenSpec's until 0.30.1.
    SharedWithOtherTools,
}

/// One directory `init` writes into, compared as a whole tree rather than as a list of files, so
/// a file that should exist and does not is a fault of the same kind as one whose contents drifted:
/// which is how `sync` went missing for three releases.
struct Installed {
    path: &'static str,
    holds: Holds,
}

const INSTALLED: [Installed; 5] = [
    Installed {
        path: ".claude/commands/htmlspec",
        holds: Holds::OursAlone,
    },
    Installed {
        path: ".claude/skills",
        holds: Holds::SharedWithOtherTools,
    },
    Installed {
        path: ".codex/skills",
        holds: Holds::SharedWithOtherTools,
    },
    Installed {
        path: ".pi/prompts",
        holds: Holds::SharedWithOtherTools,
    },
    Installed {
        path: ".pi/skills",
        holds: Holds::SharedWithOtherTools,
    },
];

/// What `init` writes into the scratch directory beside the installed files: a new project's
/// skeleton, and files for tools this repository does not install for. This repository's own
/// `htmlspec/`, `spec.nix`, and `AGENTS.md` are its project and its own text, not installed output,
/// so here they are byproducts, and the comparison leaves them out.
const BYPRODUCTS: [&str; 4] = ["htmlspec", "spec.nix", ".cursor", "AGENTS.md"];

/// The file a skill's directory holds, which is the part of a skill `init` writes.
const SKILL: &str = "SKILL.md";

/// One way the committed files differ from what `init` writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Drift {
    /// `init` wrote no directory where the table says it writes one.
    NotWritten { directory: PathBuf },
    /// `init` writes this, and it is not committed.
    NotCommitted { path: PathBuf },
    /// Committed, and `init` no longer writes it.
    NoLongerWritten { path: PathBuf },
    /// Committed, and different from what `init` writes.
    Differs { path: PathBuf },
    /// One side is missing or unreadable, so the two could not be compared.
    NotComparable { path: PathBuf },
}

impl fmt::Display for Drift {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotWritten { directory } => write!(
                formatter,
                "{}: `init` wrote no such directory, so this check's idea of where files land is \
                 stale",
                directory.display()
            ),
            Self::NotCommitted { path } => write!(
                formatter,
                "{}: `init` writes this and it is not committed",
                path.display()
            ),
            Self::NoLongerWritten { path } => write!(
                formatter,
                "{}: committed and `init` no longer writes it",
                path.display()
            ),
            Self::Differs { path } => write!(
                formatter,
                "{}: differs from what `init` writes",
                path.display()
            ),
            Self::NotComparable { path } => {
                write!(formatter, "{}: could not be compared", path.display())
            }
        }
    }
}

/// The names of the entries in `directory` this check judges, sorted: every one where the
/// directory is ours alone, and the prefixed ones where it is shared.
fn judged(directory: &Path, holds: Holds) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| holds == Holds::OursAlone || name.starts_with(PREFIX))
        .collect();
    names.sort();
    names
}

/// Whether two files hold the same bytes, or `None` where either cannot be read.
fn same(one: &Path, other: &Path) -> Option<bool> {
    let one = std::fs::read(one).ok()?;
    let other = std::fs::read(other).ok()?;
    Some(one == other)
}

/// Every way the files committed under `root` differ from those `init` wrote under `scratch`.
pub fn compare(root: &Path, scratch: &Path) -> Vec<Drift> {
    let mut found = Vec::new();
    for installed in &INSTALLED {
        let relative = Path::new(installed.path);
        let (committed, fresh) = (root.join(relative), scratch.join(relative));
        if !fresh.is_dir() {
            found.push(Drift::NotWritten {
                directory: relative.to_path_buf(),
            });
            continue;
        }
        let here = judged(&committed, installed.holds);
        let there = judged(&fresh, installed.holds);
        for name in there.iter().filter(|name| !here.contains(name)) {
            found.push(Drift::NotCommitted {
                path: relative.join(name),
            });
        }
        for name in here.iter().filter(|name| !there.contains(name)) {
            found.push(Drift::NoLongerWritten {
                path: relative.join(name),
            });
        }
        for name in here.iter().filter(|name| there.contains(name)) {
            // A skill is a directory, and its SKILL.md is what `init` writes of it.
            let (path, one, other) = if committed.join(name).is_dir() {
                (
                    relative.join(name).join(SKILL),
                    committed.join(name).join(SKILL),
                    fresh.join(name).join(SKILL),
                )
            } else {
                (relative.join(name), committed.join(name), fresh.join(name))
            };
            match same(&one, &other) {
                Some(true) => {}
                Some(false) => found.push(Drift::Differs { path }),
                None => found.push(Drift::NotComparable { path }),
            }
        }
    }
    found
}

/// How many installed files this check judges under `root`.
fn counted(root: &Path) -> usize {
    INSTALLED
        .iter()
        .map(|installed| judged(&root.join(installed.path), installed.holds).len())
        .sum()
}

/// What running `init` into the scratch directory takes, so a test can pass one that writes what
/// the test says.
pub trait Init {
    /// Runs `htmlspec init --tools claude,codex,pi --root SCRATCH` from `root`.
    fn init(&self, root: &Path, scratch: &Path) -> Result<(), NotRun>;
}

/// Why `init` could not be run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotRun {
    NotStarted(std::io::ErrorKind),
    Failed { stderr: String },
}

impl fmt::Display for NotRun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted(kind) => write!(formatter, "cargo could not be started: {kind}"),
            Self::Failed { stderr } => formatter.write_str(stderr),
        }
    }
}

/// The binary, built and run by the cargo that runs this tool.
pub struct Binary;

impl Init for Binary {
    fn init(&self, root: &Path, scratch: &Path) -> Result<(), NotRun> {
        let output = Command::new(crate::workspace::cargo())
            .args([
                "run", "--quiet", "--locked", "--bin", "htmlspec", "--", "init", "--tools", TOOLS,
                "--root",
            ])
            .arg(scratch)
            .current_dir(root)
            .output()
            .map_err(|error| NotRun::NotStarted(error.kind()))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(NotRun::Failed {
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }
}

/// Why the scratch directory could not be made ready.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ScratchFault {
    doing: &'static str,
    path: PathBuf,
    kind: std::io::ErrorKind,
}

impl fmt::Display for ScratchFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot {} {}: {}",
            self.doing,
            self.path.display(),
            self.kind
        )
    }
}

/// Tells apart the scratch directories one process makes.
static MADE: AtomicUsize = AtomicUsize::new(0);

/// A new directory under the system's temporary directory, made by this process: an existing one
/// is never reused, so nothing another user left there is read.
fn new_scratch() -> Result<PathBuf, ScratchFault> {
    let temporary = std::env::temp_dir();
    loop {
        let path = temporary.join(format!(
            "htmlspec-installed-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(ScratchFault {
                    doing: "make",
                    path,
                    kind: error.kind(),
                })
            }
        }
    }
}

/// Copies the tree at `from` to `to`, files and directories alone.
fn copy_tree(from: &Path, to: &Path) -> Result<(), ScratchFault> {
    let fault = |doing, path: &Path, error: std::io::Error| ScratchFault {
        doing,
        path: path.to_path_buf(),
        kind: error.kind(),
    };
    std::fs::create_dir_all(to).map_err(|error| fault("make", to, error))?;
    for entry in std::fs::read_dir(from).map_err(|error| fault("read", from, error))? {
        let entry = entry.map_err(|error| fault("read", from, error))?;
        let (source, target) = (entry.path(), to.join(entry.file_name()));
        if source.is_dir() {
            copy_tree(&source, &target)?;
        } else {
            std::fs::copy(&source, &target).map_err(|error| fault("copy", &source, error))?;
        }
    }
    Ok(())
}

/// Why the check could not ask: its scratch directory, or the binary.
#[derive(Clone, Debug, PartialEq, Eq)]
enum CannotAsk {
    Scratch(ScratchFault),
    Binary(NotRun),
}

impl fmt::Display for CannotAsk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scratch(fault) => fault.fmt(formatter),
            Self::Binary(fault) => write!(formatter, "  the binary could not be run: {fault}"),
        }
    }
}

/// The scratch directory made ready, `init` run into it, and the comparison.
fn ask(root: &Path, scratch: &Path, init: &dyn Init) -> Result<Vec<Drift>, CannotAsk> {
    // The committed files are what `init` refreshes rather than creates, so the scratch copy starts
    // from them: a tool that keeps existing content would otherwise be compared with what it writes
    // from nothing.
    for installed in &INSTALLED {
        let source = root.join(installed.path);
        if source.is_dir() {
            copy_tree(&source, &scratch.join(installed.path)).map_err(CannotAsk::Scratch)?;
        }
    }
    init.init(root, scratch).map_err(CannotAsk::Binary)?;
    for name in BYPRODUCTS {
        let path = scratch.join(name);
        let removed = if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else if path.exists() {
            std::fs::remove_file(&path)
        } else {
            Ok(())
        };
        removed.map_err(|error| {
            CannotAsk::Scratch(ScratchFault {
                doing: "remove",
                path,
                kind: error.kind(),
            })
        })?;
    }
    Ok(compare(root, scratch))
}

/// The check of the files committed under `root`, through `init`, using `scratch`, as it ends.
pub fn check(root: &Path, scratch: &Path, init: &dyn Init) -> Report {
    match ask(root, scratch, init) {
        // On standard output, where every line of this check is, as the Python's was.
        Err(reason) => Report {
            exit: Exit::CouldNotAsk,
            out: vec![reason.to_string()],
            err: Vec::new(),
        },
        Ok(found) if found.is_empty() => Report {
            exit: Exit::Clean,
            out: vec![format!(
                "  ok  {} installed instruction file(s) match what `init` writes",
                counted(root)
            )],
            err: Vec::new(),
        },
        Ok(found) => {
            let mut out: Vec<String> = found
                .iter()
                .map(|drift| format!("  FAIL  {drift}"))
                .collect();
            out.extend([
                String::new(),
                String::from(
                    "the installed instruction files are not what this binary writes. An agent \
                     reads the committed copy, so a body changed in the source and not regenerated \
                     is a body no agent sees.",
                ),
                String::new(),
                format!("  htmlspec init --tools {TOOLS}"),
                String::new(),
                String::from("then review the diff and commit it."),
            ]);
            Report {
                exit: Exit::Faults,
                out,
                err: Vec::new(),
            }
        }
    }
}

/// Runs the check on the repository the current directory belongs to.
pub fn run() -> ExitCode {
    let root = match Workspace::here() {
        Ok(workspace) => workspace.root,
        Err(fault) => return Report::could_not_ask(&fault).print(),
    };
    let scratch = match new_scratch() {
        Ok(scratch) => scratch,
        Err(fault) => return Report::could_not_ask(&fault).print(),
    };
    let report = check(&root, &scratch, &Binary);
    // Removed whatever the check found. A directory the removal leaves behind under the temporary
    // directory misleads nobody, and is not worth failing a clean check over.
    let _ = std::fs::remove_dir_all(&scratch);
    report.print()
}

#[cfg(test)]
mod tests;
