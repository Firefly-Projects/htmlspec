//! Whether a run of another runbook has finished and passed, for a run check to require.
//!
//! The change `require-a-run-of-another-runbook` lets a runbook declare, in its specification, that
//! a finished, passed run of another runbook in the same project must exist before its own run
//! starts. The declaration is a preflight check of kind `run` ([`crate::preflight::MachineCheck`]),
//! and this module answers the three questions that check asks of a project rather than of a
//! machine:
//!
//! - **Which runbook does the name name?** The one whose directory name equals it byte for byte,
//!   among the directories the project's runbook enumeration finds ([`runbook_names`]). Never a
//!   filesystem lookup, which a case-insensitive filesystem answers differently (the design's
//!   decision 2).
//! - **Can the order be known, and does it hold?** The declarations reached from the named runbook
//!   are followed ([`walk`]). A chain that leads back to the declaring runbook is an invalid
//!   contract ([`refuse_cycles`]); a specification on the way that cannot be read leaves the check
//!   unavailable (decision 7).
//! - **Does a record satisfy it?** Every file under the named runbook's `runs/` is decided from its
//!   projected model ([`read_records`]), by decision 3's rule, and any record that satisfies it meets
//!   the check.
//!
//! # Not `RecordedRun`
//!
//! `run --check` reads records into [`htmlspec_validate::rules::runs::RecordedRun`], which holds only
//! the identifiers a record recorded an outcome for. That type is a public struct with public fields
//! in a crate published on crates.io, so widening it would break any downstream code that builds
//! one, and it is deliberately reduced to what the conformance rule needs. The decision here reads the
//! projected model directly instead, through the same [`crate::runs::gather`] and the same recovery
//! gate `run --check` applies on its read path.

use std::collections::BTreeSet;
use std::path::Path;

use crate::preflight::{
    ContractErrors, ContractFault, DeclarationFault, Fact, MachineCheck, MachineContract,
    ProbeFault, RequirementId, RunbookName,
};
use crate::terminal::shown;
use crate::version::RunbookVersion;

/// The head meta a run's record states the runbook release it ran with.
///
/// Written by `run --as` into each new record and read back here, so a version constraint has a
/// structured value to compare. A record's masthead also names the release, in prose, and the
/// project does not read a version out of display text (the design's decision 4).
pub(crate) const RELEASE_META: &str = "htmlspec-runbook-release";

/// How many satisfying records a report, a record's method, and the listing name.
///
/// Bounded so a runbook with hundreds of nightly runs does not print hundreds of names. The first
/// ones in name order are named, which says nothing about which run came first: a record states no
/// time (the design's decision 3).
pub(crate) const NAMED: usize = 3;

/// The project's runbooks by directory name, in byte order.
///
/// The same directories `list --runbooks` enumerates. A directory whose name is not UTF-8 names no
/// runbook a declaration can spell, so it is left out rather than spelled lossily, which would let a
/// declaration match a name no directory has.
pub(crate) fn runbook_names(runbooks: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(runbooks) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}

/// Whether `name` is, byte for byte, the name of one of the project's runbooks.
pub(crate) fn names_a_runbook(runbooks: &Path, name: &str) -> bool {
    runbook_names(runbooks).iter().any(|found| found == name)
}

/// The enumerated name of the runbook directory `directory`, which `typed` reached.
///
/// A command line finds a runbook by asking the filesystem for a directory, and a filesystem that
/// folds case answers `MIGRATE` with the `migrate` directory. Every run check is decided on
/// enumerated names, byte for byte (the design's decision 2), so a self-reference or a cycle spelled
/// through another case would go unseen if the typed name were used. So the runbook is known by the
/// one enumerated name that is the same directory, asked of the filesystem by device and inode, and
/// by `typed` itself when it is already one.
///
/// Every name 1.1.1 ran still runs, so a typed name resolves in this order: itself, when it is an
/// enumerated name; else, among the enumerated names for the same directory, the one equal to it
/// ignoring case; else the only enumerated name for that directory. A symbolic link beside its
/// target lists two names for one directory, so `OUTSIDE` reaches `outside` beside `outside-link`
/// by the second rule. `None` only when it stays ambiguous: several names for the directory, none
/// equal to the typed one even ignoring case, or none at all.
pub(crate) fn enumerated_name(directory: &Path, typed: &str) -> Option<String> {
    let runbooks = directory.parent()?;
    let names = runbook_names(runbooks);
    if names.iter().any(|name| name == typed) {
        return Some(typed.to_owned());
    }
    let same: Vec<String> = names
        .into_iter()
        .filter(|name| crate::commands::same_directory(&runbooks.join(name), directory))
        .collect();
    let folded: Vec<&String> = same
        .iter()
        .filter(|name| name.to_lowercase() == typed.to_lowercase())
        .collect();
    match (folded.as_slice(), same.as_slice()) {
        ([name], _) => Some((*name).clone()),
        ([], [name]) => Some(name.clone()),
        _ => None,
    }
}

/// One run check a runbook declares.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunDeclaration {
    /// The requirement that declares it.
    pub requirement: RequirementId,
    /// The runbook it requires a run of.
    pub runbook: RunbookName,
    /// The lowest release whose runs count, where declared.
    pub at_least: Option<RunbookVersion>,
}

/// What a runbook's specification says about the runs it requires.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Declarations {
    /// The specification was read and every declaration in it is one this program reads.
    Readable(Vec<RunDeclaration>),
    /// The specification was read, and a declaration in it is malformed.
    Malformed(String),
    /// The specification cannot be read, needs error recovery, or does not validate.
    Unreadable(String),
}

/// The run checks the runbook named `name` declares.
///
/// Reads its `specification.html` the way a start reads it: present, valid, recovery-free, and every
/// declaration parsed. A runbook whose declarations cannot be known is never read as one that
/// declares nothing, because that would report an order as clear when nobody could see it.
pub(crate) fn declarations_of(runbooks: &Path, name: &str) -> Declarations {
    let path = runbooks.join(name).join(crate::runs::CONTRACT);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            return Declarations::Unreadable(format!(
                "its {} cannot be read: {error}",
                crate::runs::CONTRACT
            ))
        }
    };
    // Recovery is asked first: a document that needs it also fails validation, which would
    // otherwise name the lesser fault and hide the one the operator has to repair first.
    let parsed = htmlspec_parse::parse_document(&source);
    if !parsed.is_recovery_free() {
        return Declarations::Unreadable(format!(
            "its {} needs error recovery to parse",
            crate::runs::CONTRACT
        ));
    }
    if !crate::commands::validate_source_at(&source, &path).is_accepted() {
        return Declarations::Unreadable(format!(
            "its {} does not validate",
            crate::runs::CONTRACT
        ));
    }
    let projection = htmlspec_parse::project(&parsed.document);
    match crate::preflight::parse_contract(&projection.model) {
        Ok(contract) => Declarations::Readable(run_declarations(&contract)),
        Err(faults) => Declarations::Malformed(format!(
            "its {} declares a check this program cannot read: {faults}",
            crate::runs::CONTRACT
        )),
    }
}

/// The run checks in a contract, in document order.
pub(crate) fn run_declarations(contract: &MachineContract) -> Vec<RunDeclaration> {
    contract
        .checks()
        .iter()
        .filter_map(|declared| match &declared.check {
            MachineCheck::Run { runbook, at_least } => Some(RunDeclaration {
                requirement: declared.requirement.clone(),
                runbook: runbook.clone(),
                at_least: *at_least,
            }),
            _ => None,
        })
        .collect()
}

/// What following the declarations from one runbook found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Walk {
    /// A chain of declarations reaches the target: the runbooks in order, from the start to the
    /// target inclusive.
    Leads(Vec<String>),
    /// No chain reaches the target among what could be read, and a specification on the way could
    /// not be read, so whether one does is not known.
    Unreadable {
        /// The runbook whose specification could not be read.
        runbook: String,
        /// Why not.
        reason: String,
    },
    /// Every specification on the way was read, and no chain reaches the target.
    Clear,
}

/// Follows the declarations from `start`, looking for a chain to `target`.
///
/// Depth first, in document order, each runbook visited once, so the cycle it reports is the first
/// one in declaration order and the answer is the same on every machine. A declared name that is no
/// runbook of the project is a dead end, since it has no declarations to follow. With no target it
/// answers only whether every specification reached can be read.
pub(crate) fn walk(runbooks: &Path, start: &str, target: Option<&str>) -> Walk {
    let names: BTreeSet<String> = runbook_names(runbooks).into_iter().collect();
    let mut visited = BTreeSet::new();
    let mut unreadable = None;
    let mut path = vec![start.to_owned()];
    if let Some(chain) = descend(
        runbooks,
        &names,
        target,
        &mut visited,
        &mut path,
        &mut unreadable,
    ) {
        return Walk::Leads(chain);
    }
    match unreadable {
        Some((runbook, reason)) => Walk::Unreadable { runbook, reason },
        None => Walk::Clear,
    }
}

/// One step of [`walk`]: the runbook at the end of `path`.
fn descend(
    runbooks: &Path,
    names: &BTreeSet<String>,
    target: Option<&str>,
    visited: &mut BTreeSet<String>,
    path: &mut Vec<String>,
    unreadable: &mut Option<(String, String)>,
) -> Option<Vec<String>> {
    let current = path.last()?.clone();
    if !visited.insert(current.clone()) {
        return None;
    }
    let declarations = match declarations_of(runbooks, &current) {
        Declarations::Readable(declarations) => declarations,
        Declarations::Malformed(reason) | Declarations::Unreadable(reason) => {
            unreadable.get_or_insert((current, reason));
            return None;
        }
    };
    for declaration in declarations {
        let next = declaration.runbook.as_str();
        if target == Some(next) {
            let mut chain = path.clone();
            chain.push(next.to_owned());
            return Some(chain);
        }
        if !names.contains(next) {
            continue;
        }
        path.push(next.to_owned());
        let found = descend(runbooks, names, target, visited, path, unreadable);
        path.pop();
        if found.is_some() {
            return found;
        }
    }
    None
}

/// Refuses a contract whose run checks name its own runbook or lead back to it.
///
/// An invalid contract, reported before any record is read and before any report is written, as
/// every malformed declaration is: the runbook can never start as written. Only chains through the
/// declaring runbook are looked for. A cycle elsewhere in the project, among runbooks this one does
/// not reach, is not this runbook's fault (the design's decision 7). A specification on the way that
/// cannot be read refuses nothing here: whether its declarations lead back is not known, and the
/// check reports that as unavailable when it is observed.
pub(crate) fn refuse_cycles(
    runbooks: &Path,
    declaring: &str,
    contract: &MachineContract,
) -> Result<(), ContractErrors> {
    let mut faults = Vec::new();
    for declaration in run_declarations(contract) {
        let named = declaration.runbook.as_str();
        if named == declaring {
            faults.push(ContractFault {
                requirement: Some(declaration.requirement.clone()),
                fault: DeclarationFault::RequiresItself {
                    runbook: named.to_owned(),
                },
            });
            continue;
        }
        if !names_a_runbook(runbooks, named) {
            continue;
        }
        if let Walk::Leads(chain) = walk(runbooks, named, Some(declaring)) {
            let mut cycle = vec![declaring.to_owned()];
            cycle.extend(chain);
            faults.push(ContractFault {
                requirement: Some(declaration.requirement.clone()),
                fault: DeclarationFault::Cycle { chain: cycle },
            });
        }
    }
    if faults.is_empty() {
        Ok(())
    } else {
        Err(ContractErrors::of(faults))
    }
}

/// The release a record states in its head.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRelease {
    /// The record states no release, which is true of every record written before 1.2.0.
    Absent,
    /// The record states this release.
    One(RunbookVersion),
    /// The record states something that is not three dot-separated integers.
    Malformed(String),
    /// The record's head holds more than one release.
    Several,
}

/// The release a record's head states, from its [`RELEASE_META`].
///
/// Only a `meta` in the document's `head` states it, which is where `run --as` writes it. A `meta`
/// of that name in the body is not the record's release: the body is the person's to edit as the
/// run goes, and a release written there would let a sentence's worth of markup change which runs
/// count.
pub(crate) fn release_of(document: &kuchikiki::NodeRef) -> RecordRelease {
    let head = document.descendants().find(|node| {
        node.as_element()
            .is_some_and(|element| &*element.name.local == "head")
    });
    let stated: Vec<String> = head
        .into_iter()
        .flat_map(|head| head.children())
        .filter_map(|node| {
            let element = node.as_element()?;
            if &*element.name.local != "meta" {
                return None;
            }
            let attributes = element.attributes.borrow();
            (attributes.get("name") == Some(RELEASE_META))
                .then(|| attributes.get("content").unwrap_or_default().to_owned())
        })
        .collect();
    match stated.as_slice() {
        [] => RecordRelease::Absent,
        [one] => RunbookVersion::parse(one).map_or_else(
            |_| RecordRelease::Malformed(one.clone()),
            RecordRelease::One,
        ),
        _ => RecordRelease::Several,
    }
}

/// Why one file under a runbook's `runs/` does not satisfy a run check.
///
/// In the order the design's decision 3 reports them, and a record is given only the first that
/// applies: one reason is what an operator acts on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotSatisfying {
    /// The file is a symbolic link, or resolves outside the runbook's `runs/` folder, so the run it
    /// records may have happened in another project. Decided before the file is read.
    IsALink,
    /// The file cannot be read.
    Unreadable(String),
    /// The file parses only through error recovery.
    NeedsRecovery,
    /// The file is not a `tasks` document, which covers an empty or stray file.
    NotATasksDocument,
    /// No task is done, so the record records no work.
    NoTaskDone,
    /// The run is still open.
    StillOpen {
        /// How many tasks are done.
        done: usize,
        /// How many tasks the record holds.
        total: usize,
    },
    /// A requirement's verification records an outcome other than passed.
    OutcomeNotPassed {
        /// The requirement, or the verification where it names none.
        requirement: String,
        /// The outcome recorded.
        outcome: String,
    },
    /// A requirement the record carries has no verification.
    Unrecorded {
        /// The requirement.
        requirement: String,
    },
    /// The record states no release, and the check has a lowest release.
    StatesNoRelease,
    /// The record's release is not three dot-separated integers.
    MalformedRelease(String),
    /// The record's head states more than one release.
    SeveralReleases,
    /// The record ran a release below the check's lowest.
    Below {
        /// The release the record ran.
        release: RunbookVersion,
        /// The lowest release that counts.
        at_least: RunbookVersion,
    },
    /// The record ran a release in another major than the check's lowest.
    OtherMajor {
        /// The release the record ran.
        release: RunbookVersion,
        /// The lowest release that counts.
        at_least: RunbookVersion,
    },
}

impl std::fmt::Display for NotSatisfying {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsALink => f.write_str(
                "is a link, or resolves outside this runbook's runs, so it may record a run made \
                 elsewhere",
            ),
            Self::Unreadable(error) => write!(f, "cannot be read: {}", shown(error)),
            Self::NeedsRecovery => f.write_str(
                "needs error recovery to parse, so it has more than one reading and none is \
                 chosen",
            ),
            Self::NotATasksDocument => f.write_str("is not a run's record: not a tasks document"),
            Self::NoTaskDone => f.write_str("has no task done, so it records no work"),
            Self::StillOpen { done, total } => {
                write!(f, "is still open, with {done} of {total} steps done")
            }
            Self::OutcomeNotPassed {
                requirement,
                outcome,
            } => write!(f, "recorded {} for {}", shown(outcome), shown(requirement)),
            Self::Unrecorded { requirement } => {
                write!(f, "records no outcome for {}", shown(requirement))
            }
            Self::StatesNoRelease => f.write_str("states no release"),
            Self::MalformedRelease(given) => write!(
                f,
                "states a release that is not three dot-separated integers: {}",
                shown(given)
            ),
            Self::SeveralReleases => f.write_str("states more than one release"),
            Self::Below { release, at_least } => {
                write!(f, "ran release {release}, below {at_least}")
            }
            Self::OtherMajor { release, at_least } => write!(
                f,
                "ran release {release}, outside major {} of {at_least}",
                at_least.major()
            ),
        }
    }
}

/// A record that satisfies a run check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Satisfying {
    /// The record's name, without its extension.
    pub record: String,
    /// What it states of its release. A check without `atLeast` does not read the release, so a
    /// record satisfies it whatever this is, and the report still says what it is.
    pub release: RecordRelease,
}

impl std::fmt::Display for Satisfying {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let record = shown(&self.record);
        match &self.release {
            RecordRelease::One(release) => write!(f, "{record} ({release})"),
            RecordRelease::Absent => write!(f, "{record} (no release stated)"),
            RecordRelease::Malformed(given) => write!(
                f,
                "{record} (states a release that is not three dot-separated integers: {})",
                shown(given)
            ),
            RecordRelease::Several => write!(f, "{record} (states more than one release)"),
        }
    }
}

/// A run of the named runbook that is still under way.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenRun {
    /// The record's name, without its extension.
    pub record: String,
    /// How many tasks are done.
    pub done: usize,
    /// How many tasks it holds.
    pub total: usize,
}

impl std::fmt::Display for OpenRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} of {} steps done)",
            shown(&self.record),
            self.done,
            self.total
        )
    }
}

/// Every record of one runbook, decided against one run check.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Records {
    /// Every record that satisfies the check, in name order.
    pub satisfying: Vec<Satisfying>,
    /// Every file that does not, in name order, with its first reason.
    pub unsatisfying: Vec<(String, NotSatisfying)>,
    /// Every record that is still under way, in name order.
    pub open: Vec<OpenRun>,
}

/// Decides every file under `runs` against a run check whose lowest release is `at_least`.
pub(crate) fn read_records(runs: &Path, at_least: Option<RunbookVersion>) -> Records {
    let mut records = Records::default();
    let home = runs.canonicalize().ok();
    for found in crate::runs::gather(runs) {
        let name = match &found {
            crate::runs::Found::Unreadable(file, _) | crate::runs::Found::Readable(file, _) => file,
        };
        // A record that is a link, or resolves outside this runbook's runs, may record a run made in
        // another project, and only this project's runs count. `gather` keeps reading such files,
        // so `run --check` reviews them as 1.1.1 did. Only satisfying a run check refuses them.
        if escapes(runs, name, home.as_deref()) {
            let record = name.strip_suffix(".html").unwrap_or(name).to_owned();
            records.unsatisfying.push((record, NotSatisfying::IsALink));
            continue;
        }
        let (file, verdict, open) = match &found {
            crate::runs::Found::Unreadable(file, error) => (
                file.clone(),
                Err(NotSatisfying::Unreadable(error.clone())),
                None,
            ),
            crate::runs::Found::Readable(file, source) => {
                let (verdict, open) = decide(source, at_least);
                (file.clone(), verdict, open)
            }
        };
        let record = file.strip_suffix(".html").unwrap_or(&file).to_owned();
        if let Some((done, total)) = open {
            records.open.push(OpenRun {
                record: record.clone(),
                done,
                total,
            });
        }
        match verdict {
            Ok(release) => records.satisfying.push(Satisfying { record, release }),
            Err(reason) => records.unsatisfying.push((record, reason)),
        }
    }
    records
}

/// Whether the file `name` under `runs` is a symbolic link, or resolves anywhere but directly inside
/// `runs`, whose canonical form is `home`. A path that cannot be resolved counts as escaping.
fn escapes(runs: &Path, name: &str, home: Option<&Path>) -> bool {
    let path = runs.join(name);
    let linked = std::fs::symlink_metadata(&path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(true);
    let resolved_parent = path
        .canonicalize()
        .ok()
        .and_then(|resolved| resolved.parent().map(Path::to_path_buf));
    linked || home.is_none() || resolved_parent.as_deref() != home
}

/// Whether one record satisfies a run check, with what it states of its release where it does.
type Verdict = Result<RecordRelease, NotSatisfying>;

/// A record that is still under way, as `(done, total)` tasks.
type Progress = Option<(usize, usize)>;

/// One record's verdict, and whether it is still under way.
fn decide(source: &str, at_least: Option<RunbookVersion>) -> (Verdict, Progress) {
    // An empty file is no document at all, so it is not a tasks document, as the design's decision 3
    // says, rather than one whose markup needs recovery: the parser recovers an empty input into a
    // document, which would otherwise name it by the recovery and not by what it is.
    if source.trim().is_empty() {
        return (Err(NotSatisfying::NotATasksDocument), None);
    }
    let parsed = htmlspec_parse::parse_document(source);
    if !parsed.is_recovery_free() {
        return (Err(NotSatisfying::NeedsRecovery), None);
    }
    let model = htmlspec_parse::project(&parsed.document).model;
    if model.kind.as_deref() != Some("tasks") {
        return (Err(NotSatisfying::NotATasksDocument), None);
    }

    let total = model.tasks.len();
    let state = |task: &htmlspec_parse::model::Task| task.state.clone().unwrap_or_default();
    let done = model
        .tasks
        .iter()
        .filter(|task| state(task) == "done")
        .count();
    let settled = model
        .tasks
        .iter()
        .filter(|task| matches!(state(task).as_str(), "done" | "dropped"))
        .count();
    // Open is a run still under way: some task neither done nor dropped. A record whose every task
    // was dropped is finished and did nothing, which is the vacuous case below rather than an open
    // run.
    let open = (settled < total).then_some((done, total));

    if done == 0 {
        return (Err(NotSatisfying::NoTaskDone), open);
    }
    if settled < total {
        return (Err(NotSatisfying::StillOpen { done, total }), open);
    }

    // Every verification passed, so nothing failed, was blocked, was not run, or was only partly
    // checked. Named by the requirement it records, which is what an operator looks for.
    for verification in &model.verifications {
        let outcome = verification.outcome.clone().unwrap_or_default();
        if outcome != "passed" {
            let requirement = verification
                .references
                .first()
                .map(|reference| reference.target.clone())
                .or_else(|| verification.id.clone())
                .unwrap_or_default();
            return (
                Err(NotSatisfying::OutcomeNotPassed {
                    requirement,
                    outcome: if outcome.is_empty() {
                        String::from("no outcome")
                    } else {
                        outcome
                    },
                }),
                open,
            );
        }
    }
    // Every requirement the record carries has a verification, so nothing went unrecorded.
    let recorded: BTreeSet<&str> = model
        .verifications
        .iter()
        .flat_map(|verification| verification.references.iter())
        .map(|reference| reference.target.as_str())
        .collect();
    for requirement in &model.requirements {
        if crate::runs::is_withdrawn(&requirement.change) {
            continue;
        }
        let Some(id) = requirement.id.as_deref() else {
            continue;
        };
        if !recorded.contains(id) {
            return (
                Err(NotSatisfying::Unrecorded {
                    requirement: id.to_owned(),
                }),
                open,
            );
        }
    }

    let release = release_of(&parsed.document);
    let Some(at_least) = at_least else {
        return (Ok(release), open);
    };
    let verdict = match release {
        RecordRelease::Absent => Err(NotSatisfying::StatesNoRelease),
        RecordRelease::Malformed(given) => Err(NotSatisfying::MalformedRelease(given)),
        RecordRelease::Several => Err(NotSatisfying::SeveralReleases),
        RecordRelease::One(release) if release.major() != at_least.major() => {
            Err(NotSatisfying::OtherMajor { release, at_least })
        }
        RecordRelease::One(release) if release < at_least => {
            Err(NotSatisfying::Below { release, at_least })
        }
        RecordRelease::One(release) => Ok(RecordRelease::One(release)),
    };
    (verdict, open)
}

/// Observes one run check, against the runbooks of the project under `runbooks`.
///
/// The five steps of the design's task 2.4, after [`refuse_cycles`] has already refused a cycle
/// through the declaring runbook: the name resolved byte for byte; a name with no match unmet, as
/// missing; a specification reached from it that cannot be read making the check unavailable; and
/// otherwise the records, met or unmet, with any open run named either way.
pub(crate) fn observe(
    runbooks: &Path,
    runbook: &RunbookName,
    at_least: Option<RunbookVersion>,
) -> Result<Fact, ProbeFault> {
    if !names_a_runbook(runbooks, runbook.as_str()) {
        return Ok(Fact::RunbookMissing {
            runbook: runbook.clone(),
        });
    }
    if let Walk::Unreadable {
        runbook: unreadable,
        reason,
    } = walk(runbooks, runbook.as_str(), None)
    {
        let place = if unreadable == runbook.as_str() {
            format!("`{runbook}`")
        } else {
            format!(
                "`{}`, reached through `{runbook}`'s run checks,",
                shown(&unreadable)
            )
        };
        return Err(ProbeFault::Unavailable {
            reason: format!(
                "{place} cannot have its declarations read ({reason}), so whether its run checks \
                 lead back here is not known"
            ),
            remedy: format!(
                "make the specification of `{}` read and validate: run `htmlspec validate` on it",
                shown(&unreadable)
            ),
        });
    }
    let records = read_records(&runbooks.join(runbook.as_str()).join("runs"), at_least);
    if records.satisfying.is_empty() {
        return Ok(Fact::RunUnmet {
            runbook: runbook.clone(),
            records: records.unsatisfying,
            open: records.open,
        });
    }
    let total = records.satisfying.len();
    let mut satisfying = records.satisfying;
    satisfying.truncate(NAMED);
    Ok(Fact::RunMet {
        runbook: runbook.clone(),
        satisfying,
        total,
        open: records.open,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary directory that removes itself.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "htmlspec-prerequisites-{label}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            Self(path)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A specification whose markup needs error recovery is reported as needing it, the fault to
    /// repair first, and not only as one that does not validate.
    #[test]
    fn a_specification_needing_recovery_says_so() {
        let scratch = Scratch::new("recovering-spec");
        let runbooks = scratch.0.join("runbooks");
        std::fs::create_dir_all(runbooks.join("broken")).expect("a runbook");
        std::fs::write(
            runbooks.join("broken").join(crate::runs::CONTRACT),
            "<!doctype html><html><head><title>t</title></head><body><p>a</b></p></body></html>",
        )
        .expect("a specification that needs recovery");
        match declarations_of(&runbooks, "broken") {
            Declarations::Unreadable(reason) => {
                assert!(reason.contains("needs error recovery to parse"), "{reason}");
            }
            other => panic!("not reported as needing recovery: {other:?}"),
        }
    }

    /// A typed name resolves to itself, then to the one name for its directory equal to it
    /// ignoring case, then to the directory's only name, and stays unknown only when the directory
    /// has several names and none matches even ignoring case. So every name 1.1.1 ran still runs.
    #[cfg(unix)]
    #[test]
    fn a_typed_name_resolves_to_the_name_it_means_and_refuses_only_ambiguity() {
        let scratch = Scratch::new("resolve");
        let runbooks = scratch.0.join("runbooks");
        std::fs::create_dir_all(runbooks.join("outside")).expect("a runbook");
        std::fs::create_dir_all(runbooks.join("alone")).expect("a runbook");
        std::os::unix::fs::symlink("outside", runbooks.join("outside-link")).expect("a link");
        let outside = runbooks.join("outside");

        assert_eq!(
            enumerated_name(&outside, "outside").as_deref(),
            Some("outside")
        );
        assert_eq!(
            enumerated_name(&runbooks.join("outside-link"), "outside-link").as_deref(),
            Some("outside-link")
        );
        // A case variant beside a link to the same directory resolves to the name it folds to.
        assert_eq!(
            enumerated_name(&outside, "OUTSIDE").as_deref(),
            Some("outside")
        );
        assert_eq!(
            enumerated_name(&outside, "Outside-LINK").as_deref(),
            Some("outside-link")
        );
        // A directory with one name resolves to it, whatever reached it.
        assert_eq!(
            enumerated_name(&runbooks.join("alone"), "ALONE").as_deref(),
            Some("alone")
        );
        // Several names, none matching even ignoring case: which runbook is meant is not known.
        assert_eq!(enumerated_name(&outside, "elsewhere"), None);
    }

    /// A run's record: its tasks' states, the requirements it carries, one verification per
    /// `(requirement, outcome)`, and the release metas its head holds.
    fn record(
        tasks: &[&str],
        requirements: &[&str],
        verifications: &[(&str, &str)],
        releases: &[&str],
    ) -> String {
        let metas: String = releases
            .iter()
            .map(|release| format!("<meta content=\"{release}\" name=\"{RELEASE_META}\">"))
            .collect();
        let tasks: String = tasks
            .iter()
            .enumerate()
            .map(|(index, state)| {
                format!(
                    "<li class=\"task\" id=\"task-{index}\" data-change=\"unchanged\" \
                     data-state=\"{state}\"><h3>Step {index}</h3></li>"
                )
            })
            .collect();
        let requirements: String = requirements
            .iter()
            .map(|id| {
                format!(
                    "<section class=\"requirement\" id=\"{id}\" data-change=\"unchanged\">\
                     <h3>It</h3><p>A run MUST do it.</p>\
                     <article class=\"scenario\" id=\"scenario-{id}\" data-change=\"unchanged\">\
                     <h4>Done</h4><dl><dt>When</dt><dd>It runs.</dd><dt>Then</dt><dd>It is \
                     done.</dd></dl></article></section>"
                )
            })
            .collect();
        let verifications: String = verifications
            .iter()
            .map(|(requirement, outcome)| {
                format!(
                    "<section class=\"verification\" id=\"verify-{requirement}\" \
                     data-change=\"unchanged\" data-outcome=\"{outcome}\" \
                     data-refs=\"{requirement}\"><h3>Recorded</h3>\
                     <p data-field=\"method\">Looked.</p></section>"
                )
            })
            .collect();
        format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"1.0.0\" name=\"htmlspec-version\">\
             <meta content=\"tasks\" name=\"htmlspec-kind\">{metas}<title>A run</title></head>\
             <body><main><ul>{tasks}</ul>{requirements}{verifications}</main></body></html>"
        )
    }

    /// A finished, passed record of one done task and one passed requirement.
    fn passed(releases: &[&str]) -> String {
        record(&["done"], &["req-one"], &[("req-one", "passed")], releases)
    }

    fn decided(source: &str, at_least: Option<&str>) -> Result<RecordRelease, NotSatisfying> {
        decide(source, at_least.map(version)).0
    }

    fn version(text: &str) -> RunbookVersion {
        RunbookVersion::parse(text).expect("a release")
    }

    #[test]
    fn a_finished_passed_record_satisfies_it() {
        assert_eq!(
            decided(&passed(&["1.4.2"]), None),
            Ok(RecordRelease::One(version("1.4.2")))
        );
        assert_eq!(decided(&passed(&[]), None), Ok(RecordRelease::Absent));
        // A dropped task beside a done one is a finished run.
        let dropped = record(
            &["done", "dropped"],
            &["req-one"],
            &[("req-one", "passed")],
            &[],
        );
        assert_eq!(decided(&dropped, None), Ok(RecordRelease::Absent));
    }

    #[test]
    fn an_open_record_does_not_and_is_named_open() {
        let todo = record(&["done", "todo"], &[], &[], &[]);
        assert_eq!(
            decide(&todo, None),
            (
                Err(NotSatisfying::StillOpen { done: 1, total: 2 }),
                Some((1, 2))
            )
        );
        let in_progress = record(&["done", "in-progress"], &[], &[], &[]);
        assert_eq!(
            decide(&in_progress, None),
            (
                Err(NotSatisfying::StillOpen { done: 1, total: 2 }),
                Some((1, 2))
            )
        );
        // A run just started has no task done, and is open all the same.
        let started = record(&["todo", "todo"], &[], &[], &[]);
        assert_eq!(
            decide(&started, None),
            (Err(NotSatisfying::NoTaskDone), Some((0, 2)))
        );
    }

    #[test]
    fn every_outcome_but_passed_refuses_it() {
        for outcome in ["failed", "blocked", "not-run", "partial"] {
            let source = record(&["done"], &["req-one"], &[("req-one", outcome)], &[]);
            assert_eq!(
                decided(&source, None),
                Err(NotSatisfying::OutcomeNotPassed {
                    requirement: String::from("req-one"),
                    outcome: String::from(outcome),
                }),
                "{outcome}"
            );
        }
    }

    #[test]
    fn a_requirement_with_no_verification_refuses_it() {
        let source = record(
            &["done"],
            &["req-one", "req-two"],
            &[("req-one", "passed")],
            &[],
        );
        assert_eq!(
            decided(&source, None),
            Err(NotSatisfying::Unrecorded {
                requirement: String::from("req-two"),
            })
        );
    }

    #[test]
    fn a_record_of_no_work_is_vacuous() {
        let dropped = record(&["dropped", "dropped"], &[], &[], &[]);
        assert_eq!(
            decide(&dropped, None),
            (Err(NotSatisfying::NoTaskDone), None)
        );
        let empty = record(&[], &[], &[], &[]);
        assert_eq!(decide(&empty, None), (Err(NotSatisfying::NoTaskDone), None));
    }

    #[test]
    fn a_file_that_is_not_a_tasks_document_is_not_a_record() {
        assert_eq!(decided("", None), Err(NotSatisfying::NotATasksDocument));
        let stray = passed(&[]).replace(
            "content=\"tasks\" name=\"htmlspec-kind\"",
            "content=\"design\" name=\"htmlspec-kind\"",
        );
        assert_eq!(decided(&stray, None), Err(NotSatisfying::NotATasksDocument));
    }

    #[test]
    fn a_record_needing_recovery_is_refused() {
        let recovering = passed(&[]).replace("</main>", "</section></main>");
        assert_eq!(
            decided(&recovering, None),
            Err(NotSatisfying::NeedsRecovery)
        );
    }

    #[test]
    fn at_least_compares_within_one_major() {
        let at = Some("1.4.0");
        assert_eq!(
            decided(&passed(&["1.4.0"]), at),
            Ok(RecordRelease::One(version("1.4.0")))
        );
        assert_eq!(
            decided(&passed(&["1.9.2"]), at),
            Ok(RecordRelease::One(version("1.9.2")))
        );
        assert_eq!(
            decided(&passed(&["1.3.0"]), at),
            Err(NotSatisfying::Below {
                release: version("1.3.0"),
                at_least: version("1.4.0"),
            })
        );
        assert_eq!(
            decided(&passed(&["2.0.0"]), at),
            Err(NotSatisfying::OtherMajor {
                release: version("2.0.0"),
                at_least: version("1.4.0"),
            })
        );
        assert_eq!(
            decided(&passed(&[]), at),
            Err(NotSatisfying::StatesNoRelease)
        );
        assert_eq!(
            decided(&passed(&["1.4"]), at),
            Err(NotSatisfying::MalformedRelease(String::from("1.4")))
        );
        assert_eq!(
            decided(&passed(&["1.4.0", "1.5.0"]), at),
            Err(NotSatisfying::SeveralReleases)
        );
        // Without `atLeast` the release is not read, so a record stating none still counts.
        assert_eq!(
            decided(&passed(&["not a release"]), None),
            Ok(RecordRelease::Malformed(String::from("not a release")))
        );
    }

    /// A runs directory holding `files`, as `(name, source)`.
    fn runs(scratch: &Scratch, files: &[(&str, String)]) -> std::path::PathBuf {
        let runs = scratch.0.join("runs");
        std::fs::create_dir_all(&runs).expect("the runs directory exists");
        for (name, source) in files {
            std::fs::write(runs.join(name), source).expect("the record is written");
        }
        runs
    }

    #[test]
    fn three_satisfying_records_are_named_in_name_order_and_every_one_counted() {
        let scratch = Scratch::new("five");
        let files: Vec<(&str, String)> = ["e.html", "a.html", "d.html", "b.html", "c.html"]
            .into_iter()
            .map(|name| (name, passed(&["1.0.0"])))
            .collect();
        let runbooks = scratch.0.join("runbooks");
        let backup = runbooks.join("backup");
        std::fs::create_dir_all(&backup).expect("the runbook exists");
        std::fs::rename(runs(&scratch, &files), backup.join("runs")).expect("moved");
        std::fs::write(backup.join(crate::runs::CONTRACT), specification(&[]))
            .expect("the specification is written");

        let fact = observe(
            &runbooks,
            &RunbookName::new("backup").expect("a name"),
            None,
        )
        .expect("observed");
        let Fact::RunMet {
            satisfying, total, ..
        } = &fact
        else {
            panic!("not met: {fact:?}");
        };
        assert_eq!(*total, 5);
        let named: Vec<&str> = satisfying
            .iter()
            .map(|record| record.record.as_str())
            .collect();
        assert_eq!(named, ["a", "b", "c"]);
        assert!(
            fact.observed().contains("and 2 more"),
            "{}",
            fact.observed()
        );
    }

    #[test]
    fn a_met_check_beside_an_open_run_names_the_open_one() {
        let scratch = Scratch::new("open-beside");
        let records = read_records(
            &runs(
                &scratch,
                &[
                    ("old.html", passed(&[])),
                    ("new.html", record(&["done", "todo", "todo"], &[], &[], &[])),
                ],
            ),
            None,
        );
        assert_eq!(records.satisfying.len(), 1);
        assert_eq!(
            records.open,
            [OpenRun {
                record: String::from("new"),
                done: 1,
                total: 3,
            }]
        );
    }

    #[test]
    fn no_records_and_stray_files_are_reported_as_they_are() {
        let scratch = Scratch::new("none");
        let none = read_records(&scratch.0.join("absent"), None);
        assert!(none.satisfying.is_empty() && none.unsatisfying.is_empty());

        let records = read_records(
            &runs(
                &scratch,
                &[
                    ("empty.html", String::new()),
                    ("notes.txt", String::from("x")),
                ],
            ),
            None,
        );
        assert_eq!(
            records.unsatisfying,
            [(String::from("empty"), NotSatisfying::NotATasksDocument)],
            "only `.html` files are records, and an empty one is not a tasks document"
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_is_named_as_unreadable() {
        use std::os::unix::fs::PermissionsExt;
        let scratch = Scratch::new("unreadable");
        let directory = runs(&scratch, &[("locked.html", passed(&[]))]);
        let path = directory.join("locked.html");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000))
            .expect("the record is locked");
        let unreadable = std::fs::read_to_string(&path).is_err();
        let records = read_records(&directory, None);
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));
        // A user who can read anything reads it anyway, and there is nothing to assert then.
        if unreadable {
            assert!(
                matches!(
                    records.unsatisfying.as_slice(),
                    [(name, NotSatisfying::Unreadable(_))] if name == "locked"
                ),
                "{records:?}"
            );
        }
    }

    #[test]
    fn the_first_reason_is_the_one_named() {
        // Open and failed at once: the run being open is named, because it comes first.
        let source = record(
            &["done", "todo"],
            &["req-one"],
            &[("req-one", "failed")],
            &[],
        );
        assert_eq!(
            decided(&source, None),
            Err(NotSatisfying::StillOpen { done: 1, total: 2 })
        );
    }

    #[test]
    fn release_of_reads_the_head_meta() {
        let read = |releases: &[&str]| {
            release_of(&htmlspec_parse::parse_document(&passed(releases)).document)
        };
        assert_eq!(read(&[]), RecordRelease::Absent);
        assert_eq!(read(&["1.2.3"]), RecordRelease::One(version("1.2.3")));
        assert_eq!(read(&["x"]), RecordRelease::Malformed(String::from("x")));
        assert_eq!(read(&["1.0.0", "1.0.0"]), RecordRelease::Several);
    }

    /// A release meta in the body is not the record's release: alone it states none, and beside
    /// the head's it neither replaces it nor makes two.
    #[test]
    fn a_release_meta_outside_the_head_does_not_count() {
        let in_body = |head: &[&str]| {
            passed(head).replace(
                "<main>",
                &format!("<main><meta content=\"9.0.0\" name=\"{RELEASE_META}\">"),
            )
        };
        let read = |source: &str| release_of(&htmlspec_parse::parse_document(source).document);
        assert!(in_body(&[]).contains("<main><meta content=\"9.0.0\""));
        assert_eq!(read(&in_body(&[])), RecordRelease::Absent);
        assert_eq!(
            read(&in_body(&["0.2.0"])),
            RecordRelease::One(version("0.2.0"))
        );
        assert_eq!(
            decided(&in_body(&[]), Some("9.0.0")),
            Err(NotSatisfying::StatesNoRelease)
        );
    }

    /// Without `atLeast` the release is not read for the verdict, and a record satisfies the check
    /// whatever it states, but what it states is still named as it is, never as no release.
    #[test]
    fn a_satisfying_record_names_the_release_it_states_as_it_states_it() {
        let named = |release: RecordRelease| {
            Satisfying {
                record: String::from("b1"),
                release,
            }
            .to_string()
        };
        assert_eq!(named(RecordRelease::One(version("0.1.0"))), "b1 (0.1.0)");
        assert_eq!(named(RecordRelease::Absent), "b1 (no release stated)");
        assert_eq!(
            named(RecordRelease::Malformed(String::from("x"))),
            "b1 (states a release that is not three dot-separated integers: x)"
        );
        assert_eq!(
            named(RecordRelease::Several),
            "b1 (states more than one release)"
        );
        assert_eq!(
            decided(&passed(&["1.0.0", "2.0.0"]), None),
            Ok(RecordRelease::Several)
        );
    }

    #[test]
    fn a_reason_names_text_from_a_record_by_code_point() {
        let reason = NotSatisfying::OutcomeNotPassed {
            requirement: String::from("req-\u{1b}[2J"),
            outcome: String::from("failed"),
        };
        let text = reason.to_string();
        assert!(!text.contains('\u{1b}'), "{text:?}");
        assert!(text.contains("<U+001B>"), "{text:?}");
    }

    /// A runbook specification whose requirements each declare a run check of the named runbook.
    fn specification(requires: &[&str]) -> String {
        let units: String = requires
            .iter()
            .enumerate()
            .map(|(index, name)| {
                format!(
                    "<section class=\"requirement\" id=\"req-after-{index}\" \
                     data-change=\"unchanged\"><h2>After {index}</h2>\
                     <p>A run MUST follow it.</p>\
                     <article class=\"scenario\" id=\"scenario-after-{index}\" \
                     data-change=\"unchanged\"><h3>It ran</h3><dl><dt>When</dt><dd>A run \
                     starts.</dd><dt>Then</dt><dd>It ran first.</dd></dl></article>\
                     <p data-field=\"note\">preflight/v3 \
                     {{&quot;kind&quot;:&quot;run&quot;,&quot;runbook&quot;:&quot;{name}&quot;}}\
                     </p></section>"
                )
            })
            .collect();
        format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"1.0.0\" name=\"htmlspec-version\">\
             <meta content=\"specification\" name=\"htmlspec-kind\">\
             <title>A runbook</title></head>\
             <body><main><header class=\"hs-masthead\"><h1>A runbook</h1>\
             <p>What a run of this runbook must record.</p></header>{units}</main></body></html>"
        )
    }

    /// A runbooks directory where each `(name, requires)` declares run checks of `requires`.
    fn project(scratch: &Scratch, runbooks: &[(&str, &[&str])]) -> std::path::PathBuf {
        let root = scratch.0.join("runbooks");
        for (name, requires) in runbooks {
            let directory = root.join(name);
            std::fs::create_dir_all(&directory).expect("the runbook exists");
            std::fs::write(
                directory.join(crate::runs::CONTRACT),
                specification(requires),
            )
            .expect("the specification is written");
        }
        root
    }

    #[test]
    fn a_name_matches_a_runbook_byte_for_byte_only() {
        let scratch = Scratch::new("bytes");
        let root = project(&scratch, &[("backup", &[])]);
        assert!(names_a_runbook(&root, "backup"));
        assert!(!names_a_runbook(&root, "Backup"));
        assert!(!names_a_runbook(&root, "backup "));
    }

    #[test]
    fn a_walk_finds_a_chain_back_and_ignores_a_cycle_it_does_not_reach() {
        let scratch = Scratch::new("walk");
        let root = project(
            &scratch,
            &[
                ("a", &["b"]),
                ("b", &["c"]),
                ("c", &["a"]),
                ("x", &["y"]),
                ("y", &["x"]),
                ("backup", &[]),
                ("migrate", &["backup"]),
            ],
        );
        assert_eq!(
            walk(&root, "b", Some("a")),
            Walk::Leads(vec![
                String::from("b"),
                String::from("c"),
                String::from("a")
            ])
        );
        assert_eq!(walk(&root, "backup", Some("migrate")), Walk::Clear);
        assert_eq!(walk(&root, "backup", None), Walk::Clear);
    }

    /// An unreadable specification is reported when nothing else is known, and a chain back that
    /// the walk can still see wins over it, even when the unreadable one is met first: `hub`
    /// declares `helper`, which cannot be read, before `second`, which leads back to `migrate`.
    #[test]
    fn an_unreadable_specification_on_the_way_is_reported_and_a_cycle_still_wins() {
        let scratch = Scratch::new("unreadable-walk");
        let root = project(
            &scratch,
            &[
                ("backup", &["helper"]),
                ("helper", &[]),
                ("hub", &["helper", "second"]),
                ("second", &["migrate"]),
                ("migrate", &["hub"]),
            ],
        );
        std::fs::write(
            root.join("helper").join(crate::runs::CONTRACT),
            "<p>not a specification",
        )
        .expect("broken");
        assert!(matches!(
            walk(&root, "backup", None),
            Walk::Unreadable { runbook, .. } if runbook == "helper"
        ));
        assert!(matches!(
            walk(&root, "backup", Some("migrate")),
            Walk::Unreadable { runbook, .. } if runbook == "helper"
        ));
        assert_eq!(
            walk(&root, "hub", Some("migrate")),
            Walk::Leads(vec![
                String::from("hub"),
                String::from("second"),
                String::from("migrate")
            ])
        );

        // The same through the refusal `load` applies: the cycle is refused, not left unavailable.
        let source = std::fs::read_to_string(root.join("migrate").join(crate::runs::CONTRACT))
            .expect("the specification reads");
        let parsed = htmlspec_parse::parse_document(&source);
        let contract =
            crate::preflight::parse_contract(&htmlspec_parse::project(&parsed.document).model)
                .expect("the contract reads");
        let refused = refuse_cycles(&root, "migrate", &contract).expect_err("the cycle is refused");
        assert!(
            refused.to_string().contains(
                "`migrate` requires a run of `hub` requires a run of `second` requires a run of \
                 `migrate`"
            ),
            "{refused}"
        );
    }
}
