//! `cargo xtask check-readme-install`: the README's install and upgrade blocks, checked offline;
//! and `--print NAME`, which prints one install block for the jobs that run it.
//!
//! # What it holds the blocks to
//!
//! The README installs and upgrades htmlspec with commands a reader pastes. They are its own words
//! and not the contract, but a reader who pastes a broken one blames themselves, so the commands are
//! checked. Each block a reader runs is marked by a comment on the line before its fence, `install:`
//! for a block that installs and `upgrade:` for one that upgrades, and the two share one set of
//! names:
//!
//! ````text
//! <!-- install: linux-release -->
//! ```sh
//! ...
//! ```
//! ````
//!
//! The check fails when:
//!
//! - an install block is missing: `linux-release` and `linux-homebrew`, which the release workflow
//!   runs, and `cargo` and `macos-homebrew`, the routes the README gives beside them;
//! - an upgrade block is missing: `upgrade-homebrew`, `upgrade-cargo`, `upgrade-nix`, and
//!   `upgrade-project`, or one of them is marked as an install block. The upgrade blocks are parsed
//!   and never run, because running them would upgrade the machine the check runs on;
//! - a name marks two blocks, or a marker has no fenced block after it;
//! - `bash -n` refuses a marked block;
//! - the Linux block unpacks or installs before it checks the download, that is, it has no
//!   `sha256sum --check … SHA256SUMS` line ahead of its `tar -x` and `install` lines;
//! - the Linux block could pick a target the release does not build;
//! - the Linux block's reading of `SHA256SUMS` no longer finds each Linux tarball in the file the
//!   packer's `--sums` writes.
//!
//! The last check writes a `SHA256SUMS` the way a release does, over tarballs named for every
//! target, with the packer's own [`pack_release::write_sums`], in a new scratch directory, and runs
//! the block's own line that picks a name out of it, for each Linux architecture, with `bash`. It
//! needs no network, so it runs on every pull request and inside the Nix sandbox. What only a
//! published release can show, the downloads, is the release workflow's `install` jobs, which run
//! the block `--print` printed, verbatim, in an empty home directory. `--print` prints only an
//! install block: an upgrade block upgrades whatever machine runs it, so it is checked here and
//! never handed to a job. No block is run: `bash -n` only parses, and the one line run is the Linux
//! block's pick line, alone, in the scratch directory, which is its home too. That line is the
//! README's own text, so this check is no sandbox for it: it is held to `file="$(grep ...)"`, and a
//! `~` or `$HOME` in it reaches the scratch directory, but a path it names outright is read as named.
//!
//! It reads `README.md` in the directory it runs in, the repository's root. Exit codes: 0 when every
//! check passes, or with the block printed; 1 when a check fails, or the block asked for is an
//! upgrade block; and 2 when the README cannot be read, `bash` cannot be started, or no block has
//! the name asked for.

use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use crate::facts::{Facts, MacosJobs, Target};
use crate::pack_release;
use crate::report::{Exit, Report};
use crate::temporary::TemporaryDirectory;

/// The README, in the directory the tool runs in.
const README: &str = "README.md";

/// Every route the README gives. The release workflow extracts and runs the first two, so a README
/// without them breaks the release. The other two are what a reader on cargo or macOS pastes, and a
/// route that quietly vanished from the README is the gap this check exists to catch.
pub const REQUIRED: [&str; 4] = ["linux-release", "linux-homebrew", "cargo", "macos-homebrew"];

/// The upgrade route of each channel the README installs from, apart from the Linux release block,
/// which is its own upgrade: running it again installs the newer release. And the steps a project
/// takes once its binary is newer.
pub const REQUIRED_UPGRADE: [&str; 4] = [
    "upgrade-homebrew",
    "upgrade-cargo",
    "upgrade-nix",
    "upgrade-project",
];

/// What `uname -m` prints on the machines the Linux builds are for.
const LINUX_MACHINES: [&str; 2] = ["x86_64", "aarch64"];

/// A machine no release builds for, whose pick must find nothing.
const OTHER_MACHINE: &str = "riscv64";

/// The version the scratch tarballs are named for.
const SCRATCH_VERSION: &str = "0.0.0";

/// Which kind of block a marker marks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Install,
    Upgrade,
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Install => "install",
            Self::Upgrade => "upgrade",
        })
    }
}

/// A marked block: its kind, and its body, every line up to the closing fence, each ending in a
/// newline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: Kind,
    pub body: String,
}

/// A check that failed, with the sentence that says why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    MarkedTwice {
        kind: Kind,
        name: String,
    },
    NoFence {
        kind: Kind,
        name: String,
    },
    Unclosed {
        kind: Kind,
        name: String,
    },
    Missing {
        kind: Kind,
        name: String,
    },
    WrongKind {
        name: String,
        marked: Kind,
        kind: Kind,
    },
    Syntax {
        kind: Kind,
        name: String,
        said: String,
    },
    NoChecksum,
    UnpacksFirst,
    NoTarget,
    UnbuiltTarget {
        machine: String,
        suffix: String,
        built: Vec<Target>,
    },
    NoPick,
    Picks {
        picked: String,
        machine: String,
        listed: String,
    },
    Sums(String),
    UpgradePrinted {
        name: String,
    },
}

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MarkedTwice { kind, name } => {
                write!(
                    formatter,
                    "`{kind}: {name}` marks two blocks; each name marks one"
                )
            }
            Self::NoFence { kind, name } => write!(
                formatter,
                "`{kind}: {name}` is not followed by a ```sh fence on the next line"
            ),
            Self::Unclosed { kind, name } => {
                write!(
                    formatter,
                    "`{kind}: {name}` opens a fence that never closes"
                )
            }
            Self::Missing { kind, name } => {
                write!(formatter, "no block is marked `{kind}: {name}`")
            }
            Self::WrongKind { name, marked, kind } => write!(
                formatter,
                "`{name}` is marked `{marked}:`, and it is one of the README's {kind} blocks: mark \
                 it `{kind}: {name}`"
            ),
            Self::Syntax { kind, name, said } => {
                write!(formatter, "`{kind}: {name}` is refused by bash -n: {said}")
            }
            Self::NoChecksum => formatter.write_str(
                "`install: linux-release` has no `sha256sum --check … SHA256SUMS` line, so it \
                 would install a download nobody checked",
            ),
            Self::UnpacksFirst => formatter.write_str(
                "`install: linux-release` unpacks or installs before its `sha256sum --check` line",
            ),
            Self::NoTarget => formatter.write_str(
                "`install: linux-release` sets no target=\"$(uname -m)-…\", so nothing checks it",
            ),
            Self::UnbuiltTarget {
                machine,
                suffix,
                built,
            } => write!(
                formatter,
                "`install: linux-release` would ask for {machine}-{suffix}, and the release builds \
                 {}",
                built
                    .iter()
                    .map(|target| target.triple())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::NoPick => formatter.write_str(
                "`install: linux-release` has no file=\"$(grep … SHA256SUMS …)\" line to check",
            ),
            Self::Picks {
                picked,
                machine,
                listed,
            } => write!(
                formatter,
                "`install: linux-release` picks '{picked}' out of SHA256SUMS for {machine}, where \
                 pack-release lists '{listed}'"
            ),
            Self::Sums(reason) => write!(formatter, "pack-release --sums failed: {reason}"),
            Self::UpgradePrinted { name } => write!(
                formatter,
                "`upgrade: {name}` is an upgrade block, and --print prints only install blocks: an \
                 upgrade block is checked and never run"
            ),
        }
    }
}

/// Why the check could not ask: `bash` could not be run, or its scratch directory not made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CannotAsk {
    Bash(std::io::ErrorKind),
    Scratch(std::io::ErrorKind),
}

impl fmt::Display for CannotAsk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bash(kind) => write!(formatter, "bash is required, and could not be run: {kind}"),
            Self::Scratch(kind) => write!(formatter, "cannot make a scratch directory: {kind}"),
        }
    }
}

/// A marker line, `<!-- install: NAME -->` or `<!-- upgrade: NAME -->`, as its kind and name.
fn marker(line: &str) -> Option<(Kind, &str)> {
    let inner = line
        .trim_end()
        .strip_prefix("<!-- ")?
        .strip_suffix(" -->")?;
    let (kind, name) = if let Some(name) = inner.strip_prefix("install: ") {
        (Kind::Install, name)
    } else {
        (Kind::Upgrade, inner.strip_prefix("upgrade: ")?)
    };
    let mut characters = name.chars();
    let first = characters.next()?;
    let lowercase_or_digit =
        |character: char| character.is_ascii_lowercase() || character.is_ascii_digit();
    (lowercase_or_digit(first)
        && characters.all(|character| lowercase_or_digit(character) || character == '-'))
    .then_some((kind, name))
}

/// Whether `line` opens a `sh` or `bash` fence.
fn opens_fence(line: &str) -> bool {
    matches!(line.trim_end(), "```sh" | "```bash")
}

/// Every marked block, by name, in the order the README gives them.
///
/// A marker must sit on the line just before a `sh` fence. Install and upgrade blocks share one set
/// of names, so a name marks one block whichever kind it is.
pub fn marked(text: &str) -> Result<Vec<(String, Block)>, Fault> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found: Vec<(String, Block)> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some((kind, name)) = marker(line) else {
            continue;
        };
        let owned = name.to_owned();
        if found.iter().any(|(seen, _)| *seen == owned) {
            return Err(Fault::MarkedTwice { kind, name: owned });
        }
        if !lines.get(index + 1).is_some_and(|next| opens_fence(next)) {
            return Err(Fault::NoFence { kind, name: owned });
        }
        let following = lines.get(index + 2..).unwrap_or_default();
        let Some(end) = following.iter().position(|line| line.starts_with("```")) else {
            return Err(Fault::Unclosed { kind, name: owned });
        };
        // Every line joined and one newline after, so a block with no lines is one empty line.
        let body = format!("{}\n", following[..end].join("\n"));
        found.push((owned, Block { kind, body }));
    }
    Ok(found)
}

/// Runs `bash` as it checks a block: with the script on standard input, or with `-c`.
pub trait Bash {
    /// `bash -n`, reading `body`: `Ok(None)` where bash accepts it, `Ok(Some(what bash said))`
    /// where it refuses it.
    fn parse(&self, body: &str) -> Result<Option<String>, CannotAsk>;
    /// `bash -c script` in `directory`, with what it printed.
    fn run(&self, script: &str, directory: &Path) -> Result<String, CannotAsk>;
}

/// The `bash` on `PATH`, given nothing of this process's environment but `PATH` and the home
/// directory it is handed, so a `~` or `$HOME` in what it runs names that directory, and nothing it
/// runs inherits a variable of whoever runs the check.
pub struct RealBash;

/// `bash` with an environment of `PATH` alone, and the home directory given.
fn bash(home: &Path) -> Command {
    let mut process = Command::new("bash");
    process.env_clear().env("HOME", home);
    if let Some(path) = std::env::var_os("PATH") {
        process.env("PATH", path);
    }
    process
}

impl Bash for RealBash {
    fn parse(&self, body: &str) -> Result<Option<String>, CannotAsk> {
        // `-n` reads and runs nothing, so the home directory it is given is one that does not exist.
        let mut child = bash(Path::new("/nonexistent"))
            .arg("-n")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| CannotAsk::Bash(error.kind()))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(body.as_bytes())
                .map_err(|error| CannotAsk::Bash(error.kind()))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|error| CannotAsk::Bash(error.kind()))?;
        Ok((!output.status.success())
            .then(|| String::from_utf8_lossy(&output.stderr).trim().to_owned()))
    }

    /// The scratch directory is the script's home too, so a pick line that names `~` or `$HOME`
    /// reaches the scratch directory and never the home of whoever runs the check.
    fn run(&self, script: &str, directory: &Path) -> Result<String, CannotAsk> {
        let output = bash(directory)
            .args(["-c", script])
            .current_dir(directory)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|error| CannotAsk::Bash(error.kind()))?;
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

/// Whether `text` holds `word` with no word character on either side of it.
fn has_word(text: &str, word: &str) -> bool {
    let is_word = |character: char| character.is_alphanumeric() || character == '_';
    text.match_indices(word).any(|(start, _)| {
        !text[..start].chars().next_back().is_some_and(is_word)
            && !text[start + word.len()..]
                .chars()
                .next()
                .is_some_and(is_word)
    })
}

/// Whether `line` checks the download: `sha256sum --check`, a word ending there, and then
/// `SHA256SUMS` as a word.
fn checks_the_download(line: &str) -> bool {
    let command = "sha256sum --check";
    let Some(rest) = line.trim_start().strip_prefix(command) else {
        return false;
    };
    let ends_a_word = !rest
        .chars()
        .next()
        .is_some_and(|character| character.is_alphanumeric() || character == '_');
    ends_a_word && has_word(rest, "SHA256SUMS")
}

/// Whether `line` unpacks with `tar -…x…` or installs with `install `.
fn unpacks(line: &str) -> bool {
    let line = line.trim_start();
    if let Some(rest) = line.strip_prefix("tar") {
        let options = rest.trim_start();
        if options.len() < rest.len() {
            if let Some(flags) = options.strip_prefix('-') {
                let run: String = flags.chars().take_while(char::is_ascii_lowercase).collect();
                if run.contains('x') {
                    return true;
                }
            }
        }
    }
    // A line's end is a space too: every line of a block ends in a newline.
    line.strip_prefix("install")
        .is_some_and(|rest| rest.chars().next().is_none_or(char::is_whitespace))
}

/// The Linux block checks the download against `SHA256SUMS` before it unpacks or installs it.
fn check_checksum_first(body: &str) -> Result<(), Fault> {
    let lines: Vec<&str> = body.lines().collect();
    let Some(checksum) = lines.iter().position(|line| checks_the_download(line)) else {
        return Err(Fault::NoChecksum);
    };
    match lines.iter().position(|line| unpacks(line)) {
        Some(unpack) if unpack < checksum => Err(Fault::UnpacksFirst),
        _ => Ok(()),
    }
}

/// The suffix the Linux block appends to `uname -m`, from its `target="$(uname -m)-…"`.
fn target_suffix(body: &str) -> Option<&str> {
    let prefix = "target=\"$(uname -m)-";
    body.match_indices(prefix).find_map(|(start, _)| {
        let rest = &body[start + prefix.len()..];
        let length = rest
            .find(|character: char| {
                !(character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || matches!(character, '_' | '-'))
            })
            .unwrap_or(rest.len());
        let suffix = &rest[..length];
        let parts: Vec<&str> = suffix.split('-').collect();
        (rest[length..].starts_with('"')
            && parts.len() >= 2
            && parts.iter().all(|part| !part.is_empty()))
        .then_some(suffix)
    })
}

/// The suffix, held to the targets a release builds.
fn check_targets<'a>(body: &'a str, built: &[Target]) -> Result<&'a str, Fault> {
    let suffix = target_suffix(body).ok_or(Fault::NoTarget)?;
    for machine in LINUX_MACHINES {
        let triple = format!("{machine}-{suffix}");
        if !Target::from_triple(&triple).is_some_and(|target| built.contains(&target)) {
            return Err(Fault::UnbuiltTarget {
                machine: machine.to_owned(),
                suffix: suffix.to_owned(),
                built: built.to_vec(),
            });
        }
    }
    Ok(suffix)
}

/// The block's line that picks a tarball's name out of `SHA256SUMS`: `file="$(grep …SHA256SUMS…)"`.
fn pick_line(body: &str) -> Option<&str> {
    body.lines().map(str::trim).find(|line| {
        line.strip_prefix("file=\"$(grep ")
            .and_then(|rest| rest.strip_suffix(")\""))
            .is_some_and(|inside| inside.contains("SHA256SUMS"))
    })
}

/// The block's own pick line finds each Linux tarball the packer lists, and nothing for a machine
/// no release builds for, in a `SHA256SUMS` written as a release writes it in `scratch`.
fn check_sums(
    body: &str,
    suffix: &str,
    built: &[Target],
    bash: &dyn Bash,
    scratch: &Path,
) -> Result<Result<(), Fault>, CannotAsk> {
    let Some(pick) = pick_line(body) else {
        return Ok(Err(Fault::NoPick));
    };
    for target in built {
        let name = format!("htmlspec-{SCRATCH_VERSION}-{target}.tar.gz");
        if let Err(error) = std::fs::write(scratch.join(name), target.triple()) {
            return Err(CannotAsk::Scratch(error.kind()));
        }
    }
    if let Err(fault) = pack_release::write_sums(scratch, built) {
        return Ok(Err(Fault::Sums(fault.to_string())));
    }
    for machine in LINUX_MACHINES.into_iter().chain([OTHER_MACHINE]) {
        let triple = format!("{machine}-{suffix}");
        let script = format!("set -eu\ntarget=\"{triple}\"\n{pick}\nprintf %s \"$file\"\n");
        let picked = bash.run(&script, scratch)?;
        let listed = if Target::from_triple(&triple).is_some_and(|target| built.contains(&target)) {
            format!("htmlspec-{SCRATCH_VERSION}-{triple}.tar.gz")
        } else {
            String::new()
        };
        if picked != listed {
            return Ok(Err(Fault::Picks {
                picked,
                machine: machine.to_owned(),
                listed,
            }));
        }
    }
    Ok(Ok(()))
}

/// Every fault in a README's install blocks, or none: the blocks are read, each is parsed with
/// `bash -n`, and the Linux block's checks run in a new scratch directory `scratch` makes.
pub fn check(
    text: &str,
    built: &[Target],
    bash: &dyn Bash,
    scratch: &dyn Fn() -> Result<TemporaryDirectory, CannotAsk>,
) -> Result<Vec<Fault>, CannotAsk> {
    let every = match marked(text) {
        Ok(every) => every,
        Err(fault) => return Ok(vec![fault]),
    };
    let by_name: BTreeMap<&str, &Block> = every
        .iter()
        .map(|(name, block)| (name.as_str(), block))
        .collect();
    let mut faults = Vec::new();
    for (kind, required) in [(Kind::Install, REQUIRED), (Kind::Upgrade, REQUIRED_UPGRADE)] {
        for name in required {
            match by_name.get(name) {
                None => faults.push(Fault::Missing {
                    kind,
                    name: name.to_owned(),
                }),
                Some(block) if block.kind != kind => faults.push(Fault::WrongKind {
                    name: name.to_owned(),
                    marked: block.kind,
                    kind,
                }),
                Some(_) => {}
            }
        }
    }
    for (name, block) in &every {
        if let Some(said) = bash.parse(&block.body)? {
            faults.push(Fault::Syntax {
                kind: block.kind,
                name: name.clone(),
                said,
            });
        }
    }
    if let Some(block) = by_name.get("linux-release") {
        let linux =
            check_checksum_first(&block.body).and_then(|()| check_targets(&block.body, built));
        match linux {
            Err(fault) => faults.push(fault),
            Ok(suffix) => {
                let scratch = scratch()?;
                if let Err(fault) = check_sums(&block.body, suffix, built, bash, scratch.path())? {
                    faults.push(fault);
                }
            }
        }
    }
    Ok(faults)
}

/// The body of the install block marked `name`, for a job that runs it verbatim, or why it is not
/// printed. Only an install block is printed: an upgrade block upgrades whatever machine runs it,
/// so nothing that runs what this prints may be handed one.
pub fn block_to_print(text: &str, name: &str) -> Result<String, Unprinted> {
    let every = marked(text).map_err(Unprinted::Fault)?;
    let Some((_, block)) = every.iter().find(|(found, _)| found == name) else {
        return Err(Unprinted::NoSuchBlock(name.to_owned()));
    };
    match block.kind {
        Kind::Install => Ok(block.body.clone()),
        Kind::Upgrade => Err(Unprinted::Fault(Fault::UpgradePrinted {
            name: name.to_owned(),
        })),
    }
}

/// Why `--print` printed nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unprinted {
    /// No block has the name: exit 2, as a usage error is.
    NoSuchBlock(String),
    /// The README's markers are at fault, or the block is an upgrade block: exit 1.
    Fault(Fault),
}

/// What the tool was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Job {
    Check,
    Print(String),
}

/// The job's outcome on the README's text, as the run ends.
pub fn report(
    readme: std::io::Result<String>,
    job: &Job,
    built: &[Target],
    bash: &dyn Bash,
    scratch: &dyn Fn() -> Result<TemporaryDirectory, CannotAsk>,
) -> Report {
    let text = match readme {
        Ok(text) => text,
        Err(error) => {
            return Report::could_not_ask(&format!(
                "check-readme-install: cannot read {README}: {}",
                error.kind()
            ))
        }
    };
    match job {
        Job::Print(name) => match block_to_print(&text, name) {
            Ok(body) => Report {
                exit: Exit::Clean,
                out: body.lines().map(str::to_owned).collect(),
                err: Vec::new(),
            },
            Err(Unprinted::NoSuchBlock(name)) => Report::could_not_ask(&format!(
                "check-readme-install: no block is marked `install: {name}`"
            )),
            Err(Unprinted::Fault(fault)) => Report {
                exit: Exit::Faults,
                out: Vec::new(),
                err: vec![format!("check-readme-install: {fault}")],
            },
        },
        Job::Check => match check(&text, built, bash, scratch) {
            Err(reason) => Report::could_not_ask(&format!("check-readme-install: {reason}")),
            Ok(faults) if faults.is_empty() => {
                let mut names: Vec<String> = marked(&text)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect();
                names.sort();
                Report {
                    exit: Exit::Clean,
                    out: vec![format!(
                        "  ok  the README's install and upgrade blocks: {}",
                        names.join(", ")
                    )],
                    err: Vec::new(),
                }
            }
            Ok(faults) => Report {
                exit: Exit::Faults,
                out: Vec::new(),
                err: faults.iter().map(|fault| format!("!!! {fault}")).collect(),
            },
        },
    }
}

/// A new scratch directory for the check of `SHA256SUMS`.
fn new_scratch() -> Result<TemporaryDirectory, CannotAsk> {
    TemporaryDirectory::new("readme-install").map_err(|error| CannotAsk::Scratch(error.kind()))
}

/// Checks the README in the current directory, or prints one of its install blocks.
pub fn run(job: &Job) -> ExitCode {
    let built = match Facts::read() {
        Ok(facts) => facts.targets(MacosJobs::Run),
        Err(fault) => {
            return Report::could_not_ask(&format!(
                "check-readme-install: xtask/release-facts.json {fault}"
            ))
            .print()
        }
    };
    let readme = std::fs::read_to_string(PathBuf::from(README));
    report(readme, job, &built, &RealBash, &new_scratch).print()
}

#[cfg(test)]
mod tests;
