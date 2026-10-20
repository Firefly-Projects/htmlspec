//! The `htmlspec` command-line program.
//!
//! Five subcommands — `validate`, `format`, `assets`, `archive`, `init` — each usable with no
//! configuration in a conforming project. `assets` stands apart from the rest: its
//! subject is a project rather than a document, which is why it is not a flag on
//! `validate`. There is no configuration surface at all:
//! behavior is determined by the schema and the fixed rules, and every knob added
//! here would become a compatibility obligation before anyone had asked for one.
//!
//! Every operation has two spellings, its OpenSpec one (`validate`) and a request verb with an
//! object (`get validity`). Each operation's arguments are one `Args` struct that both spellings
//! carry, and both parse to one [`Invocation`], which is what runs. [`operation`] states the two
//! spellings, and this file's tests hold clap's declaration to it.
//!
//! The process has exactly one exit point, at the bottom of `main`. Each command
//! returns an [`exit::Outcome`] rather than exiting, which keeps the exit-code
//! contract in one place and makes every command testable without spawning a binary.

mod commands;
mod declaration;
mod discover;
mod exit;
mod install;
// Checked at test time rather than at runtime: the bodies are compiled in, so a body
// naming an absent command is a defect in this repository and never in a contributor's
// project. A runtime check would report it to the wrong person, after shipping.
#[cfg(test)]
mod invocations;
mod migrate;
mod operation;
mod output;
mod plan;
mod preflight;
mod project;
mod prompt;
mod runbook_release;
mod runs;
mod sequence;
mod show;
mod staging;
mod store;
// The released command surfaces and the check that none is lost. Test-only for the reason
// `invocations` is: it reads the parser and the published payload schemas, and the program has no
// use for either at runtime.
#[cfg(test)]
mod surface;
mod tools;
mod topology;
mod version;

use std::path::PathBuf;

use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand};
use htmlspec_validate::Strictness;

use crate::commands::{FallbackRequest, FormatMode};
use crate::output::{Form, Streams};

/// The version reported by `--version`.
///
/// The crate version plus the revision the binary was built from, so a diagnostic in
/// a bug report identifies a specific build rather than a release line.
///
/// `HTMLSPEC_REVISION` is always set by the build script — to `unknown` when there is
/// no Git checkout to read, which is a legitimate way to build this. That is why a
/// plain `env!` is correct here: matching on `option_env!` would not help, because
/// `env!` inside the `Some` arm is expanded regardless of which arm is taken and would
/// fail the build whenever the variable was genuinely absent.
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("HTMLSPEC_REVISION"),
    ")"
);

#[derive(Debug, Parser)]
#[command(
    name = "htmlspec",
    version = VERSION,
    about = "Validate, format, and create HTMLSpec documents",
    long_about = "HTMLSpec documents are HTML files that carry specification content in a \
                  closed vocabulary of semantic units. This program validates them against \
                  the published contract, formats them into their one canonical form, \
                  checks a project's asset topology, and creates new ones.\n\nEvery subcommand works with no configuration in a \
                  conforming project.",
    // A subcommand is required, and an unknown one produces an error listing the
    // available subcommands — which is what the specs ask for and what clap does by
    // default with `arg_required_else_help`.
    arg_required_else_help = true
)]
struct Cli {
    /// Emit a structured JSON payload on standard output instead of a human report.
    ///
    /// Standard output then contains nothing but the payload, so it can be piped
    /// straight into a parser. Progress and run-level messages go to standard error.
    #[arg(long, global = true)]
    json: bool,

    /// Act on a registered standalone project instead of the one containing this directory.
    ///
    /// Global, because the operations that read a project are spread across subcommands and a
    /// contributor working in a store keeps the flag on every one of them. Naming a store that
    /// is not registered, or one whose location no longer holds a project, is reported — it
    /// never falls back to the project you are standing in.
    #[arg(long, global = true)]
    store: Option<String>,

    #[command(subcommand)]
    command: Command,
}

/// The command line as clap reads it: every operation by its OpenSpec spelling, and then the five
/// request verbs.
///
/// Each operation's variant carries its `Args` struct, and so does its object under its verb, so
/// the two spellings take the same arguments by construction. [`Invocation::from`] maps both to
/// one invocation.
#[derive(Debug, Subcommand)]
enum Command {
    /// Validate documents against the contract.
    ///
    /// With no path, validates every document in the project. With a path, validates
    /// that file, or every document beneath that directory.
    Validate(ValidateArgs),

    /// Rewrite documents into their canonical form.
    ///
    /// With no path, formats every document in the project.
    Format(FormatArgs),

    /// Check a project's asset topology.
    ///
    /// Reports each set of locations holding identical asset bytes, so a project stores
    /// one copy of its runtime rather than one per document. Separate from `validate`
    /// because its subject is a project rather than a document.
    Assets(AssetsArgs),

    /// Archive a completed change.
    ///
    /// Checks the project's version, the change's specification artifact, and whether that
    /// specification's requirements reached the accepted specifications, then moves the
    /// change into the archive and records the version it shipped in.
    ///
    /// The specification itself is an agent's work, and so is applying its contract to the
    /// accepted specifications: a proposal and a design have no imposed structure, so there
    /// is nothing a parser could read reliably. This checks what an agent wrote, reports
    /// what it did not find, and performs the mechanical part.
    Archive(ArchiveArgs),

    /// Set up an HTMLSpec project in this repository.
    ///
    /// Creates the project's directories, establishes its one shared asset store, and
    /// installs the workflow instructions for the AI coding tools the project uses.
    /// Takes no arguments: the operation a contributor needs first is the one that
    /// establishes the project they are standing in.
    ///
    /// Safe to re-run. On an established project it refreshes the instructions and
    /// leaves the specifications, changes, and archive untouched.
    Init(InitArgs),

    /// List what the project holds.
    ///
    /// Active changes by default. This is the operation a workflow selects from, so it is
    /// also what makes an argument-less `archive` able to ask which change you meant.
    List(ListArgs),

    /// Start a run of a runbook, or check what its runs recorded.
    ///
    /// A run's record is a copy of the runbook's procedure with its states advanced, beside the
    /// requirements the runbook states its runs must record. The copy is not duplication for its
    /// own sake: a reference from a runbook's documents resolves within the document that makes
    /// it, so a record naming a requirement that lives in the runbook's specification would resolve
    /// to nothing.
    ///
    /// A run interrupted and continued is that record edited rather than a second one, so this
    /// starts a run and never resumes one — and it refuses to overwrite a record that exists.
    ///
    /// `--check` writes nothing and reports which requirements each run has recorded nothing for.
    /// It always exits zero: a run under way legitimately has requirements it has not reached, so
    /// the report marks work for attention rather than gating it.
    Run(RunArgs),

    /// Report whether a runbook can start in this working environment.
    ///
    /// Checks the machine prerequisites the runbook declares — tools and their versions, a pinned
    /// package-manager environment, installed agent skills, and the state of the working directory —
    /// and writes nothing. The ordering is the value: a missing tool found here costs nothing, and
    /// found at step six costs the first five steps.
    ///
    /// Reports rather than repairs. What is missing is named with a remedy, and how to supply it on
    /// this machine is the operator's judgment, because choosing a package manager for an unknown
    /// machine is not something a carried document should decide.
    ///
    /// Machine requirements are evaluated against the invocation's working directory and
    /// environment, while `--store` selects where the runbook's documents are read from. The two
    /// differ whenever `--store` is used, and the report names both.
    Preflight(PreflightArgs),

    /// Report which of a change's artifacts exist and how much of its work is done.
    Status(StatusArgs),

    /// Report whether a change's contract reached the accepted specifications.
    ///
    /// The operation archival tells a contributor to run before archiving. It judges each
    /// requirement by the change state the change declares on it: one the change adds or
    /// modifies is compared with the accepted requirement as text, one it withdraws must no
    /// longer be held, and one it carries forward is compared by identifier. A stated rename is
    /// compared by heading. It reports where content differs, never what either side says, and
    /// never edits an accepted specification: the merge itself is an agent's work, following the
    /// `sync` stage's instructions.
    Sync(SyncArgs),

    /// Show each requirement a change adds or modifies beside the accepted one.
    ///
    /// Reads the change's specification and the accepted specifications, and writes nothing. For
    /// each requirement whose content `sync` compares, it shows the paragraphs and scenarios held
    /// by both as counted, and quotes the rest from the document that holds each, with the words
    /// only one side holds marked on that side. A requirement the change withdraws or carries
    /// forward is listed. Nothing it prints is merge text: the merge is an agent's work, following
    /// the `sync` stage's instructions.
    Show(ShowArgs),

    /// Register and manage standalone projects on this machine.
    ///
    /// A store is an HTMLSpec project in its own repository. Registering one lets the
    /// operations that read a project act on it from anywhere, with `--store <id>`.
    Store {
        #[command(subcommand)]
        what: StoreCommand,
    },

    /// Print the workflow instructions for a stage.
    ///
    /// The same text `init` installs, from the same table, so an agent reads the current
    /// instructions without knowing which tool's directory holds a copy.
    Instructions(InstructionsArgs),

    /// Create a document or a change that validates immediately.
    ///
    /// References the project's existing asset store, so a new document renders offline
    /// with no further step and the project keeps one copy of its runtime.
    New {
        #[command(subcommand)]
        what: NewCommand,
    },

    /// Bring an existing OpenSpec project into HTMLSpec.
    ///
    /// Two exact operations with an agent's judgment between them. `survey` enumerates what a
    /// source project holds and writes nothing; `promote` checks that a plan accounts for
    /// everything the survey found and moves the staged project into place. Reading each
    /// document, deciding what it becomes, and writing it are the `migrate` workflow's work,
    /// because a proposal and a design carry no structure a parser could read.
    ///
    /// There is deliberately no subcommand that converts a document. One would produce a
    /// project that passes every check and has lost the reasoning it was migrating.
    Migrate {
        #[command(subcommand)]
        what: MigrateCommand,
    },

    // The five request verbs. Hidden from the list of operations above, because the top-level
    // help lists them under a heading of their own after it, and their help, and their objects',
    // is written by `operation::with_request_verbs` from the table of spellings.
    #[command(hide = true)]
    Get {
        #[command(subcommand)]
        what: GetObject,
    },
    #[command(hide = true)]
    Post {
        #[command(subcommand)]
        what: PostObject,
    },
    #[command(hide = true)]
    Put {
        #[command(subcommand)]
        what: PutObject,
    },
    #[command(hide = true)]
    Patch {
        #[command(subcommand)]
        what: PatchObject,
    },
    #[command(hide = true)]
    Delete {
        #[command(subcommand)]
        what: DeleteObject,
    },
}

/// What `migrate` does.
#[derive(Debug, Subcommand)]
enum MigrateCommand {
    /// Enumerate an OpenSpec project without writing anything.
    ///
    /// Reports every entity a plan owes a disposition for. With `--json`, emits the payload
    /// the coverage check compares a plan against — so the plan is checked against the list it
    /// was written from rather than against a second enumeration taken later.
    ///
    /// Also reports each change's own configuration: whether it holds a `.openspec.yaml`, every
    /// key that file sets, and the change it follows, which the migrated change states as
    /// `FOLLOWS`. No other value in the file is read.
    Survey(MigrateSurveyArgs),

    /// Check that a migration plan accounts for everything the survey found.
    ///
    /// The coverage comparison is promotion's first act, so a plan missing an entity kind is
    /// reported before anything is read from the staging tree. The check names every unaccounted
    /// entity rather than the first, and says nothing about whether a disposition is *right* —
    /// that judgment is what the plan is put in front of a contributor for.
    Promote(MigratePromoteArgs),
}

/// What `store` does.
#[derive(Debug, Subcommand)]
enum StoreCommand {
    /// Register a project under an identifier.
    Add(StoreAddArgs),
    /// List the registered projects.
    List(StoreListArgs),
    /// Remove a registration.
    ///
    /// Removes the registration only. The project it named is untouched.
    Remove(StoreRemoveArgs),
}

/// What `new` creates.
///
/// A nested subcommand rather than a `--change` flag conflicting with `--kind`, because
/// the two forms are a sum type: a change is a directory of several kinded documents,
/// and a document is one of a named kind. Two mutually exclusive flags encode that as a
/// pair of booleans and let a contributor ask for both.
#[derive(Debug, Subcommand)]
enum NewCommand {
    /// Create a change: a directory holding a proposal, a design, and a tasks document.
    ///
    /// Placed where the project keeps its active changes, and the resolved location is
    /// reported. The specification is not written — it is produced at archival, from the
    /// completed change.
    Change(NewChangeArgs),

    /// Create one document of a given kind.
    ///
    /// Each kind arrives in the shape that kind needs: a proposal as prose with no unit
    /// scaffolding, a design with no imposed structure at all, and a specification with
    /// the units its kind expects.
    Document(NewDocumentArgs),
}

// The objects of each request verb. Each carries the `Args` struct of the operation it runs, so it
// takes that operation's arguments and options, and nothing else. An object's help is written by
// `operation::with_request_verbs`, from its operation's.

/// The objects of `get`: operations that never write.
#[derive(Debug, Subcommand)]
enum GetObject {
    Validity(ValidateArgs),
    List(ListArgs),
    Status(StatusArgs),
    Sync(SyncArgs),
    Show(ShowArgs),
    Assets(AssetsArgs),
    Preflight(PreflightArgs),
    Instructions(InstructionsArgs),
    Registrations(StoreListArgs),
    Survey(MigrateSurveyArgs),
}

/// The objects of `post`: operations that create something that was not there.
#[derive(Debug, Subcommand)]
enum PostObject {
    Change(NewChangeArgs),
    Document(NewDocumentArgs),
    Registration(StoreAddArgs),
    Run(RunArgs),
    Promotion(MigratePromoteArgs),
}

/// The objects of `put`: operations that create or replace a whole thing, and leave it as one run
/// did when run again.
#[derive(Debug, Subcommand)]
enum PutObject {
    Project(InitArgs),
    Format(FormatArgs),
}

/// The objects of `patch`: operations that change something that exists.
#[derive(Debug, Subcommand)]
enum PatchObject {
    Archive(ArchiveArgs),
}

/// The objects of `delete`: operations that remove something.
#[derive(Debug, Subcommand)]
enum DeleteObject {
    Registration(StoreRemoveArgs),
}

// Each operation's arguments and options, carried by both of its spellings. These structs carry no
// doc comment of their own, because clap would read one as the operation's description; the
// operation's variant above describes it.

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct ValidateArgs {
    /// The file or directory to validate. Defaults to the current directory.
    path: Option<PathBuf>,

    /// Treat warnings as errors.
    ///
    /// Review markers are the exception: they report that a dependent unit
    /// references something this change altered, and the contract says such a
    /// report never gates work. Strict mode does not promote them.
    #[arg(long)]
    strict: bool,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct FormatArgs {
    /// The file or directory to format. Defaults to the current directory.
    path: Option<PathBuf>,

    /// Report which documents are not canonical without modifying any file.
    ///
    /// Exits nonzero if any document would change, so it works as a
    /// continuous-integration gate.
    #[arg(long)]
    check: bool,

    /// Also give each document the fallback stylesheet, so it stays readable
    /// where its stylesheet cannot load, as in Safari opening it from disk.
    ///
    /// Adds the current fallback where it is absent and replaces one that
    /// differs. Leaves archived changes and run records as written, because
    /// they are history. Without this flag, formatting never adds, changes, or
    /// removes a fallback. Adding it to a runbook changes its canonical
    /// documents, so its next archival asks for a new runbook version. With
    /// --check, reports each document that lacks the current fallback.
    #[arg(long)]
    fallback: bool,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct AssetsArgs {
    /// The project root to check. Defaults to the current directory.
    path: Option<PathBuf>,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct ArchiveArgs {
    /// The change directory to archive, or a change name inside the project.
    ///
    /// With no argument the project's active changes are consulted: one is selected and
    /// announced, and several are listed rather than guessed between.
    path: Option<PathBuf>,

    /// Report whether the change is ready without archiving it.
    ///
    /// Applies every condition archival applies and performs none of them, so it works
    /// as a continuous-integration gate. Exits nonzero exactly where archival refuses.
    ///
    /// One exception, and it comes first: an archival interrupted earlier is settled before
    /// the report, completed where its change moved and rolled back where it did not. A
    /// report on a project with a half-finished transaction would describe a state nothing
    /// had accounted for.
    #[arg(long)]
    check: bool,

    /// Archive over findings without asking.
    ///
    /// Incomplete tasks, and any part of the change's contract that has not reached the
    /// accepted specifications (a requirement in none of them, or a rename, a requirement's
    /// content, or a withdrawal they have not taken), are reported rather than refused, and
    /// archival asks before proceeding. This answers in advance, which is how the operation
    /// is driven where nothing can answer.
    #[arg(long)]
    yes: bool,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct InitArgs {
    /// Which tools to install instructions for: `all`, `none`, or a comma-separated
    /// list. Passing this skips the interactive selection.
    #[arg(long)]
    tools: Option<String>,

    /// Where to establish the project. Defaults to the current directory.
    #[arg(long)]
    root: Option<PathBuf>,

    /// The project's name, recorded in its declaration. Defaults to the directory
    /// name.
    #[arg(long)]
    name: Option<String>,

    /// A path, accepted only to report that document creation moved to `new`.
    ///
    /// `init <path>` created a document until this release. Hidden from help,
    /// because it is not an argument to use — it exists so the withdrawn spelling
    /// names its replacement rather than producing a bare usage error.
    #[arg(hide = true)]
    withdrawn: Option<PathBuf>,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct ListArgs {
    /// List the project's accepted specifications instead of its changes.
    #[arg(long)]
    specs: bool,

    /// List the project's runbooks instead of its changes.
    ///
    /// A flag beside `--specs` rather than a command of its own: listing what the
    /// project holds is this operation, and a second command would invent a second
    /// vocabulary for it.
    #[arg(long)]
    runbooks: bool,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct RunArgs {
    /// The runbook to run, by name.
    runbook: String,

    /// What to call this run's record. Required: an operation that invented a name would
    /// choose on your behalf, and a date collides while a sequence number derives an ordering
    /// from a directory's contents.
    #[arg(long = "as")]
    record: Option<String>,

    /// Report what this runbook's runs recorded, and write nothing.
    #[arg(long)]
    check: bool,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct PreflightArgs {
    /// The runbook to check, by name.
    runbook: String,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct StatusArgs {
    /// The change to report on, by name or by directory.
    #[arg(long)]
    change: String,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct SyncArgs {
    /// The change to compare, by name or by directory.
    #[arg(long)]
    change: String,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct ShowArgs {
    /// The change to show, by name or by directory.
    change: String,

    /// Show each requirement the change adds or modifies beside the accepted one. The one view
    /// `show` has.
    #[arg(long)]
    diff: bool,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct StoreAddArgs {
    /// The identifier to register it as.
    id: String,
    /// The project's location.
    path: PathBuf,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct StoreListArgs {}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct StoreRemoveArgs {
    /// The identifier to remove.
    id: String,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct InstructionsArgs {
    /// The stage: explore, new, continue, ff, apply, verify, or archive.
    stage: String,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct NewChangeArgs {
    /// The change's name, used as its directory name.
    name: String,

    /// The change's subject, used as each document's title and heading.
    #[arg(long)]
    subject: Option<String>,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct NewDocumentArgs {
    /// The artifact kind: proposal, design, tasks, or specification.
    kind: String,

    /// Where to write the document.
    path: PathBuf,

    /// The document's subject, used as its title and heading.
    #[arg(long, default_value = "Untitled document")]
    subject: String,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct MigrateSurveyArgs {
    /// The OpenSpec project to survey. Defaults to the current directory.
    path: Option<PathBuf>,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
struct MigratePromoteArgs {
    /// The plan: a `tasks` document, one task unit per surveyed entity.
    plan: PathBuf,

    /// The survey payload the plan was written from, as `migrate survey --json` emitted it.
    #[arg(long)]
    survey: PathBuf,

    /// The staged project. Without it, only the coverage check runs.
    ///
    /// Optional so the coverage check is usable while the plan is still being written and
    /// nothing has been staged yet. With it, the staged tree is checked structurally, the
    /// project's own validation, formatting, and asset checks run over the complete tree, and
    /// the project is moved into place.
    #[arg(long)]
    staged: Option<PathBuf>,

    /// Where to promote into. Defaults to the current directory.
    #[arg(long)]
    into: Option<PathBuf>,

    /// Run every check and move nothing.
    ///
    /// The whole operation up to the move, so a contributor sees exactly what promotion would
    /// say with the destination unchanged. Exits nonzero exactly where promotion refuses.
    #[arg(long)]
    check: bool,
}

/// One operation with its arguments, whichever of its two spellings named it.
///
/// One variant per [`Operation`](operation::Operation), carrying that operation's `Args` struct, so an invocation cannot
/// pair one operation with another's arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Invocation {
    Validate(ValidateArgs),
    Format(FormatArgs),
    Assets(AssetsArgs),
    Archive(ArchiveArgs),
    Init(InitArgs),
    List(ListArgs),
    Run(RunArgs),
    Preflight(PreflightArgs),
    Status(StatusArgs),
    Sync(SyncArgs),
    Show(ShowArgs),
    StoreAdd(StoreAddArgs),
    StoreList(StoreListArgs),
    StoreRemove(StoreRemoveArgs),
    Instructions(InstructionsArgs),
    NewChange(NewChangeArgs),
    NewDocument(NewDocumentArgs),
    MigrateSurvey(MigrateSurveyArgs),
    MigratePromote(MigratePromoteArgs),
}

impl From<Command> for Invocation {
    /// Both spellings of each operation, in one arm: the OpenSpec spelling, then the request verb.
    fn from(command: Command) -> Self {
        match command {
            Command::Validate(arguments)
            | Command::Get {
                what: GetObject::Validity(arguments),
            } => Self::Validate(arguments),
            Command::Format(arguments)
            | Command::Put {
                what: PutObject::Format(arguments),
            } => Self::Format(arguments),
            Command::Assets(arguments)
            | Command::Get {
                what: GetObject::Assets(arguments),
            } => Self::Assets(arguments),
            Command::Archive(arguments)
            | Command::Patch {
                what: PatchObject::Archive(arguments),
            } => Self::Archive(arguments),
            Command::Init(arguments)
            | Command::Put {
                what: PutObject::Project(arguments),
            } => Self::Init(arguments),
            Command::List(arguments)
            | Command::Get {
                what: GetObject::List(arguments),
            } => Self::List(arguments),
            Command::Run(arguments)
            | Command::Post {
                what: PostObject::Run(arguments),
            } => Self::Run(arguments),
            Command::Preflight(arguments)
            | Command::Get {
                what: GetObject::Preflight(arguments),
            } => Self::Preflight(arguments),
            Command::Status(arguments)
            | Command::Get {
                what: GetObject::Status(arguments),
            } => Self::Status(arguments),
            Command::Sync(arguments)
            | Command::Get {
                what: GetObject::Sync(arguments),
            } => Self::Sync(arguments),
            Command::Show(arguments)
            | Command::Get {
                what: GetObject::Show(arguments),
            } => Self::Show(arguments),
            Command::Store {
                what: StoreCommand::Add(arguments),
            }
            | Command::Post {
                what: PostObject::Registration(arguments),
            } => Self::StoreAdd(arguments),
            Command::Store {
                what: StoreCommand::List(arguments),
            }
            | Command::Get {
                what: GetObject::Registrations(arguments),
            } => Self::StoreList(arguments),
            Command::Store {
                what: StoreCommand::Remove(arguments),
            }
            | Command::Delete {
                what: DeleteObject::Registration(arguments),
            } => Self::StoreRemove(arguments),
            Command::Instructions(arguments)
            | Command::Get {
                what: GetObject::Instructions(arguments),
            } => Self::Instructions(arguments),
            Command::New {
                what: NewCommand::Change(arguments),
            }
            | Command::Post {
                what: PostObject::Change(arguments),
            } => Self::NewChange(arguments),
            Command::New {
                what: NewCommand::Document(arguments),
            }
            | Command::Post {
                what: PostObject::Document(arguments),
            } => Self::NewDocument(arguments),
            Command::Migrate {
                what: MigrateCommand::Survey(arguments),
            }
            | Command::Get {
                what: GetObject::Survey(arguments),
            } => Self::MigrateSurvey(arguments),
            Command::Migrate {
                what: MigrateCommand::Promote(arguments),
            }
            | Command::Post {
                what: PostObject::Promotion(arguments),
            } => Self::MigratePromote(arguments),
        }
    }
}

impl Invocation {
    /// The operation this invokes.
    ///
    /// Read by the tests that hold both spellings of each operation to that operation. Nothing at
    /// runtime needs it, because the variant already is the operation; were it compiled in, an
    /// operation constructed only here would escape the dead-code check that keeps every operation
    /// in [`Operation::ALL`](operation::Operation::ALL).
    #[cfg(test)]
    fn operation(&self) -> operation::Operation {
        use operation::Operation;
        match self {
            Self::Validate(_) => Operation::Validate,
            Self::Format(_) => Operation::Format,
            Self::Assets(_) => Operation::Assets,
            Self::Archive(_) => Operation::Archive,
            Self::Init(_) => Operation::Init,
            Self::List(_) => Operation::List,
            Self::Run(_) => Operation::Run,
            Self::Preflight(_) => Operation::Preflight,
            Self::Status(_) => Operation::Status,
            Self::Sync(_) => Operation::Sync,
            Self::Show(_) => Operation::Show,
            Self::StoreAdd(_) => Operation::StoreAdd,
            Self::StoreList(_) => Operation::StoreList,
            Self::StoreRemove(_) => Operation::StoreRemove,
            Self::Instructions(_) => Operation::Instructions,
            Self::NewChange(_) => Operation::NewChange,
            Self::NewDocument(_) => Operation::NewDocument,
            Self::MigrateSurvey(_) => Operation::MigrateSurvey,
            Self::MigratePromote(_) => Operation::MigratePromote,
        }
    }

    /// Runs the operation, whichever spelling named it.
    fn run<O: std::io::Write, E: std::io::Write>(
        self,
        store: Option<&str>,
        streams: &mut Streams<O, E>,
    ) -> exit::Outcome {
        match self {
            Self::Validate(ValidateArgs { path, strict }) => commands::validate(
                path.as_deref(),
                if strict {
                    Strictness::Strict
                } else {
                    Strictness::Default
                },
                streams,
            ),
            Self::Format(FormatArgs {
                path,
                check,
                fallback,
            }) => commands::format(
                path.as_deref(),
                if check {
                    FormatMode::Check
                } else {
                    FormatMode::Write
                },
                if fallback {
                    FallbackRequest::Apply
                } else {
                    FallbackRequest::Leave
                },
                streams,
            ),
            Self::Assets(AssetsArgs { path }) => commands::assets(path.as_deref(), streams),
            Self::List(ListArgs { specs, runbooks }) => {
                commands::list(specs, runbooks, store, streams)
            }
            Self::StoreAdd(StoreAddArgs { id, path }) => commands::store_add(&id, &path, streams),
            Self::StoreList(StoreListArgs {}) => commands::store_list(streams),
            Self::StoreRemove(StoreRemoveArgs { id }) => commands::store_remove(&id, streams),
            Self::Instructions(InstructionsArgs { stage }) => {
                commands::instructions(&stage, streams)
            }
            Self::Run(RunArgs {
                runbook,
                record,
                check,
            }) => match (check, record) {
                // The two modes are exclusive rather than composable, and the pairing is checked
                // here rather than expressed as a clap conflict: a contributor who passed both
                // asked to start a run and to report on one, and naming which of the two is
                // missing or surplus is a better report than clap's generic conflict message.
                (true, None) => runs::check(&runbook, store, streams),
                (false, Some(record)) => runs::start(&runbook, &record, store, streams),
                (true, Some(_)) => {
                    let _ = streams.failure(
                        "--check reports on the runs a runbook already has, so it takes no --as. \
                         Drop one: `htmlspec run <runbook> --check` to report, or `htmlspec run \
                         <runbook> --as <name>` to start one",
                    );
                    exit::Outcome::OperationalFailure
                }
                (false, None) => {
                    let _ = streams.failure(
                        "starting a run needs a name for its record: `htmlspec run <runbook> --as \
                         <name>`. There is no default, because a date collides when a procedure \
                         runs twice in a day and a sequence number derives an ordering from a \
                         directory's contents — which a runbook's name is forbidden to do. To \
                         report on the runs that exist, pass --check",
                    );
                    exit::Outcome::OperationalFailure
                }
            },
            Self::Preflight(PreflightArgs { runbook }) => {
                preflight::command(&runbook, store, streams)
            }
            Self::Status(StatusArgs { change }) => commands::status(&change, store, streams),
            Self::Sync(SyncArgs { change }) => commands::sync(&change, store, streams),
            // A missing `--diff` is named here, as `run` names a missing `--as`, rather than left to
            // clap's generic message: the bare form is kept free for a later view, and refusing it
            // costs the contributor one line that names the fix.
            Self::Show(ShowArgs { change, diff }) => {
                if diff {
                    show::command(&change, store, streams)
                } else {
                    let _ = streams.failure(show::ONE_VIEW);
                    exit::Outcome::OperationalFailure
                }
            }
            Self::Archive(ArchiveArgs { path, check, yes }) => commands::archive(
                path.as_deref(),
                if check {
                    commands::ArchiveMode::Check
                } else {
                    commands::ArchiveMode::Perform
                },
                yes,
                streams,
            ),
            Self::Init(InitArgs {
                tools,
                root,
                name,
                withdrawn,
            }) => commands::init(
                root.as_deref(),
                name.as_deref(),
                tools.as_deref(),
                withdrawn.as_deref(),
                streams,
            ),
            Self::NewChange(NewChangeArgs { name, subject }) => {
                commands::new_change(&name, subject.as_deref(), streams)
            }
            Self::NewDocument(NewDocumentArgs {
                kind,
                path,
                subject,
            }) => commands::new_document(&kind, &path, &subject, streams),
            Self::MigrateSurvey(MigrateSurveyArgs { path }) => {
                commands::migrate_survey(path.as_deref(), streams)
            }
            Self::MigratePromote(MigratePromoteArgs {
                plan,
                survey,
                staged,
                into,
                check,
            }) => commands::migrate_promote(
                &plan,
                &survey,
                staged.as_deref(),
                into.as_deref(),
                if check {
                    commands::PromoteMode::Check
                } else {
                    commands::PromoteMode::Perform
                },
                streams,
            ),
        }
    }
}

/// The command line as the program presents it: clap's declaration, with the request verbs'
/// help written from the table of spellings.
fn command() -> clap::Command {
    operation::with_request_verbs(Cli::command())
}

/// Parses the process's arguments, exiting with clap's report where they do not parse.
///
/// What `Cli::parse` does, against [`command`] rather than the bare declaration, so the help a
/// contributor asks for is the help the request verbs were given.
fn parse() -> Cli {
    let mut command = command();
    let matches = command.get_matches_mut();
    Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.format(&mut command).exit())
}

fn main() {
    let cli = parse();
    let form = if cli.json {
        Form::Structured
    } else {
        Form::Human
    };
    let mut streams = Streams::process(form);
    let outcome = Invocation::from(cli.command).run(cli.store.as_deref(), &mut streams);

    // The one exit point.
    std::process::exit(outcome.code());
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::operation::{Operation, Verb};

    /// `clap`'s own consistency check: every argument is well-formed, no two share a
    /// short flag, every help string is present. Run on the command as the program presents it.
    #[test]
    fn the_command_definition_is_internally_consistent() {
        command().debug_assert();
    }

    /// Every subcommand and option is documented, which is the discoverability
    /// requirement. Checked against the rendered help rather than the definition, so
    /// a subcommand added without a doc comment fails here.
    #[test]
    fn help_documents_every_subcommand_and_option() {
        let mut presented = command();
        let help = presented.render_long_help().to_string();
        for subcommand in [
            "validate",
            "format",
            "assets",
            "archive",
            "init",
            "new",
            "list",
            "preflight",
            "run",
            "status",
            "sync",
            "show",
            "instructions",
            "store",
            "migrate",
        ] {
            assert!(
                help.contains(subcommand),
                "the top-level help omits {subcommand}:\n{help}"
            );
        }
        for verb in Verb::ALL {
            assert!(
                help.contains(&format!("\n  {} ", verb.word())),
                "the top-level help omits the verb {}:\n{help}",
                verb.word()
            );
        }
        assert!(help.contains("--json"), "{help}");

        // And each subcommand's own options are documented. `build()` first, because
        // a global argument is propagated into subcommands during the build and a
        // subcommand rendered before that reports only its own options — which would
        // make this assert less than it appears to.
        let expected: &[(&str, &[&str])] = &[
            ("validate", &["--strict", "--json"]),
            ("format", &["--check", "--json"]),
            ("assets", &["--json"]),
            ("init", &["--tools", "--root", "--name", "--json"]),
            ("new", &["change", "document"]),
            ("migrate", &["survey", "promote"]),
            ("preflight", &["--json", "--store"]),
            ("run", &["--as", "--check", "--json"]),
            ("show", &["--diff", "--json", "--store"]),
            ("get", &["validity", "registrations", "survey"]),
            ("post", &["change", "registration", "promotion"]),
            ("put", &["project", "format"]),
            ("patch", &["archive"]),
            ("delete", &["registration"]),
        ];
        let mut built = command();
        built.build();
        for (name, options) in expected {
            let subcommand = built
                .find_subcommand_mut(name)
                .expect("the subcommand exists");
            let help = subcommand.render_long_help().to_string();
            for option in *options {
                assert!(
                    help.contains(option),
                    "{name}'s help omits {option}:\n{help}"
                );
            }
        }
    }

    /// Every path of subcommands beneath `command`, as words, down to the commands that take
    /// none. clap's own `help` is not an operation and is left out.
    fn paths(command: &clap::Command) -> BTreeSet<Vec<String>> {
        let mut found = BTreeSet::new();
        for subcommand in command.get_subcommands() {
            let name = subcommand.get_name().to_owned();
            if name == "help" {
                continue;
            }
            let beneath = paths(subcommand);
            if beneath.is_empty() {
                found.insert(vec![name]);
            } else {
                for mut path in beneath {
                    path.insert(0, name.clone());
                    found.insert(path);
                }
            }
        }
        found
    }

    /// What clap offers is exactly the operations, under both spellings.
    ///
    /// Every path beneath an OpenSpec subcommand is one operation's OpenSpec spelling, and every
    /// path beneath a verb is one operation's request-verb spelling, and each operation has both.
    /// This closes what `operation.rs`'s listing test leaves open: an operation declared to clap
    /// and missing from [`Operation::ALL`] has a path here that no listed operation spells.
    #[test]
    fn the_command_line_offers_exactly_the_operations_under_both_spellings() {
        let offered = paths(&command());
        let spelled: BTreeSet<Vec<String>> = Operation::ALL
            .into_iter()
            .flat_map(|operation| {
                let (verb, object) = operation.request();
                [
                    operation
                        .spelling()
                        .iter()
                        .map(|word| (*word).to_owned())
                        .collect::<Vec<_>>(),
                    vec![verb.word().to_owned(), object.to_owned()],
                ]
            })
            .collect();
        assert_eq!(
            offered, spelled,
            "the command line and the table of spellings disagree"
        );
    }

    /// Every argument and option an operation takes, for the parse tests below.
    ///
    /// The match has no wildcard, so an operation added later needs its arguments here.
    /// `every_option_is_given_in_the_parse_tests` checks that each option clap declares is here.
    fn every_option(operation: Operation) -> &'static [&'static str] {
        match operation {
            Operation::Validate => &["doc.html", "--strict"],
            Operation::Format => &["doc.html", "--check", "--fallback"],
            Operation::Assets => &["root"],
            Operation::Archive => &["a-change", "--check", "--yes"],
            Operation::Init => &[
                "--tools",
                "none",
                "--root",
                "root",
                "--name",
                "name",
                "withdrawn.html",
            ],
            Operation::List => &["--specs", "--runbooks"],
            Operation::Run => &["release", "--as", "first", "--check"],
            Operation::Preflight => &["release"],
            Operation::Status => &["--change", "a-change"],
            Operation::Sync => &["--change", "a-change"],
            Operation::Show => &["a-change", "--diff"],
            Operation::StoreAdd => &["an-id", "a/path"],
            Operation::StoreList => &[],
            Operation::StoreRemove => &["an-id"],
            Operation::Instructions => &["apply"],
            Operation::NewChange => &["a-change", "--subject", "A subject"],
            Operation::NewDocument => &["proposal", "p.html", "--subject", "A subject"],
            Operation::MigrateSurvey => &["source"],
            Operation::MigratePromote => &[
                "plan.html",
                "--survey",
                "survey.json",
                "--staged",
                "staging",
                "--into",
                "into",
                "--check",
            ],
        }
    }

    /// The parse of `words`, then `arguments`, then the global options, as the program parses it.
    fn parsed(words: &[&str], arguments: &[&str]) -> Cli {
        let argv: Vec<&str> = ["htmlspec"]
            .into_iter()
            .chain(words.iter().copied())
            .chain(arguments.iter().copied())
            .chain(["--json", "--store", "a-store"])
            .collect();
        let matches = command()
            .try_get_matches_from(&argv)
            .unwrap_or_else(|error| panic!("{argv:?} does not parse: {error}"));
        Cli::from_arg_matches(&matches).unwrap_or_else(|error| panic!("{argv:?}: {error}"))
    }

    /// Both spellings of every operation parse to that operation, with the same arguments.
    ///
    /// Every option the operation takes is given, so a verb spelling that ran another operation,
    /// dropped an option, or read one differently would parse to a different invocation.
    #[test]
    fn both_spellings_of_every_operation_parse_to_one_invocation() {
        for operation in Operation::ALL {
            let (verb, object) = operation.request();
            let arguments = every_option(operation);
            let by_name = parsed(operation.spelling(), arguments);
            let by_verb = parsed(&[verb.word(), object], arguments);
            assert!(by_verb.json && by_verb.store.as_deref() == Some("a-store"));
            let named = Invocation::from(by_name.command);
            let verbed = Invocation::from(by_verb.command);
            assert_eq!(
                named.operation(),
                operation,
                "`htmlspec {}` parses to another operation",
                operation.spelled()
            );
            assert_eq!(
                named,
                verbed,
                "`htmlspec {} {object}` parses otherwise than `htmlspec {}`",
                verb.word(),
                operation.spelled()
            );
        }
    }

    /// Each option clap declares for an operation is among those the parse test gives it, so the
    /// parse test cannot pass by leaving an option out.
    #[test]
    fn every_option_is_given_in_the_parse_tests() {
        let presented = command();
        for operation in Operation::ALL {
            let declared = operation
                .spelling()
                .iter()
                .try_fold(&presented, |current, word| current.find_subcommand(word))
                .expect("every operation is declared");
            let given = every_option(operation);
            for argument in declared.get_arguments() {
                if let Some(long) = argument.get_long() {
                    assert!(
                        given.contains(&format!("--{long}").as_str()),
                        "the parse test does not give `htmlspec {}` its --{long}",
                        operation.spelled()
                    );
                }
            }
            let positionals = declared.get_positionals().count();
            let given_positionals = given
                .iter()
                .enumerate()
                .filter(|(at, word)| {
                    !word.starts_with("--")
                        && !at
                            .checked_sub(1)
                            .and_then(|before| given.get(before))
                            .is_some_and(|before| {
                                before.starts_with("--")
                                    && declared.get_arguments().any(|argument| {
                                        argument.get_long() == before.strip_prefix("--")
                                            && argument.get_action().takes_values()
                                    })
                            })
                })
                .count();
            assert_eq!(
                given_positionals,
                positionals,
                "the parse test does not give `htmlspec {}` each of its positional arguments",
                operation.spelled()
            );
        }
    }

    #[test]
    fn the_reported_version_names_the_crate_version() {
        assert!(
            VERSION.starts_with(env!("CARGO_PKG_VERSION")),
            "the version does not name the crate version: {VERSION}"
        );
        // And it says something about the revision either way, so a build can always
        // be traced or identified as untraceable.
        assert!(VERSION.contains('('), "{VERSION}");
    }

    /// The default invocations are the ones the specs call configuration-free.
    #[test]
    fn the_defaults_are_whole_project_and_human_readable() {
        let cli = Cli::try_parse_from(["htmlspec", "validate"]).expect("parses");
        assert!(!cli.json);
        assert!(matches!(
            cli.command,
            Command::Validate(ValidateArgs {
                path: None,
                strict: false
            })
        ));

        let cli = Cli::try_parse_from(["htmlspec", "format"]).expect("parses");
        assert!(matches!(
            cli.command,
            Command::Format(FormatArgs {
                path: None,
                check: false,
                fallback: false
            })
        ));
    }

    /// An unknown subcommand is an error that points the reader at the real ones.
    ///
    /// This asserted that the error text named every subcommand, and it passed only by
    /// accident: clap lists *similar* subcommands, and with three of them the fuzzy match
    /// against `lint` happened to return all three. Adding a fourth changed the match and
    /// the assertion failed, having never tested what it claimed.
    ///
    /// So it now asserts what clap actually contracts — the input is rejected, the
    /// unknown name is echoed, and the reader is directed to `--help`, which is where the
    /// full list lives. `help_documents_every_subcommand_and_option` asserts that list.
    #[test]
    fn an_unknown_subcommand_is_rejected_and_points_at_help() {
        let error = Cli::try_parse_from(["htmlspec", "lint"]).expect_err("rejected");
        let rendered = error.to_string();
        assert!(
            rendered.contains("lint"),
            "the error does not echo the unknown subcommand:\n{rendered}"
        );
        assert!(
            rendered.contains("--help"),
            "the error does not direct the reader to the subcommand list:\n{rendered}"
        );
    }

    /// `init` takes no required argument, because its subject is the project a
    /// contributor is standing in. This is the first defect the change was written
    /// against: `htmlspec init` used to fail with "the following required arguments were
    /// not provided: <PATH>", so there was no invocation that meant "set this repository
    /// up".
    #[test]
    fn init_takes_no_required_argument() {
        assert!(
            Cli::try_parse_from(["htmlspec", "init"]).is_ok(),
            "bare `init` must bootstrap the project rather than demand an argument"
        );
    }

    /// The withdrawn spelling still parses, so the program can name its replacement.
    ///
    /// `init <path>` created a document until this release. Rejecting it at the parser
    /// would produce a usage line, and a reader who used the old spelling yesterday needs
    /// to be told where document creation went.
    #[test]
    fn the_withdrawn_init_path_parses_so_it_can_be_reported() {
        let cli = Cli::try_parse_from(["htmlspec", "init", "doc.html"]).expect("parses");
        assert!(
            matches!(
                cli.command,
                Command::Init(InitArgs {
                    withdrawn: Some(_),
                    ..
                })
            ),
            "the path was not captured for reporting"
        );
    }

    /// Document creation lives under `new`, in the two forms a change and a document are.
    #[test]
    fn new_creates_a_change_or_a_document() {
        assert!(Cli::try_parse_from(["htmlspec", "new", "change", "add-dark-mode"]).is_ok());
        assert!(Cli::try_parse_from(["htmlspec", "new", "document", "proposal", "p.html"]).is_ok());
        // A change takes a name, and a document takes both a kind and a path.
        assert!(Cli::try_parse_from(["htmlspec", "new", "change"]).is_err());
        assert!(Cli::try_parse_from(["htmlspec", "new", "document", "proposal"]).is_err());
        // And `new` alone is not a command.
        assert!(Cli::try_parse_from(["htmlspec", "new"]).is_err());
    }

    /// `--json` is global, so it works before or after the subcommand. An agent
    /// should not have to remember which.
    #[test]
    fn the_json_flag_is_accepted_on_either_side_of_the_subcommand() {
        for argv in [
            vec!["htmlspec", "--json", "validate"],
            vec!["htmlspec", "validate", "--json"],
            vec!["htmlspec", "--json", "get", "validity"],
            vec!["htmlspec", "get", "validity", "--json"],
        ] {
            let cli = Cli::try_parse_from(&argv).expect("parses");
            assert!(cli.json, "{argv:?} did not set the flag");
        }
    }
}
