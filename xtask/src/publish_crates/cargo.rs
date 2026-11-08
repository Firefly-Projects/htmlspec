//! `cargo publish`, for one crate, and what its error says.
//!
//! cargo builds a crate before it uploads it, so a crate refused with 429 has already built, and the
//! next attempt passes `--no-verify` rather than build it again. Whether a publish landed is the
//! registry's to say, so only cargo's own error is read here: the last line it prints that begins
//! `error: ` and the causes under it. The build's log comes before it and can hold any words at
//! all, so `429` or `already exists` in a compile error never reads as an answer from crates.io.

use std::ffi::OsString;
use std::fmt;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use super::retry::{HttpDate, RetryAfter};
use crate::facts::UserAgent;
use crate::workspace::{CrateName, PackageVersion};

/// Whether `cargo publish` builds the crate before it uploads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verification {
    /// Builds it, as cargo does by default.
    Build,
    /// Uploads it without building it again, with `--no-verify`, because an attempt this run made
    /// built it before crates.io refused the upload.
    AlreadyBuilt,
}

/// What one `cargo publish` did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CargoRun {
    Succeeded,
    Failed {
        stderr: String,
    },
    /// cargo could not be started.
    NotStarted(ErrorKind),
}

/// Publishes one crate.
pub trait Cargo {
    fn publish(&self, name: &CrateName, verification: Verification) -> CargoRun;
}

/// What a publish came to, read from what cargo did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CargoAnswer {
    /// cargo uploaded the crate, or found the registry already holds this version.
    Published,
    /// crates.io refused the upload with 429, naming when to try again.
    RateLimited(RetryAfter),
    /// Anything else.
    Failed(CargoFailure),
}

/// Why a publish failed, other than crates.io's 429.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CargoFailure {
    /// cargo could not be started.
    NotStarted { name: CrateName, kind: ErrorKind },
    /// cargo failed and printed no `error:` line of its own.
    Silent { name: CrateName },
    /// cargo's own error, the last `error:` line it printed, as it printed it but the prefix.
    Said { name: CrateName, error: String },
}

impl fmt::Display for CargoFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted { name, kind } => write!(
                formatter,
                "cargo publish -p {name}: cargo could not be run: {kind}"
            ),
            Self::Silent { name } => write!(
                formatter,
                "cargo publish -p {name} failed and printed no error of its own"
            ),
            Self::Said { name, error } => write!(formatter, "cargo publish -p {name}: {error}"),
        }
    }
}

/// What `run` comes to for `name` at `version`.
pub fn answer(run: CargoRun, name: &CrateName, version: &PackageVersion) -> CargoAnswer {
    match run {
        CargoRun::Succeeded => CargoAnswer::Published,
        CargoRun::Failed { stderr } => read_failure(&stderr, name, version),
        CargoRun::NotStarted(kind) => CargoAnswer::Failed(CargoFailure::NotStarted {
            name: name.clone(),
            kind,
        }),
    }
}

/// cargo's own error: the last line it prints that begins `error: `, and every line after it.
fn final_error(stderr: &str) -> Vec<&str> {
    let lines: Vec<&str> = stderr.lines().collect();
    match lines.iter().rposition(|line| line.starts_with("error: ")) {
        Some(start) => lines.get(start..).map(<[&str]>::to_vec).unwrap_or_default(),
        None => Vec::new(),
    }
}

/// What cargo's own error says about a publish that exited unsuccessfully.
fn read_failure(stderr: &str, name: &CrateName, version: &PackageVersion) -> CargoAnswer {
    let error = final_error(stderr);
    let Some((first, causes)) = error.split_first() else {
        return CargoAnswer::Failed(CargoFailure::Silent { name: name.clone() });
    };
    // `error: crate NAME@VERSION already exists on crates.io index`: the index had not shown it
    // yet when asked, and cargo found it, so it is published.
    let already = format!("error: crate {name}@{version} already exists on ");
    if first
        .trim()
        .strip_prefix(&already)
        .and_then(|rest| rest.strip_suffix(" index"))
        .is_some_and(|registry| !registry.is_empty())
    {
        return CargoAnswer::Published;
    }
    let upload_failed = first
        .strip_prefix("error: failed to publish")
        .is_some_and(|rest| rest.is_empty() || !rest.starts_with(is_word));
    if upload_failed && causes.iter().any(|line| names_status_429(line)) {
        let named = error.iter().find_map(|line| try_again_after(line));
        return CargoAnswer::RateLimited(named.map_or(RetryAfter::Unnamed, RetryAfter::Date));
    }
    CargoAnswer::Failed(CargoFailure::Said {
        name: name.clone(),
        error: first.strip_prefix("error: ").unwrap_or(first).to_owned(),
    })
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// Whether `line` names the status 429, as `(status 429 Too Many Requests)`.
fn names_status_429(line: &str) -> bool {
    line.match_indices("(status 429").any(|(at, found)| {
        !line
            .get(at + found.len()..)
            .and_then(|rest| rest.chars().next())
            .is_some_and(is_word)
    })
}

/// The time crates.io names in `Please try again after <HTTP date>`, where `line` names one.
fn try_again_after(line: &str) -> Option<HttpDate> {
    let (_, rest) = line.split_once("try again after ")?;
    let end = rest.find(" GMT")? + " GMT".len();
    HttpDate::parse(rest.get(..end)?)
}

/// cargo itself, in the workspace's root.
pub struct RealCargo {
    program: OsString,
    root: PathBuf,
    user_agent: UserAgent,
}

impl RealCargo {
    pub fn new(program: OsString, root: PathBuf, user_agent: UserAgent) -> Self {
        Self {
            program,
            root,
            user_agent,
        }
    }

    /// `cargo publish --locked [--no-verify] -p NAME`, with `CARGO_HTTP_USER_AGENT` set, so every
    /// request cargo makes carries the release's User-Agent. cargo reads its token from
    /// `CARGO_REGISTRY_TOKEN`, and nothing here reads it.
    fn command(&self, name: &CrateName, verification: Verification) -> Command {
        let mut command = Command::new(&self.program);
        command
            .current_dir(&self.root)
            .args(["publish", "--locked"])
            .env("CARGO_HTTP_USER_AGENT", self.user_agent.as_str());
        if verification == Verification::AlreadyBuilt {
            command.arg("--no-verify");
        }
        command.args(["-p", name.as_str()]);
        command
    }
}

impl Cargo for RealCargo {
    fn publish(&self, name: &CrateName, verification: Verification) -> CargoRun {
        let output = match self.command(name, verification).output() {
            Ok(output) => output,
            Err(error) => return CargoRun::NotStarted(error.kind()),
        };
        // What cargo said, for the log, before it is read.
        print!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if output.status.success() {
            CargoRun::Succeeded
        } else {
            CargoRun::Failed {
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::publish_crates::registry::tests::{user_agent, USER_AGENT};

    fn name(text: &str) -> CrateName {
        CrateName::parse(text).expect("a crate's name")
    }

    fn version() -> PackageVersion {
        PackageVersion::parse("9.8.7").expect("a version")
    }

    fn read(stderr: &str) -> CargoAnswer {
        read_failure(stderr, &name("htmlspec-cli"), &version())
    }

    /// The command line, and the one variable it sets: every request cargo makes carries the
    /// release's User-Agent, and the lockfile is the one the release was tested with.
    #[test]
    fn cargo_publishes_with_the_fixed_user_agent_and_the_locked_lockfile() {
        let cargo = RealCargo::new("cargo".into(), PathBuf::from("/w"), user_agent());
        for (verification, arguments) in [
            (
                Verification::Build,
                &["publish", "--locked", "-p", "htmlspec-cli"][..],
            ),
            (
                Verification::AlreadyBuilt,
                &["publish", "--locked", "--no-verify", "-p", "htmlspec-cli"],
            ),
        ] {
            let command = cargo.command(&name("htmlspec-cli"), verification);
            let found: Vec<_> = command.get_args().collect();
            assert_eq!(found, arguments);
            let environment: Vec<_> = command.get_envs().collect();
            assert_eq!(
                environment,
                [(
                    std::ffi::OsStr::new("CARGO_HTTP_USER_AGENT"),
                    Some(std::ffi::OsStr::new(USER_AGENT))
                )]
            );
            assert_eq!(command.get_current_dir(), Some(Path::new("/w")));
        }
    }

    #[test]
    fn a_429_from_crates_io_names_when_to_try_again() {
        let stderr = "   Uploading htmlspec-cli v9.8.7\n\
                      error: failed to publish htmlspec-cli v9.8.7 to registry at https://crates.io\n\
                      \n\
                      Caused by:\n  the remote server responded with an error (status 429 Too Many \
                      Requests): You have published too many new crates in a short period of time. \
                      Please try again after Sat, 31 Oct 2026 00:10:00 GMT.\n";
        assert_eq!(
            read(stderr),
            CargoAnswer::RateLimited(RetryAfter::Date(
                HttpDate::parse("Sat, 31 Oct 2026 00:10:00 GMT").expect("a date")
            ))
        );
        let unnamed = stderr.replace("Please try again after Sat, 31 Oct 2026 00:10:00 GMT.", "");
        assert_eq!(
            read(&unnamed),
            CargoAnswer::RateLimited(RetryAfter::Unnamed)
        );
    }

    #[test]
    fn a_version_cargo_finds_already_there_is_published() {
        assert_eq!(
            read("error: crate htmlspec-cli@9.8.7 already exists on crates.io index\n"),
            CargoAnswer::Published
        );
        // Another crate, or another version, is not this one.
        assert!(matches!(
            read("error: crate htmlspec-parse@9.8.7 already exists on crates.io index\n"),
            CargoAnswer::Failed(_)
        ));
    }

    #[test]
    fn only_cargos_last_error_is_read() {
        let stderr = "   Compiling htmlspec-cli v9.8.7\n\
                      error: crate htmlspec-cli@9.8.7 already exists on crates.io index\n\
                      error[E0599]: the test `a_429_too_many_requests_is_retried` is already uploaded\n  \
                      = note: (status 429 Too Many Requests): Please try again after Mon, 01 Jan 2001 00:00:00 GMT\n\
                      error: could not compile `htmlspec-cli` (lib) due to 2 previous errors\n\
                      error: failed to verify package tarball\n";
        let said = CargoFailure::Said {
            name: name("htmlspec-cli"),
            error: String::from("failed to verify package tarball"),
        };
        assert_eq!(
            said.to_string(),
            "cargo publish -p htmlspec-cli: failed to verify package tarball"
        );
        assert_eq!(read(stderr), CargoAnswer::Failed(said));
        let silent = CargoFailure::Silent {
            name: name("htmlspec-cli"),
        };
        assert_eq!(
            silent.to_string(),
            "cargo publish -p htmlspec-cli failed and printed no error of its own"
        );
        assert_eq!(read("warning: nothing\n"), CargoAnswer::Failed(silent));
    }

    #[test]
    fn a_status_is_429_only_as_a_whole_number() {
        assert!(names_status_429("(status 429 Too Many Requests)"));
        assert!(names_status_429("(status 429)"));
        assert!(!names_status_429("(status 4290)"));
        assert!(!names_status_429("(status 42)"));
    }
}
