//! How a check ends: one of three exit codes, and the lines it prints.
//!
//! The checks the suite and the release run share one contract for their exit codes: 0 when the
//! subject is clean, 1 when the check found faults, and 2 when it could not ask, because what it
//! reads is missing or unreadable. A stage and a workflow step read only the code, so the code is
//! a value a test reads, beside every line the check prints, rather than a number scattered across
//! `process::exit` calls.

use std::process::ExitCode;

/// What a check's exit code says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    /// Exit 0: nothing wrong.
    Clean,
    /// Exit 1: the check asked, and found faults.
    Faults,
    /// Exit 2: the check could not ask, so it says nothing of the subject.
    CouldNotAsk,
}

impl Exit {
    /// The process's exit code.
    pub fn code(self) -> u8 {
        match self {
            Self::Clean => 0,
            Self::Faults => 1,
            Self::CouldNotAsk => 2,
        }
    }
}

/// How a check ends: its exit, and the lines it prints to standard output and to standard error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub exit: Exit,
    pub out: Vec<String>,
    pub err: Vec<String>,
}

impl Report {
    /// A check that could not ask, saying why on standard error.
    pub fn could_not_ask(reason: &dyn std::fmt::Display) -> Self {
        Self {
            exit: Exit::CouldNotAsk,
            out: Vec::new(),
            err: vec![reason.to_string()],
        }
    }

    /// Prints the lines, and gives the exit code.
    pub fn print(self) -> ExitCode {
        for line in &self.out {
            println!("{line}");
        }
        for line in &self.err {
            eprintln!("{line}");
        }
        ExitCode::from(self.exit.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three codes a stage and a workflow step read.
    #[test]
    fn each_exit_has_its_code() {
        assert_eq!(Exit::Clean.code(), 0);
        assert_eq!(Exit::Faults.code(), 1);
        assert_eq!(Exit::CouldNotAsk.code(), 2);
    }
}
