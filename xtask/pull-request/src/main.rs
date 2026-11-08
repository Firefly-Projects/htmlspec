//! The pull-request guard's binary, `check-pull-request`: the rules alone, which the guard builds
//! without the program's library. `cargo xtask check-pull-request` takes the same arguments.

use clap::Parser;
use htmlspec_pull_request::{run, Arguments};

/// Check a pull request's title, and that its diff records the change in CHANGELOG.md.
///
/// Exits 0 when the pull request passes, 1 naming each fault, and 2 when it cannot ask.
#[derive(Parser)]
#[command(name = "check-pull-request")]
struct Cli {
    #[command(flatten)]
    arguments: Arguments,
}

fn main() -> std::process::ExitCode {
    run(Cli::parse().arguments.job())
}
