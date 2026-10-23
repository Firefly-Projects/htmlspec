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
                    if started.elapsed() > DEADLINE {
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
        }
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
    /// The flags are the requirement rather than a preference: `--offline` so no fetch can happen,
    /// `--no-write-lock-file` so the lock cannot change, `--no-allow-import-from-derivation` so an
    /// input that would need a build to be understood is refused rather than built, and
    /// `--expr` with no `develop`, `build`, or `shell` anywhere in the invocation. A test asserts
    /// the assembled argument vector, so this is a property of what the program sends.
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
            // Whatever the evaluator said is the reason. Missing locked sources, an evaluation that
            // would need a build, a shell the manifest does not define, and an unsupported Nix all
            // arrive here, and they are all unavailable readiness: this probe will not provision
            // its way to an answer.
            return Err(ProbeFault::Unavailable {
                reason: format!(
                    "the manifest could not be evaluated read-only: {}",
                    excerpt(&said(&stdout, &stderr))
                ),
                remedy: String::from(
                    "make the environment inspectable from what this machine already holds — its \
                     locked inputs in the store and the shell the declaration names in the \
                     manifest. This probe fetches, builds, and updates nothing",
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

/// The exact argument vector the adapter sends, as one function so a test can assert all of it.
///
/// Every flag here is a promise the design makes about what preflight does not do, which is why
/// they are assembled in one readable place rather than scattered through a builder chain.
pub fn evaluation_arguments(directory: &str, shell: &ShellName) -> Vec<String> {
    vec![
        String::from("--extra-experimental-features"),
        String::from("nix-command flakes"),
        String::from("eval"),
        // No network, so a missing input is a refusal rather than a download.
        String::from("--offline"),
        // The lock is read, never moved: a check that changed what it was checking would leave the
        // tree it promised not to modify.
        String::from("--no-write-lock-file"),
        // An input that can only be understood by building it is not understood. Refused rather
        // than built.
        String::from("--no-allow-import-from-derivation"),
        // `builtins.currentSystem` is a fact about this machine, and reading it is what makes the
        // expression impure.
        String::from("--impure"),
        String::from("--json"),
        String::from("--expr"),
        evaluation_expression(directory, shell),
    ]
}

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
    let mut text = taken.join(" / ");
    if text.chars().count() > 300 {
        let truncated: String = text.chars().take(300).collect();
        text = format!("{truncated}…");
    }
    if text.is_empty() {
        return String::from("nothing");
    }
    text
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

    /// **A real child process is stopped at the deadline and reaped.**
    ///
    /// The one test here that runs a program, and it exists because a scripted timeout proves the
    /// mapping from `TimedOut` to unavailable readiness and nothing about termination. The assertion
    /// is that the call returns, as a timeout, well inside the child's own thirty seconds.
    #[test]
    fn a_real_child_process_is_stopped_at_the_deadline() {
        let Some(sleep) = real_executable("sleep") else {
            // No `sleep` means no way to start a child that will not finish. Named rather than
            // passed over silently, because a skipped assertion is not an assertion.
            panic!("this test needs a `sleep` executable to start a process that will not finish");
        };
        let started = Instant::now();
        let result = SystemRunner.run(
            &ProbeRequest {
                program: sleep,
                arguments: vec![String::from("30")],
                workdir: std::env::temp_dir(),
            },
            &Cancellation::new(),
        );
        let elapsed = started.elapsed();

        assert_eq!(result, RunResult::TimedOut, "the child was not stopped");
        assert!(
            elapsed < Duration::from_secs(30),
            "the probe waited for the child rather than stopping it: {elapsed:?}"
        );
        assert!(
            elapsed >= DEADLINE,
            "the probe returned before its deadline, so the deadline was not what stopped it: \
             {elapsed:?}"
        );
    }

    /// **A real cancellation stops a running child before the call returns.**
    ///
    /// The other half of the transition the design specifies: "Cancellation stops and reaps
    /// outstanding processes … before returning." Asserted by raising the signal from another thread
    /// while the child runs, and requiring the call to come back as cancelled rather than as a
    /// timeout — which it can only do by observing the flag mid-flight.
    #[test]
    fn a_real_cancellation_stops_a_running_child() {
        let Some(sleep) = real_executable("sleep") else {
            panic!("this test needs a `sleep` executable to start a process that will not finish");
        };
        let cancellation = Cancellation::new();
        let signaller = cancellation.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            signaller.cancel();
        });
        let started = Instant::now();
        let result = SystemRunner.run(
            &ProbeRequest {
                program: sleep,
                arguments: vec![String::from("30")],
                workdir: std::env::temp_dir(),
            },
            &cancellation,
        );
        assert_eq!(result, RunResult::Cancelled, "the child was not stopped");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "cancellation did not reach the running probe"
        );
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

        for wanted in [
            "--offline",
            "--no-write-lock-file",
            "--no-allow-import-from-derivation",
            "--json",
        ] {
            assert!(
                arguments.iter().any(|argument| argument == wanted),
                "the invocation does not carry {wanted}: {joined}"
            );
        }
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

        // The expression reads both input lists and asks for store paths, which is what makes the
        // answer about the environment rather than about one attribute of it.
        let expression = arguments.last().expect("an expression");
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
                    remedy.contains("fetches, builds, and updates nothing"),
                    "{remedy}"
                );
            }
            other => panic!("a failed evaluation was reported as {other:?}"),
        }
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
