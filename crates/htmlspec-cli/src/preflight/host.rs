//! The one part of preflight that can touch a machine.
//!
//! Everything in the parent module is pure over its inputs. This module holds the four fixed probe
//! operations, the bounded process boundary they run through, and the one package-manager adapter —
//! and it is the whole of what a carried runbook can make this program do to a machine.
//!
//! # Bounds
//!
//! Every probe runs with closed stdin, a ten-second deadline, and a 64 KiB combined output limit,
//! and exceeding either limit is unavailable readiness rather than a partial answer. Both numbers
//! are the design's, and both exist for the same reason: a version process that hangs or that
//! prints a megabyte is a tool this program cannot read, and "cannot read" has to be an outcome
//! rather than a stuck command.
//!
//! # What this is not
//!
//! Not a process sandbox. `runbook-portability` withholds arbitrary commands from a declaration
//! precisely so that the only processes here are the ones this module names: a tool asked its
//! version, `git` asked for a status, and the package manager asked to evaluate a manifest. The
//! tools themselves are trusted, because the alternative would be sandboxing a compiler.
//!
//! # Injection
//!
//! The process boundary is behind [`CommandRunner`] and the host is generic over it, so ordinary
//! tests script responses and never depend on what the developer has installed. The filesystem
//! probes are not injected and do not need to be: they read a directory the test creates, which is
//! as deterministic as a scripted response and exercises the real containment check.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use command_group::CommandGroup as _;

use super::{
    Fact, HostProbe, MachineCheck, Observation, Observations, Observed, ProbeFault, ProgramName,
    ScopedRelativePath, ShellName, SkillScope, ToolProvenance, WorkingState,
};

use crate::project::ProjectRoot;
use crate::terminal::shown;
use crate::version::ToolVersion;

/// How long one probe may run before it is stopped.
pub const DEADLINE: Duration = Duration::from_secs(10);

/// How much output one probe may produce before the answer is unavailable.
pub const OUTPUT_LIMIT: usize = 64 * 1024;

/// The one argument a tool probe adds.
///
/// Fixed rather than declared, and that is the whole of the tool form's narrowness: a runbook says
/// *which* program and *what version*, never how to ask. A tool with another version interface
/// needs an adapter, because letting a carried document supply the argument would let it supply
/// anything.
pub const VERSION_ARGUMENT: &str = "--version";

/// The program the toolchain adapter evaluates a manifest with.
const NIX: &str = "nix";

/// The program the adapter queries realized closures with.
///
/// A separate program rather than a second `nix` subcommand because `nix-store --query` needs no
/// experimental feature, so the closure half of the adapter works wherever the evaluator half does.
const NIX_STORE: &str = "nix-store";

/// The program the clean-worktree probe reads.
const GIT: &str = "git";

/// How many worktree entries a report names before it counts the rest.
const DIRTY_ENTRIES_NAMED: usize = 20;

/// A cancellation signal, shared with every probe this preflight runs.
///
/// Checked between probes and inside the poll loop of a running one, so an interruption stops the
/// work and reaps it rather than letting it finish in the background. The flag is a `bool` behind
/// an `Arc` rather than a thread handle because nothing here owns a thread that could be joined:
/// the processes are the work, and a killed-and-reaped process is finished.
#[derive(Clone, Debug, Default)]
pub struct Cancellation {
    flag: Arc<AtomicBool>,
}

impl Cancellation {
    /// A signal nothing has raised.
    pub fn new() -> Self {
        Self::default()
    }

    /// A signal raised by the operator interrupting the process.
    ///
    /// **Registered rather than left to the default disposition, and the process group is why.** A
    /// probe runs in its own group so that stopping it can stop everything it started — and the same
    /// property means a terminal's interrupt no longer reaches it. Without this, `Ctrl-C` during a
    /// Nix evaluation would end this process and leave the evaluator running, which is the opposite of
    /// what `runbook-portability` requires: "An interrupted preflight SHALL stop its probes and
    /// publish no record."
    ///
    /// `signal-hook` rather than a hand-rolled handler because installing one is an `unsafe` FFI call
    /// and this workspace forbids `unsafe_code`. The crate is already in the dependency tree through
    /// `inquire`, so this adds no vendored subtree.
    pub fn on_interrupt() -> std::io::Result<Self> {
        let cancellation = Self::new();
        for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
            signal_hook::flag::register(signal, cancellation.flag.clone())?;
        }
        Ok(cancellation)
    }

    /// Raises it.
    ///
    /// The handler installed by [`Cancellation::on_interrupt`] raises the same flag, so this is
    /// the programmatic half of one mechanism rather than a second one.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "a caller that has its own reason to stop a preflight raises this rather \
                than signalling the process; the interrupt path reaches the same flag through \
                the handler"
        )
    )]
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// Whether it has been raised.
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

/// One bounded process invocation, as a probe requests it.
///
/// A program path and its arguments, never a command line: nothing here is handed to a shell, so
/// an argument carrying a metacharacter is an argument rather than a second command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeRequest {
    /// The executable, already resolved.
    pub program: PathBuf,
    /// The fixed arguments.
    pub arguments: Vec<String>,
    /// Where it runs, which is the invocation's working directory.
    pub workdir: PathBuf,
    /// What the child's environment changes from the one it inherits, and nothing more.
    pub environment: &'static [EnvironmentChange],
}

/// One change a probe's child makes to the environment it inherits.
///
/// **Static, and that is the narrowness.** Every name and value is a `&'static str` this module
/// spells, so a carried runbook can no more set a variable on a probe than choose its arguments,
/// and the change reaches the child alone: this process's environment is never touched, which is
/// what keeps one probe's settings out of every other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvironmentChange {
    /// The variable `name` is `value` for the child.
    Set {
        /// The variable.
        name: &'static str,
        /// Its value.
        value: &'static str,
    },
    /// Every variable whose name ends in `suffix`, in either case, is absent from the child's
    /// environment, whether it was inherited or set by an earlier change. For a family of
    /// variables whose members this module cannot list, such as the proxy a client reads for each
    /// scheme it knows.
    RemoveEveryEndingIn {
        /// The ending, compared without regard to case.
        suffix: &'static str,
    },
}

/// The environment a probe's child runs with: `inherited`, changed as `changes` say, in order.
///
/// Pure, so what reaches the child is a fact a test can assert without changing this process's
/// environment, which another test running beside it would see.
fn child_environment(
    inherited: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
    changes: &[EnvironmentChange],
) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    let mut environment: Vec<_> = inherited.into_iter().collect();
    for change in changes {
        match *change {
            EnvironmentChange::Set { name, value } => {
                environment.retain(|(present, _)| present != name);
                environment.push((name.into(), value.into()));
            }
            EnvironmentChange::RemoveEveryEndingIn { suffix } => {
                let suffix = suffix.to_ascii_lowercase();
                environment.retain(|(present, _)| {
                    !present
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .ends_with(&suffix)
                });
            }
        }
    }
    environment
}

/// What a bounded invocation produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunResult {
    /// It exited, with what it wrote to each stream.
    ///
    /// **The streams are kept apart**, because a probe parses what a program answers and only
    /// reports what it complains about. Joined, a harmless warning on stderr — an unknown
    /// `nix.conf` setting, a `git` advisory — became part of the answer: the evaluator's JSON no
    /// longer parsed, and a clean worktree gained an entry.
    Exited {
        /// The exit status.
        status: i32,
        /// What it wrote to stdout, within the limit: the answer a probe parses.
        stdout: String,
        /// What it wrote to stderr, within the limit: what a report quotes when the answer is wanting.
        stderr: String,
        /// Whether the output exceeded the limit and was cut.
        truncated: bool,
    },
    /// It did not finish inside the deadline and was stopped.
    TimedOut,
    /// It produced more than the limit and was stopped.
    OutputExceeded {
        /// The limit it exceeded.
        limit: usize,
    },
    /// Cancellation reached it, and it was stopped and reaped.
    Cancelled,
    /// It could not be started at all.
    NotRun {
        /// Why not.
        reason: String,
    },
}

/// Runs one bounded probe process.
///
/// Behind a trait so the tests script responses. **The trait is the boundary that keeps the suite
/// honest**: a test that ran `cargo --version` would pass on a contributor's machine and fail on a
/// clean one, and it would be testing the contributor's installation rather than the assessment.
pub trait CommandRunner {
    /// Runs `request`, stopping it at the deadline, the output limit, or cancellation.
    fn run(&self, request: &ProbeRequest, cancellation: &Cancellation) -> RunResult;
}

/// The real runner: a child in its own process group, killed as a group.
///
/// **A group rather than a process, and the reason is measured rather than imagined.** A toolchain
/// probe runs `nix eval`, which can fork. Killing only the child this program started would leave
/// the fork behind, still running after the command had reported — so a "stopped" probe would not
/// have stopped. `std` can put a child in its own group but offers no way to signal one, and the
/// workspace forbids `unsafe_code`, which is why `command-group` is a dependency at all.
///
/// The group is killed *before* the child is reaped in every path. After a reap the pid is free for
/// reuse, and signalling a group whose id has been recycled would kill processes that are not ours;
/// while the child is a zombie its pid is still reserved, so the kill can only reach the group this
/// probe created.
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run(&self, request: &ProbeRequest, cancellation: &Cancellation) -> RunResult {
        self.run_on(request, cancellation, &Monotonic)
    }
}

/// The clock a running probe's deadline is read from.
///
/// **Passed in rather than read**, so a test can stop a real child at the deadline, or cancel one
/// while it runs, without waiting the deadline out or sleeping until the child has started. The
/// runner asks it only once it has found the child still running, so a test's clock that answers
/// "past the deadline", or raises the cancellation, does so while a real process is there to stop.
trait DeadlineClock {
    /// How long the probe has run since it was started at `started`.
    fn elapsed_since(&self, started: Instant) -> Duration;
}

/// The real clock: the monotonic time since the child started.
struct Monotonic;

impl DeadlineClock for Monotonic {
    fn elapsed_since(&self, started: Instant) -> Duration {
        started.elapsed()
    }
}

impl SystemRunner {
    /// Runs one bounded probe, its deadline read from `clock`.
    fn run_on(
        &self,
        request: &ProbeRequest,
        cancellation: &Cancellation,
        clock: &dyn DeadlineClock,
    ) -> RunResult {
        let mut command = Command::new(&request.program);
        command
            .args(&request.arguments)
            .current_dir(&request.workdir)
            // Closed rather than inherited: a probe that read the operator's terminal would compete
            // with the command for input, and one that blocked waiting for it would only be stopped
            // by the deadline.
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // The whole environment, stated: what this process inherited, as the request changes it.
        command
            .env_clear()
            .envs(child_environment(std::env::vars_os(), request.environment));

        let mut child = match command.group_spawn() {
            Ok(child) => child,
            Err(error) => {
                return RunResult::NotRun {
                    reason: format!(
                        "{} could not be started: {error}",
                        shown(&request.program.display().to_string())
                    ),
                }
            }
        };

        // Read off the pipes in their own threads. Reading in the poll loop instead would block it
        // on the first read, which is how a probe that prints nothing and a probe that prints
        // endlessly become the same wait. Each stream is capped, and a reader that reaches its cap
        // keeps draining and counting rather than stopping — a stopped reader fills the pipe and
        // blocks the child, which would report a slow tool as a hung one.
        let readers = [
            child.inner().stdout.take().map(spawn_reader),
            child.inner().stderr.take().map(spawn_reader),
        ];

        let started = Instant::now();
        let status = loop {
            if cancellation.is_cancelled() {
                let _ = child.kill();
                let _ = child.wait();
                drain(readers);
                return RunResult::Cancelled;
            }
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if clock.elapsed_since(started) > DEADLINE {
                        let _ = child.kill();
                        let _ = child.wait();
                        drain(readers);
                        return RunResult::TimedOut;
                    }
                    // Polled rather than blocked on `wait`, which is the only way a deadline can be
                    // honoured without a second thread owning the child.
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    drain(readers);
                    return RunResult::NotRun {
                        reason: format!("its status could not be read: {error}"),
                    };
                }
            }
        };

        // The group is killed before the reap even on the ordinary path, so a descendant that
        // inherited a pipe cannot hold a reader thread open past the probe that started it.
        let _ = child.kill();
        let _ = child.wait();
        let ([stdout, stderr], overflow) = drain(readers);

        if overflow > 0 {
            return RunResult::OutputExceeded {
                limit: OUTPUT_LIMIT,
            };
        }
        RunResult::Exited {
            status: status.code().unwrap_or(-1),
            stdout,
            stderr,
            truncated: false,
        }
    }
}

/// Reads one stream to its end, keeping at most [`OUTPUT_LIMIT`] bytes and counting the rest.
///
/// Generic over the two pipe types because a child's stdout and stderr are distinct types in `std`
/// and the bound that matters — that they can be read — is the same for both.
fn spawn_reader<R: Read + Send + 'static>(
    mut stream: R,
) -> std::thread::JoinHandle<(Vec<u8>, usize)> {
    std::thread::spawn(move || {
        let mut taken = Vec::new();
        let mut overflow = 0usize;
        let mut buffered = [0u8; 8192];
        loop {
            match stream.read(&mut buffered) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    let room = OUTPUT_LIMIT.saturating_sub(taken.len());
                    let kept = read.min(room);
                    taken.extend_from_slice(&buffered[..kept]);
                    overflow += read - kept;
                }
            }
        }
        (taken, overflow)
    })
}

/// A reader thread's handle, and what it produces: the bytes it kept and the count it dropped.
type Reader = std::thread::JoinHandle<(Vec<u8>, usize)>;

/// Joins the reader threads, keeping what each stream produced apart.
fn drain(readers: [Option<Reader>; 2]) -> ([String; 2], usize) {
    let mut overflow = 0usize;
    let streams = readers.map(|reader| {
        // A reader that cannot be joined left its bytes behind. Reported as overflow rather than
        // dropped silently, because "no output" and "output this could not collect" are different
        // facts.
        match reader.map(std::thread::JoinHandle::join) {
            Some(Ok((taken, exceeded))) => {
                overflow += exceeded;
                String::from_utf8_lossy(&taken).into_owned()
            }
            Some(Err(_)) => {
                overflow += 1;
                String::new()
            }
            None => String::new(),
        }
    });
    // The limit is on both streams together, as the design states it: each reader caps its own
    // stream, so two streams each within the limit can still exceed it between them.
    if streams
        .iter()
        .map(|stream| stream.chars().count())
        .sum::<usize>()
        > OUTPUT_LIMIT
    {
        overflow += 1;
    }
    (streams, overflow)
}

/// What a program said on both streams, for a report quoting it.
///
/// Only ever quoted, never parsed: parsing reads [`RunResult::Exited`]'s `stdout` alone.
fn said(stdout: &str, stderr: &str) -> String {
    match (stdout.trim().is_empty(), stderr.trim().is_empty()) {
        (_, true) => stdout.to_owned(),
        (true, false) => stderr.to_owned(),
        (false, false) => format!("{stdout}\n{stderr}"),
    }
}

/// A runner whose responses are written down, for the tests.
///
/// Records every request it was asked for, so a test can assert what the adapter invoked and not
/// only what it concluded — which is how "never calls `nix develop`, `build`, `shell`, or a fetch
/// command" is checked rather than promised.
#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub struct ScriptedRunner {
    /// The responses, matched to requests in order, with the last one repeated.
    responses: Vec<RunResult>,
    /// Every request made, in order.
    pub requests: std::sync::Arc<std::sync::Mutex<Vec<ProbeRequest>>>,
}

#[cfg(test)]
impl ScriptedRunner {
    /// A runner answering each request with the matching response.
    pub fn new(responses: Vec<RunResult>) -> Self {
        Self {
            responses,
            requests: Arc::default(),
        }
    }

    /// A runner whose every request exits successfully with `output`.
    pub fn always(output: &str) -> Self {
        Self::new(vec![RunResult::Exited {
            status: 0,
            stdout: output.to_owned(),
            stderr: String::new(),
            truncated: false,
        }])
    }

    /// The requests made so far, as `program arguments…`.
    pub fn invoked(&self) -> Vec<String> {
        self.requests
            .lock()
            .map(|requests| {
                requests
                    .iter()
                    .map(|request| {
                        let mut words = vec![request
                            .program
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| request.program.display().to_string())];
                        words.extend(request.arguments.iter().cloned());
                        words.join(" ")
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
impl CommandRunner for ScriptedRunner {
    fn run(&self, request: &ProbeRequest, cancellation: &Cancellation) -> RunResult {
        if let Ok(mut requests) = self.requests.lock() {
            requests.push(request.clone());
        }
        if cancellation.is_cancelled() {
            return RunResult::Cancelled;
        }
        let index = self
            .requests
            .lock()
            .map(|requests| requests.len().saturating_sub(1))
            .unwrap_or_default();
        self.responses
            .get(index)
            .or_else(|| self.responses.last())
            .cloned()
            .unwrap_or(RunResult::NotRun {
                reason: String::from("the script has no response for this request"),
            })
    }
}

/// The host a preflight observes through.
///
/// Holds the two places a check can be resolved against — the invocation's working directory and
/// the operator's home — the search path executables resolve on, and nothing else. `--store`
/// selects where a runbook's documents are read from, and deliberately not any of these:
/// `runbook-portability` requires machine requirements to be evaluated "against the invocation's
/// working directory and environment, not the store's directory", so a stored runbook checked from
/// a separate checkout judges the checkout.
///
/// **The search path is a field rather than a read of the environment at each resolution**, which
/// is what makes the tests below deterministic. A test that set `PATH` for its own duration would
/// be racing every other test in the process, and a suite whose outcome depends on which thread ran
/// first is not a suite anybody can trust.
pub struct Host<R> {
    workdir: PathBuf,
    home: Option<PathBuf>,
    search_path: Vec<PathBuf>,
    runner: R,
    cancellation: Cancellation,
    /// The runbooks directory of the project a run check reads records from: the project the
    /// runbook was read from, which is the store's under `--store`. `None` for a host that was not
    /// given one, where a run check is unavailable rather than read from anywhere else.
    runbooks: Option<PathBuf>,
}

impl Host<SystemRunner> {
    /// A host observing the machine it is running on.
    ///
    /// The working directory is passed rather than read here, so the caller states which tree the
    /// checks judge and a test can name one it created. The search path and home are read from this
    /// process's environment once, here, rather than at each resolution.
    pub fn system(workdir: PathBuf) -> Self {
        Self {
            workdir,
            home: home_directory(),
            search_path: environment_search_path(),
            runner: SystemRunner,
            cancellation: Cancellation::new(),
            runbooks: None,
        }
    }

    /// A host that stops when the operator interrupts it.
    ///
    /// Falls back to [`Self::system`] where a handler cannot be installed, and says so rather than
    /// failing the operation: a preflight that cannot be interrupted still answers the question it was
    /// asked, and refusing to answer it would be a worse outcome than running to completion.
    pub fn interruptible<O: std::io::Write, E: std::io::Write>(
        workdir: PathBuf,
        streams: &mut crate::output::Streams<O, E>,
    ) -> Self {
        match Cancellation::on_interrupt() {
            Ok(cancellation) => Self {
                cancellation,
                ..Self::system(workdir)
            },
            Err(error) => {
                let _ = streams.progress(&format!(
                    "cannot observe an interrupt: {error}. These checks will run to completion \
                     rather than stopping at a signal"
                ));
                Self::system(workdir)
            }
        }
    }
}

impl<R: CommandRunner> Host<R> {
    /// A host over `runner`, observing `workdir`, `home`, and `search_path`.
    ///
    /// The seam the tests drive: every response a probe can get is injected through `runner`, so no
    /// test depends on what the machine running the suite has installed.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the injection seam; the command path builds \
        through `system` and `interruptible`, and a host no caller can give a runner to is a host no \
        test can drive"
        )
    )]
    pub fn new(
        workdir: PathBuf,
        home: Option<PathBuf>,
        search_path: Vec<PathBuf>,
        runner: R,
    ) -> Self {
        Self {
            workdir,
            home,
            search_path,
            runner,
            cancellation: Cancellation::new(),
            runbooks: None,
        }
    }

    /// This host, reading run checks' records from the project whose runbooks directory is
    /// `runbooks`.
    ///
    /// A separate place from the working directory on purpose. `--store` selects where a runbook
    /// and its records are read from, and a run check is a question about records, so it reads
    /// there; a machine check judges the working directory (the change
    /// `require-a-run-of-another-runbook`, its design's decisions 5 and 10).
    pub fn with_runbooks(mut self, runbooks: PathBuf) -> Self {
        self.runbooks = Some(runbooks);
        self
    }

    /// The handle that cancels this host's probes.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the handle a caller cancels through; the \
        command path raises the same flag from a signal, and a test raises it directly"
        )
    )]
    pub fn cancellation(&self) -> Cancellation {
        self.cancellation.clone()
    }

    /// Runs one bounded request and turns its result into the fault vocabulary.
    ///
    /// The one place a [`RunResult`] becomes a [`ProbeFault`], so the mapping between "the process
    /// did this" and "readiness is unavailable" is stated once rather than at each probe.
    fn run(&mut self, request: ProbeRequest) -> Result<RunResult, ProbeFault> {
        let result = self.runner.run(&request, &self.cancellation);
        match &result {
            RunResult::TimedOut => Err(ProbeFault::TimedOut { bound: DEADLINE }),
            RunResult::OutputExceeded { limit } => Err(ProbeFault::Unavailable {
                reason: format!(
                    "{} produced more than {limit} bytes of output, so the answer this probe reads \
                     is not in what it was allowed to keep",
                    shown(&request.program.display().to_string())
                ),
                remedy: String::from(
                    "the tool's version interface does not fit a bounded probe. It needs an \
                     adapter rather than a longer limit, because a limit long enough for one \
                     tool's output is long enough for another's to hang inside",
                ),
            }),
            RunResult::Cancelled => Err(ProbeFault::Failure {
                reason: String::from("the probe was cancelled and its processes were reaped"),
            }),
            RunResult::NotRun { reason } => Err(ProbeFault::Unavailable {
                reason: reason.clone(),
                remedy: String::from(
                    "make the program runnable from the working directory, or drop the \
                     declaration if the procedure no longer needs it",
                ),
            }),
            RunResult::Exited { .. } => Ok(result),
        }
    }

    /// Where a name resolves on the search path, if it resolves at all.
    ///
    /// Takes a bare string rather than a [`ProgramName`] because two of its callers pass this
    /// module's own constants — `nix`, `nix-store`, `git` — and not a runbook's declaration. Every
    /// *declared* name arrives as a `ProgramName`, which is where the rejection of a separator
    /// lives; a name this program spells cannot carry one.
    ///
    /// The first match wins, which is the resolution rule a shell uses and therefore the one that
    /// decides which executable a procedure will actually run. A probe that searched the whole path
    /// and reported every match would be answering a different question than the one that matters.
    fn which(&self, program: &str) -> Option<PathBuf> {
        self.search_path
            .iter()
            .filter(|directory| !directory.as_os_str().is_empty())
            .map(|directory| directory.join(program))
            .find(|candidate| is_executable(candidate))
    }

    /// The realized input closures of a set of store paths, in one query.
    ///
    /// One invocation rather than one per input, and only where the direct comparison already
    /// failed: a tool that resolves straight into a declared input needs no closure, and the common
    /// case stays at one subprocess.
    fn closures(&mut self, inputs: &[PathBuf]) -> Vec<PathBuf> {
        let Some(program) = self.which(NIX_STORE) else {
            return Vec::new();
        };
        if inputs.is_empty() {
            return Vec::new();
        }
        let mut arguments = vec![String::from("--query"), String::from("--requisites")];
        arguments.extend(inputs.iter().map(|input| input.display().to_string()));
        let request = ProbeRequest {
            program,
            arguments,
            workdir: self.workdir.clone(),
            environment: &[],
        };
        let Ok(RunResult::Exited {
            status: 0, stdout, ..
        }) = self.run(request)
        else {
            return Vec::new();
        };
        stdout
            .lines()
            .map(|line| PathBuf::from(line.trim()))
            .filter(|path| path.is_absolute())
            .collect()
    }
}

/// Whether a path is a file this process may execute.
///
/// The executable bit is checked on Unix and not elsewhere, because it is a Unix permission and a
/// platform with no such bit answers the question by asking whether the file is there.
fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    true
}

/// The operator's home directory, if this process can name one.
fn home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// The executable search path this process was started with.
///
/// Read once, at construction, and then a field: see [`Host`].
fn environment_search_path() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default()
}

impl<R: CommandRunner> HostProbe for Host<R> {
    fn cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Decides a run check against the records of the project this host was given.
    ///
    /// Reads documents and runs no process, so it spends none of a probe's budget.
    fn prerequisite(
        &mut self,
        runbook: &super::RunbookName,
        at_least: Option<crate::version::RunbookVersion>,
    ) -> Result<Fact, ProbeFault> {
        let Some(runbooks) = &self.runbooks else {
            return Err(ProbeFault::Unavailable {
                reason: String::from(
                    "this check was not given a project to read the runs of another runbook from",
                ),
                remedy: String::from(
                    "check it with `htmlspec preflight` or `htmlspec run`, which read the \
                     runbook's own project",
                ),
            });
        };
        crate::prerequisites::observe(runbooks, runbook, at_least)
    }

    /// Resolves the executable once and asks *that path* for its version.
    ///
    /// Resolving first and invoking the resolved path is what makes the answer about the tool the
    /// procedure will use. Invoking the bare name instead would let a shell function, an alias, or a
    /// second PATH entry answer for it — and PATH shadowing is a case `runbook-portability` names
    /// explicitly.
    fn tool(&mut self, program: &ProgramName) -> Result<Fact, ProbeFault> {
        let Some(resolved) = self.which(program.as_str()) else {
            return Ok(Fact::ToolAbsent {
                program: program.clone(),
            });
        };
        let request = ProbeRequest {
            program: resolved.clone(),
            arguments: vec![VERSION_ARGUMENT.to_owned()],
            workdir: self.workdir.clone(),
            environment: &[],
        };
        let result = self.run(request)?;
        let RunResult::Exited {
            status,
            stdout,
            stderr,
            ..
        } = result
        else {
            // `run` mapped every other result to a fault, so this arm is the `Ok` it returned.
            return Err(ProbeFault::Failure {
                reason: String::from("the probe produced no result"),
            });
        };
        if status != 0 {
            return Ok(Fact::ToolVersionUnreadable {
                program: program.clone(),
                resolved,
                problem: format!(
                    "it exited with status {status} when asked `{VERSION_ARGUMENT}`, and reported \
                     {}",
                    excerpt(&said(&stdout, &stderr))
                ),
            });
        }
        // Stdout is the answer where there is one. Some tools print their version to stderr and
        // nothing to stdout, and only then is stderr read: joining the two would let a warning that
        // happens to carry a version make the answer ambiguous.
        let answer = if stdout.trim().is_empty() {
            &stderr
        } else {
            &stdout
        };
        match version_token(answer) {
            Ok(version) => Ok(Fact::ToolVersion {
                program: program.clone(),
                resolved,
                version,
            }),
            Err(problem) => Ok(Fact::ToolVersionUnreadable {
                program: program.clone(),
                resolved,
                problem,
            }),
        }
    }

    /// Inspects a locked manifest read-only and compares each tool's provenance against it.
    ///
    /// Three refusals before anything is evaluated, and each is unavailable readiness rather than a
    /// failed check: an absent manifest, an unlocked one, and an absent package manager are all
    /// things this probe will not fix. **It never installs, builds, downloads, updates a lock, or
    /// enters a shell** — the flags below are what makes that a property of the invocation rather
    /// than a promise about it.
    fn toolchain(
        &mut self,
        manifest: &ScopedRelativePath,
        shell: &ShellName,
        tools: &[ProgramName],
    ) -> Result<Fact, ProbeFault> {
        // Relative to the invocation's working directory, which is where a manifest a runbook
        // names has to be: the runbook travelled, and the manifest belongs to the target
        // environment rather than to the copy.
        let path = manifest.within(&self.workdir);
        if !path.is_file() {
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "no manifest at {}, and this check reads one rather than fetching it",
                    shown(&path.display().to_string())
                ),
                remedy: String::from(
                    "the target work environment supplies the manifest. Put it where the \
                     declaration names it, or drop the declaration if the procedure no longer \
                     needs a pinned toolchain",
                ),
            });
        }
        let directory = path.parent().map(Path::to_owned).unwrap_or_default();
        // Locked, because an unlocked flake resolves its inputs by asking the network — which this
        // probe does not do, so it would fail either way and failing here names the reason.
        if !directory.join("flake.lock").is_file() {
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "{} has no `flake.lock` beside it, so its inputs are not pinned and reading \
                     them would mean resolving them over the network",
                    shown(&path.display().to_string())
                ),
                remedy: String::from(
                    "commit a `flake.lock` for the manifest. A pin nothing recorded is a pin this \
                     check cannot compare against",
                ),
            });
        }
        let Some(nix) = self.which(NIX) else {
            return Err(ProbeFault::Unavailable {
                reason: String::from(
                    "no `nix` resolves on the invocation's search path, and Nix is the only \
                     package manager this adapter reads",
                ),
                remedy: String::from(
                    "install Nix, or state the tools as individual `tool` checks if the procedure \
                     does not actually need the pinned environment",
                ),
            });
        };

        let inputs = self.evaluate_inputs(&nix, &directory, shell)?;
        let realized: Vec<PathBuf> = inputs
            .iter()
            .filter(|input| input.is_dir())
            .cloned()
            .collect();
        if realized.is_empty() {
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "none of the {} input(s) `{shell}` declares is realized in this machine's \
                     store, so no tool's provenance can be established from them",
                    inputs.len()
                ),
                remedy: String::from(
                    "realize the environment with the package manager. This probe builds and \
                     downloads nothing, so a machine that has never entered the shell cannot \
                     establish provenance from it",
                ),
            });
        }

        let mut provenance = Vec::with_capacity(tools.len());
        let mut unresolved = Vec::new();
        let mut outside: Vec<PathBuf> = Vec::new();
        for tool in tools {
            let Some(resolved) = self.which(tool.as_str()) else {
                // Not on the search path at all. Unavailable rather than unmet, because this probe
                // deliberately does not enter the shell: a tool the environment supplies only
                // through a shell hook is not something it can see, and reporting it as absent
                // would state a falsehood about the environment.
                unresolved.push(tool.clone());
                continue;
            };
            let canonical = std::fs::canonicalize(&resolved).unwrap_or_else(|_| resolved.clone());
            match containing_input(&canonical, &realized) {
                Some(input) => provenance.push(ToolProvenance {
                    program: tool.clone(),
                    resolved: Some(canonical),
                    from_inputs: true,
                    input: Some(input),
                }),
                None => {
                    outside.push(canonical.clone());
                    provenance.push(ToolProvenance {
                        program: tool.clone(),
                        resolved: Some(canonical),
                        from_inputs: false,
                        input: None,
                    });
                }
            }
        }

        // The closure query runs only where the direct comparison left something unexplained, and
        // only for the inputs already realized. A tool inside a *dependency* of a declared input is
        // supplied by that environment: measured in this repository, where `jq` resolves to the
        // `-bin` output of an input whose declared output is `-dev`.
        if !outside.is_empty() {
            let closure = self.closures(&realized);
            if !closure.is_empty() {
                for entry in &mut provenance {
                    if entry.from_inputs {
                        continue;
                    }
                    let Some(resolved) = entry.resolved.as_ref() else {
                        continue;
                    };
                    if let Some(input) = containing_input(resolved, &closure) {
                        entry.from_inputs = true;
                        entry.input = Some(input);
                    }
                }
            }
        }

        if !unresolved.is_empty() {
            let names = unresolved
                .iter()
                .map(|tool| format!("`{tool}`"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "{names} do not resolve on the invocation's search path, and this probe does \
                     not enter `{shell}`: a tool that environment supplies only through a shell \
                     hook is not one this check can see",
                ),
                remedy: format!(
                    "run the procedure inside `{shell}`, or make {names} available on the PATH \
                     this invocation uses. This probe enters no shell and builds nothing, so it \
                     cannot establish that provenance itself",
                ),
            });
        }

        Ok(Fact::ToolchainTools {
            manifest: manifest.clone(),
            shell: shell.clone(),
            tools: provenance,
        })
    }

    /// Reads a skill's content within its scope, refusing a path that leaves the scope.
    ///
    /// **Containment is checked on the canonicalized path**, which is the part that makes the scope
    /// mean something. A symlink inside `~/.agents/skills` pointing at a file bundled beside the
    /// runbook would satisfy a lexical check, and `runbook-portability` names exactly that case: a
    /// global skill "is not satisfied by a same-named file bundled beside the runbook".
    fn skill(&mut self, scope: &SkillScope, path: &ScopedRelativePath) -> Result<Fact, ProbeFault> {
        let root = match scope {
            SkillScope::Home => match &self.home {
                Some(home) => home.clone(),
                None => {
                    return Err(ProbeFault::Unavailable {
                        reason: String::from(
                            "this process can name no home directory, so the `home` scope has \
                             nothing to resolve within",
                        ),
                        remedy: String::from(
                            "set `HOME`, or declare the skill in the `workdir` scope if it is \
                             installed beside the work rather than beside the operator",
                        ),
                    })
                }
            },
            SkillScope::Workdir => self.workdir.clone(),
        };
        let Ok(root) = std::fs::canonicalize(&root) else {
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "the {scope} scope's root {} cannot be resolved",
                    root.display(),
                    scope = scope.name()
                ),
                remedy: String::from(
                    "the scope's root has to exist for a path inside it to mean anything",
                ),
            });
        };
        let wanted = path.within(&root);
        let absent = |problem: String| {
            Ok(Fact::SkillAbsent {
                scope: *scope,
                path: path.clone(),
                problem,
            })
        };

        let Ok(metadata) = std::fs::metadata(&wanted) else {
            return absent(String::from("nothing is at that path"));
        };
        if !metadata.is_file() {
            return absent(String::from("that path is not a file"));
        }
        // Canonicalized and compared against the canonicalized root, so a symlink whose target
        // leaves the scope is refused rather than followed.
        let Ok(canonical) = std::fs::canonicalize(&wanted) else {
            return absent(String::from("that path cannot be resolved"));
        };
        if !canonical.starts_with(&root) {
            return absent(format!(
                "it resolves to {}, which is outside the {scope} scope at {}",
                shown(&canonical.display().to_string()),
                root.display(),
                scope = scope.name()
            ));
        }
        // Readable and nonempty, established by reading rather than by trusting the length: a file
        // this process cannot open is not an installed skill whatever its size says.
        match std::fs::File::open(&canonical).and_then(|mut file| {
            let mut probe = [0u8; 1];
            file.read(&mut probe)
        }) {
            Ok(0) => absent(String::from("it is empty, so it states nothing")),
            Ok(_) => Ok(Fact::SkillInstalled {
                scope: *scope,
                path: path.clone(),
                bytes: metadata.len(),
            }),
            Err(error) => absent(format!("it cannot be read: {error}")),
        }
    }

    /// Measures one working-directory state.
    fn state(&mut self, state: &WorkingState) -> Result<Fact, ProbeFault> {
        match state {
            // The existing nearest-project boundary, and the same one `init` stops at: an operation
            // that checked a different boundary than the one it bootstraps against would refuse
            // the runs it should permit and permit the ones it should refuse.
            WorkingState::OutsideProject => match ProjectRoot::resolve(&self.workdir) {
                Ok(project) => Ok(Fact::NearestProject {
                    at: project.path().to_owned(),
                }),
                Err(_) => Ok(Fact::NoNearestProject),
            },
            WorkingState::CleanGitWorktree => self.worktree(),
        }
    }
}

impl<R: CommandRunner> Host<R> {
    /// The worktree's state, read by one fixed read-only Git operation.
    ///
    /// `--no-optional-locks` because a status that took a lock would be a write, and this probe is
    /// required to modify nothing. `--porcelain` because its output is a contract rather than a
    /// rendering, so it does not change with a user's configuration or locale.
    ///
    /// **An absent repository is unavailable rather than clean.** Nothing was measured, and
    /// reporting "clean" for a directory that is not a repository would be the one answer that
    /// makes a run start on a machine the declaration was written for somebody else's.
    fn worktree(&mut self) -> Result<Fact, ProbeFault> {
        let Some(git) = self.which(GIT) else {
            return Err(ProbeFault::Unavailable {
                reason: String::from(
                    "no `git` resolves on the invocation's search path, so the worktree cannot be \
                     read",
                ),
                remedy: String::from(
                    "install Git, or drop the declaration if the procedure does not actually need \
                     a clean tree",
                ),
            });
        };
        let request = ProbeRequest {
            program: git,
            arguments: vec![
                String::from("--no-optional-locks"),
                String::from("status"),
                String::from("--porcelain"),
                // Stated rather than left to the operator's configuration, which can otherwise
                // answer a different question: `status.showUntrackedFiles=no` hides untracked
                // files and `diff.ignoreSubmodules` hides modified submodules, and either made a
                // tree the design calls dirty report clean.
                String::from("--untracked-files=all"),
                String::from("--ignore-submodules=none"),
            ],
            workdir: self.workdir.clone(),
            environment: &[],
        };
        let result = self.run(request)?;
        let RunResult::Exited {
            status,
            stdout,
            stderr,
            ..
        } = result
        else {
            return Err(ProbeFault::Failure {
                reason: String::from("the worktree probe produced no result"),
            });
        };
        if status != 0 {
            let output = said(&stdout, &stderr);
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "the worktree could not be read: `git status` exited with {status} and said \
                     {}",
                    excerpt(&output)
                ),
                remedy: if output.contains("not a git repository") {
                    String::from(
                        "this working directory is not a Git repository, and an absent repository \
                         is not a clean one. Initialize one, or run from the checkout the \
                         procedure expects",
                    )
                } else {
                    String::from("make the working directory readable to `git status`")
                },
            });
        }
        let entries: Vec<String> = stdout
            .lines()
            .map(str::trim_end)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect();
        if entries.is_empty() {
            return Ok(Fact::WorktreeClean);
        }
        // Bounded, so a tree holding ten thousand untracked files produces a report rather than a
        // document. The count is the fact; the names are the part an operator acts on first.
        let mut named: Vec<String> = entries.into_iter().collect();
        let total = named.len();
        named.truncate(DIRTY_ENTRIES_NAMED);
        if total > named.len() {
            named.push(format!("and {} more", total - named.len()));
        }
        Ok(Fact::WorktreeDirty { entries: named })
    }

    /// Evaluates a development shell's declared inputs, read-only and offline.
    ///
    /// The settings are the requirement rather than a preference, and [`evaluation_settings`] says
    /// what each one closes: no substitution, whatever the machine's configuration says, no local
    /// build slot, no remote builder, and no lock written. `--expr` is the one verb's input, with no
    /// `develop`, `build`, or `shell` anywhere in the invocation. An import from a derivation is
    /// allowed, because reading one whose output the store holds builds nothing: a shell that
    /// imports one whose output is missing is refused, neither built nor substituted.
    ///
    /// **What the settings do not close.** `--offline` does not stop a download: an expression can
    /// ask Nix's downloader for a URL, through `builtins.fetchurl` or `builtins.fetchTarball`, and
    /// a locked flake input this machine lacks, a tarball, a `github:` input, or a `git` one, is
    /// fetched to read it. So the child's environment, [`EVALUATION_ENVIRONMENT`], sends every
    /// connection libcurl or `git` would make to a closed local port, whatever proxy the operator
    /// set, and keeps `git` from opening a `git://` or SSH connection of its own: such a fetch is
    /// refused on this machine at once. What those variables do not reach, `libgit2` where no `git`
    /// is on the search path, an `s3://` fetch's credential lookup, and any other route, the design
    /// records as accepted risks.
    ///
    /// A test asserts the assembled argument vector and environment, so this is a property of what
    /// the program sends, and the real-Nix tests hold it against a shell that imports from a
    /// derivation, a configuration that turns substitution on, and shells that fetch from a
    /// listener by every route the environment closes.
    ///
    /// `--impure` is here because the expression reads `builtins.currentSystem`, and the system is
    /// a fact about the machine being checked rather than something a runbook should restate.
    fn evaluate_inputs(
        &mut self,
        nix: &Path,
        directory: &Path,
        shell: &ShellName,
    ) -> Result<Vec<PathBuf>, ProbeFault> {
        // The directory reaches the evaluator as a Nix string, and a Nix string is text. A lossy
        // rendering of a non-UTF-8 name would evaluate some other directory, so it is refused.
        let Some(location) = directory.to_str() else {
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "{} is not a UTF-8 path, and the evaluator can only be told a directory as text",
                    shown(&directory.display().to_string())
                ),
                remedy: String::from(
                    "move the manifest to a directory whose path is valid UTF-8, or state the tools \
                     as individual `tool` checks",
                ),
            });
        };
        let request = ProbeRequest {
            program: nix.to_owned(),
            arguments: evaluation_arguments(location, shell),
            workdir: directory.to_owned(),
            environment: EVALUATION_ENVIRONMENT,
        };
        let result = self.run(request)?;
        let RunResult::Exited {
            status,
            stdout,
            stderr,
            ..
        } = result
        else {
            return Err(ProbeFault::Failure {
                reason: String::from("the manifest evaluation produced no result"),
            });
        };
        if status != 0 {
            // Whatever the evaluator said is the reason. Missing locked sources, an import from a
            // derivation whose output the store lacks, a shell the manifest does not define, and an
            // unsupported Nix all arrive here, and they are all unavailable readiness: this probe
            // will not provision its way to an answer.
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "the manifest could not be evaluated read-only: {}",
                    evaluator_excerpt(&said(&stdout, &stderr))
                ),
                remedy: String::from(
                    "make the environment inspectable from what this machine already holds — its \
                     locked inputs in the store, any file the shell imports from a derivation \
                     built there already, and the shell the declaration names in the manifest. \
                     Entering the shell once realizes them. This probe builds, substitutes, and \
                     updates nothing, and lets none of Nix's own downloads leave the machine",
                ),
            });
        }
        evaluation_inputs(&stdout)
    }
}

/// The store paths an evaluator's answer names, distinct and ordered.
///
/// Split out of the invocation that produces it, and the reason is a test that could not otherwise
/// be deterministic: the response shape is pinned against a real answer from this project's own
/// flake, whose store paths exist on a machine that has built it and not on one that has not. Parsing
/// is the half worth pinning, and parsing needs no store.
///
/// One derivation can be both a build input and a native one, and the same input twice is one place a
/// tool can come from rather than two — so the list is deduplicated here rather than at each use.
pub fn evaluation_inputs(output: &str) -> Result<Vec<PathBuf>, ProbeFault> {
    let unpinned = |reason: String| ProbeFault::Unavailable {
        reason: format!(
            "the evaluator's response is not the JSON array of store paths this adapter parses: \
             {reason}. It said {}",
            excerpt(output)
        ),
        remedy: String::from(
            "this adapter is pinned against one response shape; a package manager that answers \
             differently needs its own adapter rather than a looser parse, because a looser parse \
             is how a check starts answering a question nobody asked",
        ),
    };
    let parsed: serde_json::Value =
        serde_json::from_str(output.trim()).map_err(|error| unpinned(error.to_string()))?;
    let Some(entries) = parsed.as_array() else {
        return Err(unpinned(format!(
            "the answer was {} rather than an array",
            super::describe_value(&parsed)
        )));
    };
    let mut inputs: Vec<PathBuf> = entries
        .iter()
        .map(|entry| match entry.as_str() {
            Some(text) => Ok(PathBuf::from(text)),
            None => Err(unpinned(format!(
                "one entry was {} rather than a store path",
                super::describe_value(entry)
            ))),
        })
        .collect::<Result<Vec<_>, ProbeFault>>()?;
    if let Some(relative) = inputs.iter().find(|path| !path.is_absolute()) {
        return Err(unpinned(format!(
            "{} is not an absolute store path",
            relative.display()
        )));
    }
    inputs.sort();
    inputs.dedup();
    Ok(inputs)
}

/// The Nix settings the evaluation runs under, as one function a test can hand to `nix config show`.
///
/// Every setting here is a promise the design makes about what preflight does not do, or, for the
/// one that allows an import from a derivation, about what it reads without doing anything, which is
/// why they are assembled in one readable place rather than scattered through a builder chain. Each
/// is stated on the command line, which outranks `nix.conf` and `NIX_CONFIG`, so no configuration
/// of the machine being checked can loosen one.
pub fn evaluation_settings() -> Vec<String> {
    vec![
        // The fetcher cache is trusted as it stands and the flake registry is not asked. Alone it
        // does not keep a locked input this machine lacks from being downloaded: 1.3.1 fetched a
        // missing `tarball`, `github:`, and `git+http` input under it. What refuses those is
        // `EVALUATION_ENVIRONMENT`, which sends every client's connection nowhere.
        String::from("--offline"),
        // An import from a derivation is read where this machine already holds its output. A shell
        // `callCabal2nix` builds is a local locked flake's development shell like any other, and
        // `runbook-portability` says this adapter covers one: refusing every such import, as
        // `--no-allow-import-from-derivation` did until 1.3.2, refused imports that needed no build.
        String::from("--option"),
        String::from("allow-import-from-derivation"),
        String::from("true"),
        // And nothing builds one whose output is missing: no local build slot, and no remote
        // builder, so the evaluation is refused, unbuilt, and Nix says what it would have built.
        // `--read-only` is not this guard: with the import allowed, Nix built under it.
        String::from("--max-jobs"),
        String::from("0"),
        String::from("--builders"),
        // The empty list of builders, which the daemon accepts from any client.
        String::new(),
        // Nor is one substituted. `--offline` turns substitution off only where nothing configured
        // it, and a `substitute` line in `nix.conf` or `NIX_CONFIG` counts as configured, so with
        // `--offline` alone such a machine downloaded an import's output and reported it ready.
        // Stated, it holds whatever the configuration says, and the daemon applies it for any
        // client.
        String::from("--option"),
        String::from("substitute"),
        String::from("false"),
        // A download Nix's own downloader makes meets the closed port `EVALUATION_ENVIRONMENT`
        // names, which refuses it at once. Should something drop that connection rather than
        // refuse it, this bounds the wait: one second, rather than the default fifteen, which
        // would outlast the probe's deadline. `git` does not read it.
        String::from("--option"),
        String::from("connect-timeout"),
        String::from("1"),
    ]
}

/// The exact argument vector the adapter sends, as one function so a test can assert all of it.
pub fn evaluation_arguments(directory: &str, shell: &ShellName) -> Vec<String> {
    let mut arguments = vec![
        String::from("--extra-experimental-features"),
        String::from("nix-command flakes"),
        String::from("eval"),
    ];
    arguments.extend(evaluation_settings());
    arguments.extend([
        // The lock is read, never moved: a check that changed what it was checking would leave the
        // tree it promised not to modify.
        String::from("--no-write-lock-file"),
        // `builtins.currentSystem` is a fact about this machine, and reading it is what makes the
        // expression impure.
        String::from("--impure"),
        String::from("--json"),
        String::from("--expr"),
        evaluation_expression(directory, shell),
    ]);
    arguments
}

/// A proxy that refuses every connection at once: port 1 of the loopback interface.
///
/// A closed local port rather than an address nothing answers, because `git` reads the proxy
/// variables and not `connect-timeout`: with `192.0.2.1`, which the first version named, a missing
/// `git+https` input waited out the probe's deadline and was reported as timed out with nothing
/// known, where a refused connection fails it within a second with git's own error. Port 1 is
/// TCP's port-service multiplexer, which nothing serves in practice, and binding it on Linux takes
/// root; a process that did listen there would still be on this machine. Port 0, which no process
/// can listen on, was measured to refuse as fast, and was not chosen: libcurl reads a proxy port of
/// 0 as "use the default", 1080, in one of its interfaces, and 1080 is where a local SOCKS proxy
/// commonly runs, so a client that took a URL's `:0` that way would hand the request to one.
const CLOSED_PROXY: &str = "http://127.0.0.1:1";

/// What the evaluation's child changes in the environment it inherits.
///
/// First every variable whose name ends in `_proxy`, in either case, is removed: libcurl prefers
/// the variable named for a URL's scheme over `all_proxy`, so an operator's `ftp_proxy` would carry
/// an `ftp://` fetch past the settings below, and `no_proxy` would except hosts from them. Then
/// six are set, to [`CLOSED_PROXY`]: `http_proxy`, `https_proxy`, and `all_proxy`, in both cases,
/// because libcurl and `git` read those, every scheme without a variable of its own falls to
/// `all_proxy`, and `HTTP_PROXY`, which libcurl ignores, is set for any other client that reads it.
/// `GIT_PROXY_COMMAND` and `GIT_SSH_COMMAND` are `false`, so `git` opens no `git://` or SSH
/// connection of its own, which it would make directly, past any proxy. A request the evaluation
/// asks Nix or `git` for is therefore refused on this machine at once.
pub const EVALUATION_ENVIRONMENT: &[EnvironmentChange] = &[
    EnvironmentChange::RemoveEveryEndingIn { suffix: "_proxy" },
    EnvironmentChange::Set {
        name: "http_proxy",
        value: CLOSED_PROXY,
    },
    EnvironmentChange::Set {
        name: "HTTP_PROXY",
        value: CLOSED_PROXY,
    },
    EnvironmentChange::Set {
        name: "https_proxy",
        value: CLOSED_PROXY,
    },
    EnvironmentChange::Set {
        name: "HTTPS_PROXY",
        value: CLOSED_PROXY,
    },
    EnvironmentChange::Set {
        name: "all_proxy",
        value: CLOSED_PROXY,
    },
    EnvironmentChange::Set {
        name: "ALL_PROXY",
        value: CLOSED_PROXY,
    },
    EnvironmentChange::Set {
        name: "GIT_PROXY_COMMAND",
        value: "false",
    },
    EnvironmentChange::Set {
        name: "GIT_SSH_COMMAND",
        value: "false",
    },
];

/// The expression the adapter evaluates.
///
/// Two values reach it, and each is kept out of the expression's syntax a different way. The shell
/// is a [`ShellName`], whose constructor admits ASCII alphanumerics, `-`, and `_` and nothing else,
/// so it is interpolated as an attribute name. A shell named outside that charset is refused at
/// parse time, in the runbook's own document, before any expression exists. The directory is not
/// constrained — it is wherever the manifest is, and a path can hold a space, a quote, or `${` — so
/// it is written as an escaped string literal by [`nix_string_literal`] rather than interpolated.
///
/// Reads `nativeBuildInputs` and `buildInputs` because a tool can be supplied by either, and
/// filters to entries that have an output path because a shell's inputs can also hold strings and
/// nested lists that name no derivation.
fn evaluation_expression(directory: &str, shell: &ShellName) -> String {
    format!(
        "let flake = builtins.getFlake (\"path:\" + {directory}); \
         shell = flake.devShells.${{builtins.currentSystem}}.{shell}; \
         inputs = (shell.nativeBuildInputs or []) ++ (shell.buildInputs or []); \
         in builtins.map (input: input.outPath) \
         (builtins.filter (entry: builtins.isAttrs entry && entry ? outPath) inputs)",
        directory = nix_string_literal(directory),
        shell = shell.as_str()
    )
}

/// `text` as a double-quoted Nix string literal that evaluates to exactly `text`.
///
/// Inside a double-quoted Nix string, `\` escapes, `"` ends the string, and `${` opens an
/// interpolation. Escaping those three is sufficient: every other character, a newline included,
/// stands for itself.
fn nix_string_literal(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\\' => literal.push_str("\\\\"),
            '"' => literal.push_str("\\\""),
            '$' if characters.peek() == Some(&'{') => literal.push_str("\\$"),
            other => literal.push(other),
        }
    }
    literal.push('"');
    literal
}

/// The input whose store path contains `resolved`, if there is one.
///
/// A prefix comparison on store paths rather than a name comparison, because the question is where
/// the bytes came from and a store path is the answer Nix gives.
fn containing_input(resolved: &Path, inputs: &[PathBuf]) -> Option<PathBuf> {
    inputs
        .iter()
        .find(|input| resolved.starts_with(input))
        .cloned()
}

/// The one standalone three-component numeric version token in `output`.
///
/// **Exactly one, and "standalone" is the load-bearing word.** A tool prints more than a version —
/// `cargo 1.90.0 (840b83a10 2025-07-30)` carries a commit and a date — and a parse that took the
/// first number-like thing would read a date as a version. So a token is a maximal run of digits
/// and dots that is exactly three numeric components and is not followed by an alphanumeric
/// character, a hyphen, or an underscore: `1.2.3.4` is not a three-component token, and the `1.2.3`
/// inside `v1.2.3-beta` is a prerelease rather than a release.
///
/// **What precedes the run does not disqualify it.** Real tools glue a name or a marker to the
/// front of their version — `v22.3.0`, `jq-1.7.1`, `go1.22.1` — and a prefix changes what the
/// version is called, not which version it is. A suffix is different: it names a different
/// version, so a trailing word still refuses the run.
///
/// Zero tokens and two tokens are both refusals. Two is the interesting one: a tool that prints its
/// own version and a library's is ambiguous, and choosing between them would be this program
/// deciding which version a runbook meant.
pub fn version_token(output: &str) -> Result<ToolVersion, String> {
    let characters = output.chars().collect::<Vec<_>>();
    let mut found: Vec<String> = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        if !(characters[index].is_ascii_digit() || characters[index] == '.') {
            index += 1;
            continue;
        }
        let start = index;
        while index < characters.len()
            && (characters[index].is_ascii_digit() || characters[index] == '.')
        {
            index += 1;
        }
        // Maximal by construction, so only the following character can make it part of a longer
        // version. The preceding one is a name or a marker and leaves the version unchanged.
        let suffixed = characters
            .get(index)
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_');
        if suffixed {
            continue;
        }
        let run: String = characters[start..index].iter().collect();
        if ToolVersion::parse(&run).is_ok() {
            found.push(run);
        }
    }

    match found.len() {
        1 => ToolVersion::parse(&found[0]).map_err(|error| error.to_string()),
        0 => Err(format!(
            "its output holds no standalone three-component version: {}",
            excerpt(output)
        )),
        several => Err(format!(
            "its output holds {several} standalone three-component versions ({}), and this probe \
             does not choose between them: {}",
            found.join(", "),
            excerpt(output)
        )),
    }
}

/// The first lines of a probe's output, for a report that has to quote it.
///
/// Bounded, because the output itself is bounded but a report carrying 64 KiB of a compiler's
/// diagnostics is not a report anybody reads.
fn excerpt(output: &str) -> String {
    let taken: Vec<&str> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(3)
        .collect();
    bounded(taken.join(" / "))
}

/// The most of a program's output a report quotes, in characters.
const QUOTE_BOUND: usize = 300;

/// `text`, cut at [`QUOTE_BOUND`] characters and marked where it was cut, or `nothing`.
///
/// Counted in characters rather than bytes, so a cut never falls inside one.
fn bounded(text: String) -> String {
    if text.is_empty() {
        return String::from("nothing");
    }
    if text.chars().count() > QUOTE_BOUND {
        let truncated: String = text.chars().take(QUOTE_BOUND).collect();
        return format!("{truncated}…");
    }
    text
}

/// How Nix opens a line that states why an evaluation failed.
const EVALUATOR_ERROR: &str = "error:";

/// How Nix opens advice about its own settings, which this adapter fixes on its command line.
const EVALUATOR_HINT: &str = "Hint:";

/// What a failed evaluation said, from the first error that states a cause.
///
/// Nix opens a trace with a bare `error:` and ends it with the error that stopped the evaluation,
/// and a build it was asked for and declined reports its own errors before the trace: the 1.3.1
/// report quoted the trace's first three lines, `error: / … while calling the 'map' builtin / at
/// «string»:1:217:`, and never reached the line that said why. So the excerpt starts at the first
/// line that opens with `error:` and states something after it, or that is a bare `error:` whose
/// message follows on the next line rather than a frame of a trace, as Nix 2.22 writes a build it
/// cannot start, joined to that line. It runs to the end of the output, within [`QUOTE_BOUND`]
/// characters but no limit of lines. The first one, because where an import's
/// own dependency is what is missing, the last error Nix writes is `Build failed due to failed
/// dependency` and the first is the build it declined; and no limit of lines, because Nix gives the
/// reason a foreign import was declined, `platform mismatch` and the system it needs, on the fourth
/// to sixth lines of that error.
///
/// Each line is trimmed, because Nix indents all but the first and the quote joins them into one.
/// A `Hint:` line is left out, because it advises changing `max-jobs` or `builders`, which this
/// adapter states on the command line and an operator cannot change for it. Terminal escape
/// sequences are removed first: Nix 2.34.6 colours nothing it writes to a pipe, but a later Nix can
/// be told to, and a coloured `error:` would otherwise not be found. Output with no such line is
/// quoted as [`excerpt`] quotes any probe's.
///
/// The evaluation's alone, because the order is a fact about Nix: a tool's version output, `git
/// status`, and an answer of the wrong shape are quoted from their first lines, as they were.
fn evaluator_excerpt(output: &str) -> String {
    let plain = without_terminal_escapes(output);
    let lines: Vec<&str> = plain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    // An `error:` states a cause on its own line, or, bare, on the lines after it, as Nix 2.22
    // writes a build it cannot start. The bare one that opens a trace is followed by a frame.
    let states_a_cause = |index: &usize| {
        let Some(rest) = lines
            .get(*index)
            .and_then(|line| line.strip_prefix(EVALUATOR_ERROR))
        else {
            return false;
        };
        !rest.trim().is_empty()
            || lines.get(index + 1).is_some_and(|next| {
                !next.starts_with(TRACE_FRAME) && !next.starts_with(EVALUATOR_ERROR)
            })
    };
    let Some(start) = (0..lines.len()).find(states_a_cause) else {
        return excerpt(&plain);
    };
    let mut quoted: Vec<String> = Vec::new();
    let mut rest = lines.iter().skip(start);
    if let Some(first) = rest.next() {
        // A bare `error:` is joined with the line that states its cause, so the quote reads as one.
        quoted.push(match first.strip_prefix(EVALUATOR_ERROR) {
            Some(after) if after.trim().is_empty() => match rest.next() {
                Some(cause) => format!("{EVALUATOR_ERROR} {cause}"),
                None => (*first).to_owned(),
            },
            _ => (*first).to_owned(),
        });
    }
    quoted.extend(
        rest.filter(|line| !line.starts_with(EVALUATOR_HINT))
            .map(|line| (*line).to_owned()),
    );
    bounded(quoted.join(" / "))
}

/// How Nix opens each frame of a trace.
const TRACE_FRAME: char = '…';

/// `text` without the terminal escape sequences a program writes to colour it.
///
/// A control sequence, `ESC [` up to its final byte from `@` to `~`; an operating-system command,
/// `ESC ]` up to a bell or `ESC \`; an escape with intermediate bytes, such as `ESC ( B`, its
/// bytes from space to `/` and the one after them; and any other escape, `ESC` and the character
/// after it. Total: an escape the text ends inside is dropped with the rest of the text.
fn without_terminal_escapes(text: &str) -> String {
    const ESCAPE: char = '\u{1b}';
    const BELL: char = '\u{7}';
    let mut plain = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != ESCAPE {
            plain.push(character);
            continue;
        }
        match characters.next() {
            Some('[') => {
                for parameter in characters.by_ref() {
                    if ('@'..='~').contains(&parameter) {
                        break;
                    }
                }
            }
            Some(']') => loop {
                match characters.next() {
                    None | Some(BELL) => break,
                    Some(ESCAPE) => {
                        characters.next();
                        break;
                    }
                    Some(_) => {}
                }
            },
            Some(intermediate) if (' '..='/').contains(&intermediate) => {
                for byte in characters.by_ref() {
                    if !(' '..='/').contains(&byte) {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    plain
}

/// Observes every check a contract declares, in document order.
///
/// **Sequential, which the design requires and which costs nothing worth measuring.** A parallel
/// probe would make the order of a report depend on which tool answered first, and a report whose
/// order moves between two runs of the same machine is one an operator cannot diff.
///
/// Cancellation is checked before each probe and after each one, so an interruption between probes
/// stops the loop and an interruption during one is seen when the runner returns it. Either way
/// nothing is published: `runbook-portability` requires that an interrupted preflight "SHALL stop
/// its probes and publish no record", and a caller holding `Err` has no observations to publish
/// from.
pub fn observe_all(
    contract: &super::MachineContract,
    host: &mut impl HostProbe,
) -> Result<Observations, Interrupted> {
    let mut observed = Vec::with_capacity(contract.len());
    for declared in contract.checks() {
        if host.cancelled() {
            return Err(Interrupted);
        }
        let observation = observe_one(&declared.check, host);
        if host.cancelled() {
            return Err(Interrupted);
        }
        observed.push(Observed {
            requirement: declared.requirement.clone(),
            check: declared.check.clone(),
            observation,
        });
    }
    // `Observations::of` cannot fail here: one observation was made per declared check, each
    // carrying the identity and the check it was made for. The `Result` is the type's own guarantee
    // rather than a case this caller can reach, and mapping it to `Interrupted` would be a lie, so
    // the unreachable shape is reported as a probe failure the caller can see.
    Observations::of(contract, observed).map_err(|_| Interrupted)
}

/// An interruption that stopped the probes.
///
/// Its own type rather than an `Observation` variant, because it is not a fact about the machine:
/// it is the absence of an answer, and the CLI maps it to the cancellation exit code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Interrupted;

/// One check, observed.
fn observe_one(check: &MachineCheck, host: &mut impl HostProbe) -> Observation {
    super::observe(check, host)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preflight::{DeclaredCheck, MachineContract, RequirementId};

    /// A temporary directory that removes itself.
    ///
    /// Canonicalized on creation, because on macOS the temporary directory is itself a symlink and
    /// an uncanonicalized expectation compares `/var/...` against `/private/var/...` and fails for
    /// the wrong reason.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("htmlspec-host-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            let path = path.canonicalize().expect("the scratch path resolves");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        /// A directory beneath the scratch, created.
        fn directory(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            std::fs::create_dir_all(&path).expect("the directory is created");
            path
        }

        /// A file beneath the scratch, written.
        fn write(&self, relative: &str, contents: &str) -> PathBuf {
            let path = self.0.join(relative);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the parent exists");
            std::fs::write(&path, contents).expect("the file is written");
            path
        }

        /// An executable script beneath the scratch, in a directory of its own.
        ///
        /// Returns the directory, which is what a host takes as its search path.
        fn executable(&self, relative: &str, body: &str) -> PathBuf {
            let path = self.write(relative, body);
            make_executable(&path);
            path.parent().expect("a directory").to_owned()
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A host over a scripted runner, observing a scratch directory.
    ///
    /// The search path is passed rather than inherited, which is what keeps these tests off this
    /// machine's installations: a host built here resolves only what the test put in the directory
    /// it names.
    fn scripted(
        scratch: &Scratch,
        search: &[PathBuf],
        runner: ScriptedRunner,
    ) -> Host<ScriptedRunner> {
        Host::new(
            scratch.path().to_owned(),
            Some(scratch.path().to_owned()),
            search.to_owned(),
            runner,
        )
    }

    fn name(text: &str) -> ProgramName {
        ProgramName::new(text).expect("a name")
    }

    fn shell(text: &str) -> ShellName {
        ShellName::new(text).expect("a shell name")
    }

    fn path(text: &str) -> ScopedRelativePath {
        ScopedRelativePath::new(text).expect("a path")
    }

    /// Gives a file the executable bit, on a platform that has one.
    fn make_executable(path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(path).expect("readable").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(path, permissions).expect("the mode is set");
        }
    }

    /// A tool that reports one version is measured, and not judged.
    ///
    /// The fact/verdict split asserted at the boundary: this reports `1.90.0` whatever the
    /// declaration said, and [`crate::preflight::assess`] is what decides whether that matches.
    #[test]
    fn a_tool_reporting_one_version_is_measured() {
        let scratch = Scratch::new("tool-version");
        let search = scratch.executable("bin/cargo", "#!/bin/sh\necho cargo 1.90.0\n");
        let executable = search.join("cargo");
        let mut host = scripted(
            &scratch,
            std::slice::from_ref(&search),
            ScriptedRunner::always("cargo 1.90.0 (840b83a10 2025-07-30)"),
        );

        let fact = host.tool(&name("cargo")).expect("measured");
        assert!(
            matches!(&fact, Fact::ToolVersion { version, resolved, .. }
                if version.to_string() == "1.90.0" && *resolved == executable),
            "{fact:?}"
        );
        // The fixed argument, resolved once and invoked by path, with no shell in between.
        let invoked = host.runner.invoked();
        assert_eq!(
            invoked,
            vec![format!("cargo {VERSION_ARGUMENT}")],
            "{invoked:?}"
        );
        let request = &host.runner.requests.lock().expect("the requests")[0];
        assert_eq!(
            request.program, executable,
            "the bare name was invoked instead"
        );
        assert_eq!(
            request.workdir,
            scratch.path(),
            "the probe ran somewhere else"
        );
    }

    /// A tool that is not on the search path is a measured absence rather than an error.
    ///
    /// Absence is established by resolution, so no process is started for it — which is also why an
    /// absent tool cannot time out.
    #[test]
    fn a_tool_that_does_not_resolve_is_absent_rather_than_unknown() {
        let scratch = Scratch::new("tool-absent");
        let empty = scratch.directory("empty-bin");
        let mut host = scripted(&scratch, &[empty], ScriptedRunner::always(""));

        let fact = host
            .tool(&name("definitely-not-installed"))
            .expect("measured");
        assert!(
            matches!(fact, Fact::ToolAbsent { .. }),
            "an absent tool was reported as {fact:?}"
        );
        assert!(
            host.runner.invoked().is_empty(),
            "a process ran for a tool that does not resolve: {:?}",
            host.runner.invoked()
        );
    }

    /// The first match on the search path wins, which is the rule a shell uses.
    ///
    /// A probe that searched the whole path would answer a different question than the one that
    /// matters: which executable the procedure will actually run.
    #[test]
    fn the_first_match_on_the_search_path_is_the_one_measured() {
        let scratch = Scratch::new("search-order");
        let shadowing = scratch.executable("first/tool", "#!/bin/sh\necho 1.0.0\n");
        let shadowed = scratch.executable("second/tool", "#!/bin/sh\necho 2.0.0\n");
        let mut host = scripted(
            &scratch,
            &[shadowing.clone(), shadowed.clone()],
            ScriptedRunner::always("tool 1.0.0"),
        );

        match host.tool(&name("tool")).expect("measured") {
            Fact::ToolVersion {
                resolved, version, ..
            } => {
                assert_eq!(resolved, shadowing.join("tool"), "the later entry won");
                assert_eq!(version.to_string(), "1.0.0");
            }
            other => panic!("{other:?}"),
        }
    }

    /// Ambiguous and unparseable version output are both measured as unreadable.
    ///
    /// Two tokens is the case that matters: a tool printing its own version and a library's is
    /// ambiguous, and choosing between them would be this program deciding which version a runbook
    /// meant.
    #[test]
    fn version_output_that_is_ambiguous_or_unparseable_is_not_a_version() {
        for (output, wanted) in [
            ("cargo", "no standalone three-component version"),
            ("", "no standalone three-component version"),
            ("1.2.3.4", "no standalone three-component version"),
            ("v1.2.3-beta", "no standalone three-component version"),
            ("1.2.3rc1", "no standalone three-component version"),
            ("1.2", "no standalone three-component version"),
            (
                "tool 1.2.3 with library 4.5.6",
                "2 standalone three-component versions",
            ),
        ] {
            let problem = version_token(output).expect_err("read as a version");
            assert!(
                problem.contains(wanted),
                "{output:?} reported {problem:?} and not {wanted:?}"
            );
        }

        // And the shapes real tools print, which are the reason the rule is what it is.
        for (output, version) in [
            ("cargo 1.90.0 (840b83a10 2025-07-30)", "1.90.0"),
            ("Python 3.13.2 (main, Feb  4 2025)", "3.13.2"),
            ("git version 2.39.3 (Apple Git-145)", "2.39.3"),
            ("nix (Nix) 2.24.10", "2.24.10"),
            ("1.2.3", "1.2.3"),
            // A name or marker glued to the front is still the version.
            ("v22.3.0", "22.3.0"),
            ("jq-1.7.1", "1.7.1"),
            ("go version go1.22.1 darwin/arm64", "1.22.1"),
        ] {
            assert_eq!(
                version_token(output).expect("a version").to_string(),
                version,
                "{output:?}"
            );
        }
    }

    /// A tool that exits nonzero is measured as unreadable, naming the status and what it said.
    #[test]
    fn a_tool_that_refuses_the_question_is_measured_rather_than_assumed() {
        let scratch = Scratch::new("tool-refuses");
        let search = scratch.executable("bin/broken", "#!/bin/sh\nexit 3\n");
        let mut host = scripted(
            &scratch,
            &[search],
            ScriptedRunner::new(vec![RunResult::Exited {
                status: 3,
                stdout: String::from("it would not say"),
                stderr: String::new(),
                truncated: false,
            }]),
        );

        match host.tool(&name("broken")).expect("measured") {
            Fact::ToolVersionUnreadable { problem, .. } => {
                assert!(problem.contains("status 3"), "{problem}");
                assert!(problem.contains("it would not say"), "{problem}");
            }
            other => panic!("a nonzero exit was reported as {other:?}"),
        }
    }

    /// A timeout and an output overflow are both unavailable readiness, and neither is a fact.
    ///
    /// Two different endings with one consequence, and the reports differ because the remedies do:
    /// a slow tool might finish on another machine, while a tool whose version interface does not
    /// fit a bounded probe never will.
    #[test]
    fn a_probe_that_overruns_a_bound_reports_unavailable_readiness() {
        let scratch = Scratch::new("bounds");
        let search = scratch.executable("bin/slow", "#!/bin/sh\nsleep 30\n");

        let mut host = scripted(
            &scratch,
            std::slice::from_ref(&search),
            ScriptedRunner::new(vec![RunResult::TimedOut]),
        );
        let fault = host
            .tool(&name("slow"))
            .expect_err("a timeout is not a fact");
        assert!(
            matches!(fault, ProbeFault::TimedOut { bound } if bound == DEADLINE),
            "{fault:?}"
        );

        let mut host = scripted(
            &scratch,
            &[search],
            ScriptedRunner::new(vec![RunResult::OutputExceeded {
                limit: OUTPUT_LIMIT,
            }]),
        );
        match host
            .tool(&name("slow"))
            .expect_err("an overflow is not a fact")
        {
            ProbeFault::Unavailable { reason, remedy } => {
                assert!(reason.contains(&OUTPUT_LIMIT.to_string()), "{reason}");
                assert!(
                    remedy.contains("adapter"),
                    "the remedy does not say what would fix it: {remedy}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// A deadline clock that answers as the test says, and counts each time it was asked.
    ///
    /// The runner asks it only after finding the child still running, so its answers stand for
    /// time passing while a real process runs, and none of it is waited for.
    struct ScriptedClock {
        /// How many times the runner asked before this one, each answered with no time passed.
        within: usize,
        /// What the answer after them does: say the deadline has passed, or raise a cancellation.
        then: AfterThem,
        asked: std::cell::Cell<usize>,
    }

    enum AfterThem {
        PastTheDeadline,
        Cancel(Cancellation),
    }

    impl ScriptedClock {
        fn new(within: usize, then: AfterThem) -> Self {
            Self {
                within,
                then,
                asked: std::cell::Cell::new(0),
            }
        }
    }

    impl DeadlineClock for ScriptedClock {
        fn elapsed_since(&self, _started: Instant) -> Duration {
            let asked = self.asked.get();
            self.asked.set(asked + 1);
            if asked < self.within {
                return Duration::ZERO;
            }
            match &self.then {
                AfterThem::PastTheDeadline => DEADLINE + Duration::from_millis(1),
                AfterThem::Cancel(cancellation) => {
                    cancellation.cancel();
                    Duration::ZERO
                }
            }
        }
    }

    /// A child that will not finish by itself: `sleep 30`.
    fn sleeping_child() -> ProbeRequest {
        let Some(sleep) = real_executable("sleep") else {
            // No `sleep` means no way to start a child that will not finish. Named rather than
            // passed over silently, because a skipped assertion is not an assertion.
            panic!("this test needs a `sleep` executable to start a process that will not finish");
        };
        ProbeRequest {
            program: sleep,
            arguments: vec![String::from("30")],
            workdir: std::env::temp_dir(),
            environment: &[],
        }
    }

    /// **A real child process is stopped at the deadline and reaped.**
    ///
    /// The one test here that runs a program until it is stopped, and it exists because a scripted
    /// timeout proves the mapping from `TimedOut` to unavailable readiness and nothing about
    /// termination. The deadline is read from a clock passed in, which says it has passed on its
    /// third asking, so the runner stops a real child at its deadline without the test waiting the
    /// deadline's ten seconds out. The assertions are that the call returns as a timeout, that it did
    /// not return before its clock said the deadline had passed, and that it returned at once.
    #[test]
    fn a_real_child_process_is_stopped_at_the_deadline() {
        let clock = ScriptedClock::new(2, AfterThem::PastTheDeadline);
        let started = Instant::now();
        let result = SystemRunner.run_on(&sleeping_child(), &Cancellation::new(), &clock);
        let elapsed = started.elapsed();

        assert_eq!(result, RunResult::TimedOut, "the child was not stopped");
        assert_eq!(
            clock.asked.get(),
            3,
            "the probe returned before its clock said the deadline had passed, or ran past it"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "the probe waited for the child rather than stopping it: {elapsed:?}"
        );
    }

    /// **A real cancellation stops a running child before the call returns.**
    ///
    /// The other half of the transition the design specifies: "Cancellation stops and reaps
    /// outstanding processes … before returning." Asserted by raising the signal while the child
    /// runs, and requiring the call to come back as cancelled rather than as a timeout — which it can
    /// only do by observing the flag mid-flight. The signal is raised by the clock the runner asks
    /// once it has found the child still running, rather than by a thread that sleeps first, so the
    /// child is known to be running when it is raised and nothing waits for it to start.
    #[test]
    fn a_real_cancellation_stops_a_running_child() {
        let cancellation = Cancellation::new();
        let clock = ScriptedClock::new(1, AfterThem::Cancel(cancellation.clone()));
        let started = Instant::now();
        let result = SystemRunner.run_on(&sleeping_child(), &cancellation, &clock);

        assert_eq!(result, RunResult::Cancelled, "the child was not stopped");
        assert_eq!(
            clock.asked.get(),
            2,
            "the flag raised mid-flight was not what stopped the child at the next look"
        );
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "cancellation did not reach the running probe"
        );
    }

    /// The real deadline clock reads the monotonic time since the child started.
    #[test]
    fn the_real_deadline_clock_reads_the_time_since_the_start() {
        let started = Instant::now();
        let earlier = started.checked_sub(DEADLINE).unwrap_or(started);
        assert!(Monotonic.elapsed_since(started) < DEADLINE);
        if earlier < started {
            assert!(Monotonic.elapsed_since(earlier) >= DEADLINE);
        }
    }

    /// A real child is started in its own process group, so a probe that forks can be stopped whole.
    ///
    /// The child reports its own group id and its parent's; a group it created differs from the group
    /// it was started in, which is the property `kill` on a group child depends on. **Read from
    /// `/proc`-free shell builtins rather than from `ps`**, because the build sandbox this suite runs
    /// in has no `ps`: a test that passed locally and failed in continuous integration would be
    /// testing the contributor's machine, which is the divergence one shared suite exists to prevent.
    /// `sh` is present in both, and `$PPID` and a group id read from the kernel are enough.
    #[test]
    fn a_real_child_is_started_in_its_own_process_group() {
        let Some(shell) = real_executable("sh") else {
            panic!("this test needs a shell to ask a child which group it is in");
        };
        // `ps` is what a local run would reach for and the sandbox does not have it, so the group id
        // comes from the one place every Unix reads it: the process's own pid is its group's id when
        // it leads the group. A child that leads its group has `getpgid(0) == getpid()`, and `sh` can
        // ask the kernel for both without an external program.
        let result = SystemRunner.run(
            &ProbeRequest {
                program: shell,
                arguments: vec![
                    String::from("-c"),
                    // `kill -0 -$pid` succeeds exactly where a process group of that id exists, and
                    // fails with ESRCH where it does not — which is the question, asked of the kernel
                    // rather than of a listing.
                    String::from(
                        "if kill -0 -$$ 2>/dev/null; then echo group-leader; else echo                          not-a-group-leader; fi",
                    ),
                ],
                workdir: std::env::temp_dir(),
                environment: &[],
            },
            &Cancellation::new(),
        );
        let RunResult::Exited {
            status: 0,
            stdout: output,
            ..
        } = result
        else {
            panic!("the child did not answer: {result:?}");
        };
        assert!(
            output.contains("group-leader"),
            "the child does not lead its own process group, so a group kill would reach the wrong              processes: {output:?}"
        );
        assert!(!output.contains("not-a-group-leader"), "{output:?}");
    }

    /// An executable on this machine's own search path, for the tests that need a real process.
    fn real_executable(program: &str) -> Option<PathBuf> {
        environment_search_path()
            .into_iter()
            .map(|directory| directory.join(program))
            .find(|candidate| is_executable(candidate))
    }

    /// The adapter's invocation carries every read-only flag and no verb that provisions.
    ///
    /// Asserted on the assembled argument vector rather than on a comment, because "never calls
    /// `nix develop`, `build`, `shell`, a fetch command, or anything that realizes missing outputs"
    /// is a property of what the program sends and only a test can hold it.
    #[test]
    fn the_adapter_invocation_is_read_only_and_names_no_verb_that_provisions() {
        let arguments = evaluation_arguments("/work/project", &shell("default"));
        let joined = arguments.join(" ");

        // Every argument before the expression, as one literal. Nix takes the last value it is
        // given for a setting, so a later `--max-jobs 4` or `--option substitute true` would undo
        // one of these while a test that looked for each would still pass: only the whole vector
        // rules that out. It reads the import a store holds, builds nothing here or on a builder,
        // substitutes nothing whatever the configuration says, bounds a fetch's wait, and leaves
        // the lock alone. `--no-allow-import-from-derivation`, which refused every import from a
        // derivation, built or not, is not among them.
        let Some((expression, settled)) = arguments.split_last() else {
            panic!("the invocation is empty");
        };
        assert_eq!(
            settled,
            [
                "--extra-experimental-features",
                "nix-command flakes",
                "eval",
                "--offline",
                "--option",
                "allow-import-from-derivation",
                "true",
                "--max-jobs",
                "0",
                "--builders",
                "",
                "--option",
                "substitute",
                "false",
                "--option",
                "connect-timeout",
                "1",
                "--no-write-lock-file",
                "--impure",
                "--json",
                "--expr",
            ],
            "the invocation's arguments before its expression changed: {joined}"
        );
        // The settings a test hands `nix config show` are exactly the ones the invocation carries.
        assert_eq!(
            &settled[3..17],
            evaluation_settings().as_slice(),
            "the settings and the invocation disagree"
        );
        // The subcommand is `eval`, and it is the only verb: `develop`, `build`, `shell`, and the
        // fetch commands are all things this probe must not do, and all of them would appear here.
        assert_eq!(
            arguments.iter().position(|argument| argument == "eval"),
            Some(2),
            "the invocation's subcommand is not `eval`: {joined}"
        );
        for forbidden in [
            "develop", "build", "shell", "fetch", "run", "profile", "install",
        ] {
            assert!(
                !arguments.iter().any(|argument| argument == forbidden),
                "`{forbidden}` is an argument of the invocation: {joined}"
            );
        }

        // The child's environment, whole: every proxy variable the operator set removed, six set to
        // the closed local port, and `git`'s own connections closed.
        let closed = "http://127.0.0.1:1";
        assert_eq!(
            EVALUATION_ENVIRONMENT,
            [
                EnvironmentChange::RemoveEveryEndingIn { suffix: "_proxy" },
                EnvironmentChange::Set {
                    name: "http_proxy",
                    value: closed
                },
                EnvironmentChange::Set {
                    name: "HTTP_PROXY",
                    value: closed
                },
                EnvironmentChange::Set {
                    name: "https_proxy",
                    value: closed
                },
                EnvironmentChange::Set {
                    name: "HTTPS_PROXY",
                    value: closed
                },
                EnvironmentChange::Set {
                    name: "all_proxy",
                    value: closed
                },
                EnvironmentChange::Set {
                    name: "ALL_PROXY",
                    value: closed
                },
                EnvironmentChange::Set {
                    name: "GIT_PROXY_COMMAND",
                    value: "false"
                },
                EnvironmentChange::Set {
                    name: "GIT_SSH_COMMAND",
                    value: "false"
                },
            ],
            "the evaluation's environment changed"
        );

        // The expression reads both input lists and asks for store paths, which is what makes the
        // answer about the environment rather than about one attribute of it.
        for wanted in [
            "nativeBuildInputs",
            "buildInputs",
            "devShells.",
            ".default",
            "outPath",
            "\"/work/project\"",
        ] {
            assert!(
                expression.contains(wanted),
                "the expression does not carry {wanted:?}: {expression}"
            );
        }
    }

    /// A shell name reaching the expression cannot carry expression syntax.
    ///
    /// The constructor refuses it, so this asserts the boundary from outside: no name that could
    /// close a string or open a brace is accepted, and an accepted name lands in the expression
    /// verbatim.
    #[test]
    fn no_shell_name_can_reach_the_expression_as_syntax() {
        for hostile in [
            "a\"; builtins.trace \"x",
            "${builtins.currentTime}",
            "a b",
            "a}",
            "",
        ] {
            assert!(
                ShellName::new(hostile).is_err(),
                "{hostile:?} was accepted as a shell name"
            );
        }
        let expression = evaluation_expression("/work", &shell("rust-nightly"));
        assert!(
            expression.contains(".rust-nightly"),
            "the accepted name did not reach the expression: {expression}"
        );
    }

    /// A manifest directory reaches the expression as one escaped string, whatever it is called.
    ///
    /// The directory is not validated the way a shell name is — it is wherever the manifest sits —
    /// so a space, a quote, a backslash, and `${` all have to arrive as text rather than as syntax.
    /// Before the directory was quoted, a space split the expression and `${` was evaluated.
    #[test]
    fn a_manifest_directory_reaches_the_expression_as_text_rather_than_syntax() {
        for (directory, literal) in [
            ("/work/project", r#""/work/project""#),
            ("/work/my project", r#""/work/my project""#),
            (r#"/work/a"b"#, r#""/work/a\"b""#),
            (r"/work/a\b", r#""/work/a\\b""#),
            (
                "/work/${builtins.currentTime}",
                r#""/work/\${builtins.currentTime}""#,
            ),
            // A `$` that opens no interpolation is left alone.
            ("/work/$HOME", r#""/work/$HOME""#),
        ] {
            assert_eq!(nix_string_literal(directory), literal, "{directory:?}");
            let expression = evaluation_expression(directory, &shell("default"));
            assert!(
                expression.contains(&format!("(\"path:\" + {literal})")),
                "{directory:?} did not reach the expression as one literal: {expression}"
            );
        }
    }

    /// A scratch project holding a locked manifest and a fake package manager.
    ///
    /// The manifest and its lock are the two things the adapter refuses without, and the fake `nix`
    /// is on the search path so the evaluation runs through the scripted runner rather than through
    /// this machine's Nix.
    fn a_locked_project(scratch: &Scratch) -> PathBuf {
        scratch.write("flake.nix", "{ outputs = { }: { }; }\n");
        scratch.write("flake.lock", "{ \"nodes\": { }, \"version\": 7 }\n");
        scratch.write("bin/nix", "#!/bin/sh\n");
        make_executable(&scratch.path().join("bin/nix"));
        scratch.write("bin/nix-store", "#!/bin/sh\n");
        make_executable(&scratch.path().join("bin/nix-store"));
        scratch.path().join("bin")
    }

    /// An evaluation answering with one store path, as the pinned response shape does.
    fn an_evaluation(input: &Path) -> String {
        format!(
            "{}\n",
            serde_json::to_string(&[input.display().to_string()]).expect("serializes")
        )
    }

    /// A toolchain whose tools resolve inside the declared inputs is measured as supplied.
    #[test]
    fn a_tool_inside_a_declared_input_is_supplied_by_that_environment() {
        let scratch = Scratch::new("toolchain");
        let search = a_locked_project(&scratch);
        // One realized input holding the tool, which is the shape a store path has.
        let input = scratch.directory("store-input/bin");
        scratch.write("store-input/bin/cargo", "#!/bin/sh\necho 1.90.0\n");
        make_executable(&input.join("cargo"));
        let input = scratch.path().join("store-input");

        let mut host = scripted(
            &scratch,
            &[search, input.join("bin")],
            ScriptedRunner::always(&an_evaluation(&input)),
        );
        match host
            .toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")])
            .expect("measured")
        {
            Fact::ToolchainTools {
                tools, manifest, ..
            } => {
                assert_eq!(manifest, path("flake.nix"));
                assert_eq!(tools.len(), 1, "{tools:?}");
                assert!(
                    tools[0].from_inputs,
                    "a tool inside the declared input was not attributed to it: {:?}",
                    tools[0]
                );
                assert_eq!(tools[0].input.as_deref(), Some(input.as_path()));
            }
            other => panic!("{other:?}"),
        }
        // One invocation, and it is the evaluation: no closure query was needed because the direct
        // comparison already explained the tool.
        let invoked = host.runner.invoked();
        assert_eq!(invoked.len(), 1, "{invoked:?}");
        assert!(
            invoked[0].starts_with("nix --extra-experimental-features"),
            "{invoked:?}"
        );
    }

    /// A tool inside a *dependency* of a declared input is still supplied by that environment.
    ///
    /// Measured in this repository rather than imagined: `jq` resolves to the `-bin` output of a
    /// derivation whose declared input is the `-dev` output, so a prefix comparison against the
    /// declared inputs alone would report a tool the pinned environment does supply as a shadowed
    /// one. The closure query is what closes that, and this asserts it runs only when needed.
    #[test]
    fn a_tool_in_an_inputs_closure_is_supplied_by_that_environment() {
        let scratch = Scratch::new("closure");
        let search = a_locked_project(&scratch);
        let declared = scratch.directory("store-input-dev");
        let sibling = scratch.directory("store-input-bin/bin");
        scratch.write("store-input-bin/bin/jq", "#!/bin/sh\n");
        make_executable(&sibling.join("jq"));
        let sibling = scratch.path().join("store-input-bin");

        let mut host = scripted(
            &scratch,
            &[search, sibling.join("bin")],
            ScriptedRunner::new(vec![
                RunResult::Exited {
                    status: 0,
                    stdout: an_evaluation(&declared),
                    stderr: String::new(),
                    truncated: false,
                },
                // The closure of the declared input holds the sibling output the tool resolves to.
                RunResult::Exited {
                    status: 0,
                    stdout: format!("{}\n{}\n", declared.display(), sibling.display()),
                    stderr: String::new(),
                    truncated: false,
                },
            ]),
        );
        match host
            .toolchain(&path("flake.nix"), &shell("default"), &[name("jq")])
            .expect("measured")
        {
            Fact::ToolchainTools { tools, .. } => {
                assert!(
                    tools[0].from_inputs,
                    "a tool inside the input's closure was reported as shadowing it: {:?}",
                    tools[0]
                );
            }
            other => panic!("{other:?}"),
        }
        let invoked = host.runner.invoked();
        assert_eq!(invoked.len(), 2, "{invoked:?}");
        assert!(
            invoked[1].starts_with("nix-store --query --requisites"),
            "the second invocation is not the closure query: {invoked:?}"
        );
    }

    /// A tool resolving to another installation is measured as differing, not as supplied.
    ///
    /// The shadowing case `runbook-portability` names: the pinned environment has the tool, and the
    /// invocation's search path finds a different one first.
    #[test]
    fn a_tool_shadowing_the_pinned_one_is_measured_as_differing() {
        let scratch = Scratch::new("shadow");
        let search = a_locked_project(&scratch);
        let input = scratch.directory("store-input");
        let other = scratch.directory("other-bin");
        scratch.write("other-bin/cargo", "#!/bin/sh\necho 1.80.0\n");
        make_executable(&other.join("cargo"));

        let mut host = scripted(
            &scratch,
            &[other.clone(), search],
            ScriptedRunner::new(vec![
                RunResult::Exited {
                    status: 0,
                    stdout: an_evaluation(&input),
                    stderr: String::new(),
                    truncated: false,
                },
                RunResult::Exited {
                    status: 0,
                    stdout: format!("{}\n", input.display()),
                    stderr: String::new(),
                    truncated: false,
                },
            ]),
        );
        match host
            .toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")])
            .expect("measured")
        {
            Fact::ToolchainTools { tools, .. } => {
                assert!(
                    !tools[0].from_inputs,
                    "a shadowing installation was attributed to the pinned environment: {:?}",
                    tools[0]
                );
                assert_eq!(
                    tools[0].resolved,
                    Some(other.join("cargo").canonicalize().expect("resolved")),
                    "the report would not be able to name where it resolved: {:?}",
                    tools[0]
                );
                assert_eq!(tools[0].input, None);
            }
            other => panic!("{other:?}"),
        }
    }

    /// A tool the environment supplies only through a shell hook is unavailable, not absent.
    ///
    /// The probe does not enter the shell, so it cannot see one. Reporting absence would state a
    /// falsehood about the environment, and reporting readiness would be worse.
    #[test]
    fn a_tool_only_a_shell_hook_supplies_is_unavailable() {
        let scratch = Scratch::new("hook-only");
        let search = a_locked_project(&scratch);
        let input = scratch.directory("store-input");
        let empty = scratch.directory("empty-bin");

        let mut host = scripted(
            &scratch,
            &[empty, search],
            ScriptedRunner::always(&an_evaluation(&input)),
        );
        match host.toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")]) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(reason.contains("do not resolve"), "{reason}");
                assert!(
                    reason.contains("does not enter"),
                    "the report does not say why nothing could be seen: {reason}"
                );
                assert!(
                    remedy.contains("inside"),
                    "the remedy does not say how to make the tool visible: {remedy}"
                );
            }
            other => panic!("a tool nothing resolves was reported as {other:?}"),
        }
    }

    /// The three refusals before anything is evaluated: no manifest, no lock, no package manager.
    ///
    /// Each is unavailable readiness with the prerequisite named, and each happens without starting
    /// a process — which is what "performs no provisioning or lock update" means in practice.
    #[test]
    fn a_toolchain_check_refuses_before_it_evaluates_anything() {
        let scratch = Scratch::new("refusals");
        let empty = scratch.directory("empty-bin");

        // No manifest at the declared path.
        let mut host = scripted(
            &scratch,
            std::slice::from_ref(&empty),
            ScriptedRunner::always("[]"),
        );
        match host.toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")]) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(reason.contains("no manifest"), "{reason}");
                assert!(
                    remedy.contains("target work environment"),
                    "the remedy does not say whose manifest it is: {remedy}"
                );
            }
            other => panic!("an absent manifest was reported as {other:?}"),
        }
        assert!(
            host.runner.invoked().is_empty(),
            "a process ran for an absent manifest: {:?}",
            host.runner.invoked()
        );

        // A manifest with no lock beside it.
        scratch.write("flake.nix", "{ outputs = { }: { }; }\n");
        let mut host = scripted(
            &scratch,
            std::slice::from_ref(&empty),
            ScriptedRunner::always("[]"),
        );
        match host.toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")]) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(reason.contains("flake.lock"), "{reason}");
                assert!(remedy.contains("commit"), "{remedy}");
            }
            other => panic!("an unlocked manifest was reported as {other:?}"),
        }
        assert!(
            host.runner.invoked().is_empty(),
            "a process ran for an unlocked manifest"
        );

        // A locked manifest, and no package manager on the search path.
        scratch.write("flake.lock", "{ \"nodes\": { }, \"version\": 7 }\n");
        let mut host = scripted(&scratch, &[empty], ScriptedRunner::always("[]"));
        match host.toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")]) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(reason.contains("nix"), "{reason}");
                assert!(
                    remedy.contains("install Nix"),
                    "the remedy does not say what would make the manifest readable: {remedy}"
                );
            }
            other => panic!("an absent package manager was reported as {other:?}"),
        }
        assert!(
            host.runner.invoked().is_empty(),
            "a process ran with no package manager to run it"
        );
    }

    /// Inputs that are not realized on this machine are unavailable readiness.
    ///
    /// "Ordinary evaluator caches are not a readiness guarantee": a store path the evaluator can
    /// name is not one the machine holds, and no tool's provenance can be established from it.
    #[test]
    fn unrealized_inputs_are_unavailable_rather_than_ready() {
        let scratch = Scratch::new("unrealized");
        let search = a_locked_project(&scratch);
        let mut host = scripted(
            &scratch,
            &[search],
            ScriptedRunner::always(
                "[\"/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-not-realized\"]",
            ),
        );

        match host.toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")]) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(reason.contains("realized"), "{reason}");
                assert!(
                    remedy.contains("builds and downloads nothing"),
                    "the remedy does not say what this probe will not do: {remedy}"
                );
            }
            other => panic!("an unrealized input was reported as {other:?}"),
        }
    }

    /// An evaluator that refuses is unavailable readiness carrying what it said.
    #[test]
    fn an_evaluation_that_fails_reports_what_the_evaluator_said() {
        let scratch = Scratch::new("evaluation-fails");
        let search = a_locked_project(&scratch);
        let mut host = scripted(
            &scratch,
            &[search],
            ScriptedRunner::new(vec![RunResult::Exited {
                status: 1,
                stdout: String::new(),
                stderr: String::from("error: cannot fetch input, offline"),
                truncated: false,
            }]),
        );

        match host.toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")]) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(reason.contains("cannot fetch input"), "{reason}");
                assert!(
                    remedy.contains("builds, substitutes, and updates nothing"),
                    "{remedy}"
                );
            }
            other => panic!("a failed evaluation was reported as {other:?}"),
        }
    }

    /// The trace Nix 2.34.6 writes before the error, when the adapter's expression fails.
    ///
    /// Recorded from the real-Nix tests' flake below, with the scratch directory replaced by
    /// `/work/project`, the column numbers moved to match, and each echoed source line and its
    /// caret shortened. Nix ends every frame's block with a blank line and writes the error last.
    const NIX_TRACE: &str = "error:
       … while calling the 'map' builtin
         at «string»:1:193:
            1| let flake = builtins.getFlake (\"path:\" + \"/work/project\"); …
             |                                                              ^

       … while calling the 'filter' builtin
         at «string»:1:230:
            1| let flake = builtins.getFlake (\"path:\" + \"/work/project\"); …
             |                                                                   ^

       (stack trace truncated; use '--show-trace' to show the full, detailed trace)

";

    /// What Nix said when the 1.3.1 adapter evaluated a shell importing from a derivation.
    ///
    /// The import's output was in the store, built minutes earlier, and the flag refused it anyway.
    fn refused_every_import() -> String {
        format!(
            "{NIX_TRACE}       error: cannot build \
             '/nix/store/7jqcih3zvw6s4j9hrkysfvqyn24lchw3-htmlspec-import-from-derivation-fixture.drv^out' \
             during evaluation because the option 'allow-import-from-derivation' is disabled\n"
        )
    }

    /// What Nix says when the adapter's settings decline to build an import the store lacks.
    fn declined_a_build() -> String {
        format!(
            "{NIX_TRACE}       error: Cannot build \
             '/nix/store/bvir5q2xy21ma6kaacjz6d56nna5pk74-htmlspec-import-from-derivation-fixture.drv'.
       Reason: local builds are disabled (max-jobs = 0)
       Hint: set 'max-jobs' to a non-zero value to enable local builds, or configure remote builders via 'builders'
"
        )
    }

    /// The same, for an import built for another platform: the reason it could not be built here
    /// follows the hint, on the error's fourth to sixth lines.
    fn declined_a_foreign_build() -> String {
        format!(
            "{NIX_TRACE}       error: Cannot build \
             '/nix/store/6al9zakaihh9bkmlwdx0hlq93bl0bcjx-htmlspec-import-from-derivation-fixture.drv'.
       Reason: local builds are disabled (max-jobs = 0)
       Hint: set 'max-jobs' to a non-zero value to enable local builds, or configure remote builders via 'builders'
       Reason: platform mismatch
       Required system: 'aarch64-linux'
       Current system: 'aarch64-darwin'
"
        )
    }

    /// What Nix says when the import's own dependency is what the store lacks, as after a garbage
    /// collection. The build it declined comes first, and the error the evaluation ends on names
    /// only a failed dependency.
    fn declined_a_dependency() -> String {
        format!(
            "error: Cannot build '/nix/store/ckd3pz6hja342hs02ssn5gi91mfxnbi2-htmlspec-import-dependency.drv'.
       Reason: local builds are disabled (max-jobs = 0)
       Hint: set 'max-jobs' to a non-zero value to enable local builds, or configure remote builders via 'builders'
error: Cannot build '/nix/store/hclaqqijdvbrm7jr6aydnr26yjzhadxj-htmlspec-import-from-derivation-fixture.drv'.
       Reason: 1 dependency failed.
       Output paths:
         /nix/store/1q75s9drr62y409768wiv7dgw0w9i5ab-htmlspec-import-from-derivation-fixture
{NIX_TRACE}       error: Build failed due to failed dependency
"
        )
    }

    /// What Nix 2.22.1, the release continuous integration's runner installs, says when the
    /// adapter's settings decline to build an import the store lacks, for this machine's system or
    /// another: a bare `error:` closing the trace, its message on the lines after it, and no
    /// derivation named. Recorded from that release's own binary.
    fn could_not_start_a_build_on_nix_2_22() -> String {
        format!(
            "warning: unknown setting 'lazy-trees'\n{NIX_TRACE}       error:
                               Unable to start any build;
                               either increase '--max-jobs' or enable remote builds.

                               For more information run 'man nix.conf' and search for '/machines'.
"
        )
    }

    /// What Nix says when the manifest defines no shell of the declared name.
    fn a_missing_shell() -> String {
        format!(
            "{NIX_TRACE}       error: attribute 'missing' missing
       at «string»:1:176:
            1| let flake = builtins.getFlake (\"path:\" + \"/work/project\"); …
             |                                                                          ^
"
        )
    }

    /// A failed evaluation quotes the error Nix stated, not the trace it began with.
    ///
    /// Measured on the defect: the 1.3.1 report read `error: / … while calling the 'map' builtin /
    /// at «string»:1:217:`, the trace's first three lines, and never reached the one that said why.
    /// The remedy names the case that remains once the import is allowed: an import whose output
    /// the store no longer holds. The request carries the child's proxy variables.
    #[test]
    fn a_failed_evaluation_quotes_the_error_rather_than_the_trace() {
        let scratch = Scratch::new("evaluation-trace");
        let search = a_locked_project(&scratch);
        let mut host = scripted(
            &scratch,
            &[search],
            ScriptedRunner::new(vec![RunResult::Exited {
                status: 1,
                stdout: String::new(),
                stderr: declined_a_build(),
                truncated: false,
            }]),
        );

        match host.toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")]) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(
                    reason.starts_with(
                        "the manifest could not be evaluated read-only: error: Cannot build \
                         '/nix/store/"
                    ),
                    "the reason does not start at the evaluator's error: {reason}"
                );
                assert!(
                    reason.contains("Reason: local builds are disabled (max-jobs = 0)"),
                    "{reason}"
                );
                assert!(
                    !reason.contains("while calling"),
                    "the reason quotes the trace: {reason}"
                );
                assert!(remedy.contains("imports from a derivation"), "{remedy}");
                assert!(
                    remedy.contains("builds, substitutes, and updates nothing"),
                    "{remedy}"
                );
            }
            other => panic!("a failed evaluation was reported as {other:?}"),
        }
        let requests = host.runner.requests.lock().expect("the requests").clone();
        let [evaluation] = requests.as_slice() else {
            panic!("one evaluation was expected: {requests:?}");
        };
        assert_eq!(evaluation.environment, EVALUATION_ENVIRONMENT);
    }

    /// The excerpt of a failed evaluation starts at the first error that states a cause.
    ///
    /// Pinned on what Nix 2.34.6 wrote for the 1.3.1 refusal, for an import the store lacks, for
    /// one built for another platform, for an import whose own dependency the store lacks, and for
    /// a shell the manifest does not define. [`excerpt`], which every other probe still quotes
    /// through, is asserted beside it on the first, because the difference between the two is the
    /// defect.
    #[test]
    fn the_evaluators_excerpt_starts_at_the_first_error_that_states_a_cause() {
        let refused = refused_every_import();
        assert_eq!(
            evaluator_excerpt(&refused),
            "error: cannot build \
             '/nix/store/7jqcih3zvw6s4j9hrkysfvqyn24lchw3-htmlspec-import-from-derivation-fixture.drv^out' \
             during evaluation because the option 'allow-import-from-derivation' is disabled"
        );
        assert_eq!(
            excerpt(&refused),
            "error: /        … while calling the 'map' builtin /          at «string»:1:193:",
            "the first lines are what 1.3.1 quoted, and what every other probe still quotes"
        );

        // One import missing: the error and its reason, without the hint about `max-jobs`, which
        // the adapter sets and the operator cannot.
        assert_eq!(
            evaluator_excerpt(&declined_a_build()),
            "error: Cannot build \
             '/nix/store/bvir5q2xy21ma6kaacjz6d56nna5pk74-htmlspec-import-from-derivation-fixture.drv'. \
             / Reason: local builds are disabled (max-jobs = 0)"
        );

        // Built for another platform: the reason it could not be built here, past the third line.
        assert_eq!(
            evaluator_excerpt(&declined_a_foreign_build()),
            "error: Cannot build \
             '/nix/store/6al9zakaihh9bkmlwdx0hlq93bl0bcjx-htmlspec-import-from-derivation-fixture.drv'. \
             / Reason: local builds are disabled (max-jobs = 0) / Reason: platform mismatch / Required \
             system: 'aarch64-linux' / Current system: 'aarch64-darwin'"
        );

        // The import's dependency missing: the build Nix declined, not the failed dependency it
        // ended on.
        let dependency = evaluator_excerpt(&declined_a_dependency());
        assert!(
            dependency.starts_with(
                "error: Cannot build \
                 '/nix/store/ckd3pz6hja342hs02ssn5gi91mfxnbi2-htmlspec-import-dependency.drv'. / \
                 Reason: local builds are disabled (max-jobs = 0) / error: Cannot build"
            ),
            "{dependency}"
        );
        assert!(
            !dependency.starts_with("error: Build failed due to failed dependency"),
            "{dependency}"
        );

        // Nix 2.22: the error's message on the lines after a bare `error:`, joined to it, and the
        // bare `error:` that opens the trace passed over.
        assert_eq!(
            evaluator_excerpt(&could_not_start_a_build_on_nix_2_22()),
            "error: Unable to start any build; / either increase '--max-jobs' or enable remote \
             builds. / For more information run 'man nix.conf' and search for '/machines'."
        );

        // A shell the manifest does not define: the missing attribute, and where.
        assert!(
            evaluator_excerpt(&a_missing_shell()).starts_with(
                "error: attribute 'missing' missing / at «string»:1:176: / 1| let flake"
            ),
            "{}",
            evaluator_excerpt(&a_missing_shell())
        );
    }

    /// The bare `error:` that opens a trace starts nothing, the first line that states one does,
    /// and the lines after it follow it, trimmed, to the end and within the bound alone.
    #[test]
    fn the_first_error_that_states_a_cause_starts_the_excerpt() {
        let output =
            "warning: Git tree is dirty\nerror:\n  … while evaluating\nerror: the first\n  \
                      its detail\n  more\n  and more\n       error: the last\n\n";
        assert_eq!(
            evaluator_excerpt(output),
            "error: the first / its detail / more / and more / error: the last"
        );
        assert_eq!(
            evaluator_excerpt("error:   \n  error: stated\n"),
            "error: stated"
        );

        let long = format!("trace\n   error: {}\n", "é".repeat(400));
        let quoted = evaluator_excerpt(&long);
        assert!(quoted.starts_with("error: ééé"), "{quoted}");
        assert_eq!(
            quoted.chars().count(),
            301,
            "the bound is 300 characters and an ellipsis, cut on a character's boundary"
        );
    }

    /// Output with no line opening with `error:` and stating something is quoted from its first
    /// lines, as any probe's is.
    #[test]
    fn an_evaluation_without_an_error_line_is_quoted_as_any_probe_is() {
        for output in [
            "",
            "   \n\n",
            "warning: unknown setting 'eval-cores'\nit failed\n  and said more\nand more\n",
            "a line with error: inside it opens no error\nsecond\n",
            "error:\n       … while calling the 'map' builtin\n       (stack trace truncated)\n",
        ] {
            assert_eq!(evaluator_excerpt(output), excerpt(output), "{output:?}");
        }
        assert_eq!(evaluator_excerpt(""), "nothing");
    }

    /// Coloured, the same refusals quote the same words.
    ///
    /// Nix 2.34.6 colours nothing it writes to a pipe, but a later Nix can be told to with
    /// `FORCE_COLOR` or `CLICOLOR_FORCE`, and then no line opens with `error:` as text. The
    /// colouring here is Nix's own: bold red for `error:`, and the escape that resets it.
    #[test]
    fn a_coloured_refusal_is_quoted_as_a_plain_one() {
        for plain in [
            refused_every_import(),
            declined_a_build(),
            declined_a_dependency(),
        ] {
            let coloured = plain
                .replace("error:", "\u{1b}[31;1merror:\u{1b}[0m")
                .replace("Reason:", "\u{1b}[35;1mReason:\u{1b}[0m")
                .replace("…", "\u{1b}]8;;file:///work\u{7}…\u{1b}]8;;\u{1b}\\");
            assert_ne!(coloured, plain);
            assert_eq!(evaluator_excerpt(&coloured), evaluator_excerpt(&plain));
        }
        assert_eq!(
            without_terminal_escapes("a\u{1b}[1"),
            "a",
            "a cut escape is dropped"
        );
        assert_eq!(without_terminal_escapes("a\u{1b}"), "a");
        assert_eq!(without_terminal_escapes("a\u{1b}(Bb"), "ab");
    }

    /// **An operator's own proxy variables do not reach the evaluation.**
    ///
    /// The review's finding: libcurl prefers the variable named for a URL's scheme, so an exported
    /// `ftp_proxy` beat the child's `all_proxy`, and an `ftp://` fetch reached the operator's proxy.
    /// From an inherited environment holding one of each kind, the evaluation's child keeps the six
    /// it sets, at the closed port, and nothing else named for a proxy; what is not a proxy's
    /// passes through untouched.
    #[test]
    fn an_operators_proxy_variables_do_not_reach_the_evaluation() {
        let operator = "http://proxy.operator.example:3128";
        let inherited: Vec<(std::ffi::OsString, std::ffi::OsString)> = [
            ("PATH", "/usr/bin:/bin"),
            ("HOME", "/home/operator"),
            ("ftp_proxy", operator),
            ("FTP_PROXY", operator),
            ("socks_proxy", operator),
            ("rsync_proxy", operator),
            ("http_proxy", operator),
            ("HTTPS_PROXY", operator),
            ("no_proxy", "*"),
            ("NO_PROXY", "127.0.0.1"),
            ("PROXY_PAC", "kept: it does not end in _proxy"),
            ("GIT_SSH_COMMAND", "ssh -J bastion.operator.example"),
        ]
        .into_iter()
        .map(|(name, value)| (name.into(), value.into()))
        .collect();

        let mut child = child_environment(inherited, EVALUATION_ENVIRONMENT);
        child.sort();
        let child: Vec<(String, String)> = child
            .into_iter()
            .map(|(name, value)| {
                (
                    name.to_string_lossy().into_owned(),
                    value.to_string_lossy().into_owned(),
                )
            })
            .collect();
        let closed = "http://127.0.0.1:1";
        let expected: Vec<(String, String)> = [
            ("ALL_PROXY", closed),
            ("GIT_PROXY_COMMAND", "false"),
            ("GIT_SSH_COMMAND", "false"),
            ("HOME", "/home/operator"),
            ("HTTPS_PROXY", closed),
            ("HTTP_PROXY", closed),
            ("PATH", "/usr/bin:/bin"),
            ("PROXY_PAC", "kept: it does not end in _proxy"),
            ("all_proxy", closed),
            ("http_proxy", closed),
            ("https_proxy", closed),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
        assert_eq!(child, expected);

        // With no changes, the child's environment is what was inherited.
        let plain = vec![("PATH".into(), "/bin".into())];
        assert_eq!(child_environment(plain.clone(), &[]), plain);
    }

    /// The child's environment changes as the request says, and this process's does not.
    #[test]
    fn a_probes_environment_reaches_its_child_alone() {
        let Some(sh) = real_executable("sh") else {
            panic!("this test needs a shell to read its environment");
        };
        // `HOME` is inherited, and a suffix removes it and one set earlier in the request alike.
        const CHANGES: &[EnvironmentChange] = &[
            EnvironmentChange::Set {
                name: "HTMLSPEC_PROBE_SET",
                value: "set for the child",
            },
            EnvironmentChange::Set {
                name: "HTMLSPEC_PROBE_HOME",
                value: "set, then removed by its ending",
            },
            EnvironmentChange::RemoveEveryEndingIn { suffix: "home" },
            EnvironmentChange::Set {
                name: "HTMLSPEC_PROBE_LATER_HOME",
                value: "set after the removal",
            },
        ];
        let result = SystemRunner.run(
            &ProbeRequest {
                program: sh,
                arguments: vec![
                    String::from("-c"),
                    String::from(
                        "echo \"${HTMLSPEC_PROBE_SET-unset}|${HOME-removed}|\
                         ${HTMLSPEC_PROBE_HOME-removed}|${HTMLSPEC_PROBE_LATER_HOME-unset}\"",
                    ),
                ],
                workdir: std::env::temp_dir(),
                environment: CHANGES,
            },
            &Cancellation::new(),
        );
        let RunResult::Exited {
            status: 0, stdout, ..
        } = result
        else {
            panic!("the shell did not run: {result:?}");
        };
        assert_eq!(
            stdout.trim(),
            "set for the child|removed|removed|set after the removal"
        );
        assert!(
            std::env::var_os("HTMLSPEC_PROBE_SET").is_none(),
            "the child's variable reached this process"
        );
    }

    /// A response that is not the pinned shape is unavailable readiness, not a parse of anything.
    ///
    /// The shape is pinned against the Nix this project supports; a manager that answers differently
    /// needs its own adapter rather than a looser parse, and a looser parse is how a check starts
    /// answering a question nobody asked.
    #[test]
    fn a_response_of_another_shape_is_unavailable() {
        let scratch = Scratch::new("response-shape");
        let search = a_locked_project(&scratch);

        for output in ["not json", "{\"inputs\":[]}", "3", "null"] {
            let mut host = scripted(
                &scratch,
                std::slice::from_ref(&search),
                ScriptedRunner::always(output),
            );
            let fault = host
                .toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")])
                .expect_err("a response of another shape was parsed");
            assert!(
                matches!(fault, ProbeFault::Unavailable { .. }),
                "{output:?} produced {fault:?}"
            );
        }
    }

    /// Duplicate inputs are one place a tool can come from rather than two.
    ///
    /// The dev shell in this repository lists its Rust toolchain five times, and a report naming the
    /// same closure five times would be five lines about one fact.
    #[test]
    fn a_repeated_input_is_one_place() {
        let scratch = Scratch::new("repeated-input");
        let search = a_locked_project(&scratch);
        let input = scratch.directory("store-input/bin");
        scratch.write("store-input/bin/cargo", "#!/bin/sh\n");
        make_executable(&input.join("cargo"));
        let input = scratch.path().join("store-input");

        let repeated = format!(
            "{}\n",
            serde_json::to_string(&[
                input.display().to_string(),
                input.display().to_string(),
                input.display().to_string(),
            ])
            .expect("serializes")
        );
        let mut host = scripted(
            &scratch,
            &[search, input.join("bin")],
            ScriptedRunner::always(&repeated),
        );
        match host
            .toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")])
            .expect("measured")
        {
            Fact::ToolchainTools { tools, .. } => {
                assert_eq!(tools.len(), 1, "{tools:?}");
                assert!(tools[0].from_inputs);
            }
            other => panic!("{other:?}"),
        }
    }

    /// A skill inside its scope is measured; one outside it is not, however it got there.
    #[test]
    fn a_skill_is_measured_inside_its_scope_only() {
        let scratch = Scratch::new("skill");
        let home = scratch.directory("home");
        let work = scratch.directory("work");
        let relative = ".agents/skills/example/SKILL.md";
        let declared = path(relative);
        scratch.write(
            &format!("home/{relative}"),
            "---\nname: example\n---\n\nDo the thing.\n",
        );

        let mut host = Host::new(
            work.clone(),
            Some(home.clone()),
            Vec::new(),
            ScriptedRunner::always(""),
        );
        match host.skill(&SkillScope::Home, &declared).expect("measured") {
            Fact::SkillInstalled { bytes, scope, .. } => {
                assert!(bytes > 0, "an empty file was measured as installed");
                assert_eq!(scope, SkillScope::Home);
            }
            other => panic!("{other:?}"),
        }

        // The same relative path in the other scope is absent, and there is no fallback: the file is
        // installed in `home` and this declaration says `workdir`.
        match host
            .skill(&SkillScope::Workdir, &declared)
            .expect("measured")
        {
            Fact::SkillAbsent { scope, .. } => assert_eq!(scope, SkillScope::Workdir),
            other => panic!("the workdir scope fell back to home: {other:?}"),
        }

        // A copy bundled beside the work does not satisfy a home-scope declaration.
        scratch.write(&format!("work/{relative}"), "---\nname: example\n---\n");
        match host.skill(&SkillScope::Home, &declared).expect("measured") {
            Fact::SkillInstalled { .. } => {}
            other => panic!("the bundled copy was what satisfied it: {other:?}"),
        }
    }

    /// A symlink from inside a scope to outside it does not satisfy the scope.
    ///
    /// Containment is checked on the canonicalized path, which is the only way to see this: a
    /// lexical comparison would report the link's own path, which is inside the scope.
    #[test]
    #[cfg(unix)]
    fn a_symlink_leaving_its_scope_does_not_satisfy_it() {
        let scratch = Scratch::new("skill-symlink");
        let home = scratch.directory("home");
        let work = scratch.directory("work");
        let outside = scratch.write("work/bundled/SKILL.md", "---\nname: bundled\n---\n");
        let inside = home.join(".agents/skills/escaping");
        std::fs::create_dir_all(&inside).expect("created");
        std::os::unix::fs::symlink(&outside, inside.join("SKILL.md")).expect("the link is made");

        let mut host = Host::new(
            work.clone(),
            Some(home.clone()),
            Vec::new(),
            ScriptedRunner::always(""),
        );
        let declared = path(".agents/skills/escaping/SKILL.md");
        match host.skill(&SkillScope::Home, &declared).expect("measured") {
            Fact::SkillAbsent { problem, .. } => {
                assert!(
                    problem.contains("outside"),
                    "a symlink leaving the scope was followed: {problem}"
                );
            }
            other => panic!("a symlink leaving the scope satisfied the check: {other:?}"),
        }
    }

    /// An empty skill file is absent, and a scope with no root is unavailable.
    #[test]
    fn an_empty_skill_and_a_nameless_scope_are_each_refused() {
        let scratch = Scratch::new("skill-edges");
        let home = scratch.directory("home");
        scratch.write("home/.agents/skills/empty/SKILL.md", "");
        let declared = path(".agents/skills/empty/SKILL.md");

        let mut host = Host::new(
            scratch.path().to_owned(),
            Some(home.clone()),
            Vec::new(),
            ScriptedRunner::always(""),
        );
        match host.skill(&SkillScope::Home, &declared).expect("measured") {
            Fact::SkillAbsent { problem, .. } => assert!(problem.contains("empty"), "{problem}"),
            other => panic!("an empty file was installed content: {other:?}"),
        }

        let mut homeless = Host::new(
            scratch.path().to_owned(),
            None,
            Vec::new(),
            ScriptedRunner::always(""),
        );
        match homeless.skill(&SkillScope::Home, &declared) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(reason.contains("home directory"), "{reason}");
                assert!(remedy.contains("HOME"), "{remedy}");
            }
            other => panic!("a scope with no root was resolved as {other:?}"),
        }
    }

    /// A directory that is not a file is not an installed skill.
    #[test]
    fn a_directory_where_a_skill_file_belongs_is_absent() {
        let scratch = Scratch::new("skill-directory");
        let home = scratch.directory("home");
        scratch.directory("home/.agents/skills/example/SKILL.md");
        let mut host = Host::new(
            home.clone(),
            Some(home),
            Vec::new(),
            ScriptedRunner::always(""),
        );
        match host
            .skill(&SkillScope::Home, &path(".agents/skills/example/SKILL.md"))
            .expect("measured")
        {
            Fact::SkillAbsent { problem, .. } => {
                assert!(problem.contains("not a file"), "{problem}")
            }
            other => panic!("a directory was installed content: {other:?}"),
        }
    }

    /// A working directory inside a project is measured as inside one.
    ///
    /// The bootstrap hazard, and the regression this project measured on itself: `init` bootstrapped
    /// a second project beside the first, and the failure arrived as nine duplicate-asset errors
    /// rather than as a refused precondition.
    #[test]
    fn a_working_directory_inside_a_project_is_measured_as_inside_one() {
        let scratch = Scratch::new("outside-project");
        let project = ProjectRoot::establish(scratch.path()).expect("established");
        let nested = project.changes();
        std::fs::create_dir_all(&nested).expect("created");

        let mut host = Host::new(nested.clone(), None, Vec::new(), ScriptedRunner::always(""));
        match host.state(&WorkingState::OutsideProject).expect("measured") {
            Fact::NearestProject { at } => assert_eq!(at, project.path()),
            other => panic!("a nested project was not found: {other:?}"),
        }

        // And outside one, which is the satisfying fact. A sibling repository is outside even
        // though the project is above it, because the boundary stops at `.git`.
        let elsewhere = scratch.directory("elsewhere");
        std::fs::create_dir_all(elsewhere.join(".git")).expect("a repository boundary");
        let mut host = Host::new(elsewhere, None, Vec::new(), ScriptedRunner::always(""));
        assert_eq!(
            host.state(&WorkingState::OutsideProject).expect("measured"),
            Fact::NoNearestProject
        );
    }

    /// A clean worktree, a dirty one, and an absent repository — three answers, not two.
    ///
    /// The absent case is the one that would otherwise be wrong in the dangerous direction: an
    /// unreported "clean" for a directory that is not a repository at all.
    #[test]
    fn a_worktree_is_clean_or_dirty_or_unreadable() {
        let scratch = Scratch::new("worktree");
        let search = scratch.executable("bin/git", "#!/bin/sh\n");
        let work = scratch.directory("work");

        let mut host = Host::new(
            work.clone(),
            Some(scratch.path().to_owned()),
            vec![search.clone()],
            ScriptedRunner::always(""),
        );
        assert_eq!(
            host.state(&WorkingState::CleanGitWorktree)
                .expect("measured"),
            Fact::WorktreeClean
        );
        // The fixed operation, with optional locks disabled: a status that took a lock would be a
        // write, and this probe modifies nothing.
        assert_eq!(
            host.runner.invoked(),
            vec![
                "git --no-optional-locks status --porcelain --untracked-files=all \
                 --ignore-submodules=none"
            ],
            "{:?}",
            host.runner.invoked()
        );
        assert_eq!(
            host.runner.requests.lock().expect("the requests")[0].workdir,
            work,
            "the worktree was read somewhere other than the working directory"
        );

        let mut host = Host::new(
            work.clone(),
            None,
            vec![search.clone()],
            ScriptedRunner::always(" M Cargo.toml\n?? new.rs\nUU conflicted.rs\nA  staged.rs\n"),
        );
        match host
            .state(&WorkingState::CleanGitWorktree)
            .expect("measured")
        {
            // Modified, staged, and untracked entries all make it false, which is what
            // `--porcelain` reports and what the design requires of the answer.
            Fact::WorktreeDirty { entries } => {
                // Modified, untracked, conflicted, and staged: all four make it false, which is
                // what the design requires and what `--porcelain` reports without being asked.
                assert_eq!(
                    entries,
                    vec![
                        " M Cargo.toml",
                        "?? new.rs",
                        "UU conflicted.rs",
                        "A  staged.rs"
                    ]
                );
            }
            other => panic!("{other:?}"),
        }

        let mut host = Host::new(
            work.clone(),
            None,
            vec![search],
            ScriptedRunner::new(vec![RunResult::Exited {
                status: 128,
                stdout: String::new(),
                stderr: String::from("fatal: not a git repository (or any parent)"),
                truncated: false,
            }]),
        );
        match host.state(&WorkingState::CleanGitWorktree) {
            Err(ProbeFault::Unavailable { reason, remedy }) => {
                assert!(reason.contains("not a git repository"), "{reason}");
                assert!(
                    remedy.contains("not a clean one"),
                    "the remedy does not state the distinction: {remedy}"
                );
            }
            other => panic!("an absent repository was reported as {other:?}"),
        }
    }

    /// A warning on stderr is not part of the answer a probe parses.
    ///
    /// Measured on the defect: the two streams were joined, so an unknown `nix.conf` setting made a
    /// working evaluation "not the JSON array of store paths", and a `git` advisory on a clean tree
    /// became a dirty entry. Driven through the real runner for the separation, and through each
    /// consumer for what it parses.
    #[test]
    fn a_warning_on_stderr_is_not_part_of_the_answer() {
        let Some(sh) = real_executable("sh") else {
            panic!("this test needs a shell to write to both streams");
        };
        let result = SystemRunner.run(
            &ProbeRequest {
                program: sh,
                arguments: vec![
                    String::from("-c"),
                    String::from("echo answer; echo 'warning: unknown setting' >&2"),
                ],
                workdir: std::env::temp_dir(),
                environment: &[],
            },
            &Cancellation::new(),
        );
        let RunResult::Exited {
            status: 0,
            stdout,
            stderr,
            ..
        } = result
        else {
            panic!("the child did not answer: {result:?}");
        };
        assert_eq!(stdout, "answer\n");
        assert_eq!(stderr, "warning: unknown setting\n");

        let warned = |answer: String| RunResult::Exited {
            status: 0,
            stdout: answer,
            stderr: String::from("warning: unknown setting 'eval-cores'\n"),
            truncated: false,
        };

        let scratch = Scratch::new("stderr-toolchain");
        let search = a_locked_project(&scratch);
        let input = scratch.directory("store-input/bin");
        scratch.write("store-input/bin/cargo", "#!/bin/sh\n");
        make_executable(&input.join("cargo"));
        let input = scratch.path().join("store-input");
        let mut host = scripted(
            &scratch,
            &[search, input.join("bin")],
            ScriptedRunner::new(vec![warned(an_evaluation(&input))]),
        );
        match host.toolchain(&path("flake.nix"), &shell("default"), &[name("cargo")]) {
            Ok(Fact::ToolchainTools { tools, .. }) => assert!(tools[0].from_inputs, "{tools:?}"),
            other => panic!("a warning made a working evaluation read as {other:?}"),
        }

        let scratch = Scratch::new("stderr-worktree");
        let search = scratch.executable("bin/git", "#!/bin/sh\n");
        let mut host = scripted(
            &scratch,
            &[search],
            ScriptedRunner::new(vec![warned(String::new())]),
        );
        assert_eq!(
            host.state(&WorkingState::CleanGitWorktree)
                .expect("measured"),
            Fact::WorktreeClean,
            "a warning made a clean worktree read as dirty"
        );
    }

    /// A tool that prints its version to stderr alone is still read.
    ///
    /// The one consumer that reads stderr as an answer, and only where stdout is empty — which is
    /// what keeping the streams apart must not break.
    #[test]
    fn a_version_printed_to_stderr_alone_is_read() {
        let scratch = Scratch::new("stderr-version");
        let search = scratch.executable("bin/old", "#!/bin/sh\n");
        let mut host = scripted(
            &scratch,
            &[search],
            ScriptedRunner::new(vec![RunResult::Exited {
                status: 0,
                stdout: String::new(),
                stderr: String::from("old version 2.4.1\n"),
                truncated: false,
            }]),
        );
        match host.tool(&name("old")).expect("measured") {
            Fact::ToolVersion { version, .. } => assert_eq!(version.to_string(), "2.4.1"),
            other => panic!("a version on stderr was reported as {other:?}"),
        }
    }

    /// An untracked file makes a worktree dirty whatever the repository's configuration says.
    ///
    /// Against a real `git`, because the defect was in what the operator's configuration did to the
    /// answer and a scripted runner has no configuration: with `status.showUntrackedFiles=no`, a tree
    /// holding an untracked file was reported clean. `gitMinimal` is in the check suite's sandbox.
    #[test]
    fn an_untracked_file_is_dirty_whatever_the_configuration_says() {
        let Some(git) = real_executable("git") else {
            panic!("this test needs git, which the check suite's sandbox supplies");
        };
        let scratch = Scratch::new("worktree-configured");
        let work = scratch.directory("work");
        let configure = |arguments: &[&str]| {
            let status = Command::new(&git)
                .args(arguments)
                .current_dir(&work)
                .stdout(Stdio::null())
                .status()
                .expect("git runs");
            assert!(status.success(), "git {arguments:?} failed");
        };
        configure(&["init", "--quiet"]);
        configure(&["config", "status.showUntrackedFiles", "no"]);
        scratch.write("work/untracked.rs", "fn main() {}\n");

        let search = git.parent().expect("git is in a directory").to_owned();
        let mut host = Host::new(work.clone(), None, vec![search], SystemRunner);
        match host
            .state(&WorkingState::CleanGitWorktree)
            .expect("measured")
        {
            Fact::WorktreeDirty { entries } => {
                assert_eq!(entries, vec!["?? untracked.rs"]);
            }
            other => panic!("configuration hid an untracked file: {other:?}"),
        }
    }

    /// A worktree holding more entries than a report should name is counted as well as named.
    #[test]
    fn a_dirty_worktree_is_bounded_in_what_it_names() {
        let scratch = Scratch::new("worktree-many");
        let search = scratch.executable("bin/git", "#!/bin/sh\n");
        let many = (0..50)
            .map(|index| format!("?? file-{index}.rs"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut host = scripted(&scratch, &[search], ScriptedRunner::always(&many));

        match host
            .state(&WorkingState::CleanGitWorktree)
            .expect("measured")
        {
            Fact::WorktreeDirty { entries } => {
                assert_eq!(
                    entries.len(),
                    DIRTY_ENTRIES_NAMED + 1,
                    "the entries are not bounded: {entries:?}"
                );
                assert!(
                    entries
                        .last()
                        .expect("a last entry")
                        .contains(&(50 - DIRTY_ENTRIES_NAMED).to_string()),
                    "the remainder is not counted: {entries:?}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// No `git` on the search path is unavailable readiness rather than a clean tree.
    #[test]
    fn a_worktree_that_cannot_be_read_is_unavailable() {
        let scratch = Scratch::new("no-git");
        let empty = scratch.directory("empty-bin");
        let mut host = scripted(&scratch, &[empty], ScriptedRunner::always(""));

        match host.state(&WorkingState::CleanGitWorktree) {
            Err(ProbeFault::Unavailable { reason, .. }) => {
                assert!(reason.contains("git"), "{reason}")
            }
            other => panic!("an unreadable worktree was reported as {other:?}"),
        }
        assert!(
            host.runner.invoked().is_empty(),
            "a process ran with no git to run"
        );
    }

    /// Cancellation before any probe returns no observations and starts no process.
    #[test]
    fn cancellation_between_probes_returns_no_observations() {
        let scratch = Scratch::new("cancelled");
        let contract = MachineContract::of(vec![
            declared(
                "one",
                MachineCheck::State {
                    check: WorkingState::OutsideProject,
                },
            ),
            declared(
                "two",
                MachineCheck::State {
                    check: WorkingState::CleanGitWorktree,
                },
            ),
        ])
        .expect("a contract");

        let mut host = Host::new(
            scratch.path().to_owned(),
            None,
            Vec::new(),
            ScriptedRunner::always(""),
        );
        host.cancellation().cancel();
        assert_eq!(
            observe_all(&contract, &mut host),
            Err(Interrupted),
            "a cancelled host produced observations"
        );
        assert!(
            host.runner.invoked().is_empty(),
            "a cancelled host started a process: {:?}",
            host.runner.invoked()
        );
    }

    /// Cancellation during a probe is seen when the runner returns it, and still returns nothing.
    ///
    /// The post-probe check is what this asserts: the runner answers a cancelled request, and the
    /// loop has to notice the flag rather than record the answer as an observation.
    #[test]
    fn cancellation_during_a_probe_returns_no_observations() {
        let scratch = Scratch::new("cancelled-midway");
        let search = scratch.executable("bin/git", "#!/bin/sh\n");
        let contract = MachineContract::of(vec![declared(
            "tree",
            MachineCheck::State {
                check: WorkingState::CleanGitWorktree,
            },
        )])
        .expect("a contract");

        let mut host = Host::new(
            scratch.path().to_owned(),
            None,
            vec![search],
            ScriptedRunner::new(vec![RunResult::Cancelled]),
        );
        host.cancellation().cancel();
        assert_eq!(observe_all(&contract, &mut host), Err(Interrupted));
    }

    /// Every declared check is observed, in document order, each carrying its own identity.
    #[test]
    fn every_declared_check_is_observed_in_document_order() {
        let scratch = Scratch::new("observe-all");
        scratch.write("skills/example/SKILL.md", "---\nname: example\n---\n");
        let contract = MachineContract::of(vec![
            declared(
                "first",
                MachineCheck::State {
                    check: WorkingState::OutsideProject,
                },
            ),
            declared(
                "second",
                MachineCheck::Skill {
                    scope: SkillScope::Workdir,
                    path: path("skills/example/SKILL.md"),
                },
            ),
        ])
        .expect("a contract");

        let mut host = Host::new(
            scratch.path().to_owned(),
            Some(scratch.path().to_owned()),
            Vec::new(),
            ScriptedRunner::always(""),
        );
        let observations = observe_all(&contract, &mut host).expect("observed");
        assert_eq!(
            observations
                .observed()
                .iter()
                .map(|observed| observed.requirement.to_string())
                .collect::<Vec<_>>(),
            vec!["req-first", "req-second"],
            "the observations are not in document order, or one is missing"
        );
        assert!(
            observations
                .observed()
                .iter()
                .all(|observed| matches!(observed.observation, Observation::Measured(_))),
            "{:?}",
            observations.observed()
        );
        assert_eq!(
            observations.observed()[0].requirement.as_str(),
            "req-first",
            "the observation lost the identity of the check it observed"
        );

        // The state probe runs no process, and the skill probe reads a file: neither reached the
        // runner, which is what keeps this test off the machine it runs on.
        assert!(
            host.runner.invoked().is_empty(),
            "a filesystem check started a process: {:?}",
            host.runner.invoked()
        );
    }

    /// A contract declaring nothing observes nothing, which is the zero-check case end to end.
    #[test]
    fn a_contract_declaring_nothing_observes_nothing() {
        let scratch = Scratch::new("no-checks");
        let contract = MachineContract::of(Vec::new()).expect("a contract");
        let mut host = Host::new(
            scratch.path().to_owned(),
            None,
            Vec::new(),
            ScriptedRunner::always(""),
        );
        let observations = observe_all(&contract, &mut host).expect("observed");
        assert!(observations.observed().is_empty());
        assert!(host.runner.invoked().is_empty());
    }

    // The real-Nix tests.
    //
    // Every other test here scripts Nix's answers, which is how the defect these two exist for went
    // unseen from 0.15.0 to 1.3.1: the answer the scripted tests pin came from this repository's own
    // flake, which imports from no derivation. These run this machine's Nix through the real runner
    // and the adapter's own arguments. The suite cannot run them, because it runs in the Nix build
    // sandbox, which has no daemon and no store to write, and on a runner with no Nix at all, so each
    // is ignored by name and says how to run it. A test that passed itself where Nix is missing would
    // be a skipped assertion that reads as a pass.

    /// A flake whose `default` shell imports a Nix file that one derivation writes.
    ///
    /// The shape of `callCabal2nix`, cut down to what the adapter reads: a build writes a Nix
    /// expression, and the shell is what importing it returns, so the shell cannot be evaluated until
    /// that build's output exists. The derivation needs nothing but `/bin/sh`'s own `echo`, which a
    /// Linux build sandbox also provides, and it carries a nonce, so its output is one this store has
    /// never held until a test builds it. The shell's one input is a directory holding an executable,
    /// copied into the store with `builtins.path`, because a sandboxed builder has no `mkdir` or
    /// `chmod` to make one and the repository tracks no executable file.
    ///
    /// `@SYSTEM@` is the Nix expression naming the platform the import is built for, and `@NONCE@`
    /// the nonce. It has no inputs, so its lock is the empty one.
    const IMPORTING_FLAKE: &str = r#"{
  outputs = { self }:
    let
      systems = [ "aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux" ];
      generated = system: derivation {
        name = "htmlspec-import-from-derivation-fixture";
        system = @SYSTEM@;
        builder = "/bin/sh";
        args = [ "-c" "echo '{ tool }: { nativeBuildInputs = [ tool ]; }' > $out" ];
        nonce = "@NONCE@";
      };
      tool = { outPath = builtins.path { path = ./tool; name = "htmlspec-fixture-tool"; }; };
      each = f: builtins.listToAttrs (map (system: { name = system; value = f system; }) systems);
    in
    {
      packages = each (system: { generated = generated system; });
      devShells = each (system: { default = import (generated system) { inherit tool; }; });
    };
}
"#;

    /// This machine's Nix, which the real-Nix tests run.
    fn real_nix() -> PathBuf {
        let Some(nix) = real_executable(NIX) else {
            panic!("this test runs this machine's Nix, and no `nix` resolves on the search path");
        };
        nix
    }

    /// The directory holding this machine's Nix, which is all a host's search path needs here.
    fn directory_of(nix: &Path) -> PathBuf {
        nix.parent()
            .expect("an executable lives in a directory")
            .to_owned()
    }

    /// A scratch flake whose shell imports from a derivation built for `system`, which no earlier
    /// run built: the nonce is this process and this moment.
    fn an_importing_flake(scratch: &Scratch, system: &str) {
        let since = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock reads after the epoch");
        let nonce = format!("{}-{}", std::process::id(), since.as_nanos());
        scratch.write(
            "flake.nix",
            &IMPORTING_FLAKE
                .replace("@SYSTEM@", system)
                .replace("@NONCE@", &nonce),
        );
        scratch.write(
            "flake.lock",
            "{\"nodes\":{\"root\":{}},\"root\":\"root\",\"version\":7}\n",
        );
        scratch.executable(
            "tool/bin/htmlspec-fixture-tool",
            "#!/bin/sh\necho htmlspec-fixture-tool 1.0.0\n",
        );
    }

    /// The real runner, as a real-Nix test runs it.
    ///
    /// Every request's environment gains, after the adapter's own changes, the test's own cache
    /// directory, so no Nix a test starts reads or writes the cache in this machine's home, and
    /// loses the library search path `cargo test` gives a test binary on macOS and Linux: under
    /// `DYLD_LIBRARY_PATH`, measured, this machine's Nix started its first download ten seconds
    /// late, past the probe's deadline. The installed binary runs with neither. `operator` comes
    /// before the adapter's changes, as variables the operator's own environment would hold.
    struct IsolatedRunner {
        operator: &'static [EnvironmentChange],
        isolation: &'static [EnvironmentChange],
    }

    impl IsolatedRunner {
        fn new(cache: &Path, operator: &'static [EnvironmentChange]) -> Self {
            let cache: &'static str = cache
                .to_str()
                .expect("a UTF-8 scratch path")
                .to_owned()
                .leak();
            Self {
                operator,
                isolation: vec![
                    EnvironmentChange::Set {
                        name: "XDG_CACHE_HOME",
                        value: cache,
                    },
                    // `DYLD_LIBRARY_PATH`, `LD_LIBRARY_PATH`, and their kin.
                    EnvironmentChange::RemoveEveryEndingIn {
                        suffix: "LIBRARY_PATH",
                    },
                ]
                .leak(),
            }
        }
    }

    impl CommandRunner for IsolatedRunner {
        fn run(&self, request: &ProbeRequest, cancellation: &Cancellation) -> RunResult {
            let environment = self
                .operator
                .iter()
                .chain(request.environment)
                .chain(self.isolation)
                .copied()
                .collect::<Vec<_>>()
                .leak();
            SystemRunner.run(
                &ProbeRequest {
                    environment,
                    ..request.clone()
                },
                cancellation,
            )
        }
    }

    /// What this machine's Nix answers, run from `directory` with `cache` as its cache. A refusal
    /// stops the test, because these calls set the store up rather than being what is tested.
    fn nix_answer(nix: &Path, directory: &Path, cache: &Path, arguments: &[&str]) -> String {
        let output = Command::new(nix)
            .args(["--extra-experimental-features", "nix-command flakes"])
            .args(arguments)
            .current_dir(directory)
            .env("XDG_CACHE_HOME", cache)
            .env_remove("DYLD_LIBRARY_PATH")
            .env_remove("LD_LIBRARY_PATH")
            .stdin(Stdio::null())
            .output()
            .expect("nix starts");
        assert!(
            output.status.success(),
            "`nix {}` failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    /// The store path the flake's import is built at, asked of Nix without building it.
    fn the_import_output(nix: &Path, directory: &Path, cache: &Path) -> PathBuf {
        PathBuf::from(nix_answer(
            nix,
            directory,
            cache,
            &[
                "eval",
                "--offline",
                "--no-write-lock-file",
                "--raw",
                &format!("path:{}#generated.outPath", directory.display()),
            ],
        ))
    }

    /// A system other than this machine's, which a remote builder may serve.
    fn another_system() -> &'static str {
        if cfg!(all(target_arch = "aarch64", target_os = "linux")) {
            "x86_64-linux"
        } else {
            "aarch64-linux"
        }
    }

    /// One requirement declaring the toolchain check a runbook would over the flake's shell.
    fn the_fixture_contract() -> MachineContract {
        MachineContract::of(vec![declared(
            "fixture-toolchain",
            MachineCheck::Toolchain {
                manifest: path("flake.nix"),
                shell: shell("default"),
                tools: vec![name("htmlspec-fixture-tool")],
            },
        )])
        .expect("a contract")
    }

    /// **A shell whose import from a derivation the store holds is read, and the check passes.**
    ///
    /// The defect: the 1.3.1 adapter refused this shell though its import was built. The import is
    /// built first, as entering the shell would build it, and then the adapter evaluates the shell
    /// from the store and the declared check is ready, its tool attributed to the shell's input.
    #[test]
    #[ignore = "runs this machine's Nix against its store; run with `cargo test -p htmlspec-cli --lib -- --ignored real_nix`"]
    fn real_nix_reads_an_import_from_a_derivation_the_store_holds() {
        let nix = real_nix();
        let scratch = Scratch::new("real-nix-held");
        let cache = Scratch::new("real-nix-held-cache");
        an_importing_flake(&scratch, "system");
        let flake = format!("path:{}#generated", scratch.path().display());
        nix_answer(
            &nix,
            scratch.path(),
            cache.path(),
            &[
                "build",
                "--offline",
                "--no-link",
                "--no-write-lock-file",
                &flake,
            ],
        );
        let output = the_import_output(&nix, scratch.path(), cache.path());
        assert!(output.is_file(), "the import was not built at {output:?}");

        // The evaluation alone, through the real runner: it reads the import and names the shell's
        // one input, which the store holds.
        let mut host = Host::new(
            scratch.path().to_owned(),
            None,
            vec![directory_of(&nix)],
            IsolatedRunner::new(cache.path(), &[]),
        );
        let inputs = match host.evaluate_inputs(&nix, scratch.path(), &shell("default")) {
            Ok(inputs) => inputs,
            Err(fault) => panic!("a shell whose import the store holds was refused: {fault:?}"),
        };
        let [input] = inputs.as_slice() else {
            panic!("the shell declares one input, and the evaluation named {inputs:?}");
        };
        assert!(
            input.join("bin/htmlspec-fixture-tool").is_file(),
            "the input named is not the realized tool directory: {input:?}"
        );

        // The declared check, end to end, with the tool resolving inside that input.
        let mut host = Host::new(
            scratch.path().to_owned(),
            None,
            vec![input.join("bin"), directory_of(&nix)],
            IsolatedRunner::new(cache.path(), &[]),
        );
        let contract = the_fixture_contract();
        let observations = observe_all(&contract, &mut host).expect("observed");
        let assessment = crate::preflight::assess(&contract, observations);
        assert!(
            assessment.is_ready(),
            "a shell whose import the store holds was not ready: {assessment:?}"
        );
    }

    /// **An import from a derivation the store lacks is refused, unbuilt, with Nix's own error.**
    ///
    /// Once for this machine's system, which only a local build slot could build, and once for
    /// another, which a remote builder could: on a machine with one configured, that case is what
    /// holds `--builders ''`. Each refusal is unavailable readiness, its reason starts at the error
    /// Nix ended on and names the derivation it declined to build, and the import's output is not in
    /// the store afterwards.
    #[test]
    #[ignore = "runs this machine's Nix against its store; run with `cargo test -p htmlspec-cli --lib -- --ignored real_nix`"]
    fn real_nix_refuses_an_import_from_a_derivation_the_store_lacks_without_building_it() {
        let nix = real_nix();
        let foreign = format!("\"{}\"", another_system());
        for (label, system) in [("native", "system"), ("foreign", foreign.as_str())] {
            let scratch = Scratch::new(&format!("real-nix-lacked-{label}"));
            let cache = Scratch::new(&format!("real-nix-lacked-{label}-cache"));
            an_importing_flake(&scratch, system);
            let output = the_import_output(&nix, scratch.path(), cache.path());
            assert!(
                !output.exists(),
                "{label}: the nonce did not make a fresh import, because {output:?} is in the store"
            );

            let mut host = Host::new(
                scratch.path().to_owned(),
                None,
                vec![directory_of(&nix)],
                IsolatedRunner::new(cache.path(), &[]),
            );
            let contract = the_fixture_contract();
            let observations = observe_all(&contract, &mut host).expect("observed");
            let crate::preflight::PreflightAssessment::NotReady(findings) =
                crate::preflight::assess(&contract, observations)
            else {
                panic!("{label}: a shell whose import the store lacks was ready");
            };
            let [finding] = findings.findings() else {
                panic!("{label}: one check was declared: {findings:?}");
            };
            assert_eq!(
                finding.outcome,
                crate::preflight::CheckOutcome::Unavailable,
                "{label}: {finding:?}"
            );
            assert!(
                finding
                    .observed
                    .starts_with("the manifest could not be evaluated read-only: error: "),
                "{label}: {}",
                finding.observed
            );
            // Nix 2.34 names the derivation it declined to build; Nix 2.22, which continuous
            // integration's runner installs, says it can start no build and names none.
            assert!(
                finding.observed.contains("Cannot build '/nix/store/")
                    && finding
                        .observed
                        .contains("htmlspec-import-from-derivation-fixture.drv")
                    || finding
                        .observed
                        .contains("error: Unable to start any build;"),
                "{label}: the report does not quote the build Nix declined: {}",
                finding.observed
            );
            assert!(
                !finding.observed.contains("while calling"),
                "{label}: the report quotes Nix's trace rather than its error: {}",
                finding.observed
            );
            assert!(
                !output.exists(),
                "{label}: the check built the import it was to refuse, at {output:?}"
            );
        }
    }

    /// **No configuration of the machine turns substitution back on.**
    ///
    /// The review's finding: `--offline` turns substitution off only where nothing configured it,
    /// and a `substitute` line in `nix.conf` or `NIX_CONFIG` counts as configured, so on such a
    /// machine the adapter downloaded an import's dependency from a binary cache and reported the
    /// shell ready. Asked of `nix config show` with exactly the adapter's settings, and
    /// `NIX_CONFIG='substitute = true'` set on that child alone: first without the settings, so a
    /// configuration that failed to reach the child cannot make this pass, then with them.
    #[test]
    #[ignore = "runs this machine's Nix against its store; run with `cargo test -p htmlspec-cli --lib -- --ignored real_nix`"]
    fn real_nix_substitutes_nothing_whatever_the_configuration_says() {
        let nix = real_nix();
        let substitute = |settings: &[String]| -> String {
            let output = Command::new(&nix)
                .args(["--extra-experimental-features", "nix-command flakes"])
                .args(["config", "show", "substitute"])
                .args(settings)
                .env("NIX_CONFIG", "substitute = true")
                .stdin(Stdio::null())
                .output()
                .expect("nix starts");
            assert!(
                output.status.success(),
                "`nix config show substitute` failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        };
        assert_eq!(
            substitute(&[]),
            "true",
            "the configuration did not reach the child, so the next assertion would prove nothing"
        );
        assert_eq!(
            substitute(&evaluation_settings()),
            "false",
            "the adapter's settings leave substitution on where the configuration turns it on"
        );
    }

    /// A flake whose `default` shell's one input is `@SOURCE@`, a Nix expression that fetches, and
    /// whose one flake input, where `@INPUTS@` declares one, is `missing`.
    const FETCHING_FLAKE: &str = r#"{
  @INPUTS@
  outputs = { self, ... }@inputs:
    let
      systems = [ "aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux" ];
      each = f: builtins.listToAttrs (map (system: { name = system; value = f system; }) systems);
    in
    {
      devShells = each (system: {
        default = { nativeBuildInputs = [ { outPath = @SOURCE@; } ]; };
      });
    };
}
"#;

    /// A lock holding one input, `missing`, locked as `locked`, which no store or cache holds.
    fn a_lock_holding(locked: &str, original: &str) -> String {
        format!(
            "{{\"nodes\":{{\"missing\":{{\"flake\":false,\"locked\":{locked},\"original\":{original}}},\
             \"root\":{{\"inputs\":{{\"missing\":\"missing\"}}}}}},\"root\":\"root\",\"version\":7}}\n"
        )
    }

    /// **Nothing the evaluation fetches leaves the machine, by any route the environment closes.**
    ///
    /// The review's findings, older than 1.3.2: under every release's settings, a shell's
    /// `builtins.fetchurl` and `builtins.fetchTarball`, a missing locked flake input, `git://` and
    /// SSH fetches, and an `ftp://` fetch through an operator's own `ftp_proxy` all reached the
    /// network. A listener on the loopback interface stands in for the network and for that
    /// operator's proxy, which the runner sets before the adapter's changes as the operator's own
    /// environment would hold it. Each case is refused within five seconds, its reason starting at
    /// Nix's error, and no connection reaches the listener: libcurl and `git` meet the closed local
    /// port, the operator's `ftp_proxy` is removed, and `git` opens no `git://` or SSH connection.
    /// `git+https` is the case a closed port exists for: `git` reads the proxy and not
    /// `connect-timeout`, so behind an address nothing answered it waited out the deadline.
    #[test]
    #[ignore = "runs this machine's Nix against its store; run with `cargo test -p htmlspec-cli --lib -- --ignored real_nix`"]
    fn real_nix_fetches_nothing_a_shell_asks_for() {
        let nix = real_nix();
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback listener binds");
        let port = listener
            .local_addr()
            .expect("the listener's address")
            .port();
        listener
            .set_nonblocking(true)
            .expect("the listener stops blocking");
        let arrived = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let watching = {
            let arrived = Arc::clone(&arrived);
            let stop = Arc::clone(&stop);
            std::thread::spawn(move || {
                while !stop.load(Ordering::SeqCst) {
                    match listener.accept() {
                        Ok(_) => arrived.store(true, Ordering::SeqCst),
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Err(_) => break,
                    }
                }
            })
        };
        let at = format!("127.0.0.1:{port}");
        let operator: &'static [EnvironmentChange] = vec![EnvironmentChange::Set {
            name: "ftp_proxy",
            value: format!("http://{at}").leak(),
        }]
        .leak();
        let nar_hash = format!("sha256-{}=", "A".repeat(43));
        let missing =
            |url: &str| format!("inputs.missing = {{ url = \"{url}\"; flake = false; }};");

        // (label, flake inputs, the shell's input, lock)
        let cases: Vec<(&str, String, String, Option<String>)> = vec![
            (
                "fetchurl",
                String::new(),
                format!("builtins.fetchurl \"http://{at}/htmlspec-fetchurl\""),
                None,
            ),
            (
                "fetchTarball",
                String::new(),
                format!("builtins.fetchTarball \"http://{at}/htmlspec-fetchTarball.tar.gz\""),
                None,
            ),
            (
                "ftp through the operator's ftp_proxy",
                String::new(),
                format!("builtins.fetchurl \"ftp://{at}/htmlspec-ftp\""),
                None,
            ),
            (
                "fetchGit over git://",
                String::new(),
                format!("builtins.fetchGit {{ url = \"git://{at}/htmlspec-git\"; }}"),
                None,
            ),
            (
                "fetchGit over SSH",
                String::new(),
                format!("builtins.fetchGit {{ url = \"ssh://git@{at}/htmlspec-ssh\"; }}"),
                None,
            ),
            (
                "a missing locked tarball input",
                missing(&format!("http://{at}/htmlspec-missing.tar.gz")),
                String::from("\"${inputs.missing}\""),
                Some(a_lock_holding(
                    &format!(
                        "{{\"narHash\":\"{nar_hash}\",\"type\":\"tarball\",\
                         \"url\":\"http://{at}/htmlspec-missing.tar.gz\"}}"
                    ),
                    &format!(
                        "{{\"type\":\"tarball\",\"url\":\"http://{at}/htmlspec-missing.tar.gz\"}}"
                    ),
                )),
            ),
            (
                "a missing locked git+https input",
                missing(&format!("git+https://{at}/htmlspec-missing.git")),
                String::from("\"${inputs.missing}\""),
                Some(a_lock_holding(
                    &format!(
                        "{{\"lastModified\":1,\"narHash\":\"{nar_hash}\",\"rev\":\"{}\",\
                         \"revCount\":1,\"type\":\"git\",\
                         \"url\":\"https://{at}/htmlspec-missing.git\"}}",
                        "1".repeat(40)
                    ),
                    &format!("{{\"type\":\"git\",\"url\":\"https://{at}/htmlspec-missing.git\"}}"),
                )),
            ),
        ];

        for (label, inputs, source, lock) in cases {
            let slug: String = label
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect();
            let scratch = Scratch::new(&format!("real-nix-fetch-{slug}"));
            let cache = Scratch::new(&format!("real-nix-fetch-{slug}-cache"));
            scratch.write(
                "flake.nix",
                &FETCHING_FLAKE
                    .replace("@INPUTS@", &inputs)
                    .replace("@SOURCE@", &source),
            );
            scratch.write(
                "flake.lock",
                &lock.unwrap_or_else(|| {
                    String::from("{\"nodes\":{\"root\":{}},\"root\":\"root\",\"version\":7}\n")
                }),
            );
            let mut host = Host::new(
                scratch.path().to_owned(),
                None,
                vec![directory_of(&nix)],
                IsolatedRunner::new(cache.path(), operator),
            );
            let started = Instant::now();
            let result = host.evaluate_inputs(&nix, scratch.path(), &shell("default"));
            let took = started.elapsed();
            match result {
                Err(ProbeFault::Unavailable { reason, .. }) => {
                    assert!(
                        reason
                            .starts_with("the manifest could not be evaluated read-only: error: "),
                        "{label}: {reason}"
                    );
                    // A failed fetch, not some other refusal: Nix 2.34 names the URL it could not
                    // fetch, and Nix 2.22, which continuous integration's runner installs, ends a
                    // failed `fetchGit` on the reference it then could not resolve.
                    assert!(
                        reason.contains(&at) || reason.contains("error: resolving Git reference"),
                        "{label}: the refusal is not the failed fetch: {reason}"
                    );
                }
                other => panic!("{label}: a shell that fetches was reported as {other:?}"),
            }
            assert!(
                took < Duration::from_secs(5),
                "{label}: the refusal took {took:?}"
            );
        }

        stop.store(true, Ordering::SeqCst);
        watching.join().expect("the listener's thread ends");
        assert!(
            !arrived.load(Ordering::SeqCst),
            "a request the check made reached the listener"
        );
    }

    /// A declared check paired with the requirement that declared it.
    fn declared(id: &str, check: MachineCheck) -> DeclaredCheck {
        DeclaredCheck {
            requirement: RequirementId::new(&format!("req-{id}")).expect("an identifier"),
            check,
            remedy: None,
        }
    }
}

#[cfg(test)]
mod pinned {
    //! The adapter's response shape, pinned against the Nix this project supports.
    //!
    //! The design requires the invocation *and* the parsed response to be pinned in tests, and this
    //! is the response half. The bytes below are what `nix eval` answered for this repository's own
    //! `flake.nix` — Nix 2.34.6, the version `flake.lock` pins through `nixpkgs` — with the store
    //! paths left exactly as they came back. Recording a real answer is the point: a hand-written
    //! sample would only prove the adapter can parse what the author of the sample expected.

    use super::*;

    /// One real answer: five entries, four of them the same Rust toolchain.
    ///
    /// The repetition is not padding. This repository's dev shell lists its toolchain as several
    /// inputs that resolve to one derivation, and an adapter that kept all five would report the
    /// same closure five times — five lines about one fact, in a report an operator reads.
    const REAL_EVALUATION: &str = r#"["/nix/store/33m8ldxvjsvyk07lcwz3wn3yyzqni9pj-rust-minimal-1.90.0","/nix/store/33m8ldxvjsvyk07lcwz3wn3yyzqni9pj-rust-minimal-1.90.0","/nix/store/33m8ldxvjsvyk07lcwz3wn3yyzqni9pj-rust-minimal-1.90.0","/nix/store/33m8ldxvjsvyk07lcwz3wn3yyzqni9pj-rust-minimal-1.90.0","/nix/store/33m8ldxvjsvyk07lcwz3wn3yyzqni9pj-rust-minimal-1.90.0","/nix/store/dy66gn3kmw5s0ph65yvx1750x6yqg0w2-cargo-nextest-0.9.136","/nix/store/hc2cqly37cgl5fyc4w4dz386z97p0f43-jq-1.8.2-dev"]"#;

    /// A temporary directory that removes itself.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-pinned-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            Self(path.canonicalize().expect("the scratch path resolves"))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The real answer parses into distinct store paths, in order.
    ///
    /// **This is the pin, and it needs nothing from this machine.** The bytes above came from a real
    /// evaluator; whether the store paths in them are realized here is a different question, asked by
    /// `unrealized_inputs_are_unavailable_rather_than_ready` and `a_realized_input_attributes_the_
    /// tools_it_supplies` against directories a test creates. A test that reached into `/nix/store`
    /// would pass on a contributor's machine and fail on a clean one.
    #[test]
    fn the_recorded_answer_parses_into_distinct_store_paths() {
        let inputs = evaluation_inputs(REAL_EVALUATION).expect("the recorded answer parses");
        assert_eq!(
            inputs,
            vec![
                PathBuf::from("/nix/store/33m8ldxvjsvyk07lcwz3wn3yyzqni9pj-rust-minimal-1.90.0"),
                PathBuf::from("/nix/store/dy66gn3kmw5s0ph65yvx1750x6yqg0w2-cargo-nextest-0.9.136"),
                PathBuf::from("/nix/store/hc2cqly37cgl5fyc4w4dz386z97p0f43-jq-1.8.2-dev"),
            ],
            "the five repetitions of one toolchain did not collapse to it, or the order moved"
        );

        // Whitespace around the answer is not significant, because a evaluator's last newline is not
        // a fact about the environment.
        assert_eq!(
            evaluation_inputs(&format!("  {REAL_EVALUATION}\n\n")).expect("parses"),
            inputs
        );
    }

    /// An answer of any other shape is refused rather than partially read.
    ///
    /// Every case here is one a future package-manager version could plausibly produce, and each is
    /// unavailable readiness: an adapter that read what it could would report a tool as unpinned
    /// because it failed to see the input that pins it.
    #[test]
    fn an_answer_of_another_shape_is_refused_whole() {
        for (output, wanted) in [
            ("not json", "not the JSON array"),
            ("{}", "rather than an array"),
            ("3", "rather than an array"),
            ("null", "rather than an array"),
            (
                "[\"/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-one\", 3]",
                "rather than a store path",
            ),
            ("[\"relative/store/path\"]", "is not an absolute store path"),
        ] {
            let fault = evaluation_inputs(output).expect_err("accepted");
            match fault {
                ProbeFault::Unavailable { reason, remedy } => {
                    assert!(reason.contains(wanted), "{output:?} reported {reason}");
                    assert!(
                        remedy.contains("pinned against one response shape"),
                        "the remedy does not say why a looser parse is not the fix: {remedy}"
                    );
                }
                other => panic!("{output:?} produced {other:?}"),
            }
        }

        // An empty environment is a parsed answer and not a malformed one: no inputs means nothing is
        // realized, which the caller reports as unavailable readiness.
        assert!(evaluation_inputs("[]").expect("parses").is_empty());
    }

    /// A realized input attributes the tools it supplies.
    ///
    /// Stands in for the machine that has built the flake, using a directory shaped like a store
    /// path: what the adapter compares is a prefix, so a realized input and a fabricated one are the
    /// same question.
    #[test]
    fn a_realized_input_attributes_the_tools_it_supplies() {
        let scratch = Scratch::new("realized");
        let store = scratch.path().join("store");
        let rust = store.join("33m8ldxvjsvyk07lcwz3wn3yyzqni9pj-rust-minimal-1.90.0");
        std::fs::create_dir_all(rust.join("bin")).expect("created");
        std::fs::write(rust.join("bin/cargo"), "#!/bin/sh\n").expect("written");
        std::fs::write(rust.join("bin/rustc"), "#!/bin/sh\n").expect("written");
        #[cfg(unix)]
        for name in ["cargo", "rustc"] {
            use std::os::unix::fs::PermissionsExt;
            let path = rust.join("bin").join(name);
            let mut permissions = std::fs::metadata(&path).expect("readable").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&path, permissions).expect("the mode is set");
        }

        std::fs::write(
            scratch.path().join("flake.nix"),
            "{ outputs = { }: { }; }\n",
        )
        .expect("written");
        std::fs::write(
            scratch.path().join("flake.lock"),
            "{ \"nodes\": { }, \"version\": 7 }\n",
        )
        .expect("written");
        std::fs::create_dir_all(scratch.path().join("bin")).expect("created");
        for program in ["nix", "nix-store"] {
            let path = scratch.path().join("bin").join(program);
            std::fs::write(&path, "#!/bin/sh\n").expect("written");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut permissions = std::fs::metadata(&path).expect("readable").permissions();
                permissions.set_mode(0o755);
                std::fs::set_permissions(&path, permissions).expect("the mode is set");
            }
        }

        let evaluation = format!(
            "{}\n",
            serde_json::to_string(&[rust.display().to_string()]).expect("serializes")
        );
        let mut host = Host::new(
            scratch.path().to_owned(),
            None,
            vec![rust.join("bin"), scratch.path().join("bin")],
            ScriptedRunner::always(&evaluation),
        );
        match host.toolchain(
            &ScopedRelativePath::new("flake.nix").expect("a path"),
            &ShellName::new("default").expect("a name"),
            &[
                ProgramName::new("cargo").expect("a name"),
                ProgramName::new("rustc").expect("a name"),
            ],
        ) {
            Ok(Fact::ToolchainTools { tools, .. }) => {
                assert_eq!(tools.len(), 2, "{tools:?}");
                assert!(
                    tools.iter().all(|tool| tool.from_inputs),
                    "a tool the pinned environment supplies was not attributed to it: {tools:?}"
                );
                assert!(
                    tools
                        .iter()
                        .all(|tool| tool.input.as_deref() == Some(rust.as_path())),
                    "{tools:?}"
                );
            }
            other => panic!("realized inputs were reported as {other:?}"),
        }
    }
}
