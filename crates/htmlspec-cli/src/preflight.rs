//! What a runbook requires of a machine, and whether this machine satisfies it.
//!
//! The subject is a **contract rather than a bundle**: a runbook states what it needs and this
//! module reports what is absent, instead of the runbook carrying copies of the things it needs.
//! `runbook-portability` states the boundary, and the reason it exists is measured in this
//! repository — a vendored copy of generated content goes stale the moment its source moves, which
//! is what `scripts/check-installed` was written to catch.
//!
//! # Three separations, each one load-bearing
//!
//! **Declaration from evaluation.** [`parse_contract`] reads the wire format a runbook's
//! requirement notes carry and produces a [`MachineContract`]. Nothing downstream sees the JSON:
//! a raw map handed to an evaluator is a second parser waiting to be written, and the design
//! forbids asking downstream code to reparse notes.
//!
//! **Facts from judgment.** A probe measures and returns a [`Fact`]; [`assess`] compares the facts
//! against what was declared and produces the verdict. A tool reporting `3.12.0` where `3.13.2` was
//! declared is one measured fact and one unmet condition, and folding them into a single "failed"
//! value would lose the two numbers the report has to name.
//!
//! **Readiness from publication.** [`PreflightAssessment`] is a report. It carries no token, no
//! path, and no constructor that could turn it into permission: [`crate::runs::prepare_run`]
//! observes again and builds its own [`crate::runs::ReadyRun`]. `runbook-portability` requires that
//! "a standalone preflight success SHALL NOT be reusable as permission to skip this check", and
//! this module cannot express reuse because it holds nothing reusable.
//!
//! # Capability discipline
//!
//! [`HostProbe`] is the only thing here that can touch a machine, and it is passed in rather than
//! reached for. Parsing, assessment, and report assembly are pure over their inputs, so the tests
//! below inject observations and never depend on what the developer happens to have installed —
//! which `runbook-portability` needs, because a suite that passes only on a machine with `cargo`
//! would report a missing tool as a passing check.

use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::terminal::{first_hidden, shown};
use crate::version::{RunbookVersion, ToolVersion};

pub mod host;

/// The marker a machine-check declaration's note begins with.
///
/// An explicit, narrow use of an existing string as a wire format, and the proposal's
/// no-schema-change boundary is what makes it necessary: a machine requirement is an ordinary
/// requirement unit with an ordinary note, because the vocabulary is closed and this change adds
/// nothing to it.
pub const MARKER: &str = "preflight/v1";

/// The marker of a declaration that may carry a remedy: every form [`MARKER`] has, with an optional
/// `remedy` on each.
///
/// A version of its own rather than a field added to v1, which is the change
/// `ship-the-crates-notices-and-runbook-affordances` decided (its design's decision 10). v1 refuses a
/// field it does not know, so a 1.0 reader given a v1 declaration carrying a remedy would blame the
/// runbook. Given this marker it reports a format it does not read, which points at the program,
/// the thing that is out of date.
pub const MARKER_V2: &str = "preflight/v2";

/// The declaration formats this program reads, by the marker that opens each.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Format {
    /// The four forms, and nothing else.
    V1,
    /// The four forms, each with an optional `remedy`.
    V2,
}

impl Format {
    /// The format a marker names, or nothing for a marker this program does not read.
    fn of(marker: &str) -> Option<Self> {
        match marker {
            MARKER => Some(Self::V1),
            MARKER_V2 => Some(Self::V2),
            _ => None,
        }
    }
}

/// The stem every declaration marker shares, whatever version follows it.
///
/// A note beginning this stem attempts a declaration even when the version is not one this program
/// reads. Silently treating `preflight/v2 {…}` as prose would report a runbook as ready when the
/// runbook asked for checks this binary cannot perform — so an unknown version is an error, and
/// this stem is what makes the two cases distinguishable.
const MARKER_STEM: &str = "preflight/";

/// The JSON type one field of a declaration must hold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldType {
    /// A string.
    Text,
    /// A list of strings.
    TextList,
}

impl FieldType {
    /// Whether a value is of this type.
    fn accepts(self, value: &serde_json::Value) -> bool {
        match self {
            Self::Text => value.is_string(),
            Self::TextList => value
                .as_array()
                .is_some_and(|items| items.iter().all(|item| item.is_string())),
        }
    }

    /// This type in words, for a report.
    const fn wanted(self) -> &'static str {
        match self {
            Self::Text => "a string",
            Self::TextList => "a list of strings",
        }
    }
}

/// The closed vocabulary of declaration forms: each `kind`, and each field it takes with its type.
///
/// One table rather than a table beside a match arm beside a deserializer, so the set of accepted
/// spellings is stated once and the three questions about a declaration — is this field allowed, is
/// this field present, is this field the right type — are all answered from it. `runbook-portability`
/// requires that arbitrary commands are not accepted, and a form or field absent from this table is
/// exactly that: unknown, and reported.
const FORMS: &[(&str, &[(&str, FieldType)])] = &[
    (
        "tool",
        &[
            ("kind", FieldType::Text),
            ("program", FieldType::Text),
            ("version", FieldType::Text),
        ],
    ),
    (
        "toolchain",
        &[
            ("kind", FieldType::Text),
            ("manifest", FieldType::Text),
            ("shell", FieldType::Text),
            ("tools", FieldType::TextList),
        ],
    ),
    (
        "skill",
        &[
            ("kind", FieldType::Text),
            ("scope", FieldType::Text),
            ("path", FieldType::Text),
        ],
    ),
    (
        "state",
        &[("kind", FieldType::Text), ("check", FieldType::Text)],
    ),
];

/// A requirement's identifier, as the declaration was found under it.
///
/// Retained in every fault, because `runbook-portability` requires that a malformed declaration is
/// "reported against the requirement": a report naming a field and no requirement sends an author
/// searching a document for a string that may appear in several places.
///
/// **Not re-validated against the schema's identifier pattern.** That pattern is the schema's, and
/// `tests/schema_rust_split.rs` exists in this repository because a value the schema owns was
/// restated in Rust and the two drifted. Document validation runs before preflight, so an
/// identifier reaching here already satisfies it; what this type adds is that a value exists.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RequirementId(String);

impl RequirementId {
    /// A requirement's identifier, refusing an empty one.
    ///
    /// Empty rather than any shape check, for the reason on the type.
    pub fn new(text: &str) -> Result<Self, DeclarationFault> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(DeclarationFault::NoIdentifier);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The identifier as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequirementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A bare executable name, resolved on the invocation's search path.
///
/// Rejects separators and option-like spellings, which the design requires and which are two
/// different attacks on one field. A separator would make the name a path, and a path is not
/// resolved on PATH at all — it is opened wherever it points, which is how a carried document
/// would reach an executable outside the environment being checked. An option-like spelling is
/// worse: the name reaches a process argument vector, and `--version` is the fixed argument this
/// program adds, so a name beginning `-` is an argument the probe did not choose.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProgramName(String);

impl ProgramName {
    /// An executable name, refusing a path or an option wearing one.
    pub fn new(text: &str) -> Result<Self, NameFault> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(NameFault::Empty);
        }
        if trimmed.contains('/') || trimmed.contains('\\') {
            return Err(NameFault::Separator);
        }
        if trimmed.starts_with('-') {
            return Err(NameFault::OptionLike);
        }
        if trimmed.split_whitespace().count() > 1 {
            return Err(NameFault::Whitespace);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The name as written.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Shown with each character a terminal would not show as itself named by code point, so a carried
/// runbook cannot write an escape sequence into a report through a tool's name. The probe runs
/// [`ProgramName::as_str`], the name as written.
impl std::fmt::Display for ProgramName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&shown(&self.0))
    }
}

/// Why a string could not be an executable name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameFault {
    /// Nothing was given.
    Empty,
    /// The name carries a path separator, so it is a path.
    Separator,
    /// The name begins with a hyphen, so it reads as an option.
    OptionLike,
    /// The name holds whitespace, so it is two names or an argument vector.
    Whitespace,
}

impl std::fmt::Display for NameFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "an executable name is not empty",
            Self::Separator => {
                "an executable name is bare, with no path separator: it is resolved on the \
                 invocation's search path, and a name carrying `/` would be opened wherever it \
                 points instead"
            }
            Self::OptionLike => {
                "an executable name does not begin with `-`: the probe adds its own fixed \
                 argument, and a name shaped like an option would be one this program did not \
                 choose"
            }
            Self::Whitespace => {
                "an executable name holds no space: it is one name, not a command line"
            }
        })
    }
}

/// A named development shell inside a package manifest.
///
/// Restricted to the characters a Nix identifier is made of, and the restriction is a safety
/// boundary rather than a taste: the name reaches an attribute path in an expression this program
/// assembles. A name carrying a quote or a brace would be a fragment of expression supplied by a
/// carried document, which is precisely the power `runbook-portability` withholds when it says
/// arbitrary commands SHALL NOT be accepted. A shell named outside this charset needs an adapter
/// that quotes properly, not a loosened check.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ShellName(String);

impl ShellName {
    /// A development shell's name.
    pub fn new(text: &str) -> Result<Self, ()> {
        let trimmed = text.trim();
        let acceptable = !trimmed.is_empty()
            && trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if acceptable {
            Ok(Self(trimmed.to_owned()))
        } else {
            Err(())
        }
    }

    /// The name as written.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for ShellName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A relative path that cannot leave the directory it is resolved within.
///
/// Rejects an absolute root and any `..` component. Used for a skill's path, resolved within its
/// scope, and for a manifest's, resolved against the invocation's working directory — both cases
/// where the runbook names a place and the program must not be led somewhere else.
///
/// **Rejected rather than normalized.** Dropping a `..` would resolve the declaration to a path the
/// author did not write, and `artifact-kinds`' habit in this project is that a quietly corrected
/// value is a decision taken on a contributor's behalf.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ScopedRelativePath(PathBuf);

impl ScopedRelativePath {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "part of the path's API; the probes compose with `within`, and a caller that has to name the path as written should not have to reach through the constructor"
        )
    )]
    /// The path as written.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// A relative path staying inside its scope.
    pub fn new(text: &str) -> Result<Self, PathFault> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(PathFault::Empty);
        }
        let path = Path::new(trimmed);
        if path.is_absolute() {
            return Err(PathFault::Absolute);
        }
        if path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(PathFault::Escapes);
        }
        Ok(Self(path.to_owned()))
    }

    /// This path resolved beneath `root`.
    ///
    /// Composed here rather than at a call site, so the containment this type promises is the same
    /// containment the probe relies on.
    pub fn within(&self, root: &Path) -> PathBuf {
        root.join(&self.0)
    }
}

/// Shown with each character a terminal would not show as itself named by code point, as
/// [`ProgramName`] is. The probe resolves [`ScopedRelativePath::within`], the path as written.
impl std::fmt::Display for ScopedRelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&shown(&self.0.display().to_string()))
    }
}

/// Why a string could not be a scoped relative path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathFault {
    /// Nothing was given.
    Empty,
    /// The path has an absolute root, so it names a place outside every scope.
    Absolute,
    /// The path climbs out with `..`.
    Escapes,
}

impl std::fmt::Display for PathFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "a path is not empty",
            Self::Absolute => {
                "a path is relative: it is resolved within the scope the declaration names, and \
                 an absolute root would name a place outside it"
            }
            Self::Escapes => {
                "a path does not climb out with `..`: it is resolved within the scope the \
                 declaration names, and `..` would leave it"
            }
        })
    }
}

/// The installation scope a skill's path is resolved within.
///
/// Two scopes and no fallback between them, which the design states and which is what makes the
/// check mean something: "a global skill is not satisfied by a same-named file bundled beside the
/// runbook". A search that fell back from `home` to `workdir` would report a bundled file as an
/// installed skill, and the whole subject of the check is whether the skill is installed.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SkillScope {
    /// The operator's home directory.
    Home,
    /// The invocation's working directory.
    Workdir,
}

impl SkillScope {
    /// Reads a scope name.
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "home" => Some(Self::Home),
            "workdir" => Some(Self::Workdir),
            _ => None,
        }
    }

    /// The scope's name in a declaration and a report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Workdir => "workdir",
        }
    }
}

/// A state the procedure assumes of the working directory.
///
/// Two supported states. A third needs an adapter rather than an escape hatch, because the
/// alternative — a runbook supplying shell code to decide whether its own preconditions hold —
/// grants a carried document the power to run procedure steps during what the operator asked for
/// as an inspection.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkingState {
    /// The working directory resolves to no project.
    OutsideProject,
    /// The working directory's Git worktree holds no modification.
    CleanGitWorktree,
}

impl WorkingState {
    /// Reads a state name.
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "outside-project" => Some(Self::OutsideProject),
            "clean-git-worktree" => Some(Self::CleanGitWorktree),
            _ => None,
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the declaration's own spelling of this state; `expected` and `remedy` describe it in prose, and a report that echoes what an author wrote reads this"
        )
    )]
    /// The state's name in a declaration and a report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::OutsideProject => "outside-project",
            Self::CleanGitWorktree => "clean-git-worktree",
        }
    }

    /// What satisfying this state looks like, for a report.
    pub const fn expected(self) -> &'static str {
        match self {
            Self::OutsideProject => "the working directory resolves to no HTMLSpec project",
            Self::CleanGitWorktree => {
                "the working directory's Git worktree holds no modified, staged, conflicted, or \
                 untracked file"
            }
        }
    }
}

/// The most characters a remedy may hold.
pub const REMEDY_LIMIT: usize = 200;

/// What a runbook's author says to do when one of its checks does not pass.
///
/// **Text, and nothing else.** `runbook-portability` restricts declarations to built-in checks and
/// refuses arbitrary commands, and a remedy keeps that line: a report shows it, labelled as the
/// runbook's words, and nothing here runs it, opens an address in it, or treats it as satisfying
/// anything. Tools and skills still come from the receiving machine, and the remedy only says where.
///
/// One line of 1 to [`REMEDY_LIMIT`] characters, every one of them shown as what it is, which is
/// the design's decision 11. The character rule is the safety half: a carried runbook's remedy
/// reaches an operator's terminal, and what the operator reads must be all the remedy holds. So
/// [`crate::terminal::hidden`] refuses each character a terminal acts on, reorders by, or draws as
/// nothing, and every whitespace character but the plain space, which keeps the remedy on one line. Spaces around
/// it are trimmed, and the limit counts what remains.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Remedy(String);

impl Remedy {
    /// A remedy, trimmed of the spaces around it, refusing one that holds a character a terminal
    /// would not show as itself, one with no visible character, and a long one.
    pub fn new(text: &str) -> Result<Self, RemedyFault> {
        if let Some(found) = first_hidden(text) {
            return Err(RemedyFault::Unprintable { found });
        }
        // Only plain spaces and the two joiners can remain unseen, and a remedy of nothing else says
        // nothing.
        let trimmed = text.trim_matches(' ');
        if trimmed
            .chars()
            .all(|character| matches!(character, ' ' | '\u{200C}' | '\u{200D}'))
        {
            return Err(RemedyFault::Empty);
        }
        let characters = trimmed.chars().count();
        if characters > REMEDY_LIMIT {
            return Err(RemedyFault::TooLong { characters });
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl std::fmt::Display for Remedy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a string could not be a remedy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemedyFault {
    /// No visible character was given.
    Empty,
    /// More than [`REMEDY_LIMIT`] characters were given.
    TooLong {
        /// How many there were.
        characters: usize,
    },
    /// A character a terminal would not show as itself was given.
    Unprintable {
        /// The first one found.
        found: char,
    },
}

impl std::fmt::Display for RemedyFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str(
                "a remedy says something, and this one is empty. Leave `remedy` out where there \
                 is nothing to add to the program's own",
            ),
            Self::TooLong { characters } => write!(
                f,
                "a remedy is at most {REMEDY_LIMIT} characters, and this one is {characters}. It \
                 points somewhere: say the rest in the requirement's prose"
            ),
            Self::Unprintable { found } => write!(
                f,
                "a remedy is one line of printable text, and this one holds U+{:04X}. A remedy \
                 reaches the operator's terminal, so a character the terminal would not show as \
                 itself — a control character or escape, whitespace other than a plain space, or \
                 an invisible formatting character — is refused",
                u32::from(*found)
            ),
        }
    }
}

/// One machine check a runbook declares.
///
/// A closed sum type with one variant per supported need, and no variant carrying a command.
/// `runbook-portability`: "The declaration format SHALL be documented and restricted to built-in
/// checks; arbitrary commands SHALL NOT be accepted."
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MachineCheck {
    /// A bare executable at an exact version on the invocation's search path.
    Tool {
        /// The executable's name.
        program: ProgramName,
        /// The version it must report.
        version: ToolVersion,
    },
    /// A toolchain supplied by a pinned package-manager environment.
    ///
    /// Names the manifest rather than restating its pins, which is the counter-argument the
    /// proposal raises and the design settles: a requirement that says "the toolchain `flake.nix`
    /// pins" is checkable and does not fork the pin. No version is copied out of the manifest into
    /// a second dependency catalog.
    Toolchain {
        /// The manifest, relative to the invocation's working directory.
        manifest: ScopedRelativePath,
        /// The development shell inside it.
        shell: ShellName,
        /// The executables that environment must supply. Never empty.
        tools: Vec<ProgramName>,
    },
    /// An installed agent skill, as readable content in a named scope.
    ///
    /// Establishes that the content is installed and readable, not that it is good: quality and
    /// behavioural correctness are not observable from a file, and claiming otherwise would be a
    /// report an operator learns to distrust.
    Skill {
        /// The scope the path resolves within.
        scope: SkillScope,
        /// The skill's `SKILL.md`, relative to that scope.
        path: ScopedRelativePath,
    },
    /// A state the procedure assumes.
    State {
        /// The state it assumes.
        check: WorkingState,
    },
}

impl MachineCheck {
    /// What this check expects of the machine, for a report.
    ///
    /// States the condition rather than restating the declaration's JSON, because the report is
    /// read by the operator who has to act on it.
    pub fn expected(&self) -> String {
        match self {
            Self::Tool { program, version } => {
                format!("`{program}` at exactly {version}, resolved on the invocation's PATH")
            }
            Self::Toolchain {
                manifest,
                shell,
                tools,
            } => {
                let names = tools
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "`{manifest}`'s `{shell}` development shell supplies {names}, and each one \
                     resolves to that environment rather than to another installation"
                )
            }
            Self::Skill { scope, path } => format!(
                "a readable, nonempty `SKILL.md` at {path} within the {scope} scope",
                scope = scope.name()
            ),
            Self::State { check } => check.expected().to_owned(),
        }
    }

    /// What to do about this check being unmet, for a report.
    ///
    /// Reports rather than remedies, and the distinction is the proposal's: "Preflight reports; it
    /// does not remediate." Choosing how to install something on an unknown machine is judgment,
    /// and `artifact-kinds` reserves judgment for an agent.
    pub fn remedy(&self) -> String {
        match self {
            Self::Tool { program, .. } => format!(
                "install `{program}` at the declared version, or change the declaration to the \
                 version this procedure actually needs"
            ),
            Self::Toolchain {
                manifest, shell, ..
            } => format!(
                "realize `{manifest}`'s `{shell}` environment on this machine — the inspection \
                 builds and downloads nothing — and make sure the tools resolve to it rather than \
                 to another installation shadowing it on PATH"
            ),
            Self::Skill { scope, path } => format!(
                "install the skill at {path} in the {scope} scope. A copy bundled beside the \
                 runbook does not satisfy it, because the check is whether the skill is installed",
                scope = scope.name()
            ),
            Self::State { check } => match check {
                WorkingState::OutsideProject => String::from(
                    "run this from a directory that is not inside a project, or drop the \
                     declaration if the procedure no longer assumes one",
                ),
                WorkingState::CleanGitWorktree => String::from(
                    "commit, stash, or remove what the worktree holds. An absent repository is \
                     reported as unavailable rather than clean, so establish one if the procedure \
                     expects it",
                ),
            },
        }
    }
}

/// One check and the requirement that declares it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredCheck {
    /// The declaring requirement, whose identifier every report and fault carries.
    pub requirement: RequirementId,
    /// What it declares.
    pub check: MachineCheck,
    /// What the runbook says to do when the check does not pass, where it says anything.
    ///
    /// Only a `preflight/v2` declaration can carry one.
    pub remedy: Option<Remedy>,
}

/// Every machine check one runbook declares, in document order.
///
/// Document order rather than an arbitrary one, because `toolchain-packaging` requires results to
/// follow it: a report an author reads beside the document they wrote is a report they can act on
/// without searching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineContract {
    checks: Vec<DeclaredCheck>,
}

impl MachineContract {
    /// A contract from the checks a document declared.
    ///
    /// **Refuses two checks under one requirement identity.** A requirement carries at most one
    /// declaration, and while [`parse_contract`] enforces that on the document, this constructor
    /// enforces it on the value — so a contract holding a duplicate cannot be built by any path,
    /// including a test's.
    pub fn of(checks: Vec<DeclaredCheck>) -> Result<Self, ContractErrors> {
        let mut faults = Vec::new();
        for (index, declared) in checks.iter().enumerate() {
            if checks[..index]
                .iter()
                .any(|earlier| earlier.requirement == declared.requirement)
            {
                faults.push(ContractFault {
                    requirement: Some(declared.requirement.clone()),
                    fault: DeclarationFault::DuplicateDeclaration,
                });
            }
        }
        if faults.is_empty() {
            Ok(Self { checks })
        } else {
            Err(ContractErrors(faults))
        }
    }

    /// The declared checks, in document order.
    pub fn checks(&self) -> &[DeclaredCheck] {
        &self.checks
    }

    /// Whether the runbook declares nothing.
    ///
    /// Legal, and reported as zero rather than as an unstated pass: `runbook-portability` requires
    /// that "a contract with no machine declarations SHALL report zero checks, without claiming to
    /// have established unstated prerequisites".
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the question a report asks before saying how many checks passed; `len` answers it too, and a caller asking whether there is anything to check should not compare against zero"
        )
    )]
    pub fn is_empty(&self) -> bool {
        self.checks.is_empty()
    }

    /// How many checks the runbook declares.
    pub fn len(&self) -> usize {
        self.checks.len()
    }

    /// The remedy the runbook declares beside the check under `requirement`, if it declares one.
    pub fn remedy_of(&self, requirement: &RequirementId) -> Option<&Remedy> {
        self.checks
            .iter()
            .find(|declared| &declared.requirement == requirement)
            .and_then(|declared| declared.remedy.as_ref())
    }
}

/// Why one declaration could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarationFault {
    /// The note's text after the marker is not one JSON object.
    NotOneObject {
        /// What was found instead, as a short description.
        found: String,
    },
    /// The note begins the marker's stem but names a format version this program does not read.
    UnsupportedFormat {
        /// The marker as written.
        marker: String,
    },
    /// The declaration carries no `kind`, so there is no form to read it as.
    NoKind,
    /// The declaration names a form this program does not provide.
    UnknownKind {
        /// The kind as written.
        kind: String,
    },
    /// The declaration omits a field its form requires.
    MissingField {
        /// The field's name.
        field: &'static str,
    },
    /// The declaration carries a field its form does not have.
    UnknownField {
        /// The field's name.
        field: String,
    },
    /// A field holds a value of the wrong JSON type.
    WrongType {
        /// The field's name.
        field: String,
        /// What was found instead.
        found: String,
        /// What the form takes.
        wanted: &'static str,
    },
    /// A program name is not one.
    InvalidProgram {
        /// The value as written.
        given: String,
        /// Why it is not a name.
        fault: NameFault,
    },
    /// A version is not three numeric components.
    InvalidVersion {
        /// The value as written.
        given: String,
        /// The shared parser's reason.
        problem: String,
    },
    /// A path is not a scoped relative one.
    InvalidPath {
        /// The value as written.
        given: String,
        /// Why it is not one.
        fault: PathFault,
    },
    /// A development shell's name is not one this program can put in an expression safely.
    InvalidShell {
        /// The value as written.
        given: String,
    },
    /// A scope is not one of the two supported.
    InvalidScope {
        /// The value as written.
        given: String,
    },
    /// A state is not one of the two supported.
    InvalidState {
        /// The value as written.
        given: String,
    },
    /// A remedy is not one line of printable text within its limit.
    ///
    /// Carries the fault and not the text, because the text may be long, and may hold the very
    /// character the fault is about.
    InvalidRemedy {
        /// Why it is not one.
        fault: RemedyFault,
    },
    /// A toolchain declares no tool, so there is nothing to check.
    NoTools,
    /// One requirement carries more than one declaration.
    DuplicateDeclaration,
    /// A declaration sits on a requirement with no identifier.
    ///
    /// Refused rather than checked anonymously, and the reason is downstream: a passed check is
    /// copied into the record as a verification naming its requirement, and a requirement nothing
    /// can name is one a record cannot attest to.
    NoIdentifier,
}

impl std::fmt::Display for DeclarationFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOneObject { found } => write!(
                f,
                "the declaration is followed by {found}, and a machine-check declaration is one \
                 JSON object. Write it as `{MARKER} {{\"kind\":…}}`"
            ),
            Self::UnsupportedFormat { marker } => write!(
                f,
                "{marker:?} is not a format this program reads; it reads {MARKER:?} and \
                 {MARKER_V2:?}. A note \
                 beginning {MARKER_STEM:?} is a declaration rather than prose, so this is reported \
                 instead of being skipped — skipping it would report the runbook ready when it \
                 asked for checks nothing performed"
            ),
            Self::NoKind => write!(
                f,
                "the declaration names no `kind`. The forms are {}, and each one needs its `kind` \
                 stated",
                forms_list()
            ),
            Self::UnknownKind { kind } => write!(
                f,
                "{kind:?} is not a check this program provides; the forms are {}. A runbook \
                 cannot supply its own command, so an unsupported condition stays a manual \
                 obligation stated in prose",
                forms_list()
            ),
            Self::MissingField { field } => write!(f, "the declaration has no `{field}` field"),
            // The key is the declaration's, so it is shown with every character a terminal would not
            // show as itself named by code point: echoed as written, a key holding an escape
            // sequence would draw on the operator's terminal. The other faults name declaration text
            // through `{:?}`, which escapes those characters already.
            Self::UnknownField { field } => write!(
                f,
                "`{}` is not a field this form has, and unknown fields are refused rather \
                 than ignored: a field this program does not read is a condition it does not \
                 check, and a report that stayed quiet about it would claim a check that never \
                 ran",
                shown(field)
            ),
            Self::WrongType {
                field,
                found,
                wanted,
            } => write!(f, "`{field}` holds {found} and its form takes {wanted}"),
            Self::InvalidProgram { given, fault } => {
                write!(f, "{given:?} is not an executable name: {fault}")
            }
            Self::InvalidVersion { given, problem } => write!(
                f,
                "{given:?} is not a tool version: {problem}. A declared tool version is exactly \
                 three numeric components, compared numerically against what the tool reports"
            ),
            Self::InvalidPath { given, fault } => {
                write!(f, "{given:?} is not a usable path: {fault}")
            }
            Self::InvalidShell { given } => write!(
                f,
                "{given:?} is not a development-shell name this program can use: a name is made of \
                 ASCII letters, digits, `-`, and `_`, because it reaches an attribute path in an \
                 expression this program assembles"
            ),
            Self::InvalidScope { given } => write!(
                f,
                "{given:?} is not an installation scope; the scopes are `home` and `workdir`, and \
                 there is no fallback between them"
            ),
            Self::InvalidState { given } => write!(
                f,
                "{given:?} is not a state this program can check; the states are \
                 `outside-project` and `clean-git-worktree`. An unsupported state needs an \
                 adapter rather than an escape hatch, so until one exists it stays a manual \
                 obligation stated in prose"
            ),
            Self::InvalidRemedy { fault } => write!(f, "`remedy` is refused: {fault}"),
            Self::NoTools => write!(
                f,
                "the toolchain declaration lists no tool, so there is nothing for it to check. \
                 Name the executables that environment must supply"
            ),
            Self::DuplicateDeclaration => write!(
                f,
                "this requirement carries more than one declaration, and one requirement declares \
                 at most one check. Split them into separate requirements, each with its own \
                 identifier, so a report can name the one that failed"
            ),
            Self::NoIdentifier => write!(
                f,
                "this requirement has no identifier, so nothing can name it in a report or attest \
                 to it in a run's record. Give the requirement an `id`"
            ),
        }
    }
}

/// The form's names, for a message that has to say what they are.
fn forms_list() -> String {
    FORMS
        .iter()
        .map(|(kind, _)| format!("`{kind}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// One fault and the requirement it was found under.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractFault {
    /// The requirement carrying the declaration.
    ///
    /// `None` only where the requirement has no identifier to carry, which is itself a fault — so
    /// a fault never loses its subject silently.
    pub requirement: Option<RequirementId>,
    /// What is wrong with the declaration.
    pub fault: DeclarationFault,
}

impl std::fmt::Display for ContractFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.requirement {
            Some(requirement) => write!(f, "{requirement}: {}", self.fault),
            None => write!(f, "a requirement with no identifier: {}", self.fault),
        }
    }
}

/// Every fault a runbook's declarations carry.
///
/// All of them rather than the first, which is this project's habit and earns its keep here: an
/// author fixing one declaration and re-running to find the next is doing the program's work.
/// Nonempty wherever it is returned as an error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractErrors(Vec<ContractFault>);

impl ContractErrors {
    /// The faults found.
    pub fn faults(&self) -> &[ContractFault] {
        &self.0
    }
}

impl std::fmt::Display for ContractErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, fault) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str("\n")?;
            }
            write!(f, "{fault}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ContractErrors {}

/// The wire shape of one declaration's fields, as serde reads them.
///
/// One struct per form rather than one internally tagged enum, and the reason is the report: an
/// internally tagged enum buffers its content and reports a wrong-typed field with no name attached.
/// The tag is checked in [`check_fields`], which names the field itself, so the enum here stays
/// closed by construction and the message stays actionable.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireTool {
    program: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireToolchain {
    manifest: String,
    shell: String,
    tools: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSkill {
    scope: String,
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireState {
    check: String,
}

/// The closed tagged enum a declaration deserializes into, one variant per form.
#[derive(Debug)]
enum WireCheck {
    Tool(WireTool),
    Toolchain(WireToolchain),
    Skill(WireSkill),
    State(WireState),
}

/// Reads every machine-check declaration a projected document carries.
///
/// Reads the requirement notes and nothing else. `runbook-portability`: preflight "SHALL NOT infer
/// checks from prose, requirement names, or executable examples" — so a requirement stating in
/// prose that the procedure needs `cargo` declares no check, stays a logging obligation, and is
/// reported as unchecked rather than guessed at.
///
/// # Withdrawn requirements
///
/// A requirement this document withdraws is skipped, and the design does not say so. It follows
/// the line [`crate::runs`] already draws twice — the record does not copy a withdrawn requirement
/// and `--check` does not demand an outcome for one — through the same schema-vocabulary helper.
/// A check declared by a withdrawn requirement would be copied into no verification, so it could
/// never be attested to; refusing a run over an obligation that no longer applies would be the
/// defect that helper exists to prevent.
pub fn parse_contract(
    model: &htmlspec_parse::DocumentModel,
) -> Result<MachineContract, ContractErrors> {
    let mut checks = Vec::new();
    let mut faults = Vec::new();

    for requirement in &model.requirements {
        if crate::runs::is_withdrawn(&requirement.change) {
            continue;
        }
        // Any note beginning the stem attempts a declaration, so the version question is asked of
        // every one of them and an unsupported version cannot hide behind a second note.
        let declared: Vec<&String> = requirement
            .notes
            .iter()
            .filter(|note| note.trim_start().starts_with(MARKER_STEM))
            .collect();
        if declared.is_empty() {
            continue;
        }

        let id = match requirement.id.as_deref().map(RequirementId::new) {
            Some(Ok(id)) => Some(id),
            // Reported once for the requirement rather than once per declaration: the fault is
            // that nothing can name it, and saying that twice does not say it better.
            Some(Err(_)) | None => {
                faults.push(ContractFault {
                    requirement: None,
                    fault: DeclarationFault::NoIdentifier,
                });
                None
            }
        };
        if declared.len() > 1 {
            faults.push(ContractFault {
                requirement: id.clone(),
                fault: DeclarationFault::DuplicateDeclaration,
            });
        }
        let (Some(id), Some(note)) = (id, declared.first()) else {
            continue;
        };
        match declaration_in(note) {
            Ok((check, remedy)) => checks.push(DeclaredCheck {
                requirement: id,
                check,
                remedy,
            }),
            Err(found) => faults.extend(found.faults().iter().map(|fault| ContractFault {
                requirement: Some(id.clone()),
                fault: fault.fault.clone(),
            })),
        }
    }

    if !faults.is_empty() {
        return Err(ContractErrors(faults));
    }
    // Through the same validated constructor every other caller uses, which re-checks requirement
    // uniqueness. The loop above reports a duplicated declaration on one requirement; this catches
    // two requirements sharing an identifier, which document validation also reports and which a
    // contract must not silently accept.
    MachineContract::of(checks)
}

/// The one declaration in a note's projected text.
///
/// Takes the projected text rather than the markup, which is the design's choice and the reason a
/// declaration survives canonical formatting: the formatter never rewraps text, and projection
/// collapses whitespace runs, so the JSON reaches here as the author wrote it. The corollary is
/// that a declaration must not depend on whitespace *inside* a string value, since projection
/// would collapse it — stated in the authoring documentation rather than checked here, because
/// nothing at this point can tell the two apart.
///
/// # The remedy
///
/// `preflight/v2` is v1 with one optional field, so its remedy is taken out of the object before the
/// form's fields are checked, and the rest is read exactly as v1 reads it. Under v1 the field is
/// left where it is, and the form's check refuses it as a field the form does not have: nothing
/// about a v1 declaration changes. A bad remedy and a bad check field are reported together, as
/// every other pair of faults in one declaration is.
fn declaration_in(note: &str) -> Result<(MachineCheck, Option<Remedy>), ContractErrors> {
    let text = note.trim();
    let marker = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_owned();
    let Some(format) = Format::of(&marker) else {
        return Err(one(DeclarationFault::UnsupportedFormat { marker }));
    };
    let rest = text[marker.len()..].trim();
    let value: serde_json::Value = serde_json::from_str(rest).map_err(|error| {
        one(DeclarationFault::NotOneObject {
            found: if rest.is_empty() {
                String::from("nothing")
            } else {
                format!("text that is not JSON ({error})")
            },
        })
    })?;
    let Some(object) = value.as_object() else {
        return Err(one(DeclarationFault::NotOneObject {
            found: describe_value(&value),
        }));
    };

    let mut fields = object.clone();
    let mut remedy_faults = Vec::new();
    let remedy = match format {
        Format::V1 => None,
        Format::V2 => match fields.remove("remedy") {
            None => None,
            Some(serde_json::Value::String(given)) => Remedy::new(&given)
                .map_err(|fault| remedy_faults.push(DeclarationFault::InvalidRemedy { fault }))
                .ok(),
            Some(other) => {
                remedy_faults.push(DeclarationFault::WrongType {
                    field: String::from("remedy"),
                    found: describe_value(&other),
                    wanted: FieldType::Text.wanted(),
                });
                None
            }
        },
    };

    let remedy_faults = remedy_faults.into_iter().map(|fault| ContractFault {
        requirement: None,
        fault,
    });
    match form_in(&fields) {
        Ok(check) => {
            let faults: Vec<ContractFault> = remedy_faults.collect();
            if faults.is_empty() {
                Ok((check, remedy))
            } else {
                Err(ContractErrors(faults))
            }
        }
        Err(mut faults) => {
            faults.0.extend(remedy_faults);
            Err(faults)
        }
    }
}

/// The one check a declaration's fields describe, its `kind` and the form's own fields.
fn form_in(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<MachineCheck, ContractErrors> {
    check_fields(object)?;
    // The field set and every type in it are already known to be the form's, so the deserializer's
    // only job here is to hand the values to the typed structs. `kind` is the tag this function
    // selected the form by and not a field of it, so it is dropped before deserializing: a struct
    // that refused unknown fields would otherwise refuse its own tag.
    let mut fields = object.clone();
    fields.remove("kind");
    let without_tag = serde_json::Value::Object(fields);
    let kind = object
        .get("kind")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    // One arm per form, so a value reaching a struct is the value that struct describes. The error
    // is unreachable after `check_fields` and is reported rather than unwrapped, because a panic in
    // a command that has written nothing is a worse answer than a fault.
    let wire: WireCheck = match kind {
        "tool" => serde_json::from_value::<WireTool>(without_tag).map(WireCheck::Tool),
        "toolchain" => {
            serde_json::from_value::<WireToolchain>(without_tag).map(WireCheck::Toolchain)
        }
        "skill" => serde_json::from_value::<WireSkill>(without_tag).map(WireCheck::Skill),
        _ => serde_json::from_value::<WireState>(without_tag).map(WireCheck::State),
    }
    .map_err(|error| {
        one(DeclarationFault::WrongType {
            field: String::from("declaration"),
            found: error.to_string(),
            wanted: "the fields its form takes",
        })
    })?;
    validate(wire)
}

/// What a JSON value is, in words, for a report.
///
/// Crate-visible because the host's adapter has to say what an evaluator answered when the answer
/// is not the shape it parses.
pub(super) fn describe_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::from("a JSON null"),
        serde_json::Value::Bool(_) => String::from("a JSON boolean"),
        serde_json::Value::Number(_) => String::from("a JSON number"),
        serde_json::Value::String(_) => String::from("a JSON string"),
        serde_json::Value::Array(items) => format!("a JSON array of {} item(s)", items.len()),
        serde_json::Value::Object(_) => String::from("a JSON object"),
    }
}

/// Rejects a field the form does not have and reports one the form requires but lacks.
///
/// Both directions, and the unknown-field half is the one that matters. An ignored field is a
/// condition this program does not check, and a readiness report that stayed quiet about it would
/// claim a check that never ran — the failure the whole module exists to prevent.
/// Rejects a field the form does not have, reports one the form requires but lacks, and refuses a
/// value of the wrong type — naming the field in all three cases.
///
/// The unknown-field half is the load-bearing one. An ignored field is a condition this program does
/// not check, and a readiness report that stayed quiet about it would claim a check that never ran.
///
/// Types are checked here rather than left to the deserializer because the deserializer's message
/// names a JSON offset and not a field, and an author told "invalid type at column 60" has to count
/// characters to find out what is wrong.
fn check_fields(object: &serde_json::Map<String, serde_json::Value>) -> Result<(), ContractErrors> {
    let mut faults: Vec<DeclarationFault> = Vec::new();
    let kind = match object.get("kind") {
        None => {
            faults.push(DeclarationFault::NoKind);
            None
        }
        Some(value) => match value.as_str() {
            Some(kind) => Some(kind.to_owned()),
            None => {
                faults.push(DeclarationFault::WrongType {
                    field: String::from("kind"),
                    found: describe_value(value),
                    wanted: FieldType::Text.wanted(),
                });
                None
            }
        },
    };
    let allowed = kind
        .as_deref()
        .and_then(|kind| FORMS.iter().find(|(name, _)| *name == kind));
    if let Some(kind) = &kind {
        if allowed.is_none() {
            faults.push(DeclarationFault::UnknownKind { kind: kind.clone() });
        }
    }
    // With no recognized form there is no field set to check against, and the faults above already
    // say why. Guessing at a form's fields would report a second problem about the first one.
    let Some((_, fields)) = allowed else {
        return refuse(faults);
    };

    for (field, value) in object {
        let Some((_, wanted)) = fields.iter().find(|(name, _)| name == field) else {
            faults.push(DeclarationFault::UnknownField {
                field: field.clone(),
            });
            continue;
        };
        if !wanted.accepts(value) {
            faults.push(DeclarationFault::WrongType {
                field: field.clone(),
                found: describe_value(value),
                wanted: wanted.wanted(),
            });
        }
    }
    for (field, _) in *fields {
        if *field != "kind" && !object.contains_key(*field) {
            faults.push(DeclarationFault::MissingField { field });
        }
    }
    refuse(faults)
}

/// Faults as a refusal, or nothing where there are none.
fn refuse(faults: Vec<DeclarationFault>) -> Result<(), ContractErrors> {
    if faults.is_empty() {
        Ok(())
    } else {
        Err(ContractErrors(
            faults
                .into_iter()
                .map(|fault| ContractFault {
                    requirement: None,
                    fault,
                })
                .collect(),
        ))
    }
}

/// One fault, as the `ContractErrors` this module returns everywhere.
fn one(fault: DeclarationFault) -> ContractErrors {
    ContractErrors(vec![ContractFault {
        requirement: None,
        fault,
    }])
}

/// The wire form through the validated constructors.
///
/// Every field passes a constructor that can refuse it, so nothing downstream re-checks: the
/// program name reaching a process argument vector is a [`ProgramName`], the path reaching a
/// filesystem join is a [`ScopedRelativePath`], and the version reaching a numeric comparison is a
/// [`ToolVersion`]. That is `AGENTS.md` §4's "parse at boundaries and preserve the evidence".
fn validate(wire: WireCheck) -> Result<MachineCheck, ContractErrors> {
    let mut faults: Vec<DeclarationFault> = Vec::new();
    let mut check = None;

    match wire {
        WireCheck::Tool(WireTool { program, version }) => {
            let parsed_program = program_value(&program, &mut faults);
            let parsed_version = ToolVersion::parse(&version).map_err(|error| {
                faults.push(DeclarationFault::InvalidVersion {
                    given: version.clone(),
                    problem: error.to_string(),
                });
            });
            if let (Some(program), Ok(version)) = (parsed_program, parsed_version) {
                check = Some(MachineCheck::Tool { program, version });
            }
        }
        WireCheck::Toolchain(WireToolchain {
            manifest,
            shell,
            tools,
        }) => {
            let parsed_manifest = path_value(&manifest, &mut faults);
            let parsed_shell = ShellName::new(&shell).map_err(|()| {
                faults.push(DeclarationFault::InvalidShell {
                    given: shell.clone(),
                });
            });
            if tools.is_empty() {
                faults.push(DeclarationFault::NoTools);
            }
            let names: Option<Vec<ProgramName>> = tools
                .iter()
                .map(|tool| program_value(tool, &mut faults))
                .collect();
            if let (Some(manifest), Ok(shell), Some(names)) = (parsed_manifest, parsed_shell, names)
            {
                check = Some(MachineCheck::Toolchain {
                    manifest,
                    shell,
                    tools: names,
                });
            }
        }
        WireCheck::Skill(WireSkill { scope, path }) => {
            let parsed_scope = SkillScope::parse(&scope).ok_or_else(|| {
                faults.push(DeclarationFault::InvalidScope {
                    given: scope.clone(),
                });
            });
            let parsed_path = path_value(&path, &mut faults);
            if let (Ok(scope), Some(path)) = (parsed_scope, parsed_path) {
                check = Some(MachineCheck::Skill { scope, path });
            }
        }
        WireCheck::State(WireState { check: state }) => {
            let parsed = WorkingState::parse(&state).ok_or_else(|| {
                faults.push(DeclarationFault::InvalidState {
                    given: state.clone(),
                });
            });
            if let Ok(parsed) = parsed {
                check = Some(MachineCheck::State { check: parsed });
            }
        }
    }

    match (check, faults.is_empty()) {
        (Some(check), true) => Ok(check),
        // A value that could not be built reports why, and a value that was built anyway is
        // discarded rather than half-trusted: a check whose program name was refused must not run
        // with the name it was given.
        (_, false) => Err(ContractErrors(
            faults
                .into_iter()
                .map(|fault| ContractFault {
                    requirement: None,
                    fault,
                })
                .collect(),
        )),
        (None, true) => Err(one(DeclarationFault::NoKind)),
    }
}

/// A program name through its constructor, recording the fault rather than returning it.
///
/// Collects instead of short-circuiting, so a declaration with two bad fields reports both.
fn program_value(given: &str, faults: &mut Vec<DeclarationFault>) -> Option<ProgramName> {
    ProgramName::new(given)
        .map_err(|fault| {
            faults.push(DeclarationFault::InvalidProgram {
                given: given.to_owned(),
                fault,
            });
        })
        .ok()
}

/// A scoped relative path through its constructor, recording the fault.
fn path_value(given: &str, faults: &mut Vec<DeclarationFault>) -> Option<ScopedRelativePath> {
    ScopedRelativePath::new(given)
        .map_err(|fault| {
            faults.push(DeclarationFault::InvalidPath {
                given: given.to_owned(),
                fault,
            });
        })
        .ok()
}

/// One thing a probe measured about this machine.
///
/// **A fact, not a verdict.** Every variant here is something observed; whether it satisfies the
/// declaration is [`assess`]'s answer. The distinction is what lets a report name both numbers in a
/// version mismatch, and it is why "no executable of this name resolves" is a fact rather than an
/// error: absence is measured, and the machine is exactly as it is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fact {
    /// The executable resolved here and reported this version.
    ToolVersion {
        /// The declared name.
        program: ProgramName,
        /// Where it resolved to.
        resolved: PathBuf,
        /// The version it reported.
        version: ToolVersion,
    },
    /// No executable of this name resolves on the invocation's search path.
    ToolAbsent {
        /// The declared name.
        program: ProgramName,
    },
    /// The executable resolved but did not report one three-component version.
    ///
    /// Distinct from absence and from a mismatch: the tool is here and this program cannot read it,
    /// which calls for a different remedy than either.
    ToolVersionUnreadable {
        /// The declared name.
        program: ProgramName,
        /// Where it resolved to.
        resolved: PathBuf,
        /// Why its output could not be read as a version.
        problem: String,
    },
    /// Where each requested tool resolved and whether that place is inside the pinned environment.
    ToolchainTools {
        /// The manifest inspected.
        manifest: ScopedRelativePath,
        /// The development shell inspected.
        shell: ShellName,
        /// One entry per requested tool, in declaration order.
        tools: Vec<ToolProvenance>,
    },
    /// A readable, nonempty `SKILL.md` inside the declared scope.
    SkillInstalled {
        /// The scope it was found in.
        scope: SkillScope,
        /// The path it was found at.
        path: ScopedRelativePath,
        /// How many bytes it holds, as evidence that it is not empty.
        bytes: u64,
    },
    /// No readable nonempty skill file inside the declared scope.
    SkillAbsent {
        /// The scope looked in.
        scope: SkillScope,
        /// The path looked at.
        path: ScopedRelativePath,
        /// What was found instead.
        problem: String,
    },
    /// The working directory resolves to no project.
    NoNearestProject,
    /// The working directory resolves to a project here.
    NearestProject {
        /// Where the project was found.
        at: PathBuf,
    },
    /// The worktree holds nothing.
    WorktreeClean,
    /// The worktree holds these entries.
    WorktreeDirty {
        /// The entries, as the read-only status operation reported them, bounded.
        entries: Vec<String>,
    },
}

/// Where one requested tool resolved, and whether that place is the pinned environment's.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolProvenance {
    /// The requested name.
    pub program: ProgramName,
    /// Where it resolved, or `None` where nothing of that name is on the search path.
    pub resolved: Option<PathBuf>,
    /// Whether the resolved place is inside one of the declared inputs' realized closures.
    pub from_inputs: bool,
    /// The input closure it came from, where it came from one.
    pub input: Option<PathBuf>,
}

impl Fact {
    /// The fact in words, for a report.
    ///
    /// Names what was observed rather than what should be done about it, which is
    /// [`MachineCheck::remedy`]'s job.
    pub fn observed(&self) -> String {
        match self {
            Self::ToolVersion {
                program,
                resolved,
                version,
            } => format!(
                "`{program}` resolved to {} and reported {version}",
                shown(&resolved.display().to_string())
            ),
            Self::ToolAbsent { program } => {
                format!("no `{program}` resolves on the invocation's search path")
            }
            Self::ToolVersionUnreadable {
                program,
                resolved,
                problem,
            } => format!(
                "`{program}` resolved to {} but reported no version this program can read: \
                 {problem}",
                shown(&resolved.display().to_string())
            ),
            Self::ToolchainTools {
                manifest,
                shell,
                tools,
            } => {
                let described = tools
                    .iter()
                    .map(
                        |tool| match (&tool.resolved, tool.from_inputs, &tool.input) {
                            (None, _, _) => format!("`{}` is not on the search path", tool.program),
                            (Some(resolved), true, Some(input)) => format!(
                                "`{}` resolves to {} inside {}",
                                tool.program,
                                shown(&resolved.display().to_string()),
                                shown(&input.display().to_string())
                            ),
                            (Some(resolved), _, _) => format!(
                                "`{}` resolves to {}, which is not inside the environment `{}`'s \
                             `{}` shell declares",
                                tool.program,
                                shown(&resolved.display().to_string()),
                                manifest,
                                shell
                            ),
                        },
                    )
                    .collect::<Vec<_>>()
                    .join("; ");
                described
            }
            Self::SkillInstalled { scope, path, bytes } => format!(
                "a readable `SKILL.md` of {bytes} byte(s) at {path} within the {scope} scope",
                scope = scope.name()
            ),
            Self::SkillAbsent {
                scope,
                path,
                problem,
            } => format!(
                "no skill at {} within the {scope} scope: {problem}",
                path,
                scope = scope.name()
            ),
            Self::NoNearestProject => String::from("the working directory resolves to no project"),
            Self::NearestProject { at } => {
                format!(
                    "the working directory resolves to a project at {}",
                    at.display()
                )
            }
            Self::WorktreeClean => String::from("the worktree holds no modification"),
            Self::WorktreeDirty { entries } => format!(
                "the worktree holds {} entr{}: {}",
                entries.len(),
                if entries.len() == 1 { "y" } else { "ies" },
                entries.join(", ")
            ),
        }
    }
}

/// What one probe produced: a measured fact, or why it measured nothing.
///
/// Four outcomes and the fourth is the one that makes the type honest. `runbook-portability`
/// requires that "failed, unknown, timed-out, and unsupported results SHALL refuse" record
/// creation, and a probe that could only say "measured" or "not measured" would have nowhere to put
/// a tool that hung — which would either be reported as absent (a falsehood) or as ready (worse).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Observation {
    /// The probe measured this.
    Measured(Fact),
    /// Readiness cannot be established from what this machine holds.
    Unavailable {
        /// Why not.
        reason: String,
        /// What would make it available.
        remedy: String,
    },
    /// The probe did not finish inside its bound.
    TimedOut {
        /// The bound it exceeded.
        bound: Duration,
    },
    /// The probe itself failed, so nothing is known.
    ProbeFailure {
        /// Why.
        reason: String,
    },
}

/// Why a probe measured nothing, as the host reports it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbeFault {
    /// Readiness cannot be established without provisioning something.
    Unavailable {
        /// Why not.
        reason: String,
        /// What would make it available.
        remedy: String,
    },
    /// The probe exceeded its bound and was stopped.
    TimedOut {
        /// The bound.
        bound: Duration,
    },
    /// The probe could not be performed at all.
    Failure {
        /// Why.
        reason: String,
    },
}

impl From<ProbeFault> for Observation {
    fn from(fault: ProbeFault) -> Self {
        match fault {
            ProbeFault::Unavailable { reason, remedy } => Self::Unavailable { reason, remedy },
            ProbeFault::TimedOut { bound } => Self::TimedOut { bound },
            ProbeFault::Failure { reason } => Self::ProbeFailure { reason },
        }
    }
}

/// What a probe can do to a machine, and nothing else.
///
/// **The narrowest sufficient capability**, per `AGENTS.md` §6: four fixed operations, each taking
/// the validated values a declaration produced. There is no method here that runs an arbitrary
/// command, reads an arbitrary path, or reaches the network, so a check kind added later has to add
/// an operation rather than widen one — and the operation it adds is the review surface.
///
/// Everything takes `&mut self` because a probe may spend a budget: the design bounds each one at
/// ten seconds and 64 KiB, and a host that tracked what it had spent could not do so behind a
/// shared reference.
pub trait HostProbe {
    /// Whether cancellation has reached this host.
    ///
    /// Consulted between probes and inside a running one, so an interruption stops the work rather
    /// than letting it finish in the background. On the trait rather than read from a global,
    /// because a test host has to be able to say "cancelled" without touching this process's
    /// signals.
    fn cancelled(&self) -> bool;

    /// Measure one executable: where it resolves and what version it reports.
    fn tool(&mut self, program: &ProgramName) -> Result<Fact, ProbeFault>;

    /// Measure where each requested tool resolves and whether that place is the pinned
    /// environment's.
    ///
    /// Performs no build, download, lock update, or shell entry. Where the environment cannot be
    /// inspected from what the machine already holds, this reports [`ProbeFault::Unavailable`]
    /// rather than provisioning: "Neither a manifest's existence nor `IN_NIX_SHELL` proves
    /// readiness", and a check that fetched its way to an answer would have changed the machine it
    /// was asked about.
    fn toolchain(
        &mut self,
        manifest: &ScopedRelativePath,
        shell: &ShellName,
        tools: &[ProgramName],
    ) -> Result<Fact, ProbeFault>;

    /// Measure whether an installed skill's content is readable inside its scope.
    fn skill(&mut self, scope: &SkillScope, path: &ScopedRelativePath) -> Result<Fact, ProbeFault>;

    /// Measure one working-directory state.
    fn state(&mut self, state: &WorkingState) -> Result<Fact, ProbeFault>;
}

/// Observes one declared check against a host.
///
/// The one place a [`MachineCheck`] meets a [`HostProbe`], so the pairing between a check kind and
/// the operation that measures it is stated once. Pure over its inputs apart from the capability it
/// is handed.
pub fn observe(check: &MachineCheck, host: &mut impl HostProbe) -> Observation {
    let measured = match check {
        MachineCheck::Tool { program, .. } => host.tool(program),
        MachineCheck::Toolchain {
            manifest,
            shell,
            tools,
        } => host.toolchain(manifest, shell, tools),
        MachineCheck::Skill { scope, path } => host.skill(scope, path),
        MachineCheck::State { check } => host.state(check),
    };
    match measured {
        Ok(fact) => Observation::Measured(fact),
        Err(fault) => fault.into(),
    }
}

/// One observation, bound to the check it observed.
///
/// **The binding is the invariant.** An observation carrying no identity could be matched to any
/// check, and a report built from that pairing would name one requirement while describing another
/// machine's state. `runbook-portability` requires every result to identify its requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observed {
    /// The requirement whose declaration was observed.
    pub requirement: RequirementId,
    /// The declaration observed.
    pub check: MachineCheck,
    /// What the probe produced.
    pub observation: Observation,
}

/// Why a set of observations could not be matched to a contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationFault {
    /// A declared check has no observation, so nothing is known about it.
    Missing {
        /// The requirement nothing observed.
        requirement: RequirementId,
    },
    /// One declared check has two observations, and the pair cannot both be the fact.
    Duplicate {
        /// The requirement observed twice.
        requirement: RequirementId,
    },
    /// An observation names a requirement the contract does not declare.
    Undeclared {
        /// The requirement named.
        requirement: RequirementId,
    },
    /// An observation carries a check other than the one its requirement declares.
    Mismatched {
        /// The requirement whose observation disagrees with its declaration.
        requirement: RequirementId,
    },
}

impl std::fmt::Display for ObservationFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { requirement } => write!(
                f,
                "{requirement} declares a check that was not observed, so nothing is known about \
                 it and this cannot report readiness"
            ),
            Self::Duplicate { requirement } => write!(
                f,
                "{requirement} was observed twice, and two facts about one condition cannot both \
                 be the one this report names"
            ),
            Self::Undeclared { requirement } => write!(
                f,
                "{requirement} was observed but declares no machine check, so the observation has \
                 nothing to be compared against"
            ),
            Self::Mismatched { requirement } => write!(
                f,
                "{requirement}'s observation is not of the check it declares, so the fact and the \
                 condition belong to different requirements"
            ),
        }
    }
}

/// Every observation of one contract's checks, matched to them by requirement identity.
///
/// Built only through [`Observations::of`], which is what makes [`assess`] total: a value of this
/// type holds exactly one observation per declared check, so the assessment cannot meet a hole.
/// "Missing or duplicate facts cannot produce a ready result" is therefore a property of the type
/// rather than a check the assessment remembers to perform.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observations {
    observed: Vec<Observed>,
}

impl Observations {
    /// Matches observations to a contract's checks, in the contract's document order.
    ///
    /// Reorders rather than trusting the caller's order, because the report has to follow document
    /// order and a probe loop that stopped early would otherwise leave the ordering as evidence of
    /// where it stopped.
    pub fn of(
        contract: &MachineContract,
        observed: Vec<Observed>,
    ) -> Result<Self, Vec<ObservationFault>> {
        let mut faults = Vec::new();
        let mut matched: Vec<Observed> = Vec::with_capacity(contract.checks.len());

        for declared in &contract.checks {
            let ours: Vec<&Observed> = observed
                .iter()
                .filter(|found| found.requirement == declared.requirement)
                .collect();
            match ours.len() {
                0 => faults.push(ObservationFault::Missing {
                    requirement: declared.requirement.clone(),
                }),
                1 => {
                    let found = ours[0];
                    if found.check != declared.check {
                        faults.push(ObservationFault::Mismatched {
                            requirement: declared.requirement.clone(),
                        });
                        continue;
                    }
                    matched.push(found.clone());
                }
                _ => faults.push(ObservationFault::Duplicate {
                    requirement: declared.requirement.clone(),
                }),
            }
        }
        for extra in &observed {
            if !contract
                .checks
                .iter()
                .any(|declared| declared.requirement == extra.requirement)
            {
                faults.push(ObservationFault::Undeclared {
                    requirement: extra.requirement.clone(),
                });
            }
        }

        if faults.is_empty() {
            Ok(Self { observed: matched })
        } else {
            Err(faults)
        }
    }

    /// The observations, in document order.
    pub fn observed(&self) -> &[Observed] {
        &self.observed
    }
}

/// One declared check whose condition this machine satisfies, with the evidence.
///
/// Carries the measured fact rather than a boolean, because the fact is what a run's record copies
/// into its verification: `runbook-execution` requires an outcome *and how it was reached*, and
/// "passed" with no observation is a claim nobody can repeat.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PassedCheck {
    /// The requirement that declared it.
    pub requirement: RequirementId,
    /// What was declared.
    pub check: MachineCheck,
    /// What was measured.
    pub fact: Fact,
}

/// Every check that passed. Empty for a runbook declaring none, which is legal.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PassedChecks {
    passed: Vec<PassedCheck>,
}

impl PassedChecks {
    /// The checks that passed, in document order.
    pub fn checks(&self) -> &[PassedCheck] {
        &self.passed
    }

    /// How many passed.
    pub fn len(&self) -> usize {
        self.passed.len()
    }

    /// Whether nothing passed, which for a runbook declaring nothing is the correct answer.
    pub fn is_empty(&self) -> bool {
        self.passed.is_empty()
    }
}

/// Why one declared check is not satisfied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    /// The requirement that declared it.
    pub requirement: RequirementId,
    /// What was declared.
    pub check: MachineCheck,
    /// What the declaration expected.
    pub expected: String,
    /// What was observed instead.
    pub observed: String,
    /// How the check ended.
    pub outcome: CheckOutcome,
    /// What to do about it.
    pub remedy: String,
}

/// How one check ended, where the ending is not a pass.
///
/// Four variants because `toolchain-packaging` requires an unknown observation to be "reported as
/// unmet readiness rather than as success" while staying distinguishable from a condition that was
/// measured and found false: "An invalid declaration SHALL remain distinguishable from a valid
/// check whose condition is false." An unavailable environment and an absent tool call for
/// different acts, and one `Failed` variant would name both as the same thing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckOutcome {
    /// The condition was measured and is false.
    Unmet,
    /// Readiness could not be established from what this machine holds.
    Unavailable,
    /// The probe did not finish inside its bound.
    TimedOut,
    /// The probe failed, so nothing is known.
    ProbeFailed,
}

impl CheckOutcome {
    /// The outcome's name in a report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unmet => "unmet",
            Self::Unavailable => "unavailable",
            Self::TimedOut => "timed-out",
            Self::ProbeFailed => "probe-failed",
        }
    }
}

impl std::fmt::Display for CheckOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Every check that is not satisfied. Nonempty by construction.
///
/// The nonemptiness is in the type rather than in a caller's memory, because
/// [`PreflightAssessment::NotReady`] holding zero findings would be a refusal that names nothing —
/// a report an operator cannot act on, and indistinguishable in its effect from a bug.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonEmptyFindings {
    findings: Vec<Finding>,
}

impl NonEmptyFindings {
    /// Findings, refusing an empty set.
    ///
    /// `None` rather than an error type: an empty set is not a failure to build findings, it is the
    /// other assessment, and the caller decides which one it has.
    pub fn of(findings: Vec<Finding>) -> Option<Self> {
        if findings.is_empty() {
            None
        } else {
            Some(Self { findings })
        }
    }

    /// The findings, in document order.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
}

/// Whether this machine can start a run of the runbook that declared these checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreflightAssessment {
    /// Every declared check passed.
    Ready(PassedChecks),
    /// At least one did not, and each one is named.
    NotReady(NonEmptyFindings),
}

impl PreflightAssessment {
    /// Whether the machine is ready.
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }
}

/// Judges measured facts against what was declared.
///
/// **Pure and total.** No process, no clock, no filesystem: `AGENTS.md` §6 again, and the practical
/// payoff is that every verdict below is a test over injected facts rather than a test over the
/// developer's machine.
///
/// A fact of the wrong shape for its check — which [`Observations::of`] makes unreachable through
/// the public path — is reported as a probe failure rather than assumed away. Totality here means
/// no `unreachable!`, and the alternative to reporting it would be a panic in a command that has
/// already written nothing.
pub fn assess(contract: &MachineContract, facts: Observations) -> PreflightAssessment {
    let mut passed = Vec::new();
    let mut findings = Vec::new();

    for observed in facts.observed() {
        let declared = contract
            .checks
            .iter()
            .find(|check| check.requirement == observed.requirement);
        let Some(declared) = declared else {
            // `Observations::of` refuses an undeclared requirement, so this is unreachable through
            // the public path. Reported rather than unwrapped, for the reason above.
            findings.push(Finding {
                requirement: observed.requirement.clone(),
                check: observed.check.clone(),
                expected: observed.check.expected(),
                observed: String::from("this requirement declares no machine check"),
                outcome: CheckOutcome::ProbeFailed,
                remedy: observed.check.remedy(),
            });
            continue;
        };
        match judge(&declared.check, &observed.observation) {
            Verdict::Satisfied(fact) => passed.push(PassedCheck {
                requirement: declared.requirement.clone(),
                check: declared.check.clone(),
                fact,
            }),
            Verdict::Refused { observed, outcome } => findings.push(Finding {
                requirement: declared.requirement.clone(),
                check: declared.check.clone(),
                expected: declared.check.expected(),
                observed,
                outcome,
                remedy: declared.check.remedy(),
            }),
        }
    }

    match NonEmptyFindings::of(findings) {
        Some(findings) => PreflightAssessment::NotReady(findings),
        None => PreflightAssessment::Ready(PassedChecks { passed }),
    }
}

/// One observation's verdict against the check it observed.
enum Verdict {
    /// The condition holds, with the fact that shows it.
    Satisfied(Fact),
    /// The condition does not hold, or nothing is known about it.
    Refused {
        /// What was observed, in words.
        observed: String,
        /// How the check ended.
        outcome: CheckOutcome,
    },
}

/// Judges one observation against one check.
///
/// The whole of the evaluation, in one match, so the pairing between a check kind and the fact that
/// satisfies it is readable in one place.
fn judge(check: &MachineCheck, observation: &Observation) -> Verdict {
    let fact = match observation {
        Observation::Measured(fact) => fact,
        Observation::Unavailable { reason, remedy } => {
            return refused(format!("{reason} ({remedy})"), CheckOutcome::Unavailable)
        }
        Observation::TimedOut { bound } => {
            return refused(
                format!(
                    "the probe did not finish within {} second(s), so it was stopped and nothing \
                     is known about this condition",
                    bound.as_secs()
                ),
                CheckOutcome::TimedOut,
            )
        }
        Observation::ProbeFailure { reason } => {
            return refused(reason.clone(), CheckOutcome::ProbeFailed)
        }
    };

    match (check, fact) {
        (MachineCheck::Tool { program, version }, Fact::ToolVersion { version: found, .. }) => {
            if version == found {
                Verdict::Satisfied(fact.clone())
            } else {
                refused(
                    format!("`{program}` reported {found}, and the declaration says {version}"),
                    CheckOutcome::Unmet,
                )
            }
        }
        (MachineCheck::Tool { program, .. }, Fact::ToolAbsent { .. }) => refused(
            format!("no `{program}` resolves on the invocation's search path"),
            CheckOutcome::Unmet,
        ),
        (MachineCheck::Tool { program, .. }, Fact::ToolVersionUnreadable { problem, .. }) => {
            refused(
                format!(
                    "`{program}` is installed but reported no version this program can read: \
                     {problem}. The declaration states an exact version, so an unreadable one is \
                     not a match"
                ),
                CheckOutcome::Unmet,
            )
        }
        (MachineCheck::Toolchain { .. }, Fact::ToolchainTools { tools, .. }) => {
            let differing: Vec<&ToolProvenance> = tools
                .iter()
                .filter(|tool| tool.resolved.is_none() || !tool.from_inputs)
                .collect();
            if differing.is_empty() {
                Verdict::Satisfied(fact.clone())
            } else {
                let named = differing
                    .iter()
                    .map(|tool| match &tool.resolved {
                        None => format!("`{}` is not on the search path", tool.program),
                        Some(resolved) => format!(
                            "`{}` resolves to {}, which is not inside the declared environment",
                            tool.program,
                            shown(&resolved.display().to_string())
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                refused(named, CheckOutcome::Unmet)
            }
        }
        (MachineCheck::Skill { .. }, Fact::SkillInstalled { .. }) => {
            Verdict::Satisfied(fact.clone())
        }
        (MachineCheck::Skill { .. }, Fact::SkillAbsent { problem, .. }) => {
            refused(problem.clone(), CheckOutcome::Unmet)
        }
        (
            MachineCheck::State {
                check: WorkingState::OutsideProject,
            },
            Fact::NoNearestProject,
        ) => Verdict::Satisfied(fact.clone()),
        (
            MachineCheck::State {
                check: WorkingState::OutsideProject,
            },
            Fact::NearestProject { at },
        ) => refused(
            format!(
                "the working directory is inside a project at {}, which is the state this \
                 declaration exists to refuse: the procedure's first step would act on a project \
                 the operator did not choose",
                at.display()
            ),
            CheckOutcome::Unmet,
        ),
        (
            MachineCheck::State {
                check: WorkingState::CleanGitWorktree,
            },
            Fact::WorktreeClean,
        ) => Verdict::Satisfied(fact.clone()),
        (
            MachineCheck::State {
                check: WorkingState::CleanGitWorktree,
            },
            Fact::WorktreeDirty { .. },
        ) => refused(fact.observed(), CheckOutcome::Unmet),
        // A fact that is not the one this check produces. Unreachable through `observe`, which
        // pairs them; reported rather than assumed away, so a check kind added without a matching
        // arm fails a run loudly instead of passing it silently.
        (_, _) => refused(
            format!(
                "the probe measured {} for a check of another kind, so nothing is known about \
                 this condition",
                fact.observed()
            ),
            CheckOutcome::ProbeFailed,
        ),
    }
}

/// A refusal, with the words the report needs.
fn refused(observed: String, outcome: CheckOutcome) -> Verdict {
    Verdict::Refused { observed, outcome }
}

/// The facts a report names, taken before the snapshot that produced them is consumed.
///
/// Separate from [`PreflightReport`] because a report also carries a verdict, and the verdict does
/// not exist until the assessment does. A refusal has to name the runbook, its version, and both
/// locations just as a readiness does, so the naming half is a value of its own.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportContext {
    /// The runbook's name.
    pub runbook: String,
    /// The version it carries.
    pub version: crate::version::RunbookVersion,
    /// Where its documents were read from, which is the store's directory.
    pub source: PathBuf,
    /// Where the machine checks were evaluated, which is the invocation's working directory.
    pub workdir: PathBuf,
    /// What was declared.
    pub contract: MachineContract,
}

impl ReportContext {
    /// This context's report of `assessment`.
    pub fn report(self, assessment: PreflightAssessment) -> PreflightReport {
        PreflightReport {
            runbook: self.runbook,
            version: self.version,
            source: self.source,
            workdir: self.workdir,
            contract: self.contract,
            assessment,
        }
    }
}

/// Writes a report in the form the caller asked for.
///
/// One function rather than a branch at each command, because the routing rule — a structured
/// payload and nothing else on standard output — is the rule [`crate::output::Streams`] exists to
/// hold, and a second place that writes a report is a second place that can break it.
pub fn report_to<O: std::io::Write, E: std::io::Write>(
    streams: &mut crate::output::Streams<O, E>,
    report: &PreflightReport,
) {
    if streams.form().is_structured() {
        if let Ok(json) = serde_json::to_string_pretty(&report.payload()) {
            let _ = streams.payload(&json);
        }
        return;
    }
    let _ = streams.report(&report.render_human());
}

/// One preflight's whole answer, as a report.
///
/// Everything `toolchain-packaging` requires the report to identify, and nothing that could be
/// reused as permission: the assessment is a value, and [`crate::runs::prepare_run`] observes again
/// rather than reading this.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreflightReport {
    /// The runbook's name.
    pub runbook: String,
    /// The version it carries.
    pub version: RunbookVersion,
    /// Where its documents were read from, which is the store's directory.
    pub source: PathBuf,
    /// Where the machine checks were evaluated, which is the invocation's working directory.
    ///
    /// Named beside the source because the two differ whenever `--store` is used, and a report
    /// that named only one would leave an operator guessing which tree a worktree check judged.
    pub workdir: PathBuf,
    /// What was declared.
    pub contract: MachineContract,
    /// The verdict.
    pub assessment: PreflightAssessment,
}

impl PreflightReport {
    /// How this report exits.
    ///
    /// The whole mapping `toolchain-packaging` states, in one place: a completed preflight with
    /// unmet requirements exits 1, a successful one exits 0. An invalid contract or a load failure
    /// never becomes a report at all — the caller reports it as an operational failure at exit 2 —
    /// and cancellation is likewise the caller's, because a cancelled preflight has no verdict to
    /// render.
    pub const fn outcome(&self) -> crate::exit::Outcome {
        match self.assessment {
            PreflightAssessment::Ready(_) => crate::exit::Outcome::Success,
            PreflightAssessment::NotReady(_) => crate::exit::Outcome::PrerequisiteFailure,
        }
    }

    /// The human report, leading with readiness.
    ///
    /// Leads with the verdict rather than with the checks, which `toolchain-packaging` requires and
    /// which is also the project's reporting habit: state the condition first, then the
    /// consequence. An operator who reads one line learns whether they can start.
    pub fn render_human(&self) -> String {
        let mut out = String::new();
        match &self.assessment {
            PreflightAssessment::Ready(passed) => {
                out.push_str(&format!("ready: {} can start here\n", self.runbook));
                out.push_str(&self.identifies());
                if passed.is_empty() {
                    // Zero stated rather than glossed, because "ready" over a runbook that declared
                    // nothing must not read as "every prerequisite was established".
                    out.push_str(
                        "\nthis runbook declares no machine check, so nothing was established \
                         about this machine. Its other requirements are still obligations of a \
                         run, recorded as the run reaches them\n",
                    );
                } else {
                    out.push_str(&format!("\n{} check(s) passed:\n", passed.len()));
                    for check in passed.checks() {
                        out.push_str(&format!(
                            "  ok  {} — {}\n      {}\n",
                            check.requirement,
                            check.check.expected(),
                            check.fact.observed()
                        ));
                    }
                }
            }
            PreflightAssessment::NotReady(findings) => {
                out.push_str(&format!("not ready: {} cannot start here\n", self.runbook));
                out.push_str(&self.identifies());
                out.push_str(&format!(
                    "\n{} of {} check(s) are not satisfied:\n",
                    findings.findings().len(),
                    self.contract.len()
                ));
                for finding in findings.findings() {
                    out.push_str(&format!(
                        "  !!  {} [{}] — {}\n      expected: {}\n      observed: {}\n      \
                         remedy: {}\n",
                        finding.requirement,
                        finding.outcome,
                        finding.check.expected(),
                        finding.expected,
                        finding.observed,
                        finding.remedy
                    ));
                    // Every finding is a check that did not pass, which is where a person needs the
                    // author's pointer. After the program's own remedy, and labelled, so a reader
                    // can tell the runbook's words from the program's.
                    if let Some(remedy) = self.contract.remedy_of(&finding.requirement) {
                        out.push_str(&format!("      the runbook says: {remedy}\n"));
                    }
                }
                out.push_str(
                    "\nno run was started and nothing was written. Preflight reports rather than \
                     installs: what is missing is named above, and how to supply it on this \
                     machine is yours to decide\n",
                );
            }
        }
        out
    }

    /// The lines naming the runbook, its version, and the two locations.
    fn identifies(&self) -> String {
        format!(
            "  runbook: {} {}\n  source:  {}\n  checked: {}\n",
            shown(&self.runbook.to_string()),
            self.version,
            shown(&self.source.display().to_string()),
            shown(&self.workdir.display().to_string())
        )
    }

    /// The structured payload: one object, with no probe output inside it.
    ///
    /// Typed structures serialized rather than arbitrary probe JSON passed through, which the
    /// design requires and which is what keeps a probe's own output — a tool's `--version` text, a
    /// Nix evaluator's diagnostics — out of the payload an agent parses.
    ///
    /// A check whose declaration carries a remedy has `runbookRemedy`, whatever its outcome: the
    /// payload describes what the runbook declares, and a passed check's remedy is still declared.
    pub fn payload(&self) -> serde_json::Value {
        serde_json::json!({
            "runbook": self.runbook,
            "version": self.version.to_string(),
            "source": self.source,
            "workdir": self.workdir,
            "ready": self.assessment.is_ready(),
            "declared": self.contract.len(),
            "checks": match &self.assessment {
                PreflightAssessment::Ready(passed) => passed
                    .checks()
                    .iter()
                    .map(|check| self.with_remedy(&check.requirement, serde_json::json!({
                        "requirement": check.requirement.to_string(),
                        "expected": check.check.expected(),
                        "observed": check.fact.observed(),
                        "outcome": "passed",
                        "remedy": null,
                    })))
                    .collect::<Vec<_>>(),
                PreflightAssessment::NotReady(findings) => {
                    // Every declared check appears, in document order: the passed ones beside the
                    // findings, because a report naming only failures leaves an agent unable to
                    // tell a check that passed from one nothing reached.
                    let mut all: Vec<serde_json::Value> = Vec::new();
                    let findings = findings.findings();
                    for declared in self.contract.checks() {
                        match findings
                            .iter()
                            .find(|finding| finding.requirement == declared.requirement)
                        {
                            Some(finding) => all.push(self.with_remedy(
                                &finding.requirement,
                                serde_json::json!({
                                    "requirement": finding.requirement.to_string(),
                                    "expected": finding.expected,
                                    "observed": finding.observed,
                                    "outcome": finding.outcome.name(),
                                    "remedy": finding.remedy,
                                }),
                            )),
                            None => all.push(self.with_remedy(
                                &declared.requirement,
                                serde_json::json!({
                                    "requirement": declared.requirement.to_string(),
                                    "expected": declared.check.expected(),
                                    "observed": "the condition held",
                                    "outcome": "passed",
                                    "remedy": null,
                                }),
                            )),
                        }
                    }
                    all
                }
            },
        })
    }

    /// A check's payload object, with the remedy its runbook declares where it declares one.
    ///
    /// On every check whose declaration carries a remedy, whatever its outcome: the payload
    /// describes what the runbook declares, and a remedy beside a check that passed is still
    /// declared. The human report is the one that shows it only where a check did not pass.
    fn with_remedy(
        &self,
        requirement: &RequirementId,
        mut check: serde_json::Value,
    ) -> serde_json::Value {
        if let (Some(remedy), Some(object)) =
            (self.contract.remedy_of(requirement), check.as_object_mut())
        {
            object.insert(
                String::from("runbookRemedy"),
                serde_json::Value::String(remedy.to_string()),
            );
        }
        check
    }
}

/// Reports whether `runbook` can start in this working environment.
///
/// The whole of the `htmlspec preflight` command. It writes nothing: not a record, not a directory,
/// not an edit to the runbook or its skills. `runbook-portability` requires that of the operation,
/// and the only capability this function holds that could write is the one it hands to a probe —
/// which runs `--version`, a read-only `git status`, and a read-only manifest evaluation.
///
/// # What each exit code means here
///
/// `toolchain-packaging` states the mapping and this function is the only place it is decided: a
/// completed preflight with unmet requirements exits 1, a successful one exits 0, an invalid
/// invocation or contract or operational failure exits 2, and cancellation exits 3. **An invalid
/// declaration is exit 2 and not exit 1**, which is the distinction the same requirement draws: a
/// malformed declaration is not a check whose condition is false, it is a contract this program
/// cannot read, and reporting it as an unmet prerequisite would tell an author to install something.
pub fn command<O: std::io::Write, E: std::io::Write>(
    runbook: &str,
    store: Option<&str>,
    streams: &mut crate::output::Streams<O, E>,
) -> crate::exit::Outcome {
    use crate::exit::Outcome;

    let project = match crate::store::project_for(store) {
        Ok(project) => project,
        Err(report) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };
    // Through the same constructor every name-taking operation uses, so `preflight ../other` is
    // refused rather than resolved somewhere the project does not keep runbooks.
    let directory = match project.named_runbook(runbook) {
        Ok(directory) => directory,
        Err(error) => {
            let _ = streams.failure(&error.to_string());
            return Outcome::OperationalFailure;
        }
    };
    if !directory.is_dir() {
        let _ = streams.failure(&format!(
            "no runbook named {} in {}. To see what the project holds, run \
             `htmlspec list --runbooks`",
            shown(runbook),
            shown(&project.runbooks().display().to_string())
        ));
        return Outcome::OperationalFailure;
    }

    let source = match crate::runs::load(&directory, runbook) {
        Ok(source) => source,
        Err(fault) => {
            let _ = streams.failure(&fault.to_string());
            return Outcome::OperationalFailure;
        }
    };

    let context = ReportContext {
        runbook: source.name.clone(),
        version: source.version,
        source: source.directory.clone(),
        // The invocation's working directory, and not the store's: `--store` selects documents, and
        // a stored runbook checked from a separate checkout judges the checkout.
        workdir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        contract: source.contract.clone(),
    };

    let mut host = host::Host::interruptible(context.workdir.clone(), streams);
    let observations = match host::observe_all(&source.contract, &mut host) {
        Ok(observations) => observations,
        Err(_interrupted) => {
            let _ = streams.failure(
                "preflight was cancelled. Every probe was stopped and reaped before this was \
                 reported, and nothing was written",
            );
            return Outcome::Cancelled;
        }
    };

    let report = context.report(assess(&source.contract, observations));
    let outcome = report.outcome();
    report_to(streams, &report);
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A specification document holding requirements whose notes are given.
    ///
    /// Built as markup and projected, rather than assembled as a model, so these tests drive the
    /// same path the command drives: a note reaching [`parse_contract`] has been through the
    /// projection's whitespace normalization, and a test that skipped it would not be testing the
    /// wire format as it arrives.
    fn contract_of(requirements: &[(&str, &[&str])]) -> Result<MachineContract, ContractErrors> {
        let mut body = String::new();
        for (index, (id, notes)) in requirements.iter().enumerate() {
            let mut unit = format!(
                "<section class=\"requirement\" id=\"req-{id}\" data-change=\"unchanged\">\
                 <h2>Requirement {index}</h2>\
                 <p>A run MUST satisfy it.</p>\
                 <article class=\"scenario\" id=\"scenario-{id}\" data-change=\"unchanged\">\
                 <h3>Its scenario</h3>\
                 <dl><dt>When</dt><dd>A run starts.</dd><dt>Then</dt><dd>It records it.</dd></dl>\
                 </article>"
            );
            for note in *notes {
                unit.push_str(&format!(
                    "<p data-field=\"note\">{}</p>",
                    htmlspec_assets::escape_text(note)
                ));
            }
            unit.push_str("</section>");
            body.push_str(&unit);
        }
        let source = format!(
            "<!DOCTYPE html><html lang=\"en\"><head>\
             <meta charset=\"utf-8\">\
             <meta content=\"0.1.0\" name=\"htmlspec-version\">\
             <meta content=\"specification\" name=\"htmlspec-kind\">\
             <title>Fixture</title></head>\
             <body><main>{body}</main></body></html>"
        );
        let parsed = htmlspec_parse::parse_document(&source);
        assert!(
            parsed.is_recovery_free(),
            "the fixture itself needs error recovery: {source}"
        );
        let projection = htmlspec_parse::project(&parsed.document);
        parse_contract(&projection.model)
    }

    /// One requirement carrying one note.
    fn one(note: &str) -> Result<MachineContract, ContractErrors> {
        contract_of(&[("one", &[note])])
    }

    /// The declared checks, as `(requirement, kind)` for a compact assertion.
    fn declared(contract: &MachineContract) -> Vec<(String, String)> {
        contract
            .checks
            .iter()
            .map(|check| {
                (
                    check.requirement.to_string(),
                    match &check.check {
                        MachineCheck::Tool { .. } => String::from("tool"),
                        MachineCheck::Toolchain { .. } => String::from("toolchain"),
                        MachineCheck::Skill { .. } => String::from("skill"),
                        MachineCheck::State { .. } => String::from("state"),
                    },
                )
            })
            .collect()
    }

    /// The faults a parse produced, as text, for an assertion message.
    fn reported(error: &ContractErrors) -> String {
        error.to_string()
    }

    /// Each of the four public forms parses, and parses into the variant it names.
    ///
    /// The accepting direction first: a parser that refused everything would pass every negative
    /// test below.
    #[test]
    fn each_supported_form_parses() {
        let contract = contract_of(&[
            (
                "tool",
                &["preflight/v1 {\"kind\":\"tool\",\"program\":\"python3\",\"version\":\"3.13.2\"}"],
            ),
            (
                "chain",
                &[
                    "preflight/v1 {\"kind\":\"toolchain\",\"manifest\":\"flake.nix\",\
                     \"shell\":\"default\",\"tools\":[\"cargo\",\"rustc\"]}",
                ],
            ),
            (
                "skill",
                &[
                    "preflight/v1 {\"kind\":\"skill\",\"scope\":\"home\",\
                     \"path\":\".agents/skills/example/SKILL.md\"}",
                ],
            ),
            (
                "state",
                &["preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\"}"],
            ),
        ])
        .expect("every form parses");

        assert_eq!(
            declared(&contract),
            vec![
                (String::from("req-tool"), String::from("tool")),
                (String::from("req-chain"), String::from("toolchain")),
                (String::from("req-skill"), String::from("skill")),
                (String::from("req-state"), String::from("state")),
            ],
            "the checks are not in document order, or a form parsed as another"
        );

        // And the values inside them are the values written, which is what makes the constructors
        // worth having: a `ProgramName` here has already refused a path.
        match &contract.checks[1].check {
            MachineCheck::Toolchain {
                manifest,
                shell,
                tools,
            } => {
                assert_eq!(manifest.as_path(), Path::new("flake.nix"));
                assert_eq!(shell.as_str(), "default");
                assert_eq!(
                    tools.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    vec!["cargo", "rustc"]
                );
            }
            other => panic!("the toolchain declaration parsed as {other:?}"),
        }
    }

    /// Both supported states parse, and each keeps its own name.
    #[test]
    fn each_supported_state_parses() {
        for (spelling, expected) in [
            ("outside-project", WorkingState::OutsideProject),
            ("clean-git-worktree", WorkingState::CleanGitWorktree),
        ] {
            let contract = one(&format!(
                "preflight/v1 {{\"kind\":\"state\",\"check\":\"{spelling}\"}}"
            ))
            .expect("parses");
            assert_eq!(
                contract.checks[0].check,
                MachineCheck::State { check: expected },
                "{spelling} parsed as something else"
            );
            assert_eq!(expected.name(), spelling);
        }
    }

    /// An ordinary note is prose, and prose is not a check.
    ///
    /// `runbook-portability`: preflight "SHALL NOT infer checks from prose, requirement names, or
    /// executable examples". A requirement whose note says the procedure needs `cargo` declares
    /// nothing, so it stays a logging obligation.
    #[test]
    fn an_ordinary_note_declares_nothing() {
        let contract = contract_of(&[(
            "prose",
            &[
                "The procedure needs cargo, python3, and a clean tree.",
                "Run `cargo test` before continuing.",
            ],
        )])
        .expect("prose is not a fault");
        assert!(
            contract.is_empty(),
            "prose was read as a declaration: {:?}",
            declared(&contract)
        );
        assert_eq!(contract.len(), 0);
    }

    /// A runbook declaring nothing has a contract, and that contract is empty.
    ///
    /// Zero checks is legal and is reported as zero. Without this, an empty contract and a failed
    /// parse would be indistinguishable to a caller.
    #[test]
    fn a_runbook_with_no_declaration_has_an_empty_contract() {
        let contract = contract_of(&[]).expect("no requirement is no fault");
        assert!(contract.is_empty());
        assert!(contract.checks().is_empty());

        let assessment = assess(
            &contract,
            Observations::of(&contract, Vec::new()).expect("nothing to match"),
        );
        assert!(
            assessment.is_ready(),
            "a runbook declaring no check was refused"
        );
        match assessment {
            PreflightAssessment::Ready(ref passed) => assert!(
                passed.is_empty(),
                "a runbook declaring nothing reported a passed check"
            ),
            PreflightAssessment::NotReady(_) => panic!("refused"),
        }
        // And the report says so rather than implying that prerequisites were established.
        let report = PreflightReport {
            runbook: String::from("release"),
            version: RunbookVersion::parse("0.1.0").expect("parses"),
            source: PathBuf::from("htmlspec/runbooks/release"),
            workdir: PathBuf::from("."),
            contract: contract.clone(),
            assessment: assessment.clone(),
        };
        let human = report.render_human();
        assert!(human.starts_with("ready:"), "{human}");
        assert!(
            human.contains("declares no machine check"),
            "a zero-check report did not say that nothing was established:\n{human}"
        );
    }

    /// A note beginning the stem with a version this program does not read is an error, not prose.
    ///
    /// The failure this prevents is the quiet one: skipping `preflight/v3` would report a runbook
    /// ready when the runbook asked for checks this binary cannot perform.
    #[test]
    fn an_unsupported_format_version_is_refused_rather_than_skipped() {
        for note in [
            "preflight/v3 {\"kind\":\"state\",\"check\":\"outside-project\"}",
            "preflight/v0 {\"kind\":\"tool\",\"program\":\"cargo\",\"version\":\"1.90.0\"}",
            "preflight/latest {}",
            "preflight/",
        ] {
            let error = one(note).expect_err("an unknown version was read as prose or as v1");
            let text = reported(&error);
            assert!(
                error.faults().iter().all(|fault| matches!(
                    fault.fault,
                    DeclarationFault::UnsupportedFormat { .. }
                        | DeclarationFault::NotOneObject { .. }
                )),
                "{note:?} reported {text}"
            );
            assert!(
                text.contains("preflight"),
                "the report does not name the marker: {text}"
            );
        }
    }

    /// `preflight/v2` reads every v1 form, with and without a remedy, and a v1 declaration reads as
    /// it always has.
    #[test]
    fn v2_reads_every_v1_form_with_an_optional_remedy() {
        let contract = contract_of(&[
            (
                "tool",
                &["preflight/v2 {\"kind\":\"tool\",\"program\":\"python3\",\"version\":\"3.13.2\",\
                   \"remedy\":\"brew install python@3.13\"}"],
            ),
            (
                "toolchain",
                &["preflight/v2 {\"kind\":\"toolchain\",\"manifest\":\"flake.nix\",\
                   \"shell\":\"default\",\"tools\":[\"cargo\"],\
                   \"remedy\":\"run this inside nix develop\"}"],
            ),
            (
                "skill",
                &["preflight/v2 {\"kind\":\"skill\",\"scope\":\"home\",\
                   \"path\":\".claude/skills/login/SKILL.md\",\
                   \"remedy\":\"see https://example.com/skills\"}"],
            ),
            (
                "state",
                &["preflight/v2 {\"kind\":\"state\",\"check\":\"outside-project\"}"],
            ),
            (
                "old",
                &["preflight/v1 {\"kind\":\"state\",\"check\":\"clean-git-worktree\"}"],
            ),
        ])
        .expect("every form reads under v2, and v1 still reads");

        assert_eq!(
            declared(&contract),
            [
                ("req-tool", "tool"),
                ("req-toolchain", "toolchain"),
                ("req-skill", "skill"),
                ("req-state", "state"),
                ("req-old", "state"),
            ]
            .map(|(id, kind)| (String::from(id), String::from(kind)))
        );
        let remedy = |id: &str| {
            contract
                .remedy_of(&RequirementId::new(id).expect("an identifier"))
                .map(ToString::to_string)
        };
        assert_eq!(
            remedy("req-tool").as_deref(),
            Some("brew install python@3.13")
        );
        assert_eq!(
            remedy("req-skill").as_deref(),
            Some("see https://example.com/skills")
        );
        assert_eq!(
            remedy("req-state"),
            None,
            "a v2 declaration with no remedy has none"
        );
        assert_eq!(remedy("req-old"), None, "a v1 declaration has none");
    }

    /// v1 is unchanged: a remedy is a field its forms do not have, so it is refused and named.
    #[test]
    fn v1_still_refuses_a_remedy() {
        let error = one(
            "preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\",\
             \"remedy\":\"move out of the project\"}",
        )
        .expect_err("v1 accepted a remedy");
        assert!(
            error.faults().iter().any(|fault| matches!(
                &fault.fault,
                DeclarationFault::UnknownField { field } if field == "remedy"
            )),
            "{error}"
        );
    }

    /// A remedy at each limit reads, and one past either limit is refused.
    #[test]
    fn a_remedy_reads_at_its_limits_and_is_refused_past_them() {
        let declaration = |remedy: &str| {
            format!(
                "preflight/v2 {{\"kind\":\"state\",\"check\":\"outside-project\",\
                 \"remedy\":{}}}",
                serde_json::Value::String(remedy.to_owned())
            )
        };

        for accepted in ["x", &"é".repeat(REMEDY_LIMIT)] {
            let contract = one(&declaration(accepted)).expect("a remedy within its limits");
            assert_eq!(
                contract
                    .remedy_of(&RequirementId::new("req-one").expect("an identifier"))
                    .map(ToString::to_string)
                    .as_deref(),
                Some(accepted)
            );
        }

        // Counted in characters rather than bytes: 201 two-byte characters are one too many, and
        // 200 of them, above, are not.
        let error = one(&declaration(&"é".repeat(REMEDY_LIMIT + 1))).expect_err("too long");
        assert!(
            error.faults().iter().any(|fault| matches!(
                fault.fault,
                DeclarationFault::InvalidRemedy {
                    fault: RemedyFault::TooLong { characters: 201 }
                }
            )),
            "{error}"
        );

        for empty in ["", "   "] {
            let error = one(&declaration(empty)).expect_err("an empty remedy");
            assert!(
                error.faults().iter().any(|fault| matches!(
                    fault.fault,
                    DeclarationFault::InvalidRemedy {
                        fault: RemedyFault::Empty
                    }
                )),
                "{empty:?}: {error}"
            );
        }
    }

    /// `text` as a JSON string literal with every character outside printable ASCII escaped.
    ///
    /// The JSON escape is what an author writes in a document to state such a character: the markup
    /// itself cannot hold most of them without needing error recovery. A character beyond the Basic
    /// Multilingual Plane is written as its UTF-16 surrogate pair, which is how JSON spells it.
    fn json_escaped(text: &str) -> String {
        let mut out = String::from("\"");
        for character in text.chars() {
            if character.is_ascii_graphic() && character != '"' && character != '\\' {
                out.push(character);
            } else if character == ' ' {
                out.push(' ');
            } else {
                let mut units = [0_u16; 2];
                for unit in character.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
        }
        out.push('"');
        out
    }

    /// The declaration of an `outside-project` check whose remedy is `remedy`.
    fn remedied_with(remedy: &str) -> Result<MachineContract, ContractErrors> {
        one(&format!(
            "preflight/v2 {{\"kind\":\"state\",\"check\":\"outside-project\",\"remedy\":{}}}",
            json_escaped(remedy)
        ))
    }

    /// The remedy `remedy` reads as, once accepted.
    fn accepted_remedy(remedy: &str) -> String {
        remedied_with(remedy)
            .unwrap_or_else(|error| panic!("{remedy:?} was refused: {error}"))
            .remedy_of(&RequirementId::new("req-one").expect("an identifier"))
            .map(ToString::to_string)
            .expect("a remedy")
    }

    /// Asserts that `remedy` is refused for `found`, and that the refusal does not echo it.
    fn refused_for(remedy: &str, found: char) {
        let error = remedied_with(remedy).expect_err("an unprintable remedy was accepted");
        assert!(
            error.faults().iter().any(|fault| fault.fault
                == DeclarationFault::InvalidRemedy {
                    fault: RemedyFault::Unprintable { found }
                }),
            "{remedy:?}: {error}"
        );
        // The fault names the character by its code point rather than echoing it.
        assert!(!error.to_string().contains(found), "{remedy:?}: {error}");
    }

    /// A remedy holding a character a terminal acts on rather than shows is refused.
    ///
    /// A carried runbook's remedy reaches an operator's terminal, so an escape sequence, a line
    /// break, or a bidirectional override in it would be the runbook drawing on that terminal.
    #[test]
    fn a_remedy_holding_an_escape_or_a_reordering_character_is_refused() {
        for (given, found) in [
            ("run \u{1b}[2Jthis", '\u{1b}'),
            ("first line\nsecond line", '\n'),
            ("a\ttab", '\t'),
            ("delete\u{7f}", '\u{7f}'),
            ("next\u{85}line", '\u{85}'),
            ("hello \u{202e}dlrow", '\u{202e}'),
            ("isolate \u{2066}this\u{2069}", '\u{2066}'),
        ] {
            refused_for(given, found);
        }
    }

    /// A remedy holding an invisible formatting character is refused, so what a terminal shows is
    /// all the remedy holds.
    ///
    /// Each class the review of this change found accepted: zero-width and word-joining characters,
    /// the byte-order mark, the soft hyphen, the Mongolian vowel separator, interlinear annotation,
    /// and the tag characters, which spell hidden ASCII.
    #[test]
    fn a_remedy_holding_an_invisible_formatting_character_is_refused() {
        for found in [
            '\u{200B}',
            '\u{2060}',
            '\u{2061}',
            '\u{2062}',
            '\u{2063}',
            '\u{2064}',
            '\u{FEFF}',
            '\u{00AD}',
            '\u{180E}',
            '\u{FFF9}',
            '\u{FFFA}',
            '\u{FFFB}',
            '\u{E0001}',
            '\u{E0020}',
            '\u{E0041}',
            '\u{E007F}',
            '\u{0600}',
            '\u{110BD}',
            '\u{1BCA0}',
            '\u{1D173}',
        ] {
            refused_for(&format!("brew install x{found}"), found);
        }
    }

    /// A remedy holding a default-ignorable character is refused, so what a terminal shows is all
    /// the remedy holds.
    ///
    /// The pre-archival review of this change found these accepted while the rule refused only the
    /// format category: variation selectors, the Hangul fillers, the combining grapheme joiner, the
    /// Khmer inherent vowels, and a Mongolian free variation selector each draw as nothing.
    #[test]
    fn a_remedy_holding_a_default_ignorable_character_is_refused() {
        for found in [
            '\u{FE00}',
            '\u{FE01}',
            '\u{FE02}',
            '\u{3164}',
            '\u{FFA0}',
            '\u{115F}',
            '\u{034F}',
            '\u{17B4}',
            '\u{180B}',
            '\u{E0100}',
            '\u{E01EF}',
            // Neither default-ignorable nor whitespace, and drawn as a blank all the same.
            '\u{2800}',
        ] {
            refused_for(&format!("brew install x{found}"), found);
        }
    }

    /// An emoji presentation selector is kept directly after a visible character, where it chooses
    /// how that character is drawn, and refused anywhere else, where it would draw as nothing.
    #[test]
    fn a_remedy_keeps_a_presentation_selector_only_after_a_visible_character() {
        for given in [
            "ask in \u{2764}\u{FE0F} the channel",
            "see \u{2139}\u{FE0E} the wiki",
        ] {
            assert_eq!(accepted_remedy(given), given);
        }
        for (given, found) in [
            ("\u{FE0F}ask", '\u{FE0F}'),
            ("ask \u{FE0F}", '\u{FE0F}'),
            ("ask\u{2764}\u{FE0F}\u{FE0F}", '\u{FE0F}'),
            ("ask\u{200D}\u{FE0E}", '\u{FE0E}'),
        ] {
            refused_for(given, found);
        }
    }

    /// A declared tool's name and a skill's path reach a report with every character a terminal
    /// would not show as itself named by code point, as a field's name does.
    #[test]
    fn a_declared_name_or_path_is_shown_with_its_hidden_characters_named() {
        let program = ProgramName::new("x\u{1b}[31mred").expect("a bare name");
        assert_eq!(program.to_string(), "x<U+001B>[31mred");
        assert_eq!(
            program.as_str(),
            "x\u{1b}[31mred",
            "the probe runs the name as written"
        );
        let path = ScopedRelativePath::new("a\u{1b}[2Jb/SKILL.md").expect("a relative path");
        assert_eq!(path.to_string(), "a<U+001B>[2Jb/SKILL.md");
        let check = MachineCheck::Tool {
            program,
            version: ToolVersion::parse("1.2.3").expect("a version"),
        };
        assert!(!check.expected().contains('\u{1b}'), "{}", check.expected());
        assert!(!check.remedy().contains('\u{1b}'), "{}", check.remedy());
    }

    /// The two joiners stay, because a script or an emoji sequence needs them, and they are the
    /// only formatting characters a remedy keeps.
    #[test]
    fn a_remedy_keeps_the_joiners_a_script_needs() {
        for given in ["family \u{1F468}\u{200D}\u{1F469}", "می\u{200C}خواهم"] {
            assert_eq!(accepted_remedy(given), given);
        }
    }

    /// One line, with plain spaces between its words: every other whitespace character is refused,
    /// the Unicode line and paragraph separators included.
    #[test]
    fn a_remedy_holding_a_line_separator_or_other_whitespace_is_refused() {
        for found in [
            '\u{2028}', '\u{2029}', '\u{00A0}', '\u{2003}', '\u{3000}', '\u{000B}', '\u{000C}',
        ] {
            refused_for(&format!("first{found}second"), found);
        }
    }

    /// A remedy with no visible character says nothing, so it is refused as empty.
    #[test]
    fn a_remedy_with_no_visible_character_is_empty() {
        for given in ["\u{200D}", " \u{200C} ", "\u{200C}\u{200D}"] {
            let error = remedied_with(given).expect_err("an invisible remedy was accepted");
            assert!(
                error.faults().iter().any(|fault| fault.fault
                    == DeclarationFault::InvalidRemedy {
                        fault: RemedyFault::Empty
                    }),
                "{given:?}: {error}"
            );
        }
    }

    /// Spaces around a remedy are trimmed, and its limit is counted on what remains.
    #[test]
    fn a_remedy_is_trimmed_of_the_spaces_around_it() {
        assert_eq!(accepted_remedy("  brew install x  "), "brew install x");
        let full = "x".repeat(REMEDY_LIMIT);
        assert_eq!(accepted_remedy(&format!("   {full}   ")), full);
    }

    /// A field name a declaration holds is shown with its unprintable characters named by code
    /// point, so a key holding an escape sequence cannot write one into the report.
    ///
    /// A gap since `preflight/v1` shipped in 1.0: the unknown-field fault echoed the key as written.
    #[test]
    fn an_unknown_field_holding_an_escape_is_named_without_it() {
        for (key, found) in [("x\u{1b}[2J", '\u{1b}'), ("y\u{202e}z", '\u{202e}')] {
            let note = format!(
                "preflight/v1 {{\"kind\":\"state\",\"check\":\"outside-project\",{}:\"a\"}}",
                json_escaped(key)
            );
            let error = one(&note).expect_err("an unknown field was accepted");
            let text = error.to_string();
            assert!(!text.contains(found), "{key:?}: {text}");
            assert!(
                text.contains(&format!("U+{:04X}", u32::from(found))),
                "{key:?} is not named by its code point: {text}"
            );
        }
    }

    /// A remedy of another type is refused and named, beside every other fault in the declaration.
    #[test]
    fn a_remedy_of_another_type_is_named_beside_the_forms_own_faults() {
        let error = one(
            "preflight/v2 {\"kind\":\"tool\",\"program\":\"/usr/bin/cargo\",\"version\":\"1.90.0\",\
             \"remedy\":[\"install it\"]}",
        )
        .expect_err("two faults");
        assert!(
            error.faults().iter().any(|fault| matches!(
                &fault.fault,
                DeclarationFault::WrongType { field, .. } if field == "remedy"
            )),
            "the remedy's type is not named: {error}"
        );
        assert!(
            error
                .faults()
                .iter()
                .any(|fault| matches!(fault.fault, DeclarationFault::InvalidProgram { .. })),
            "the form's own fault was dropped: {error}"
        );
        assert!(
            error
                .faults()
                .iter()
                .all(|fault| fault.requirement.is_some()),
            "a fault lost its requirement: {error}"
        );
    }

    /// A malformed declaration retains the requirement that made it.
    ///
    /// Every fault, because a report naming a field and no requirement sends an author searching a
    /// document for a string that may appear in several places.
    #[test]
    fn every_fault_names_the_requirement_that_declared_it() {
        let error = contract_of(&[
            (
                "good",
                &["preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\"}"],
            ),
            ("bad", &["preflight/v1 {\"kind\":\"state\"}"]),
            ("worse", &["preflight/v1 {\"kind\":\"nope\"}"]),
        ])
        .expect_err("two malformed declarations");

        let subjects = error
            .faults()
            .iter()
            .filter_map(|fault| fault.requirement.as_ref())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert!(
            subjects.contains(&String::from("req-bad")),
            "the missing-field fault lost its requirement: {error}"
        );
        assert!(
            subjects.contains(&String::from("req-worse")),
            "the unknown-kind fault lost its requirement: {error}"
        );
        assert!(
            error
                .faults()
                .iter()
                .all(|fault| fault.requirement.is_some()),
            "a fault carries no requirement: {error}"
        );
    }

    /// An unknown field is refused rather than ignored.
    ///
    /// The load-bearing half of "reject unknown fields": an ignored field is a condition this
    /// program does not check, and a readiness report that stayed quiet about it would claim a
    /// check that never ran.
    #[test]
    fn an_unknown_field_is_refused_and_named() {
        let error = one(
            "preflight/v1 {\"kind\":\"tool\",\"program\":\"cargo\",\"version\":\"1.90.0\",\
             \"command\":\"cargo build\"}",
        )
        .expect_err("an extra field was accepted");
        let text = reported(&error);
        assert!(
            error.faults().iter().any(|fault| matches!(
                &fault.fault,
                DeclarationFault::UnknownField { field } if field == "command"
            )),
            "the report does not name the field: {text}"
        );
        assert!(
            text.contains("does not check"),
            "the report does not say why an ignored field is refused: {text}"
        );
    }

    /// A missing required field is named, and so is every other fault in the same declaration.
    #[test]
    fn a_missing_field_is_named_beside_the_other_faults() {
        let error =
            one("preflight/v1 {\"kind\":\"tool\",\"program\":\"cargo\"}").expect_err("no version");
        assert!(
            error.faults().iter().any(|fault| matches!(
                fault.fault,
                DeclarationFault::MissingField { field: "version" }
            )),
            "the missing field was not named: {error}"
        );

        // Two bad fields in one declaration report both, so an author fixes it in one pass.
        let error = one(
            "preflight/v1 {\"kind\":\"tool\",\"program\":\"/usr/bin/cargo\",\"version\":\"1.90\"}",
        )
        .expect_err("two invalid values");
        assert_eq!(
            error.faults().len(),
            2,
            "one of the two faults was dropped: {error}"
        );
    }

    /// An unknown kind is refused, and the report names what the forms are.
    #[test]
    fn an_unknown_kind_is_refused_naming_the_forms() {
        let error = one("preflight/v1 {\"kind\":\"command\",\"run\":\"cargo build\"}")
            .expect_err("an arbitrary command form was accepted");
        let text = reported(&error);
        assert!(
            error.faults().iter().any(|fault| matches!(
                &fault.fault,
                DeclarationFault::UnknownKind { kind } if kind == "command"
            )),
            "{text}"
        );
        for form in ["tool", "toolchain", "skill", "state"] {
            assert!(
                text.contains(form),
                "the report does not name {form}: {text}"
            );
        }
        assert!(
            text.contains("cannot supply its own command"),
            "the report does not say why a runbook cannot declare a command: {text}"
        );
    }

    /// A declaration with no `kind` says so rather than reporting a missing field for every form.
    #[test]
    fn a_declaration_with_no_kind_is_reported_as_having_none() {
        let error = one("preflight/v1 {\"program\":\"cargo\"}").expect_err("no kind");
        assert!(
            error
                .faults()
                .iter()
                .any(|fault| fault.fault == DeclarationFault::NoKind),
            "{error}"
        );
    }

    /// A declaration that is not one JSON object is refused, whatever it is instead.
    #[test]
    fn a_declaration_that_is_not_one_object_is_refused() {
        for (note, wanted) in [
            ("preflight/v1", "nothing"),
            ("preflight/v1 []", "array"),
            ("preflight/v1 \"tool\"", "string"),
            ("preflight/v1 not json at all", "not JSON"),
        ] {
            let error = one(note).expect_err("accepted");
            let text = reported(&error);
            assert!(
                text.contains(wanted),
                "{note:?} did not report {wanted:?}: {text}"
            );
        }
    }

    /// Two declarations on one requirement are refused, and the report says how to split them.
    #[test]
    fn two_declarations_on_one_requirement_are_refused() {
        let error = contract_of(&[(
            "twice",
            &[
                "preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\"}",
                "preflight/v1 {\"kind\":\"state\",\"check\":\"clean-git-worktree\"}",
            ],
        )])
        .expect_err("a duplicate declaration was accepted");
        assert!(
            error.faults().iter().any(|fault| fault.fault
                == DeclarationFault::DuplicateDeclaration
                && fault
                    .requirement
                    .as_ref()
                    .is_some_and(|id| id.as_str() == "req-twice")),
            "the duplicate was not reported against its requirement: {error}"
        );
        assert!(
            reported(&error).contains("separate requirements"),
            "the report does not say how to fix it: {error}"
        );

        // And the value-level constructor holds the same line, so no path builds such a contract.
        let check = MachineCheck::State {
            check: WorkingState::OutsideProject,
        };
        let id = RequirementId::new("req-twice").expect("an identifier");
        let built = MachineContract::of(vec![
            DeclaredCheck {
                requirement: id.clone(),
                check: check.clone(),
                remedy: None,
            },
            DeclaredCheck {
                requirement: id,
                check,
                remedy: None,
            },
        ]);
        assert!(
            built.is_err(),
            "MachineContract::of built a contract with two checks under one requirement"
        );
    }

    /// A declaration on a requirement with no identifier is refused rather than checked
    /// anonymously.
    ///
    /// A passed check is copied into a verification naming its requirement, so a requirement
    /// nothing can name is one a record cannot attest to.
    #[test]
    fn a_declaration_on_an_unidentified_requirement_is_refused() {
        let source = "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"0.1.0\" name=\"htmlspec-version\">\
             <meta content=\"specification\" name=\"htmlspec-kind\"><title>F</title></head>\
             <body><main>\
             <section class=\"requirement\" data-change=\"unchanged\"><h2>No identifier</h2>\
             <p>A run MUST start outside a project.</p>\
             <article class=\"scenario\" data-change=\"unchanged\"><h3>S</h3>\
             <dl><dt>When</dt><dd>A run starts.</dd><dt>Then</dt><dd>It records it.</dd></dl>\
             </article>\
             <p data-field=\"note\">preflight/v1 {&quot;kind&quot;:&quot;state&quot;,\
             &quot;check&quot;:&quot;outside-project&quot;}</p>\
             </section></main></body></html>";
        let parsed = htmlspec_parse::parse_document(source);
        let projection = htmlspec_parse::project(&parsed.document);
        let error = parse_contract(&projection.model).expect_err("an anonymous check was accepted");
        assert!(
            error
                .faults()
                .iter()
                .any(|fault| fault.fault == DeclarationFault::NoIdentifier),
            "{error}"
        );
        assert!(
            reported(&error).contains("no identifier"),
            "the report does not say what is missing: {error}"
        );
    }

    /// A withdrawn requirement's declaration is not a check.
    ///
    /// The record does not copy a withdrawn requirement, so a check it declared could never be
    /// attested to — and refusing a run over an obligation that no longer applies is the defect
    /// [`crate::runs::is_withdrawn`] exists to prevent.
    #[test]
    fn a_withdrawn_requirement_declares_nothing() {
        let source = "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"0.1.0\" name=\"htmlspec-version\">\
             <meta content=\"specification\" name=\"htmlspec-kind\"><title>F</title></head>\
             <body><main>\
             <section class=\"requirement\" id=\"req-gone\" data-change=\"removed\"><h2>Gone</h2>\
             <p>A run MUST start outside a project.</p>\
             <p data-field=\"note\">preflight/v1 {&quot;kind&quot;:&quot;state&quot;,\
             &quot;check&quot;:&quot;outside-project&quot;}</p>\
             </section></main></body></html>";
        let parsed = htmlspec_parse::parse_document(source);
        let projection = htmlspec_parse::project(&parsed.document);
        let contract = parse_contract(&projection.model).expect("a withdrawn unit is not a fault");
        assert!(
            contract.is_empty(),
            "a withdrawn requirement's declaration became a check: {:?}",
            declared(&contract)
        );
    }

    /// A program name that is a path, an option, or a command line is refused.
    ///
    /// Each case is a different way the field would reach something it should not: a path bypasses
    /// the search path being checked, an option becomes an argument the probe did not choose, and a
    /// command line is two names.
    #[test]
    fn a_program_name_that_is_not_a_name_is_refused() {
        for (given, fault) in [
            ("/usr/bin/cargo", NameFault::Separator),
            ("bin/cargo", NameFault::Separator),
            ("bin\\cargo", NameFault::Separator),
            ("--version", NameFault::OptionLike),
            ("-c", NameFault::OptionLike),
            ("cargo build", NameFault::Whitespace),
            ("", NameFault::Empty),
        ] {
            let error = ProgramName::new(given).expect_err("{given:?} was accepted");
            assert_eq!(error, fault, "{given:?} reported {error:?}");

            // And through the declaration, which is where it arrives.
            let declared = one(&format!(
                "preflight/v1 {{\"kind\":\"tool\",\"program\":\"{given}\",\"version\":\"1.2.3\"}}"
            ));
            assert!(
                declared.is_err(),
                "{given:?} reached a check through a declaration"
            );
        }
        assert_eq!(
            ProgramName::new("  cargo  ")
                .expect("surrounding space is trimmed")
                .as_str(),
            "cargo"
        );
    }

    /// A declared tool version is exactly three numeric components.
    ///
    /// The comparison is numeric, so `3.13` and `3.13.0` are not the same declaration and only one
    /// of them can be checked against what a tool reports.
    #[test]
    fn a_tool_version_is_three_numeric_components() {
        for given in ["3.13", "3", "3.13.2.1", "", "3.13.x"] {
            assert!(
                ToolVersion::parse(given).is_err(),
                "{given:?} was accepted as a tool version"
            );
            assert!(
                one(&format!(
                    "preflight/v1 {{\"kind\":\"tool\",\"program\":\"python3\",\
                     \"version\":\"{given}\"}}"
                ))
                .is_err(),
                "{given:?} reached a check through a declaration"
            );
        }
        let parsed = ToolVersion::parse("3.13.2").expect("parses");
        assert_eq!(parsed.to_string(), "3.13.2");
        assert!(parsed > ToolVersion::parse("3.9.0").expect("parses"));
    }

    /// A path with an absolute root or a `..` is refused, for both path-taking forms.
    #[test]
    fn a_path_that_leaves_its_scope_is_refused() {
        for (given, fault) in [
            ("/etc/passwd", PathFault::Absolute),
            ("../outside/SKILL.md", PathFault::Escapes),
            (".agents/../../outside", PathFault::Escapes),
            ("", PathFault::Empty),
        ] {
            assert_eq!(
                ScopedRelativePath::new(given).expect_err("accepted"),
                fault,
                "{given:?}"
            );
            assert!(
                one(&format!(
                    "preflight/v1 {{\"kind\":\"skill\",\"scope\":\"home\",\"path\":\"{given}\"}}"
                ))
                .is_err(),
                "{given:?} reached a skill check"
            );
            assert!(
                one(&format!(
                    "preflight/v1 {{\"kind\":\"toolchain\",\"manifest\":\"{given}\",\
                     \"shell\":\"default\",\"tools\":[\"cargo\"]}}"
                ))
                .is_err(),
                "{given:?} reached a toolchain check"
            );
        }

        // A nested relative path is fine, which is the ordinary case.
        let nested = ScopedRelativePath::new(".agents/skills/example/SKILL.md").expect("parses");
        assert_eq!(
            nested.within(Path::new("/home/operator")),
            PathBuf::from("/home/operator/.agents/skills/example/SKILL.md")
        );
    }

    /// An unknown scope or state is refused, and neither falls back to the other.
    #[test]
    fn an_unknown_scope_or_state_is_refused() {
        let error =
            one("preflight/v1 {\"kind\":\"skill\",\"scope\":\"project\",\"path\":\"a/SKILL.md\"}")
                .expect_err("an unknown scope");
        assert!(
            reported(&error).contains("no fallback"),
            "the report does not say the scopes do not fall back: {error}"
        );

        let error = one("preflight/v1 {\"kind\":\"state\",\"check\":\"oven-preheated\"}")
            .expect_err("an unsupported state");
        let text = reported(&error);
        assert!(
            text.contains("outside-project") && text.contains("clean-git-worktree"),
            "the report does not name the supported states: {text}"
        );
        assert!(
            text.contains("manual obligation"),
            "the report does not say where an unsupported condition goes instead: {text}"
        );
    }

    /// A toolchain declaring no tool has nothing to check, and says so.
    #[test]
    fn a_toolchain_with_no_tool_is_refused() {
        let error = one(
            "preflight/v1 {\"kind\":\"toolchain\",\"manifest\":\"flake.nix\",\"shell\":\"default\",\
             \"tools\":[]}",
        )
        .expect_err("an empty tool list");
        assert!(
            error
                .faults()
                .iter()
                .any(|fault| fault.fault == DeclarationFault::NoTools),
            "{error}"
        );
    }

    /// A field of the wrong JSON type is refused naming the field.
    #[test]
    fn a_field_of_the_wrong_type_is_refused_naming_the_field() {
        let error = one(
            "preflight/v1 {\"kind\":\"toolchain\",\"manifest\":\"flake.nix\",\"shell\":\"default\",\
             \"tools\":\"cargo\"}",
        )
        .expect_err("a string where a list belongs");
        assert!(
            reported(&error).contains("tools"),
            "the report does not name the field: {error}"
        );

        let error = one("preflight/v1 {\"kind\":\"tool\",\"program\":3,\"version\":\"1.2.3\"}")
            .expect_err("a number where a name belongs");
        assert!(
            reported(&error).contains("program"),
            "the report does not name the field: {error}"
        );
    }

    /// A shell name that could not go into an expression safely is refused.
    ///
    /// The name reaches an attribute path this program assembles, so a quote in it is a fragment of
    /// expression supplied by a carried document — the power `runbook-portability` withholds.
    #[test]
    fn a_shell_name_that_would_reach_an_expression_is_refused() {
        for given in ["default\" --offline \"", "a b", "", "shell;rm", "${out}"] {
            assert!(
                ShellName::new(given).is_err(),
                "{given:?} was accepted as a shell name"
            );
        }
        assert!(ShellName::new("default").is_ok());
        assert!(ShellName::new("rust-nightly").is_ok());
    }

    /// Whitespace inside the declaration survives projection, which is what makes the note a
    /// usable wire format at all.
    ///
    /// Asserted through the real projection rather than against a hand-built model: the note's text
    /// reaches the parser as the projection produced it, and a declaration that only parsed from
    /// hand-written JSON would be a format no author could write.
    #[test]
    fn a_declaration_survives_projection_and_formatting() {
        const NOTE: &str =
            "preflight/v1 {\"kind\":\"tool\",\"program\":\"python3\",\"version\":\"3.13.2\"}";
        let source = format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"0.1.0\" name=\"htmlspec-version\">\
             <meta content=\"specification\" name=\"htmlspec-kind\"><title>F</title></head>\
             <body><main>\
             <section class=\"requirement\" id=\"req-tool\" data-change=\"unchanged\"><h2>T</h2>\
             <p>A run MUST find python3.</p>\
             <article class=\"scenario\" id=\"scenario-tool\" data-change=\"unchanged\"><h3>S</h3>\
             <dl><dt>When</dt><dd>A run starts.</dd><dt>Then</dt><dd>It records it.</dd></dl>\
             </article>\
             <p data-field=\"note\">{NOTE}</p>\
             </section></main></body></html>"
        );

        // The declaration parses from the source as authored.
        let parsed = htmlspec_parse::parse_document(&source);
        let projection = htmlspec_parse::project(&parsed.document);
        let before = parse_contract(&projection.model).expect("parses as authored");

        // And from the same document after canonical formatting, which is the state it is read in
        // whenever `htmlspec format` has run — the formatter never rewraps text, so the JSON is
        // byte-identical, and this asserts that rather than assuming it.
        let formatted = htmlspec_format::format(&source).expect("formats");
        let parsed = htmlspec_parse::parse_document(&formatted);
        let projection = htmlspec_parse::project(&parsed.document);
        let after = parse_contract(&projection.model).expect("parses after formatting");

        assert_eq!(
            before, after,
            "canonical formatting changed what the declaration means"
        );
        assert_eq!(
            declared(&after),
            vec![(String::from("req-tool"), String::from("tool"))]
        );
    }

    /// A note that merely mentions the marker inside prose is not a declaration.
    ///
    /// The stem is matched at the start of the projected text, so a note *about* the format stays
    /// prose. Without this, documenting the format in a runbook's own specification would add a
    /// check to it.
    #[test]
    fn prose_mentioning_the_marker_is_not_a_declaration() {
        let contract = contract_of(&[(
            "prose",
            &["Declare a check with preflight/v1 followed by a JSON object."],
        )])
        .expect("prose");
        assert!(contract.is_empty(), "prose about the format became a check");
    }

    /// Observations are matched to checks by requirement identity, and a hole is refused.
    ///
    /// This is what makes [`assess`] total: a value of [`Observations`] holds exactly one
    /// observation per declared check, so the assessment cannot meet a missing fact and cannot be
    /// tricked by a duplicate one.
    #[test]
    fn observations_must_cover_every_declared_check_exactly_once() {
        let contract = contract_of(&[
            (
                "one",
                &["preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\"}"],
            ),
            (
                "two",
                &["preflight/v1 {\"kind\":\"state\",\"check\":\"clean-git-worktree\"}"],
            ),
        ])
        .expect("parses");
        let state = |check: WorkingState| MachineCheck::State { check };
        let observed = |id: &str, check: WorkingState| Observed {
            requirement: RequirementId::new(id).expect("an identifier"),
            check: state(check),
            observation: Observation::Measured(Fact::NoNearestProject),
        };

        // Both present, in either order: the constructor reorders to document order.
        let both = Observations::of(
            &contract,
            vec![
                observed("req-two", WorkingState::CleanGitWorktree),
                observed("req-one", WorkingState::OutsideProject),
            ],
        )
        .expect("both observed");
        assert_eq!(
            both.observed()
                .iter()
                .map(|entry| entry.requirement.to_string())
                .collect::<Vec<_>>(),
            vec!["req-one", "req-two"],
            "the observations are not in document order"
        );

        // One missing: refused, so no assessment can call it ready.
        let missing = Observations::of(
            &contract,
            vec![observed("req-one", WorkingState::OutsideProject)],
        )
        .expect_err("a missing fact was accepted");
        assert!(
            missing.iter().any(|fault| matches!(
                fault,
                ObservationFault::Missing { requirement } if requirement.as_str() == "req-two"
            )),
            "{missing:?}"
        );

        // One duplicated: refused for the same reason.
        let duplicated = Observations::of(
            &contract,
            vec![
                observed("req-one", WorkingState::OutsideProject),
                observed("req-one", WorkingState::OutsideProject),
                observed("req-two", WorkingState::CleanGitWorktree),
            ],
        )
        .expect_err("a duplicate fact was accepted");
        assert!(
            duplicated
                .iter()
                .any(|fault| matches!(fault, ObservationFault::Duplicate { .. })),
            "{duplicated:?}"
        );

        // One undeclared: refused, because nothing can compare it to a condition.
        let undeclared = Observations::of(
            &contract,
            vec![
                observed("req-one", WorkingState::OutsideProject),
                observed("req-two", WorkingState::CleanGitWorktree),
                observed("req-three", WorkingState::OutsideProject),
            ],
        )
        .expect_err("an undeclared observation was accepted");
        assert!(
            undeclared
                .iter()
                .any(|fault| matches!(fault, ObservationFault::Undeclared { .. })),
            "{undeclared:?}"
        );

        // One carrying the wrong check: refused, because the fact and the condition would be about
        // different requirements.
        let mismatched = Observations::of(
            &contract,
            vec![
                observed("req-one", WorkingState::OutsideProject),
                observed("req-two", WorkingState::OutsideProject),
            ],
        )
        .expect_err("a mismatched observation was accepted");
        assert!(
            mismatched
                .iter()
                .any(|fault| matches!(fault, ObservationFault::Mismatched { .. })),
            "{mismatched:?}"
        );
    }

    /// A measured fact that does not satisfy the declaration is an unmet condition naming both
    /// values.
    #[test]
    fn a_version_mismatch_names_the_expected_and_observed_versions() {
        let contract =
            one("preflight/v1 {\"kind\":\"tool\",\"program\":\"python3\",\"version\":\"3.13.2\"}")
                .expect("parses");
        let program = ProgramName::new("python3").expect("a name");
        let observations = Observations::of(
            &contract,
            vec![Observed {
                requirement: RequirementId::new("req-one").expect("an identifier"),
                check: contract.checks[0].check.clone(),
                observation: Observation::Measured(Fact::ToolVersion {
                    program: program.clone(),
                    resolved: PathBuf::from("/usr/bin/python3"),
                    version: ToolVersion::parse("3.12.0").expect("parses"),
                }),
            }],
        )
        .expect("matched");

        match assess(&contract, observations) {
            PreflightAssessment::Ready(_) => panic!("a mismatch was reported as ready"),
            PreflightAssessment::NotReady(findings) => {
                assert_eq!(findings.findings().len(), 1);
                let finding = &findings.findings()[0];
                assert_eq!(finding.outcome, CheckOutcome::Unmet);
                assert_eq!(finding.requirement.as_str(), "req-one");
                assert!(
                    finding.observed.contains("3.12.0"),
                    "the report does not name the observed version: {}",
                    finding.observed
                );
                assert!(
                    finding.expected.contains("3.13.2"),
                    "the report does not name the declared version: {}",
                    finding.expected
                );
                assert!(
                    finding.remedy.contains("python3"),
                    "the remedy does not name the tool: {}",
                    finding.remedy
                );
                assert!(
                    !finding.remedy.contains("install python3 automatically"),
                    "the remedy claims an installation this program does not perform"
                );
            }
        }
        assert!(program.as_str() == "python3");
    }

    /// An absent tool is a measured fact, and an unmet condition.
    #[test]
    fn a_missing_tool_is_unmet_rather_than_unknown() {
        let contract =
            one("preflight/v1 {\"kind\":\"tool\",\"program\":\"cargo\",\"version\":\"1.90.0\"}")
                .expect("parses");
        let assessment = assess(
            &contract,
            Observations::of(
                &contract,
                vec![Observed {
                    requirement: RequirementId::new("req-one").expect("an identifier"),
                    check: contract.checks[0].check.clone(),
                    observation: Observation::Measured(Fact::ToolAbsent {
                        program: ProgramName::new("cargo").expect("a name"),
                    }),
                }],
            )
            .expect("matched"),
        );
        match assessment {
            PreflightAssessment::NotReady(findings) => {
                assert_eq!(findings.findings()[0].outcome, CheckOutcome::Unmet);
                assert!(
                    findings.findings()[0].observed.contains("search path"),
                    "the report does not say where the tool was looked for: {}",
                    findings.findings()[0].observed
                );
            }
            PreflightAssessment::Ready(_) => panic!("an absent tool was reported as ready"),
        }
    }

    /// An unknown observation is unmet readiness rather than success, and stays distinguishable
    /// from a condition that was measured and found false.
    ///
    /// `toolchain-packaging` requires both halves: "Unknown observations such as an unavailable
    /// manifest environment or a timed-out probe SHALL be reported as unmet readiness rather than
    /// as success", and an unavailable environment is not the same report as a missing tool.
    #[test]
    fn an_unknown_observation_refuses_readiness_and_names_its_kind() {
        let contract = contract_of(&[
            (
                "gone",
                &["preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\"}"],
            ),
            (
                "slow",
                &["preflight/v1 {\"kind\":\"tool\",\"program\":\"cargo\",\"version\":\"1.90.0\"}"],
            ),
            (
                "nix",
                &[
                    "preflight/v1 {\"kind\":\"toolchain\",\"manifest\":\"flake.nix\",\
                     \"shell\":\"default\",\"tools\":[\"cargo\"]}",
                ],
            ),
            (
                "broken",
                &["preflight/v1 {\"kind\":\"skill\",\"scope\":\"home\",\"path\":\"a/SKILL.md\"}"],
            ),
        ])
        .expect("parses");

        let assessment = assess(
            &contract,
            Observations::of(
                &contract,
                vec![
                    Observed {
                        requirement: RequirementId::new("req-gone").expect("an identifier"),
                        check: contract.checks[0].check.clone(),
                        observation: Observation::Measured(Fact::NearestProject {
                            at: PathBuf::from("/work/portable"),
                        }),
                    },
                    Observed {
                        requirement: RequirementId::new("req-slow").expect("an identifier"),
                        check: contract.checks[1].check.clone(),
                        observation: Observation::TimedOut {
                            bound: Duration::from_secs(10),
                        },
                    },
                    Observed {
                        requirement: RequirementId::new("req-nix").expect("an identifier"),
                        check: contract.checks[2].check.clone(),
                        observation: Observation::Unavailable {
                            reason: String::from(
                                "the pinned environment's inputs are not realized",
                            ),
                            remedy: String::from("realize them with the package manager"),
                        },
                    },
                    Observed {
                        requirement: RequirementId::new("req-broken").expect("an identifier"),
                        check: contract.checks[3].check.clone(),
                        observation: Observation::ProbeFailure {
                            reason: String::from("the scope's root could not be read"),
                        },
                    },
                ],
            )
            .expect("matched"),
        );

        match assessment {
            PreflightAssessment::Ready(_) => panic!("four refusals were reported as ready"),
            PreflightAssessment::NotReady(findings) => {
                let outcomes = findings
                    .findings()
                    .iter()
                    .map(|finding| (finding.requirement.to_string(), finding.outcome))
                    .collect::<Vec<_>>();
                assert_eq!(
                    outcomes,
                    vec![
                        (String::from("req-gone"), CheckOutcome::Unmet),
                        (String::from("req-slow"), CheckOutcome::TimedOut),
                        (String::from("req-nix"), CheckOutcome::Unavailable),
                        (String::from("req-broken"), CheckOutcome::ProbeFailed),
                    ],
                    "the four endings were not kept distinct, or are out of document order"
                );
                // A timeout names its bound, because "it did not finish" without a number is not
                // actionable.
                assert!(
                    findings.findings()[1].observed.contains("10"),
                    "the timeout does not name its bound: {}",
                    findings.findings()[1].observed
                );
            }
        }
    }

    /// The bootstrap hazard: a working directory that resolves to a project is refused, and the
    /// report says what the consequence would have been.
    #[test]
    fn a_working_directory_inside_a_project_is_refused() {
        let contract =
            one("preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\"}").expect("parses");
        let assessment = assess(
            &contract,
            Observations::of(
                &contract,
                vec![Observed {
                    requirement: RequirementId::new("req-one").expect("an identifier"),
                    check: contract.checks[0].check.clone(),
                    observation: Observation::Measured(Fact::NearestProject {
                        at: PathBuf::from("/work/scratch/portable"),
                    }),
                }],
            )
            .expect("matched"),
        );
        match assessment {
            PreflightAssessment::NotReady(findings) => {
                let finding = &findings.findings()[0];
                assert_eq!(finding.outcome, CheckOutcome::Unmet);
                assert!(
                    finding.observed.contains("/work/scratch/portable"),
                    "the report does not name where the project was found: {}",
                    finding.observed
                );
            }
            PreflightAssessment::Ready(_) => panic!("the bootstrap hazard passed"),
        }

        // And the satisfying fact is the absence of a project, not a path.
        let ready = assess(
            &contract,
            Observations::of(
                &contract,
                vec![Observed {
                    requirement: RequirementId::new("req-one").expect("an identifier"),
                    check: contract.checks[0].check.clone(),
                    observation: Observation::Measured(Fact::NoNearestProject),
                }],
            )
            .expect("matched"),
        );
        assert!(ready.is_ready());
    }

    /// A dirty worktree names what it holds; a clean one passes.
    #[test]
    fn a_dirty_worktree_names_what_it_holds() {
        let contract = one("preflight/v1 {\"kind\":\"state\",\"check\":\"clean-git-worktree\"}")
            .expect("parses");
        let observation = |fact: Fact| {
            Observations::of(
                &contract,
                vec![Observed {
                    requirement: RequirementId::new("req-one").expect("an identifier"),
                    check: contract.checks[0].check.clone(),
                    observation: Observation::Measured(fact),
                }],
            )
            .expect("matched")
        };

        assert!(assess(&contract, observation(Fact::WorktreeClean)).is_ready());
        match assess(
            &contract,
            observation(Fact::WorktreeDirty {
                entries: vec![String::from(" M Cargo.toml"), String::from("?? new.rs")],
            }),
        ) {
            PreflightAssessment::NotReady(findings) => {
                let observed = &findings.findings()[0].observed;
                assert!(
                    observed.contains("Cargo.toml") && observed.contains("new.rs"),
                    "the report does not name the entries: {observed}"
                );
                assert!(
                    observed.contains("2 entr"),
                    "the report does not count them: {observed}"
                );
            }
            PreflightAssessment::Ready(_) => panic!("a dirty worktree passed"),
        }
    }

    /// A tool that resolves outside the pinned environment is refused naming both places.
    ///
    /// This is the shadowing case: the pinned environment supplies the tool, and the invocation's
    /// search path finds a different installation first. Neither half alone is the fault.
    #[test]
    fn a_tool_shadowing_a_pinned_one_is_refused_naming_where_it_came_from() {
        let contract = one(
            "preflight/v1 {\"kind\":\"toolchain\",\"manifest\":\"flake.nix\",\"shell\":\"default\",\
             \"tools\":[\"cargo\",\"rustc\"]}",
        )
        .expect("parses");
        let provenance =
            |program: &str, resolved: Option<&str>, from_inputs: bool| ToolProvenance {
                program: ProgramName::new(program).expect("a name"),
                resolved: resolved.map(PathBuf::from),
                from_inputs,
                input: from_inputs.then(|| PathBuf::from("/nix/store/pinned-rust")),
            };

        let assessment = |tools: Vec<ToolProvenance>| {
            assess(
                &contract,
                Observations::of(
                    &contract,
                    vec![Observed {
                        requirement: RequirementId::new("req-one").expect("an identifier"),
                        check: contract.checks[0].check.clone(),
                        observation: Observation::Measured(Fact::ToolchainTools {
                            manifest: ScopedRelativePath::new("flake.nix").expect("a path"),
                            shell: ShellName::new("default").expect("a name"),
                            tools,
                        }),
                    }],
                )
                .expect("matched"),
            )
        };

        // Both from the declared inputs: ready.
        assert!(assessment(vec![
            provenance("cargo", Some("/nix/store/pinned-rust/bin/cargo"), true),
            provenance("rustc", Some("/nix/store/pinned-rust/bin/rustc"), true),
        ])
        .is_ready());

        // One shadowed by a rustup install: refused, naming where it resolved.
        match assessment(vec![
            provenance("cargo", Some("/home/operator/.cargo/bin/cargo"), false),
            provenance("rustc", Some("/nix/store/pinned-rust/bin/rustc"), true),
        ]) {
            PreflightAssessment::NotReady(findings) => {
                let observed = &findings.findings()[0].observed;
                assert!(
                    observed.contains("/home/operator/.cargo/bin/cargo"),
                    "the report does not name the shadowing installation: {observed}"
                );
                assert!(
                    !observed.contains("rustc"),
                    "the report blames a tool that came from the pinned environment: {observed}"
                );
            }
            PreflightAssessment::Ready(_) => panic!("a shadowed tool passed"),
        }

        // One absent from the search path entirely: also refused, and differently worded.
        match assessment(vec![
            provenance("cargo", None, false),
            provenance("rustc", Some("/nix/store/pinned-rust/bin/rustc"), true),
        ]) {
            PreflightAssessment::NotReady(findings) => assert!(
                findings.findings()[0]
                    .observed
                    .contains("not on the search path"),
                "{}",
                findings.findings()[0].observed
            ),
            PreflightAssessment::Ready(_) => panic!("an absent tool passed"),
        }
    }

    /// A skill found in its scope passes; one absent from it does not, and the report names the
    /// scope rather than only the path.
    #[test]
    fn a_skill_is_checked_inside_the_scope_it_declares() {
        let contract = one(
            "preflight/v1 {\"kind\":\"skill\",\"scope\":\"home\",\"path\":\
             \".agents/skills/example/SKILL.md\"}",
        )
        .expect("parses");
        let path = ScopedRelativePath::new(".agents/skills/example/SKILL.md").expect("a path");
        let assessment = |fact: Fact| {
            assess(
                &contract,
                Observations::of(
                    &contract,
                    vec![Observed {
                        requirement: RequirementId::new("req-one").expect("an identifier"),
                        check: contract.checks[0].check.clone(),
                        observation: Observation::Measured(fact),
                    }],
                )
                .expect("matched"),
            )
        };

        assert!(assessment(Fact::SkillInstalled {
            scope: SkillScope::Home,
            path: path.clone(),
            bytes: 4211,
        })
        .is_ready());

        match assessment(Fact::SkillAbsent {
            scope: SkillScope::Home,
            path: path.clone(),
            problem: String::from("no file at that path"),
        }) {
            PreflightAssessment::NotReady(findings) => {
                let finding = &findings.findings()[0];
                assert!(
                    finding.expected.contains("home"),
                    "the report does not name the scope: {}",
                    finding.expected
                );
                assert!(
                    finding.remedy.contains("does not satisfy"),
                    "the remedy does not say a bundled copy is not an installed skill: {}",
                    finding.remedy
                );
            }
            PreflightAssessment::Ready(_) => panic!("an absent skill passed"),
        }
    }

    /// A tool installed but unreadable is unmet rather than unknown, and says why it is neither a
    /// mismatch nor an absence.
    #[test]
    fn a_tool_that_reports_no_readable_version_is_unmet() {
        let contract =
            one("preflight/v1 {\"kind\":\"tool\",\"program\":\"cargo\",\"version\":\"1.90.0\"}")
                .expect("parses");
        let assessment = assess(
            &contract,
            Observations::of(
                &contract,
                vec![Observed {
                    requirement: RequirementId::new("req-one").expect("an identifier"),
                    check: contract.checks[0].check.clone(),
                    observation: Observation::Measured(Fact::ToolVersionUnreadable {
                        program: ProgramName::new("cargo").expect("a name"),
                        resolved: PathBuf::from("/usr/bin/cargo"),
                        problem: String::from("two version tokens in the output"),
                    }),
                }],
            )
            .expect("matched"),
        );
        match assessment {
            PreflightAssessment::NotReady(findings) => {
                let finding = &findings.findings()[0];
                assert_eq!(finding.outcome, CheckOutcome::Unmet);
                assert!(
                    finding.observed.contains("two version tokens"),
                    "the report does not carry the probe's reason: {}",
                    finding.observed
                );
            }
            PreflightAssessment::Ready(_) => panic!("an unreadable version passed"),
        }
    }

    /// A fact of the wrong shape for its check is reported rather than assumed away.
    ///
    /// [`Observations::of`] makes this unreachable through the public path, so the arm exists for
    /// the day a check kind is added without one: it fails a run loudly instead of passing it
    /// silently, and it is not an `unreachable!` in a command that has already written nothing.
    #[test]
    fn a_fact_of_the_wrong_shape_is_reported_rather_than_panicking() {
        let contract =
            one("preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\"}").expect("parses");
        let assessment = assess(
            &contract,
            Observations::of(
                &contract,
                vec![Observed {
                    requirement: RequirementId::new("req-one").expect("an identifier"),
                    check: contract.checks[0].check.clone(),
                    observation: Observation::Measured(Fact::WorktreeClean),
                }],
            )
            .expect("matched"),
        );
        match assessment {
            PreflightAssessment::NotReady(findings) => {
                assert_eq!(findings.findings()[0].outcome, CheckOutcome::ProbeFailed);
                assert!(
                    findings.findings()[0].observed.contains("another kind"),
                    "{}",
                    findings.findings()[0].observed
                );
            }
            PreflightAssessment::Ready(_) => panic!("a worktree fact satisfied a project check"),
        }
    }

    /// The exit mapping the specs state, asserted against the codes rather than the variant names.
    ///
    /// A readiness report and a violated document share exit 1 and are still different outcomes, so
    /// this asserts both halves: the code a caller sees, and that the two are not the same value.
    #[test]
    fn a_report_exits_by_its_verdict() {
        let contract = MachineContract::of(Vec::new()).expect("a contract");
        let ready = PreflightReport {
            runbook: String::from("release"),
            version: RunbookVersion::parse("0.1.0").expect("parses"),
            source: PathBuf::from("htmlspec/runbooks/release"),
            workdir: PathBuf::from("."),
            assessment: PreflightAssessment::Ready(PassedChecks::default()),
            contract: contract.clone(),
        };
        assert_eq!(ready.outcome(), crate::exit::Outcome::Success);
        assert_eq!(ready.outcome().code(), 0);

        let refused = PreflightReport {
            assessment: PreflightAssessment::NotReady(
                NonEmptyFindings::of(vec![Finding {
                    requirement: RequirementId::new("req-one").expect("an identifier"),
                    check: MachineCheck::State {
                        check: WorkingState::OutsideProject,
                    },
                    expected: String::from("no project here"),
                    observed: String::from("a project here"),
                    outcome: CheckOutcome::Unmet,
                    remedy: String::from("run elsewhere"),
                }])
                .expect("nonempty"),
            ),
            ..ready.clone()
        };
        assert_eq!(
            refused.outcome(),
            crate::exit::Outcome::PrerequisiteFailure,
            "an unmet prerequisite did not report as one"
        );
        assert_eq!(
            refused.outcome().code(),
            1,
            "`toolchain-packaging` puts it at 1"
        );
        assert_ne!(
            refused.outcome(),
            crate::exit::Outcome::DocumentViolation,
            "a machine failure was folded into a document violation"
        );
        assert!(!refused.outcome().is_success());
        let _ = contract;
    }

    /// A report leads with readiness, names both locations, and follows document order.
    #[test]
    fn a_report_leads_with_readiness_and_names_both_locations() {
        let contract = contract_of(&[
            (
                "first",
                &["preflight/v1 {\"kind\":\"state\",\"check\":\"outside-project\"}"],
            ),
            (
                "second",
                &["preflight/v1 {\"kind\":\"tool\",\"program\":\"cargo\",\"version\":\"1.90.0\"}"],
            ),
        ])
        .expect("parses");
        let report = PreflightReport {
            runbook: String::from("release"),
            version: RunbookVersion::parse("0.1.0").expect("parses"),
            source: PathBuf::from("/store/htmlspec/runbooks/release"),
            workdir: PathBuf::from("/work/target"),
            assessment: assess(
                &contract,
                Observations::of(
                    &contract,
                    vec![
                        Observed {
                            requirement: RequirementId::new("req-first").expect("an identifier"),
                            check: contract.checks[0].check.clone(),
                            observation: Observation::Measured(Fact::NoNearestProject),
                        },
                        Observed {
                            requirement: RequirementId::new("req-second").expect("an identifier"),
                            check: contract.checks[1].check.clone(),
                            observation: Observation::Measured(Fact::ToolAbsent {
                                program: ProgramName::new("cargo").expect("a name"),
                            }),
                        },
                    ],
                )
                .expect("matched"),
            ),
            contract: contract.clone(),
        };

        let human = report.render_human();
        assert!(
            human.starts_with("not ready:"),
            "the report does not lead with readiness:\n{human}"
        );
        // Both locations, because `--store` makes them different places and a worktree check
        // judged one of them.
        assert!(
            human.contains("/store/htmlspec/runbooks/release"),
            "the report does not name where the documents were read:\n{human}"
        );
        assert!(
            human.contains("/work/target"),
            "the report does not name where the checks were evaluated:\n{human}"
        );
        assert!(
            human.contains("0.1.0"),
            "the report does not name the carried version:\n{human}"
        );
        assert!(
            human.find("req-first") < human.find("req-second"),
            "the report does not follow document order:\n{human}"
        );
        assert!(
            human.contains("no run was started"),
            "the report does not say what was and was not done:\n{human}"
        );

        // The structured payload holds every declared check, including the one that passed, so an
        // agent can tell a pass from a check nothing reached.
        let payload = report.payload();
        assert_eq!(payload["ready"], serde_json::Value::Bool(false));
        assert_eq!(payload["declared"], serde_json::json!(2));
        let checks = payload["checks"].as_array().expect("an array");
        assert_eq!(checks.len(), 2, "{payload}");
        assert_eq!(checks[0]["requirement"], serde_json::json!("req-first"));
        assert_eq!(checks[0]["outcome"], serde_json::json!("passed"));
        assert_eq!(checks[1]["outcome"], serde_json::json!("unmet"));
        assert!(
            checks[1]["remedy"]
                .as_str()
                .is_some_and(|remedy| !remedy.is_empty()),
            "a refusal carries no remedy: {payload}"
        );
        assert_eq!(
            payload["source"],
            serde_json::json!("/store/htmlspec/runbooks/release"),
            "{payload}"
        );
    }

    /// The ready report names what passed, with the observation that shows it.
    #[test]
    fn a_ready_report_carries_the_evidence_it_will_copy_into_a_record() {
        let contract =
            one("preflight/v1 {\"kind\":\"tool\",\"program\":\"python3\",\"version\":\"3.13.2\"}")
                .expect("parses");
        let report = PreflightReport {
            runbook: String::from("release"),
            version: RunbookVersion::parse("0.1.0").expect("parses"),
            source: PathBuf::from("htmlspec/runbooks/release"),
            workdir: PathBuf::from("."),
            assessment: assess(
                &contract,
                Observations::of(
                    &contract,
                    vec![Observed {
                        requirement: RequirementId::new("req-one").expect("an identifier"),
                        check: contract.checks[0].check.clone(),
                        observation: Observation::Measured(Fact::ToolVersion {
                            program: ProgramName::new("python3").expect("a name"),
                            resolved: PathBuf::from("/usr/bin/python3"),
                            version: ToolVersion::parse("3.13.2").expect("parses"),
                        }),
                    }],
                )
                .expect("matched"),
            ),
            contract,
        };
        let human = report.render_human();
        assert!(human.starts_with("ready:"), "{human}");
        assert!(
            human.contains("/usr/bin/python3") && human.contains("3.13.2"),
            "the report does not carry the observation a record would copy:\n{human}"
        );
        let payload = report.payload();
        assert_eq!(payload["ready"], serde_json::Value::Bool(true));
        assert_eq!(
            payload["checks"][0]["outcome"],
            serde_json::json!("passed"),
            "{payload}"
        );
    }

    /// A report of `observation` against a runbook declaring a remedied tool, a remedied state that
    /// holds, and a state with no remedy that holds.
    fn remedied_report(observation: Observation) -> PreflightReport {
        let contract = contract_of(&[
            (
                "tool",
                &["preflight/v2 {\"kind\":\"tool\",\"program\":\"python3\",\"version\":\"3.13.2\",\
                   \"remedy\":\"brew install python@3.13\"}"],
            ),
            (
                "outside",
                &["preflight/v2 {\"kind\":\"state\",\"check\":\"outside-project\",\
                   \"remedy\":\"cd somewhere else first\"}"],
            ),
            (
                "clean",
                &["preflight/v1 {\"kind\":\"state\",\"check\":\"clean-git-worktree\"}"],
            ),
        ])
        .expect("parses");
        let observed = |id: &str, observation: Observation| Observed {
            requirement: RequirementId::new(id).expect("an identifier"),
            check: contract
                .checks()
                .iter()
                .find(|declared| declared.requirement.as_str() == id)
                .expect("declared")
                .check
                .clone(),
            observation,
        };
        let observations = Observations::of(
            &contract,
            vec![
                observed("req-tool", observation),
                observed("req-outside", Observation::Measured(Fact::NoNearestProject)),
                observed("req-clean", Observation::Measured(Fact::WorktreeClean)),
            ],
        )
        .expect("matched");
        PreflightReport {
            runbook: String::from("release"),
            version: RunbookVersion::parse("0.2.0").expect("parses"),
            source: PathBuf::from("htmlspec/runbooks/release"),
            workdir: PathBuf::from("."),
            assessment: assess(&contract, observations),
            contract,
        }
    }

    /// Each way a check can fail to pass shows the runbook's remedy after the program's own,
    /// labelled as the runbook's.
    #[test]
    fn a_check_that_did_not_pass_shows_its_runbooks_remedy() {
        for (observation, outcome) in [
            (
                Observation::Measured(Fact::ToolAbsent {
                    program: ProgramName::new("python3").expect("a name"),
                }),
                "unmet",
            ),
            (
                Observation::Unavailable {
                    reason: String::from("the search path could not be read"),
                    remedy: String::from("set PATH"),
                },
                "unavailable",
            ),
            (
                Observation::TimedOut {
                    bound: Duration::from_secs(10),
                },
                "timed-out",
            ),
            (
                Observation::ProbeFailure {
                    reason: String::from("the probe could not start"),
                },
                "probe-failed",
            ),
        ] {
            let human = remedied_report(observation).render_human();
            assert!(human.starts_with("not ready:"), "{human}");
            let program = human
                .find("      remedy: install `python3`")
                .unwrap_or_else(|| panic!("{outcome}: the program's remedy is missing:\n{human}"));
            let runbook = human
                .find("      the runbook says: brew install python@3.13\n")
                .unwrap_or_else(|| panic!("{outcome}: the runbook's remedy is missing:\n{human}"));
            assert!(
                program < runbook,
                "{outcome}: the runbook's remedy comes before the program's:\n{human}"
            );
            assert!(
                human.contains(&format!("[{outcome}]")),
                "{outcome}: the outcome is not the one tested:\n{human}"
            );
            assert!(
                !human.contains("cd somewhere else first"),
                "{outcome}: a passed check's remedy was shown:\n{human}"
            );
        }
    }

    /// The payload carries a declared remedy on every check, passed or not, and none where nothing
    /// was declared. The human report of a ready machine shows none.
    #[test]
    fn the_payload_carries_every_declared_remedy_and_a_ready_report_shows_none() {
        let refused = remedied_report(Observation::Measured(Fact::ToolAbsent {
            program: ProgramName::new("python3").expect("a name"),
        }))
        .payload();
        let checks = refused["checks"].as_array().expect("checks");
        assert_eq!(checks.len(), 3, "{refused}");
        assert_eq!(checks[0]["outcome"], serde_json::json!("unmet"));
        assert_eq!(
            checks[0]["runbookRemedy"],
            serde_json::json!("brew install python@3.13")
        );
        assert_eq!(checks[1]["outcome"], serde_json::json!("passed"));
        assert_eq!(
            checks[1]["runbookRemedy"],
            serde_json::json!("cd somewhere else first"),
            "a passed check's declared remedy is missing from the payload: {refused}"
        );
        assert!(
            checks[2].get("runbookRemedy").is_none(),
            "a check declaring no remedy carries one: {refused}"
        );

        let ready = remedied_report(Observation::Measured(Fact::ToolVersion {
            program: ProgramName::new("python3").expect("a name"),
            resolved: PathBuf::from("/usr/bin/python3"),
            version: ToolVersion::parse("3.13.2").expect("parses"),
        }));
        let human = ready.render_human();
        assert!(human.starts_with("ready:"), "{human}");
        assert!(
            !human.contains("the runbook says"),
            "a ready report showed a remedy:\n{human}"
        );
        let payload = ready.payload();
        assert_eq!(
            payload["checks"][0]["runbookRemedy"],
            serde_json::json!("brew install python@3.13"),
            "{payload}"
        );
        assert!(
            payload["checks"][2].get("runbookRemedy").is_none(),
            "{payload}"
        );
    }
}
