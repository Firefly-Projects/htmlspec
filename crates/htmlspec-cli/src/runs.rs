//! Starting a run of a runbook, and checking what its runs recorded.
//!
//! A run's record is a `tasks` document under `htmlspec/runbooks/<name>/runs/`, carrying the
//! runbook's procedure with its states advanced, plus the requirements it records outcomes for,
//! copied from the runbook's specification. Every part of that shape is forced by a measurement
//! rather than chosen, and the measurements are in
//! `openspec/changes/log-a-runbook-run/evidence/the-shape-is-forced-2026-09-18.md`:
//!
//! - **The kind is `tasks`.** The same body declared `specification` draws
//!   `kind/task-in-specification` once per copied step — two steps, two errors, measured.
//! - **The requirements are copied rather than referenced.** A verification in the record naming a
//!   requirement in the runbook's `specification.html` fails `refs/unresolved`, an *error*, because
//!   a reference from a runbook's documents resolves within the document that makes it: only a
//!   change's documents and the accepted specifications reach further. A record that pointed
//!   outward would be invalid on the day it was written.
//! - **Each copied requirement keeps its scenarios, its modal, and `data-change="unchanged"`.** The
//!   first two are schema errors without them (`/requirements/0/scenarios` has less than 1 item;
//!   `schema/weak-modal`) and the third is a `review/stale-verification` warning, each measured from
//!   its own single edit.
//!
//! **The copy is a DOM clone rather than a re-render from the projected model, and that is the one
//! substantive implementation decision here.** A runbook's prose is full of `<code>`, `<strong>`,
//! and `<q>` — the release runbook's four task notes carry 30-odd inline elements between them —
//! and the projection flattens a unit's text to strings. Rendering from the model would produce a
//! record whose procedure reads as a worse version of the procedure that ran, which defeats the
//! point of carrying the steps: the record exists to say *which version of this procedure ran*.
//! Cloning the subtree preserves it byte for byte, and `htmlspec_format::format` then makes the
//! result canonical.

use std::io::Write;
use std::path::{Path, PathBuf};

use htmlspec_validate::rules::runs::{RecordedRun, RunbookUnderReview};

use crate::exit::Outcome;
use crate::output::Streams;
use crate::preflight::MachineContract;
use crate::project::ProjectRoot;
use crate::version::RunbookVersion;

/// The document holding a runbook's procedure.
///
/// Crate-visible because the release fingerprint is of exactly these two documents and nothing else:
/// a run's record is history rather than procedure, so it must not move a fingerprint, and the only
/// way to keep that true is for both halves to name the documents through one constant.
pub(crate) const PROCEDURE: &str = "tasks.html";

/// The document holding what a runbook is for and what its runs must record.
///
/// Crate-visible for the reason [`PROCEDURE`] is.
pub(crate) const CONTRACT: &str = "specification.html";

/// The extension a run's record is written with.
const RECORD_SUFFIX: &str = ".html";

/// Starts a run of `runbook`, writing its record under `--as`.
///
/// **Eagerly: the whole procedure is copied at the moment the run starts.** A record grown as the
/// run proceeded would, between two steps, present itself as a complete procedure — nothing in a
/// list of steps says there are more — so a reader would see a version of the procedure that never
/// existed. `runbook-execution` states this as an obligation.
///
/// Refuses an existing record. A run continued is that record *edited*, which
/// `runbook-execution` settles: a run interrupted and picked up later is one piece of work, and two
/// records would each be wrong about it. So this operation starts a run and never resumes one, and
/// overwriting would destroy the history the record exists to hold.
/// Starts a run of `runbook`, writing its record under `--as`.
///
/// **Eagerly: the whole procedure is copied at the moment the run starts.** A record grown as the
/// run proceeded would, between two steps, present itself as a complete procedure — nothing in a
/// list of steps says there are more — so a reader would see a version of the procedure that never
/// existed. `runbook-execution` states this as an obligation.
///
/// **Gated on this machine, before anything is created.** `runbook-portability` requires starting a
/// run to perform preflight against the invocation's current working directory and environment
/// before creating the record or its directory, and requires that a standalone preflight success
/// cannot be reused as permission to skip that. Both are structural here rather than remembered: the
/// only value [`publish_run`] accepts is a [`ReadyRun`], the only way to hold one is
/// [`prepare_run`], and [`prepare_run`] observes. There is no token, no cache, and no
/// `--skip-preflight`.
///
/// Refuses an existing record. A run continued is that record *edited*, which
/// `runbook-execution` settles: a run interrupted and picked up later is one piece of work, and two
/// records would each be wrong about it. So this operation starts a run and never resumes one, and
/// overwriting would destroy the history the record exists to hold.
pub fn start<O: Write, E: Write>(
    runbook: &str,
    record: &str,
    store: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let project = match crate::store::project_for(store) {
        Ok(project) => project,
        Err(report) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };

    // Both names through the constructor that refuses a path wearing a name. The runbook's name
    // reaches `ProjectRoot::runbook`, which composes unconditionally, and the record's name reaches
    // a `join` — so without this, `--as ../../../sneaky` would write a document outside the
    // project. `AGENTS.md` §2: the invariant lives in the constructor rather than in a caller's
    // memory, which is where the previous change put it after re-litigating it three times.
    for (label, name) in [("runbook", runbook), ("record", record)] {
        if !ProjectRoot::is_a_name(name) {
            let _ = streams.failure(&format!(
                "{name:?} is a path rather than a {label} name. A runbook lives where the project \
                 keeps runbooks and its records live beneath it, so a name carrying `/` or `..` \
                 would put the record somewhere the project does not keep one. Pass the name alone"
            ));
            return Outcome::OperationalFailure;
        }
    }

    let directory = project.runbook(runbook);
    if !directory.is_dir() {
        let _ = streams.failure(&format!(
            "no runbook named {runbook} in {}. To see what the project holds, run \
             `htmlspec list --runbooks`",
            project.runbooks().display()
        ));
        return Outcome::OperationalFailure;
    }

    // Unresolved -> Validated. Both canonical documents, the version, and the changelog are read and
    // validated here, before a single machine observation is collected, and the snapshot is what
    // every later stage composes from.
    let source = match load(&directory, runbook) {
        Ok(source) => source,
        Err(fault) => {
            let _ = streams.failure(&fault.to_string());
            return Outcome::OperationalFailure;
        }
    };

    // Validated -> Ready. The checks run against the invocation's working directory, which is *not*
    // necessarily the store's: `--store` selects documents and not the tree a state check judges.
    let workdir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut host = crate::preflight::host::Host::interruptible(workdir.clone(), streams);
    // Taken before the snapshot is consumed, because the refusal has to report the runbook it refused
    // and the version it carries, and both are facts about the snapshot.
    let context = crate::preflight::ReportContext {
        runbook: source.name.clone(),
        version: source.version,
        source: source.directory.clone(),
        workdir: workdir.clone(),
        contract: source.contract.clone(),
    };
    let run = match prepare_run(source, &mut host) {
        Ok(run) => run,
        Err(StartFailure::Cancelled) => {
            let _ = streams.failure(
                "the run was cancelled while its prerequisites were being checked. Every probe was \
                 stopped and reaped, and no record was created",
            );
            return Outcome::Cancelled;
        }
        Err(StartFailure::NotReady(findings)) => {
            // Named before the findings, because the finding an operator reads first is the one they
            // act on and "no record was created" is the half that is easy to miss.
            let report = context.report(crate::preflight::PreflightAssessment::NotReady(findings));
            crate::preflight::report_to(streams, &report);
            // Last rather than first in human output, because `failure` writes to standard error and
            // the report above is what an operator reads; in structured mode this is the only thing
            // on standard error besides the payload, and it is the half that is easy to miss.
            let _ = streams.failure(
                "no record was created and nothing was written, because this machine does not \
                 satisfy every prerequisite the runbook declares",
            );
            return Outcome::PrerequisiteFailure;
        }
    };

    // Ready -> Prepared -> Started. The destination is resolved as vacant first so a collision is
    // reported as one, and publication then makes it true rather than hoping.
    let destination = match VacantRecord::at(&project, runbook, record) {
        Ok(destination) => destination,
        Err(report) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };
    let started = match publish_run(run, destination) {
        Ok(started) => started,
        Err(error) => {
            // Every publication failure is an operational one, including a collision: nothing was
            // determined about the documents, and a refused write is not a document violation.
            let _ = streams.failure(&error.to_string());
            return Outcome::OperationalFailure;
        }
    };

    if streams.form().is_structured() {
        let payload = serde_json::json!({
            "runbook": runbook,
            "record": record,
            "path": started.path,
            "steps": started.steps,
            "recording": started.requirements,
            // Added rather than replacing anything: `toolchain-packaging` requires the existing
            // fields on successful run creation to be preserved.
            "version": started.version.to_string(),
            "preflight": {
                "checked": started.checked,
                "passed": started.passed,
            },
        });
        if let Ok(json) = serde_json::to_string_pretty(&payload) {
            let _ = streams.payload(&json);
        }
    } else {
        let _ = streams.report(&format!(
            "{}: started a run of {runbook} {}\n  {} step(s) to work, {} requirement(s) to \
             record, {} of {} machine check(s) passed before it started\n",
            started.path.display(),
            started.version,
            started.steps,
            started.requirements,
            started.passed,
            started.checked
        ));
        if let Some(leftover) = &started.leftover {
            let _ = streams.progress(leftover);
        }
    }
    Outcome::Success
}

/// A runbook's two documents, its release metadata, and its machine contract — read once.
///
/// **A snapshot rather than a directory**, and the ordering is the reason. The documents are read
/// before any observation is collected, so an edit made while preflight runs cannot change which
/// procedure receives the passed evidence: a [`ReadyRun`] holding a path would compose from whatever
/// the directory held at publication time, and the evidence in the record would then attest to a
/// document nobody checked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedRunbook {
    /// The runbook's name, which is its directory name.
    pub name: String,
    /// Where its documents were read from.
    pub directory: PathBuf,
    /// The procedure's source, as read.
    pub procedure: String,
    /// The specification's source, as read.
    pub specification: String,
    /// The changelog, as read.
    pub changelog: String,
    /// The release version the runbook declares.
    pub version: RunbookVersion,
    /// The machine checks its specification declares.
    pub contract: MachineContract,
}

/// Why a runbook could not be loaded for a new start.
///
/// **Absent and unreadable are separate variants**, and the distinction is the one the design asks
/// for: an absent specification used to be read as an empty one by a `read_to_string(…).ok()`, which
/// made a runbook whose contract could not be read indistinguishable from one that declared nothing.
/// A machine contract bypassed silently is the failure this exists to prevent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadFault {
    /// A document or metadata file a new start requires is not there.
    Absent {
        /// What to do about it.
        report: String,
    },
    /// It is there and could not be read.
    Unreadable {
        /// What happened, naming the file.
        report: String,
    },
    /// It was read and is not usable: it does not validate, or its declaration cannot be read.
    Invalid {
        /// What is wrong with it.
        report: String,
    },
}

impl std::fmt::Display for LoadFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absent { report } | Self::Unreadable { report } | Self::Invalid { report } => {
                f.write_str(report)
            }
        }
    }
}

impl std::error::Error for LoadFault {}

/// Reads and validates everything a new start composes from.
///
/// Refuses a missing specification, version, or changelog, which is a change from what a run used to
/// require and is deliberate: `runbook-authority` makes the version the only way a carried runbook
/// can name its release, and a start that proceeded without one would produce a record that cannot
/// say which release ran. **Old record review is untouched** — [`check`] reads neither file, so a
/// record created before this obligation stays readable, which is what the same requirement asks
/// for.
pub fn load(directory: &Path, name: &str) -> Result<ValidatedRunbook, LoadFault> {
    let absent = |file: &str, guidance: &str| LoadFault::Absent {
        report: format!("{name} has no {file}, and starting a run needs it. {guidance}"),
    };
    let procedure = read_document(directory, PROCEDURE, &absent(
        PROCEDURE,
        "A run copies the runbook's procedure, so a runbook with no procedure has nothing to run",
    ))?;
    let specification = read_document(
        directory,
        CONTRACT,
        &absent(
            CONTRACT,
            "A runbook's specification is where it states what its runs must record and what they \
             need of a machine. A runbook predating that obligation can be given an empty one, but \
             it cannot be started without it: an absent specification read as an empty one is how a \
             machine contract gets bypassed silently",
        ),
    )?;
    let changelog = read_document(directory, crate::runbook_release::CHANGELOG_FILE, &absent(
        crate::runbook_release::CHANGELOG_FILE,
        &format!(
            "A carried runbook identifies its release by its {} and the heading naming that version \
             in it, because the archive that would otherwise say where it came from did not travel \
             with it",
            crate::runbook_release::VERSION_FILE
        ),
    ))?;
    let version =
        crate::runbook_release::read_version(directory).map_err(|problem| LoadFault::Absent {
            report: format!("{name} {problem}"),
        })?;

    // The changelog has to head the version declared, and heading it twice cannot say which entry is
    // the release. Both are checked here rather than only at publication, because a run started
    // against a version its own changelog does not name is a record nobody can place.
    let headings = crate::runbook_release::changelog_headings(&changelog);
    if !headings.contains(&version) {
        return Err(LoadFault::Invalid {
            report: format!(
                "{name} declares {version} and its {} has no heading naming it. A carried runbook \
                 identifies its release by the pair, so add the heading or correct the version",
                crate::runbook_release::CHANGELOG_FILE
            ),
        });
    }
    let repeated = crate::runbook_release::duplicate_headings(&changelog);
    if let Some(repeated) = repeated.first() {
        return Err(LoadFault::Invalid {
            report: format!(
                "{name}'s {} heads {repeated} more than once, so it cannot say which entry is the \
                 release. One heading per release",
                crate::runbook_release::CHANGELOG_FILE
            ),
        });
    }

    // Both documents validated, not merely parsed. A document that reports an error is one whose
    // units cannot be trusted to copy, and the record composed from it would be invalid anyway —
    // refused here so the report names the runbook rather than the record.
    for (file, source) in [(PROCEDURE, &procedure), (CONTRACT, &specification)] {
        let report = crate::commands::validate_source_at(source, &directory.join(file));
        if !report.is_accepted() {
            return Err(LoadFault::Invalid {
                report: format!(
                    "{name}'s {file} does not validate, so a run cannot start from it:\n{}",
                    htmlspec_validate::render_human(&report)
                ),
            });
        }
    }

    let parsed = htmlspec_parse::parse_document(&specification);
    if !parsed.is_recovery_free() {
        return Err(LoadFault::Invalid {
            report: format!(
                "{name}'s {CONTRACT} requires error recovery, so it has more than one reading and a \
                 copy would pick one. Run `htmlspec validate` on the runbook first"
            ),
        });
    }
    let projection = htmlspec_parse::project(&parsed.document);
    let contract = crate::preflight::parse_contract(&projection.model).map_err(|faults| {
        LoadFault::Invalid {
            report: format!(
                "{name}'s {CONTRACT} declares a machine check this program cannot read, so it \
                 cannot be started — a declaration that was skipped would be a prerequisite \
                 reported as checked:\n{faults}"
            ),
        }
    })?;

    Ok(ValidatedRunbook {
        name: name.to_owned(),
        directory: directory.to_owned(),
        procedure,
        specification,
        changelog,
        version,
        contract,
    })
}

/// Reads one of a runbook's files, keeping absence and unreadability apart.
fn read_document(directory: &Path, file: &str, absent: &LoadFault) -> Result<String, LoadFault> {
    let path = directory.join(file);
    match std::fs::read_to_string(&path) {
        Ok(source) => Ok(source),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(absent.clone()),
        Err(error) => Err(LoadFault::Unreadable {
            report: format!(
                "{} cannot be read: {error}. It is present, so this is not a runbook missing a \
                 document — it is one this operation cannot see, and it will not be read as empty",
                path.display()
            ),
        }),
    }
}

/// Why a run could not be prepared.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StartFailure {
    /// Cancellation reached a probe, and the probes were stopped and reaped.
    Cancelled,
    /// The machine does not satisfy every declared check.
    NotReady(crate::preflight::NonEmptyFindings),
}

/// A run whose prerequisites were observed and passed, holding the snapshot it will be composed
/// from.
///
/// **Its constructor is private and it is consumed on publication.** Those two facts are the whole
/// of the gate: the only way to hold a `ReadyRun` is [`prepare_run`], which observes the machine
/// afresh, and the only way to spend one is [`publish_run`], which takes it by value so it cannot be
/// spent twice. `runbook-portability` requires that a standalone readiness report cannot be reused
/// as permission to skip the check, and a report is not a `ReadyRun` — there is no conversion, no
/// token, and nothing persisted that could become one.
pub struct ReadyRun {
    source: ValidatedRunbook,
    passed: crate::preflight::PassedChecks,
}

impl ReadyRun {
    /// The runbook this run is of.
    pub fn name(&self) -> &str {
        &self.source.name
    }

    /// The release version the runbook declares, which the record carries.
    pub fn version(&self) -> RunbookVersion {
        self.source.version
    }

    /// The machine checks that passed, with the observations that show it.
    pub fn passed(&self) -> &crate::preflight::PassedChecks {
        &self.passed
    }
}

/// Observes every machine check a validated runbook declares, against this machine.
///
/// The Validated -> Ready transition, and the only path to a [`ReadyRun`]. Takes the snapshot by
/// value so the evidence it collects is bound to the documents that will be composed: the design's
/// invariant is "all facts bound to this snapshot", and a signature taking a reference would leave
/// the binding to the caller's good behaviour.
pub fn prepare_run(
    source: ValidatedRunbook,
    host: &mut impl crate::preflight::HostProbe,
) -> Result<ReadyRun, StartFailure> {
    let observations = crate::preflight::host::observe_all(&source.contract, host)
        .map_err(|_interrupted| StartFailure::Cancelled)?;
    match crate::preflight::assess(&source.contract, observations) {
        crate::preflight::PreflightAssessment::Ready(passed) => Ok(ReadyRun { source, passed }),
        crate::preflight::PreflightAssessment::NotReady(findings) => {
            Err(StartFailure::NotReady(findings))
        }
    }
}

/// A record path known to be vacant, with what composing into it needs.
///
/// Built only through [`VacantRecord::at`], which asks the filesystem whether the path is free. The
/// answer is not a guarantee — another start can claim it a moment later — so publication is
/// exclusive in its own right and this is the early, well-worded refusal.
pub struct VacantRecord {
    /// Where the record will be written.
    pub path: PathBuf,
    /// The directory holding it, created on publication if publication creates it.
    pub runs: PathBuf,
    /// The record's name.
    pub record: String,
    /// The asset prefix the record's references use.
    pub assets: String,
}

impl VacantRecord {
    /// Resolves a record's destination inside a project's runbook, refusing an occupied one.
    pub fn at(project: &ProjectRoot, runbook: &str, record: &str) -> Result<Self, String> {
        if !ProjectRoot::is_a_name(record) {
            return Err(format!(
                "{record:?} is a path rather than a record name. A run's record lives beneath the \
                 runbook, so a name carrying `/` or `..` would put it somewhere the project does \
                 not keep one. Pass the name alone"
            ));
        }
        let runs = project.runs(runbook);
        let path = runs.join(format!("{record}{RECORD_SUFFIX}"));
        nothing_is_at(&path)?;
        Ok(Self {
            assets: crate::commands::assets_prefix_for(&path, project),
            path,
            runs,
            record: record.to_owned(),
        })
    }
}

/// Why a prepared run could not be published.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublishError {
    /// The record could not be composed from the snapshot.
    Uncomposable {
        /// What went wrong.
        report: String,
    },
    /// The composed record does not validate, so it was not written.
    Invalid {
        /// The validator's report.
        report: String,
    },
    /// Another record claimed the destination between the vacancy check and the write.
    Occupied {
        /// What is there.
        report: String,
    },
    /// The filesystem refused.
    Io {
        /// What happened, and what state the destination is in.
        report: String,
    },
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uncomposable { report }
            | Self::Invalid { report }
            | Self::Occupied { report }
            | Self::Io { report } => f.write_str(report),
        }
    }
}

impl std::error::Error for PublishError {}

/// What a published run produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartedRun {
    /// Where the record is.
    pub path: PathBuf,
    /// How many steps it carries.
    pub steps: usize,
    /// How many requirements it records outcomes for.
    pub requirements: usize,
    /// How many machine checks the runbook declares.
    pub checked: usize,
    /// How many of them passed before the run started.
    pub passed: usize,
    /// The release version the record carries.
    pub version: RunbookVersion,
    /// A temporary file that could not be removed, as a diagnostic for the operator.
    ///
    /// `None` in the ordinary case. Present does not mean the publication failed: the record is
    /// complete at its final path, and this is the tidying that did not happen.
    pub leftover: Option<String>,
}

/// Composes, validates, and publishes a prepared run's record.
///
/// The Prepared -> Started transition, and the ordering inside it is the invariant: **the record is
/// complete and valid, or it is absent.** A fully written temporary file is linked into place, and a
/// link is the one operation that both creates a name and refuses to replace what is already there —
/// so a successful publication cannot have overwritten another record, and a failed one leaves the
/// destination exactly as it was.
///
/// A rename would not do this. `std::fs::rename` replaces its destination, which is the right
/// behaviour for a move and the wrong one here: two concurrent starts of the same record name would
/// each believe it had written the record, and the second would destroy the first.
pub fn publish_run(run: ReadyRun, destination: VacantRecord) -> Result<StartedRun, PublishError> {
    let document =
        compose(&run, &destination).map_err(|report| PublishError::Uncomposable { report })?;

    // **Validated before it is written, and the order is the fix rather than a tidying.**
    // `write_document` validates after writing and leaves the file in place, which is defensible
    // there: the fault is in a template this program ships, so the contributor can do nothing but
    // look at it. Here the fault is in the **runbook** in every case measured — a requirement with no
    // identifier, two identifiers colliding, markup a requirement's own identifier injected — and
    // that is something the contributor can fix. So a record that does not validate must never reach
    // disk.
    //
    // Three defects collapse into this ordering. An invalid record on disk made the vacancy refusal
    // fire on the retry, wedging the run with no route back through the CLI. The report announced
    // success before the validation that then failed, so `--json` said `started a run` on a run that
    // exited 2. And a record carrying an injected remote script existed as a file somebody could
    // open, which is the worst of the three.
    let report = crate::commands::validate_source_at(&document, &destination.path);
    if !report.is_accepted() {
        return Err(PublishError::Invalid {
            report: format!(
                "the record composed from {} does not validate, so it was not written. The copy is \
                 faithful, so this reports a fault in the runbook rather than in the record — a \
                 requirement with no identifier, two identifiers that collide, or markup inside \
                 one. Fix the runbook and run this again:\n{}",
                run.name(),
                htmlspec_validate::render_human(&report)
            ),
        });
    }

    let counted = count(&document);
    let checked = run.source.contract.len();
    let passed = run.passed.len();
    let version = run.source.version;

    // The directory is created only here, and only if the publication goes on to succeed: a
    // `runs/` left behind by a refused start is a place the project did not have before and has no
    // record in.
    let created = !destination.runs.is_dir();
    if created {
        std::fs::create_dir_all(&destination.runs).map_err(|error| PublishError::Io {
            report: format!("cannot create {}: {error}", destination.runs.display()),
        })?;
    }

    let temporary = Temporary::in_directory(&destination.runs, &destination.record)
        .map_err(|report| PublishError::Io { report })?;
    let written = std::fs::write(temporary.path(), &document);
    if let Err(error) = written {
        // The path is named before the handle is dropped, because dropping it is what removes the
        // file and the report has to say which file was cleaned up.
        let report = format!(
            "cannot write {}: {error}. Nothing was published and no record was replaced",
            temporary.path().display()
        );
        drop(temporary);
        remove_if_empty(created, &destination.runs);
        return Err(PublishError::Io { report });
    }

    // The exclusive step. `link(2)` fails with `EEXIST` where the name is taken — including where it
    // is taken by a broken symlink, which is the case `Path::exists` fails open on and
    // `nothing_is_at` was written to refuse. The record appears at its final name atomically and
    // complete, because the temporary file was fully written first.
    if let Err(error) = std::fs::hard_link(temporary.path(), &destination.path) {
        return Err(unpublished(temporary, &error, created, &destination));
    }

    // Disarmed: the temporary file's name is now a second link to the record, so removing it is
    // what leaves exactly one. A failure to remove it does not un-publish the record, and it is not
    // swallowed either — it travels with the result, because a hidden file an operator finds in the
    // records directory and cannot account for is worse than one they were told about.
    let leftover = match temporary.remove() {
        Ok(()) => None,
        Err(_) => Some(leftover_diagnostic(&destination.runs)),
    };

    Ok(StartedRun {
        path: destination.path,
        steps: counted.steps,
        requirements: counted.requirements,
        checked,
        passed,
        version,
        leftover,
    })
}

/// Settles a publication whose exclusive link failed, and says what it left behind.
///
/// **Kept only where the name was taken.** There the temporary file holds the record this start
/// composed while another is at its name, and the operator may want to compare the two, so it is
/// named in the report. Any other failure — no hard links on this filesystem, a permission or device
/// error — leaves nothing worth looking at, so the file is removed, and the report says it was only
/// once that removal succeeded. Keeping it in both cases was the defect: the report said "removed"
/// while a `.tmp` sat in `runs/`, and the directory this start created stayed with it.
fn unpublished(
    temporary: Temporary,
    error: &std::io::Error,
    created: bool,
    destination: &VacantRecord,
) -> PublishError {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        let kept = temporary.keep();
        return PublishError::Occupied {
            report: format!(
                "{} appeared between this operation checking that it was free and writing to it, so \
                 nothing was published and the record that is there was not touched. A run \
                 interrupted and continued is that record edited rather than a second one; to start \
                 a separate run, give it another name. What this start composed is at {kept}, \
                 which is safe to remove",
                destination.path.display()
            ),
        };
    }
    let composed = temporary.path().display().to_string();
    let cleanup = match temporary.remove() {
        Ok(()) => {
            remove_if_empty(created, &destination.runs);
            String::from("What this start composed was removed with it")
        }
        Err(removal) => format!(
            "What this start composed is still at {composed}, because removing it failed too \
             ({removal}), and it is safe to remove"
        ),
    };
    PublishError::Io {
        report: format!(
            "cannot publish {}: {error}. Nothing was published and no record was replaced. \
             {cleanup}",
            destination.path.display()
        ),
    }
}

/// What an operator should know about the temporary files a records directory holds.
///
/// Names the files rather than counting them, because the safe act is to remove a specific one and a
/// count leaves them to find it. Hidden `.tmp` files are what an interrupted start leaves, and no
/// record enumeration reads them.
fn leftover_diagnostic(runs: &Path) -> String {
    let found: Vec<String> = std::fs::read_dir(runs)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.extension().is_some_and(|extension| extension == "tmp")
                        && path
                            .file_name()
                            .is_some_and(|name| name.to_string_lossy().starts_with('.'))
                })
                .filter_map(|path| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                })
                .collect()
        })
        .unwrap_or_default();
    if found.is_empty() {
        return String::from(
            "a temporary file this start wrote could not be removed. Temporary files are hidden and \
             end in `.tmp`, no record enumeration reads them, and they are safe to remove"
        );
    }
    format!(
        "{} temporary file(s) from an interrupted or partially completed start are in {}: {}. They \
         are hidden and end in `.tmp`, no record enumeration reads them, and they are safe to \
         remove",
        found.len(),
        runs.display(),
        found.join(", ")
    )
}

/// Removes a `runs/` directory this operation created and then had nothing to put in.
///
/// Only ever a directory this call created, and only when it is empty: removing a directory that was
/// already there, or one holding another process's record, would be destroying history to tidy up
/// after a failure.
fn remove_if_empty(created: bool, runs: &Path) {
    if !created {
        return;
    }
    let empty = std::fs::read_dir(runs).is_ok_and(|entries| entries.count() == 0);
    if empty {
        let _ = std::fs::remove_dir(runs);
    }
}

/// A temporary file this operation owns, removed unless it was disarmed.
///
/// **Hidden and without the record's extension**, so a crash between writing it and linking it leaves
/// something record enumeration ignores rather than a document a reader might open. `gather` filters
/// on the `.html` extension, and a name beginning `.` is not a record anybody wrote.
///
/// Archival publishes its adoption marker through this too, for the same reason a run's record does:
/// a staging name that was not created exclusively is one two writers can share.
pub(crate) struct Temporary {
    path: PathBuf,
    owned: bool,
}

impl Temporary {
    /// Creates a uniquely named temporary file in `directory`.
    ///
    /// Exclusive creation rather than a guessed-unique name: two concurrent starts must not write
    /// the same temporary file, and `create_new` is the primitive that says so.
    pub(crate) fn in_directory(directory: &Path, record: &str) -> Result<Self, String> {
        for attempt in 0..64u32 {
            let path = directory.join(format!(
                ".htmlspec-{record}-{}-{attempt}.tmp",
                std::process::id()
            ));
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(_) => return Ok(Self { path, owned: true }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "cannot create a temporary file in {}: {error}. A file is written \
                         complete before it is published, so a place to write it is not optional",
                        directory.display()
                    ))
                }
            }
        }
        Err(format!(
            "every temporary-file name this operation tried in {} is taken, which means interrupted \
             operations have left files behind. They are hidden and end in `.tmp`, nothing reads \
             them, and they are safe to remove",
            directory.display()
        ))
    }

    /// Where the file is.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Stops this handle from removing the file, returning its path for a report.
    fn keep(mut self) -> String {
        self.owned = false;
        self.path.display().to_string()
    }

    /// Removes the file, which publication does once the record is linked into place.
    pub(crate) fn remove(mut self) -> std::io::Result<()> {
        self.owned = false;
        std::fs::remove_file(&self.path)
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        if self.owned {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Whether the record's path is free, refusing anything it cannot read as free.
///
/// **`symlink_metadata` rather than `Path::exists`, and this project already learned why.**
/// `commands.rs`'s `refuse_a_runbook` states it: `Path::exists` *follows* symlinks and reports
/// `false` for a broken one, which is how the archival guard failed open a third time. The lesson was
/// stated in one place and not applied in the sibling.
///
/// Measured against the version using `exists`: with `runs/x.html` a broken symlink pointing at
/// `../../../../ESCAPED.html`, `run rb --as x` exited 0, reported `started a run of rb`, and wrote
/// 19574 bytes outside the project. The path looked free because what it pointed at was not there,
/// and `std::fs::write` then followed the link.
///
/// So the three arms are `refuse_a_runbook`'s, for its reasons. `NotFound` is the one honest
/// "proceed": nothing is at this path. Anything present refuses, because a run never overwrites a
/// record. And any other I/O error refuses rather than proceeding, because this decides whether to
/// write and an error it cannot interpret must not read as permission.
fn nothing_is_at(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "cannot read {} to determine whether a record is already there: {error}. Refusing \
             rather than proceeding, because a run that overwrote a record would destroy the \
             history the record exists to hold",
            path.display()
        )),
        // Present, whatever it is. A symlink is named rather than folded into "already exists",
        // because the two call for different actions: a record is resumed by editing it, and a
        // symlink is something nothing in this program creates.
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "{} is a symbolic link. Nothing in this program creates one there, and writing through \
             it would put a run's record wherever it points — outside the project, if that is where \
             it points. Remove it, or start this run under another name",
            path.display()
        )),
        Ok(_) => Err(format!(
            "{} already exists. A run interrupted and continued is that record edited rather than \
             a second one, so this operation starts a run and never overwrites one — overwriting \
             would destroy the history the record exists to hold",
            path.display()
        )),
    }
}

/// Reports what a runbook's runs recorded against what the runbook states they must.
///
/// Writes nothing, and **exits zero whatever it finds**. The rule is `review/*`, which never gates,
/// and `runbook-execution` requires it: a run is one record from start to finish, so a record
/// legitimately attests to nothing while the run is under way. A check that failed a run halfway
/// through is one an operator learns to skip.
///
/// The gathering is here and the judging is in [`htmlspec_validate::rules::runs`], which takes the
/// gathered facts and no filesystem capability at all. That split is
/// [`htmlspec_validate::rules::topology::duplicates_across`]'s, for its stated reason.
pub fn check<O: Write, E: Write>(
    runbook: &str,
    store: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let project = match crate::store::project_for(store) {
        Ok(project) => project,
        Err(report) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };
    if !ProjectRoot::is_a_name(runbook) {
        let _ = streams.failure(&format!(
            "{runbook:?} is a path rather than a runbook name. Pass the name alone"
        ));
        return Outcome::OperationalFailure;
    }

    let directory = project.runbook(runbook);
    if !directory.is_dir() {
        let _ = streams.failure(&format!(
            "no runbook named {runbook} in {}. To see what the project holds, run \
             `htmlspec list --runbooks`",
            project.runbooks().display()
        ));
        return Outcome::OperationalFailure;
    }

    // The same `Option` as the write path, for the same reason: absent is not empty.
    let required = std::fs::read_to_string(directory.join(CONTRACT))
        .ok()
        .map(|source| requirement_ids(&source))
        .unwrap_or_default();
    let records = gather(&project.runs(runbook));

    // **A record this operation cannot read is reported and excluded, not silently dropped.** It is
    // neither an absent record nor a conforming one: nothing is known about it, and the distinction
    // between "nothing is there" and "something is there and I could not read it" is the one
    // `refuse_a_runbook` fails closed on for the same reason.
    //
    // Two unreadable cases, and they are separate questions. Bytes that will not read at all — a
    // permission denied, a device error — and bytes that parse only through error recovery, which
    // means the document has more than one reading and this would pick one silently. The second is the
    // recovery gate `compose` applies on the write path, applied here on the read path, which is
    // where it was missing.
    let mut unreadable: Vec<String> = Vec::new();
    let mut runs: Vec<RecordedRun> = Vec::new();
    for found in &records {
        match found {
            Found::Unreadable(name, error) => unreadable.push(format!("{name}: {error}")),
            Found::Readable(name, source) => {
                let parsed = htmlspec_parse::parse_document(source);
                if !parsed.is_recovery_free() {
                    unreadable.push(format!(
                        "{name}: its markup requires error recovery, so it has more than one \
                         reading and this would pick one. Run `htmlspec validate` on it"
                    ));
                    continue;
                }
                runs.push(RecordedRun {
                    record: name.clone(),
                    recorded: recorded_ids(&parsed),
                });
            }
        }
    }

    let diagnostics = htmlspec_validate::rules::runs::conformance(&RunbookUnderReview {
        name: runbook,
        required: &required,
        runs: &runs,
    });

    if streams.form().is_structured() {
        let payload = serde_json::json!({
            "runbook": runbook,
            "required": required,
            "unreadable": unreadable,
            "runs": runs
                .iter()
                .map(|run| serde_json::json!({
                    "record": run.record,
                    "recorded": run.recorded.iter().collect::<Vec<_>>(),
                }))
                .collect::<Vec<_>>(),
            "diagnostics": diagnostics
                .iter()
                .map(htmlspec_validate::StructuredDiagnostic::of)
                .collect::<Vec<_>>(),
        });
        if let Ok(json) = serde_json::to_string_pretty(&payload) {
            let _ = streams.payload(&json);
        }
        return Outcome::Success;
    }

    let _ = streams.progress(&format!(
        "checking {} run(s) of {runbook} against {} requirement(s) it states",
        runs.len(),
        required.len()
    ));
    // Named before the conformance report, because a reader who cannot see a record cannot act on
    // what is said about the ones that were read.
    for report in &unreadable {
        let _ = streams.progress(&format!(
            "cannot read the record {report}. It is excluded from what follows, so this report is \
             about the {} record(s) that could be read",
            runs.len()
        ));
    }
    for diagnostic in &diagnostics {
        let _ = streams.report(&format!("{diagnostic}\n"));
    }
    if diagnostics.is_empty() {
        // The two silent cases are distinguished, because they mean different things and a reader
        // told "each run records what the runbook states" about a runbook with no runs would read
        // it as a pass.
        let _ = streams.report(&match (runs.is_empty(), required.is_empty()) {
            // `records.is_empty()` rather than `runs.is_empty()`, so a runbook whose only record
            // could not be read is not reported as a runbook nobody has run. That conflation is the
            // defect this arm had: the two produced byte-identical output.
            (true, _) if records.is_empty() => format!("{runbook} has no runs on record\n"),
            (true, _) => format!(
                "{runbook} has {} record(s) and none of them could be read, so nothing is known \
                 about its runs\n",
                records.len()
            ),
            (false, true) => format!(
                "{runbook} states no requirement of its runs, so there is nothing to check\n"
            ),
            (false, false) => format!("each run of {runbook} records what the runbook states\n"),
        });
    } else {
        let _ = streams.report(&format!(
            "\n{} unrecorded requirement(s) across {} run(s). This never gates the run: a run \
             under way legitimately has requirements it has not reached\n",
            diagnostics.len(),
            runs.len()
        ));
    }
    // Zero whatever was found. `review/*` never gates, and `--strict` cannot promote it.
    Outcome::Success
}

/// Every run record beneath `runs`, as `(name, source)`, in name order.
///
/// An absent directory is a runbook nobody has run, which is not a fault: `runbook-execution`
/// states that the place is created by the operation that writes the first record, so its absence
/// is the ordinary state of a runbook before its first run.
/// One record `gather` found, and whether it could be read at all.
///
/// **A sum type rather than dropping the unreadable ones**, which is the fix. `filter_map` with
/// `.ok()?` turned a record that could not be read into an *absent* record, so a runbook with one
/// unreadable record reported `checking 0 run(s)` / `has no runs on record`, exit 0 — byte-identical
/// to a runbook nobody has run. That is a distinction this code makes deliberately three lines from
/// where it was losing it.
///
/// The precedent cuts this way too. `commands.rs`'s `completion` returns `None` for a document it
/// cannot read, and the enumeration reports that as *a change with no tasks document* rather than as
/// no change — the project's habit is to preserve the distinction.
enum Found {
    /// The record's name and its source.
    Readable(String, String),
    /// The record's name and why it could not be read.
    Unreadable(String, String),
}

impl Found {
    fn name(&self) -> &str {
        match self {
            Self::Readable(name, _) | Self::Unreadable(name, _) => name,
        }
    }
}

fn gather(runs: &Path) -> Vec<Found> {
    let Ok(entries) = std::fs::read_dir(runs) else {
        return Vec::new();
    };
    let mut found: Vec<Found> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "html")
        })
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().into_owned();
            Some(match std::fs::read_to_string(&path) {
                Ok(source) => Found::Readable(name, source),
                Err(error) => Found::Unreadable(name, error.to_string()),
            })
        })
        .collect();
    found.sort_by(|left, right| left.name().cmp(right.name()));
    found
}

/// The requirement identifiers a document states of its runs, in document order.
///
/// **A withdrawn requirement is excluded, and this is the same exclusion `units_of` makes on the
/// write path.** Applying it there and not here is the stated-once-applied-once pattern arriving
/// inside the fix for it: the record correctly stopped copying a `removed` requirement, and `--check`
/// went on demanding an outcome for it — so a run was reported as short of an obligation that no
/// longer applies and that the record is right not to carry.
///
/// Measured before the fix: a runbook whose one requirement is `removed` produced a record with zero
/// requirements and a report of `1 unrecorded requirement(s)`.
///
/// The change state is read from the projection rather than from the markup, which is what
/// `rules::review` does for the same question.
fn requirement_ids(source: &str) -> Vec<String> {
    let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(source).document);
    projection
        .model
        .requirements
        .iter()
        .filter(|requirement| !is_withdrawn(&requirement.change))
        .filter_map(|requirement| requirement.id.clone())
        .collect()
}

/// Whether a change state withdraws the unit.
///
/// Found by a stem out of the schema's own vocabulary rather than by writing `"removed"`, which is
/// `rules::review`'s discipline for exactly this question — its `material_states` excludes the
/// withdrawn state the same way, so the two cannot disagree about which value that is.
///
/// **Crate-visible rather than private to this module**, because a second caller needs the same
/// answer: [`crate::preflight::parse_contract`] skips a withdrawn requirement's machine-check
/// declaration for the reason the two call sites here skip it. A third copy of the stem match would
/// be the fourth statement of one schema value, which is the drift `tests/schema_rust_split.rs`
/// exists to catch.
pub(crate) fn is_withdrawn(change: &str) -> bool {
    htmlspec_validate::schema_facts::enum_values("definitions/changeState")
        .into_iter()
        .filter(|state| state.contains("remov"))
        .any(|state| state == change)
}

/// The identifiers some verification in this document recorded an outcome for.
///
/// **Two filters, and each one is a case the rule would otherwise get wrong.**
///
/// Read from the verifications' references rather than from the requirements present, because
/// recording is what a verification does. A record that copied a requirement and recorded nothing
/// about it is precisely the case the conformance rule reports, so counting the copy would make the
/// rule report nothing ever.
///
/// And a verification declaring the "not run" outcome is excluded, because it recorded no outcome.
/// That matters because it is exactly what [`start`] writes — one per requirement, so the record
/// enumerates what it owes — so counting them would report every run as conforming from the instant
/// it began. `runbook-execution` requires a record to state an outcome *and* how it was reached, and
/// "not run" states neither.
fn recorded_ids(parsed: &htmlspec_parse::ParseOutcome) -> std::collections::BTreeSet<String> {
    let projection = htmlspec_parse::project(&parsed.document);
    projection
        .model
        .verifications
        .iter()
        .filter(|verification| has_run(verification.outcome.as_deref()))
        .flat_map(|verification| verification.references.iter())
        .map(|reference| reference.target.clone())
        .collect()
}

/// Whether a recorded outcome says the check actually ran.
///
/// The values come from the schema rather than from a list here, which is
/// [`htmlspec_validate::rules::review`]'s discipline for the same question: only the "not run"
/// outcome means nothing was checked, so it is found by a stem and everything else counts as having
/// run. A literal `"not-run"` here would be a second statement of a value the schema owns, and
/// `tests/schema_rust_split.rs` exists because that is a thing this project has done wrong before.
///
/// An absent outcome counts as *not* having run. The schema requires one, so a verification without
/// it is a document the validator reports — and treating an absent value as a recorded outcome would
/// make a malformed record satisfy the contract.
fn has_run(outcome: Option<&str>) -> bool {
    let Some(outcome) = outcome else {
        return false;
    };
    let enumerated =
        htmlspec_validate::schema_facts::enum_values("definitions/verification/properties/outcome");
    // **The outcome must be one the schema enumerates, and this is the fix.** The previous version
    // asked only "is it not the not-run one", which an unenumerated value answers `true` — so
    // `data-outcome="notrun"`, a typo, reported the run as conforming. Measured: `each run of rb
    // records what the runbook states`.
    //
    // A value outside the vocabulary is not a recorded outcome. It is a document the validator
    // reports, and treating it as a recorded pass would let a typo satisfy the contract — the same
    // reasoning that makes an *absent* outcome count as not having run.
    //
    // Still no literal `"not-run"` here: the vocabulary and the not-run member both come from the
    // schema, which is what `tests/schema_rust_split.rs` exists to keep true.
    if !enumerated.iter().any(|value| value == outcome) {
        return false;
    }
    !enumerated
        .into_iter()
        .filter(|value| value.contains("not"))
        .any(|value| value == outcome)
}

/// How much a record holds, for its report.
struct Counted {
    steps: usize,
    requirements: usize,
}

fn count(source: &str) -> Counted {
    let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(source).document);
    Counted {
        steps: projection.model.tasks.len(),
        requirements: projection.model.requirements.len(),
    }
}

/// Builds the record's source from the runbook's two documents.
///
/// Pure over its inputs apart from the asset store it resolves, so the composition is testable
/// without a project on disk and the one effect it does need is the one that decides the asset
/// prefix. Kept separate from [`start`] for that reason.
fn compose(run: &ReadyRun, destination: &VacantRecord) -> Result<String, String> {
    let runbook = run.name();
    let record = destination.record.as_str();
    // **Both documents gated, which is the fix.** The gate was applied to the procedure and not to
    // the contract twelve lines below it — the stated-once-applied-once shape. A document whose
    // markup needs error recovery has more than one reading, and copying one of them picks a reading
    // on the author's behalf. The formatter refuses the same case for the same reason, and this is
    // the earlier boundary.
    //
    // Gated again here although [`load`] already refused a document needing recovery, and the repeat
    // is deliberate: `compose` is where the bytes are copied, so the guarantee belongs at the copy.
    // A caller that reached it with an unsnapshotted source would otherwise have no gate at all.
    let parsed = recovery_free(&run.source.procedure, runbook, PROCEDURE)?;
    let contract_parsed = recovery_free(&run.source.specification, runbook, CONTRACT)?;

    // **The whole of the procedure's `main`, not its task units.** `runbook-execution` requires a
    // record to carry "the runbook's procedure, copied", and the record's own summary claims the
    // steps are copied as they stood — so copying only the units made both statements false.
    //
    // Measured against this repository's real release runbook: the record carried the four task units
    // and dropped the `hs-what-this-consumes` section, which is where the procedure states that it
    // needs `cargo`, `python3`, `git`, a clean tree, and a repository that is not already an
    // html-spec project. That last one is a hazard this project discovered by violating it, so the
    // dropped content was the most valuable prose in the document.
    //
    // **The contract was right and the code was wrong**, so the code changed. Narrowing the
    // requirement to "the task units" was the alternative and it is worse: a record that cannot say
    // what the procedure consumed cannot answer the question the record exists for, and the input gap
    // this change records means that prose is the *only* place a prerequisite can live.
    //
    // This also removes the invented `<ul>` wrapper, which was a second defect: the runbook's own
    // list structure now comes across intact, so a procedure whose steps are `<section>` elements no
    // longer lands inside a `<ul>` that nothing validates.
    let procedure_content = main_content(&parsed.document).ok_or_else(|| {
        format!(
            "the procedure in {runbook} has no main content to copy. A runbook's {PROCEDURE} \
             carries its steps inside the document's main element"
        )
    })?;
    if units_of(&parsed.document, "task").is_empty() {
        return Err(format!(
            "the procedure in {runbook} enumerates no step, so a run of it would record nothing. \
             A runbook's {PROCEDURE} carries one task unit per step"
        ));
    }

    let requirements = units_of(&contract_parsed.document, "requirement");

    let mut sections = procedure_content;
    if !requirements.is_empty() {
        sections.push_str(&obligations_section(&requirements));
        sections.push_str(&record_section(
            runbook,
            run.version(),
            &requirements,
            run.passed(),
        ));
    }

    let source = shell(
        runbook,
        record,
        &destination.assets,
        &masthead(runbook, record, run.version(), requirements.len()),
        &sections,
    );
    // Canonical from the moment it is written, so `format --check` is not a second step a
    // contributor has to know about. The formatter refuses only ambiguity, and everything above is
    // either a clone of markup that already parsed cleanly or generated here.
    htmlspec_format::format(&source)
        .map_err(|error| format!("the record this operation composed cannot be formatted: {error}"))
}

/// Parses `source`, refusing a document whose markup needs error recovery.
///
/// **Applied to both of a runbook's documents, which is the fix.** It was applied to the procedure
/// and not to the contract, twelve lines apart — the same stated-once-applied-once shape that made
/// the archival guard fail open. Factored into a function rather than duplicated, so a third document
/// cannot be parsed here without passing through it.
fn recovery_free(
    source: &str,
    runbook: &str,
    document: &str,
) -> Result<htmlspec_parse::ParseOutcome, String> {
    let parsed = htmlspec_parse::parse_document(source);
    if !parsed.is_recovery_free() {
        return Err(format!(
            "{document} in {runbook} requires error recovery, so it has more than one reading and a \
             copy would pick one. Run `htmlspec validate` on the runbook first"
        ));
    }
    Ok(parsed)
}

/// The content of the document's `main` element, as canonical markup with its units settled.
///
/// **The children rather than the element**, because the record supplies its own `main` and a nested
/// one would be invalid. `None` for a document with no `main`, which the caller reports against the
/// runbook: a document whose content is outside `main` is one the projection reads differently too.
fn main_content(document: &kuchikiki::NodeRef) -> Option<String> {
    let main = document
        .descendants()
        .find(|node| node.as_element().is_some_and(|e| &*e.name.local == "main"))?;
    settle_change_states(&main);
    let mut out = String::new();
    for child in main.children() {
        out.push_str(&serialize_fragment(&child));
    }
    Some(out)
}

/// Every unit of `class` in the document, in document order, with its markup and its identifier.
///
/// **Cloned and re-serialized rather than rebuilt from the projected model**, which is the decision
/// the module header states: a runbook's prose carries `<code>`, `<strong>`, and `<q>` that the
/// projection flattens to strings, and a record whose requirements read as a worse version of the
/// runbook's would defeat the point of copying them.
///
/// **The identifier comes from the tree rather than from the serialized text**, and that is a fix.
/// Reading it back textually found the first ` id="` anywhere in the subtree — so a requirement with
/// no identifier of its own silently yielded its *scenario's*, and the record then wrote a
/// verification attesting to a scenario. Measured: `data-refs="scenario-real"`. The tree already knows
/// which element owns the attribute, so asking it is both correct and simpler.
///
/// `data-change` is forced to `unchanged` on the unit and on every descendant unit.
/// `runbook-authority` requires it, and the reason applies exactly here: the obligation a run
/// attests to existed before the run started, so a copy declaring itself `added` draws
/// `review/stale-verification` — measured.
///
/// **A unit this change *withdraws* is not copied at all.** Forcing its state to `unchanged` produced
/// `data-change="unchanged"` beside a `data-reason` saying the unit is withdrawn — two attributes
/// contradicting each other — and preserving `removed` is no better: the schema states that no
/// reference may target a removed unit, and `refs/removed-target` is an **error**, so the verification
/// the record writes for it would refuse the whole record. Both branches are wrong, so the unit is
/// excluded: a run cannot attest to an obligation that no longer applies.
struct CopiedUnit {
    /// The unit's markup, canonical and with its change states settled.
    markup: String,
    /// The unit's own `id` attribute, read from the element that carries it.
    id: Option<String>,
}

fn units_of(document: &kuchikiki::NodeRef, class: &str) -> Vec<CopiedUnit> {
    let mut found = Vec::new();
    for node in document.descendants() {
        let Some(element) = node.as_element() else {
            continue;
        };
        let (is_wanted, id, withdrawn) = {
            let attributes = element.attributes.borrow();
            (
                attributes
                    .get("class")
                    .is_some_and(|value| value.split_whitespace().any(|name| name == class)),
                attributes.get("id").map(|value| value.trim().to_owned()),
                // The same question `requirement_ids` asks on the read path, through the same
                // helper. A literal here would be a second statement of a value the schema owns.
                attributes
                    .get("data-change")
                    .is_some_and(|value| is_withdrawn(value.trim())),
            )
        };
        if !is_wanted || withdrawn {
            continue;
        }
        // A nested unit of the same class would be copied twice — once on its own and once inside
        // its ancestor. No such nesting exists in the vocabulary (a requirement holds scenarios, a
        // task holds nothing), and skipping a node whose ancestor was already taken is cheaper than
        // relying on that.
        if node.ancestors().any(|ancestor| has_class(&ancestor, class)) {
            continue;
        }
        let copy = node.clone();
        settle_change_states(&copy);
        found.push(CopiedUnit {
            markup: serialize_fragment(&copy),
            id,
        });
    }
    found
}

fn has_class(node: &kuchikiki::NodeRef, class: &str) -> bool {
    node.as_element().is_some_and(|element| {
        element
            .attributes
            .borrow()
            .get("class")
            .is_some_and(|value| value.split_whitespace().any(|name| name == class))
    })
}

/// Declares every unit in the subtree unchanged.
///
/// On the subtree rather than on its root, because a requirement's scenarios are units too and a
/// scenario declaring itself `added` inside a requirement declaring itself `unchanged` is the
/// half-fixed state the previous change shipped and a review caught. That defect is recorded in
/// `the-canonical-documents-change-state-2026-09-17.md`: the general claim was written from a fix
/// applied to one document.
fn settle_change_states(node: &kuchikiki::NodeRef) {
    for descendant in node.inclusive_descendants() {
        let Some(element) = descendant.as_element() else {
            continue;
        };
        let mut attributes = element.attributes.borrow_mut();
        if attributes.get("data-change").is_some() {
            attributes.insert("data-change", String::from("unchanged"));
        }
    }
}

/// One node's own markup, without the document scaffolding a full serialization adds.
fn serialize_fragment(node: &kuchikiki::NodeRef) -> String {
    let mut out = Vec::new();
    // Infallible over a `Vec<u8>`: the only error a serializer returns here is an I/O error from
    // the writer, and a vector does not fail to grow. Reported rather than unwrapped anyway,
    // because `AGENTS.md` §5 asks for a total function and an empty string is a visible wrong
    // answer rather than a panic in a command that has already written nothing.
    if node.serialize(&mut out).is_err() {
        return String::new();
    }
    String::from_utf8(out).unwrap_or_default()
}

/// The record's masthead, whose paragraph is the document's summary.
///
/// Written rather than copied from the procedure's, because `review/example-content` reports a
/// summary that is a template's placeholder and because the summary is where a reader learns what
/// the document is. A record carrying the runbook's own summary would say what the *procedure* is
/// for rather than what this run did.
fn masthead(runbook: &str, record: &str, version: RunbookVersion, requirements: usize) -> String {
    let subject = htmlspec_assets::escape_text(&format!("{runbook} run: {record}"));
    let runbook = htmlspec_assets::escape_text(runbook);
    // **Three cases, because the zero case was describing a section that is not there.** With no
    // requirements the summary read "The 0 requirements below are copied" and no requirements section
    // followed it — and that string is the document's projected `summary`, which is where a consumer
    // reads what the document is. A runbook that states nothing about its runs is a legitimate
    // runbook, so the record says so rather than counting to zero.
    let obligations = match requirements {
        0 => String::from(
            "This runbook states no requirement of its runs, so this record carries none and there \
             is nothing for it to attest to.",
        ),
        1 => String::from(
            "The one requirement below is copied for the same reason and because a reference \
             from a runbook's documents resolves within the document that makes it.",
        ),
        several => format!(
            "The {several} requirements below are copied for the same reason and because a \
             reference from a runbook's documents resolves within the document that makes it."
        ),
    };
    // **The runbook's release version in readable prose, and never in `htmlspec-version`.** That
    // meta tag identifies the document schema, and putting a runbook's release in it would make every
    // consumer of the tag read a different fact — which is the "stuffing new meaning into an
    // existing field" shape `AGENTS.md` §10 names. The version a run carries belongs in the sentence
    // that already says what the record is for.
    //
    // **The contract version the record declares, in the masthead as in the head.** A record is
    // written by the current program and links the current stylesheet, so it declares the version a
    // new document declares, which document-template's "A document declares its identity and
    // provenance" asks of every document. The two read one constant, so they cannot disagree.
    let contract = htmlspec_schema::CONTRACT_VERSION;
    format!(
        "      <header class=\"hs-masthead\">\n\
         \x20       <h1>{subject}</h1>\n\
         \x20       <p>What one run of the {runbook} procedure did, at the {version} release of \
         that runbook. The steps are copied from the runbook as they stood when this run started, \
         so this record says which version of the procedure ran; their states advance here rather \
         than in the runbook. {obligations}</p>\n\
         \x20       <dl class=\"hs-meta\">\n\
         \x20         <dt>Kind</dt>\n\
         \x20         <dd><span class=\"hs-kind\">tasks</span></dd>\n\
         \x20         <dt>Contract</dt>\n\
         \x20         <dd>{contract}</dd>\n\
         \x20         <dt>Subject</dt>\n\
         \x20         <dd>{subject}</dd>\n\
         \x20       </dl>\n\
         \x20     </header>\n"
    )
}

fn obligations_section(requirements: &[CopiedUnit]) -> String {
    let mut out = String::from(
        "        <section id=\"hs-requirements\">\n\
         \x20         <h2>What this run must record</h2>\n\
         \x20         <p>Copied from the runbook's specification, which is forced rather than \
         chosen: a reference from a runbook's documents resolves within the document that makes \
         it, so a record naming a requirement that lives elsewhere resolves to nothing. These state what this runbook requires of a \
         run, not what the project requires of itself.</p>\n",
    );
    for requirement in requirements {
        out.push_str(&requirement.markup);
        out.push('\n');
    }
    out.push_str("        </section>\n");
    out
}

/// One `not-run` verification per copied requirement, which is the log this run fills in.
///
/// `not-run` rather than absent, and the difference decides what a reader sees. An absent
/// verification is reported by the conformance rule as a requirement nothing attests to, which is
/// correct for a run somebody abandoned and wrong for a run that has just started — so writing them
/// present-and-not-run means the record *enumerates* what it owes. The method paragraph is a
/// placeholder the run replaces, and `review/example-content` does not fire on it because it is not
/// a template's text.
fn record_section(
    runbook: &str,
    version: RunbookVersion,
    requirements: &[CopiedUnit],
    passed: &crate::preflight::PassedChecks,
) -> String {
    // Keyed by requirement identifier, which is what a passed check and a copied requirement have in
    // common. A check whose requirement was not copied — a withdrawn one, or one with no identifier
    // — matches nothing here, and neither case can arise: `parse_contract` skips withdrawn
    // requirements and refuses an unidentified one.
    let evidence: std::collections::BTreeMap<&str, &crate::preflight::PassedCheck> = passed
        .checks()
        .iter()
        .map(|check| (check.requirement.as_str(), check))
        .collect();
    let checked = evidence.len();
    let mut out = String::from(&format!(
        "        <section id=\"hs-verification\">\n\
         \x20         <h2>What this run recorded</h2>\n\
         \x20         <p>One entry per requirement above. {opening} Replace each remaining method \
         with what was actually done, precisely enough that someone else could repeat it, and set \
         the outcome to what happened — a recorded failure is a correct record of a failed check \
         rather than a fault in this document.</p>\n",
        opening = match checked {
            0 => String::from("None of them has been run yet."),
            _ => format!(
                "The {checked} machine prerequisite(s) this runbook declares were checked before \
                 the run started, at the {version} release, and each entry below records what was \
                 observed. Those checks are obligations of the run like any other: a passed \
                 prerequisite is recorded, not discharged."
            ),
        }
    ));
    for requirement in requirements {
        let Some(id) = requirement.id.as_deref() else {
            continue;
        };
        // **`strip_prefix` rather than `trim_start_matches`, which strips every repetition.**
        // Measured: `req-req-real` and `req-real` both yielded `verify-real`, and both identifiers
        // are schema-legal — so a runbook holding the pair produced a record with two verifications
        // under one identifier, failing `schema/duplicate-id`. `trim_start_matches` answers "remove
        // this prefix as many times as it appears", which is not the question.
        let stem = id.strip_prefix("req-").unwrap_or(id);
        let verification = format!("verify-{stem}");
        // **Both escapes, chosen per site, and the attribute half is the one that was missing.**
        // These identifiers come from a runbook, which is content a contributor wrote, and they are
        // interpolated into a source string that is then parsed — so they are untrusted input at a
        // markup boundary whatever the schema says about their shape.
        //
        // Measured on the unescaped version: a requirement identifier of
        // `req-x<script src=https://evil.example/x.js></script>y` produced **three** live
        // `<script src="https://evil.example/x.js">` elements in a document whose whole contract is
        // that it renders with no network access. The `"` case is the attribute counterpart —
        // `data-refs="x" onload="…"` — and it had no escaping function in this file at all, which is
        // why a reader checking for missing `escape_text` calls could not see it.
        //
        // Both come from `htmlspec_assets` rather than from a copy here. See the note on
        // `escape_text` there: this file's own half-copy is what let the attribute case through.
        let id_text = htmlspec_assets::escape_text(id);
        let id_attribute = htmlspec_assets::escape_attribute(id);
        let verification_text = htmlspec_assets::escape_text(&verification);
        let verification_attribute = htmlspec_assets::escape_attribute(&verification);
        // A machine check that passed is recorded as passed, with what was observed and how. Every
        // other verification stays `not-run`, which is the distinction `log-a-runbook-run` drew and
        // this change does not blur: passing preflight does not discharge a logging obligation, it
        // records the outcome of one of them.
        let (outcome, method) = match evidence.get(id) {
            Some(check) => (
                "passed",
                format!(
                    "Checked before this run started by `htmlspec preflight {runbook}`: {}. \
                     Repeat it with that command, which observes the machine afresh rather than \
                     trusting this record.",
                    htmlspec_assets::escape_text(&check.fact.observed())
                ),
            ),
            None => (
                "not-run",
                String::from(
                    "Not run yet. State how this requirement was checked during this run, \
                     precisely enough that someone else could repeat it, and set the outcome \
                     accordingly.",
                ),
            ),
        };
        out.push_str(&format!(
            "          <section class=\"verification\" id=\"{verification_attribute}\" \
             data-change=\"unchanged\" data-outcome=\"{outcome}\" data-refs=\"{id_attribute}\">\n\
             \x20           <h3>What this run recorded for {id_text} <span class=\"hs-id\">\
             {verification_text}</span></h3>\n\
             \x20           <p data-field=\"method\">{method}</p>\n\
             \x20         </section>\n"
        ));
    }
    out.push_str("        </section>\n");
    out
}

/// The document around the body: the head, the page wrapper, and the asset references.
///
/// Spelled here rather than taken from `TEMPLATES`, and the reason is that a template is a
/// *starting point for an author* while this is a document composed from existing content. The
/// tasks template carries two example task units and a placeholder summary, both of which
/// `review/example-content` reports, so rendering it and then replacing its body would mean
/// stripping content this function never wanted.
fn shell(runbook: &str, record: &str, assets: &str, masthead: &str, sections: &str) -> String {
    let subject = htmlspec_assets::escape_text(&format!("{runbook} run: {record}"));
    let mut out = format!(
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         \x20 <head>\n\
         \x20   <meta charset=\"utf-8\">\n\
         \x20   <meta content=\"width=device-width, initial-scale=1\" name=\"viewport\">\n\
         \x20   <meta content=\"{contract}\" name=\"htmlspec-version\">\n\
         \x20   <meta content=\"tasks\" name=\"htmlspec-kind\">\n\
         \x20   <title>{subject}</title>\n\
         \x20   {fallback}\n",
        // A record is a document the program writes, so it carries the fallback stylesheet every
        // such document carries: document-template's "A document stays readable when its
        // stylesheet cannot load". A record sits three folders below the store, which is where
        // Safari refuses a stylesheet from disk.
        fallback = htmlspec_assets::fallback::ELEMENT,
        // The version the masthead names too: see `masthead`.
        contract = htmlspec_schema::CONTRACT_VERSION,
    );
    for (name, extra) in [
        ("htmlspec.css", ""),
        ("htmlspec-print.css", " media=\"print\""),
    ] {
        out.push_str(&format!(
            "    <link href=\"{assets}/{}\"{extra} rel=\"stylesheet\">\n",
            path_of(name)
        ));
    }
    out.push_str(&format!(
        "  </head>\n\
         \x20 <body>\n\
         \x20   <a class=\"hs-skip-link\" href=\"#hs-content\">Skip to content</a>\n\
         \x20   <div class=\"hs-page\">\n\
         {masthead}\
         \x20     <main class=\"hs-main\" id=\"hs-content\">\n\
         {sections}\
         \x20     </main>\n\
         \x20   </div>\n"
    ));
    for name in [
        "mermaid.min.js",
        "vega.min.js",
        "vega-lite.min.js",
        "vega-embed.min.js",
        "htmlspec.js",
    ] {
        out.push_str(&format!(
            "    <script defer=\"\" src=\"{assets}/{}\"></script>\n",
            path_of(name)
        ));
    }
    out.push_str("  </body>\n</html>\n");
    out
}

/// One asset's stored path, from the embedded table.
///
/// Read from the table rather than spelled, which is the whole reason the template carries
/// `{{ASSET:<name>}}` placeholders: an address written into source is a second place an address
/// lives and it goes stale the first time the bytes change.
fn path_of(name: &str) -> String {
    htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == name)
        .map(|asset| asset.path.to_owned())
        .unwrap_or_else(|| String::from(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A release version for the tests that compose a record.
    fn released() -> RunbookVersion {
        RunbookVersion::parse("0.1.0").expect("parses")
    }

    /// No machine-check evidence, which is the state every requirement a run has not reached is in.
    fn no_evidence() -> crate::preflight::PassedChecks {
        crate::preflight::PassedChecks::default()
    }

    /// A requirement identifier carrying markup is escaped, in content **and** in attributes.
    ///
    /// **Driven at the composition layer on purpose.** Through the CLI the schema's identifier
    /// pattern now refuses such an id before anything is composed — which is a second line of defence
    /// and not the one under test. If that pattern ever loosened, or a future unit type took a freer
    /// identifier, the escaping is what stands between a runbook and a record carrying a live remote
    /// script. So this drives `record_section` directly, where the escaping actually lives.
    ///
    /// Measured on the unescaped version: an identifier of
    /// `req-x<script src=https://evil.example/x.js></script>y` produced **three** live
    /// `<script src="https://evil.example/x.js">` elements in a document whose whole contract is that
    /// it renders with no network access.
    #[test]
    fn a_requirement_identifier_carrying_markup_is_escaped_everywhere_it_lands() {
        const HOSTILE: &str = "req-x<script src=https://evil.example/x.js></script>y";
        let units = [CopiedUnit {
            markup: String::new(),
            id: Some(HOSTILE.to_owned()),
        }];
        let section = record_section("release", released(), &units, &no_evidence());

        // The injection itself: no `<script` and no `>` from the identifier survives as markup.
        assert!(
            !section.contains("<script"),
            "the identifier's markup survived into the record:\n{section}"
        );
        assert!(
            section.contains("&lt;script"),
            "the identifier's `<` was not escaped:\n{section}"
        );
        // And the record still *names* the identifier, so the escaping did not discard the fact.
        assert!(
            section.contains("&lt;script src=https://evil.example/x.js&gt;&lt;/script&gt;y"),
            "the escaped identifier is not recognisable:\n{section}"
        );

        // The attribute half, which had no escaping function in this file at all. A `"` closes the
        // attribute early, which is how an interpolated value becomes a second attribute.
        let quoted = [CopiedUnit {
            markup: String::new(),
            id: Some(String::from("req-a\" onload=\"alert(1)")),
        }];
        let section = record_section("release", released(), &quoted, &no_evidence());
        // **Asserted by parsing rather than by searching for a string.** A substring check is what
        // this test first got wrong: `onload="` appears legitimately inside the *escaped content*
        // text, so searching for it reported a defect that is not there. What matters is whether the
        // parser sees a second attribute, and only the parser can answer that.
        let parsed = htmlspec_parse::parse_document(&format!(
            "<!doctype html><html><body><main>{section}</main></body></html>"
        ));
        let injected = parsed
            .document
            .descendants()
            .filter_map(|node| {
                node.as_element()
                    .map(|e| e.attributes.borrow().contains("onload"))
            })
            .any(|found| found);
        assert!(
            !injected,
            "a quote in the identifier closed the attribute and opened another:\n{section}"
        );
        assert!(
            section.contains("&quot;"),
            "the identifier's quote was not escaped:\n{section}"
        );
    }

    /// The verification's identifier keeps a repeated `req-` rather than stripping every one.
    ///
    /// `req-real` and `req-req-real` are both schema-legal, and `trim_start_matches` mapped both to
    /// `verify-real` — so a runbook holding the pair produced a record with two verifications under
    /// one identifier, which `schema/duplicate-id` refuses. `strip_prefix` removes the prefix once,
    /// which is the question being asked.
    #[test]
    fn a_repeated_requirement_prefix_is_not_stripped_twice() {
        let units = [
            CopiedUnit {
                markup: String::new(),
                id: Some(String::from("req-real")),
            },
            CopiedUnit {
                markup: String::new(),
                id: Some(String::from("req-req-real")),
            },
        ];
        let section = record_section("release", released(), &units, &no_evidence());
        assert!(
            section.contains("id=\"verify-real\"") && section.contains("id=\"verify-req-real\""),
            "two distinct requirements did not produce two distinct verifications:\n{section}"
        );
        assert_eq!(
            section.matches("class=\"verification\"").count(),
            2,
            "one of the two verifications is missing:\n{section}"
        );
    }

    /// A unit with no identifier of its own does not borrow a descendant's.
    ///
    /// The identifier used to be read back out of the serialized markup with a search for the first
    /// ` id="` in the subtree, so a requirement with no identifier silently yielded its *scenario's* —
    /// measured as `data-refs="scenario-real"` in a real record. The tree knows which element carries
    /// the attribute, so it is asked.
    #[test]
    fn a_unit_with_no_identifier_does_not_borrow_its_childs() {
        const SPECIFICATION: &str = "<!doctype html><html><body><main>\
             <section class=\"requirement\" data-change=\"unchanged\"><h2>R</h2>\
             <p>A run MUST record it.</p>\
             <article class=\"scenario\" id=\"scenario-real\" data-change=\"unchanged\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article>\
             </section></main></body></html>";

        let parsed = htmlspec_parse::parse_document(SPECIFICATION);
        let units = units_of(&parsed.document, "requirement");
        assert_eq!(units.len(), 1);
        assert!(
            units[0].id.is_none(),
            "the requirement has no identifier of its own, and {:?} was read instead",
            units[0].id
        );
        // And nothing is written for it, rather than a verification naming the scenario.
        assert!(
            record_section("release", released(), &units, &no_evidence())
                .matches("class=\"verification\"")
                .count()
                == 0,
            "a verification was written for a requirement that cannot be referenced"
        );
    }

    /// A withdrawn requirement is not copied, on the write path and the read path alike.
    ///
    /// Both halves, because applying it to one was the defect: the record correctly stopped carrying a
    /// `removed` requirement while `--check` went on demanding an outcome for it.
    ///
    /// Forcing its state to `unchanged` was the original behaviour and produced
    /// `data-change="unchanged"` beside a `data-reason` saying the unit is withdrawn. Preserving
    /// `removed` is no better: the schema states no reference may target a removed unit and
    /// `refs/removed-target` is an error, so the verification would refuse the whole record.
    #[test]
    fn a_withdrawn_requirement_is_carried_by_neither_path() {
        const SPECIFICATION: &str = "<!doctype html><html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body><main>\
             <section class=\"requirement\" id=\"req-gone\" data-change=\"removed\"><h2>R</h2>\
             <p>A run MUST record it.</p>\
             <article class=\"scenario\" id=\"scenario-gone\" data-change=\"removed\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article>\
             </section>\
             <section class=\"requirement\" id=\"req-here\" data-change=\"unchanged\"><h2>R</h2>\
             <p>A run MUST record this one.</p>\
             <article class=\"scenario\" id=\"scenario-here\" data-change=\"unchanged\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article>\
             </section></main></body></html>";

        let parsed = htmlspec_parse::parse_document(SPECIFICATION);
        let copied = units_of(&parsed.document, "requirement");
        assert_eq!(
            copied
                .iter()
                .filter_map(|unit| unit.id.as_deref())
                .collect::<Vec<_>>(),
            vec!["req-here"],
            "the write path copied a withdrawn requirement"
        );

        assert_eq!(
            requirement_ids(SPECIFICATION),
            vec![String::from("req-here")],
            "the read path demands an outcome for a withdrawn requirement, which the record is \
             right not to carry"
        );
    }

    /// An outcome the schema does not enumerate is not a recorded outcome.
    ///
    /// Measured before the fix: `data-outcome="notrun"` — a typo — reported the run as conforming,
    /// because the filter kept schema values containing `not` and an unknown string matched none of
    /// them. An absent outcome was already handled; an unrecognized one fell through to "ran".
    #[test]
    fn only_an_outcome_the_schema_enumerates_counts_as_recorded() {
        assert!(
            has_run(Some("passed")),
            "a passed check did not count as run"
        );
        assert!(
            has_run(Some("failed")),
            "a failed check recorded an outcome"
        );
        assert!(
            has_run(Some("blocked")),
            "a blocked check recorded an outcome"
        );
        assert!(
            has_run(Some("partial")),
            "a check that reached part of its requirement recorded an outcome"
        );
        assert!(!has_run(Some("not-run")), "not-run records no outcome");
        assert!(!has_run(None), "an absent outcome records none");
        for typo in ["notrun", "not run", "NOT-RUN", "ran", ""] {
            assert!(
                !has_run(Some(typo)),
                "{typo:?} is outside the schema's vocabulary and counted as a recorded outcome"
            );
        }
    }

    /// The whole of the procedure's `main` is carried, not only its units.
    ///
    /// `runbook-execution` requires a record to carry "the runbook's procedure, copied", and the
    /// record's own summary claims the steps are copied as they stood. Copying only the task units made
    /// both false — measured against this repository's release runbook, where the dropped section is
    /// the one stating that the procedure needs `python3` and a repository that is not already a
    /// project. **The contract was right and the code was wrong.**
    #[test]
    fn the_procedures_prose_is_carried_beside_its_units() {
        const PROCEDURE: &str = "<!doctype html><html><body><main>\
             <section id=\"hs-what-this-consumes\"><h2>What this consumes</h2>\
             <p><code>python3</code>, and a repository that is not already a project.</p>\
             </section>\
             <section id=\"hs-tasks\"><h2>Tasks</h2>\
             <ul><li class=\"task\" id=\"task-one\" data-change=\"unchanged\" data-state=\"todo\">\
             <h3>A step</h3></li></ul></section></main></body></html>";

        let parsed = htmlspec_parse::parse_document(PROCEDURE);
        let carried = main_content(&parsed.document).expect("the procedure has a main");

        assert!(
            carried.contains("hs-what-this-consumes") && carried.contains("<code>python3</code>"),
            "the record dropped the prose the procedure states its prerequisites in:\n{carried}"
        );
        assert!(
            carried.contains("id=\"task-one\""),
            "the record dropped the procedure's step:\n{carried}"
        );
        // The runbook's own list structure comes across, so nothing invents a wrapper around it.
        assert!(
            carried.contains("<ul>") && carried.contains("</ul>"),
            "the procedure's list structure was not preserved:\n{carried}"
        );
        assert!(
            !carried.contains("<main"),
            "the copy nested a second main element inside the record's:\n{carried}"
        );
    }

    /// The masthead does not count to zero about a section that is not there.
    ///
    /// With no requirements the summary read "The 0 requirements below are copied" and no requirements
    /// section followed — and that string is the document's projected `summary`, which is where a
    /// consumer reads what the document is.
    #[test]
    fn the_summary_does_not_describe_requirements_that_are_absent() {
        let none = masthead("release", "0-14-0", released(), 0);
        assert!(
            !none.contains(" 0 requirement"),
            "the summary counts to zero about a section that is not there:\n{none}"
        );
        assert!(
            none.contains("states no requirement of its runs"),
            "the summary does not say why there is nothing to attest to:\n{none}"
        );
        assert!(masthead("release", "x", released(), 1).contains("The one requirement below"));
        assert!(masthead("release", "x", released(), 5).contains("The 5 requirements below"));
    }

    /// A runbook or record name carrying markup is escaped where it reaches the document.
    ///
    /// `is_a_name` refuses a path separator, which is a different question: `&` and `<` are legal in a
    /// name. Asserted at both sites, since the name reaches the title, the heading, and the summary.
    #[test]
    fn a_name_carrying_markup_is_escaped_in_the_document() {
        let hostile = "a<b&c";
        let head = masthead(hostile, hostile, released(), 1);
        assert!(!head.contains("a<b&c"), "the name went in raw:\n{head}");
        assert!(
            head.contains("a&lt;b&amp;c"),
            "the name was not escaped:\n{head}"
        );

        let whole = shell(hostile, hostile, "assets", "", "");
        assert!(!whole.contains("a<b&c"), "the name went in raw:\n{whole}");
        assert!(whole.contains("a&lt;b&amp;c"), "the name was not escaped");
    }

    /// A link that failed for any reason but a taken name removes what it composed, and says so.
    ///
    /// Driven at [`unpublished`] because the failure it handles — a filesystem without hard links, a
    /// device or permission error — cannot be provoked portably through the CLI. Measured on the
    /// defect: the report said the composed file "was removed with it" while it sat in `runs/`, and a
    /// `runs/` this start created stayed because the file kept it from being empty.
    #[test]
    fn a_failed_link_removes_what_it_composed_unless_the_name_was_taken() {
        for (kind, removed) in [
            (std::io::ErrorKind::PermissionDenied, true),
            (std::io::ErrorKind::Unsupported, true),
            (std::io::ErrorKind::AlreadyExists, false),
        ] {
            let root = std::env::temp_dir().join(format!(
                "htmlspec-unpublished-{kind:?}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            let runs = root.join("runs");
            std::fs::create_dir_all(&runs).expect("runs is created");
            let destination = VacantRecord {
                path: runs.join("first.html"),
                runs: runs.clone(),
                record: String::from("first"),
                assets: String::new(),
            };
            let temporary = Temporary::in_directory(&runs, "first").expect("temporary is created");
            std::fs::write(temporary.path(), "composed").expect("written");
            let composed = temporary.path().to_owned();

            let error = unpublished(temporary, &std::io::Error::from(kind), true, &destination);
            match (&error, removed) {
                (PublishError::Io { report }, true) => {
                    assert!(
                        !composed.exists(),
                        "{kind:?} left {} behind",
                        composed.display()
                    );
                    assert!(
                        !runs.exists(),
                        "{kind:?} left the runs/ directory this start created"
                    );
                    assert!(
                        report.contains("was removed with it"),
                        "{kind:?} did not say what it removed: {report}"
                    );
                }
                (PublishError::Occupied { report }, false) => {
                    assert!(
                        composed.is_file(),
                        "a taken name discarded what was composed"
                    );
                    assert!(
                        report.contains(&composed.display().to_string()),
                        "the kept file is not named: {report}"
                    );
                }
                (other, _) => panic!("{kind:?} reported {other:?}"),
            }
            let _ = std::fs::remove_dir_all(&root);
        }
    }
}
