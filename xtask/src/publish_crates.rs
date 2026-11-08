//! `cargo xtask publish-crates`: publish the workspace's crates to crates.io, one at a time,
//! finishing what a run before left.
//!
//! **Why one at a time, and why a rerun finishes.** crates.io lets an account publish a burst of
//! five new crates and then refills slowly, so the sixth new crate of a first release is answered
//! HTTP 429, "You have published too many new crates in a short period of time". `cargo publish
//! --workspace` cannot finish that release: rerun, it checks every package first and stops at the
//! first one that already exists, and cargo has no flag to skip existing versions. So this publishes
//! each crate with `cargo publish -p`, in the order [`Workspace::publishing_order`] derives, and
//! before each asks the registry's sparse index whether that version is already there, skipping it
//! when it is. When crates.io answers 429, it waits until the time the answer names and tries
//! again, and when the next step would pass its budget it stops and names the crates that remain. A
//! first release therefore publishes in two waves, and a run stopped anywhere is finished by running
//! it again. The limit binds new crates only, so a later release publishes in one wave.
//!
//! **The budget** is time on the [`Clock`]'s monotonic clock from the moment the run begins, 30
//! minutes unless told otherwise and never more than [`MAXIMUM_BUDGET`], and everything spends it:
//! each `cargo publish`, which builds the crate before it uploads it, each question to the index,
//! and each wait. So no wait is longer than the budget. A wait is never shorter than ten seconds,
//! even when the time crates.io names has already passed, so a run never asks again at once.
//! Whether a crate is published is the registry's to say: the index, asked before each attempt and
//! again after one that failed, since cargo can fail after the upload landed.
//!
//! **What it sends.** Every request carries one fixed User-Agent, from the release's facts, which
//! names the project and its public repository and nothing else: no account, no address, no host
//! name. A review's probe once sent a contributor's email address to the crates.io API in that
//! header, and a release must never repeat it. cargo's own requests carry it through
//! `CARGO_HTTP_USER_AGENT`. cargo reads the token from `CARGO_REGISTRY_TOKEN`, and this never
//! reads it.
//!
//! **Where it refuses to run.** A crate records the repository it came from, and crates.io never
//! forgets a published version, so this publishes only a workspace whose `repository` is the public
//! one, and refuses any other before it sends a request. It refuses too a workspace one of whose
//! published crates depends on a member marked `publish = false`, which crates.io would refuse after
//! the crates before it were published.
//!
//! **`--plan`** reads the index the same way and says what a run would publish, in what order, and
//! what it would skip, and publishes nothing: [`plan`] is given no [`Cargo`] to publish with. It
//! runs on a workspace naming any repository, saying so, so a pull request in the private working
//! repository can read the plan its release would follow.
//!
//! The core, [`publish`] and [`plan`], takes the clock, the index, and cargo as capabilities, so
//! every test runs it on a virtual clock that waits no real time. What a run prints and how it exits
//! is [`Closing`], a value too, so the exit codes and their messages are tested. The edge, [`run`],
//! reads the workspace through `cargo metadata`, passes the real capabilities, and prints.

mod cargo;
mod clock;
mod registry;
mod retry;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt;
use std::process::ExitCode;
use std::time::Duration;

use cargo::{Cargo, CargoAnswer, CargoFailure, RealCargo, Verification};
use clock::{Clock, SystemClock};
use registry::{CurlRegistry, IndexAnswer, IndexRefusal, Proxies, Registry, CRATES_IO_INDEX};
use retry::Requested;

use crate::facts::{Facts, PublicRepository};
use crate::workspace::{CrateName, OrderFault, PackageVersion, RepositoryAddress, Workspace};

/// The budget a run has unless told otherwise. The release job's timeout is the budget and enough
/// to build the tool and begin one more publish, so the run stops itself, naming what remains,
/// before the job is killed partway through.
pub const DEFAULT_BUDGET: Duration = Duration::from_secs(30 * 60);

/// The longest budget a run may be given, and so the longest any one wait can be: an hour, twice the
/// default, beyond which the release job would have been killed long before.
pub const MAXIMUM_BUDGET: Duration = Duration::from_secs(60 * 60);

/// The shortest wait after any 429. A time to try again that has already passed, as a clock running
/// ahead of crates.io's reads it, would otherwise send the next attempt at once, and the one after.
pub const MINIMUM_WAIT: Duration = Duration::from_secs(10);

/// Whether a run publishes, or only says what it would.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Publish,
    Plan,
}

impl Mode {
    /// The tool's name in what it prints.
    fn name(self) -> &'static str {
        match self {
            Self::Publish => "publish-crates",
            Self::Plan => "publish-crates --plan",
        }
    }
}

/// What a release publishes: the workspace's publishable crates, in their order, at their one
/// version, from their one repository.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    version: PackageVersion,
    repository: RepositoryAddress,
    /// At least one crate, each after every other one it depends on.
    crates: Vec<CrateName>,
}

/// Why a workspace is no release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseFault {
    NothingPublished,
    Order(OrderFault),
    Versions(Vec<PackageVersion>),
    Repositories(Vec<RepositoryAddress>),
    NoRepository(CrateName),
}

/// `items`, each as it displays, joined by "and".
fn joined<T: fmt::Display>(items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" and ")
}

impl fmt::Display for ReleaseFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingPublished => formatter.write_str("the workspace publishes no crate"),
            Self::Order(fault) => fault.fmt(formatter),
            Self::Versions(versions) => write!(
                formatter,
                "the published crates are at more than one version, {}, and a release has one",
                joined(versions)
            ),
            Self::Repositories(repositories) => write!(
                formatter,
                "the published crates name more than one repository, {}",
                joined(repositories)
            ),
            Self::NoRepository(name) => write!(formatter, "{name} names no repository"),
        }
    }
}

impl Release {
    /// The release `workspace` makes.
    pub fn of(workspace: &Workspace) -> Result<Self, ReleaseFault> {
        let order = workspace.publishing_order().map_err(ReleaseFault::Order)?;
        if order.is_empty() {
            return Err(ReleaseFault::NothingPublished);
        }
        let versions: Vec<&PackageVersion> = order
            .iter()
            .map(|member| &member.version)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let [version] = versions.as_slice() else {
            return Err(ReleaseFault::Versions(
                versions.into_iter().cloned().collect(),
            ));
        };
        let mut repositories: BTreeSet<&RepositoryAddress> = BTreeSet::new();
        for member in &order {
            let repository = member
                .repository
                .as_ref()
                .ok_or_else(|| ReleaseFault::NoRepository(member.name.clone()))?;
            repositories.insert(repository);
        }
        let repositories: Vec<&RepositoryAddress> = repositories.into_iter().collect();
        let [repository] = repositories.as_slice() else {
            return Err(ReleaseFault::Repositories(
                repositories.into_iter().cloned().collect(),
            ));
        };
        Ok(Self {
            version: (*version).clone(),
            repository: (*repository).clone(),
            crates: order.iter().map(|member| member.name.clone()).collect(),
        })
    }

    /// Refuses a release that is not from the public repository, the only one crates publish from.
    pub fn require_public(&self, public: &PublicRepository) -> Result<(), Refusal> {
        if public.is(&self.repository) {
            Ok(())
        } else {
            Err(Refusal::NotThePublicRepository {
                named: self.repository.clone(),
                public: public.clone(),
            })
        }
    }
}

/// Why a run waited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitReason {
    /// The index answered 429 when asked about this crate.
    IndexAskedLater(CrateName),
    /// crates.io refused this crate's upload with 429.
    UploadAskedLater(CrateName),
}

impl fmt::Display for WaitReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexAskedLater(name) => write!(formatter, "the index answered 429 for {name}"),
            Self::UploadAskedLater(name) => write!(formatter, "crates.io answered 429 for {name}"),
        }
    }
}

/// What a run did, as it did it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// The index already holds the crate at this version.
    Skip(CrateName),
    /// A plan's crate the index does not hold yet.
    WouldPublish(CrateName),
    /// A publish begins.
    Publish(CrateName, Verification),
    /// It landed.
    Published(CrateName),
    /// cargo failed, and the index holds the crate, so its upload landed.
    Landed(CrateName),
    /// The run waits, for at least [`MINIMUM_WAIT`].
    Wait(Duration, WaitReason),
}

/// One line of a run's log, for an event at `version`.
pub fn line(event: &Event, version: &PackageVersion) -> String {
    match event {
        Event::Skip(name) => format!("  skip {name} {version}: the index already holds it"),
        Event::WouldPublish(name) => format!("  would publish {name} {version}"),
        Event::Publish(name, Verification::Build) => format!("  publish {name} {version}"),
        Event::Publish(name, Verification::AlreadyBuilt) => {
            format!("  publish {name} {version}, without building it again")
        }
        Event::Published(name) => format!("  ok   {name} {version}"),
        Event::Landed(name) => {
            format!("  ok   {name} {version}: cargo failed, and the index holds it")
        }
        Event::Wait(length, reason) => format!("  wait {}: {reason}", seconds(*length)),
    }
}

/// A length as whole seconds, as the log writes it.
fn seconds(length: Duration) -> String {
    format!("{:.0}s", length.as_secs_f64())
}

/// Why a run stopped with crates remaining.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The budget was spent before the next crate began.
    BudgetSpent {
        name: CrateName,
        version: PackageVersion,
    },
    /// The wait asked for would pass the budget.
    WaitPastBudget {
        reason: WaitReason,
        wait: Duration,
        left: Duration,
    },
    /// The index could not say whether it holds a crate.
    IndexRefused(IndexRefusal),
    /// cargo failed, and the index does not hold the crate.
    CargoFailed(CargoFailure),
}

impl fmt::Display for Stop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BudgetSpent { name, version } => write!(
                formatter,
                "the budget is spent, so {name} {version} was not begun"
            ),
            Self::WaitPastBudget { reason, wait, left } => write!(
                formatter,
                "{reason}, and waiting {} would pass the budget, which has {} left",
                seconds(*wait),
                seconds(*left)
            ),
            Self::IndexRefused(refusal) => refusal.fmt(formatter),
            Self::CargoFailed(failure) => failure.fmt(formatter),
        }
    }
}

/// How a run ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every crate is published, or for a plan, every crate was asked about.
    Finished,
    /// The run stopped. `remaining` is the crate it stopped at and every one after it.
    Stopped {
        stop: Stop,
        remaining: Vec<CrateName>,
    },
}

/// The time a run may still take, on the clock from the moment it began.
struct Budget<'clock, C: Clock> {
    clock: &'clock C,
    deadline: Duration,
}

impl<'clock, C: Clock> Budget<'clock, C> {
    fn starting(clock: &'clock C, length: Duration) -> Self {
        Self {
            clock,
            deadline: clock.monotonic().saturating_add(length.min(MAXIMUM_BUDGET)),
        }
    }

    /// What is left, nothing once it is spent.
    fn left(&self) -> Duration {
        self.deadline.saturating_sub(self.clock.monotonic())
    }

    /// Refuses to begin a crate once the budget is spent.
    fn begin(&self, name: &CrateName, version: &PackageVersion) -> Result<(), Stop> {
        if self.left().is_zero() {
            return Err(Stop::BudgetSpent {
                name: name.clone(),
                version: version.clone(),
            });
        }
        Ok(())
    }

    /// Waits what `requested` asks, and never less than [`MINIMUM_WAIT`], or stops when that
    /// would pass the budget.
    fn wait(
        &self,
        requested: Requested,
        reason: WaitReason,
        report: &mut impl FnMut(Event),
    ) -> Result<(), Stop> {
        let wait = match requested {
            Requested::For(length) => length.max(MINIMUM_WAIT),
            Requested::Passed => MINIMUM_WAIT,
        };
        let left = self.left();
        if wait > left {
            return Err(Stop::WaitPastBudget { reason, wait, left });
        }
        report(Event::Wait(wait, reason));
        self.clock.sleep(wait);
        Ok(())
    }
}

/// What one step of a run leaves to do.
enum Step {
    /// The crate is done; go on to the next.
    Next,
    /// Ask about the same crate again.
    Again,
}

/// Asks the index about `name`, waiting out a 429. `Ok(None)` when the run waited and should ask
/// again.
fn ask<C: Clock>(
    registry: &impl Registry,
    budget: &Budget<'_, C>,
    release: &Release,
    name: &CrateName,
    report: &mut impl FnMut(Event),
) -> Result<Option<bool>, Stop> {
    match registry.ask(name, &release.version) {
        IndexAnswer::Held => Ok(Some(true)),
        IndexAnswer::Absent => Ok(Some(false)),
        IndexAnswer::AskLater(retry) => {
            budget.wait(
                retry.requested(budget.clock.wall()),
                WaitReason::IndexAskedLater(name.clone()),
                report,
            )?;
            Ok(None)
        }
        IndexAnswer::Refused(refusal) => Err(Stop::IndexRefused(refusal)),
    }
}

/// Runs `step` over each crate in order, until every one is done or a step stops the run.
fn each_crate(
    release: &Release,
    mut step: impl FnMut(&CrateName) -> Result<Step, Stop>,
) -> Outcome {
    let mut index = 0;
    while let Some(name) = release.crates.get(index) {
        match step(name) {
            Ok(Step::Next) => index += 1,
            Ok(Step::Again) => {}
            Err(stop) => {
                return Outcome::Stopped {
                    stop,
                    remaining: release.crates.get(index..).unwrap_or_default().to_vec(),
                }
            }
        }
    }
    Outcome::Finished
}

/// Publishes each crate of `release` the index lacks, in order, within `budget`.
pub fn publish<C: Clock>(
    release: &Release,
    clock: &C,
    registry: &impl Registry,
    cargo: &impl Cargo,
    budget: Duration,
    report: &mut impl FnMut(Event),
) -> Outcome {
    let budget = Budget::starting(clock, budget);
    // The crates crates.io refused with 429 in this run, each built by the attempt it refused.
    let mut built: BTreeSet<CrateName> = BTreeSet::new();
    each_crate(release, |name| {
        budget.begin(name, &release.version)?;
        match ask(registry, &budget, release, name, report)? {
            None => return Ok(Step::Again),
            Some(true) => {
                report(Event::Skip(name.clone()));
                return Ok(Step::Next);
            }
            Some(false) => {}
        }
        let verification = if built.contains(name) {
            Verification::AlreadyBuilt
        } else {
            Verification::Build
        };
        report(Event::Publish(name.clone(), verification));
        match cargo::answer(cargo.publish(name, verification), name, &release.version) {
            CargoAnswer::Published => {
                report(Event::Published(name.clone()));
                Ok(Step::Next)
            }
            CargoAnswer::RateLimited(retry) => {
                built.insert(name.clone());
                budget.wait(
                    retry.requested(clock.wall()),
                    WaitReason::UploadAskedLater(name.clone()),
                    report,
                )?;
                Ok(Step::Again)
            }
            // cargo can fail after the upload has landed, so the registry is asked whether it holds
            // the version before the run stops.
            CargoAnswer::Failed(failure) => match registry.ask(name, &release.version) {
                IndexAnswer::Held => {
                    report(Event::Landed(name.clone()));
                    Ok(Step::Next)
                }
                IndexAnswer::Refused(refusal) => Err(Stop::IndexRefused(refusal)),
                IndexAnswer::Absent | IndexAnswer::AskLater(_) => Err(Stop::CargoFailed(failure)),
            },
        }
    })
}

/// Says which crates of `release` a run would publish, in order, and which it would skip, within
/// `budget`. It is given no [`Cargo`], so it publishes nothing.
pub fn plan<C: Clock>(
    release: &Release,
    clock: &C,
    registry: &impl Registry,
    budget: Duration,
    report: &mut impl FnMut(Event),
) -> Outcome {
    let budget = Budget::starting(clock, budget);
    each_crate(release, |name| {
        budget.begin(name, &release.version)?;
        match ask(registry, &budget, release, name, report)? {
            None => Ok(Step::Again),
            Some(true) => {
                report(Event::Skip(name.clone()));
                Ok(Step::Next)
            }
            Some(false) => {
                report(Event::WouldPublish(name.clone()));
                Ok(Step::Next)
            }
        }
    })
}

/// Why a run did not start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The workspace names another repository than the public one.
    NotThePublicRepository {
        named: RepositoryAddress,
        public: PublicRepository,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotThePublicRepository { named, public } => write!(
                formatter,
                "this workspace's repository is {named}, not {public}. Crates publish only from \
                 the public repository, because a published crate names where it came from and \
                 crates.io never forgets it"
            ),
        }
    }
}

/// Publishes `release`, refusing before a single request unless it is from `public`.
pub fn publish_release<C: Clock>(
    release: &Release,
    public: &PublicRepository,
    clock: &C,
    registry: &impl Registry,
    cargo: &impl Cargo,
    budget: Duration,
    report: &mut impl FnMut(Event),
) -> Result<Outcome, Refusal> {
    release.require_public(public)?;
    Ok(publish(release, clock, registry, cargo, budget, report))
}

/// How a run ends: its exit code, and the lines it prints to standard output and to standard error
/// as it ends.
///
/// 0 when every crate is published, or the plan reached every crate; 1 when the run stopped with
/// crates remaining, which the release job must not take for success; and 2 when the run did not
/// start, for any reason, a refusal of the repository among them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Closing {
    pub code: u8,
    pub out: Vec<String>,
    pub err: Vec<String>,
}

impl Closing {
    /// The run did not start, for `reason`.
    pub fn not_started(mode: Mode, reason: &dyn fmt::Display) -> Self {
        Self {
            code: 2,
            out: Vec::new(),
            err: vec![format!("{}: {reason}", mode.name())],
        }
    }

    /// The run of `release` ended with `outcome`.
    pub fn ended(mode: Mode, release: &Release, outcome: &Outcome) -> Self {
        let version = &release.version;
        match outcome {
            Outcome::Finished => Self {
                code: 0,
                out: vec![match mode {
                    Mode::Publish => format!("every crate is published at {version}"),
                    Mode::Plan => String::from("the plan reached every crate"),
                }],
                err: Vec::new(),
            },
            Outcome::Stopped { stop, remaining } => {
                let names: Vec<&str> = remaining.iter().map(CrateName::as_str).collect();
                let count = names.len();
                let names = names.join(", ");
                Self {
                    code: 1,
                    out: Vec::new(),
                    err: vec![
                        format!("{}: stopped: {stop}", mode.name()),
                        match mode {
                            Mode::Publish => format!(
                                "{}: {count} crate(s) remain: {names}. Run this again to publish \
                                 them; it skips what is already published",
                                mode.name()
                            ),
                            Mode::Plan => format!(
                                "{}: the plan has no answer for {count} crate(s): {names}",
                                mode.name()
                            ),
                        },
                    ],
                }
            }
        }
    }

    /// The lines a run of `release` prints before it begins, asking `index`, or how it ends without
    /// beginning: a publish refuses a release that is not from `public` before it prints anything.
    pub fn opening(
        mode: Mode,
        release: &Release,
        public: &PublicRepository,
        index: &str,
    ) -> Result<Vec<String>, Self> {
        let count = release.crates.len();
        let version = &release.version;
        Ok(match mode {
            Mode::Publish => {
                if let Err(refusal) = release.require_public(public) {
                    return Err(Self::not_started(mode, &refusal));
                }
                vec![format!(
                    "publishing {count} crate(s) at {version}, asking {index}"
                )]
            }
            Mode::Plan => {
                let mut lines = vec![format!(
                    "the plan for {count} crate(s) at {version}, asking {index}, publishing nothing:"
                )];
                if release.require_public(public).is_err() {
                    lines.push(format!(
                        "  this workspace's repository is {}, not {public}, so a run here would \
                         publish nothing. The plan is what a run from the public repository would \
                         do",
                        release.repository
                    ));
                }
                lines
            }
        })
    }

    /// Prints the lines, and gives the exit code.
    fn print(self) -> ExitCode {
        for line in &self.out {
            println!("{line}");
        }
        for line in &self.err {
            eprintln!("{line}");
        }
        ExitCode::from(self.code)
    }
}

/// Runs the tool from the current directory, on the real clock, index, and cargo, within `budget`,
/// which is at most [`MAXIMUM_BUDGET`]. How it ends is [`Closing`]'s.
pub fn run(mode: Mode, budget: Duration) -> ExitCode {
    let facts = match Facts::read() {
        Ok(facts) => facts,
        Err(fault) => {
            let reason = format!("xtask/release-facts.json {fault}");
            return Closing::not_started(mode, &reason).print();
        }
    };
    // The cargo that runs this tool, where one does, so the workspace is read and published by the
    // toolchain the repository pins.
    let program: OsString = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let directory = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(error) => {
            let reason = format!("cannot read the current directory: {}", error.kind());
            return Closing::not_started(mode, &reason).print();
        }
    };
    let workspace = match Workspace::read(&program, &directory) {
        Ok(workspace) => workspace,
        Err(fault) => return Closing::not_started(mode, &fault).print(),
    };
    let release = match Release::of(&workspace) {
        Ok(release) => release,
        Err(fault) => return Closing::not_started(mode, &fault).print(),
    };
    let public = facts.public_repository();
    // A publish refuses another repository before it prints, asks, or publishes anything.
    let opening = match Closing::opening(mode, &release, public, CRATES_IO_INDEX) {
        Ok(lines) => lines,
        Err(closing) => return closing.print(),
    };
    for line in opening {
        println!("{line}");
    }
    let clock = SystemClock::new();
    let registry = CurlRegistry::new(
        CRATES_IO_INDEX,
        facts.user_agent().clone(),
        Proxies::FromTheEnvironment,
    );
    let version = release.version.clone();
    let mut report = |event: Event| println!("{}", line(&event, &version));
    let outcome = match mode {
        Mode::Publish => {
            let cargo = RealCargo::new(program, workspace.root, facts.user_agent().clone());
            match publish_release(
                &release,
                public,
                &clock,
                &registry,
                &cargo,
                budget,
                &mut report,
            ) {
                Ok(outcome) => outcome,
                Err(refusal) => return Closing::not_started(mode, &refusal).print(),
            }
        }
        Mode::Plan => plan(&release, &clock, &registry, budget, &mut report),
    };
    Closing::ended(mode, &release, &outcome).print()
}
