//! The `htmlspec` command-line program.
//!
//! Everything it does is in the library beside it, `src/lib.rs`, whose `run` parses the process's
//! arguments, runs the operation they name, and returns the exit code. This is the process's one
//! exit point.

fn main() {
    std::process::exit(htmlspec_cli::run());
}
