//! The formula check's cases, on a fake Homebrew held in memory.
//!
//! The fake answers as Homebrew 6 and 7 do where the check depends on it. It loads a formula from a
//! tap only once the formula is trusted, except that `brew install` naming the formula in full
//! trusts it first, and it keeps the trusted names in a store that is absent when nothing is left
//! in it. `brew uninstall` needs no trust, and once it succeeds it removes the formula's trust
//! itself. `brew untap` loads a formula of the tap that is still installed, so it fails while that
//! formula is untrusted. Or it answers as a Homebrew from before tap trust, which has no `trust` or
//! `untrust` command and loads any formula.
//!
//! No test builds the real [`Homebrew`] to run it: the one test that names it reads the process it
//! would start, and starts none. So nothing here can reach a machine's Homebrew, whatever `PATH`
//! holds.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::*;

/// Spelled out rather than read from the module: these are the names the check must use, and a
/// test that read them from the code under test would pass whatever that code said.
const SPELLED_TAP: &str = "htmlspec-check/local";
const SPELLED_FORMULA: &str = "htmlspec-check/local/htmlspec";

/// Where the fake keeps its taps, and its prefix.
const TAPS: &str = "/fake-homebrew/Library/Taps";
const PREFIX: &str = "/fake-homebrew";

/// The fake's trust store: the formulae and the taps it trusts. Absent, as Homebrew's `trust.json`
/// is, when both are empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Store {
    formulae: BTreeSet<String>,
    taps: BTreeSet<String>,
}

/// How the fake answers.
#[derive(Clone, Copy, Debug, Default)]
struct Answers {
    trust: bool,
    audit_fails: bool,
    uninstall_fails: bool,
    untrust_fails: bool,
    /// No `brew` to start, as on a machine without Homebrew.
    absent: bool,
}

/// A Homebrew held in memory.
struct FakeHomebrew {
    answers: Answers,
    /// Each tap, by name, with the formula written into it, where one was.
    taps: BTreeMap<String, Option<String>>,
    store: Store,
    /// The formula installed, by its full name.
    installed: Option<String>,
    /// Each command given, with the formulae trusted when it was given.
    calls: Vec<(BrewCommand, BTreeSet<String>)>,
    /// Every formula text written, with where.
    written: Vec<(PathBuf, String)>,
}

impl FakeHomebrew {
    fn new(answers: Answers) -> Self {
        Self {
            answers,
            taps: BTreeMap::new(),
            store: Store::default(),
            installed: None,
            calls: Vec::new(),
            written: Vec::new(),
        }
    }

    fn trusting(mut self, store: Store) -> Self {
        self.store = store;
        self
    }

    fn tap_path(tap: &str) -> PathBuf {
        let (user, repository) = tap.split_once('/').unwrap_or((tap, ""));
        Path::new(TAPS)
            .join(user)
            .join(format!("homebrew-{repository}"))
    }

    /// Loads a formula by its full name, as Homebrew does before it audits, installs, or tests it.
    fn load(&self, name: &str) -> bool {
        let tap = name.rsplit_once('/').map_or("", |(tap, _)| tap);
        let written = self.taps.get(tap).is_some_and(Option::is_some);
        written && (!self.answers.trust || self.store.formulae.contains(name))
    }

    fn calls(&self) -> Vec<BrewCommand> {
        self.calls.iter().map(|(command, _)| *command).collect()
    }

    /// The store, as Homebrew keeps it: absent when nothing is in it.
    fn store(&self) -> Option<Store> {
        (self.store != Store::default()).then(|| self.store.clone())
    }
}

fn answered(succeeded: bool, stdout: &str) -> Ran {
    if succeeded {
        Ran::Succeeded {
            stdout: stdout.to_owned(),
        }
    } else {
        Ran::Failed
    }
}

impl Brew for FakeHomebrew {
    fn run(&mut self, command: BrewCommand) -> Ran {
        self.calls.push((command, self.store.formulae.clone()));
        if self.answers.absent {
            return Ran::NotStarted(ErrorKind::NotFound);
        }
        let trust = self.answers.trust;
        match command {
            BrewCommand::Taps => answered(
                true,
                &self
                    .taps
                    .keys()
                    .map(|tap| format!("{tap}\n"))
                    .collect::<String>(),
            ),
            BrewCommand::Installed => answered(self.installed.is_some(), "htmlspec\n"),
            BrewCommand::Developer => answered(true, "Developer mode is disabled.\n"),
            BrewCommand::DeveloperOff => answered(true, ""),
            BrewCommand::TrustHelp => {
                answered(trust, "Usage: brew trust [--formula] [target ...]\n")
            }
            BrewCommand::TapNew => {
                let fresh = !self.taps.contains_key(SPELLED_TAP);
                self.taps.insert(SPELLED_TAP.to_owned(), None);
                answered(fresh, "")
            }
            BrewCommand::Repository => answered(
                true,
                &format!("{}\n", Self::tap_path(SPELLED_TAP).display()),
            ),
            BrewCommand::Trust => {
                if trust {
                    self.store.formulae.insert(SPELLED_FORMULA.to_owned());
                }
                answered(trust, "")
            }
            BrewCommand::Untrust => {
                if !trust || self.answers.untrust_fails {
                    return Ran::Failed;
                }
                self.store.formulae.remove(SPELLED_FORMULA);
                answered(true, "")
            }
            BrewCommand::Audit => {
                answered(self.load(SPELLED_FORMULA) && !self.answers.audit_fails, "")
            }
            BrewCommand::Install => {
                if trust {
                    self.store.formulae.insert(SPELLED_FORMULA.to_owned());
                }
                if !self.load(SPELLED_FORMULA) {
                    return Ran::Failed;
                }
                self.installed = Some(SPELLED_FORMULA.to_owned());
                answered(true, "")
            }
            BrewCommand::Test => answered(self.load(SPELLED_FORMULA), ""),
            BrewCommand::Prefix => answered(true, &format!("{PREFIX}\n")),
            BrewCommand::Uninstall => {
                let Some(installed) = self.installed.clone() else {
                    return answered(true, "");
                };
                if self.answers.uninstall_fails {
                    return Ran::Failed;
                }
                self.installed = None;
                let tap = installed.rsplit_once('/').map_or("", |(tap, _)| tap);
                if !self.store.taps.contains(tap) {
                    self.store.formulae.remove(&installed);
                }
                answered(true, "")
            }
            BrewCommand::Untap => {
                if let Some(installed) = self.installed.clone() {
                    if !self.load(&installed) {
                        return Ran::Failed;
                    }
                }
                answered(self.taps.remove(SPELLED_TAP).is_some(), "")
            }
        }
    }

    fn write_formula(&mut self, repository: &Path, formula: &str) -> Result<(), ErrorKind> {
        self.written
            .push((repository.to_path_buf(), formula.to_owned()));
        let tap = self
            .taps
            .keys()
            .find(|tap| Self::tap_path(tap) == repository)
            .cloned()
            .ok_or(ErrorKind::NotFound)?;
        self.taps.insert(tap, Some(formula.to_owned()));
        Ok(())
    }

    fn installed_version(&mut self, prefix: &Path) -> Ran {
        answered(
            self.installed.is_some() && prefix == Path::new(PREFIX),
            "htmlspec 9.8.7 (0123456789ab)\n",
        )
    }
}

/// What a run printed, and how it ended.
struct Outcome {
    exit: Exit,
    out: Vec<String>,
    err: Vec<String>,
}

const FORMULA_TEXT: &str = "class Htmlspec < Formula\nend\n";

fn run_check(brew: &mut FakeHomebrew) -> Outcome {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let exit = check(brew, FORMULA_TEXT, &mut |line| match line {
        Line::Out(text) => out.push(text),
        Line::Err(text) => err.push(text),
    });
    Outcome { exit, out, err }
}

fn position(calls: &[BrewCommand], command: BrewCommand) -> usize {
    calls
        .iter()
        .position(|given| *given == command)
        .unwrap_or_else(|| panic!("{command} was never given: {calls:?}"))
}

/// The formula is trusted after it is written and before the audit, by its full name and nothing
/// wider, and untrusted only after the untap, which loads it if it is still installed.
fn assert_trust_given_and_taken_back(calls: &[BrewCommand]) {
    let trusts: Vec<String> = calls
        .iter()
        .filter(|command| command.arguments().first() == Some(&"trust"))
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        trusts,
        [
            String::from("brew trust --help"),
            format!("brew trust --formula {SPELLED_FORMULA}")
        ]
    );
    let untrusts: Vec<String> = calls
        .iter()
        .filter(|command| command.arguments().first() == Some(&"untrust"))
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        untrusts,
        [format!("brew untrust --formula {SPELLED_FORMULA}")]
    );
    assert!(position(calls, BrewCommand::TapNew) < position(calls, BrewCommand::Trust));
    assert!(position(calls, BrewCommand::Trust) < position(calls, BrewCommand::Audit));
    assert!(position(calls, BrewCommand::Untap) < position(calls, BrewCommand::Untrust));
}

#[test]
fn with_tap_trust_the_formula_is_trusted_before_the_audit_and_untrusted_in_cleanup() {
    let before = Store {
        formulae: BTreeSet::from([String::from("someone/tools/other")]),
        taps: BTreeSet::from([String::from("someone/tools")]),
    };
    let mut brew = FakeHomebrew::new(Answers {
        trust: true,
        ..Answers::default()
    })
    .trusting(before.clone());
    let ran = run_check(&mut brew);
    assert_eq!(ran.exit, Exit::Clean, "{:?}", ran.err);
    assert!(ran.out.iter().any(|line| line == PASSED));
    assert_trust_given_and_taken_back(&brew.calls());
    assert_eq!(brew.store(), Some(before));
    assert!(brew.taps.is_empty(), "the check's tap is left behind");
    assert!(brew.installed.is_none(), "the install is left behind");
    assert!(!ran.err.iter().any(|line| line.contains("warning")));
    assert_eq!(
        brew.written,
        [(
            PathBuf::from("/fake-homebrew/Library/Taps/htmlspec-check/homebrew-local"),
            String::from(FORMULA_TEXT)
        )]
    );
}

#[test]
fn a_failed_uninstall_is_untapped_while_still_trusted_and_untrusted_last() {
    let mut brew = FakeHomebrew::new(Answers {
        trust: true,
        uninstall_fails: true,
        ..Answers::default()
    });
    let ran = run_check(&mut brew);
    assert_eq!(ran.exit, Exit::Clean, "{:?}", ran.err);
    assert!(ran.out.iter().any(|line| line == PASSED));
    assert_trust_given_and_taken_back(&brew.calls());
    let untap = brew
        .calls
        .iter()
        .find(|(command, _)| *command == BrewCommand::Untap)
        .map(|(_, trusted)| trusted.clone())
        .unwrap_or_default();
    assert!(
        untap.contains(SPELLED_FORMULA),
        "the untap loads the formula still installed"
    );
    assert!(
        brew.taps.is_empty(),
        "the untap was refused, so the check's tap is left behind"
    );
    assert_eq!(
        brew.store(),
        None,
        "the formula is still trusted after the check"
    );
}

#[test]
fn with_tap_trust_a_failed_audit_still_takes_the_trust_back() {
    let mut brew = FakeHomebrew::new(Answers {
        trust: true,
        audit_fails: true,
        ..Answers::default()
    });
    let ran = run_check(&mut brew);
    assert_eq!(ran.exit, Exit::Faults);
    assert!(ran
        .err
        .iter()
        .any(|line| line
            == "check-formula: brew audit --strict htmlspec-check/local/htmlspec failed"));
    assert!(!ran.out.iter().any(|line| line == PASSED));
    let calls = brew.calls();
    assert_trust_given_and_taken_back(&calls);
    assert!(!calls.contains(&BrewCommand::Install));
    assert_eq!(
        brew.store(),
        None,
        "a store the check found absent is left absent"
    );
}

#[test]
fn an_untrust_that_fails_is_named_with_the_command_that_finishes_it() {
    let mut brew = FakeHomebrew::new(Answers {
        trust: true,
        audit_fails: true,
        untrust_fails: true,
        ..Answers::default()
    });
    let ran = run_check(&mut brew);
    assert_eq!(ran.exit, Exit::Faults);
    assert_eq!(
        brew.store(),
        Some(Store {
            formulae: BTreeSet::from([String::from(SPELLED_FORMULA)]),
            taps: BTreeSet::new(),
        })
    );
    assert!(ran
        .err
        .iter()
        .any(|line| line.contains("brew untrust --formula htmlspec-check/local/htmlspec")));
    let calls = brew.calls();
    assert!(
        position(&calls, BrewCommand::Untrust) < position(&calls, BrewCommand::DeveloperOff),
        "a failed untrust stops the rest of the cleanup"
    );
}

#[test]
fn without_tap_trust_the_check_passes_and_asks_only_whether_trust_exists() {
    let mut brew = FakeHomebrew::new(Answers::default());
    let ran = run_check(&mut brew);
    assert_eq!(ran.exit, Exit::Clean, "{:?}", ran.err);
    assert!(ran.out.iter().any(|line| line == PASSED));
    let calls = brew.calls();
    for command in &calls {
        let first = command.arguments().first().copied();
        if first == Some("trust") || first == Some("untrust") {
            assert_eq!(
                *command,
                BrewCommand::TrustHelp,
                "a Homebrew without trust has no command to call"
            );
        }
    }
    for step in [BrewCommand::Audit, BrewCommand::Install, BrewCommand::Test] {
        assert!(calls.contains(&step), "{step}");
    }
    assert_eq!(brew.store(), None);
}

/// Beyond the Python's cases: the check refuses to start where its tap exists or htmlspec is
/// installed through Homebrew, exit 2, and gives Homebrew no command that changes anything.
#[test]
fn a_tap_or_an_install_already_there_stops_the_check_before_it_changes_anything() {
    let mut tapped = FakeHomebrew::new(Answers::default());
    tapped.taps.insert(SPELLED_TAP.to_owned(), None);
    let ran = run_check(&mut tapped);
    assert_eq!(ran.exit, Exit::CouldNotAsk);
    assert_eq!(tapped.calls(), [BrewCommand::Taps]);
    assert_eq!(
        ran.err,
        ["the tap htmlspec-check/local already exists; remove it with brew untap htmlspec-check/local first"]
    );
    let mut installed = FakeHomebrew::new(Answers::default());
    installed.installed = Some(String::from("someone/tools/htmlspec"));
    let ran = run_check(&mut installed);
    assert_eq!(ran.exit, Exit::CouldNotAsk);
    assert_eq!(
        installed.calls(),
        [BrewCommand::Taps, BrewCommand::Installed]
    );
}

/// A Homebrew that cannot be started stops the check at its first question, exit 2, saying so,
/// before it makes anything to clean up.
#[test]
fn a_homebrew_that_cannot_be_started_stops_the_check_before_it_makes_anything() {
    let mut absent = FakeHomebrew::new(Answers {
        absent: true,
        ..Answers::default()
    });
    let ran = run_check(&mut absent);
    assert_eq!(ran.exit, Exit::CouldNotAsk);
    assert_eq!(absent.calls(), [BrewCommand::Taps]);
    assert_eq!(
        ran.err,
        ["check-formula: brew tap could not be started: entity not found"]
    );
}

/// Each command names the check's own tap and formula, spelled out here, and the steps the check
/// exists for are the ones whose words the reader sees.
#[test]
fn each_command_names_the_check_s_own_tap_and_formula() {
    let said: Vec<String> = [
        BrewCommand::TapNew,
        BrewCommand::Trust,
        BrewCommand::Audit,
        BrewCommand::Install,
        BrewCommand::Test,
        BrewCommand::Uninstall,
        BrewCommand::Untap,
        BrewCommand::Untrust,
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    assert_eq!(
        said,
        [
            "brew tap-new --no-git htmlspec-check/local",
            "brew trust --formula htmlspec-check/local/htmlspec",
            "brew audit --strict htmlspec-check/local/htmlspec",
            "brew install htmlspec-check/local/htmlspec",
            "brew test htmlspec-check/local/htmlspec",
            "brew uninstall --formula htmlspec-check/local/htmlspec",
            "brew untap htmlspec-check/local",
            "brew untrust --formula htmlspec-check/local/htmlspec",
        ]
    );
    for shown in [
        BrewCommand::TapNew,
        BrewCommand::Trust,
        BrewCommand::Audit,
        BrewCommand::Install,
        BrewCommand::Test,
    ] {
        assert!(!shown.is_query(), "{shown}");
    }
    assert!(BrewCommand::Uninstall.is_query());
}

/// The real Homebrew is told not to update itself, send analytics, clean up, hint, or autoremove,
/// so the cleanup's uninstall removes htmlspec and none of the machine's other leftovers. The
/// process is read here and never started.
#[test]
fn the_real_homebrew_is_kept_from_updating_reporting_and_autoremoving() {
    let process = homebrew_command(BrewCommand::Uninstall);
    assert_eq!(process.get_program(), "brew");
    let arguments: Vec<&std::ffi::OsStr> = process.get_args().collect();
    assert_eq!(arguments, ["uninstall", "--formula", SPELLED_FORMULA]);
    let environment: BTreeMap<String, String> = process
        .get_envs()
        .filter_map(|(name, value)| Some((name.to_str()?.to_owned(), value?.to_str()?.to_owned())))
        .collect();
    for name in [
        "HOMEBREW_NO_AUTO_UPDATE",
        "HOMEBREW_NO_ANALYTICS",
        "HOMEBREW_NO_INSTALL_CLEANUP",
        "HOMEBREW_NO_ENV_HINTS",
        "HOMEBREW_NO_AUTOREMOVE",
    ] {
        assert_eq!(
            environment.get(name).map(String::as_str),
            Some("1"),
            "{name}"
        );
    }
    assert_eq!(environment.len(), 5);
}

/// A DIST with no `SHA256SUMS` is a usage error, exit 2, before anything is rendered.
#[test]
fn a_dist_without_sums_is_refused_before_homebrew_is_asked_anything() {
    let scratch = crate::scratch::Scratch::new("check-formula");
    let stopped = local_formula(scratch.path()).err();
    assert_eq!(stopped.as_ref().map(Stopped::exit), Some(Exit::CouldNotAsk));
}
