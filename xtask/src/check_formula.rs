//! `cargo xtask check-formula DIST`: check the Homebrew formula for a set of release tarballs,
//! before any of it is published.
//!
//! Current Homebrew refuses to audit or install a formula named by a file path, so the check makes
//! a tap for the purpose. `brew tap-new --no-git` creates it, the formula rendered from DIST's
//! `SHA256SUMS` goes into it with addresses pointing at the tarballs in DIST, and then `brew audit
//! --strict`, `brew install`, and `brew test` each name the formula through that tap. The formula's
//! test runs `htmlspec --version`. The install, the tap, and the trust below are removed
//! afterwards, whatever the outcome.
//!
//! From Homebrew 6, a formula from a tap Homebrew does not maintain loads only once it or its tap is
//! trusted, so the audit is refused until then (<https://docs.brew.sh/Tap-Trust>). The check trusts
//! the one formula it wrote, with `brew trust --formula`, and not the tap, and takes that trust back
//! with `brew untrust --formula` after it removes the tap, so it leaves the trust store as it found
//! it. A trust left behind would also trust the formula of any real tap of that name someone later
//! added, so when the untrust fails the check says so and names the command to run. An older
//! Homebrew has no tap trust and no `trust` command, so the check asks whether the command exists
//! rather than assume it does.
//!
//! Homebrew is kept from updating itself and from sending analytics while it runs, so the check
//! makes no request beyond what an install from a local file needs. It is also kept from
//! autoremoving, so the cleanup's uninstall removes htmlspec and none of the machine's other
//! leftover dependencies.
//!
//! It refuses to run where htmlspec is already installed through Homebrew or the check's tap already
//! exists, rather than replace either, and it renders the formula before it gives Homebrew any
//! command, so a formula that cannot be rendered touches nothing.
//!
//! Every command goes through the [`Brew`] capability. The real one, `Homebrew`, is built in
//! [`run`] alone, which only the release workflow's formula job runs, on a macOS runner with a stock
//! Homebrew, and a test build compiles neither: `Homebrew`, its `Brew` implementation, and the `run`
//! that builds it exist only outside `cfg(test)`. The tests pass a fake Homebrew held in memory,
//! which answers as Homebrew 6 and 7 do and as one from before tap trust does, so no test can reach a
//! machine's Homebrew, whatever its `PATH` says: there is none in the build to reach it with.
//!
//! Exit codes: 0 when the formula audits, installs, and passes its test; 1 when a step failed or the
//! formula could not be rendered; and 2 when the check refused to start, or cannot read DIST.

use std::fmt;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use crate::render_formula::{self, BaseUrl, Stopped};
use crate::report::Exit;

/// A tap that exists only for the length of the check, on this machine.
pub const TAP: &str = "htmlspec-check/local";

/// The formula, named through that tap.
pub const FORMULA: &str = "htmlspec-check/local/htmlspec";

/// What the check prints when every step passed.
pub const PASSED: &str = "the formula audits, installs, and passes its test";

/// The environment every command is given: no self-update, no analytics, no cleanup or autoremove
/// beyond what the check names, and no hints.
pub const ENVIRONMENT: [(&str, &str); 5] = [
    ("HOMEBREW_NO_AUTO_UPDATE", "1"),
    ("HOMEBREW_NO_ANALYTICS", "1"),
    ("HOMEBREW_NO_INSTALL_CLEANUP", "1"),
    ("HOMEBREW_NO_ENV_HINTS", "1"),
    ("HOMEBREW_NO_AUTOREMOVE", "1"),
];

/// One command the check gives Homebrew. Each names the check's own tap or formula, and nothing
/// else, so no command can reach another tap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrewCommand {
    /// `brew tap`: the taps Homebrew has.
    Taps,
    /// `brew list --formula htmlspec`: whether htmlspec is installed.
    Installed,
    /// `brew developer`: whether developer mode is on, which `tap-new` turns on.
    Developer,
    /// `brew developer off`.
    DeveloperOff,
    /// `brew trust --help`: whether this Homebrew has tap trust.
    TrustHelp,
    /// `brew tap-new --no-git TAP`.
    TapNew,
    /// `brew --repository TAP`: the tap's directory.
    Repository,
    /// `brew trust --formula FORMULA`.
    Trust,
    /// `brew audit --strict FORMULA`.
    Audit,
    /// `brew install FORMULA`.
    Install,
    /// `brew test FORMULA`.
    Test,
    /// `brew --prefix`.
    Prefix,
    /// `brew uninstall --formula FORMULA`.
    Uninstall,
    /// `brew untap TAP`.
    Untap,
    /// `brew untrust --formula FORMULA`.
    Untrust,
}

impl BrewCommand {
    /// The command's arguments, after `brew`.
    pub fn arguments(self) -> &'static [&'static str] {
        match self {
            Self::Taps => &["tap"],
            Self::Installed => &["list", "--formula", "htmlspec"],
            Self::Developer => &["developer"],
            Self::DeveloperOff => &["developer", "off"],
            Self::TrustHelp => &["trust", "--help"],
            Self::TapNew => &["tap-new", "--no-git", TAP],
            Self::Repository => &["--repository", TAP],
            Self::Trust => &["trust", "--formula", FORMULA],
            Self::Audit => &["audit", "--strict", FORMULA],
            Self::Install => &["install", FORMULA],
            Self::Test => &["test", FORMULA],
            Self::Prefix => &["--prefix"],
            Self::Uninstall => &["uninstall", "--formula", FORMULA],
            Self::Untap => &["untap", TAP],
            Self::Untrust => &["untrust", "--formula", FORMULA],
        }
    }

    /// Whether its output is the check's to read, or to discard, rather than the reader's to see.
    /// The steps the check exists for show Homebrew's own words.
    pub fn is_query(self) -> bool {
        !matches!(
            self,
            Self::TapNew | Self::Trust | Self::Audit | Self::Install | Self::Test
        )
    }
}

impl fmt::Display for BrewCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "brew {}", self.arguments().join(" "))
    }
}

/// What one command did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ran {
    /// It exited 0. A query's standard output is kept; another command's went to the reader.
    Succeeded { stdout: String },
    /// It exited otherwise.
    Failed,
    /// It could not be started.
    NotStarted(ErrorKind),
}

impl Ran {
    fn succeeded(&self) -> bool {
        matches!(self, Self::Succeeded { .. })
    }
}

/// Homebrew, as the check uses it.
pub trait Brew {
    /// Gives Homebrew one command.
    fn run(&mut self, command: BrewCommand) -> Ran;
    /// Writes the formula into the tap's directory, at `Formula/htmlspec.rb` beneath it.
    fn write_formula(&mut self, repository: &Path, formula: &str) -> Result<(), ErrorKind>;
    /// Runs `bin/htmlspec --version` beneath Homebrew's prefix.
    fn installed_version(&mut self, prefix: &Path) -> Ran;
}

/// One line the check prints, to standard output or to standard error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Out(String),
    Err(String),
}

/// A step that failed.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Failure {
    Command(BrewCommand, Ran),
    Formula {
        repository: PathBuf,
        kind: ErrorKind,
    },
    Version(Ran),
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let how = |ran: &Ran| match ran {
            Ran::NotStarted(kind) => format!("could not be started: {kind}"),
            Ran::Failed | Ran::Succeeded { .. } => String::from("failed"),
        };
        match self {
            Self::Command(command, ran) => write!(formatter, "{command} {}", how(ran)),
            Self::Formula { repository, kind } => write!(
                formatter,
                "the formula could not be written into {}: {kind}",
                repository.display()
            ),
            Self::Version(ran) => {
                write!(formatter, "the installed htmlspec --version {}", how(ran))
            }
        }
    }
}

/// Gives `brew` the command, and fails naming it unless it succeeded.
fn step(brew: &mut dyn Brew, command: BrewCommand) -> Result<String, Failure> {
    match brew.run(command) {
        Ran::Succeeded { stdout } => Ok(stdout),
        other => Err(Failure::Command(command, other)),
    }
}

/// The steps between making the tap and the end of the test, stopping at the first that fails.
fn steps(
    brew: &mut dyn Brew,
    formula: &str,
    trust: bool,
    say: &mut dyn FnMut(Line),
) -> Result<(), Failure> {
    step(brew, BrewCommand::TapNew)?;
    let repository = PathBuf::from(step(brew, BrewCommand::Repository)?.trim());
    brew.write_formula(&repository, formula)
        .map_err(|kind| Failure::Formula { repository, kind })?;
    if trust {
        step(brew, BrewCommand::Trust)?;
    }
    for (header, command) in [
        ("brew audit --strict", BrewCommand::Audit),
        ("brew install", BrewCommand::Install),
        ("brew test", BrewCommand::Test),
    ] {
        say(Line::Out(String::new()));
        say(Line::Out(format!("=== {header} ===")));
        step(brew, command)?;
    }
    let prefix = PathBuf::from(step(brew, BrewCommand::Prefix)?.trim());
    match brew.installed_version(&prefix) {
        Ran::Succeeded { .. } => Ok(()),
        other => Err(Failure::Version(other)),
    }
}

/// Removes what the check made, whatever the outcome: the install, then the tap, then the trust,
/// then developer mode where the check turned it on.
fn cleanup(brew: &mut dyn Brew, trust: bool, developer: &str, say: &mut dyn FnMut(Line)) {
    brew.run(BrewCommand::Uninstall);
    brew.run(BrewCommand::Untap);
    // After the untap. Where the uninstall failed and the formula is still installed, `brew untap`
    // loads it, and an untrusted formula makes the untap fail and leaves the tap behind. A
    // successful uninstall may already have taken the trust back, and the untrust then finds
    // nothing to remove.
    if trust && !brew.run(BrewCommand::Untrust).succeeded() {
        say(Line::Err(format!(
            "warning: Homebrew still trusts {FORMULA}, and would trust that formula in any tap of \
             that name; run: {}",
            BrewCommand::Untrust
        )));
    }
    if developer.contains("disabled") {
        brew.run(BrewCommand::DeveloperOff);
    }
}

/// The check of `formula`, through `brew`, saying each line as it goes, as the run ends.
pub fn check(brew: &mut dyn Brew, formula: &str, say: &mut dyn FnMut(Line)) -> Exit {
    match brew.run(BrewCommand::Taps) {
        Ran::Succeeded { stdout } if stdout.lines().any(|line| line == TAP) => {
            say(Line::Err(format!(
                "the tap {TAP} already exists; remove it with brew untap {TAP} first"
            )));
            return Exit::CouldNotAsk;
        }
        Ran::Succeeded { .. } => {}
        other => {
            say(Line::Err(format!(
                "check-formula: {}",
                Failure::Command(BrewCommand::Taps, other)
            )));
            return Exit::CouldNotAsk;
        }
    }
    if brew.run(BrewCommand::Installed).succeeded() {
        say(Line::Err(String::from(
            "htmlspec is already installed through Homebrew, and this check would replace it",
        )));
        return Exit::CouldNotAsk;
    }
    // `tap-new` is a developer command, and Homebrew turns developer mode on when one runs. The
    // check puts the setting back as it found it, because developer mode changes what `brew update`
    // installs afterwards.
    let developer = match brew.run(BrewCommand::Developer) {
        Ran::Succeeded { stdout } => stdout.lines().next().unwrap_or_default().to_owned(),
        Ran::Failed | Ran::NotStarted(_) => String::new(),
    };
    // Whether this Homebrew has tap trust. An unknown command's `--help` fails, a known one's
    // succeeds.
    let trust = brew.run(BrewCommand::TrustHelp).succeeded();
    let outcome = steps(brew, formula, trust, say);
    cleanup(brew, trust, &developer, say);
    match outcome {
        Ok(()) => {
            say(Line::Out(String::new()));
            say(Line::Out(String::from(PASSED)));
            Exit::Clean
        }
        Err(failure) => {
            say(Line::Err(format!("check-formula: {failure}")));
            Exit::Faults
        }
    }
}

/// The process the real Homebrew runs a command as: `brew` with the command's arguments and the
/// check's environment, reading nothing from standard input. It only builds the process: starting
/// it is [`Homebrew`]'s, which a test build does not compile, so a test can read what it would
/// start and cannot start it.
pub fn homebrew_command(command: BrewCommand) -> Command {
    let mut process = Command::new("brew");
    process.args(command.arguments());
    for (name, value) in ENVIRONMENT {
        process.env(name, value);
    }
    process.stdin(Stdio::null());
    process
}

/// The machine's Homebrew, the `brew` on `PATH`. Built by [`run`] alone, and compiled only outside
/// a test build: no test can construct it, whatever it is handed, so no test can reach a machine's
/// Homebrew.
#[cfg(not(test))]
pub struct Homebrew;

#[cfg(not(test))]
impl Brew for Homebrew {
    fn run(&mut self, command: BrewCommand) -> Ran {
        let mut process = homebrew_command(command);
        if command.is_query() {
            match process.output() {
                Ok(output) if output.status.success() => Ran::Succeeded {
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                },
                Ok(_) => Ran::Failed,
                Err(error) => Ran::NotStarted(error.kind()),
            }
        } else {
            match process.status() {
                Ok(status) if status.success() => Ran::Succeeded {
                    stdout: String::new(),
                },
                Ok(_) => Ran::Failed,
                Err(error) => Ran::NotStarted(error.kind()),
            }
        }
    }

    fn write_formula(&mut self, repository: &Path, formula: &str) -> Result<(), ErrorKind> {
        std::fs::write(repository.join("Formula").join("htmlspec.rb"), formula)
            .map_err(|error| error.kind())
    }

    fn installed_version(&mut self, prefix: &Path) -> Ran {
        let mut process = Command::new(prefix.join("bin").join("htmlspec"));
        process.arg("--version").stdin(Stdio::null());
        for (name, value) in ENVIRONMENT {
            process.env(name, value);
        }
        match process.status() {
            Ok(status) if status.success() => Ran::Succeeded {
                stdout: String::new(),
            },
            Ok(_) => Ran::Failed,
            Err(error) => Ran::NotStarted(error.kind()),
        }
    }
}

/// The formula for the tarballs in `dist`, fetched from there, or why it was not rendered.
fn local_formula(dist: &Path) -> Result<String, Stopped> {
    let sums = dist.join("SHA256SUMS");
    if !sums.is_file() {
        return Err(Stopped::CouldNotAsk(format!(
            "usage: cargo xtask check-formula DIST, where DIST holds the tarballs and SHA256SUMS; \
             {} is not a file",
            sums.display()
        )));
    }
    let absolute = std::fs::canonicalize(dist).map_err(|error| {
        Stopped::CouldNotAsk(format!("cannot read {}: {}", dist.display(), error.kind()))
    })?;
    render_formula::from_files(
        &sums,
        Some(BaseUrl::new(format!("file://{}", absolute.display()))),
    )
}

/// Checks the formula for the tarballs in `dist` with the machine's Homebrew.
#[cfg(not(test))]
pub fn run(dist: &Path) -> ExitCode {
    let formula = match local_formula(dist) {
        Ok(formula) => formula,
        Err(stopped) => {
            eprintln!("check-formula: {stopped}");
            return ExitCode::from(stopped.exit().code());
        }
    };
    let exit = check(&mut Homebrew, &formula, &mut |line| match line {
        Line::Out(text) => println!("{text}"),
        Line::Err(text) => eprintln!("{text}"),
    });
    ExitCode::from(exit.code())
}

/// The entry point a test build compiles in its place, so the command line still type-checks. A
/// test build has no Homebrew to give commands to, and its harness never calls this: it renders
/// nothing, runs nothing, and says so, exiting 2.
#[cfg(test)]
pub fn run(_dist: &Path) -> ExitCode {
    eprintln!("check-formula: a test build has no Homebrew to give commands to");
    ExitCode::from(Exit::CouldNotAsk.code())
}

#[cfg(test)]
mod tests;
