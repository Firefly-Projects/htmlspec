//! The publisher's cases, on a virtual clock, an index held in memory, and a cargo that writes what
//! cargo writes.
//!
//! [`World`] is all three capabilities at once, so one value holds the time, what the index holds,
//! and every attempt to publish. Its clock moves only when the publisher waits or a publish takes
//! time, and it records each wait, so the cases that wait out a 429 take no real time and still
//! assert how long the wait was. Its cargo answers each attempt as the case plans: with 429 and a
//! time to try again, already published, a failed build, a failed build whose log holds every word
//! the publisher acts on, an upload that landed although cargo then failed, or a publish that takes
//! a while. It writes cargo's error as cargo does, so the publisher reads it as it reads cargo's.
//!
//! These are the cases the publisher's tests held before it was ported, each under the name it had.
//! The request headers are held on the wire, by a real `curl` against a listener on the loopback
//! interface, because the index in memory sends no request.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::cargo::{Cargo, CargoFailure, CargoRun, Verification};
use super::clock::Clock;
use super::registry::tests::{assert_only_the_user_agent, entries, reply, Listener};
use super::registry::{IndexAnswer, IndexRefusal, Registry};
use super::retry::{imf_fixdate, RetryAfter};
use super::*;
use crate::workspace::tests::{metadata, THIS_WORKSPACE};

const VERSION: &str = "9.8.7";
const PUBLIC: &str = "https://github.com/Firefly-Projects/htmlspec";

/// The six crates in the order they publish, spelled out rather than read from the code under
/// test.
const CRATES: [&str; 6] = [
    "htmlspec-schema",
    "htmlspec-parse",
    "htmlspec-assets",
    "htmlspec-format",
    "htmlspec-validate",
    "htmlspec-cli",
];

/// The virtual clock's wall time at its start, 2026-10-31T00:00:00Z.
fn origin() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_793_404_800)
}

fn name(text: &str) -> CrateName {
    CrateName::parse(text).expect("a crate's name")
}

fn names(texts: &[&str]) -> Vec<CrateName> {
    texts.iter().map(|text| name(text)).collect()
}

/// The release this workspace makes at [`VERSION`], from `repository`.
fn release_from(repository: &str) -> Release {
    let workspace = Workspace::parse(&metadata(&THIS_WORKSPACE, VERSION, repository))
        .expect("the workspace reads");
    Release::of(&workspace).expect("the workspace is a release")
}

fn release() -> Release {
    release_from(PUBLIC)
}

fn public() -> PublicRepository {
    Facts::read()
        .expect("the facts read")
        .public_repository()
        .clone()
}

const fn seconds(count: u64) -> Duration {
    Duration::from_secs(count)
}

/// How cargo answers one attempt to publish a crate.
#[derive(Clone, Copy, Debug)]
enum Attempt {
    /// crates.io refuses the upload with 429, naming a time to try again this many seconds after
    /// the attempt; negative for a time already passed.
    RateLimited(i64),
    /// cargo finds the version already on the index.
    Exists,
    /// The package fails to verify.
    Fails,
    /// The package fails to verify, and the build's log holds every word the publisher acts on.
    FailsNoisily,
    /// The upload lands, and cargo then fails waiting for its answer.
    Lost,
    /// The publish takes this many seconds, and lands.
    Slow(u64),
}

#[derive(Default)]
struct State {
    /// Time on the virtual clock since it started.
    now: Duration,
    waits: Vec<Duration>,
    /// The crates the index holds at [`VERSION`].
    held: BTreeSet<String>,
    /// How many more times the index answers 429 for each crate.
    asks_later: BTreeMap<String, usize>,
    asked: Vec<String>,
    attempts: BTreeMap<String, VecDeque<Attempt>>,
    calls: Vec<(String, Verification)>,
}

/// The clock, the index, and cargo, in memory.
#[derive(Default)]
struct World(RefCell<State>);

impl World {
    fn hold(&self, names: &[&str]) {
        self.0
            .borrow_mut()
            .held
            .extend(names.iter().map(|name| (*name).to_owned()));
    }

    fn plan(&self, name: &str, attempts: &[Attempt]) {
        self.0
            .borrow_mut()
            .attempts
            .insert(name.to_owned(), attempts.iter().copied().collect());
    }

    fn asks_later(&self, name: &str, times: usize) {
        self.0
            .borrow_mut()
            .asks_later
            .insert(name.to_owned(), times);
    }

    fn published(&self) -> Vec<String> {
        self.0
            .borrow()
            .calls
            .iter()
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn calls_for(&self, name: &str) -> Vec<Verification> {
        self.0
            .borrow()
            .calls
            .iter()
            .filter(|(called, _)| called == name)
            .map(|(_, verification)| *verification)
            .collect()
    }

    fn forget_calls(&self) {
        self.0.borrow_mut().calls.clear();
    }

    fn waits(&self) -> Vec<Duration> {
        self.0.borrow().waits.clone()
    }

    fn held(&self) -> BTreeSet<String> {
        self.0.borrow().held.clone()
    }

    fn asked(&self) -> Vec<String> {
        self.0.borrow().asked.clone()
    }

    /// A publish of the release within `budget`, and what it reported.
    fn publish(&self, budget: Duration) -> (Outcome, Vec<Event>) {
        let mut events = Vec::new();
        let outcome = publish(&release(), self, self, self, budget, &mut |event| {
            events.push(event)
        });
        (outcome, events)
    }
}

impl Clock for World {
    fn monotonic(&self) -> Duration {
        self.0.borrow().now
    }

    fn wall(&self) -> SystemTime {
        origin() + self.0.borrow().now
    }

    fn sleep(&self, length: Duration) {
        let mut state = self.0.borrow_mut();
        state.waits.push(length);
        state.now += length;
    }
}

impl Registry for World {
    fn ask(&self, name: &CrateName, _version: &PackageVersion) -> IndexAnswer {
        let mut state = self.0.borrow_mut();
        state.asked.push(name.to_string());
        if let Some(left) = state.asks_later.get_mut(name.as_str()) {
            if *left > 0 {
                *left -= 1;
                return IndexAnswer::AskLater(RetryAfter::Seconds(1));
            }
        }
        if state.held.contains(name.as_str()) {
            IndexAnswer::Held
        } else {
            IndexAnswer::Absent
        }
    }
}

impl Cargo for World {
    fn publish(&self, name: &CrateName, verification: Verification) -> CargoRun {
        let wall = self.wall();
        let mut state = self.0.borrow_mut();
        state.calls.push((name.to_string(), verification));
        let attempt = state
            .attempts
            .get_mut(name.as_str())
            .and_then(VecDeque::pop_front);
        let failed = |stderr: String| CargoRun::Failed { stderr };
        match attempt {
            Some(Attempt::RateLimited(after)) => {
                let offset = Duration::from_secs(after.unsigned_abs());
                let moment = if after >= 0 {
                    wall + offset
                } else {
                    wall - offset
                };
                return failed(format!(
                    "error: failed to publish {name} v{VERSION} to registry at https://crates.io\n\
                     \n\
                     Caused by:\n  the remote server responded with an error (status 429 Too Many \
                     Requests): You have published too many new crates in a short period of time. \
                     Please try again after {}.\n",
                    imf_fixdate(moment)
                ));
            }
            Some(Attempt::Exists) => {
                return failed(format!(
                    "error: crate {name}@{VERSION} already exists on crates.io index\n"
                ))
            }
            Some(Attempt::Fails) => {
                return failed(String::from("error: failed to verify package tarball\n"))
            }
            Some(Attempt::FailsNoisily) => {
                return failed(format!(
                    "   Compiling {name} v{VERSION}\n\
                     error: crate {name}@{VERSION} already exists on crates.io index\n\
                     error[E0599]: the test `a_429_too_many_requests_is_retried` is already uploaded\n  \
                     = note: (status 429 Too Many Requests): Please try again after Mon, 01 Jan 2001 00:00:00 GMT\n\
                     error: could not compile `{name}` (lib) due to 2 previous errors\n\
                     error: failed to verify package tarball\n"
                ))
            }
            Some(Attempt::Lost) => {
                state.held.insert(name.to_string());
                return failed(format!(
                    "error: failed to publish {name} v{VERSION} to registry at https://crates.io\n\
                     \n\
                     Caused by:\n  [28] Timeout was reached (Operation timed out after 30000 milliseconds)\n"
                ));
            }
            Some(Attempt::Slow(taking)) => state.now += seconds(taking),
            None => {}
        }
        state.held.insert(name.to_string());
        CargoRun::Succeeded
    }
}

/// cargo's failure when the package does not verify.
fn failed_to_verify(crate_name: &str) -> CargoFailure {
    CargoFailure::Said {
        name: name(crate_name),
        error: String::from("failed to verify package tarball"),
    }
}

fn stopped(stop: Stop, remaining: &[&str]) -> Outcome {
    Outcome::Stopped {
        stop,
        remaining: names(remaining),
    }
}

fn all_but(left_out: &[&str]) -> Vec<String> {
    CRATES
        .iter()
        .filter(|name| !left_out.contains(name))
        .map(|name| (*name).to_owned())
        .collect()
}

#[test]
fn a_rerun_that_finds_three_published_publishes_the_other_three() {
    let world = World::default();
    world.hold(&["htmlspec-schema", "htmlspec-parse", "htmlspec-assets"]);
    let (outcome, events) = world.publish(DEFAULT_BUDGET);
    assert_eq!(outcome, Outcome::Finished);
    assert_eq!(
        world.published(),
        ["htmlspec-format", "htmlspec-validate", "htmlspec-cli"]
    );
    for skipped in ["htmlspec-schema", "htmlspec-parse", "htmlspec-assets"] {
        assert!(events.contains(&Event::Skip(name(skipped))), "{events:?}");
        assert_eq!(
            line(&Event::Skip(name(skipped)), &release().version),
            format!("  skip {skipped} {VERSION}: the index already holds it")
        );
    }
    assert_eq!(world.waits(), []);
}

#[test]
fn a_429_answered_once_is_waited_out_and_the_set_finishes() {
    let world = World::default();
    world.plan("htmlspec-cli", &[Attempt::RateLimited(2)]);
    let (outcome, events) = world.publish(DEFAULT_BUDGET);
    assert_eq!(outcome, Outcome::Finished);
    let mut expected = all_but(&[]);
    expected.push(String::from("htmlspec-cli"));
    assert_eq!(world.published(), expected);
    assert_eq!(world.held().len(), CRATES.len());
    // Two seconds named, and the floor of ten waited.
    let waited = Event::Wait(
        MINIMUM_WAIT,
        WaitReason::UploadAskedLater(name("htmlspec-cli")),
    );
    assert!(events.contains(&waited), "{events:?}");
    assert_eq!(
        line(&waited, &release().version),
        "  wait 10s: crates.io answered 429 for htmlspec-cli"
    );
    assert_eq!(world.waits(), [MINIMUM_WAIT]);
}

#[test]
fn a_crate_refused_with_429_is_not_built_again() {
    let world = World::default();
    world.plan("htmlspec-cli", &[Attempt::RateLimited(60)]);
    let (outcome, _) = world.publish(DEFAULT_BUDGET);
    assert_eq!(outcome, Outcome::Finished);
    assert_eq!(
        world.calls_for("htmlspec-cli"),
        [Verification::Build, Verification::AlreadyBuilt]
    );
    for other in all_but(&["htmlspec-cli"]) {
        assert_eq!(world.calls_for(&other), [Verification::Build], "{other}");
    }
    assert_eq!(world.waits(), [seconds(60)]);
}

#[test]
fn the_budget_counts_the_time_publishing_takes() {
    // Two publishes of three seconds each against a budget of four: no wait is ever asked for, so a
    // budget that counted waiting alone would publish all six.
    let world = World::default();
    world.plan("htmlspec-schema", &[Attempt::Slow(3)]);
    world.plan("htmlspec-parse", &[Attempt::Slow(3)]);
    let (outcome, _) = world.publish(seconds(4));
    let expected = stopped(
        Stop::BudgetSpent {
            name: name("htmlspec-assets"),
            version: release().version,
        },
        &CRATES[2..],
    );
    assert_eq!(outcome, expected);
    if let Outcome::Stopped { stop, .. } = outcome {
        assert_eq!(
            stop.to_string(),
            "the budget is spent, so htmlspec-assets 9.8.7 was not begun"
        );
    }
    assert_eq!(world.published(), ["htmlspec-schema", "htmlspec-parse"]);
    assert_eq!(world.waits(), []);
}

#[test]
fn a_time_to_try_again_that_has_passed_is_still_waited_for() {
    // Within a budget of five seconds, the floor of ten passes it.
    let world = World::default();
    world.plan("htmlspec-cli", &[Attempt::RateLimited(-30)]);
    let (outcome, _) = world.publish(seconds(5));
    let stop = Stop::WaitPastBudget {
        reason: WaitReason::UploadAskedLater(name("htmlspec-cli")),
        wait: MINIMUM_WAIT,
        left: seconds(5),
    };
    assert_eq!(
        stop.to_string(),
        "crates.io answered 429 for htmlspec-cli, and waiting 10s would pass the budget, which \
         has 5s left"
    );
    assert_eq!(outcome, stopped(stop, &["htmlspec-cli"]));
    assert_eq!(
        world.calls_for("htmlspec-cli").len(),
        1,
        "it tried again at once"
    );

    // Within the whole budget, the floor is waited, and the crate is tried again.
    let world = World::default();
    world.plan("htmlspec-cli", &[Attempt::RateLimited(-30)]);
    let (outcome, _) = world.publish(DEFAULT_BUDGET);
    assert_eq!(outcome, Outcome::Finished);
    assert_eq!(world.waits(), [MINIMUM_WAIT]);
    assert_eq!(world.calls_for("htmlspec-cli").len(), 2);
}

#[test]
fn words_in_the_build_log_are_not_read_as_answers() {
    let world = World::default();
    world.plan("htmlspec-format", &[Attempt::FailsNoisily]);
    let (outcome, events) = world.publish(DEFAULT_BUDGET);
    assert_eq!(
        outcome,
        stopped(
            Stop::CargoFailed(failed_to_verify("htmlspec-format")),
            &CRATES[3..]
        )
    );
    assert_eq!(world.calls_for("htmlspec-format").len(), 1);
    assert!(
        !events.iter().any(|event| matches!(event, Event::Wait(..))),
        "{events:?}"
    );
    assert_eq!(world.waits(), []);
}

#[test]
fn a_publish_that_failed_after_its_upload_landed_counts_as_published() {
    let world = World::default();
    world.plan("htmlspec-schema", &[Attempt::Lost]);
    let (outcome, events) = world.publish(DEFAULT_BUDGET);
    assert_eq!(outcome, Outcome::Finished);
    let landed = Event::Landed(name("htmlspec-schema"));
    assert!(events.contains(&landed), "{events:?}");
    assert_eq!(
        line(&landed, &release().version),
        "  ok   htmlspec-schema 9.8.7: cargo failed, and the index holds it"
    );
    assert_eq!(world.published(), CRATES);
}

#[test]
fn a_wait_past_the_budget_stops_and_names_what_remains_and_a_rerun_finishes() {
    let world = World::default();
    world.plan("htmlspec-cli", &[Attempt::RateLimited(3600)]);
    let (outcome, _) = world.publish(DEFAULT_BUDGET);
    assert_eq!(
        outcome,
        stopped(
            Stop::WaitPastBudget {
                reason: WaitReason::UploadAskedLater(name("htmlspec-cli")),
                wait: seconds(3600),
                left: DEFAULT_BUDGET,
            },
            &["htmlspec-cli"]
        )
    );
    assert_eq!(
        world.waits(),
        [],
        "it waited although the wait passed the budget"
    );
    assert_eq!(
        world.held(),
        all_but(&["htmlspec-cli"]).into_iter().collect()
    );

    world.forget_calls();
    let (finished, _) = world.publish(DEFAULT_BUDGET);
    assert_eq!(finished, Outcome::Finished);
    assert_eq!(world.published(), ["htmlspec-cli"]);
}

#[test]
fn the_budget_given_bounds_the_wait() {
    let world = World::default();
    world.plan("htmlspec-cli", &[Attempt::RateLimited(30)]);
    let (outcome, _) = world.publish(seconds(5));
    assert_eq!(
        outcome,
        stopped(
            Stop::WaitPastBudget {
                reason: WaitReason::UploadAskedLater(name("htmlspec-cli")),
                wait: seconds(30),
                left: seconds(5),
            },
            &["htmlspec-cli"]
        )
    );

    // A budget with room for it waits the thirty seconds named.
    let world = World::default();
    world.plan("htmlspec-cli", &[Attempt::RateLimited(30)]);
    assert_eq!(world.publish(seconds(31)).0, Outcome::Finished);
    assert_eq!(world.waits(), [seconds(30)]);
}

#[test]
fn an_index_answering_429_is_waited_out() {
    let world = World::default();
    world.asks_later("htmlspec-schema", 1);
    let (outcome, events) = world.publish(DEFAULT_BUDGET);
    assert_eq!(outcome, Outcome::Finished);
    let waited = Event::Wait(
        MINIMUM_WAIT,
        WaitReason::IndexAskedLater(name("htmlspec-schema")),
    );
    assert!(events.contains(&waited), "{events:?}");
    assert_eq!(
        line(&waited, &release().version),
        "  wait 10s: the index answered 429 for htmlspec-schema"
    );
    assert_eq!(world.waits(), [MINIMUM_WAIT]);
    assert_eq!(world.published(), CRATES);
}

#[test]
fn a_version_cargo_finds_already_published_counts_as_published() {
    let world = World::default();
    world.plan("htmlspec-schema", &[Attempt::Exists]);
    let (outcome, events) = world.publish(DEFAULT_BUDGET);
    assert_eq!(outcome, Outcome::Finished);
    assert!(events.contains(&Event::Published(name("htmlspec-schema"))));
    assert_eq!(world.published(), CRATES);
}

#[test]
fn any_other_failure_stops_and_names_what_remains() {
    let world = World::default();
    world.plan("htmlspec-format", &[Attempt::Fails]);
    let (outcome, _) = world.publish(DEFAULT_BUDGET);
    assert_eq!(
        outcome,
        stopped(
            Stop::CargoFailed(failed_to_verify("htmlspec-format")),
            &["htmlspec-format", "htmlspec-validate", "htmlspec-cli"]
        )
    );
    assert_eq!(world.published(), CRATES[..4]);
}

/// cargo failed, and the index, asked whether the upload landed anyway, could not say. The run
/// stops with the index's refusal, as the Python's did, and names what remains.
#[test]
fn an_index_that_refuses_after_cargo_failed_stops_the_run_with_its_refusal() {
    /// The world's index, which refuses the second question about one crate.
    struct RefusingTheSecond<'world> {
        world: &'world World,
        asked: RefCell<usize>,
    }
    impl Registry for RefusingTheSecond<'_> {
        fn ask(&self, name: &CrateName, version: &PackageVersion) -> IndexAnswer {
            if name.as_str() == "htmlspec-format" {
                *self.asked.borrow_mut() += 1;
                if *self.asked.borrow() == 2 {
                    return IndexAnswer::Refused(IndexRefusal::Status {
                        name: name.clone(),
                        status: 503,
                    });
                }
            }
            self.world.ask(name, version)
        }
    }
    let world = World::default();
    world.plan("htmlspec-format", &[Attempt::Fails]);
    let index = RefusingTheSecond {
        world: &world,
        asked: RefCell::new(0),
    };
    let outcome = publish(
        &release(),
        &world,
        &index,
        &world,
        DEFAULT_BUDGET,
        &mut |_| {},
    );
    assert_eq!(
        outcome,
        stopped(
            Stop::IndexRefused(IndexRefusal::Status {
                name: name("htmlspec-format"),
                status: 503,
            }),
            &CRATES[3..]
        )
    );
}

/// The index is a listener on the loopback interface, asked by a real `curl`, and it answers 429
/// once, so a retried request is held too. The clock is still virtual, so the wait is not.
#[test]
fn every_request_carries_the_fixed_user_agent_and_nothing_else() {
    let held: Arc<Mutex<BTreeSet<String>>> = Arc::new(Mutex::new(BTreeSet::from([String::from(
        "htmlspec-schema",
    )])));
    let asked_later = Arc::new(Mutex::new(false));
    let listener = {
        let held = Arc::clone(&held);
        Listener::answering(move |path| {
            let crate_name = path.rsplit('/').next().unwrap_or_default().to_owned();
            let mut once = asked_later.lock().unwrap_or_else(PoisonError::into_inner);
            if crate_name == "htmlspec-parse" && !*once {
                *once = true;
                return reply(429, &[("Retry-After", "1")], "");
            }
            if held
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .contains(&crate_name)
            {
                reply(200, &[], &entries(&crate_name, &["0.0.1", VERSION]))
            } else {
                reply(404, &[], "")
            }
        })
    };
    /// A cargo whose uploads land on the listener's index.
    struct Uploads(Arc<Mutex<BTreeSet<String>>>);
    impl Cargo for Uploads {
        fn publish(&self, name: &CrateName, _: Verification) -> CargoRun {
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(name.to_string());
            CargoRun::Succeeded
        }
    }
    let clock = World::default();
    let outcome = publish(
        &release(),
        &clock,
        &listener.registry(),
        &Uploads(Arc::clone(&held)),
        DEFAULT_BUDGET,
        &mut |_| {},
    );
    assert_eq!(outcome, Outcome::Finished);
    assert_eq!(clock.waits(), [MINIMUM_WAIT]);
    let received = listener.received();
    // Six crates asked about, and parse asked twice.
    assert_eq!(received.len(), CRATES.len() + 1, "{received:?}");
    for request in &received {
        assert_only_the_user_agent(request);
    }
}

#[test]
fn a_workspace_naming_another_repository_publishes_nothing() {
    let world = World::default();
    let refused = publish_release(
        &release_from("https://example.invalid/somewhere-else"),
        &public(),
        &world,
        &world,
        &world,
        DEFAULT_BUDGET,
        &mut |_| {},
    );
    match refused {
        Err(refusal) => assert!(
            refusal
                .to_string()
                .contains("Crates publish only from the public repository"),
            "{refusal}"
        ),
        Ok(outcome) => panic!("another repository's workspace ran: {outcome:?}"),
    }
    assert_eq!(world.asked(), Vec::<String>::new());
    assert_eq!(world.published(), Vec::<String>::new());

    // The public one runs.
    let world = World::default();
    assert_eq!(
        publish_release(
            &release(),
            &public(),
            &world,
            &world,
            &world,
            DEFAULT_BUDGET,
            &mut |_| {}
        ),
        Ok(Outcome::Finished)
    );
}

/// `--plan` asks the index as a run does, and says what it would do. It is given no cargo, so it
/// cannot publish.
#[test]
fn the_plan_says_what_a_run_would_publish_and_skip_in_order() {
    let world = World::default();
    world.hold(&["htmlspec-schema", "htmlspec-parse"]);
    world.asks_later("htmlspec-assets", 1);
    let mut events = Vec::new();
    let outcome = plan(&release(), &world, &world, DEFAULT_BUDGET, &mut |event| {
        events.push(event)
    });
    assert_eq!(outcome, Outcome::Finished);
    assert_eq!(
        events,
        [
            Event::Skip(name("htmlspec-schema")),
            Event::Skip(name("htmlspec-parse")),
            Event::Wait(
                MINIMUM_WAIT,
                WaitReason::IndexAskedLater(name("htmlspec-assets"))
            ),
            Event::WouldPublish(name("htmlspec-assets")),
            Event::WouldPublish(name("htmlspec-format")),
            Event::WouldPublish(name("htmlspec-validate")),
            Event::WouldPublish(name("htmlspec-cli")),
        ]
    );
    assert_eq!(world.published(), Vec::<String>::new());
    assert_eq!(world.held().len(), 2);
}

#[test]
fn a_plan_stops_where_the_index_refuses_naming_what_it_did_not_reach() {
    /// An index that holds nothing and refuses one crate.
    struct Refusing;
    impl Registry for Refusing {
        fn ask(&self, name: &CrateName, _: &PackageVersion) -> IndexAnswer {
            if name.as_str() == "htmlspec-format" {
                IndexAnswer::Refused(IndexRefusal::Status {
                    name: name.clone(),
                    status: 500,
                })
            } else {
                IndexAnswer::Absent
            }
        }
    }
    let outcome = plan(
        &release(),
        &World::default(),
        &Refusing,
        DEFAULT_BUDGET,
        &mut |_| {},
    );
    assert_eq!(
        outcome,
        stopped(
            Stop::IndexRefused(IndexRefusal::Status {
                name: name("htmlspec-format"),
                status: 500,
            }),
            &CRATES[3..]
        )
    );
}

/// The edge reads this workspace through `cargo metadata`, so the release it makes is held here:
/// the six crates in their order, at the workspace's own version.
#[test]
fn this_workspace_is_a_release_of_the_six_crates_at_its_own_version() {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let workspace = Workspace::read(&cargo, &root).expect("this workspace reads");
    let release = Release::of(&workspace).expect("this workspace is a release");
    assert_eq!(release.version.as_str(), env!("CARGO_PKG_VERSION"));
    assert_eq!(release.crates, names(&CRATES));
}

#[test]
fn a_workspace_at_two_versions_or_with_nothing_to_publish_is_no_release() {
    let mut workspace =
        Workspace::parse(&metadata(&THIS_WORKSPACE, VERSION, PUBLIC)).expect("the workspace reads");
    if let Some(member) = workspace.members.first_mut() {
        member.version = PackageVersion::parse("9.8.8").expect("a version");
    }
    assert_eq!(
        Release::of(&workspace),
        Err(ReleaseFault::Versions(vec![
            PackageVersion::parse("9.8.7").expect("a version"),
            PackageVersion::parse("9.8.8").expect("a version"),
        ]))
    );
    for member in &mut workspace.members {
        member.publishable = false;
    }
    assert_eq!(Release::of(&workspace), Err(ReleaseFault::NothingPublished));
}

/// The release job's step fails on any exit but 0, so a run stopped with crates remaining must
/// never end 0. Each closing is held whole: its code, and each line it prints and where.
#[test]
fn a_finished_publish_exits_zero_saying_every_crate_is_published() {
    assert_eq!(
        Closing::ended(Mode::Publish, &release(), &Outcome::Finished),
        Closing {
            code: 0,
            out: vec![String::from("every crate is published at 9.8.7")],
            err: Vec::new(),
        }
    );
}

#[test]
fn a_stopped_publish_exits_one_naming_what_remains() {
    let world = World::default();
    world.plan("htmlspec-format", &[Attempt::Fails]);
    let (outcome, _) = world.publish(DEFAULT_BUDGET);
    assert_eq!(
        Closing::ended(Mode::Publish, &release(), &outcome),
        Closing {
            code: 1,
            out: Vec::new(),
            err: vec![
                String::from(
                    "publish-crates: stopped: cargo publish -p htmlspec-format: failed to verify \
                     package tarball"
                ),
                String::from(
                    "publish-crates: 3 crate(s) remain: htmlspec-format, htmlspec-validate, \
                     htmlspec-cli. Run this again to publish them; it skips what is already \
                     published"
                ),
            ],
        }
    );

    let world = World::default();
    world.plan("htmlspec-cli", &[Attempt::RateLimited(3600)]);
    let (outcome, _) = world.publish(DEFAULT_BUDGET);
    let closing = Closing::ended(Mode::Publish, &release(), &outcome);
    assert_eq!(closing.code, 1);
    assert_eq!(
        closing.err,
        [
            "publish-crates: stopped: crates.io answered 429 for htmlspec-cli, and waiting 3600s \
             would pass the budget, which has 1800s left",
            "publish-crates: 1 crate(s) remain: htmlspec-cli. Run this again to publish them; it \
             skips what is already published",
        ]
    );
}

/// A plan that stops counts the crates it has no answer for, the one the index refused among
/// them.
#[test]
fn a_plan_exits_zero_when_it_reached_every_crate_and_one_when_it_did_not() {
    assert_eq!(
        Closing::ended(Mode::Plan, &release(), &Outcome::Finished),
        Closing {
            code: 0,
            out: vec![String::from("the plan reached every crate")],
            err: Vec::new(),
        }
    );
    let stopped = stopped(
        Stop::IndexRefused(IndexRefusal::Status {
            name: name("htmlspec-format"),
            status: 500,
        }),
        &CRATES[3..],
    );
    assert_eq!(
        Closing::ended(Mode::Plan, &release(), &stopped),
        Closing {
            code: 1,
            out: Vec::new(),
            err: vec![
                String::from(
                    "publish-crates --plan: stopped: the index answered HTTP 500 for \
                     htmlspec-format"
                ),
                String::from(
                    "publish-crates --plan: the plan has no answer for 3 crate(s): \
                     htmlspec-format, htmlspec-validate, htmlspec-cli"
                ),
            ],
        }
    );
}

/// A publish refuses another repository in its opening, before it prints a line; a plan says so
/// and goes on.
#[test]
fn a_publish_from_another_repository_exits_two_before_it_prints_anything() {
    let other = release_from("https://example.invalid/somewhere-else");
    assert_eq!(
        Closing::opening(Mode::Publish, &other, &public(), "https://index.crates.io"),
        Err(Closing {
            code: 2,
            out: Vec::new(),
            err: vec![String::from(
                "publish-crates: this workspace's repository is \
                 https://example.invalid/somewhere-else, not \
                 https://github.com/Firefly-Projects/htmlspec. Crates publish only from the public \
                 repository, because a published crate names where it came from and crates.io \
                 never forgets it"
            )],
        })
    );
    assert_eq!(
        Closing::opening(
            Mode::Publish,
            &release(),
            &public(),
            "https://index.crates.io"
        ),
        Ok(vec![String::from(
            "publishing 6 crate(s) at 9.8.7, asking https://index.crates.io"
        )])
    );
    let planned = Closing::opening(Mode::Plan, &other, &public(), "https://index.crates.io");
    assert_eq!(planned.as_ref().map(Vec::len), Ok(2), "{planned:?}");
}

/// The irreversible case: with `htmlspec-format` marked `publish = false`, a run would publish
/// schema, parse, and assets, and then crates.io would refuse validate. The release is refused,
/// and the run exits 2 before it asks the index anything.
#[test]
fn a_release_whose_crate_depends_on_an_unpublished_one_exits_two_and_publishes_nothing() {
    use crate::workspace::tests::Sketch;
    let members: Vec<Sketch<'_>> = THIS_WORKSPACE
        .into_iter()
        .map(|member| Sketch {
            publish: member.publish && member.name != "htmlspec-format",
            ..member
        })
        .collect();
    let workspace =
        Workspace::parse(&metadata(&members, VERSION, PUBLIC)).expect("the workspace reads");
    let Err(fault) = Release::of(&workspace) else {
        panic!("a release whose crate depends on an unpublished one was made");
    };
    assert!(
        matches!(&fault, ReleaseFault::Order(OrderFault::DependsOnUnpublishable(pairs)) if pairs.len() == 2),
        "{fault:?}"
    );
    let closing = Closing::not_started(Mode::Publish, &fault);
    assert_eq!(closing.code, 2);
    assert_eq!(closing.out, Vec::<String>::new());
    assert!(
        closing.err.first().is_some_and(
            |line| line.starts_with("publish-crates: htmlspec-cli depends on htmlspec-format")
        ),
        "{closing:?}"
    );
}
