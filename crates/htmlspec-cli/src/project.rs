//! The project a command operates on.
//!
//! Every command until now took a path and worked on whatever was there. That is
//! right for a document and wrong for a project: once commands find their project by
//! walking upward, "the path the contributor typed" and "the project that path
//! belongs to" are different things with the same representation, and a signature
//! taking `&Path` cannot tell them apart.
//!
//! So a [`ProjectRoot`] is a directory *known* to hold an `htmlspec/` directory, and
//! the only ways to get one are [`ProjectRoot::resolve`], which walked upward and
//! found it, and [`ProjectRoot::establish`], which just created it. The inner path is
//! private, so no caller can widen a bare path into a project without checking.
//!
//! The one place this matters most is the asset store. `assets()` derives the store's
//! location from the *project*, and that is the whole fix for a defect measured at 6
//! MB per change: the previous code derived it from each document's own directory, so
//! two changes got two copies of the same vendored runtime, and the command that
//! exists to report duplicate assets could not see them because it never descended.
//! Deriving the location here, and only here, makes one store per project the only
//! reachable outcome rather than a default a contributor has to remember to ask for.

use std::path::{Path, PathBuf};

/// The directory that marks a project, relative to its root.
pub const DIRECTORY: &str = "htmlspec";

/// Where a project keeps its accepted specifications.
const SPECS: &str = "specs";

/// Where a project keeps its active changes.
const CHANGES: &str = "changes";

/// Where a project keeps its archived changes, beneath [`CHANGES`].
const ARCHIVE: &str = "archive";

/// The file name a change's specification artifact takes.
///
/// One per change, and it is also that change's as-built record — the same file, not
/// two. A change directory holding two specifications would leave a reader deciding
/// which one states the contract. Here rather than with the commands that write it,
/// because where a document sits is decided by this module, and a change's specification
/// is a place a reference can reach.
pub const CHANGE_SPECIFICATION: &str = "specification.html";

/// Where a project keeps its runbooks.
///
/// The fourth place beneath the project root, beside [`SPECS`], [`CHANGES`], and [`ARCHIVE`].
/// A runbook holds a procedure that is run repeatedly, so it cannot live under [`CHANGES`]:
/// archival ends a change, and a procedure ended after its first run is a procedure destroyed.
const RUNBOOKS: &str = "runbooks";

/// Where one runbook keeps the records of its runs, beneath its own directory.
///
/// Not a fifth place beside [`SPECS`], [`CHANGES`], and [`RUNBOOKS`]: this is per-runbook, so
/// there is no location for it until a runbook exists. That is why [`ProjectRoot::establish`] does
/// not create it and [`ProjectRoot::runs`] takes a runbook's name.
const RUNS: &str = "runs";

/// Where one change keeps its evidence, directly within its own directory.
///
/// Per-change like [`RUNS`] is per-runbook, and for the same reason not a place
/// [`ProjectRoot::establish`] creates. artifact-kinds' "A change's evidence is kept in the change and
/// is not one of its documents" states the location and what follows from it: every file beneath it
/// is a record, not a document of the project.
const EVIDENCE: &str = "evidence";

/// The file a project declares its name, version, and expected toolchain in.
pub const DECLARATION: &str = "spec.nix";

/// The file an archived change records the version it shipped in.
///
/// `toolchain-packaging` requires each archived change to record its version *in a form the
/// project's own suite can read without version control history*, and `scripts/check-release`
/// already reads this file. The archival version gate reads the same one, so the check that
/// reports a missing version and the gate that compares versions cannot disagree about where
/// a version lives.
pub const RECORDED_VERSION: &str = "VERSION";

/// The file a change states the change it follows in.
///
/// A whole-directory fact about a change, stored beside its documents rather than inside one of
/// them. `artifact-kinds` requires that placement: a reference addresses a *unit* and a change
/// is a directory, so a `data-refs` attribute naming another change resolves to nothing and
/// `refs/unresolved` reports it as an error.
///
/// [`RECORDED_VERSION`] is the precedent and it is nearly exact — a version is also a fact
/// about a change as a whole, also kept in a file beside the documents, also readable without
/// version control history. A second configuration language for one string is what
/// `project-bootstrap` argues against.
pub const FOLLOWS: &str = "FOLLOWS";

/// Why a string could not be used as a change's name.
///
/// Structured rather than a bare `bool`, for the reason [`NotAProject`] is: the report has to name
/// what was given, and a contributor who typed a path meant something the operation can name back
/// to them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotAName {
    /// The string as given.
    pub given: String,
}

impl std::fmt::Display for NotAName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} is a path rather than a change name. A change is named, and the project decides \
             where that name lives — so a name carrying `/` or `..` would put the change \
             somewhere the project does not keep changes. Pass the name alone",
            self.given
        )
    }
}

impl std::error::Error for NotAName {}

/// A directory known to hold a project.
///
/// Construction is the proof: a value of this type exists only where an `htmlspec/`
/// directory was found or created, so a function taking one does not re-check and
/// cannot be handed a directory that is not a project.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRoot(PathBuf);

/// Why no project could be resolved.
///
/// Structured rather than a bare `Option`, because the report a contributor needs
/// names where the search started and what it was looking for. "No project found" on
/// its own leaves them guessing which directory the tool was even standing in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotAProject {
    /// The directory the search started from.
    pub from: PathBuf,
}

impl std::fmt::Display for NotAProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "no {DIRECTORY}/ directory in {} or any directory above it, so there is no \
             project here. Run `htmlspec init` to establish one",
            self.from.display()
        )
    }
}

impl std::error::Error for NotAProject {}

impl ProjectRoot {
    /// Finds the project containing `from`.
    ///
    /// Ascends looking for a child `htmlspec/` directory, and stops at a filesystem
    /// root or at a `.git` directory — whichever comes first.
    ///
    /// **Stopping at `.git` is the part worth stating.** Without it, a repository with
    /// no project of its own, nested inside a directory that happens to hold one,
    /// would resolve outward past its own boundary and write into a project it is not
    /// part of. A repository is the outermost thing a command should reach, and `.git`
    /// is the marker for one.
    pub fn resolve(from: &Path) -> Result<Self, NotAProject> {
        let start = absolute(from);
        let mut directory = start.as_path();

        loop {
            if directory.join(DIRECTORY).is_dir() {
                return Ok(Self(directory.to_owned()));
            }
            // Checked *after* the project test, so a repository root that is also a
            // project resolves to itself rather than being refused at its own boundary.
            if directory.join(".git").exists() {
                break;
            }
            match directory.parent() {
                Some(parent) => directory = parent,
                None => break,
            }
        }

        Err(NotAProject { from: start })
    }

    /// Establishes a project at `root`, creating the directories it is made of.
    ///
    /// The counterpart to [`Self::resolve`]: this is the one path that produces a
    /// `ProjectRoot` for a directory that was not a project a moment ago.
    pub fn establish(root: &Path) -> std::io::Result<Self> {
        let root = absolute(root);
        let project = Self(root);
        // `changes()` is absent because `archive()` is beneath it, so `create_dir_all` makes
        // both. `runbooks()` has nothing beneath it and is therefore named.
        //
        // Created at bootstrap rather than on a project's first runbook, because
        // `project-bootstrap` requires the operation to produce the project's places: a place
        // that does not exist until something is in it is not one a contributor can find.
        for directory in [
            project.htmlspec(),
            project.specs(),
            project.archive(),
            project.runbooks(),
        ] {
            std::fs::create_dir_all(&directory)?;
        }
        Ok(project)
    }

    /// Whether a project already exists at `root`, without creating one.
    ///
    /// This is the extend-mode question: a bootstrap that finds a project refreshes
    /// it rather than establishing it, and the two differ in what they are allowed to
    /// touch.
    pub fn exists(root: &Path) -> bool {
        root.join(DIRECTORY).is_dir()
    }

    /// The directory holding the project, which is what a contributor sees as "the
    /// repository".
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// The project's own directory.
    pub fn htmlspec(&self) -> PathBuf {
        self.0.join(DIRECTORY)
    }

    /// Where the project's accepted specifications live.
    pub fn specs(&self) -> PathBuf {
        self.htmlspec().join(SPECS)
    }

    /// Where the project's active changes live.
    pub fn changes(&self) -> PathBuf {
        self.htmlspec().join(CHANGES)
    }

    /// Where the project's archived changes live.
    pub fn archive(&self) -> PathBuf {
        self.changes().join(ARCHIVE)
    }

    /// Where the project's runbooks live.
    pub fn runbooks(&self) -> PathBuf {
        self.htmlspec().join(RUNBOOKS)
    }

    /// The directory a runbook of this name occupies.
    ///
    /// Unconditional for the same reason [`Self::change`] is: a runbook goes where the project
    /// keeps runbooks, and a root that does not determine that is not doing the job the root
    /// exists for.
    pub fn runbook(&self, name: &str) -> PathBuf {
        self.runbooks().join(name)
    }

    /// Where the project records the first release subject to runbook publication.
    ///
    /// At the planning root rather than inside an archive, because the question it answers is about
    /// the history as a whole — "from which release does a missing inventory mean something is
    /// wrong?" — and an answer that lived inside one archive would move whenever that archive did.
    ///
    /// [`RECORDED_VERSION`] is the precedent for a whole-directory fact in a file beside the
    /// documents, and [`FOLLOWS`] for one at a planning root.
    pub fn runbook_releases(&self) -> PathBuf {
        self.htmlspec().join(crate::runbook_release::ADOPTION_FILE)
    }

    /// Where the records of one runbook's runs live.
    ///
    /// Derived from [`Self::runbook`] rather than composed from a path a caller holds, which is
    /// what keeps the layout a fact this module states. The measured cost of the alternative is
    /// recorded on [`Self::runbook`]: a location read from a directory entry is a second statement
    /// of where a thing lives, and the two agree until the day the layout moves.
    ///
    /// **Not created by [`Self::establish`].** The project's four places are created at bootstrap
    /// because a place that does not exist until something is in it is not one a contributor can
    /// find — and that argument is about the places a contributor looks for, of which there are
    /// four. This one is per-runbook and cannot exist before its runbook does, so the operation
    /// that writes the first record creates it.
    pub fn runs(&self, runbook: &str) -> PathBuf {
        self.runbook(runbook).join(RUNS)
    }

    /// Whether `name` is a name at all, rather than a path wearing one.
    ///
    /// **A name carrying a path separator is not a name, and this had to be re-litigated at three
    /// separate call sites before it was stated once here.** `AGENTS.md` §2 names that pattern
    /// exactly: an invariant callers must remember is a candidate for enforcement by a
    /// constructor.
    ///
    /// The measured cost of not having it: `htmlspec new change ../runbooks/sneaky` composed
    /// `htmlspec/changes/../runbooks/sneaky` and created a change directory *inside the runbooks
    /// directory*. It then listed as a runbook, was invisible to `htmlspec list`, and every
    /// change-taking operation refused it — a directory nothing could act on. Worse, before the
    /// runbook guard existed the same typo produced a directory that was merely odd and still
    /// archivable, so the guard converted a recoverable mistake into an unrecoverable one.
    ///
    /// Rejected rather than sanitized. Silently rewriting `../runbooks/sneaky` to `sneaky` would
    /// create a change under a name the contributor did not type, and `artifact-kinds` requires a
    /// derived name to be reported so a contributor who disagrees can say so — a name quietly
    /// corrected is a decision taken on their behalf.
    ///
    /// Both separators are rejected on every platform. A backslash is a separator on Windows and
    /// a legal filename character elsewhere, so accepting it here would make the same name mean
    /// two different things depending on where the command ran.
    pub fn is_a_name(name: &str) -> bool {
        !name.is_empty()
            && !name.contains('/')
            && !name.contains('\\')
            && name != "."
            && name != ".."
    }

    /// The directory a change of this name occupies, refusing a name that is a path.
    ///
    /// The checked counterpart to [`Self::change`], for the operations that take a name from a
    /// contributor. [`Self::change`] stays infallible because most of its callers compose from a
    /// name the program itself produced — a directory entry it just read — and a `Result` there
    /// would be a error case that cannot happen.
    pub fn named_change(&self, name: &str) -> Result<PathBuf, NotAName> {
        if Self::is_a_name(name) {
            Ok(self.change(name))
        } else {
            Err(NotAName {
                given: name.to_owned(),
            })
        }
    }

    /// The project's one asset store.
    ///
    /// Derived from the project rather than from any document, which is the property
    /// that makes a single shared store the only thing that can happen. See the module
    /// comment for the defect this closes.
    pub fn assets(&self) -> PathBuf {
        self.htmlspec().join(htmlspec_assets::store::DIRECTORY)
    }

    /// The project's declaration file.
    pub fn declaration(&self) -> PathBuf {
        self.0.join(DECLARATION)
    }

    /// The directory a change of this name occupies.
    ///
    /// Unconditional: a change goes where the project keeps changes. Accepting a path
    /// here would let a change be created outside `changes/`, and a root that does not
    /// determine where changes live is not doing the job the root exists for.
    pub fn change(&self, name: &str) -> PathBuf {
        self.changes().join(name)
    }

    /// Whether `directory` is a change's evidence directory: named [`EVIDENCE`], and directly
    /// within an active change or an archived one.
    ///
    /// **Positional, not by name.** A directory called `evidence` anywhere else is not a change's
    /// evidence, and the case that makes this matter is a capability whose path is `evidence`: a
    /// name test would stop `htmlspec/specs/evidence/spec.html` being validated, with nothing to
    /// say so. `changes/archive/evidence` is not one either, because the archive is not a change.
    /// And `changes/archive` is always the archive, never an active change named "archive".
    ///
    /// The name is read from `directory` as given and only its parent is canonicalized, so the
    /// answer is about where the entry sits rather than where a symbolic link called `evidence`
    /// points. The parent is compared on canonical paths, because a walk composes its paths from
    /// whatever root it was handed — `.`, `htmlspec`, or a path through a link — and a lexical
    /// comparison would miss the exclusion whenever the spellings differ.
    pub fn is_change_evidence(&self, directory: &Path) -> bool {
        if directory.file_name() != Some(std::ffi::OsStr::new(EVIDENCE)) {
            return false;
        }
        let Some(change) = directory.parent().map(canonical) else {
            return false;
        };
        let Some(holder) = change.parent() else {
            return false;
        };
        let archive = canonical(&self.archive());
        (holder == canonical(&self.changes()) && change != archive) || holder == archive
    }

    /// Whether `directory` is an archived change: one directly within the project's archive.
    ///
    /// Positional and on canonical paths, as [`Self::is_change_evidence`] is, so a path spelled
    /// through `..` or reached through a linked project is judged by where it sits. A directory is
    /// an archived change where it sits directly in the archive, or where it leads to one that
    /// does: a symbolic link in the archive is an entry of the archive whatever it points at, and a
    /// link elsewhere that points at an archived change still names one. The archive itself is not
    /// a change, and neither is a directory inside an archived change.
    pub fn is_archived_change(&self, directory: &Path) -> bool {
        let archive = canonical(&self.archive());
        let target = canonical(directory);
        if target == archive {
            return false;
        }
        let sits_in_archive = directory.file_name().is_some()
            && directory.parent().map(canonical).as_deref() == Some(archive.as_path());
        let leads_into_archive = target.parent() == Some(archive.as_path());
        sits_in_archive || leads_into_archive
    }

    /// Whether the document at `path` is history: in an archived change, or a run's record.
    ///
    /// `format --fallback` leaves history as written. An archived change renders with what it was
    /// reviewed against, and a run's record is runbook-execution's "A run's record is history rather
    /// than contract". Positional: anything beneath the archive, and a document directly within a
    /// runbook's `runs` directory.
    ///
    /// Judged on the path as spelled as well as resolved, as [`Self::is_archived_change`] judges a
    /// directory. Resolved alone, a link in the archive that points elsewhere, or a `runs` directory
    /// that is a link, would be judged by its target, and `format --fallback` would rewrite an
    /// archived change or a run's record through it. Each ancestor as spelled is placed by its
    /// canonical parent, so a link anywhere on the way is an entry of whatever directory holds it.
    pub fn is_history(&self, path: &Path) -> bool {
        let archive = canonical(&self.archive());
        let runbooks = canonical(&self.runbooks());
        let runs = Some(std::ffi::OsStr::new(RUNS));

        let spelled = directory_of(path);
        let resolved = canonical(&spelled);
        if resolved.starts_with(&archive) {
            return true;
        }
        if resolved.file_name() == runs
            && resolved.parent().and_then(Path::parent) == Some(runbooks.as_path())
        {
            return true;
        }

        let sits_in_archive = spelled.ancestors().any(|ancestor| {
            ancestor.file_name().is_some()
                && ancestor.parent().map(canonical).as_deref() == Some(archive.as_path())
        });
        sits_in_archive
            || (spelled.file_name() == runs
                && spelled
                    .parent()
                    .and_then(Path::parent)
                    .map(canonical)
                    .as_deref()
                    == Some(runbooks.as_path()))
    }

    /// Where the document at `path` sits, as far as its references are concerned.
    ///
    /// document-schema's "Cross-references address units by identifier" decides what a reference
    /// reaches by where a document sits, and this is that decision's one derivation. Positional and
    /// on canonical paths, as [`Self::is_change_evidence`] is, so a path spelled through `..` or
    /// through a symbolic link is placed as its target is. A document that is not written yet is
    /// placed by its directory and its file name, because `run` validates a record before it writes
    /// one.
    ///
    /// A change's evidence is loose: artifact-kinds says it is not one of the change's documents.
    /// So is the archive's own directory, a subdirectory of a change, and anything else beneath
    /// `htmlspec/` that none of the project's places names.
    pub fn placement(&self, path: &Path) -> htmlspec_validate::Placement {
        use htmlspec_validate::Placement;
        if self.is_within_change_evidence(path) {
            return Placement::Loose;
        }
        let Some(name) = path.file_name() else {
            return Placement::Loose;
        };
        let parent = canonical(&directory_of(path));
        if parent.starts_with(canonical(&self.specs())) {
            return Placement::AcceptedSpecification;
        }
        if parent.starts_with(canonical(&self.runbooks())) {
            return Placement::RunbookDocument;
        }
        let archive = canonical(&self.archive());
        let holder = parent.parent();
        let in_a_change = parent != archive
            && (holder == Some(canonical(&self.changes()).as_path())
                || holder == Some(archive.as_path()));
        if !in_a_change {
            return Placement::Loose;
        }
        if name == CHANGE_SPECIFICATION {
            Placement::ChangeSpecification
        } else {
            Placement::ChangeDocument
        }
    }

    /// Whether `path` is a change's evidence directory or lies anywhere beneath one.
    ///
    /// The question a writer asks before touching a file, where [`Self::is_change_evidence`] is
    /// the one a walk asks of each directory it meets. artifact-kinds states that no operation
    /// rewrites a file beneath a change's evidence, including one a contributor names, so this
    /// answers for any spelling of the path. It is canonicalized first: from inside a change,
    /// `evidence/a.html` has no ancestor that names the change, and `evidence/../design.html`
    /// names `evidence` without lying beneath it.
    pub fn is_within_change_evidence(&self, path: &Path) -> bool {
        canonical(path)
            .ancestors()
            .any(|ancestor| ancestor.is_dir() && self.is_change_evidence(ancestor))
    }
}

/// Absolutizes a path so the upward walk terminates.
///
/// `parent()` on a relative path stops at the empty path, which would end the search
/// before it reached the directories above the current one — the bug that makes a
/// command work from the project root and fail one directory down.
fn absolute(path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_owned()
    } else {
        match std::env::current_dir() {
            Ok(current) => current.join(path),
            Err(_) => path.to_owned(),
        }
    };
    // Collecting the components drops `.` segments, so a report reads
    // `/tmp/project/htmlspec` rather than `/tmp/project/./htmlspec` when the search
    // started from the current directory. Every path this type produces is derived from
    // the stored root, so normalizing once here fixes all of them.
    joined.components().collect()
}

/// The directory the document at `path` sits in: its parent, or the current directory where `path`
/// is a bare file name.
///
/// `Path::parent` of a bare file name is the empty path, which names no directory and resolves to
/// nothing. The placement, the store walk, and the scope each ask where a document sits, so they ask
/// it here. The code review of PR #43 found the placement asking it differently from the scope, so
/// `validate tasks.html` inside a change was placed loose and `validate ./tasks.html` was not.
pub(crate) fn directory_of(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_owned(),
        _ => PathBuf::from("."),
    }
}

/// A path with its symbolic links resolved, or made absolute where it cannot be resolved.
///
/// The one rule for comparing a discovered path with a location this module derives, used by both
/// positional questions the program asks: whether a store is the project's
/// ([`crate::topology::partition`]) and whether a directory is a change's evidence
/// ([`ProjectRoot::is_change_evidence`]). A path that cannot be resolved is one that vanished or
/// cannot be read, and comparing its absolute form is the best evidence left.
pub(crate) fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path)
        .or_else(|_| std::path::absolute(path))
        .unwrap_or_else(|_| path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary directory that removes itself.
    ///
    /// Hand-rolled to match `discover.rs`, which does the same for the same reason: a
    /// `tempfile` dependency for one directory in one test module is more surface than
    /// the lines it replaces.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-project-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            // Canonicalized because the walk absolutizes, and on macOS the temporary
            // directory is a symlink — so an uncanonicalized expectation compares
            // `/var/...` against `/private/var/...` and fails for the wrong reason.
            let path = path.canonicalize().expect("the scratch path resolves");
            Self(path)
        }

        fn directory(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            std::fs::create_dir_all(&path).expect("the directory is created");
            path
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn establishing_a_project_creates_the_directories_it_is_made_of() {
        let scratch = Scratch::new("establish");
        let project = ProjectRoot::establish(&scratch.0).expect("the project is established");

        assert!(project.htmlspec().is_dir());
        assert!(project.specs().is_dir());
        assert!(project.changes().is_dir());
        assert!(project.archive().is_dir());
        // The fourth place. `project-bootstrap` requires bootstrapping to produce the places
        // rather than to acquire them on first use, so an empty `runbooks/` is the correct
        // outcome and this asserts it rather than asserting a runbook exists.
        assert!(
            project.runbooks().is_dir(),
            "a bootstrapped project has no place for a runbook, so a contributor cannot find \
             where one goes"
        );
    }

    /// A name carrying a path separator is refused before anything is composed from it.
    ///
    /// **The measured wedge this closes.** `htmlspec new change ../runbooks/sneaky` composed
    /// `htmlspec/changes/../runbooks/sneaky` and created a change directory inside the runbooks
    /// directory: it then listed as a runbook, was invisible to `htmlspec list`, and every
    /// change-taking operation refused it. Verified against `88759b6^` that the same typo was
    /// merely odd and still archivable before the runbook guard existed, so the guard is what made
    /// it unrecoverable.
    #[test]
    fn a_change_name_carrying_a_path_is_not_a_name() {
        let scratch = Scratch::new("named-change");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        assert_eq!(
            project
                .named_change("add-dark-mode")
                .expect("a name is a name"),
            project.change("add-dark-mode")
        );

        // The traversal that produced the wedge, and the degenerate spellings beside it. `\\` is
        // included because it is a separator on Windows and a legal filename character elsewhere,
        // so accepting it would make one name mean two things depending on where the command ran.
        for given in [
            "../runbooks/sneaky",
            "runbooks/sneaky",
            "..",
            ".",
            "",
            "a\\b",
            "/absolute",
        ] {
            let error = project
                .named_change(given)
                .expect_err("a path was accepted as a change name");
            assert_eq!(error.given, given);
            assert!(
                error
                    .to_string()
                    .contains("is a path rather than a change name"),
                "the report does not name the problem: {error}"
            );
        }
    }

    /// The layout is derived from the root, so a runbook's path is not a caller's to compose.
    ///
    /// Asserted against the *shape* rather than against a spelled-out string, because the
    /// constants are private on purpose: a test hard-coding `htmlspec/runbooks` would state the
    /// layout a second time, which is the shape this module exists to prevent.
    #[test]
    fn a_runbook_lives_beneath_the_projects_runbooks() {
        let scratch = Scratch::new("runbook-path");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        assert_eq!(
            project.runbook("release"),
            project.runbooks().join("release")
        );
        assert!(project.runbooks().starts_with(project.htmlspec()));
        // Beside the changes rather than beneath them, which is the whole point of a fourth
        // place: archival ends a change, and a procedure is run again.
        assert!(!project.runbooks().starts_with(project.changes()));
    }

    /// A runbook's records live beneath that runbook, which is what makes the archival guard
    /// cover them for free.
    ///
    /// Asserted against the shape rather than against `htmlspec/runbooks/release/runs`, for the
    /// reason the test above is: a spelled-out expectation would state the layout a second time.
    ///
    /// **The `starts_with` assertion is the one that matters.** `refuse_a_runbook` compares a
    /// canonicalized argument against a canonicalized `runbooks()` with exactly that operation, so
    /// this is the property that makes a run's record unarchivable — stated here as a fact about
    /// the layout rather than left implicit in the guard. Measured end to end on the real binary
    /// too, in `evidence/the-guard-covers-runs-2026-09-18.md`, because a path relationship and a
    /// refusal are two claims.
    #[test]
    fn a_runs_directory_lives_beneath_its_runbook() {
        let scratch = Scratch::new("runs-path");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        assert_eq!(
            project.runs("release"),
            project.runbook("release").join("runs")
        );
        assert!(project
            .runs("release")
            .starts_with(project.runbook("release")));
        assert!(project.runs("release").starts_with(project.runbooks()));
        assert!(!project.runs("release").starts_with(project.changes()));
    }

    /// Bootstrapping creates the project's four places and no `runs/` anywhere.
    ///
    /// The negative half of the decision recorded on [`ProjectRoot::runs`]. Without this, removing
    /// the reasoning and creating the directory eagerly would pass every other test in this module:
    /// nothing else asks whether it is absent.
    #[test]
    fn bootstrapping_creates_no_place_for_a_runs_record() {
        let scratch = Scratch::new("runs-absent");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        assert!(project.runbooks().is_dir());
        let entries: Vec<String> = std::fs::read_dir(project.runbooks())
            .expect("the runbooks directory is readable")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            entries.is_empty(),
            "bootstrapping put {entries:?} under the runbooks directory; a place for a run's \
             records cannot exist before the runbook it belongs to"
        );
    }

    #[test]
    fn a_project_resolves_from_a_directory_nested_inside_it() {
        let scratch = Scratch::new("resolve-nested");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        let deep = scratch.directory("htmlspec/changes/add-dark-mode/evidence");

        let resolved = ProjectRoot::resolve(&deep).expect("the project is found");
        assert_eq!(resolved, project);
    }

    /// The boundary the `.git` stop exists for.
    ///
    /// A repository with no project of its own, nested inside a directory that has
    /// one. Without the stop this resolves outward and a command writes into a
    /// project the contributor is not working in.
    #[test]
    fn the_search_stops_at_a_repository_boundary() {
        let scratch = Scratch::new("resolve-boundary");
        ProjectRoot::establish(&scratch.0).expect("the outer project exists");
        let inner = scratch.directory("vendor/nested-repository");
        std::fs::create_dir_all(inner.join(".git")).expect("the inner repository exists");

        let error = ProjectRoot::resolve(&inner).expect_err("the outer project is not reached");
        assert_eq!(error.from, inner);
        assert!(
            error.to_string().contains("htmlspec init"),
            "the report does not name the operation that creates a project: {error}"
        );
    }

    /// A repository that *is* a project resolves to itself. The `.git` test runs after
    /// the project test for exactly this case.
    #[test]
    fn a_repository_that_is_a_project_resolves_to_itself() {
        let scratch = Scratch::new("resolve-self");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        std::fs::create_dir_all(scratch.0.join(".git")).expect("the repository exists");

        assert_eq!(
            ProjectRoot::resolve(&scratch.0).expect("resolved"),
            project,
            "a repository root holding a project must resolve to itself"
        );
    }

    #[test]
    fn no_project_is_reported_with_the_directory_searched_from() {
        let scratch = Scratch::new("resolve-absent");
        let inside = scratch.directory("somewhere/below");
        std::fs::create_dir_all(scratch.0.join(".git")).expect("the repository exists");

        let error = ProjectRoot::resolve(&inside).expect_err("no project");
        assert_eq!(error.from, inside);
        assert!(error.to_string().contains(&inside.display().to_string()));
    }

    /// The property that makes one store per project true by construction: the store's
    /// location depends on the project, never on where the command was invoked.
    #[test]
    fn the_store_location_does_not_depend_on_where_resolution_started() {
        let scratch = Scratch::new("assets-stable");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        let expected = project.assets();

        for relative in [
            "htmlspec",
            "htmlspec/specs",
            "htmlspec/changes",
            "htmlspec/changes/one/evidence",
            "htmlspec/changes/another",
        ] {
            let from = scratch.directory(relative);
            let resolved = ProjectRoot::resolve(&from).expect("the project is found");
            assert_eq!(
                resolved.assets(),
                expected,
                "the store moved when resolution started at {relative}"
            );
        }
    }

    /// Two changes in one project name one store. This is the measured defect stated
    /// as an assertion: the previous code derived the store from each document's own
    /// directory, which gave each change its own 6 MB copy.
    #[test]
    fn every_change_in_a_project_shares_one_store() {
        let scratch = Scratch::new("assets-shared");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        let one = ProjectRoot::resolve(&scratch.directory("htmlspec/changes/one"))
            .expect("the project is found");
        let another = ProjectRoot::resolve(&scratch.directory("htmlspec/changes/another"))
            .expect("the project is found");

        assert_eq!(one.assets(), another.assets());
        assert_eq!(one.assets(), project.assets());
    }

    #[test]
    fn a_change_is_placed_where_the_project_keeps_changes() {
        let scratch = Scratch::new("change-path");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        assert_eq!(
            project.change("add-dark-mode"),
            project.changes().join("add-dark-mode")
        );
    }

    #[test]
    fn existence_is_answerable_without_establishing_anything() {
        let scratch = Scratch::new("exists");
        assert!(!ProjectRoot::exists(&scratch.0));
        ProjectRoot::establish(&scratch.0).expect("established");
        assert!(ProjectRoot::exists(&scratch.0));
    }

    /// Every position the design names, in one tree. The rule is positional, so each case is a
    /// directory called `evidence` and the answer depends only on what holds it.
    #[test]
    fn a_directory_is_a_changes_evidence_exactly_when_a_change_holds_it() {
        let scratch = Scratch::new("evidence");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        for (relative, expected) in [
            // An active change's, and an archived change's.
            ("htmlspec/changes/a-change/evidence", true),
            (
                "htmlspec/changes/archive/2026-01-01-a-change/evidence",
                true,
            ),
            // A change may be named `evidence`; the change is not its own evidence, and its
            // evidence is.
            ("htmlspec/changes/evidence", false),
            ("htmlspec/changes/evidence/evidence", true),
            // The archive is not a change, and neither the project's places nor a capability are.
            ("htmlspec/changes/archive/evidence", false),
            ("htmlspec/specs/evidence", false),
            ("htmlspec/evidence", false),
            ("htmlspec/runbooks/release/evidence", false),
            // Beneath a change's evidence is still evidence to the walk, but it is not *the*
            // evidence directory, so it is walked when named and never excluded twice.
            ("htmlspec/changes/a-change/evidence/evidence", false),
            // Outside `htmlspec/` there are no changes.
            ("fixtures/a-change/evidence", false),
            ("evidence", false),
        ] {
            let directory = scratch.directory(relative);
            assert_eq!(
                project.is_change_evidence(&directory),
                expected,
                "{relative}"
            );
        }

        // Only a directory called `evidence` qualifies, whatever holds it.
        let named_otherwise = scratch.directory("htmlspec/changes/a-change/transcripts");
        assert!(!project.is_change_evidence(&named_otherwise));
    }

    /// The walk composes paths from whatever root it was handed, so the same directory arrives
    /// spelled several ways. Each spelling gets the same answer.
    #[test]
    fn a_changes_evidence_is_recognized_however_its_path_is_spelled() {
        let scratch = Scratch::new("evidence-spelling");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        scratch.directory("htmlspec/changes/a-change/evidence");

        let roundabout = scratch
            .0
            .join("htmlspec/specs/../changes/./a-change/evidence");
        assert!(
            project.is_change_evidence(&roundabout),
            "{}",
            roundabout.display()
        );
    }

    /// A project reached through a symbolic link recognizes evidence discovered through the link's
    /// target, and the reverse — the case `topology.rs` covers for stores, for the same reason.
    #[cfg(unix)]
    #[test]
    fn a_changes_evidence_is_recognized_through_a_symbolic_link() {
        let scratch = Scratch::new("evidence-symlink");
        let real = scratch.directory("real");
        let project = ProjectRoot::establish(&real).expect("established");
        let evidence = scratch.directory("real/htmlspec/changes/a-change/evidence");

        let link = scratch.0.join("link");
        std::os::unix::fs::symlink(&real, &link).expect("the link is created");
        let through_link = ProjectRoot::resolve(&link).expect("the link resolves to a project");

        assert!(through_link.is_change_evidence(&evidence));
        assert!(project.is_change_evidence(&link.join("htmlspec/changes/a-change/evidence")));
    }

    /// `show-what-a-modification-changed`'s task 3.4: a directory is an archived change exactly
    /// where it sits directly in the archive.
    #[test]
    fn a_directory_is_an_archived_change_exactly_when_it_sits_in_the_archive() {
        let scratch = Scratch::new("archived-change");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        for (relative, expected) in [
            ("htmlspec/changes/archive/2026-01-01-a-change", true),
            ("htmlspec/changes/a-change", false),
            // The archive is not a change in it.
            ("htmlspec/changes/archive", false),
            // A directory inside an archived change is not one.
            (
                "htmlspec/changes/archive/2026-01-01-a-change/evidence",
                false,
            ),
            ("htmlspec/specs/surface", false),
            ("htmlspec/runbooks/release", false),
        ] {
            let directory = scratch.directory(relative);
            assert_eq!(
                project.is_archived_change(&directory),
                expected,
                "{relative}"
            );
        }

        // Spelled through `..`, it is judged by where it sits.
        let roundabout = scratch
            .0
            .join("htmlspec/changes/a-change/../archive/2026-01-01-a-change");
        assert!(
            project.is_archived_change(&roundabout),
            "{}",
            roundabout.display()
        );
        let into_the_archive = scratch
            .0
            .join("htmlspec/changes/archive/2026-01-01-a-change/..");
        assert!(!project.is_archived_change(&into_the_archive));
    }

    /// An archived change reached through a linked project, through a link in the archive, and
    /// through a link elsewhere that points at it.
    #[cfg(unix)]
    #[test]
    fn an_archived_change_is_recognized_through_a_symbolic_link() {
        let scratch = Scratch::new("archived-symlink");
        let real = scratch.directory("real");
        let project = ProjectRoot::establish(&real).expect("established");
        let archived = scratch.directory("real/htmlspec/changes/archive/2026-01-01-a-change");

        let link = scratch.0.join("link");
        std::os::unix::fs::symlink(&real, &link).expect("the link is created");
        let through_link = ProjectRoot::resolve(&link).expect("the link resolves to a project");
        assert!(through_link.is_archived_change(&archived));
        assert!(
            project.is_archived_change(&link.join("htmlspec/changes/archive/2026-01-01-a-change"))
        );

        let elsewhere = scratch.directory("elsewhere/a-change");
        let in_the_archive = real.join("htmlspec/changes/archive/2026-01-02-linked");
        std::os::unix::fs::symlink(&elsewhere, &in_the_archive).expect("the link is created");
        assert!(project.is_archived_change(&in_the_archive));

        let pointing_in = scratch.0.join("pointing-in");
        std::os::unix::fs::symlink(&archived, &pointing_in).expect("the link is created");
        assert!(project.is_archived_change(&pointing_in));
        assert!(!project.is_archived_change(&elsewhere));
    }

    /// A document reached through a link in the archive, or through a linked `runs` directory, is
    /// history whatever the link points at, as [`ProjectRoot::is_archived_change`] judges a link in
    /// the archive. The same document reached by its own path is not.
    #[cfg(unix)]
    #[test]
    fn a_document_reached_through_a_symbolic_link_into_history_is_history() {
        let scratch = Scratch::new("history-symlink");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        let elsewhere = scratch.directory("elsewhere/a-change");
        let linked_change = project.archive().join("2026-01-01-linked");
        std::os::unix::fs::symlink(&elsewhere, &linked_change).expect("the link is created");
        assert!(project.is_history(&linked_change.join("design.html")));
        assert!(!project.is_history(&elsewhere.join("design.html")));

        let records = scratch.directory("elsewhere/records");
        let runbook = scratch.directory("htmlspec/runbooks/release");
        let runs = runbook.join("runs");
        std::os::unix::fs::symlink(&records, &runs).expect("the link is created");
        assert!(project.is_history(&runs.join("a-run.html")));
        assert!(!project.is_history(&records.join("a-run.html")));
        assert!(!project.is_history(&runbook.join("tasks.html")));
    }

    /// design decision 1's table, every row and each case task 3.5 names. Directories are created
    /// and documents are not, so every case is also a document that is not written yet; the last
    /// one is written, and is placed the same way.
    #[test]
    fn every_document_is_placed_by_where_it_sits() {
        use htmlspec_validate::Placement::{
            AcceptedSpecification, ChangeDocument, ChangeSpecification, Loose, RunbookDocument,
        };
        let scratch = Scratch::new("placement");
        let project = ProjectRoot::establish(&scratch.0).expect("established");

        for (relative, expected) in [
            // A change's documents, active and archived.
            ("htmlspec/changes/a-change/proposal.html", ChangeDocument),
            ("htmlspec/changes/a-change/design.html", ChangeDocument),
            ("htmlspec/changes/a-change/tasks.html", ChangeDocument),
            (
                "htmlspec/changes/a-change/specification.html",
                ChangeSpecification,
            ),
            (
                "htmlspec/changes/archive/2026-01-01-a-change/tasks.html",
                ChangeDocument,
            ),
            (
                "htmlspec/changes/archive/2026-01-01-a-change/specification.html",
                ChangeSpecification,
            ),
            // The accepted specifications, at any depth.
            ("htmlspec/specs/spec.html", AcceptedSpecification),
            ("htmlspec/specs/cap/spec.html", AcceptedSpecification),
            ("htmlspec/specs/a/b/spec.html", AcceptedSpecification),
            // A runbook's documents, a run's record included.
            ("htmlspec/runbooks/release/tasks.html", RunbookDocument),
            (
                "htmlspec/runbooks/release/specification.html",
                RunbookDocument,
            ),
            ("htmlspec/runbooks/release/runs/0-1-0.html", RunbookDocument),
            // Loose: a change's evidence, the archive's own directory, a subdirectory of a change,
            // anywhere else beneath `htmlspec/`, and outside it.
            ("htmlspec/changes/a-change/evidence/record.html", Loose),
            (
                "htmlspec/changes/a-change/evidence/specification.html",
                Loose,
            ),
            ("htmlspec/changes/archive/x.html", Loose),
            ("htmlspec/changes/a-change/notes/specification.html", Loose),
            ("htmlspec/changes/x.html", Loose),
            ("htmlspec/x.html", Loose),
            ("fixtures/a-change/tasks.html", Loose),
            ("x.html", Loose),
        ] {
            let path = scratch.0.join(relative);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("created");
            assert_eq!(project.placement(&path), expected, "{relative}");
        }

        let written = scratch.0.join("htmlspec/changes/a-change/tasks.html");
        std::fs::write(&written, "").expect("written");
        assert_eq!(project.placement(&written), ChangeDocument);
    }

    /// A bare file name sits in the current directory, and any other path in its parent.
    #[test]
    fn a_documents_directory_is_its_parent_or_the_current_one() {
        assert_eq!(directory_of(Path::new("tasks.html")), PathBuf::from("."));
        assert_eq!(directory_of(Path::new("./tasks.html")), PathBuf::from("."));
        assert_eq!(
            directory_of(Path::new("htmlspec/changes/a-change/tasks.html")),
            PathBuf::from("htmlspec/changes/a-change")
        );
    }

    /// A project reached through a symbolic link places a document spelled through the link as it
    /// places the same document spelled through its target, and the reverse.
    #[cfg(unix)]
    #[test]
    fn a_document_is_placed_through_a_symbolic_link() {
        let scratch = Scratch::new("placement-symlink");
        let real = scratch.directory("real");
        let project = ProjectRoot::establish(&real).expect("established");
        scratch.directory("real/htmlspec/changes/a-change");
        let link = scratch.0.join("link");
        std::os::unix::fs::symlink(&real, &link).expect("the link is created");
        let through_link = ProjectRoot::resolve(&link).expect("the link resolves to a project");

        let spelled_through_link = link.join("htmlspec/changes/a-change/specification.html");
        let spelled_through_target = real.join("htmlspec/changes/a-change/specification.html");
        for (root, path) in [
            (&project, &spelled_through_link),
            (&through_link, &spelled_through_target),
            (&through_link, &spelled_through_link),
        ] {
            assert_eq!(
                root.placement(path),
                htmlspec_validate::Placement::ChangeSpecification,
                "{}",
                path.display()
            );
        }
    }

    /// A writer's question: is this file a record? Answered for the record itself, anything nested
    /// beneath the evidence directory, the directory itself, and nothing else.
    #[test]
    fn a_path_lies_within_a_changes_evidence_exactly_when_an_evidence_directory_holds_it() {
        let scratch = Scratch::new("within-evidence");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        scratch.directory("htmlspec/changes/a-change/evidence/transcripts");
        scratch.directory("htmlspec/specs/evidence");
        for file in [
            "htmlspec/changes/a-change/design.html",
            "htmlspec/changes/a-change/evidence/a-record.html",
            "htmlspec/changes/a-change/evidence/transcripts/a-session.html",
            "htmlspec/specs/evidence/spec.html",
        ] {
            std::fs::write(scratch.0.join(file), "").expect("written");
        }

        for (relative, expected) in [
            ("htmlspec/changes/a-change/evidence", true),
            ("htmlspec/changes/a-change/evidence/a-record.html", true),
            (
                "htmlspec/changes/a-change/evidence/transcripts/a-session.html",
                true,
            ),
            ("htmlspec/changes/a-change", false),
            ("htmlspec/changes/a-change/design.html", false),
            ("htmlspec/specs/evidence/spec.html", false),
            // Names `evidence` on the way and lies beside it, which a lexical test would miss.
            ("htmlspec/changes/a-change/evidence/../design.html", false),
        ] {
            assert_eq!(
                project.is_within_change_evidence(&scratch.0.join(relative)),
                expected,
                "{relative}"
            );
        }
    }
}
