//! This repository's own tools, as the subcommands of one binary.
//!
//! `.cargo/config.toml` names the alias that runs it, so every tool is `cargo xtask TOOL`, run
//! from the repository's root, where `scripts/check`, the workflows, and the runbooks' steps run.
//! The crate is a member of the root workspace and outside its `default-members`, so a plain
//! `cargo build` or `cargo test` at the root compiles none of it, and it is never published.
//!
//! Each tool is a module with a pure core and a thin edge. The edge reads what the tool needs from
//! the world and reports; the core takes values, so a test passes values rather than shadowing a
//! program on `PATH` or waiting on a clock.
//!
//! The binary's modules live in this library, with the command line behind one entry point,
//! [`run`], which `src/main.rs` calls. The library exists for the private working repository's
//! export, a workspace of its own that depends on this crate by path for what the tools already
//! type: the release's facts, the workspace as `cargo metadata` reads it, and the pull-request
//! rules, which its tests hold its titles to. So it exports those three modules, the rules from
//! the member of their own the pull-request guard builds, `htmlspec-pull-request`, and keeps every
//! other one private, where an item nothing calls still draws the compiler's dead-code warning.

mod check_formula;
mod check_installed;
mod check_readme_install;
mod check_release;
mod crate_notices;
pub mod facts;
mod no_python;
mod pack_release;
mod publish_crates;
mod release_notes;
mod render_formula;
#[cfg(test)]
mod scratch;
mod set_version;
mod temporary;
pub mod workspace;

pub use htmlspec_pull_request as check_pull_request;
use htmlspec_pull_request::report;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};

/// The command line: one subcommand for each tool.
#[derive(Parser)]
#[command(
    name = "cargo xtask",
    bin_name = "cargo xtask",
    about = "This repository's own tools, run from its root as `cargo xtask TOOL`"
)]
struct Cli {
    #[command(subcommand)]
    tool: Tool,
}

#[derive(Subcommand)]
enum Tool {
    /// Fail on Python the repository holds or runs, and on any runnable file but the four
    /// permitted shell files, outside the allow-list and the private list's permit.
    ///
    /// Reads the tracked files: those under `HTMLSPEC_SOURCE_TREE` where the flake sets it, and
    /// otherwise those Git's index lists, each as the working tree holds it. Exits 1 naming each
    /// finding, and 2 when it cannot tell tracked files from anything else.
    NoPython,
    /// Check a pull request's title, and that its diff records the change in CHANGELOG.md.
    ///
    /// The changed files are the three-dot diff against --base, the set GitHub shows under "Files
    /// changed", unless --changed lists them. The `skip-changelog` label excuses the changelog.
    /// Exits 0 when the pull request passes, 1 naming each fault, and 2 when it cannot ask, because
    /// the diff cannot be read or an argument cannot be. The pull-request guard runs the same rules
    /// as the binary of their own member, `check-pull-request`.
    CheckPullRequest(check_pull_request::Arguments),
    /// Check the release record: every archived change records a distinct version with a changelog
    /// entry, every published runbook release is accounted for, and every published crate pins the
    /// declared version and carries the root's licence.
    ///
    /// Reads the archive, the changelog, and the runbooks through the code archival runs, and the
    /// workspace through `cargo metadata`. Exits 0 when the record is coherent, 1 naming each fault,
    /// and 2 when it cannot ask.
    CheckRelease {
        /// A release tag, vX.Y.Z, which must name exactly the version the published crates
        /// declare. The release workflow passes the tag it runs for.
        #[arg(long, value_name = "TAG")]
        tag: Option<String>,
    },
    /// Check that this repository's installed instruction files are what the binary writes, by
    /// running `htmlspec init --tools claude,codex,pi` into a scratch directory and comparing.
    ///
    /// Exits 0 when they match, 1 naming each file that drifted, and 2 when the binary cannot run.
    CheckInstalled,
    /// Move the workspace's version and every internal dependency pin to VERSION in one edit of
    /// Cargo.toml, check the edit through `cargo metadata`, and relock offline.
    ///
    /// Relocks `publish/Cargo.lock` too, where the private workspace exists. When any step fails,
    /// every file it wrote is restored. Exits 0 when the version moved, 1 when a step failed and the
    /// files were restored, and 2 when the manifest cannot be read or edited.
    SetVersion {
        /// The release version: three numbers, written as a release names them, as in 0.33.0.
        #[arg(value_name = "VERSION")]
        version: htmlspec_cli::version::ReleaseVersion,
    },
    /// Print one version's entry from CHANGELOG.md, without its heading, as the notes of its
    /// GitHub release.
    ///
    /// Reads the changelog in the current directory, and needs no cargo. Exits 0 with the notes, 1
    /// when no heading names the version, two do, or its entry is empty, and 2 when the changelog
    /// cannot be read.
    ReleaseNotes {
        /// The release version: three numbers, written as a release names them, as in 0.33.0.
        #[arg(value_name = "VERSION")]
        version: htmlspec_cli::version::ReleaseVersion,
    },
    /// Print a fact the release states once: the User-Agent, the targets a release builds, or the
    /// release workflow's build matrix, from `xtask/release-facts.json`, or the version a release
    /// of this commit names.
    Facts {
        #[command(subcommand)]
        fact: facts::Fact,
    },
    /// Write `crates/htmlspec-cli/CRATE-LICENSES`, the notices of the Rust crates the binary is
    /// built from, for the four targets a release builds.
    ///
    /// Reads the workspace through `cargo metadata`, and the reviewed table beside the file. Exits
    /// 0 when the notices are written or match, 1 naming each crate, row, or difference at fault,
    /// and 2 when the workspace or a file cannot be read.
    CrateNotices {
        /// Compare the committed notices with what the lockfile reaches, and the Homebrew formula's
        /// template with what they credit, and write nothing.
        #[arg(long)]
        check: bool,
    },
    /// Pack a built binary as a release tarball, with its license texts: TARGET BINARY OUT. Or,
    /// with --sums OUT, write OUT/SHA256SUMS over the release tarballs in OUT.
    ///
    /// Every entry's owner, mode, and time is fixed, so the same binary packs to the same bytes.
    /// The version is the one the tools were built at, `[workspace.package]`'s. Exits 0 naming
    /// the file written, 1 when OUT holds what is not one release's tarballs, and 2 when a file
    /// cannot be read or written.
    PackRelease {
        /// Write OUT/SHA256SUMS over every release tarball in OUT.
        #[arg(long, value_name = "OUT", conflicts_with_all = ["target", "binary", "out"])]
        sums: Option<PathBuf>,
        /// The target the binary was built for.
        #[arg(value_name = "TARGET", required_unless_present = "sums")]
        target: Option<facts::Target>,
        /// The built binary.
        #[arg(value_name = "BINARY", required_unless_present = "sums")]
        binary: Option<PathBuf>,
        /// The directory the tarball is written into.
        #[arg(value_name = "OUT", required_unless_present = "sums")]
        out: Option<PathBuf>,
    },
    /// Print the Homebrew formula for a release, rendered from its SHA256SUMS.
    ///
    /// Exits 0 with the formula, 1 when the sums or the template are refused, and 2 when a file
    /// cannot be read.
    RenderFormula {
        /// The release's SHA256SUMS.
        #[arg(long, value_name = "SHA256SUMS")]
        sums: PathBuf,
        /// Where the tarballs are fetched from, `{version}` standing for the version. The version's
        /// GitHub release download directory unless given.
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
    },
    /// Audit, install, and test the Homebrew formula for the tarballs in DIST, in a tap made for
    /// the purpose, with the machine's Homebrew, and remove all of it after.
    ///
    /// Run by the release workflow's formula job, on a macOS runner. Exits 0 when the formula
    /// passes, 1 when a step fails, and 2 when the check refuses to start.
    CheckFormula {
        /// The directory holding the tarballs and SHA256SUMS.
        #[arg(value_name = "DIST")]
        dist: PathBuf,
    },
    /// Check the README's install and upgrade blocks offline, or print one install block.
    ///
    /// Exits 0 when every check passes, 1 naming each fault, and 2 when the README cannot be read,
    /// bash cannot be run, or no block has the name given.
    CheckReadmeInstall {
        /// Print the body of the install block marked NAME, for a job that runs it verbatim.
        #[arg(long, value_name = "NAME")]
        print: Option<String>,
    },
    /// Publish the workspace's crates to crates.io, one at a time in dependency order, skipping
    /// each version the index already holds and waiting out a rate limit within the budget.
    ///
    /// Run from the public repository's root, by the release workflow's crates job. A run stopped
    /// anywhere is finished by running it again. Exits 0 when every crate is published, 1 when the
    /// run stopped naming the crates that remain, and 2 when it could not start, which includes a
    /// workspace that names another repository than the public one.
    PublishCrates {
        /// Ask the index what a run would publish, in what order, and publish nothing. Runs from
        /// any repository, saying when it is not the public one.
        #[arg(long)]
        plan: bool,
        /// Seconds the run may take in all, publishing, asking, and waiting together, and so the
        /// longest any one wait may be. At most 3600.
        #[arg(
            long,
            value_name = "SECONDS",
            default_value_t = publish_crates::DEFAULT_BUDGET.as_secs(),
            value_parser = clap::value_parser!(u64).range(..=publish_crates::MAXIMUM_BUDGET.as_secs())
        )]
        budget: u64,
    },
}

/// Runs the tool the command line names, and gives the exit code it ends with.
pub fn run() -> ExitCode {
    match Cli::parse().tool {
        Tool::NoPython => no_python::run(),
        Tool::CheckPullRequest(arguments) => check_pull_request::run(arguments.job()),
        Tool::CheckRelease { tag } => check_release::run(tag.as_deref()),
        Tool::SetVersion { version } => set_version::run(version),
        Tool::CheckInstalled => check_installed::run(),
        Tool::ReleaseNotes { version } => release_notes::run(version),
        Tool::Facts { fact } => facts::run(fact),
        Tool::CrateNotices { check } => crate_notices::run(if check {
            crate_notices::Mode::Check
        } else {
            crate_notices::Mode::Write
        }),
        Tool::PackRelease {
            sums,
            target,
            binary,
            out,
        } => match packing(sums, target, binary, out) {
            Some(job) => pack_release::run(&job),
            None => {
                eprintln!("pack-release: give TARGET BINARY OUT, or --sums OUT");
                ExitCode::from(2)
            }
        },
        Tool::RenderFormula { sums, base_url } => render_formula::run(&sums, base_url),
        Tool::CheckFormula { dist } => check_formula::run(&dist),
        Tool::CheckReadmeInstall { print } => check_readme_install::run(&match print {
            Some(name) => check_readme_install::Job::Print(name),
            None => check_readme_install::Job::Check,
        }),
        Tool::PublishCrates { plan, budget } => publish_crates::run(
            if plan {
                publish_crates::Mode::Plan
            } else {
                publish_crates::Mode::Publish
            },
            Duration::from_secs(budget),
        ),
    }
}

/// The packer's job, from its arguments: TARGET BINARY OUT, or `--sums` OUT and nothing else. clap
/// refuses every other combination, and this refuses them too, so no combination is assumed away.
fn packing(
    sums: Option<PathBuf>,
    target: Option<facts::Target>,
    binary: Option<PathBuf>,
    out: Option<PathBuf>,
) -> Option<pack_release::Job> {
    match (sums, target, binary, out) {
        (Some(out), None, None, None) => Some(pack_release::Job::Sums { out }),
        (None, Some(target), Some(binary), Some(out)) => Some(pack_release::Job::Pack {
            target,
            binary,
            out,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::{CommandFactory, Parser};

    use super::{facts::Fact, facts::Target, pack_release::Job, packing, Cli, Tool};

    /// clap's own consistency check of the command line.
    #[test]
    fn the_command_line_is_consistent() {
        Cli::command().debug_assert();
    }

    fn parsed(arguments: &[&str]) -> Result<Tool, clap::Error> {
        Cli::try_parse_from(std::iter::once("cargo xtask").chain(arguments.iter().copied()))
            .map(|cli| cli.tool)
    }

    #[test]
    fn each_fact_is_asked_for_by_its_name() {
        for (arguments, fact) in [
            (&["facts", "user-agent"][..], Fact::UserAgent),
            (
                &["facts", "targets"],
                Fact::Targets {
                    without_macos: false,
                },
            ),
            (
                &["facts", "matrix", "--without-macos"],
                Fact::Matrix {
                    without_macos: true,
                },
            ),
        ] {
            match parsed(arguments) {
                Ok(Tool::Facts { fact: found }) => assert_eq!(found, fact, "{arguments:?}"),
                _ => panic!("{arguments:?} did not ask for a fact"),
            }
        }
    }

    /// A budget past the hour is a usage error, so no wait can be longer than an hour.
    #[test]
    fn the_publisher_s_budget_is_at_most_an_hour() {
        for (budget, allowed) in [("0", true), ("120", true), ("3600", true), ("3601", false)] {
            let parsed = parsed(&["publish-crates", "--plan", "--budget", budget]);
            match parsed {
                Ok(Tool::PublishCrates {
                    plan,
                    budget: given,
                }) => {
                    assert!(allowed, "{budget} was taken");
                    assert!(plan);
                    assert_eq!(given.to_string(), budget);
                }
                Ok(_) => panic!("{budget} parsed as another tool"),
                Err(error) => {
                    assert!(!allowed, "{budget} was refused: {error}");
                    assert_eq!(error.exit_code(), 2);
                }
            }
        }
        match parsed(&["publish-crates"]) {
            Ok(Tool::PublishCrates { plan, budget }) => {
                assert!(!plan);
                assert_eq!(budget, 1800);
            }
            _ => panic!("publish-crates did not parse"),
        }
    }

    /// A version argument is a release version: one the one grammar reads and a release writes
    /// as given. Anything else is a usage error, which exits 2, as the Python tools' were.
    #[test]
    fn a_version_argument_is_a_release_version() {
        for tool in ["set-version", "release-notes"] {
            match parsed(&[tool, "1.2.3"]) {
                Ok(Tool::SetVersion { version } | Tool::ReleaseNotes { version }) => {
                    assert_eq!(version.to_string(), "1.2.3")
                }
                _ => panic!("{tool} 1.2.3 did not parse"),
            }
            for version in ["0.1", "0.1.0.0", "01.2.3", "1. 2.3", "+1.2.3", "v1.2.3", ""] {
                let refused = parsed(&[tool, version]).err();
                assert_eq!(
                    refused.map(|error| error.exit_code()),
                    Some(2),
                    "{tool} {version:?} was not a usage error"
                );
            }
        }
    }

    /// The packer takes TARGET BINARY OUT, or `--sums` OUT and nothing else, and a target a release
    /// builds, as the Python's command line did; anything else is a usage error, exit 2.
    #[test]
    fn the_packer_takes_a_target_a_binary_and_a_directory_or_the_sums_alone() {
        match parsed(&[
            "pack-release",
            "x86_64-unknown-linux-musl",
            "target/htmlspec",
            "dist",
        ]) {
            Ok(Tool::PackRelease {
                sums,
                target,
                binary,
                out,
            }) => assert_eq!(
                packing(sums, target, binary, out),
                Some(Job::Pack {
                    target: Target::X8664UnknownLinuxMusl,
                    binary: PathBuf::from("target/htmlspec"),
                    out: PathBuf::from("dist"),
                })
            ),
            _ => panic!("a pack did not parse"),
        }
        match parsed(&["pack-release", "--sums", "dist"]) {
            Ok(Tool::PackRelease {
                sums,
                target,
                binary,
                out,
            }) => assert_eq!(
                packing(sums, target, binary, out),
                Some(Job::Sums {
                    out: PathBuf::from("dist")
                })
            ),
            _ => panic!("--sums did not parse"),
        }
        for arguments in [
            &[
                "pack-release",
                "riscv64gc-unknown-linux-gnu",
                "target/htmlspec",
                "dist",
            ][..],
            &[
                "pack-release",
                "x86_64-unknown-linux-musl",
                "target/htmlspec",
            ],
            &[
                "pack-release",
                "--sums",
                "dist",
                "x86_64-unknown-linux-musl",
            ],
            &["pack-release"],
        ] {
            let refused = parsed(arguments).err();
            assert_eq!(
                refused.map(|error| error.exit_code()),
                Some(2),
                "{arguments:?} was not a usage error"
            );
        }
        assert_eq!(packing(None, None, Some(PathBuf::from("b")), None), None);
    }

    #[test]
    fn each_release_tool_is_asked_for_by_its_name() {
        assert!(matches!(
            parsed(&["crate-notices", "--check"]),
            Ok(Tool::CrateNotices { check: true })
        ));
        assert!(matches!(
            parsed(&["crate-notices"]),
            Ok(Tool::CrateNotices { check: false })
        ));
        match parsed(&[
            "render-formula",
            "--sums",
            "dist/SHA256SUMS",
            "--base-url",
            "file:///d",
        ]) {
            Ok(Tool::RenderFormula { sums, base_url }) => {
                assert_eq!(sums, PathBuf::from("dist/SHA256SUMS"));
                assert_eq!(base_url.as_deref(), Some("file:///d"));
            }
            _ => panic!("render-formula did not parse"),
        }
        assert_eq!(
            parsed(&["render-formula"])
                .err()
                .map(|error| error.exit_code()),
            Some(2)
        );
        assert!(matches!(
            parsed(&["check-formula", "dist"]),
            Ok(Tool::CheckFormula { .. })
        ));
        assert_eq!(
            parsed(&["check-formula"])
                .err()
                .map(|error| error.exit_code()),
            Some(2)
        );
        match parsed(&["check-readme-install", "--print", "linux-release"]) {
            Ok(Tool::CheckReadmeInstall { print }) => {
                assert_eq!(print.as_deref(), Some("linux-release"))
            }
            _ => panic!("--print did not parse"),
        }
        assert!(matches!(
            parsed(&["check-readme-install"]),
            Ok(Tool::CheckReadmeInstall { print: None })
        ));
        assert!(matches!(
            parsed(&["facts", "version"]),
            Ok(Tool::Facts {
                fact: Fact::Version
            })
        ));
    }

    /// The facts' third case from before the port, whose reader exited 2 on an argument it did
    /// not know. clap exits 2 on a usage error too.
    #[test]
    fn an_argument_it_does_not_know_is_a_usage_error() {
        for arguments in [
            &["facts", "matrix", "--linux"][..],
            &["facts", "targets", "--without-macos", "extra"],
            &["facts", "runners"],
        ] {
            let refused = parsed(arguments).err();
            assert_eq!(
                refused.map(|error| error.exit_code()),
                Some(2),
                "{arguments:?} was not a usage error"
            );
        }
    }
}
