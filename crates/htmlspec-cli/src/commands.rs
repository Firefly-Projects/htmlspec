//! The five subcommands: `validate`, `format`, `assets`, `archive`, and `init`.
//!
//! Each returns an [`Outcome`] rather than exiting, so the process has exactly one
//! exit point and a command is testable without spawning a binary.
//!
//! `archive` is the smallest of them, and deliberately: it validates the
//! specification an agent produced from a completed change and refuses while that
//! reports an error. The production is not here and must not arrive here, because the
//! upstream kinds carry no imposed structure and there is nothing to parse reliably.
//!
//! Every command runs to completion over every document before reporting its
//! outcome. Stopping at the first failure would make the tool's report depend on
//! discovery order, and a contributor fixing three invalid documents would need three
//! runs to find them.

use std::io::Write;
use std::path::{Path, PathBuf};

use htmlspec_assets::instructions::WORKFLOWS;
use htmlspec_assets::Store;
use htmlspec_parse::model::{HeadingName, Requirement, Scenario};
use htmlspec_validate::{engine, rules, Place, Report, Scope, Strictness, Unread};
use serde::Serialize;

use crate::exit::Outcome;
use crate::output::Streams;
use crate::project::ProjectRoot;
use crate::terminal::printable;
use crate::tools::ToolId;

/// The version recorded in generated instruction files.
///
/// The crate version alone, without the build revision `--version` reports: this is
/// written into files a project commits, and a revision would make every contributor's
/// checkout produce a different byte for the same release.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One document's validation result, as a structured payload carries it.
///
/// The per-document report is `htmlspec-validate`'s own wire type, so the CLI adds
/// the path and nothing else: a consumer that already parses a validation report does
/// not have to learn a second shape.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentReport {
    /// The document's path, as given or discovered.
    path: String,
    /// The report for that document, or `null` when it could not be read.
    #[serde(skip_serializing_if = "Option::is_none")]
    report: Option<htmlspec_validate::StructuredReport>,
    /// Why the document could not be evaluated, when that is the case.
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// The whole run, as a structured payload.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunReport {
    /// Whether every document was accepted.
    accepted: bool,
    /// How many documents were evaluated.
    documents: usize,
    /// Total errors and warnings across every document.
    errors: usize,
    /// Total warnings across every document.
    warnings: usize,
    /// One entry per document, in discovery order.
    results: Vec<DocumentReport>,
}

/// Resolves what a command with no path argument operates on.
///
/// With no path, the subject is the project: resolved by walking upward, so the
/// command works from any directory inside it. With a path, the subject is exactly
/// what the contributor named — a file means that file, and a directory means that
/// directory, whether or not either is in a project. Requiring a project to validate
/// one named file would be a regression, and it would also break every conformance
/// fixture, which lives outside any project on purpose.
///
/// Either way, a walk of the subject passes over every change's evidence beneath it, and a
/// subject that is itself a change's evidence is walked in full. [`crate::discover::documents`]
/// makes that distinction, so no command restates it. What a command may do with a named record
/// is its own question: checking it is ordinary, and [`format`] refuses to write it.
fn subject(path: Option<&Path>) -> PathBuf {
    match path {
        Some(path) => path.to_owned(),
        // The project's own directory rather than the repository root, so a command
        // does not walk into `target/` or the three 4 MB explainer pages sitting beside
        // it. A project's documents are the ones under `htmlspec/`.
        //
        // No project falls back to the current directory, which keeps the tool usable
        // on a loose document and on the fixture suite — neither is in a project, and
        // the fixtures never will be.
        None => ProjectRoot::resolve(Path::new("."))
            .map(|project| project.htmlspec())
            .unwrap_or_else(|_| PathBuf::from(".")),
    }
}

/// Validates documents.
///
/// `root` is a file or a directory; `None` means the project, resolved upward, which is
/// what makes whole-project validation the no-argument default.
pub fn validate<O: Write, E: Write>(
    root: Option<&Path>,
    strictness: Strictness,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let root = subject(root);
    let root = root.as_path();
    let documents = match crate::discover::documents(root) {
        Ok(found) => found,
        Err(error) => {
            let _ = streams.failure(&error.to_string());
            return Outcome::OperationalFailure;
        }
    };

    if documents.is_empty() {
        let _ = streams.progress(&format!("no documents found under {}", root.display()));
    }

    let mut results = Vec::new();
    let mut violations = 0usize;
    let mut unreadable = 0usize;
    // Read once for the whole run: every document a reference can reach, the first time one does.
    let mut reach = Reach::default();
    let mut errors = 0usize;
    let mut warnings = 0usize;

    for path in &documents {
        let _ = streams.progress(&format!("validating {}", path.display()));

        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                unreadable += 1;
                let _ = streams.failure(&format!("cannot read {}: {error}", path.display()));
                results.push(DocumentReport {
                    path: path.display().to_string(),
                    report: None,
                    error: Some(error.to_string()),
                });
                continue;
            }
        };

        let report = validate_source(&source, path, &mut reach, strictness);
        errors += report.errors().count();
        warnings += report.warnings().count();
        if !report.is_accepted() {
            violations += 1;
        }

        if streams.form().is_structured() {
            results.push(DocumentReport {
                path: path.display().to_string(),
                report: Some(htmlspec_validate::StructuredReport::of(&report)),
                error: None,
            });
        } else {
            // The document's name precedes its diagnostics, because a whole-project
            // run reports many and a diagnostic with no document is unactionable.
            let _ = streams.report(&format!("{}\n", path.display()));
            let _ = streams.report(&indent(&htmlspec_validate::render_human(&report)));
        }
    }

    if streams.form().is_structured() {
        let run = RunReport {
            accepted: violations == 0 && unreadable == 0,
            documents: documents.len(),
            errors,
            warnings,
            results,
        };
        match serde_json::to_string_pretty(&run) {
            Ok(json) => {
                let _ = streams.payload(&json);
            }
            Err(error) => {
                let _ = streams.failure(&format!("cannot serialize the report: {error}"));
                return Outcome::OperationalFailure;
            }
        }
    } else if documents.len() > 1 {
        let _ = streams.report(&format!(
            "\n{} document(s): {errors} error(s), {warnings} warning(s)\n",
            documents.len()
        ));
    }

    // An unreadable file is an operational failure even when other documents were
    // invalid: the run did not do what was asked, and reporting only the violations
    // would let a typo in a path read as a clean project.
    if unreadable > 0 {
        return Outcome::OperationalFailure;
    }
    if violations > 0 {
        return Outcome::DocumentViolation;
    }
    Outcome::Success
}

/// Validates one document's text, as the document at `path`.
///
/// Both of the document's surroundings come from `path`, so no caller can pair a document with
/// another document's surroundings, or leave out where its references reach:
///
/// - the asset store, which is the project's store or [`Store::Absent`] when the document is not
///   inside a project that has one. That distinction is the entire scope of the `asset/*` rules:
///   with no store there are no bytes for a reference to disagree with, so the references are left
///   unresolved rather than reported. The store is seen from the document ([`store_seen_from`]), so
///   a reference resolves only through a path that reaches it;
/// - the scope, which says where the document sits and what each place its references reach
///   declares, as document-schema's "Cross-references address units by identifier" decides.
///
/// `reach` holds what those places declare, read at most once for a run. A caller validating one
/// document passes a fresh one.
fn validate_source(source: &str, path: &Path, reach: &mut Reach, strictness: Strictness) -> Report {
    let outcome = htmlspec_parse::parse_document(source);
    let projection = htmlspec_parse::project(&outcome.document);
    let scope = reach.scope_for(path);
    engine::run(
        &outcome,
        &projection,
        &store_seen_from(path),
        &scope,
        rules::ALL,
        strictness,
    )
}

/// Validates a document the program just composed, against the store it will sit beside.
///
/// The pairing [`write_document`] performs, exposed for [`crate::runs`], which writes a document by
/// composing it rather than by rendering a template and owes the same promise: a document that
/// validates immediately. Resolving a real store rather than passing [`Store::Absent`] is what makes
/// the promise the strong form — the document is checked against the bytes actually beside it, so a
/// composed reference naming an address the store does not hold is caught here rather than the first
/// time a reader opens the page.
pub(crate) fn validate_source_at(source: &str, path: &Path) -> Report {
    validate_source(source, path, &mut Reach::default(), Strictness::Default)
}

/// What the places a document's references reach declare, read at most once for a run.
///
/// A whole-project `validate` meets the same few reachable documents from every document it checks:
/// each change's specification, and every accepted specification. So each one is read and projected
/// once, the first time a document reaches it, and every later document is given what it declares.
/// A rule never reads a file. It is given the identifiers, for the reason `Subject` gives for the
/// asset store.
#[derive(Default)]
pub(crate) struct Reach {
    /// Each project's accepted specifications, by where the project keeps them, each with its
    /// canonical path so a document can leave itself out.
    accepted: std::collections::BTreeMap<PathBuf, Vec<(PathBuf, Reading)>>,
    /// Each change's specification, by its path. `None` where the change has none yet, which is a
    /// change in progress and declares nothing, rather than a document that could not be read.
    changes: std::collections::BTreeMap<PathBuf, Option<Reading>>,
}

/// One reachable document, as resolving a reference needs it.
enum Reading {
    /// Its projection, boxed because it is far larger than the other case.
    Read(Box<htmlspec_parse::Projection>),
    /// It could not be read, or it has no determinate content. It declares nothing a reference
    /// can rely on, and a report names it rather than reading it as empty.
    Unread(Unread),
}

impl Reading {
    /// The document at `path`, named `document` in a report.
    fn of(path: &Path, document: String) -> Self {
        match std::fs::read_to_string(path) {
            Err(error) => Self::Unread(Unread {
                document,
                reason: format!("it cannot be read: {error}"),
            }),
            Ok(source) => {
                let outcome = htmlspec_parse::parse_document(&source);
                if outcome.is_recovery_free() {
                    Self::Read(Box::new(htmlspec_parse::project(&outcome.document)))
                } else {
                    Self::Unread(Unread {
                        document,
                        reason: String::from("its markup needs the parser to recover"),
                    })
                }
            }
        }
    }
}

impl Reach {
    /// Where the document at `path` sits, and what each place it reaches declares.
    ///
    /// A document in no project is loose, as is every document the project's layout does not
    /// place. [`ProjectRoot::placement`] decides that, and [`Scope::gather`] asks for exactly the
    /// places the placement reaches.
    fn scope_for(&mut self, path: &Path) -> Scope {
        let start = crate::project::directory_of(path);
        let Ok(project) = ProjectRoot::resolve(&start) else {
            return Scope::loose();
        };
        let placement = project.placement(path);
        Scope::gather(placement, |place| match place {
            Place::ChangeSpecification => self.change_specification(&project, &start),
            Place::AcceptedSpecifications => self.accepted_specifications(&project, path),
        })
    }

    /// What the specification of the change whose directory is `change` declares.
    fn change_specification(
        &mut self,
        project: &ProjectRoot,
        change: &Path,
    ) -> htmlspec_validate::references::Declared {
        let specification = crate::project::canonical(change).join(CHANGE_SPECIFICATION);
        let reading = self
            .changes
            .entry(specification.clone())
            .or_insert_with(|| {
                specification
                    .is_file()
                    .then(|| Reading::of(&specification, named_in(project, &specification)))
            });
        declared_by(reading.iter())
    }

    /// What every accepted specification declares, less the document at `path` where it is one.
    ///
    /// The document itself is the first place a reference is tried, and it is read as it was
    /// given, which can differ from the copy on disk.
    fn accepted_specifications(
        &mut self,
        project: &ProjectRoot,
        path: &Path,
    ) -> htmlspec_validate::references::Declared {
        let readings = self
            .accepted
            .entry(crate::project::canonical(&project.specs()))
            .or_insert_with(|| {
                accepted_specifications(project)
                    .into_iter()
                    .map(|document| {
                        let reading = Reading::of(&document, named_in(project, &document));
                        (crate::project::canonical(&document), reading)
                    })
                    .collect()
            });
        let own = crate::project::canonical(path);
        declared_by(
            readings
                .iter()
                .filter(|(document, _)| *document != own)
                .map(|(_, reading)| reading),
        )
    }
}

/// What `readings` declare, with the ones that could not be read named.
fn declared_by<'a>(
    readings: impl Iterator<Item = &'a Reading>,
) -> htmlspec_validate::references::Declared {
    let mut projections = Vec::new();
    let mut unread = Vec::new();
    for reading in readings {
        match reading {
            Reading::Read(projection) => projections.push(&**projection),
            Reading::Unread(document) => unread.push(document.clone()),
        }
    }
    htmlspec_validate::references::Declared::gather(projections, unread)
}

/// `path` as a report names it: relative to the project, where it is inside one.
fn named_in(project: &ProjectRoot, path: &Path) -> String {
    let root = crate::project::canonical(project.path());
    let path = crate::project::canonical(path);
    path.strip_prefix(&root).map_or_else(
        |_| path.display().to_string(),
        |relative| relative.display().to_string(),
    )
}

/// The project's accepted specifications: every document discovery finds where the project keeps
/// them.
///
/// One definition, read by [`merge_state`] and by reference resolution both, so `sync` and
/// validation cannot disagree about which documents are the accepted contract.
fn accepted_specifications(project: &ProjectRoot) -> Vec<PathBuf> {
    crate::discover::documents(&project.specs()).unwrap_or_default()
}

/// The reference a document at `path` should use for `project`'s store.
///
/// [`assets_prefix`] answers this from a resolved store, which is the formatter's question — is this
/// reference correct about bytes that exist. This answers the creation question, where the document
/// does not exist yet, so the prefix is derived from the project's layout the way
/// [`write_document`] derives it. Both derive from the *project* rather than from a document's
/// neighbours, which is the property that keeps one store per project the only reachable outcome.
pub(crate) fn assets_prefix_for(path: &Path, project: &ProjectRoot) -> String {
    let directory = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_owned(),
        _ => PathBuf::from("."),
    };
    relative_reference(&directory, &project.assets()).unwrap_or_else(|| String::from("assets"))
}

/// The path from the document at `path` to `store`'s directory, for a rewritten
/// reference.
///
/// Derived from where the store actually is, not from what the document's existing
/// references happen to say. Those differ exactly when a document's references are wrong,
/// which is the case a rewrite exists to repair — so trusting the old prefix would
/// preserve the error it is meant to fix.
///
/// Falls back to `"assets"` when the two paths cannot be related, which is the reference a
/// document beside its store carries. A wrong-but-conventional prefix is easier for a
/// contributor to spot and correct than an absolute path that breaks the moment the
/// project moves.
fn assets_prefix(path: &Path, store: &Store) -> String {
    let Store::Present(contents) = store else {
        return String::from("assets");
    };
    let document_directory = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_owned(),
        _ => PathBuf::from("."),
    };
    relative_reference(&document_directory, &contents.root)
        .unwrap_or_else(|| String::from("assets"))
}

/// The asset store a document at `path` belongs to, as the document sees it.
///
/// Seen from the document's directory ([`Store::seen_from`]), so a reference resolves only where its
/// path, followed from there, names the store's directory. document-validation's "An asset reference
/// must match the bytes it names" rejects a reference that does not resolve to stored bytes, and a
/// browser resolves a reference by following its path: an address at the end of a path that reaches
/// another directory names bytes the document never loads.
///
/// The formatter is given the store unseen, so it resolves by address as before: deterministic-
/// formatting's "Formatting preserves meaning" leaves a reference naming stored bytes as written.
fn store_seen_from(path: &Path) -> Store {
    store_for(path).seen_from(&crate::project::directory_of(path))
}

/// Finds the asset store a document at `path` belongs to.
///
/// Walks upward from the document's own directory to the filesystem root, taking the
/// nearest ancestor holding an `assets/` directory. Upward rather than "the directory
/// the command was pointed at", because the specs scope the rule to *a document
/// validated within a project that has an asset store*, and a document is no less
/// inside its project when a contributor names it directly. Anchoring on the given
/// root instead would mean `htmlspec validate doc.html` skipped a check that
/// `htmlspec validate .` applied, so whether a stale reference was caught would
/// depend on how the tool was invoked — which is how the defect this change exists to
/// fix survived in the first place.
///
/// Returns [`Store::Absent`] when no ancestor has one. A document with no store above
/// it claims no provenance, which is the ordinary case for a conformance fixture.
fn store_for(path: &Path) -> Store {
    let start = crate::project::directory_of(path);

    // Absolutized so the walk terminates: `parent()` on a relative path stops at the
    // empty path, which would miss a store above the current directory.
    let mut directory = if start.is_absolute() {
        start
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(start)
    };

    loop {
        let store = Store::discover(&directory);
        if store.is_present() {
            return store;
        }
        if !directory.pop() {
            return Store::Absent;
        }
    }
}

/// Indents a block by two spaces, so a document's diagnostics sit under its name.
fn indent(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                String::from("\n")
            } else {
                format!("  {line}\n")
            }
        })
        .collect()
}

/// Checks a project's asset topology.
///
/// A separate subcommand rather than part of `validate`, because its subject is a
/// project rather than a document. `validate` reports per document and takes a file
/// path; this takes a project root and reports once. Folding it into `validate` would
/// mean `htmlspec validate one-document.html` either reported a violation the named
/// document did not cause, or silently skipped a check depending on the argument — and
/// a check whose presence depends on how the tool was invoked is the failure mode this
/// change exists to remove.
///
/// `root` is the directory to check; `None` means the project, resolved upward.
///
/// **Every store in the project, not just one.** The previous version called
/// `Store::discover` on the directory it was handed and did not descend, so a project
/// holding two 6 MB stores two directories down reported `no asset store under .` and
/// exited zero — the duplicate-detection requirement satisfied by a command that could
/// not observe the duplication the tool's own default produced. Descending is what makes
/// the check mean what its name says.
pub fn assets<O: Write, E: Write>(root: Option<&Path>, streams: &mut Streams<O, E>) -> Outcome {
    let resolved = ProjectRoot::resolve(root.unwrap_or_else(|| Path::new(".")));
    let root = match root {
        Some(root) => root.to_owned(),
        None => resolved
            .as_ref()
            .map(|project| project.path().to_owned())
            .unwrap_or_else(|_| PathBuf::from(".")),
    };
    let root = root.as_path();

    // Cross-store comparison is a statement about **one project**, so it applies only where
    // a project is in view, and only among that project's stores. A directory that merely
    // contains several independent projects — this repository's conformance fixtures, each
    // carrying its own deliberate store — holds no duplication to report: those bytes are
    // separate projects' single copies, not one project's second copy. Comparing across them
    // reported the fixture suite as duplicated, which is the check misreading its own subject.
    //
    // The first repair stopped comparing when no project resolved. It did not decide what to
    // compare when one does, so an empty `htmlspec/` at a repository root compared every store
    // in the repository and reported nine duplicates, none of them the project's.
    // `crate::topology` decides it: a project's stores are the ones within its `htmlspec/`.
    let project = resolved.as_ref().ok();
    let topology = crate::topology::check(project, root);
    let stores = topology.partition.project.len() + topology.partition.others.len();
    if stores == 0 {
        // Not a failure. A directory with no store has nothing to duplicate, which the
        // specs require explicitly, and a contributor who ran this in the wrong place is
        // better served by hearing so than by a silent pass.
        let _ = streams.progress(&format!(
            "no asset store under {}, so there is nothing to check",
            root.display()
        ));
        return Outcome::Success;
    }

    let crate::topology::Topology {
        partition,
        stored,
        diagnostics,
    } = topology;
    if project.is_none() {
        if stores > 1 {
            let _ = streams.progress(&format!(
                "note: {} is not a project, so its {stores} stores were each checked on their \
                 own. Bytes shared between separate projects are not duplication",
                root.display(),
            ));
        }
    } else {
        // Stated whenever a project is in view, as document-validation requires, and the stores
        // outside it named rather than counted. A contributor whose duplicate stopped being
        // reported needs to see that the store was checked on its own rather than wonder whether
        // it was checked. Each store is named by its `assets/` directory: discovery returns the
        // directory holding it, and for a repository root's store that reads as a bare `.`.
        let outside = if partition.others.is_empty() {
            String::from("No store outside it was found")
        } else {
            format!(
                "{} store(s) outside it belong to something else and were each checked on its \
                 own: {}",
                partition.others.len(),
                store_names(&partition.others).join(", ")
            )
        };
        let _ = streams.progress(&format!(
            "note: {} store(s) within the project's htmlspec/ were compared with one another. \
             {outside}",
            partition.project.len(),
        ));
    }

    let _ = streams.progress(&format!(
        "checking {stored} stored asset(s) in {stores} store(s) under {}",
        root.display()
    ));

    if streams.form().is_structured() {
        let payload = serde_json::json!({
            "accepted": diagnostics.is_empty(),
            "storedAssets": stored,
            // The total, as before. The two lists below say how it divides.
            "stores": stores,
            "projectStores": store_names(&partition.project),
            "otherStores": store_names(&partition.others),
            "diagnostics": diagnostics
                .iter()
                .map(htmlspec_validate::StructuredDiagnostic::of)
                .collect::<Vec<_>>(),
        });
        match serde_json::to_string_pretty(&payload) {
            Ok(json) => {
                let _ = streams.payload(&json);
            }
            Err(error) => {
                let _ = streams.failure(&format!("cannot serialize the report: {error}"));
                return Outcome::OperationalFailure;
            }
        }
    } else {
        for diagnostic in &diagnostics {
            let _ = streams.report(&format!("{diagnostic}\n"));
        }
        if diagnostics.is_empty() {
            let _ = streams.report("each stored asset appears once\n");
        } else {
            let _ = streams.report(&format!(
                "\n{} duplicated asset(s): the project stores bytes it could share\n",
                diagnostics.len()
            ));
        }
    }

    if diagnostics.is_empty() {
        Outcome::Success
    } else {
        Outcome::DocumentViolation
    }
}

/// Each store named by its `assets/` directory, as a report and the payload name it.
fn store_names(stores: &[PathBuf]) -> Vec<String> {
    stores
        .iter()
        .map(|store| {
            store
                .join(htmlspec_assets::store::DIRECTORY)
                .display()
                .to_string()
        })
        .collect()
}

/// What `format` does with a document that is not canonical.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormatMode {
    /// Rewrite the document in place.
    Write,
    /// Report whether it is canonical and modify nothing.
    Check,
}

/// Whether `format` gives documents the toolchain's fallback stylesheet.
///
/// Asked for, never assumed: document-template's "A document stays readable when its stylesheet
/// cannot load" puts the fallback in every document the toolchain writes, and deterministic-
/// formatting's "Formatting preserves meaning" lets formatting add it to an existing document only on
/// request. The reasons are in `htmlspec_format::fallback`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackRequest {
    /// Leave each document's fallback, present, absent, or stale, as written.
    Leave,
    /// Add the current fallback where it is absent and replace one that differs, except in history.
    Apply,
}

/// Whether `format` moves documents onto the store's current assets.
///
/// Asked for, never assumed: deterministic-formatting's "Formatting preserves meaning" leaves a
/// reference to stored bytes as written, "even when the store also holds a newer revision", and
/// makes re-pointing it the third exception, on a contributor's request. The reasons are on
/// [`htmlspec_format::upgrade`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetRequest {
    /// Leave each reference to stored bytes as written.
    Keep,
    /// Rewrite each reference to the store's current revision of its asset, except in history.
    Upgrade,
}

/// One document a walk found, placed where it resolves.
struct Placed {
    /// The path as the walk found it, which is how every report names the document.
    path: PathBuf,
    /// Where the document resolves: the walked path with every symbolic link followed. Read,
    /// judged, and written there, so a link to a document is never judged by the link's place.
    real: PathBuf,
    /// Whether the resolved document is history: in an archived change, or a run's record.
    is_history: bool,
    /// Whether the resolved document is within a change's evidence, which no operation rewrites.
    is_evidence: bool,
}

/// The project each directory resolves into, asked once per directory.
#[derive(Default)]
struct Places {
    projects: std::collections::HashMap<PathBuf, Option<ProjectRoot>>,
}

impl Places {
    /// Places the document at `path`, judging it by the project its resolved directory belongs to.
    ///
    /// A document that cannot be resolved, or resolves into no project, is placed by its path as
    /// given and has no history to leave alone, as a walk outside any project always had.
    fn place(&mut self, path: &Path) -> Placed {
        let real = path.canonicalize().unwrap_or_else(|_| path.to_owned());
        let directory = crate::project::directory_of(&real);
        let project = self
            .projects
            .entry(directory.clone())
            .or_insert_with(|| ProjectRoot::resolve(&directory).ok());
        let (is_history, is_evidence) = match project {
            Some(project) => (
                project.is_history(&real),
                project.is_within_change_evidence(&real),
            ),
            None => (false, false),
        };
        Placed {
            path: path.to_owned(),
            real,
            is_history,
            is_evidence,
        }
    }
}

/// Formats documents, or checks whether they are canonical.
///
/// `root` is a file or a directory; `None` means the project, resolved upward, on the
/// same terms as [`validate`].
///
/// **Writing never reaches a change's evidence, even when a contributor names it.**
/// artifact-kinds states that no operation rewrites a file beneath a change's evidence, and a
/// walk never meets one. A named record is still found, because a named path is walked as named
/// and checking it is the ordinary answer to an ordinary question. So writing mode refuses before
/// anything is written: each record is named, nothing is formatted, and the exit is the one for an
/// operation that could not run. Refusing the whole run rather than skipping the records keeps the
/// outcome one fact, where a partial write would report success over a request it half ignored.
/// Checking mode is not writing and is unaffected. A review of the pull request found that the
/// first version rewrote a named evidence directory in full.
pub fn format<O: Write, E: Write>(
    root: Option<&Path>,
    mode: FormatMode,
    fallback: FallbackRequest,
    assets: AssetRequest,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let root = subject(root);
    let root = root.as_path();
    let documents = match crate::discover::documents(root) {
        Ok(found) => found,
        Err(error) => {
            let _ = streams.failure(&error.to_string());
            return Outcome::OperationalFailure;
        }
    };
    // Each document is judged where it resolves, by the project it resolves into. **Not by the
    // project the walk's root resolves into**, which is what this did until PR #80's review: a root
    // above a project resolves into none, so `format --upgrade-assets` and `format --fallback` given
    // the directory holding a project rewrote its archived changes and run records, and a link to an
    // archived document from a living directory was judged as living. Resolving per document closes
    // both, and the `--fallback` half was a gap before this change.
    let mut places = Places::default();
    let placed: Vec<Placed> = documents.iter().map(|path| places.place(path)).collect();

    if mode == FormatMode::Write {
        let records: Vec<&PathBuf> = placed
            .iter()
            .filter(|document| document.is_evidence)
            .map(|document| &document.path)
            .collect();
        if !records.is_empty() {
            for record in &records {
                let _ = streams.failure(&format!(
                    "refusing to write {}: it is a change's evidence, a record kept as it was \
                     written, and no operation rewrites one. `htmlspec format --check` and \
                     `htmlspec validate` still check it",
                    record.display()
                ));
            }
            let _ = streams.failure("nothing was written");
            return Outcome::OperationalFailure;
        }
    }

    let mut changed: Vec<PathBuf> = Vec::new();
    let mut refused = 0usize;
    let mut failed = 0usize;
    // The stores this run has already given the current assets, so each is written once.
    let mut established: Vec<PathBuf> = Vec::new();

    for Placed {
        path,
        real,
        is_history,
        ..
    } in &placed
    {
        let source = match std::fs::read_to_string(real) {
            Ok(source) => source,
            Err(error) => {
                failed += 1;
                let _ = streams.failure(&format!("cannot read {}: {error}", path.display()));
                continue;
            }
        };

        // The store and the prefix are the resolved file's, so a link formats the document it
        // points at as that document's own references read.
        let mut store = store_for(real);
        let prefix = assets_prefix(real, &store);
        let mut current = source.clone();
        // Whether `--check` found the document behind the store's current assets. Reported apart
        // from canonical form, because a canonical document can be behind and the reverse.
        let mut behind = false;

        // The upgrade, only when asked for and never in history: an archived change renders with
        // what it was reviewed against, and a run's record is history rather than contract. Before
        // the fill below, so a reference the upgrade moved is one the fill then finds present.
        if assets == AssetRequest::Upgrade {
            if *is_history {
                let _ = streams.progress(&format!(
                    "{}: history, so its asset references are left as written",
                    path.display()
                ));
            } else {
                let upgraded = htmlspec_format::upgrade(&source, &store, &prefix);
                for blocked in &upgraded.blocked {
                    let _ = streams.progress(&format!(
                        "{}: {} is left as written, because the store holds other bytes at {} \
                         than the bytes that address names: that file was edited or replaced. \
                         Restore it from version control, or delete it and run `htmlspec format \
                         --upgrade-assets` again, which writes this program's own copy there",
                        path.display(),
                        blocked.was,
                        blocked.now
                    ));
                }
                if !upgraded.upgraded.is_empty() {
                    for moved in &upgraded.upgraded {
                        let _ = streams.progress(&match mode {
                            FormatMode::Write => format!(
                                "{}: {} now names the store's current revision, {}",
                                path.display(),
                                moved.name,
                                moved.now
                            ),
                            FormatMode::Check => format!(
                                "{}: {} names {}, and the store's current revision is {}",
                                path.display(),
                                moved.name,
                                moved.was,
                                moved.now
                            ),
                        });
                    }
                    match mode {
                        // The bytes the document now names come from this binary and are written
                        // into the store, never downloaded.
                        FormatMode::Write => {
                            if let Store::Present(contents) = &store {
                                if !established.contains(&contents.root) {
                                    if let Err(error) = establish_assets(&contents.root, streams) {
                                        failed += 1;
                                        let _ = streams.failure(&error);
                                        continue;
                                    }
                                    established.push(contents.root.clone());
                                }
                            }
                            store = store_for(real);
                            current = upgraded.rewritten;
                        }
                        // Check mode writes nothing, the store included. It reports the document
                        // as behind and judges its canonical form on what is on disk.
                        FormatMode::Check => {
                            behind = true;
                            let _ = streams.report(&format!(
                                "{}: references an asset behind the store's current revision\n",
                                path.display()
                            ));
                        }
                    }
                }
            }
        }

        // Asset references are filled before canonicalization, so a document that is
        // both unformatted and referencing absent bytes is settled in one pass.
        //
        // Only *absent* references are filled. A reference naming bytes the store holds
        // is reported and left alone, because re-pointing an archived document at newer
        // bytes would change what a reviewed artifact renders with — a side effect no
        // contributor asked for by running the formatter.
        let rewritten = htmlspec_format::rewrite(&current, &store, &prefix);
        for change in rewritten.filled() {
            if let htmlspec_format::ReferenceChange::Filled { name, now, .. } = change {
                let _ = streams.progress(&format!(
                    "{}: {name} now names the bytes stored at {now}",
                    path.display()
                ));
            }
        }

        // The fallback stylesheet, only when asked for, and never in history. Applied after the
        // references are filled, so a document that needs both is settled in one pass, and before
        // canonicalization, which the request's output already satisfies.
        let mut settled = rewritten.rewritten;
        if fallback == FallbackRequest::Apply {
            if *is_history {
                let _ = streams.progress(&format!(
                    "{}: history, so its fallback stylesheet is left as written",
                    path.display()
                ));
            } else {
                match htmlspec_format::fallback::apply(&settled) {
                    Ok(applied) => {
                        match applied.change {
                            htmlspec_format::fallback::FallbackChange::Added => {
                                let _ = streams.progress(&format!(
                                    "{}: gains the fallback stylesheet",
                                    path.display()
                                ));
                            }
                            htmlspec_format::fallback::FallbackChange::Refreshed => {
                                let _ = streams.progress(&format!(
                                    "{}: its fallback stylesheet is replaced by the current one",
                                    path.display()
                                ));
                            }
                            htmlspec_format::fallback::FallbackChange::Current => {}
                        }
                        settled = applied.source;
                    }
                    Err(error) => {
                        refused += 1;
                        let _ = streams.failure(&format!("{}: {error}", path.display()));
                        continue;
                    }
                }
            }
        }

        // A document too ambiguous to format is refused with a reason rather than
        // canonicalized into one of its readings, which would be a silent repair.
        let outcome = match htmlspec_format::check(&settled) {
            Ok(outcome) => outcome,
            Err(error) => {
                refused += 1;
                let _ = streams.failure(&format!("{}: {error}", path.display()));
                continue;
            }
        };
        // Canonical means "identical to what should be on disk", which now includes the
        // filled references. A document whose serialization is canonical but whose
        // references name absent bytes is not yet settled, and check mode must say so.
        let outcome = htmlspec_format::CheckOutcome {
            is_canonical: outcome.is_canonical && source == settled,
            canonical: outcome.canonical,
        };

        if outcome.is_canonical {
            if behind {
                changed.push(path.clone());
            } else {
                let _ = streams.progress(&format!("{} is canonical", path.display()));
            }
            continue;
        }

        changed.push(path.clone());
        match mode {
            FormatMode::Check => {
                // Check mode names the document and modifies nothing.
                let _ = streams.report(&format!("{}: not canonically formatted\n", path.display()));
            }
            FormatMode::Write => {
                // Written where the document resolves, which is where it was read from.
                if let Err(error) = std::fs::write(real, &outcome.canonical) {
                    failed += 1;
                    let _ = streams.failure(&format!("cannot write {}: {error}", path.display()));
                    continue;
                }
                let _ = streams.report(&format!("{}: formatted\n", path.display()));
            }
        }
    }

    if failed > 0 || refused > 0 {
        return Outcome::OperationalFailure;
    }
    match mode {
        // Check mode exits nonzero if any document would change, which is what makes
        // it usable as a continuous-integration gate.
        FormatMode::Check if !changed.is_empty() => Outcome::DocumentViolation,
        _ => Outcome::Success,
    }
}

/// Creates a new document of `kind`, referencing the project's asset store.
///
/// The kind decides the document's shape rather than only a metadata value, and the
/// asymmetry between the shapes is the point: a `proposal` arrives as prose with no
/// unit scaffolding, and a `specification` arrives with the units its kind expects.
///
/// The store comes from the project rather than from this document's own directory,
/// which is what keeps a project to one copy of its runtime. Outside a project the
/// document's own directory is used, so a loose document still works.
pub fn new_document<O: Write, E: Write>(
    kind: &str,
    path: &Path,
    subject: &str,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let assets_root = project_store_root(path);
    write_document(path, subject, kind, assets_root.as_deref(), streams)
}

/// The directory holding the store a new document at `destination` references, where that
/// store is its project's: the project's own directory, `htmlspec/`. `None` where the document
/// is to have a store beside it.
///
/// **Decided by the destination, never by the working directory.** document-template's "A
/// project can be initialized with a conforming document" has the new document pass validation
/// immediately, and validation looks for a document's store in the document's own directory and
/// above it ([`store_for`]). So the store has to be one that search reaches from where the
/// document will live. A document beneath a project's `htmlspec/` reaches the project's store,
/// and project-bootstrap's "A project has a declared root holding one asset store" has it
/// reference that store. A document anywhere else, in another directory of the project's root or
/// outside every project, cannot reach it, and gets a store of its own.
///
/// The working directory decided this before, and three of the four ways it can differ from the
/// destination went wrong: a document validation rejected, or a second store inside a project.
/// `new change` still searches from the working directory, because its subject is a project.
///
/// The destination is walked as spelled, normalized lexically, because [`ProjectRoot::resolve`]
/// and [`store_for`] both walk a path as spelled. Where it is beneath the project is asked of the
/// filesystem, the way the walk that found the project asked it: the destination's first directory
/// below the project's root is the project's own directory when the two are one directory, however
/// each is spelled. So `HTMLSPEC/changes/` on a filesystem that ignores case, or a link to
/// `htmlspec/` at that level, uses the project's store. The store is returned in the destination's
/// own spelling, which is the path validation walks to find it.
fn project_store_root(destination: &Path) -> Option<PathBuf> {
    let directory = normalize(&crate::project::directory_of(destination));
    let project = ProjectRoot::resolve(&directory).ok()?;
    let first = directory
        .strip_prefix(project.path())
        .ok()?
        .components()
        .next()?;
    let own = project.path().join(first);
    same_directory(&own, &project.htmlspec()).then_some(own)
}

/// Whether two paths name one directory, asked of the filesystem rather than of their spelling.
///
/// A path that does not exist names no directory, so it is the same as nothing.
pub(crate) fn same_directory(one: &Path, other: &Path) -> bool {
    let (Ok(one), Ok(other)) = (std::fs::metadata(one), std::fs::metadata(other)) else {
        return false;
    };
    one.is_dir() && other.is_dir() && same_node(&one, &other)
}

#[cfg(unix)]
fn same_node(one: &std::fs::Metadata, other: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    one.dev() == other.dev() && one.ino() == other.ino()
}

/// Without device and inode numbers, two directories are compared by everything else the
/// filesystem reports about them, which a second directory almost never matches exactly.
#[cfg(not(unix))]
fn same_node(one: &std::fs::Metadata, other: &std::fs::Metadata) -> bool {
    one.created().ok() == other.created().ok() && one.modified().ok() == other.modified().ok()
}

/// Writes one document, establishing the store it references if that is absent.
///
/// The shared body of [`new_document`] and [`new_change`]: both write a document of a
/// declared kind against a store, and differ only in how many and where.
fn write_document<O: Write, E: Write>(
    path: &Path,
    subject: &str,
    kind: &str,
    assets_root: Option<&Path>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    if path.exists() {
        let _ = streams.failure(&format!(
            "{} already exists; new will not overwrite a document",
            path.display()
        ));
        return Outcome::OperationalFailure;
    }

    // The kind is checked before anything is written, so a misspelling does not leave
    // an assets directory behind. `design` is named separately because it is a real
    // kind with no template rather than a typo, and a contributor who asked for one
    // deserves to know which it is.
    if htmlspec_assets::template_for(kind).is_none() {
        let available: Vec<&str> = htmlspec_assets::TEMPLATES
            .iter()
            .map(|template| template.kind)
            .collect();
        let _ = streams.failure(&format!(
            "{kind} is not an artifact kind. Available: {}",
            available.join(", ")
        ));
        return Outcome::OperationalFailure;
    }

    write_rendered(
        path,
        kind,
        |reference| htmlspec_assets::render_kind(kind, subject, reference),
        assets_root,
        streams,
    )
}

/// Writes one document rendered by `render`, which takes the reference from the document to the
/// store, then validates what it wrote.
///
/// The part of [`write_document`] that does not depend on which template renders the document, so
/// a runbook's two documents are written, stored against, and checked exactly as a change's are.
fn write_rendered<O: Write, E: Write>(
    path: &Path,
    kind: &str,
    render: impl FnOnce(&str) -> Option<String>,
    assets_root: Option<&Path>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    if path.exists() {
        let _ = streams.failure(&format!(
            "{} already exists; new will not overwrite a document",
            path.display()
        ));
        return Outcome::OperationalFailure;
    }

    // Where the shared assets live, and how the new document refers to them. The
    // reference is relative, because an absolute one would break the moment the
    // project moved and the whole premise is a file that opens anywhere.
    let document_directory = path.parent().unwrap_or_else(|| Path::new("."));
    let assets_directory = match assets_root {
        Some(root) => root.join("assets"),
        None => document_directory.join("assets"),
    };
    let reference = match relative_reference(document_directory, &assets_directory) {
        Some(reference) => reference,
        None => {
            let _ = streams.failure(&format!(
                "cannot express {} relative to {}; the document must sit inside the project \
                 whose store it references",
                assets_directory.display(),
                document_directory.display()
            ));
            return Outcome::OperationalFailure;
        }
    };

    if let Err(error) = establish_assets(&assets_directory, streams) {
        let _ = streams.failure(&error);
        return Outcome::OperationalFailure;
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                let _ = streams.failure(&format!("cannot create {}: {error}", parent.display()));
                return Outcome::OperationalFailure;
            }
        }
    }

    let Some(document) = render(&reference) else {
        // Unreachable for `write_document`, which checked the kind before anything was written.
        let _ = streams.failure(&format!("no template produces the {kind} kind"));
        return Outcome::OperationalFailure;
    };
    if let Err(error) = std::fs::write(path, &document) {
        let _ = streams.failure(&format!("cannot write {}: {error}", path.display()));
        return Outcome::OperationalFailure;
    }
    let _ = streams.report(&format!("{}: created a {kind}\n", path.display()));

    // The promise is a document that validates immediately, so it is validated here
    // rather than asserted. A failure means a defect in the template, and the file is
    // left in place so a contributor can see what it produced.
    //
    // Validated against the store `establish_assets` just wrote, which is the strongest
    // available form of the promise: the document is checked against the bytes actually
    // beside it, so a template referencing an address `init` did not write is caught
    // here rather than the first time a reader opens the page. That is the defect class
    // of task 6.4, and it is why this resolves a real store rather than passing Absent.
    let report = validate_source(&document, path, &mut Reach::default(), Strictness::Default);
    if !report.is_accepted() {
        let _ = streams.failure(&format!(
            "the created document does not validate, which is a defect in the template:\n{}",
            htmlspec_validate::render_human(&report)
        ));
        return Outcome::OperationalFailure;
    }
    Outcome::Success
}

/// Refuses a directory that is a runbook rather than a change.
///
/// **A runbook resolved as a change is destroyed by archival, which moves the directory into
/// `changes/archive/`.** Measured rather than reasoned about: with a real runbook — two
/// documents, real content, the project version advanced — `htmlspec archive
/// htmlspec/runbooks/release --yes` reported `archived to …/changes/archive/2026-09-17-release`
/// and left `runbooks/` empty.
///
/// `runbook-authority` states the obligation this enforces: a procedure that is run repeatedly
/// is not a change and does not archive, and a representation that archives it after its first
/// run destroys the thing being kept. The contract said so and nothing checked it, which is the
/// shape `project-bootstrap` names — a rule nothing checks is not a rule.
///
/// **Why this was invisible to the tests that existed.** Four unrelated gates refuse before
/// archival moves anything: the version-ordering gate, the absent-`specification.html` gate, the
/// example-identifier gate, and the placeholder-summary gate — the last two share
/// `review/example-content` but fire on different content. A half-built runbook bounces off all
/// four, so probing with one looks safe. Only a *complete* runbook reaches the move, which is
/// exactly the runbook a project has once this change ships.
///
/// The count was three until the regression test was run against the pre-fix code and its failure
/// read: refused for the placeholder summary. A test that passes against the defect for an
/// unrelated reason is worse than no test, so the fourth gate is cleared explicitly in both
/// tests.
///
/// Checked on the resolved directory rather than on the argument's spelling, because the two
/// differ: a path can reach the runbooks directory by traversal (`../runbooks/release`) or by
/// being absolute, and comparing the typed string would miss both.
///
/// **`sync` and `status` are guarded too, and neither destroys anything.** The reason is not
/// symmetry. `status` reports a runbook against a change's four-document shape, so it says
/// `absent proposal.html` about a document a runbook never has. `sync` is worse than merely
/// wrong: its report names each requirement no accepted specification holds and directs an agent
/// to merge it — and `runbook-authority` states that a requirement in a runbook's specification
/// is not an obligation of the project. Measured: `sync --change htmlspec/runbooks/release`
/// listed all five of the release runbook's requirements as `absent` and told an agent to merge
/// them. That is the same failure `contract-authority` records about the comparison operation and
/// example content — an obligation nobody wrote entering the contract by the hand of an agent
/// doing as it was told.
///
/// [`Refusal`] is why the message differs per operation: "it is not archived" is the reason for
/// `archive` and a false statement for the other two.
///
/// # The project is the caller's, and this guard fails closed
///
/// **Two bypasses were measured against the version of this function that derived its own
/// project from `directory`, and both destroyed a runbook.** `ProjectRoot::resolve` ascends from
/// the path it is given and returns the *first* ancestor holding a child `htmlspec/`, including
/// the directory itself:
///
/// - A `htmlspec/` **inside** the runbook made the runbook resolve as its own project, so
///   `runbooks()` became `<runbook>/htmlspec/runbooks`, which does not exist. The guard read that
///   as "not a runbook" and archival moved it.
/// - A `.git` inside the runbook — a submodule, or a vendored repository a runbook ships as an
///   example — hit `resolve`'s repository-boundary stop, so no project was found at all. Same
///   result.
///
/// So `project` is now a parameter: every caller has already resolved the project it is operating
/// on, and rediscovering one from the argument asks a question that has a wrong answer available.
/// The `--store` arm passes the store's project, which is what it resolved to compose the
/// candidate path in the first place.
///
/// **And the unknowable cases now refuse rather than proceed.** Each early return here used to be
/// `Ok(())`, meaning "not a runbook, carry on", on three conditions that actually mean *I could
/// not tell*: no project, a directory that would not canonicalize, a `runbooks/` that would not
/// canonicalize. This function exists to prevent an irreversible `rename`, so an I/O error it
/// cannot interpret must not read as permission. A broken symlink at `htmlspec/runbooks`, or a
/// parent that denies traversal, previously failed open into destruction.
///
/// The one case that legitimately proceeds is a project with no `runbooks/` directory at all —
/// bootstrapped before runbooks existed — and that is distinguished by asking whether the path
/// *exists* rather than by whether canonicalizing it failed.
///
/// **The `..` defect below and these two are one root cause seen twice: the guard trusted a path
/// it derived instead of the project it was given.**
fn refuse_a_runbook(
    project: &ProjectRoot,
    directory: &Path,
    refusal: Refusal,
) -> Result<(), String> {
    let runbooks = project.runbooks();
    // The one honest "proceed": this project has no runbooks directory at all, so nothing can be
    // inside it. A project bootstrapped before runbooks existed takes this route.
    //
    // **`symlink_metadata` rather than `exists`, and this is a third fail-open path found by
    // probing rather than by review.** `Path::exists` follows symlinks and reports `false` for a
    // broken one, so a symlink at `htmlspec/runbooks` pointing nowhere read as "no runbooks
    // directory" and the guard proceeded — measured, on the first build of this fix. Something is
    // there; what it points at cannot be read, which is the case that must refuse.
    match std::fs::symlink_metadata(&runbooks) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "cannot read {} to determine whether {} is a runbook: {error}. Refusing rather \
                 than proceeding, because a runbook archived is a procedure destroyed",
                runbooks.display(),
                directory.display()
            ));
        }
        Ok(_) => {}
    }
    // **Canonicalized rather than component-collected, and the difference is a measured
    // defect.** `Path::components` drops `.` and *keeps* `..`, so the first version of this
    // guard compared `…/htmlspec/changes/../runbooks/release` against `…/htmlspec/runbooks` and
    // found no prefix — the traversal route destroyed the runbook with the guard in place. Only
    // `canonicalize` resolves `..`, and it is available here because the directory exists: every
    // caller has already tested `is_dir`.
    let resolved = directory.canonicalize().map_err(|error| {
        format!(
            "cannot determine whether {} is a change or a runbook: {error}. Refusing rather \
             than proceeding, because this operation would move the directory and a check that \
             cannot read its subject must not stand in for one that can",
            directory.display()
        )
    })?;
    let runbooks = runbooks.canonicalize().map_err(|error| {
        format!(
            "cannot read {} to determine whether {} is a runbook: {error}. Refusing rather than \
             proceeding, because a runbook archived is a procedure destroyed",
            project.runbooks().display(),
            directory.display()
        )
    })?;
    if !resolved.starts_with(&runbooks) {
        return Ok(());
    }
    // The message says what the caller probably meant, because the two operations a contributor
    // reaches for here are the two named. "Not a change" alone leaves them with a refusal and
    // nowhere to go.
    Err(format!(
        "{} is a runbook, not a change. {} To see what the project holds, run `htmlspec list \
         --runbooks`; to check the runbook's documents, run `htmlspec validate {}`",
        directory.display(),
        refusal.reason(),
        directory.display()
    ))
}

/// Why an operation refuses a runbook, which differs per operation.
///
/// One reason per operation rather than one shared sentence, because the reasons are genuinely
/// different and a shared one would be false for two of the three. `archive` would destroy the
/// runbook; `sync` would direct a merge the contract forbids; `status` would report it against a
/// shape it does not have. A refusal whose stated reason a contributor can check and find wrong
/// is worse than a vaguer one, because it makes the refusal itself look unreliable — which is the
/// finding the previous change recorded at its task 5.1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Refusal {
    /// Archival, which moves the directory.
    Archival,
    /// The comparison against the accepted specifications.
    Comparison,
    /// The per-change report.
    Report,
    /// The view of a change's modified requirements beside the accepted ones.
    View,
}

impl Refusal {
    /// The sentence stating why this operation refuses.
    fn reason(self) -> &'static str {
        match self {
            Self::Archival => {
                "A runbook holds a procedure that is run repeatedly, so it is not archived — \
                 archival ends a change, and a procedure ended after its first run is a \
                 procedure destroyed."
            }
            Self::Comparison => {
                "A runbook's specification states what that runbook requires of a run rather \
                 than what the project requires of itself, so comparing it against the accepted \
                 specifications would report requirements to merge that belong in no accepted \
                 specification."
            }
            Self::Report => {
                "This reports a change's four documents and how much of its work is done, and a \
                 runbook has two documents whose steps stay unstarted because a run advances a \
                 copy."
            }
            Self::View => {
                "A runbook's specification states what that runbook requires of a run, so no \
                 accepted specification holds its requirements, and there is nothing to show \
                 beside them."
            }
        }
    }
}

/// Resolves the change directory a contributor named.
///
/// A path that is a directory means that directory. Otherwise the argument is taken as
/// a change *name* and looked up where the project keeps its changes, because a change's
/// location is now derived from the project rather than typed out. Both forms are
/// accepted: an agent scripting against a path should not break, and a contributor
/// typing `htmlspec archive add-dark-mode` from anywhere in the project should work.
///
/// A directory that is a runbook is refused by [`refuse_a_runbook`], on both routes.
///
/// **The project is resolved before either route rather than inside the name route**, because the
/// guard needs the caller's project and the path route used to reach a decision without one. That
/// ordering is what let a `htmlspec/` or a `.git` inside a runbook decide which project the guard
/// was judging against; see [`refuse_a_runbook`] for the two measured bypasses.
///
/// Resolution failing is not an error on the path route. A directory of documents outside any
/// project is still a change directory by the only test available, which is the behaviour that
/// predates runbooks — and a project with no `runbooks/` cannot contain one anyway.
fn resolve_change(target: &Path, refusal: Refusal) -> Result<PathBuf, String> {
    let project = ProjectRoot::resolve(Path::new("."));
    if target.is_dir() {
        if let Ok(project) = &project {
            refuse_a_runbook(project, target, refusal)?;
        }
        return Ok(target.to_owned());
    }
    if target.exists() {
        return Err(format!(
            "{} is not a change directory. A change is a directory of kinded documents, \
             so archival takes the directory rather than one file",
            target.display()
        ));
    }

    // Not a path that exists, so treat it as a name within the project.
    let project = project.map_err(|error| {
        format!("{error}. Archival takes a change directory, or a change name inside a project")
    })?;
    let candidate = project.change(&target.to_string_lossy());
    if candidate.is_dir() {
        // Checked on this route too, although `project.change` composes under `changes/`. A
        // name carrying a traversal — `../runbooks/release` — lands inside `runbooks/` through
        // this branch, so the guard belongs on the resolved directory rather than on the branch
        // that looked safe.
        refuse_a_runbook(&project, &candidate, refusal)?;
        return Ok(candidate);
    }
    Err(format!(
        "no change named {} in {}, and no directory at {}",
        target.display(),
        project.changes().display(),
        target.display()
    ))
}

/// The change to archive, where none was named.
///
/// One active change is the answer and is announced, because a project with one change in
/// flight is the ordinary case and making a contributor type its name teaches nothing. Several
/// are listed rather than chosen between: archival moves a directory and rewrites a contract,
/// and guessing which one would be a guess about the wrong thing to guess about.
fn select_change<O: Write, E: Write>(streams: &mut Streams<O, E>) -> Result<PathBuf, String> {
    let project = ProjectRoot::resolve(Path::new(".")).map_err(|error| {
        format!("{error}. Archival takes a change directory, or a change name inside a project")
    })?;
    let changes = active_changes(&project);
    match changes.as_slice() {
        [] => Err(format!(
            "no active changes in {}. Create one with `htmlspec new change <name>`",
            project.changes().display()
        )),
        [only] => {
            let _ = streams.progress(&format!("archiving {only}, the only active change"));
            Ok(project.change(only))
        }
        several => Err(format!(
            "several active changes: {}. Name the one to archive",
            several.join(", ")
        )),
    }
}

/// A change, resolved within a selected project where one was named.
///
/// The `Some` arm is a genuinely different path from [`resolve_change`] rather than a wrapper
/// around it, so it needs [`refuse_a_runbook`] of its own. Measured: with a store registered,
/// `htmlspec --store <id> status --change ../runbooks/release` reported on the runbook, because
/// `project.change` composes the name under `changes/` and a name carrying a traversal walks
/// straight back out of it.
fn resolve_change_in(
    store: Option<&str>,
    target: &Path,
    refusal: Refusal,
) -> Result<PathBuf, String> {
    match store {
        None => resolve_change(target, refusal),
        Some(_) => {
            let project = crate::store::project_for(store)?;
            let candidate = project.change(&target.to_string_lossy());
            if candidate.is_dir() {
                // The store's project, which is the one that composed `candidate`. Passing it
                // rather than letting the guard rediscover one is the whole of the second fix:
                // the guard was deriving a project from the argument, and a `htmlspec/` inside a
                // runbook made the runbook its own project.
                refuse_a_runbook(&project, &candidate, refusal)?;
                return Ok(candidate);
            }
            Err(format!(
                "no change named {} in {}",
                target.display(),
                project.changes().display()
            ))
        }
    }
}

/// Registers a project under an identifier.
pub fn store_add<O: Write, E: Write>(
    id: &str,
    path: &Path,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let absolute = match std::fs::canonicalize(path) {
        Ok(absolute) => absolute,
        Err(error) => {
            let _ = streams.failure(&format!("cannot read {}: {error}", path.display()));
            return Outcome::OperationalFailure;
        }
    };
    // Checked at registration rather than only at use. A registration that was wrong when it
    // was made is a typo, and reporting it now costs one line where reporting it later costs a
    // confused debugging session.
    if ProjectRoot::resolve(&absolute).is_err() {
        let _ = streams.failure(&format!(
            "{} does not hold a project. Establish one there with `htmlspec init` first",
            absolute.display()
        ));
        return Outcome::OperationalFailure;
    }

    let mut registrations = crate::store::read();
    registrations.insert(id.to_owned(), absolute.clone());
    if let Err(error) = crate::store::write(&registrations) {
        let _ = streams.failure(&format!("cannot record the registration: {error}"));
        return Outcome::OperationalFailure;
    }
    let _ = streams.report(&format!("registered {id} at {}\n", absolute.display()));
    Outcome::Success
}

/// Lists the registered projects.
pub fn store_list<O: Write, E: Write>(streams: &mut Streams<O, E>) -> Outcome {
    let registrations = crate::store::read();
    if streams.form().is_structured() {
        let payload = serde_json::json!({
            "stores": registrations
                .iter()
                .map(|(id, path)| serde_json::json!({
                    "id": id,
                    "path": path,
                    "resolves": ProjectRoot::resolve(path).is_ok(),
                }))
                .collect::<Vec<_>>(),
        });
        if let Ok(json) = serde_json::to_string_pretty(&payload) {
            let _ = streams.payload(&json);
        }
        return Outcome::Success;
    }

    if registrations.is_empty() {
        let _ = streams.report("no stores registered on this machine\n");
        return Outcome::Success;
    }
    for (id, path) in &registrations {
        // A registration whose project is gone is listed and marked, not hidden. Hiding it
        // would leave a contributor unable to see the entry they need to remove.
        let mark = if ProjectRoot::resolve(path).is_ok() {
            ""
        } else {
            "  (unresolvable)"
        };
        let _ = streams.report(&format!("{id}\t{}{mark}\n", path.display()));
    }
    Outcome::Success
}

/// Removes a registration, leaving the project it named untouched.
pub fn store_remove<O: Write, E: Write>(id: &str, streams: &mut Streams<O, E>) -> Outcome {
    let mut registrations = crate::store::read();
    if registrations.remove(id).is_none() {
        let _ = streams.failure(&format!("no store registered as {id}"));
        return Outcome::OperationalFailure;
    }
    if let Err(error) = crate::store::write(&registrations) {
        let _ = streams.failure(&format!("cannot record the removal: {error}"));
        return Outcome::OperationalFailure;
    }
    let _ = streams.report(&format!(
        "removed the registration {id}. The project it named is untouched\n"
    ));
    Outcome::Success
}

/// The documents `init --change` writes, as `(file name, artifact kind)`.
///
/// A proposal, a design, and a tasks document. The **specification is absent on
/// purpose**: it states the contract the change established, and at the moment a
/// change is created nothing has been established. Writing an empty one would offer
/// a contract to be filled in before the work that decides it — and the specs put its
/// production at archival, from the completed change.
///
/// Each document comes from its own kind's template, including the design — whose
/// template is deliberately almost empty. I first wrote the design from the proposal
/// template, and running it showed the mistake: the output announced "created a
/// proposal" for a design, and it imposed a proposal's headings on the one kind whose
/// structure is explicitly the author's.
const CHANGE_DOCUMENTS: &[(&str, &str)] = &[
    ("proposal.html", "proposal"),
    ("design.html", "design"),
    ("tasks.html", "tasks"),
];

pub use crate::project::CHANGE_SPECIFICATION;

/// Checks a change's specification artifact, which is archival's gate.
///
/// **This is the whole of what the program contributes to archival, and the narrowness
/// is the design.** An agent produces the specification from the completed change,
/// following stated instructions, and this checks what the agent wrote. Everything
/// else in the archival sequence is instructions.
///
/// The division exists because the mapping is judgment rather than transformation. A
/// proposal and a design may be structured any way their authors chose, so no rule can
/// locate the obligation inside them: deciding which paragraph states one, what its
/// identifier is, and which scenarios express it is reading rather than parsing. An
/// extractor that guessed would emit a specification that satisfies every check and
/// misstates its source, which is worse than producing nothing because it looks
/// authoritative.
///
/// So this command reads no proposal and no design. It resolves one file, validates
/// it, and refuses while it reports an error. `the_archive_check_reads_only_the_
/// specification` pins that it stays that way.
pub fn archive<O: Write, E: Write>(
    target: Option<&Path>,
    mode: ArchiveMode,
    assume_yes: bool,
    streams: &mut Streams<O, E>,
) -> Outcome {
    // An interrupted publication is settled before this one is accepted, and in `--check` too: a
    // readiness report that ignored a half-finished transaction would be reporting on a project
    // whose state it had not accounted for.
    //
    // **Settled before the change is looked up**, because an interruption past the commit point has
    // already moved the change out of `changes/`. Looking it up first made the re-run that is meant
    // to settle the archival fail with "no change named …", and recovery never ran.
    let recovery_root = target
        .filter(|target| target.exists())
        .unwrap_or(Path::new("."));
    let recovered = match recover_publication_for(recovery_root, streams) {
        Ok(recovered) => recovered,
        Err(outcome) => return outcome,
    };
    // An unnamed archival re-run after an interruption is the re-run that settles it. Selecting some
    // other change to archive next would be doing something nobody asked for.
    if target.is_none() && matches!(recovered, Recovery::Completed { .. }) {
        return Outcome::Success;
    }

    let resolved = match target {
        Some(target) => resolve_change(target, Refusal::Archival),
        None => select_change(streams),
    };
    let directory = match (resolved, &recovered) {
        (Ok(directory), _) => directory,
        // The change this invocation named is the one recovery just finished archiving, so what was
        // asked for is done. Reporting it as absent would send the contributor looking for a change
        // that is exactly where archival puts one.
        (Err(_), Recovery::Completed { change }) if names_the_change(target, change) => {
            return Outcome::Success;
        }
        (Err(report), _) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };
    let directory = directory.as_path();

    // The version gate, checked before the specification is read. A change whose version
    // has not advanced is refused whatever its specification says, so reporting the
    // specification's diagnostics first would send a contributor to fix documents when the
    // thing blocking them is one line in `spec.nix`.
    //
    // This reads and compares. It never writes: advancing the version is the contributor's
    // or the agent's act, on the same division archival already uses for the specification
    // itself. `the_archive_check_does_not_write` pins that.
    if let Some(outcome) = check_version(streams) {
        return outcome;
    }

    // Resolved after the gate, which already reported an absent project in its own terms.
    let project = match ProjectRoot::resolve(Path::new(".")) {
        Ok(project) => project,
        Err(error) => {
            let _ = streams.failure(&format!(
                "{error}. Archival needs the project it archives into"
            ));
            return Outcome::OperationalFailure;
        }
    };

    // The runbook release gate, checked beside the project version gate and for the same reason: a
    // runbook whose canonical content moved without its version moving is refused whatever the
    // specification says, so reporting the specification's diagnostics first would send a
    // contributor to fix documents when the thing blocking them is a line in a runbook's `VERSION`.
    //
    // **Readiness-only archival reaches this too**, which `project-bootstrap` requires — "Archival
    // SHALL apply the same comparison and write nothing" — and it does because the gate runs before
    // the mode is consulted.
    let staged = match runbook_releases(&project, streams) {
        Ok(staged) => staged,
        Err(outcome) => return outcome,
    };

    let path = directory.join(CHANGE_SPECIFICATION);
    if !path.exists() {
        let _ = streams.failure(&format!(
            "{} has no {CHANGE_SPECIFICATION}. Archival produces it from the completed \
             change — its proposal, design, tasks, and the implemented result — and that \
             production is an agent's work following stated instructions rather than \
             something this program derives",
            directory.display()
        ));
        return Outcome::OperationalFailure;
    }

    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            let _ = streams.failure(&format!("cannot read {}: {error}", path.display()));
            return Outcome::OperationalFailure;
        }
    };

    // The declared kind is checked before the rules, because a specification that
    // declares itself something else is a different mistake from an invalid one, and
    // the `kind/*` rules would not report it: they check what a specification carries,
    // not that this file is one.
    let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);
    let declared = projection.model.kind.as_deref();
    if declared != Some("specification") {
        let _ = streams.failure(&format!(
            "{} declares {} rather than the specification kind. A change's specification \
             artifact is what survives as contract, and archival applies its contract \
             changes to the project's accepted specifications",
            path.display(),
            match declared {
                Some(kind) => format!("the {kind} kind"),
                None => String::from("no kind"),
            }
        ));
        return Outcome::OperationalFailure;
    }

    let report = validate_source(&source, &path, &mut Reach::default(), Strictness::Default);
    let _ = streams.report(&format!("{}\n", path.display()));
    let _ = streams.report(&indent(&htmlspec_validate::render_human(&report)));

    if !report.is_accepted() {
        // Refused rather than reported and continued. A change archived with an
        // invalid specification would leave the accepted contract in a state the
        // project's own tools reject.
        let _ = streams.failure(
            "archival does not proceed while the specification reports an error. \
             Reconcile the diagnostics above and run this again",
        );
        return Outcome::DocumentViolation;
    }

    // Example content refuses here rather than at validation, and the two severities are the
    // whole point. `review/example-content` warns, because `htmlspec new document` writes the
    // template and the contract requires that result to be valid — an error there would report a
    // defect the program itself introduced. Archival is where the change's contract enters the
    // project's accepted specifications, so `req-example` reaching this point is an obligation
    // nobody wrote about to become one.
    //
    // A placeholder summary refuses here on the same reasoning and for a second cost: an archived
    // change is the project's permanent record, and one archived with the template's masthead puts
    // "Replace this paragraph" into it. That has already happened once in this repository, which
    // this change's evidence records.
    //
    // Read from the projection this function already made, and matched against what the templates
    // carry. Nothing is recomputed and no list is restated.
    let kept = example_content_in(&projection);
    if !kept.is_empty() {
        // Named individually, because the fix is to replace one unit at a time and a
        // contributor told only "this specification carries example content" has to find it.
        let named = kept.named();
        // The reason is the consequence of what was actually found. Stating the requirement
        // reason for a document whose only fault is its summary would name a defect it does not
        // have, and a contributor who checked would find the reason wrong and the refusal
        // suspect.
        let because = if kept.identifiers.is_empty() {
            "A template's placeholder summary states nothing about what the document is for, so \
             archiving it would put a contract into the project's permanent record with no \
             statement of what it is for."
        } else {
            "An example requirement states no obligation, so archiving one would put an \
             obligation nobody wrote into the project's accepted specifications."
        };
        let _ = streams.failure(&format!(
            "archival does not proceed while the specification carries a template's example \
             content: {}. {because} Replace {} with the change's own, and run this again",
            named.join(", "),
            if named.len() == 1 {
                "it"
            } else {
                "each of them"
            },
        ));
        return Outcome::DocumentViolation;
    }

    // The other documents are reported but not gated. They survive archival — a
    // reader must be able to open an archived change and find the argument that
    // motivated it, the reasoning that shaped it, and the work that delivered it —
    // and one of them being mid-edit is not a reason to refuse the contract.
    let mut absent: Vec<&str> = Vec::new();
    for (name, _kind) in CHANGE_DOCUMENTS {
        if !directory.join(name).exists() {
            absent.push(name);
        }
    }
    if !absent.is_empty() {
        let _ = streams.progress(&format!(
            "note: this change has no {}. Archival keeps every document it has, so the \
             argument and the reasoning stay recoverable alongside the contract",
            absent.join(" and no ")
        ));
    }

    // The gate is satisfied. What follows is the work the program used to leave undone.
    let findings = findings_for(&project, directory, &projection);

    if mode == ArchiveMode::Check {
        report_findings(&findings, streams);
        // The summary names the condition a contributor would otherwise have to read the notes
        // above to find. The predecessor comes first for the reason `question_for` puts it
        // first: it is the finding that cannot be repaired after archival.
        let _ = streams.report(
            match (
                findings.is_empty(),
                findings.out_of_order(),
                findings.needs_sync(),
            ) {
                (true, _, _) => "ready: archival would archive this change\n",
                (false, true, _) => {
                    "ready with questions: this change follows a change that is not archived, \
                     and archival would report the findings above and ask\n"
                }
                (false, false, true) => {
                    "ready with questions: this change has not been synced, and archival would \
                     report the findings above and ask\n"
                }
                (false, false, false) => {
                    "ready with questions: archival would report the findings above \
                               and ask\n"
                }
            },
        );
        emit_archive_payload(streams, &findings, false, None);
        return Outcome::Success;
    }

    report_findings(&findings, streams);
    let asked = if decision(&findings, assume_yes, None) == Decision::Ask {
        ask(&findings, streams)
    } else {
        None
    };
    match decision(&findings, assume_yes, asked) {
        Decision::Proceed => {}
        Decision::Ask | Decision::Stop => {
            if asked == Some(false) {
                let _ = streams.failure(&format!(
                    "archival stopped at your request. Nothing moved, and the project is as \
                     it was{}",
                    // The next step, named where there is one. The predecessor comes first
                    // because archiving it is what makes this archival correct rather than
                    // merely permitted.
                    match (&findings.unresolved_predecessor, findings.needs_sync()) {
                        (Some(follows), _) => format!(
                            ". Archive {} first, or state that this change no longer follows it",
                            follows.predecessor
                        ),
                        (None, true) => format!(
                            ". Run `htmlspec sync --change {}` to see what has not reached the \
                             accepted specifications",
                            findings.change
                        ),
                        (None, false) => String::new(),
                    }
                ));
                emit_archive_payload(streams, &findings, false, None);
                return Outcome::Cancelled;
            }
            let _ = streams.failure(
                "archival found the above and cannot ask, because nothing here can answer. \
                 Pass --yes to archive over these findings, or resolve them",
            );
            emit_archive_payload(streams, &findings, false, None);
            return Outcome::OperationalFailure;
        }
    }

    perform(&project, directory, streams, &findings, &staged)
}

/// Settles an interrupted publication, resolving the project from `from`.
///
/// Split from [`recover_publication`] because recovery has to run before archival resolves a project
/// of its own, and a caller that had to resolve one to ask the question would resolve it twice. No
/// project is no journal: there is nowhere one could have been written.
fn recover_publication_for<O: Write, E: Write>(
    from: &Path,
    streams: &mut Streams<O, E>,
) -> Result<Recovery, Outcome> {
    match ProjectRoot::resolve(from) {
        Ok(project) => recover_publication(&project, streams),
        Err(_) => Ok(Recovery::Nothing),
    }
}

/// What settling an interrupted publication did, where it could settle one.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Recovery {
    /// There was no interrupted publication.
    Nothing,
    /// The interruption came before the move; what it staged is removed and the change is active.
    RolledBack,
    /// The interruption came after the move; the change is archived.
    Completed {
        /// Where the archived change was active, as the journal recorded it.
        change: PathBuf,
    },
}

/// Whether an invocation's target names the change a completed recovery just archived.
///
/// By the change's name, which is all that survives the move — the path the contributor typed no
/// longer exists.
fn names_the_change(target: Option<&Path>, change: &Path) -> bool {
    target
        .and_then(Path::file_name)
        .is_some_and(|name| Some(name) == change.file_name())
}

/// Where an in-flight archival records what it owns.
///
/// Hidden, at the planning root, and named for what it is: a journal of one transaction rather than a
/// lock. Nothing consults it to decide whether another archival may proceed — the archive destination
/// being vacant is what decides that — and it exists so an interrupted publication can be settled
/// rather than guessed at.
fn journal_path(project: &ProjectRoot) -> PathBuf {
    project.htmlspec().join(".archive-journal.json")
}

/// What one archival is about to write, recorded before it writes any of it.
///
/// **Paths and digests rather than content**, and the split is deliberate: the journal is what lets a
/// later run tell "this file is the one that transaction intended" from "somebody edited it", and a
/// digest answers that where a path alone cannot. The content of an inventory is recomputable from the
/// runbooks it describes; the fact that a *particular* set of bytes was intended is not.
///
/// **Every path is absolute, and the constructor is what says so.** Recovery runs from wherever the
/// next command is invoked, and a relative path resolves against that directory rather than the one
/// the interrupted run started in — so a relative journal could roll back files in the wrong place.
/// [`Journal::new`] and [`Journal::read`] both refuse one, which leaves no route to a journal whose
/// paths depend on the working directory.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Journal {
    /// Where the change was.
    change: PathBuf,
    /// Where it was going.
    target: PathBuf,
    /// Each file the transaction owns, and the digest it intended.
    writes: Vec<(PathBuf, crate::runbook_release::ContentDigest)>,
    /// Each of the change's documents the transaction rewrites, and the digest of its rewrite.
    ///
    /// Named by the document rather than by a path, because a replacement is applied in whichever of
    /// the change and the archive holds the change when it is applied: staged beside the document
    /// before the move, and put in its place after it.
    replacements: Vec<Replacement>,
}

/// One of a change's documents, rewritten for the archive, as the journal records it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Replacement {
    /// The document it replaces.
    document: DocumentName,
    /// The digest of the rewritten bytes, staged as [`DocumentName::staged_in`] names.
    digest: crate::runbook_release::ContentDigest,
}

/// A file directly within a change directory, named by its file name alone.
///
/// The parser is what keeps a replacement inside the change: a name holding a separator, naming `.`
/// or `..`, or beginning with a dot is refused, so a journal cannot name a file anywhere else or a
/// staged copy as if it were a document.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DocumentName(String);

impl DocumentName {
    /// `name` as a document in a change, or why it is not one.
    fn parse(name: &str) -> Result<Self, String> {
        let plain = !name.is_empty()
            && !name.starts_with('.')
            && !name.contains(['/', '\\'])
            && Path::new(name).file_name() == Some(std::ffi::OsStr::new(name));
        if plain {
            Ok(Self(name.to_owned()))
        } else {
            Err(format!("{name:?} is not a document in the change"))
        }
    }

    /// The document in `directory`.
    fn in_directory(&self, directory: &Path) -> PathBuf {
        directory.join(&self.0)
    }

    /// Where its rewrite is staged in `directory`: beside it, hidden, and named for the operation.
    fn staged_in(&self, directory: &Path) -> PathBuf {
        directory.join(format!(".{}{STAGED_SUFFIX}", self.0))
    }
}

/// The suffix of a staged document's name, after a leading dot and the document's own name.
const STAGED_SUFFIX: &str = ".archiving";

/// The format a journal declares, so a file of some other shape is refused rather than read.
const JOURNAL_FORMAT: &str = "htmlspec-archive-journal/v2";

/// The format an earlier program wrote, which rewrote no document and so records no replacement.
///
/// Still read, so an archival interrupted under that program is settled by this one. The reverse is
/// refused by the earlier program, which reads only its own format: completing a v2 transaction
/// without its replacements would leave the archive's documents pointing one level short.
const EARLIER_JOURNAL_FORMAT: &str = "htmlspec-archive-journal/v1";

/// A journal exists and cannot be acted on.
///
/// Distinct from an absent journal, which is the ordinary state. A journal that will not read is a
/// transaction nothing can account for, and ignoring it would let the next failure blame some other
/// file for a state this one describes.
#[derive(Clone, Debug, Eq, PartialEq)]
struct UnreadableJournal {
    /// What is wrong with it.
    problem: String,
}

impl Journal {
    /// A journal for one transaction, refusing any path that is not absolute.
    fn new(
        change: PathBuf,
        target: PathBuf,
        writes: Vec<(PathBuf, crate::runbook_release::ContentDigest)>,
        replacements: Vec<Replacement>,
    ) -> Result<Self, String> {
        let relative = std::iter::once(&change)
            .chain(std::iter::once(&target))
            .chain(writes.iter().map(|(path, _)| path))
            .find(|path| !path.is_absolute());
        if let Some(path) = relative {
            return Err(format!(
                "{} is not an absolute path, and a journal is read from whatever directory the next \
                 command runs in",
                path.display()
            ));
        }
        Ok(Self {
            change,
            target,
            writes,
            replacements,
        })
    }

    /// Reads a journal left behind by an interrupted archival, or `None` where there is none.
    fn read(project: &ProjectRoot) -> Result<Option<Self>, UnreadableJournal> {
        let text = match std::fs::read_to_string(journal_path(project)) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(UnreadableJournal {
                    problem: format!("it could not be read: {error}"),
                })
            }
        };
        Self::parse(&text)
            .map(Some)
            .map_err(|problem| UnreadableJournal { problem })
    }

    /// A journal's text as a journal, naming the first thing wrong with it.
    fn parse(text: &str) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|error| format!("it is not JSON: {error}"))?;
        let field = |value: &serde_json::Value, name: &str| -> Result<String, String> {
            value
                .get(name)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("it has no `{name}` string"))
        };
        let format = field(&value, "format")?;
        if format != JOURNAL_FORMAT && format != EARLIER_JOURNAL_FORMAT {
            return Err(format!(
                "it declares the format {format:?} rather than {JOURNAL_FORMAT:?} or \
                 {EARLIER_JOURNAL_FORMAT:?}"
            ));
        }
        let change = PathBuf::from(field(&value, "change")?);
        let target = PathBuf::from(field(&value, "target")?);
        let writes = value
            .get("writes")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| String::from("it has no `writes` list"))?
            .iter()
            .map(|entry| {
                let path = PathBuf::from(field(entry, "path")?);
                let digest = crate::runbook_release::ContentDigest::parse(&field(entry, "digest")?)
                    .map_err(|error| format!("{} has no usable digest: {error}", path.display()))?;
                Ok((path, digest))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let replacements = if format == EARLIER_JOURNAL_FORMAT {
            Vec::new()
        } else {
            value
                .get("replacements")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| String::from("it has no `replacements` list"))?
                .iter()
                .map(|entry| {
                    let document = DocumentName::parse(&field(entry, "document")?)?;
                    let digest =
                        crate::runbook_release::ContentDigest::parse(&field(entry, "digest")?)
                            .map_err(|error| {
                                format!("{:?} has no usable digest: {error}", document.0)
                            })?;
                    Ok(Replacement { document, digest })
                })
                .collect::<Result<Vec<_>, String>>()?
        };
        Self::new(change, target, writes, replacements)
    }

    /// Writes the journal. Exclusive, because two transactions cannot own one set of paths.
    fn publish(&self, project: &ProjectRoot) -> Result<(), String> {
        let writes: Vec<serde_json::Value> = self
            .writes
            .iter()
            .map(|(path, digest)| serde_json::json!({ "path": path, "digest": digest.as_str() }))
            .collect();
        let replacements: Vec<serde_json::Value> = self
            .replacements
            .iter()
            .map(|replacement| {
                serde_json::json!({
                    "document": replacement.document.0,
                    "digest": replacement.digest.as_str(),
                })
            })
            .collect();
        let document = serde_json::json!({
            "format": JOURNAL_FORMAT,
            "change": self.change,
            "target": self.target,
            "writes": writes,
            "replacements": replacements,
        });
        let rendered = serde_json::to_string_pretty(&document)
            .map_err(|error| format!("the journal could not be written: {error}"))?;
        publish_without_replacement(&journal_path(project), &format!("{rendered}\n"))
    }

    /// Removes the journal, which is what a completed transaction does last.
    fn clear(&self, project: &ProjectRoot) {
        let _ = std::fs::remove_file(journal_path(project));
    }
}

/// Settles a publication an earlier run left unfinished, before this one is accepted.
///
/// **Completes where the move happened and rolls back where it did not, and refuses where it cannot
/// tell.** The move is the commit point, so which side of it the interruption fell on is answerable
/// from the filesystem: the target exists or the change does. That is what makes recovery possible
/// without a log of steps, and it is why the transaction is ordered the way it is.
///
/// A file this transaction owns holding bytes it did not intend is a conflicting edit, and recovery
/// refuses rather than choosing — completing would publish content nobody compared, and rolling back
/// would destroy somebody's work.
fn recover_publication<O: Write, E: Write>(
    project: &ProjectRoot,
    streams: &mut Streams<O, E>,
) -> Result<Recovery, Outcome> {
    let journal = match Journal::read(project) {
        Ok(Some(journal)) => journal,
        Ok(None) => return Ok(Recovery::Nothing),
        Err(unreadable) => {
            let _ = streams.failure(&format!(
                "an interrupted archival left a journal at {} that cannot be acted on: {}. It \
                 records what that archival owned, and without it nothing here can tell a staged \
                 file from somebody's work: read it, settle the change it names by hand, delete \
                 it, and run this again",
                journal_path(project).display(),
                unreadable.problem
            ));
            return Err(Outcome::OperationalFailure);
        }
    };
    let moved = journal.target.is_dir();
    let still_active = journal.change.is_dir();

    if moved && still_active {
        let _ = streams.failure(&format!(
            "an interrupted archival left a journal at {}, and both {} and {} exist. Nothing here \
             can say which of the two holds the change, so this will not choose: look at both, \
             remove the one that is not the change, delete the journal, and run this again",
            journal_path(project).display(),
            journal.change.display(),
            journal.target.display()
        ));
        return Err(Outcome::OperationalFailure);
    }

    // The rewritten documents are looked for in whichever directory holds the change. A staged copy
    // still there is the transaction's; after the move, one already put in place has made its
    // document the file the transaction owns.
    //
    // **Before the move, a staged copy is the transaction's whatever it holds**, so its digest is
    // not asked about. Nothing but archival writes a `.<name>.archiving` file, it is derived from a
    // document still in place and untouched, and a run killed while writing it leaves part of the
    // rewrite: refusing that as somebody's edit would send the contributor to delete archival's own
    // file on a false premise, and removing it loses nothing. After the move the copy is about to be
    // published, every copy was written whole before the move could happen, and a digest that
    // disagrees is an edit, which is refused.
    let holding = if moved {
        &journal.target
    } else {
        &journal.change
    };
    let mut staged: Vec<(PathBuf, &Replacement)> = Vec::new();
    let mut owned: Vec<(PathBuf, &crate::runbook_release::ContentDigest)> = journal
        .writes
        .iter()
        .map(|(path, digest)| (path.clone(), digest))
        .collect();
    for replacement in &journal.replacements {
        let copy = replacement.document.staged_in(holding);
        if copy.is_file() {
            if moved {
                owned.push((copy.clone(), &replacement.digest));
            }
            staged.push((copy, replacement));
        } else if moved {
            owned.push((
                replacement.document.in_directory(holding),
                &replacement.digest,
            ));
        }
    }

    for (path, digest) in &owned {
        let Ok(contents) = std::fs::read(path) else {
            continue;
        };
        if crate::runbook_release::ContentDigest::of(&contents) != **digest {
            let _ = streams.failure(&format!(
                "an interrupted archival left a journal at {} naming {}, and that file does not \
                 hold what the transaction intended. Somebody has edited it since, so completing \
                 would publish content nobody compared and rolling back would destroy their work: \
                 read the file, settle it, delete the journal, and run this again",
                journal_path(project).display(),
                path.display()
            ));
            return Err(Outcome::OperationalFailure);
        }
    }

    if moved {
        // Past the commit point: the change is archived, so what remains is finishing the writes.
        // Every write this transaction owns is inside the archive it moved, and the digest check
        // above established that what landed is what was intended. Each rewritten document still
        // staged takes its document's place, which is the step the interruption may have cut short.
        for (copy, replacement) in &staged {
            let document = replacement.document.in_directory(holding);
            if let Err(error) = std::fs::rename(copy, &document) {
                let _ = streams.failure(&format!(
                    "an interrupted archival left a journal at {}, and {} could not take the place \
                     of {}: {error}. The change is archived and the journal is kept, so running this \
                     again completes it",
                    journal_path(project).display(),
                    copy.display(),
                    document.display()
                ));
                return Err(Outcome::OperationalFailure);
            }
        }
        journal.clear(project);
        let _ = streams.progress(&format!(
            "recovered an interrupted archival: {} is archived and everything the transaction \
             owned is in place",
            journal.target.display()
        ));
        return Ok(Recovery::Completed {
            change: journal.change,
        });
    }

    // Before the commit point: the change never moved, so everything the transaction staged is
    // removed and the change is left exactly as it was. Its documents were never touched: a rewrite
    // replaces one only after the move.
    for (path, _) in &journal.writes {
        let _ = std::fs::remove_file(path);
    }
    for (copy, _) in &staged {
        let _ = std::fs::remove_file(copy);
    }
    journal.clear(project);
    let _ = streams.progress(&format!(
        "rolled back an interrupted archival: {} is still an active change and the {} file(s) that \
         were staged for it are removed",
        journal.change.display(),
        journal.writes.len() + staged.len()
    ));
    Ok(Recovery::RolledBack)
}

/// What archival publishes about its runbooks, staged before anything moves.
///
/// Staged rather than written here, because the whole of the guarantee is that a refusal leaves the
/// runbook, the previous archive, and the active change unchanged. Computing the inventory before any
/// mutation is what makes that true by ordering rather than by cleanup: there is nothing to undo
/// because nothing was done.
struct StagedReleases {
    /// Every runbook the project holds, compared against what it last published.
    inventory: crate::runbook_release::Inventory,
    /// The adoption marker to publish with it, where the project has none yet.
    ///
    /// `None` once adoption has happened, and never advanced afterwards: the marker names the first
    /// project version subject to inventories, and moving it would turn recorded history into a
    /// legacy exemption.
    adoption: Option<crate::runbook_release::Adoption>,
}

/// Compares every runbook the project holds against the last release it published.
///
/// The gate `project-bootstrap` adds to archival, and the consumer that makes a runbook's version
/// mean something: `runbook-authority` requires the version to identify a release, and a version no
/// gate compares is the "accuracy without authority" failure `establish-a-runbook` argues against.
///
/// **Compares and never edits.** Choosing whether a change earned a patch or a minor increment is
/// judgment, and `artifact-kinds` reserves judgment for an agent — the same division the project
/// version gate above already draws. This reports the comparison; the author advances the version.
fn runbook_releases<O: Write, E: Write>(
    project: &ProjectRoot,
    streams: &mut Streams<O, E>,
) -> Result<StagedReleases, Outcome> {
    let (adoption, snapshots) =
        match crate::runbook_release::history(&project.archive(), &project.runbook_releases()) {
            Ok(history) => history,
            Err(faults) => {
                let _ = streams.failure(&format!(
                "the published runbook history cannot be read, so archival cannot compare against \
                 it. A history this operation cannot read is not a history it may ignore — \
                 skipping a defective snapshot would make the next release look like a first \
                 one:\n{faults}"
            ));
                return Err(Outcome::OperationalFailure);
            }
        };

    let present = crate::runbook_release::present(&project.runbooks());
    let mut faults: Vec<crate::runbook_release::ReleaseFault> = Vec::new();
    let mut releases = Vec::with_capacity(present.len());
    for (name, directory) in &present {
        match crate::runbook_release::release_of(directory, name) {
            Ok(release) => {
                let previous = crate::runbook_release::predecessor(&snapshots, name);
                match crate::runbook_release::compare_release(previous, release.clone()) {
                    Ok(publishable) => releases.push(publishable.release),
                    Err(found) => faults.extend(found.faults().iter().cloned()),
                }
            }
            // A runbook that cannot be read is a fault rather than an absent runbook: skipping it
            // would publish an inventory that silently omits a runbook the project holds.
            Err(found) => faults.extend(found.faults().iter().cloned()),
        }
    }

    if !faults.is_empty() {
        let _ = streams.failure(&format!(
            "archival does not proceed while a runbook's release does not follow the one before \
             it. Advancing a version is your work or your agent's; this command compares and never \
             edits:\n{}",
            crate::runbook_release::ReleaseFaults::one(faults[0].clone())
        ));
        for fault in &faults[1..] {
            let _ = streams.failure(&fault.to_string());
        }
        return Err(Outcome::DocumentViolation);
    }

    let inventory = match crate::runbook_release::Inventory::of(releases) {
        Ok(inventory) => inventory,
        Err(found) => {
            let _ = streams.failure(&found.to_string());
            return Err(Outcome::OperationalFailure);
        }
    };

    // The first inventory carries the marker with it, so a history either has both or neither. The
    // marker is what stops somebody deleting the first inventory and turning recorded history into a
    // legacy exemption.
    let declared = crate::declaration::read(&project.declaration())
        .map(|declaration| crate::version::ProjectVersion::from(declaration.version));
    let adoption = match (adoption, declared) {
        (Some(_), _) => None,
        (None, Ok(version)) => Some(crate::runbook_release::Adoption { version }),
        // An undeclarable project version is already refused by the version gate above, so this arm
        // reports rather than invents: a marker naming no version would be a history with no
        // beginning.
        (None, Err(error)) => {
            let _ = streams.failure(&error.to_string());
            return Err(Outcome::OperationalFailure);
        }
    };

    if !inventory.is_empty() || adoption.is_some() {
        let _ = streams.progress(&format!(
            "staging {} runbook release(s){}",
            inventory.len(),
            match adoption {
                Some(marker) => format!(
                    ", adopting publication at project version {}",
                    marker.version
                ),
                None => String::new(),
            }
        ));
    }
    Ok(StagedReleases {
        inventory,
        adoption,
    })
}

/// The project's active changes, in name order.
///
/// The archive is excluded, and it lives beneath `changes/` — so an enumeration that listed
/// directories would list it as a change named `archive`. Every caller of this wants the
/// changes that can still be worked on.
pub fn active_changes(project: &ProjectRoot) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(project.changes()) else {
        return Vec::new();
    };
    let mut found: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != "archive")
        .collect();
    found.sort();
    found
}

/// One active change in the enumeration: its name, and how much of its work is done.
///
/// The completion is `Option` because a change whose tasks document is unwritten has none, and
/// the distinction has to survive as far as the report. Collapsing it to a zeroed count here
/// would make `0/0` the only thing either output could say.
struct Listed {
    /// The change's directory name, which is how every other operation names it.
    name: String,
    /// What its tasks document states, or `None` when it has none.
    completion: Option<Completion>,
    /// What it follows and how that resolved, or `None` where it states no predecessor.
    ///
    /// `Option` for the same reason `completion` is one, and a different reason too: most changes
    /// follow nothing, and the contract requires that an absent declaration not be reported. A
    /// sentinel predecessor would make "follows nothing" a value the report had to recognise
    /// rather than a case it does not reach.
    follows: Option<crate::sequence::Follows>,
}

/// One runbook in the enumeration: its name, and which of its two documents it holds.
///
/// **The documents are what a selector needs, and they are the reason this is not a bare
/// `String`.** `Listed` carries a completion fraction because that is what distinguishes two
/// changes; a runbook's canonical procedure is `todo` at every moment, so the fraction is the
/// same value forever and says nothing. What distinguishes two runbooks is whether the
/// procedure is there at all — a runbook with no `tasks.html` has none, and a name alone cannot
/// tell a selector that.
struct Runbook {
    /// The directory name, which is the runbook's name and the only place it is recorded.
    name: String,
    /// Which of [`RUNBOOK_DOCUMENTS`] are present, in the order that constant states them.
    ///
    /// A `Vec` of the kinds found rather than two booleans: the pair is what gets printed and
    /// what goes in the payload, and two flags would each need a name at every use site.
    documents: Vec<&'static str>,
    /// The release its `VERSION` declares, or `None` where it has none a version can be read from.
    ///
    /// runbook-authority's "The project enumerates the runbooks it holds" requires each entry to
    /// carry what a selector needs, and a version and a title are what a person chooses by. `None`
    /// rather than a placeholder version, because a runbook predating `VERSION` has no release to
    /// name, and the payload omits the field rather than inventing one.
    version: Option<crate::version::RunbookVersion>,
    /// Its tasks document's title, or its specification's where the tasks document has none.
    title: Option<String>,
}

impl Runbook {
    /// The runbook named `name` in `project`, as an enumeration reports it.
    fn read(project: &ProjectRoot, name: String) -> Self {
        // The directory comes from `ProjectRoot::runbook` rather than from a directory entry's own
        // path: the accessor is the one statement of where a runbook lives, so a location read from
        // an entry would be a second one, and the two would agree until the day the layout moved.
        let directory = project.runbook(&name);
        let documents = RUNBOOK_DOCUMENTS
            .iter()
            .filter(|(file, _kind)| directory.join(file).is_file())
            .map(|(_file, kind)| *kind)
            .collect();
        let version = crate::runbook_release::read_version(&directory).ok();
        // The procedure's title first, because the procedure is what a person runs, and a runbook
        // without one still has a name its specification states.
        let title = ["tasks.html", CHANGE_SPECIFICATION]
            .iter()
            .find_map(|file| document_title(&directory.join(file)));
        Self {
            name,
            documents,
            version,
            title,
        }
    }
}

/// The title a document at `path` projects, or `None` where it is unreadable or has none.
fn document_title(path: &Path) -> Option<String> {
    let source = std::fs::read_to_string(path).ok()?;
    let parsed = htmlspec_parse::parse_document(&source);
    htmlspec_parse::project(&parsed.document)
        .model
        .subject
        .filter(|title| !title.is_empty())
}

/// The two documents a runbook holds, and the artifact kind each declares.
///
/// **The split is forced rather than chosen.** `artifact-kinds` states that a `specification`
/// SHALL NOT carry task units and that this is the only placement rule in the vocabulary, so
/// the procedure cannot live inside the canonical document.
///
/// Spelled here rather than derived from [`CHANGE_DOCUMENTS`] and [`CHANGE_SPECIFICATION`].
/// The file names agree, deliberately — a reader should not learn two vocabularies for
/// documents of the same two kinds — but a runbook's shape is not a change's, and deriving one
/// from the other would couple them for no reason. A change has four documents and a runbook
/// has two.
const RUNBOOK_DOCUMENTS: &[(&str, &str)] = &[
    (CHANGE_SPECIFICATION, "specification"),
    ("tasks.html", "tasks"),
];

/// What an enumeration found.
///
/// A sum type rather than one list of names, because the arms no longer carry the same thing: a
/// change carries its completion, an accepted specification has no tasks to count, and a
/// runbook carries which of its documents exist. Sharing a `Vec<String>` between them would
/// mean the arm that has counts and the arms that cannot have them are indistinguishable to
/// every function downstream.
enum Enumerated {
    /// The project's active changes, in name order.
    Changes(Vec<Listed>),
    /// The project's accepted specifications, by path. Unaffected by what changes carry.
    Specifications(Vec<String>),
    /// The project's runbooks, in name order.
    Runbooks(Vec<Runbook>),
}

impl Enumerated {
    /// The word the reports and the payload name this enumeration by.
    fn label(&self) -> &'static str {
        match self {
            Self::Changes(_) => "changes",
            Self::Specifications(_) => "specifications",
            Self::Runbooks(_) => "runbooks",
        }
    }

    /// Whether the project holds none of what was asked for.
    fn is_empty(&self) -> bool {
        match self {
            Self::Changes(changes) => changes.is_empty(),
            Self::Specifications(names) => names.is_empty(),
            Self::Runbooks(runbooks) => runbooks.is_empty(),
        }
    }
}

/// The project's runbooks, in name order, with the documents each holds.
///
/// **A directory read rather than a document walk**, which is where this differs from the
/// `--specs` arm and the difference is load-bearing. `discover::documents` yields every document
/// beneath a root, so applied here it would return `release/tasks.html` and
/// `release/specification.html` — one runbook presented as two entries. What is being
/// enumerated is the runbook.
///
/// Derived from what the place holds rather than from a list anybody maintains, which is the
/// shape `project-bootstrap` refuses when it forbids reading the last archived version out of a
/// directory name: a value kept beside the facts it summarizes depends on a convention nothing
/// enforces.
///
/// No `archive` filter, unlike [`active_changes`]: nothing but runbooks lives under
/// `runbooks/`.
fn runbooks(project: &ProjectRoot) -> Vec<Runbook> {
    let Ok(entries) = std::fs::read_dir(project.runbooks()) else {
        // An absent directory is a project bootstrapped before this existed, and it holds no
        // runbooks. Reported as none rather than as a failure, for the reason `assets` reports
        // a directory with no store: a contributor is better served by hearing there is
        // nothing than by an error about a directory they never made.
        return Vec::new();
    };
    let mut found: Vec<Runbook> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .map(|name| Runbook::read(project, name))
        .collect();
    found.sort_by(|left, right| left.name.cmp(&right.name));
    found
}

/// The project's runbooks as rows a person chooses between: name, version, documents, and title.
///
/// Shared by `list --runbooks` and by `run` named with no runbook, so the two print one table and
/// cannot disagree about what a runbook is called or which release it is at.
fn runbook_rows(runbooks: &[Runbook]) -> Vec<String> {
    let cells: Vec<[String; 4]> = runbooks
        .iter()
        .map(|runbook| {
            [
                printable(&runbook.name),
                // Said rather than left blank, as `no documents` is: a gap reads as a rendering
                // fault, and a runbook with no version is one `run` refuses to start.
                runbook
                    .version
                    .map_or_else(|| String::from("no version"), |version| version.to_string()),
                if runbook.documents.is_empty() {
                    // The column says so rather than being blank, exactly as `no tasks` does for a
                    // change. A directory holding neither document is not a procedure anybody can
                    // run.
                    String::from("no documents")
                } else {
                    runbook.documents.join(", ")
                },
                runbook.title.as_deref().map(printable).unwrap_or_default(),
            ]
        })
        .collect();
    // Widened to each column's widest cell, for the reason the change table already is: a column
    // is read down, and a ragged one is read wrong.
    let widths: [usize; 3] = std::array::from_fn(|column| {
        cells
            .iter()
            .map(|row| row[column].chars().count())
            .max()
            .unwrap_or(0)
    });
    cells
        .iter()
        .map(|[name, version, documents, title]| {
            format!(
                "{name:<name_width$}  {version:<version_width$}  {documents:<documents_width$}  {title}",
                name_width = widths[0],
                version_width = widths[1],
                documents_width = widths[2],
            )
            .trim_end()
            .to_owned()
        })
        .collect()
}

/// The order of the runbooks `list --runbooks` found: each one's prerequisites and their state now.
///
/// Read once per listing, in the listing's order, so the rows, the payload, and the graph show one
/// answer (the change `require-a-run-of-another-runbook`, design decision 8).
fn read_order(project: &ProjectRoot, runbooks: &[Runbook]) -> crate::order::Order {
    let names: Vec<String> = runbooks
        .iter()
        .map(|runbook| runbook.name.clone())
        .collect();
    crate::order::read(&project.runbooks(), &names)
}

/// The runbook rows, each followed by its prerequisites and their state, indented beneath it.
///
/// Shared by `list --runbooks` and a bare `run`, as [`runbook_rows`] is, so a reader sees what can
/// start before choosing, in the same words in both places.
fn runbook_listing(runbooks: &[Runbook], order: &crate::order::Order) -> Vec<String> {
    let mut listing = Vec::new();
    for (row, entry) in runbook_rows(runbooks).into_iter().zip(&order.runbooks) {
        listing.push(row);
        listing.extend(
            crate::order::lines(entry)
                .into_iter()
                .map(|line| format!("  {line}")),
        );
    }
    listing
}

/// What `run` prints when it is named with no runbook: the runbooks it could start, and how.
///
/// The change's design, decision 18. A refusal that names the candidates, as `archive` with no
/// change names the active ones, not a second listing command: runbook-authority's "The project
/// enumerates the runbooks it holds" keeps the listing in `list --runbooks`.
pub fn runbooks_to_start(project: &ProjectRoot) -> String {
    let found = runbooks(project);
    if found.is_empty() {
        return format!(
            "run needs a runbook, and {} holds none. Create one with `htmlspec new runbook <name>`",
            project.runbooks().display()
        );
    }
    let order = read_order(project, &found);
    let rows: Vec<String> = runbook_listing(&found, &order)
        .into_iter()
        .map(|row| format!("  {row}"))
        .collect();
    format!(
        "run needs a runbook. This project's runbooks:\n{}\nStart a run with `htmlspec run \
         <runbook> --as <record>`, or report on a runbook's runs with `htmlspec run <runbook> \
         --check`",
        rows.join("\n")
    )
}

/// One runbook's entry in the `list --runbooks --json` payload.
///
/// `documents` is always present, empty where a runbook directory holds neither. `version` and
/// `title` are present where the runbook has them. `declarations`, `prerequisitesMet`, and
/// `prerequisites` are always present: an empty `prerequisites` beside `readable` is a runbook that
/// requires nothing, and beside `malformed` or `unreadable` one whose requirements are not known.
fn runbook_entry(runbook: &Runbook, ordered: &crate::order::RunbookOrder) -> serde_json::Value {
    let mut entry = serde_json::Map::new();
    entry.insert("name".into(), runbook.name.clone().into());
    entry.insert("documents".into(), runbook.documents.clone().into());
    if let Some(version) = runbook.version {
        entry.insert("version".into(), version.to_string().into());
    }
    if let Some(title) = &runbook.title {
        entry.insert("title".into(), title.clone().into());
    }
    entry.insert("declarations".into(), ordered.declarations.label().into());
    entry.insert("prerequisitesMet".into(), ordered.met().into());
    let prerequisites: Vec<serde_json::Value> = ordered
        .prerequisites
        .iter()
        .map(|prerequisite| {
            let mut declared = serde_json::Map::new();
            declared.insert(
                "requirement".into(),
                prerequisite.requirement.clone().into(),
            );
            declared.insert("runbook".into(), prerequisite.runbook.clone().into());
            if let Some(release) = prerequisite.at_least {
                declared.insert("atLeast".into(), release.to_string().into());
            }
            declared.insert("state".into(), prerequisite.state.kind().label().into());
            declared.insert(
                "satisfiedBy".into(),
                prerequisite.state.satisfied_by().to_vec().into(),
            );
            declared.insert("satisfying".into(), prerequisite.state.satisfying().into());
            declared.insert(
                "openRuns".into(),
                prerequisite.state.open_runs().to_vec().into(),
            );
            serde_json::Value::Object(declared)
        })
        .collect();
    entry.insert("prerequisites".into(), prerequisites.into());
    serde_json::Value::Object(entry)
}

/// Enumerates what a project holds.
///
/// The operation a workflow selects from. Its absence is why the installed HTMLSpec
/// instructions have no change-selection step: an agent asked to archive "the change we were
/// working on" had no way to find out what the candidates were, so the step was written out
/// rather than written badly.
///
/// **It carries each change's completion, because selection is what it is for.** A list of names
/// can be selected from on no basis but order, and the operation is on the path of `archive` —
/// which moves a directory and rewrites a contract — so an enumeration that offered only names
/// asked a contributor to pick blind at the one moment that is hard to undo. The counts come
/// from [`completion`], the same function the completion operation and archival use, so no two
/// of the three can disagree about one change.
pub fn list<O: Write, E: Write>(
    specs: bool,
    runbooks: bool,
    graph: bool,
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
    list_drawn(&project, specs, runbooks, graph, streams)
}

/// What one invocation of the enumeration was asked for.
///
/// Two independent booleans reach the command surface, matching the `--specs` flag that was
/// already there, and they are resolved into this before anything reads them. A sum type at the
/// point of use rather than at the point of parsing: a `bool` pair is representable in states
/// that mean nothing — both set — and this is where that is decided once instead of at each
/// branch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Subject {
    /// The project's active changes. The default, and what an argument-less `list` prints.
    Changes,
    /// The project's accepted specifications.
    Specifications,
    /// The project's runbooks.
    Runbooks,
}

impl Subject {
    /// Which subject a pair of flags names.
    ///
    /// `--runbooks` wins where both are passed. Arbitrary between two equally-wrong readings and
    /// stated rather than left to the order of an `if` chain: a contributor who passed both asked
    /// two questions and gets an answer to one, which is better than an error for a combination
    /// that harms nothing.
    fn of(specs: bool, runbooks: bool) -> Self {
        match (runbooks, specs) {
            (true, _) => Self::Runbooks,
            (false, true) => Self::Specifications,
            (false, false) => Self::Changes,
        }
    }
}

/// [`list_drawn`] without `--graph`, the shape the unit tests below enumerate through.
#[cfg(test)]
fn list_in<O: Write, E: Write>(
    project: &ProjectRoot,
    specs: bool,
    wants_runbooks: bool,
    streams: &mut Streams<O, E>,
) -> Outcome {
    list_drawn(project, specs, wants_runbooks, false, streams)
}

/// Enumerates a resolved project, which is everything [`list`] does after resolution.
///
/// Split out so the enumeration is reachable from a test without registering a store. The
/// registry lives in the contributor's own configuration directory, so a test that registered
/// one would write outside its scratch directory to check a listing.
///
/// `graph` is `--graph`: the runbooks' order drawn as a Mermaid flowchart instead of rows. The
/// command surface accepts it only beside `--runbooks`, so it is read only on the runbook arm. With
/// `--json` it changes nothing: the payload already holds every node, edge, state, and cycle the
/// graph draws (the change `require-a-run-of-another-runbook`, design decision 8).
fn list_drawn<O: Write, E: Write>(
    project: &ProjectRoot,
    specs: bool,
    wants_runbooks: bool,
    graph: bool,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let found = match Subject::of(specs, wants_runbooks) {
        Subject::Specifications => {
            let documents = crate::discover::documents(&project.specs()).unwrap_or_default();
            let mut names: Vec<String> = documents
                .iter()
                .filter_map(|path| path.strip_prefix(project.specs()).ok())
                .map(|path| path.to_string_lossy().into_owned())
                .collect();
            names.sort();
            Enumerated::Specifications(names)
        }
        Subject::Runbooks => Enumerated::Runbooks(runbooks(project)),
        // Name order, established by `active_changes` and preserved here. The order must not
        // follow the completion: a listing that reordered itself as work progressed could not be
        // selected from twice.
        Subject::Changes => Enumerated::Changes(
            active_changes(project)
                .into_iter()
                .map(|name| {
                    let directory = project.change(&name);
                    Listed {
                        completion: completion(&directory),
                        follows: crate::sequence::Follows::read(project, &directory),
                        name,
                    }
                })
                .collect(),
        ),
    };

    if streams.form().is_structured() {
        let payload = match &found {
            Enumerated::Changes(changes) => serde_json::json!({
                "changes": changes
                    .iter()
                    .map(|listed| {
                        let mut entry = serde_json::Map::new();
                        entry.insert("name".into(), listed.name.clone().into());
                        // Omitted rather than zeroed for a change with no tasks document, so a
                        // caller reading the payload cannot mistake an unwritten document for
                        // finished work.
                        if let Some(counted) = &listed.completion {
                            entry.insert("state".into(), counted.state().label().into());
                            entry.insert("total".into(), counted.total().into());
                            entry.insert(
                                "tasks".into(),
                                serde_json::json!({
                                    "todo": counted.todo,
                                    "inProgress": counted.in_progress,
                                    "done": counted.done,
                                    "dropped": counted.dropped,
                                    "undeclared": counted.undeclared,
                                }),
                            );
                            // Beside `total` rather than inside `tasks`, which holds one count
                            // per state and sums to `total`: these tasks are already counted as
                            // `done`, and a subset among the states would make a caller's sum
                            // exceed the total. Present when zero, as `undeclared` is, because
                            // the payload states the fact.
                            entry.insert(
                                "doneDifferently".into(),
                                counted.done_differently.len().into(),
                            );
                        }
                        // Omitted for a change that states no predecessor, like the counts
                        // above: most changes follow nothing, and a null would make a caller
                        // check a field rather than its absence. `resolved` is stated beside
                        // `resolution` because it is the question a selecting agent asks, and
                        // `resolution` keeps the three outcomes distinguishable — which the
                        // boolean alone cannot do.
                        if let Some(follows) = &listed.follows {
                            entry.insert(
                                "follows".into(),
                                serde_json::json!({
                                    "change": follows.predecessor,
                                    "resolution": follows.resolution.label(),
                                    "resolved": follows.resolution.is_resolved(),
                                }),
                            );
                        }
                        serde_json::Value::Object(entry)
                    })
                    .collect::<Vec<_>>(),
                // Project-wide rather than per change, because a cycle is a fact about a set of
                // changes and not about any one of them. Reported once here, where the whole
                // project is in view; archival reports the cycle the change it is archiving sits
                // in. Empty is the ordinary case.
                "followsCycles": crate::sequence::cycles(project),
            }),
            Enumerated::Specifications(names) => serde_json::json!({ "specifications": names }),
            // Keyed by the same word the flag uses, so a caller who read the help does not have
            // to guess at the field. `documents` is always present, empty where a runbook
            // directory holds neither — an empty array is the fact, unlike the omitted
            // completion above, which distinguishes an unwritten document from finished work.
            // `version` and `title` are present where the runbook has them and omitted where it
            // does not, as `completion` is for a change: an absent field says the runbook has no
            // VERSION, where a placeholder would be a version nobody released.
            //
            // `prerequisites`, `declarations`, and `prerequisitesMet` follow the precedent the change
            // listing set for `follows`: a state per edge, kept beside the boolean a selecting agent
            // asks, so "unmet" and "missing" and "cycle" stay distinguishable. `prerequisiteCycles`
            // is project-wide, as `followsCycles` is, because a cycle is a fact about a set of
            // runbooks and not about any one of them.
            Enumerated::Runbooks(runbooks) => {
                let order = read_order(project, runbooks);
                serde_json::json!({
                    "runbooks": runbooks
                        .iter()
                        .zip(&order.runbooks)
                        .map(|(runbook, ordered)| runbook_entry(runbook, ordered))
                        .collect::<Vec<_>>(),
                    "prerequisiteCycles": order.cycles,
                })
            }
        };
        if let Ok(json) = serde_json::to_string_pretty(&payload) {
            let _ = streams.payload(&json);
        }
        return Outcome::Success;
    }

    if graph {
        if let Enumerated::Runbooks(runbooks) = &found {
            // Printed for an empty project too, as the header alone: a document that pastes the
            // output gets a diagram that renders, rather than prose Mermaid cannot read.
            let _ = streams.report(&crate::order::graph(&read_order(project, runbooks)));
            return Outcome::Success;
        }
    }

    if found.is_empty() {
        let _ = streams.report(&format!("no active {}\n", found.label()));
        return Outcome::Success;
    }

    match &found {
        Enumerated::Specifications(names) => {
            for name in names {
                let _ = streams.report(&format!("{name}\n"));
            }
        }
        Enumerated::Runbooks(runbooks) => {
            // No completion fraction, deliberately. A canonical procedure's tasks stay `todo`
            // because nobody works them — a run copies the document and advances the copy — so
            // the column would read `0/9` on every runbook at every moment, and a column that
            // never moves teaches a reader to ignore it. A version and a title are what a person
            // chooses by instead.
            for row in runbook_listing(runbooks, &read_order(project, runbooks)) {
                let _ = streams.report(&format!("{row}\n"));
            }
        }
        Enumerated::Changes(changes) => {
            // The report is read as a column, and a ragged one is read wrong: the fractions have
            // to line up for two changes to be compared at a glance. Both columns are widened to
            // their widest member rather than to a guess, so a long change name pushes the
            // fractions right instead of wrapping the line.
            let widest_name = changes
                .iter()
                .map(|listed| listed.name.chars().count())
                .max()
                .unwrap_or(0);
            let fractions: Vec<String> = changes
                .iter()
                .map(|listed| match &listed.completion {
                    Some(counted) => format!("{}/{}", counted.done, counted.total()),
                    // The column the fraction would occupy says so, rather than being blank:
                    // a gap reads as a rendering fault, and this is a fact about the change.
                    None => String::from("no tasks"),
                })
                .collect();
            let widest_fraction = fractions
                .iter()
                .map(|fraction| fraction.chars().count())
                .max()
                .unwrap_or(0);
            // The state column is widened too, now that something follows it. Left ragged, the
            // predecessors would start at a different column on each row and the two facts
            // would read as one run-on phrase.
            let widest_state = changes
                .iter()
                .map(|listed| match &listed.completion {
                    Some(counted) => counted.state().label().chars().count(),
                    None => 0,
                })
                .max()
                .unwrap_or(0);
            // The tasks done differently, in words, after the state and before the predecessor.
            // After the state so the fraction and the state stay where a reader compares them, and
            // blank at zero, because a figure reading zero on nearly every row teaches a reader to
            // pass over it. The fraction is not touched: such a task is done.
            let differently: Vec<String> = changes
                .iter()
                .map(|listed| match &listed.completion {
                    Some(counted) if !counted.done_differently.is_empty() => {
                        format!("{} done differently", counted.done_differently.len())
                    }
                    _ => String::new(),
                })
                .collect();
            let widest_differently = differently
                .iter()
                .map(|words| words.chars().count())
                .max()
                .unwrap_or(0);
            for ((listed, fraction), differently) in
                changes.iter().zip(&fractions).zip(&differently)
            {
                // The column, with the two spaces that separate it, exists only where some row
                // has something to put in it. Reserved on every listing, it would move the
                // predecessor right on a listing of work done as written, which is meant to print
                // exactly what it printed before the column existed.
                let differently = if widest_differently == 0 {
                    String::new()
                } else {
                    format!("  {differently:<widest_differently$}")
                };
                let row = format!(
                    "{:<widest_name$}  {:>widest_fraction$}  {:<widest_state$}{differently}  {}",
                    listed.name,
                    fraction,
                    match &listed.completion {
                        Some(counted) => counted.state().label(),
                        None => "",
                    },
                    // Named after the state rather than before it, so the columns the fraction
                    // and the state occupy are where they were. That alignment was a stated
                    // requirement one change ago, and a predecessor is the less common fact —
                    // most changes follow nothing and print nothing here.
                    match &listed.follows {
                        Some(follows) if follows.resolution.is_resolved() => {
                            format!("follows {}", follows.predecessor)
                        }
                        // The unresolved case is marked, because this is the listing a workflow
                        // selects from and an open predecessor is the strongest reason not to
                        // select a change. `active` and `unmatched` are named apart: one is work
                        // to wait for and the other a declaration to correct.
                        Some(follows) => format!(
                            "follows {} ({})",
                            follows.predecessor,
                            follows.resolution.label()
                        ),
                        None => String::new(),
                    }
                );
                // A change with no completion states no state either, and the padding that
                // aligned the column would otherwise be left at the end of the line. The same
                // trim removes the state column's padding from a change that follows nothing.
                let _ = streams.report(&format!("{}\n", row.trim_end()));
            }
            // Below the table rather than in it, because a cycle is a fact about a set of
            // changes: marking each member in its own row would state one contradiction as
            // several. Progress rather than the report, so the table stays a table when it is
            // piped somewhere.
            for cycle in crate::sequence::cycles(project) {
                let _ = streams.progress(&format!(
                    "note: {} each follow one another, so the ordering they state is a \
                     contradiction. No order holds among them until one of those declarations \
                     changes",
                    cycle.join(", ")
                ));
            }
        }
    }
    Outcome::Success
}

/// The project and the change directory `change` names, for an operation that reads a change
/// beside the accepted contract: `sync`, and `show --diff`.
///
/// The project first, so a store that is gone is reported as such rather than as a change that
/// could not be found in it. `status` resolves in this order for the same reason. Each refusal is
/// reported here, and the outcome it ends with is returned.
pub(crate) fn resolve_for_comparison<O: Write, E: Write>(
    change: &str,
    store: Option<&str>,
    refusal: Refusal,
    streams: &mut Streams<O, E>,
) -> Result<(ProjectRoot, PathBuf), Outcome> {
    let project = crate::store::project_for(store).map_err(|report| {
        let _ = streams.failure(&report);
        Outcome::OperationalFailure
    })?;
    let directory = resolve_change_in(store, Path::new(change), refusal).map_err(|report| {
        let _ = streams.failure(&report);
        Outcome::OperationalFailure
    })?;
    Ok((project, directory))
}

/// The specification of the change in `directory`, projected, for an operation that compares it
/// with the accepted contract: `sync`, and `show --diff`.
///
/// The three refusals archival makes, in the same order, with this wording rather than archival's.
/// Deliberately not shared with `archive`: the checks coincide but the messages do not, because
/// archival's explain what archival is about to do and these explain what a comparison would mean.
/// The fact that is shared — the comparison itself — is `merge_state`, and that is the one that
/// would matter if it drifted.
pub(crate) fn load_specification<O: Write, E: Write>(
    directory: &Path,
    streams: &mut Streams<O, E>,
) -> Result<htmlspec_parse::Projection, Outcome> {
    let path = directory.join(CHANGE_SPECIFICATION);
    if !path.exists() {
        let _ = streams.failure(&format!(
            "{} has no {CHANGE_SPECIFICATION}. There is no contract to compare until an agent \
             produces one from the completed change",
            directory.display()
        ));
        return Err(Outcome::OperationalFailure);
    }

    let source = std::fs::read_to_string(&path).map_err(|error| {
        let _ = streams.failure(&format!("cannot read {}: {error}", path.display()));
        Outcome::OperationalFailure
    })?;

    let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);
    let declared = projection.model.kind.as_deref();
    if declared != Some("specification") {
        let _ = streams.failure(&format!(
            "{} declares {} rather than the specification kind. The requirements a change \
             states live in its specification, so there is nothing here to compare",
            path.display(),
            match declared {
                Some(kind) => format!("the {kind} kind"),
                None => String::from("no kind"),
            }
        ));
        return Err(Outcome::OperationalFailure);
    }

    let report = validate_source(&source, &path, &mut Reach::default(), Strictness::Default);
    if !report.is_accepted() {
        // Refused rather than reported and continued. A comparison made against a contract the
        // project's own tools reject is a comparison whose result means nothing, and reporting
        // it as though it did is worse than declining to report.
        let _ = streams.report(&format!("{}\n", path.display()));
        let _ = streams.report(&indent(&htmlspec_validate::render_human(&report)));
        let _ = streams.failure(
            "the comparison does not run while the specification reports an error. A contract \
             read from an invalid document states nothing this could check",
        );
        return Err(Outcome::DocumentViolation);
    }
    Ok(projection)
}

/// Whether a change's contract reached the project's accepted specifications.
///
/// **This is the operation archival tells a contributor to run, and until now there was none.**
/// When a change states a requirement no accepted specification holds, archival reports it and
/// says to apply the change's contract before archiving. Nothing performed that, and nothing
/// could: `Archival assesses the merge and never performs it` correctly excludes it from
/// archival, and no requirement assigned it anywhere else.
///
/// **It reports and never writes.** `artifact-kinds` names updating the accepted specifications
/// the fourth of four agent-driven operations — the only true merge in the model — and states
/// that the rules directing a merge are supplied as instructions while what the project
/// implements in code is the checking. So this does the checking and stops. The merge is the
/// `sync` stage's instructions, performed by an agent that reads both sides for meaning.
///
/// The name is inherited rather than chosen: OpenSpec provides this operation, and
/// `toolchain-packaging` requires that an operation OpenSpec provides carries OpenSpec's name.
/// OpenSpec's own version is agent-driven too, so the name imports no programmatic merger.
///
/// **A non-empty absent set is a success.** This is the operation someone runs *during* the
/// merge, and a change mid-implementation legitimately states requirements that have not reached
/// the contract. Failing on that would fail on the normal state of the work this exists to be
/// run against. What it refuses on is a question it cannot answer meaningfully: a change it
/// cannot resolve, an absent specification, or one that does not validate.
pub fn sync<O: Write, E: Write>(
    change: &str,
    store: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let (project, directory) =
        match resolve_for_comparison(change, store, Refusal::Comparison, streams) {
            Ok(resolved) => resolved,
            Err(outcome) => return outcome,
        };
    let projection = match load_specification(&directory, streams) {
        Ok(projection) => projection,
        Err(outcome) => return outcome,
    };

    // The fourth refusal, and it belongs here rather than only in archival. This operation and
    // archival are the two that put a change's contract into the accepted specifications, so both
    // are places example content can enter it — and this one is worse if it passes. The report
    // below names every requirement no accepted specification holds and tells an agent to merge
    // each one into the specification that should hold it. On a specification straight from the
    // template that instruction reads "merge `req-example`", and an agent that complies has put
    // an obligation nobody wrote into the contract with every check still green.
    //
    // A placeholder summary refuses here too, and the reason is the same one shifted by one step:
    // this report directs an agent to merge requirements out of a document that does not say what
    // it is for. An agent deciding which accepted specification a requirement belongs under reads
    // the summary to decide — that is what the summary is for — so a placeholder there is a merge
    // directed by a document with the deciding text missing.
    //
    // Before `merge_state`, so the refusal arrives instead of that instruction rather than after
    // it.
    let kept = example_content_in(&projection);
    if !kept.is_empty() {
        let named = kept.named();
        let because = if kept.identifiers.is_empty() {
            "A template's placeholder summary states nothing about what the document is for, so \
             this would direct a merge out of a specification whose own account of itself is \
             missing."
                .to_owned()
        } else {
            format!(
                "An example requirement states no obligation, so this would report {} to merge \
                 into an accepted specification.",
                if kept.identifiers.len() == 1 {
                    "it as a requirement"
                } else {
                    "them as requirements"
                },
            )
        };
        let _ = streams.failure(&format!(
            "the comparison does not run while the specification carries a template's example \
             content: {}. {because} Replace {} with the change's own, and run this again",
            named.join(", "),
            if named.len() == 1 {
                "it"
            } else {
                "each of them"
            },
        ));
        return Outcome::DocumentViolation;
    }

    let state = merge_state(&project, &projection);

    if streams.form().is_structured() {
        let payload = serde_json::json!({
            "change": directory.file_name().map(|name| name.to_string_lossy()),
            "stated": state.stated,
            "reached": state.reached,
            "absent": state.absent,
            "untakenRenames": state
                .untaken_renames
                .iter()
                .map(UntakenRename::payload)
                .collect::<Vec<_>>(),
            // Each naming where the content differs, and never what either side says.
            "untakenContent": state
                .untaken_content
                .iter()
                .map(UntakenContent::payload)
                .collect::<Vec<_>>(),
            "untakenRemovals": state.untaken_removals,
        });
        if let Ok(json) = serde_json::to_string_pretty(&payload) {
            let _ = streams.payload(&json);
        }
        return Outcome::Success;
    }

    // Named individually rather than counted, on the reason `report_findings` already states: a
    // contributor can check three identifiers in under a minute and cannot check "3
    // requirements".
    if state.stated.is_empty() {
        let _ = streams.report(
            "the specification states no requirement identifiers, so there is nothing to \
             compare\n",
        );
        return Outcome::Success;
    }

    let _ = streams.report(&format!(
        "{} of {} requirement(s) this change states have reached the accepted specifications\n",
        state.reached.len(),
        state.stated.len()
    ));
    if !state.reached.is_empty() {
        let _ = streams.report(&format!("  reached  {}\n", state.reached.join(", ")));
    }
    if state.reached_in_full() {
        let _ = streams.report("  the change's contract has been applied in full\n");
        return Outcome::Success;
    }
    if !state.absent.is_empty() {
        let _ = streams.report(&format!("  absent   {}\n", state.absent.join(", ")));
        let _ = streams.progress(&format!(
            "note: {} requirement(s) appear in no accepted specification. Merge each into the \
             specification that should hold it, or confirm it was abandoned. This command reports \
             the comparison and never edits an accepted specification",
            state.absent.len()
        ));
    }
    // One line per rename rather than a joined list, because each names two headings and a
    // heading can hold a comma.
    for rename in &state.untaken_renames {
        let _ = streams.report(&format!("  renamed  {}\n", rename.described()));
    }
    if !state.untaken_renames.is_empty() {
        let _ = streams.progress(&format!(
            "note: {} rename(s) this change states have not reached the accepted \
             specifications. Give each accepted requirement the heading this change states, keep \
             its identifier, and leave out the renamed-from paragraph. This command reports the \
             comparison and never edits an accepted specification",
            state.untaken_renames.len()
        ));
    }
    // One line per requirement, naming where its content differs: a statement position and
    // scenario identifiers, never a text from either side, because a text in a worklist reads as
    // the text to write.
    for content in &state.untaken_content {
        let _ = streams.report(&format!("  differs  {}\n", content.described()));
    }
    if !state.untaken_content.is_empty() {
        let _ = streams.progress(&format!(
            "note: {} requirement(s) this change adds or modifies hold other content in the \
             accepted specifications. Give each accepted requirement the statements and scenarios \
             this change states, reading from the statement and the scenarios named. This command \
             reports where the two differ and never edits an accepted specification",
            state.untaken_content.len()
        ));
    }
    for id in &state.untaken_removals {
        let _ = streams.report(&format!("  held     {id} (this change marks it removed)\n"));
    }
    if !state.untaken_removals.is_empty() {
        let _ = streams.progress(&format!(
            "note: {} requirement(s) this change withdraws are still held in the accepted \
             specifications. Delete each from the accepted specification that holds it. This \
             command reports the comparison and never edits an accepted specification",
            state.untaken_removals.len()
        ));
    }
    Outcome::Success
}

/// What a change holds and how much of its work is done.
///
/// `archive` derives its report about incomplete work from [`open_tasks`], which is the same
/// function this calls. Two counts of one thing drift, and archival disagreeing with the
/// command a contributor just ran is the failure that obligation exists to prevent.
pub fn status<O: Write, E: Write>(
    change: &str,
    store: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    // A selection resolves the project first, so a store that is gone is reported as such
    // rather than as a change that could not be found in it.
    if let Err(report) = crate::store::project_for(store) {
        let _ = streams.failure(&report);
        return Outcome::OperationalFailure;
    }
    let directory = match resolve_change_in(store, Path::new(change), Refusal::Report) {
        Ok(directory) => directory,
        Err(report) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };
    status_of(&directory, streams)
}

/// Reports on a resolved change directory, which is everything [`status`] does after resolution.
///
/// Split out for the reason [`list_in`] is: resolution reads the working directory or the
/// registry, and a test of what the report says should not have to stand in a project or register
/// a store to reach it.
fn status_of<O: Write, E: Write>(directory: &Path, streams: &mut Streams<O, E>) -> Outcome {
    let mut artifacts: Vec<(&str, bool)> = CHANGE_DOCUMENTS
        .iter()
        .map(|(name, _)| (*name, directory.join(name).exists()))
        .collect();
    artifacts.push((
        CHANGE_SPECIFICATION,
        directory.join(CHANGE_SPECIFICATION).exists(),
    ));

    // One traversal of the change's tasks, where this previously made two — `open_tasks` and
    // `total_tasks` each read and parsed the same document. Both numbers are projections of the
    // one count, which is what keeps this operation, the enumeration, and archival in agreement.
    let counted = completion(directory).unwrap_or_default();
    let total = counted.total();
    let Completion {
        open,
        done_differently,
        ..
    } = counted;

    if streams.form().is_structured() {
        let payload = serde_json::json!({
            "change": directory.file_name().map(|name| name.to_string_lossy()),
            "artifacts": artifacts
                .iter()
                .map(|(name, present)| serde_json::json!({ "name": name, "present": present }))
                .collect::<Vec<_>>(),
            // The tasks done differently mirror the open ones: a count, and the identifiers
            // behind it, because a reviewer checks each deviation against the work and a count
            // alone would leave them searching the tasks document for the tasks it counts.
            "tasks": {
                "total": total,
                "open": open.len(),
                "openTasks": open,
                "doneDifferently": done_differently.len(),
                "doneDifferentlyTasks": done_differently,
            },
        });
        if let Ok(json) = serde_json::to_string_pretty(&payload) {
            let _ = streams.payload(&json);
        }
        return Outcome::Success;
    }

    for (name, present) in &artifacts {
        let _ = streams.report(&format!(
            "  {} {name}\n",
            if *present { "present" } else { "absent " }
        ));
    }
    let _ = streams.report(&format!(
        "  {} of {total} task(s) done or dropped\n",
        total.saturating_sub(open.len())
    ));
    // A second line, and only where there is something to name. Such a task is among the done
    // ones counted above, and the line says which they are.
    if !done_differently.is_empty() {
        let _ = streams.report(&format!(
            "  {} done differently: {}\n",
            done_differently.len(),
            done_differently.join(", ")
        ));
    }
    Outcome::Success
}

/// How much of one change's work is done, counted by the state each task declares.
///
/// **One counter, and every caller that reports completion derives its numbers from this.** The
/// enumeration, the completion operation, and archival each need a different projection of the
/// same fact, and computing them separately would be three answers to one question. The accepted
/// contract already requires two of the three to agree — "Archival SHALL derive its report about
/// incomplete work from this same operation" — and the third is this type's other caller.
///
/// A count per state rather than a done-over-total pair, because the pair cannot separate
/// readings a contributor selects between: `0/24` is untouched work if every task is `todo` and
/// work wholly under way if every task is `in-progress`, and `24/24` is finished whether the last
/// four were completed or abandoned. The fraction is a rendering of this, not the record.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Completion {
    /// Tasks stated and not started.
    todo: usize,
    /// Tasks under way.
    in_progress: usize,
    /// Tasks finished.
    done: usize,
    /// Tasks abandoned, each of which states why.
    dropped: usize,
    /// Tasks whose state the document did not declare, or declared outside the contract's
    /// vocabulary.
    ///
    /// The schema requires a state, so such a task is a document that does not satisfy it — and
    /// these counts are read from a projection rather than from a validated document, so the case
    /// is reachable. Counted apart from the four rather than folded into `todo`, because
    /// attributing to a document a state it never declared would report something it does not
    /// say.
    undeclared: usize,
    /// Identifiers of the tasks that are neither `done` nor `dropped`, in document order.
    ///
    /// Carried beside the counts rather than recomputed, so the identifiers archival reports and
    /// the numbers the listing reports come from one traversal.
    open: Vec<String>,
    /// Identifiers of the `done` tasks that state a deviation, in document order.
    ///
    /// A subset of `done`, carried apart from the per-state counts rather than as a fifth state:
    /// such a task is finished, so it stays in `done`, in the numerator, and among the resolved.
    /// A task in another state that carries a deviation is a document the schema rejects, and is
    /// counted under the state it declares only — the reading `undeclared` already takes.
    ///
    /// Identifiers rather than a bare count, beside `open`, because the completion report names
    /// each one, and names computed anywhere else would be a second traversal of the same tasks.
    done_differently: Vec<String>,
}

impl Completion {
    /// Every task the change states, including the ones it abandoned.
    ///
    /// `dropped` is in the denominator deliberately. Excluding it would shrink a change's total
    /// as work was abandoned, so `18/20` could become `18/18` with nothing completed, and a
    /// denominator that moves is one nobody can compare two readings of.
    fn total(&self) -> usize {
        self.todo + self.in_progress + self.done + self.dropped + self.undeclared
    }

    /// Tasks nobody is waiting on: finished or deliberately abandoned.
    ///
    /// A dropped task states why it was dropped, and that reason is a decision rather than an
    /// omission — which is why it resolves the task without completing it.
    fn resolved(&self) -> usize {
        self.done + self.dropped
    }

    /// What state the change's work as a whole is in, as a function of the breakdown.
    ///
    /// Derived rather than stored: a change's state is already determined by its tasks, and
    /// keeping it beside them would be a second answer that could disagree with the first.
    fn state(&self) -> ChangeState {
        if self.total() > 0 && self.resolved() == self.total() {
            return ChangeState::Done;
        }
        if self.done == 0 && self.in_progress == 0 && self.dropped == 0 {
            return ChangeState::Todo;
        }
        ChangeState::InProgress
    }
}

/// Where a change's work as a whole stands.
///
/// Three states rather than the task vocabulary's four: `dropped` is a thing that happens to one
/// task, not a state a change rests in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ChangeState {
    /// Nothing has been finished, abandoned, or started.
    Todo,
    /// Some work is resolved or under way, and some is not.
    InProgress,
    /// Every task is resolved.
    Done,
}

impl ChangeState {
    /// The word the reports use, spelled as the task vocabulary spells its states.
    fn label(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in-progress",
            Self::Done => "done",
        }
    }
}

/// What one change's tasks document states, or `None` when it has none.
///
/// `None` is the distinction the listing exists to preserve: a change whose tasks document is
/// unwritten has no completion, and reporting it as `0/0` would read as a change with no work
/// outstanding — the opposite of what an unwritten tasks document means.
fn completion(directory: &Path) -> Option<Completion> {
    let source = std::fs::read_to_string(directory.join("tasks.html")).ok()?;
    let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);
    let mut counted = Completion::default();
    for task in &projection.model.tasks {
        // How a report names the task: its identifier, or its title where it declares none.
        let named = || {
            task.id
                .clone()
                .or_else(|| task.title.clone())
                .unwrap_or_else(|| String::from("an unnamed task"))
        };
        match task.state.as_deref() {
            Some("done") => {
                counted.done += 1;
                // Only here, in the `done` arm: a deviation on a task in any other state is a
                // document the schema rejects, and it is counted under its own state alone.
                if task.deviation.is_some() {
                    counted.done_differently.push(named());
                }
            }
            Some("dropped") => counted.dropped += 1,
            Some("in-progress") => counted.in_progress += 1,
            Some("todo") => counted.todo += 1,
            _ => counted.undeclared += 1,
        }
        // Not `done` and not `dropped` is open, and a task declaring no state at all is open —
        // a task nobody has said anything about is not finished.
        if !matches!(task.state.as_deref(), Some("done" | "dropped")) {
            counted.open.push(named());
        }
    }
    Some(counted)
}

/// Emits a stage's workflow instructions.
///
/// The same text `init` installs, read from the same table. An agent obtains the instructions
/// without knowing which tool's directory holds them, and without the risk of reading a copy an
/// earlier version of the toolchain installed. One source is what keeps the emitted text and the
/// installed text from stating different things.
pub fn instructions<O: Write, E: Write>(stage: &str, streams: &mut Streams<O, E>) -> Outcome {
    let Some(workflow) = htmlspec_assets::instructions::workflow_for(stage) else {
        let names: Vec<&str> = WORKFLOWS.iter().map(|workflow| workflow.name).collect();
        let _ = streams.failure(&format!(
            "no workflow named {stage}. The stages are: {}",
            names.join(", ")
        ));
        return Outcome::OperationalFailure;
    };

    if streams.form().is_structured() {
        let payload = serde_json::json!({
            "stage": workflow.name,
            "summary": workflow.summary,
            "argumentHint": workflow.argument_hint,
            "body": workflow.body,
        });
        if let Ok(json) = serde_json::to_string_pretty(&payload) {
            let _ = streams.payload(&json);
        }
        return Outcome::Success;
    }

    let _ = streams.report(workflow.body);
    Outcome::Success
}

/// Enumerates an OpenSpec project, writing nothing.
///
/// The first of a migration's three exact parts. Everything it reports is a fact about what the
/// source holds; nothing it reports is a judgment about what any of it means, because
/// `project-migration`'s survey requirement forbids the survey from interpreting what it finds —
/// a survey reporting both would present a judgment as a fact.
///
/// The human report leads with the counts and then names the entities a plan will have the most
/// trouble with: the changes recording no version, the versions several changes share, and the
/// delta sections whose change state the destination cannot express. Those are the parts a
/// contributor has to decide about, so they are what the report is for.
pub fn migrate_survey<O: Write, E: Write>(
    path: Option<&Path>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let given = path.unwrap_or_else(|| Path::new("."));
    let survey = match crate::migrate::Survey::of(given) {
        Ok(survey) => survey,
        Err(report) => {
            let _ = streams.failure(&report.to_string());
            return Outcome::OperationalFailure;
        }
    };

    if streams.form().is_structured() {
        if let Ok(json) = serde_json::to_string_pretty(&survey.payload()) {
            let _ = streams.payload(&json);
        }
        return Outcome::Success;
    }

    let mut report = String::new();
    report.push_str(&format!(
        "{} entities in {}\n",
        survey.entities.len(),
        survey.root.display()
    ));
    for (kind, count) in survey.counts() {
        report.push_str(&format!("  {count:>4}  {kind}\n"));
    }

    let archived = survey
        .changes
        .iter()
        .filter(|change| change.archived)
        .count();
    report.push_str(&format!(
        "\n{} changes, {archived} archived\n",
        survey.changes.len()
    ));

    // A change with no recorded version is named rather than counted. The plan has to record
    // that the source carried none, and that is a different fact from a version being dropped —
    // so a contributor needs the name, not the total.
    let versionless: Vec<&str> = survey
        .changes
        .iter()
        .filter(|change| change.version.is_none())
        .map(|change| change.name.as_str())
        .collect();
    if versionless.is_empty() {
        match survey.greatest_version() {
            Some(version) => report.push_str(&format!(
                "every change records a version; the greatest is {version}\n"
            )),
            None => report.push_str("no change records a version\n"),
        }
    } else {
        report.push_str(&format!(
            "these changes record no version: {}\n",
            versionless.join(", ")
        ));
    }

    // A change that states its predecessor is named with it, for the reason a versionless change
    // is: each becomes a `FOLLOWS` declaration in the migrated project, written from this survey,
    // and a contributor checks the plan against the names rather than against a total.
    let following: Vec<String> = survey
        .changes
        .iter()
        .filter_map(|change| {
            let follows = change.configuration.as_ref()?.follows.as_deref()?;
            Some(format!("{} follows {follows}", change.name))
        })
        .collect();
    if following.is_empty() {
        report.push_str("no change's configuration states a predecessor\n");
    } else {
        report.push_str(&format!(
            "these changes' configurations state a predecessor: {}\n",
            following.join(", ")
        ));
    }

    let shared = survey.shared_versions();
    if !shared.is_empty() {
        report.push_str(
            "\nversions recorded by more than one change. The destination requires recorded \
             versions to be distinct and provides for this with a release-record entry naming \
             the changes that shipped together, so carry that record rather than inventing \
             versions to separate them:\n",
        );
        for (version, changes) in &shared {
            report.push_str(&format!(
                "  {version}  {} changes: {}\n",
                changes.len(),
                changes.join(", ")
            ));
        }
    }

    let mut sections: Vec<&str> = survey
        .changes
        .iter()
        .flat_map(|change| change.delta_sections.iter().map(String::as_str))
        .collect();
    sections.sort_unstable();
    sections.dedup();
    if !sections.is_empty() {
        report.push_str(&format!(
            "\ndelta sections in use: {}\n",
            sections.join(", ")
        ));
    }

    if survey.configuration.keys.is_empty() {
        report.push_str("\nthe project's configuration sets no keys\n");
    } else {
        report.push_str(&format!(
            "\nthe project's configuration sets: {}\n",
            survey.configuration.keys.join(", ")
        ));
    }

    let _ = streams.report(&report);
    let _ = streams.progress(
        "nothing was written. This is the survey; reading each document and deciding what it \
         becomes is the migrate workflow's work\n",
    );
    Outcome::Success
}

/// How far `migrate promote` goes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromoteMode {
    /// Run every check and move nothing.
    ///
    /// The whole operation up to the move, so a contributor sees exactly what promotion would say
    /// without the destination changing. `project-migration` requires the destination to be
    /// unchanged where migration stops before promotion, and this is the mode that makes stopping
    /// the cheap thing to do.
    Check,
    /// Run every check and then move the staged project into place.
    Perform,
}

/// What promotion looked at, and what it found.
///
/// One value rather than four locals, because both output forms report the same findings and a
/// second assembly of them is where the human report and the payload drift apart.
struct Promotion {
    /// How many entities the survey found.
    surveyed: usize,
    /// How many items the plan holds.
    planned: usize,
    /// What the plan failed to account for.
    coverage: crate::plan::Coverage,
    /// What is structurally wrong with the staged tree, where one was given.
    faults: Vec<crate::staging::Fault>,
    /// Every conformance diagnostic across the whole staged tree, where it was checked.
    diagnostics: Vec<String>,
    /// How many documents the conformance checks read.
    documents: usize,
    /// What was moved into the destination.
    moved: Vec<String>,
}

impl Promotion {
    /// Whether promotion may proceed.
    fn is_clear(&self) -> bool {
        self.coverage.is_complete() && self.faults.is_empty() && self.diagnostics.is_empty()
    }
}

/// Promotes a staged migration into a destination project.
///
/// The second of a migration's three exact parts. Four stages, in this order, and the order is the
/// requirement rather than an implementation detail:
///
/// 1. **Coverage.** The plan against the survey. First, because a plan missing an entity kind is
///    reported before anything is read from the staging tree — so a contributor repairs the plan
///    rather than a half-checked tree.
/// 2. **Structure.** The staged tree is outside the destination, resolves as a project, holds one
///    asset store, records its versions where the archival gate reads them, and the destination
///    holds nothing promotion would place.
/// 3. **Conformance.** `validate`, `format --check`, and `assets` over the **complete** staged
///    project. Every occurrence together, never the first.
/// 4. **Move.** One step, whole or none.
///
/// The gate never judges a disposition. That division is the one archival already uses — the agent
/// decides what each entity becomes, the program checks that a decision was recorded for all of
/// them — and it exists because whether a disposition is *right* is the thing the plan is put in
/// front of a contributor for.
pub fn migrate_promote<O: Write, E: Write>(
    plan_path: &Path,
    survey_path: &Path,
    staged: Option<&Path>,
    destination: Option<&Path>,
    mode: PromoteMode,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let plan = match crate::plan::Plan::read(plan_path) {
        Ok(plan) => plan,
        Err(report) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };

    let payload = match std::fs::read_to_string(survey_path)
        .map_err(|error| format!("{} could not be read: {error}", survey_path.display()))
        .and_then(|text| {
            serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|error| format!("{} is not JSON: {error}", survey_path.display()))
        })
        .and_then(|payload| crate::plan::surveyed_paths(&payload))
    {
        Ok(surveyed) => surveyed,
        Err(report) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };

    let mut promotion = Promotion {
        surveyed: payload.len(),
        planned: plan.items.len(),
        coverage: crate::plan::Coverage::of(&plan, &payload),
        faults: Vec::new(),
        diagnostics: Vec::new(),
        documents: 0,
        moved: Vec::new(),
    };

    // The staged tree is optional so that the coverage check is usable on its own, while the plan
    // is still being written and nothing has been staged. That is the stage order made reachable
    // rather than merely documented: an agent runs the gate at stage 4 and runs the whole of
    // promotion at stage 7, with one command.
    if let Some(staged) = staged {
        let destination = destination.unwrap_or_else(|| Path::new("."));
        match crate::staging::Staged::resolve(staged, destination) {
            Ok(ready) => {
                let (documents, diagnostics) = conformance(ready.root(), streams);
                promotion.documents = documents;
                promotion.diagnostics = diagnostics;

                // The move is last, and only where everything above is clear. A coverage failure
                // and a staging fault and a diagnostic are all reasons the destination must stay
                // as it is.
                if mode == PromoteMode::Perform && promotion.is_clear() {
                    match ready.promote() {
                        Ok(moved) => promotion.moved = moved,
                        Err(error) => {
                            let _ = streams.failure(&format!(
                                "the staged project could not be moved into {}: {error}. The \
                                 destination is as it was, and the staged project is whole, so \
                                 this is safe to run again",
                                ready.destination().display()
                            ));
                            return Outcome::OperationalFailure;
                        }
                    }
                }
            }
            Err(faults) => promotion.faults = faults,
        }
    }

    if streams.form().is_structured() {
        report_promotion_payload(plan_path, survey_path, staged, &promotion, streams);
    } else {
        report_promotion(&promotion, mode, staged.is_some(), streams);
    }

    if promotion.is_clear() {
        Outcome::Success
    } else {
        Outcome::DocumentViolation
    }
}

/// Runs the project's conformance checks over the complete staged project.
///
/// **Every document, then every diagnostic, together.** A mapping fault is systematic: a rule that
/// is wrong for one scenario is wrong for every scenario, so halting at the first occurrence
/// presents a class of failure as one document's defect — which a contributor then repairs one
/// document at a time, in a tree that is now partly migrated. Completing the tree first makes the
/// shape of the fault visible, and the shape is what points at the repair.
///
/// The three checks are the project's own, unchanged. A migration adds no check of its own: what a
/// conforming project is has already been settled, and a fourth definition applied only to
/// migrated projects would let a migration produce something the toolchain would not otherwise
/// accept.
///
/// **So carried evidence is not read here**, because the project's own walk does not read it.
/// project-migration's "Evidence crosses unchanged" carries records byte-identical, including
/// documents in this format at a superseded contract, and a check that refused them would leave a
/// migration two choices: edit the record, or leave it behind.
fn conformance<O: Write, E: Write>(
    staged: &Path,
    streams: &mut Streams<O, E>,
) -> (usize, Vec<String>) {
    let mut diagnostics = Vec::new();
    let documents = crate::discover::documents(staged).unwrap_or_default();
    let mut reach = Reach::default();

    for path in &documents {
        let display = path.strip_prefix(staged).unwrap_or(path).display();
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                diagnostics.push(format!("{display}: cannot read it: {error}"));
                continue;
            }
        };

        let store = store_for(path);
        let report = validate_source(&source, path, &mut reach, Strictness::Default);
        for error in report.errors() {
            diagnostics.push(format!("{display}: {} {}", error.rule, error.message));
        }

        // Formatting, checked rather than written. A migration that reformatted its own output
        // here would hide the mapping fault that produced non-canonical documents.
        //
        // **Two causes, reported apart.** A document can differ from its canonical form because its
        // serialization is not canonical, or because it names asset bytes the store does not hold
        // and the filled reference differs from what is on disk. Those need different repairs, and
        // `format` at the top of this file already distinguishes them. Reporting both as "not
        // canonically formatted" sent a contributor to the serializer for a reference fault —
        // across every document of the kind, since a migration produces them in bulk. A review of
        // the pull request found this.
        let prefix = assets_prefix(path, &store);
        let rewritten = htmlspec_format::rewrite(&source, &store, &prefix);
        for change in rewritten.filled() {
            if let htmlspec_format::ReferenceChange::Filled { name, now, .. } = change {
                diagnostics.push(format!(
                    "{display}: {name} names bytes this project does not store; the staged \
                     reference must be {now}"
                ));
            }
        }
        match htmlspec_format::check(&rewritten.rewritten) {
            Ok(outcome) => {
                if outcome.canonical != rewritten.rewritten {
                    diagnostics.push(format!("{display}: not canonically formatted"));
                }
            }
            Err(error) => diagnostics.push(format!("{display}: cannot be formatted: {error}")),
        }
    }

    // Asset topology, which is a statement about the project rather than about a document — so it
    // runs once over the staged tree rather than per document. The same function `assets` calls,
    // so a staged project and a promoted one are judged alike. The staged tree is a project by
    // the staging check that runs first; where it somehow is not, no store is compared across.
    let project = ProjectRoot::resolve(staged).ok();
    let topology = crate::topology::check(project.as_ref(), staged);
    let stored = topology.stored;
    for duplicate in topology.diagnostics {
        diagnostics.push(format!("assets: {}", duplicate.message));
    }

    let _ = streams.progress(&format!(
        "checked {} document(s) and {stored} stored asset(s) across the complete staged project",
        documents.len()
    ));
    (documents.len(), diagnostics)
}

/// The human report for a promotion.
fn report_promotion<O: Write, E: Write>(
    promotion: &Promotion,
    mode: PromoteMode,
    was_staged: bool,
    streams: &mut Streams<O, E>,
) {
    let _ = streams.report(&format!(
        "{} surveyed entities, {} plan items\n",
        promotion.surveyed, promotion.planned
    ));

    if promotion.coverage.is_complete() {
        let _ = streams.report("every surveyed entity has a disposition\n");
    } else {
        let _ = streams.report(&format!("\n{}", promotion.coverage.report()));
    }

    if !promotion.faults.is_empty() {
        let _ = streams.report(&format!("\n{} staging fault(s):\n", promotion.faults.len()));
        for fault in &promotion.faults {
            let _ = streams.report(&format!("  {fault}\n"));
        }
    }

    if !promotion.diagnostics.is_empty() {
        // Every occurrence, and the count first. "401 documents report the same diagnostic" is a
        // different finding from "document 3 is broken", and it is the finding that points at the
        // repair.
        let _ = streams.report(&format!(
            "\n{} conformance diagnostic(s) across {} staged document(s):\n",
            promotion.diagnostics.len(),
            promotion.documents
        ));
        for diagnostic in &promotion.diagnostics {
            let _ = streams.report(&format!("  {diagnostic}\n"));
        }
    }

    if !promotion.is_clear() {
        let _ = streams.failure(
            "promotion does not proceed. Every occurrence is above rather than the first, because \
             a mapping fault is systematic: a rule that is wrong for one document is wrong for \
             every document of its kind, and repairing them one at a time hides the shape of the \
             fault. The destination is unchanged",
        );
        return;
    }

    if !promotion.moved.is_empty() {
        let _ = streams.report(&format!("\npromoted: {}\n", promotion.moved.join(", ")));
        let _ = streams.progress(
            "the source project is untouched. Removing it is a separate act you take after \
             reading the result\n",
        );
        return;
    }

    let _ = streams.progress(if was_staged && mode == PromoteMode::Check {
        "every check passed and nothing was moved. Run this without --check to promote\n"
    } else if was_staged {
        "every check passed\n"
    } else {
        "the coverage check passed. Whether each disposition is right is what this plan is \
         reviewed for, and the check says nothing about it. Pass --staged <path> to check a \
         staged project and promote it\n"
    });
}

/// The structured payload for a promotion.
fn report_promotion_payload<O: Write, E: Write>(
    plan_path: &Path,
    survey_path: &Path,
    staged: Option<&Path>,
    promotion: &Promotion,
    streams: &mut Streams<O, E>,
) {
    let coverage = &promotion.coverage;
    let json = serde_json::json!({
        "plan": plan_path.display().to_string(),
        "survey": survey_path.display().to_string(),
        "staged": staged.map(|path| path.display().to_string()),
        "surveyed": promotion.surveyed,
        "planned": promotion.planned,
        "promoted": !promotion.moved.is_empty(),
        "moved": promotion.moved,
        "clear": promotion.is_clear(),
        "covered": coverage.is_complete(),
        "unaccounted": coverage.unaccounted,
        "withoutDisposition": coverage.without_disposition,
        "unknownDisposition": coverage
            .unknown_disposition
            .iter()
            .map(|(source, word)| serde_json::json!({ "source": source, "wrote": word }))
            .collect::<Vec<_>>(),
        "withoutReason": coverage.without_reason,
        "duplicated": coverage.duplicated,
        "unsurveyed": coverage.unsurveyed,
        "sourceless": coverage.sourceless,
        "withoutDestination": coverage.without_destination,
        "unreadableDerivations": coverage
            .unreadable_derivations
            .iter()
            .map(|(source, line)| serde_json::json!({ "source": source, "wrote": line }))
            .collect::<Vec<_>>(),
        "collisions": coverage
            .collisions
            .iter()
            .map(|collision| serde_json::json!({
                "destination": collision.destination,
                "identifier": collision.identifier,
                "sources": collision
                    .sources
                    .iter()
                    .map(|(source, heading)| serde_json::json!({
                        "source": source,
                        "heading": heading,
                    }))
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
        "stagingFaults": promotion
            .faults
            .iter()
            .map(|fault| fault.to_string())
            .collect::<Vec<_>>(),
        "documents": promotion.documents,
        "diagnostics": promotion.diagnostics,
    });
    if let Ok(text) = serde_json::to_string_pretty(&json) {
        let _ = streams.payload(&text);
    }
}

/// What `archive` does with a change whose gate it satisfied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveMode {
    /// Report whether the change is ready and change nothing.
    Check,
    /// Perform the archival.
    Perform,
}

/// What archival found that it reports rather than refuses on.
///
/// The contract refuses and the work asks. These are the work: a task nobody finished, and a part
/// of the contract that never reached the accepted specifications. Both are legitimate — a dropped
/// task is a normal outcome and an abandoned requirement is legitimately absent — so neither can
/// be an error, and neither can be silent either.
#[derive(Debug, Default)]
struct Findings {
    /// The change these are about, so a report can name the command that addresses them.
    change: String,
    /// Tasks in states other than `done` or `dropped`.
    open_tasks: Vec<String>,
    /// The comparison `sync` makes, carried whole rather than copied set by set, so a set added
    /// to the comparison is among archival's findings without a second edit here.
    merge: MergeState,
    /// The change this one follows, where it states one that is not archived.
    ///
    /// `None` covers two cases that call for the same silence: the change states no predecessor,
    /// or it states one whose work is done. A finding here means the sequence the author recorded
    /// is being completed out of order, which is the last moment that is observable — archival is
    /// the operation that ends a change.
    unresolved_predecessor: Option<crate::sequence::Follows>,
    /// The changes in a cycle of `follows` declarations this change sits in.
    ///
    /// Empty where there is none. A cycle is a contradiction in the recorded facts rather than an
    /// ordering violation, so it is carried apart from the predecessor above: the program reports
    /// it and chooses no ordering within it.
    cycle: Vec<String>,
}

impl Findings {
    fn is_empty(&self) -> bool {
        self.open_tasks.is_empty()
            && self.merge.reached_in_full()
            && self.unresolved_predecessor.is_none()
            && self.cycle.is_empty()
    }

    /// Whether the change is being archived before the change it says it follows.
    ///
    /// A predicate over the finding rather than a second computation, for the reason `needs_sync`
    /// is one: two answers to one question can disagree, and neither operation would be wrong on
    /// its own terms.
    fn out_of_order(&self) -> bool {
        self.unresolved_predecessor.is_some()
    }

    /// Whether the change's contract has not reached the accepted specifications.
    ///
    /// The same question `sync` answers, asked of the finding archival already computed. Both
    /// derive it from `merge_state`, so archival cannot say a change needs syncing while `sync`
    /// says it does not.
    ///
    /// Asked of [`MergeState::reached_in_full`] rather than restated over the sets, so a set the
    /// comparison gains counts here without an edit. A rename, a requirement's content, and a withdrawal the
    /// accepted specifications have not taken all count: each has an identifier the comparison of
    /// identifiers alone once read as reached, and a change whose only unapplied work was one of
    /// them was reported as synced.
    fn needs_sync(&self) -> bool {
        !self.merge.reached_in_full()
    }
}

/// What archival does once it knows what it found.
///
/// A pure function of three inputs, so every case is a constructed value rather than a
/// simulated terminal. `prompt` states the reason for this shape: an interactive path with no
/// test is the one part of the program a contributor cannot check, and it is also the part
/// that decides whether their change moves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Decision {
    /// Archive.
    Proceed,
    /// Findings, and nobody has answered yet.
    Ask,
    /// Findings, and the answer was no or nobody could give one.
    Stop,
}

/// What to do about the findings.
///
/// `asked` is `None` before anyone has been asked and `Some(answer)` after. Splitting it this
/// way means the caller runs this twice — once to learn whether to ask, once to act on the
/// answer — and the second call is what a test drives directly.
fn decision(findings: &Findings, assume_yes: bool, asked: Option<bool>) -> Decision {
    if findings.is_empty() || assume_yes {
        return Decision::Proceed;
    }
    match asked {
        None => Decision::Ask,
        Some(true) => Decision::Proceed,
        Some(false) => Decision::Stop,
    }
}

/// The findings for a change that passed the gate.
fn findings_for(
    project: &ProjectRoot,
    directory: &Path,
    specification: &htmlspec_parse::Projection,
) -> Findings {
    let change = directory
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    Findings {
        open_tasks: open_tasks(directory),
        merge: merge_state(project, specification),
        // Only an unresolved predecessor is a finding. An archived one is the in-order case,
        // which is the common one, and a report that fired there would fire on every correct
        // archival — a report a reader learns to skip.
        unresolved_predecessor: crate::sequence::Follows::read(project, directory)
            .filter(|follows| !follows.resolution.is_resolved()),
        cycle: crate::sequence::cycle_through(project, &change).unwrap_or_default(),
        change,
    }
}

/// Tasks the change has not finished.
///
/// A projection of [`completion`] rather than its own traversal, so archival's finding and the
/// numbers the other two operations report cannot disagree. Archival's behavior is unchanged: a
/// change with no tasks document produces no finding, which is `None` flattening to no
/// identifiers — reported instead by the absent-document note.
fn open_tasks(directory: &Path) -> Vec<String> {
    completion(directory)
        .map(|counted| counted.open)
        .unwrap_or_default()
}

/// What a change's specification states, and how much of it the accepted specifications hold.
///
/// Several sets rather than one. Archival needs the sets of work not done, because a finding is
/// actionable from them alone. `sync` is the operation someone runs *while deciding what to
/// merge*, and an agent that has to compute a set difference to find its worklist will compute
/// it — a report that makes its caller do arithmetic to reach the answer it exists to give is a
/// report that gets used wrong.
///
/// Every stated identifier is in exactly one of four places: `reached`, `absent`,
/// `untaken_removals`, or the untaken pair. Within the pair it is in `untaken_renames`, in
/// `untaken_content`, or in both, because a rename and a modification are two facts about one
/// requirement.
#[derive(Debug, Default)]
pub struct MergeState {
    /// Every requirement identifier the change's specification carries.
    pub stated: Vec<String>,
    /// Those of them that have reached the accepted specifications, as the change state the
    /// change declares on each one decides: held with the content the change states, held under
    /// the heading it states where it renames one, or no longer held where it withdraws one.
    pub reached: Vec<String>,
    /// Those of them no accepted specification holds, and that the change does not withdraw.
    pub absent: Vec<String>,
    /// Those of them the change renames and the accepted specifications still hold under
    /// another heading.
    ///
    /// Not also in `reached`, so `reached` keeps meaning "the contract took this". An identifier
    /// is held and its heading is not the one the change states, so the rename is the part of the
    /// change's contract that did not arrive.
    pub untaken_renames: Vec<UntakenRename>,
    /// Those of them the change adds or modifies whose identifier is held and whose content the
    /// accepted requirement does not hold.
    pub untaken_content: Vec<UntakenContent>,
    /// Those of them the change withdraws that an accepted specification still holds.
    pub untaken_removals: Vec<String>,
}

impl MergeState {
    /// Whether the change's contract has reached the accepted specifications in full.
    ///
    /// The one statement of the condition. `sync`'s "applied in full" and archival's
    /// `needs_sync` both ask this, so a set added to the comparison reaches both without a second
    /// edit, and the two cannot disagree about whether a change is synced.
    pub fn reached_in_full(&self) -> bool {
        self.absent.is_empty()
            && self.untaken_renames.is_empty()
            && self.untaken_content.is_empty()
            && self.untaken_removals.is_empty()
    }
}

/// What a change does to the accepted contract for a requirement it states, as the comparison
/// judges it: what the accepted specifications must hold for the requirement to have reached them.
///
/// Three cases rather than the schema's four, because two of the schema's states call for the
/// same comparison. Read once per requirement from its `data-change`, so the judgment below is a
/// match over this type and never a comparison of strings. No document declares any of the three:
/// each is the comparison's reading of a declared state, which is why the type is not named for
/// a declaration, as `htmlspec_validate::references::Declared` is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContractEffect {
    /// `added` or `modified`. The change states text the merge writes, so the requirement has
    /// reached the contract only where the accepted requirement holds that text.
    Establishes,
    /// `removed`. The change withdraws the requirement, so it has reached the contract when no
    /// accepted specification holds it.
    Withdraws,
    /// `unchanged`, and any state the schema adds later. The change carries the requirement
    /// forward, so it is compared by identifier, as every requirement was before content was.
    CarriesForward,
}

impl ContractEffect {
    /// The case a stated requirement falls in.
    ///
    /// Its own state, except where that state carries it forward and one of its scenarios is one
    /// the change adds, modifies, or withdraws. Then the change states content inside it, so it is
    /// judged as a requirement the change modifies. Nothing in the contract requires such a
    /// requirement to be marked `modified`, and judged by its own state alone it was read as
    /// carried forward and its changed scenario was never compared.
    pub(crate) fn of_requirement(requirement: &Requirement) -> Self {
        match Self::of(&requirement.change) {
            Self::CarriesForward
                if requirement
                    .scenarios
                    .iter()
                    .any(|scenario| Self::of(&scenario.change) != Self::CarriesForward) =>
            {
                Self::Establishes
            }
            effect => effect,
        }
    }

    /// The case `change`, a unit's declared change state, falls in.
    pub(crate) fn of(change: &str) -> Self {
        if crate::runs::is_withdrawn(change) {
            Self::Withdraws
        } else if establishing_states().iter().any(|state| state == change) {
            Self::Establishes
        } else {
            Self::CarriesForward
        }
    }
}

/// The schema's change states with which a change establishes text: `added` and `modified`.
///
/// Read from the schema by their stems, as `runs::is_withdrawn` finds the withdrawn state, so no
/// state the schema owns is written here as a literal. The validator's `material_states` finds the
/// same two by excluding the other two instead. The difference is what a state the schema adds
/// later becomes: found by stem, it is carried forward and compared by identifier, which is how
/// every requirement was compared before this comparison read content.
fn establishing_states() -> Vec<String> {
    htmlspec_validate::schema_facts::enum_values("definitions/changeState")
        .into_iter()
        .filter(|state| state.contains("add") || state.contains("modif"))
        .collect()
}

/// Every accepted requirement, by identifier, each with the specification that holds it.
///
/// The index the comparison reads, and the view after it, so the two pair a change's requirement
/// with the same accepted ones. An identifier two accepted specifications both hold is a fault in
/// those specifications, and both holders are kept, in the order the specifications are
/// discovered: which of them a requirement is compared with is the comparison's choice, made in
/// one place.
#[derive(Debug, Default)]
pub(crate) struct AcceptedRequirements(std::collections::BTreeMap<String, Vec<Held>>);

/// One accepted requirement, and where it is held.
#[derive(Clone, Debug)]
pub(crate) struct Held {
    /// The accepted specification holding it, relative to the project's root.
    pub path: PathBuf,
    /// The requirement, as projected.
    pub requirement: Requirement,
}

impl AcceptedRequirements {
    /// The index of `held`, in the order given.
    pub(crate) fn of(held: Vec<Held>) -> Self {
        let mut index = std::collections::BTreeMap::<String, Vec<Held>>::new();
        for entry in held {
            if let Some(id) = entry.requirement.id.clone() {
                index.entry(id).or_default().push(entry);
            }
        }
        Self(index)
    }

    /// Every accepted requirement carrying `id`, or `None` where no accepted specification holds
    /// it.
    pub(crate) fn get(&self, id: &str) -> Option<Holders<'_>> {
        self.0.get(id).and_then(|held| Holders::of(held))
    }
}

/// The accepted requirements that carry one identifier: at least one, in discovery order.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Holders<'a> {
    first: &'a Held,
    rest: &'a [Held],
}

impl<'a> Holders<'a> {
    fn of(held: &'a [Held]) -> Option<Self> {
        let (first, rest) = held.split_first()?;
        Some(Self { first, rest })
    }

    /// The first, in the order the accepted specifications are discovered.
    pub(crate) fn first(self) -> &'a Held {
        self.first
    }

    /// Each of them, the first first.
    pub(crate) fn iter(self) -> impl Iterator<Item = &'a Held> {
        std::iter::once(self.first).chain(self.rest)
    }
}

/// Every requirement the project's accepted specifications hold, by identifier, each with the
/// specification that holds it.
///
/// One reading of the accepted contract, which [`merge_state`] and the view both take, so the two
/// pair a change's requirement with the same accepted ones. Read without validation: a fault in an
/// accepted specification shows up as a difference, which is where a reader looks.
pub(crate) fn accepted_requirements(project: &ProjectRoot) -> AcceptedRequirements {
    let mut held = Vec::new();
    for path in accepted_specifications(project) {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);
        let relative = path
            .strip_prefix(project.path())
            .map_or_else(|_| path.clone(), Path::to_path_buf);
        held.extend(
            projection
                .model
                .requirements
                .into_iter()
                .map(|requirement| Held {
                    path: relative.clone(),
                    requirement,
                }),
        );
    }
    AcceptedRequirements::of(held)
}

pub use untaken::UntakenContent;

/// Content the accepted specifications have not taken.
///
/// A module of its own so that its fields are private to it: the one way to build an entry is
/// [`UntakenContent::new`], which builds none that names no difference.
mod untaken {
    use std::num::NonZeroUsize;

    /// A requirement the change adds or modifies whose identifier is held, and where its content
    /// and the accepted requirement's differ.
    ///
    /// It says where to read, never what either side says. A text in a worklist reads as the text
    /// to write, and the comparison never proposes the text of a merge.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct UntakenContent {
        /// The requirement's identifier, which the change and the accepted specification share.
        ///
        /// Public, as an untaken rename's is: no value of it names a difference or fails to. The
        /// fields below are the ones [`UntakenContent::new`] guards.
        pub id: String,
        first_differing_statement: Option<NonZeroUsize>,
        scenarios_not_held: Vec<String>,
        scenarios_not_kept: Vec<String>,
    }

    impl UntakenContent {
        /// The entry for `id`, or `None` where nothing differs.
        ///
        /// `first_differing_statement` counts from one. `scenarios_not_held` names the scenarios
        /// the change keeps whose text the accepted requirement does not hold, as the change's
        /// specification names them. `scenarios_not_kept` names the scenarios the accepted
        /// requirement holds whose text the change does not keep, as the accepted specification
        /// names them.
        pub fn new(
            id: &str,
            first_differing_statement: Option<NonZeroUsize>,
            scenarios_not_held: Vec<String>,
            scenarios_not_kept: Vec<String>,
        ) -> Option<Self> {
            if first_differing_statement.is_none()
                && scenarios_not_held.is_empty()
                && scenarios_not_kept.is_empty()
            {
                return None;
            }
            Some(Self {
                id: id.to_owned(),
                first_differing_statement,
                scenarios_not_held,
                scenarios_not_kept,
            })
        }

        /// The entry as the structured payloads carry it.
        pub fn payload(&self) -> serde_json::Value {
            serde_json::json!({
                "id": self.id,
                "firstDifferingStatement": self.first_differing_statement,
                "scenariosNotHeld": self.scenarios_not_held,
                "scenariosNotKept": self.scenarios_not_kept,
            })
        }

        /// The entry in one clause: the identifier, then where each side differs.
        pub fn described(&self) -> String {
            let mut parts = Vec::new();
            if let Some(position) = self.first_differing_statement {
                parts.push(format!("statements from paragraph {position}"));
            }
            if !self.scenarios_not_held.is_empty() {
                parts.push(format!("not held: {}", self.scenarios_not_held.join(", ")));
            }
            if !self.scenarios_not_kept.is_empty() {
                parts.push(format!("not kept: {}", self.scenarios_not_kept.join(", ")));
            }
            format!("{} ({})", self.id, parts.join("; "))
        }
    }
}

/// A rename the accepted specifications have not taken.
///
/// The change states the requirement under a new heading with a `renamed-from` paragraph, and
/// the accepted requirement carrying the same identifier still has another heading. Both headings
/// are heading names (projection specification, section 4.1.2), so a document that shows the
/// identifier inside its heading and one that does not are not reported as a rename.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UntakenRename {
    /// The requirement's identifier, which the change and the accepted specification share.
    pub id: String,
    /// The heading the change states.
    pub heading: HeadingName,
    /// The heading the accepted specification holds under the same identifier.
    ///
    /// `None` where that requirement has no heading, which is a fault in the accepted
    /// specification and still not the heading the change states.
    pub accepted: Option<HeadingName>,
}

impl UntakenRename {
    /// The rename as the structured payloads carry it.
    fn payload(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "heading": self.heading.as_str(),
            "acceptedHeading": self.accepted.as_ref().map(HeadingName::as_str),
        })
    }

    /// The rename in one clause, naming the identifier and both headings.
    fn described(&self) -> String {
        match &self.accepted {
            Some(accepted) => format!(
                "{} (\"{accepted}\" in the accepted specifications, \"{}\" in this change)",
                self.id, self.heading
            ),
            None => format!(
                "{} (no heading in the accepted specifications, \"{}\" in this change)",
                self.id, self.heading
            ),
        }
    }
}

/// What a specification carries of its template's example content.
///
/// Two fields rather than one list of strings, because the two carry different consequences and
/// each caller states a reason. An example *requirement* becomes an obligation nobody wrote; a
/// placeholder *summary* is a contract that does not say what it is for. A refusal that gave one
/// reason for both would be wrong about one of them, and a refusal reading "example content: req-example,
/// summary" would list a field name among identifiers as though the document declared a unit
/// called `summary`.
#[derive(Debug, Default)]
struct ExampleContent {
    /// Every identifier a template writes as example content that the document still carries.
    identifiers: Vec<String>,
    /// Whether the document's whole summary is a paragraph a template writes as a placeholder.
    placeholder_summary: bool,
}

impl ExampleContent {
    /// Whether the specification carries none of it, which is what lets an operation proceed.
    fn is_empty(&self) -> bool {
        self.identifiers.is_empty() && !self.placeholder_summary
    }

    /// What was found, named the way a contributor has to act on it.
    ///
    /// The summary is named as "the masthead summary" rather than listed among the identifiers,
    /// which is task 5.2: an identifier is a string the contributor searches the file for, and the
    /// summary is a paragraph they open the top of the file to read. Listing a bare `summary`
    /// beside `req-example` would read as a unit the document declares.
    ///
    /// The summary comes last, so a refusal naming both reads as identifiers followed by the one
    /// thing that is not one.
    fn named(&self) -> Vec<String> {
        let mut named = self.identifiers.clone();
        if self.placeholder_summary {
            named.push("the masthead summary".to_owned());
        }
        named
    }
}

/// Every part of `specification` that a template wrote and nobody replaced.
///
/// One function for the same reason [`merge_state`] is one: archival and `sync` both put a
/// change's contract into the accepted specifications, so both refuse on this, and two copies of
/// the collection would drift the first time a unit kind was added to one of them.
///
/// Units and figures both, because `Model::units()` does not yield figures and `figure-example`
/// is one of the identifiers the specification template writes.
///
/// **And the summary, on its text rather than an identifier**, because a document header's
/// paragraphs carry none. `review/example-content` reports the same condition as a warning at
/// validation; this is where it refuses. The comparison is whole-summary equality, so a
/// specification whose summary quotes the template's wording inside the author's own statement is
/// the author's and passes — `htmlspec_assets::is_example_placeholder` states why.
fn example_content_in(specification: &htmlspec_parse::Projection) -> ExampleContent {
    let identifiers = specification
        .model
        .units()
        .into_iter()
        .filter_map(|unit| unit.id)
        .chain(
            specification
                .model
                .figures
                .iter()
                .filter_map(|figure| figure.id.as_deref()),
        )
        .filter(|id| htmlspec_assets::is_example_identifier(id))
        .map(str::to_owned)
        .collect();

    // Joined, for the reason the validator's rule states: a header carrying several paragraphs
    // projects several entries and the summary is all of them.
    let summary = specification.model.summary.join(" ");
    let placeholder_summary =
        !summary.is_empty() && htmlspec_assets::is_example_placeholder(&summary);

    ExampleContent {
        identifiers,
        placeholder_summary,
    }
}

/// The comparison archival and `sync` both make, in one place because two of it would drift.
///
/// **This compares identifiers and text, and reads nothing for meaning.** Deciding which accepted
/// requirement a scenario belongs under is judgment, and `artifact-kinds` reserves it for an
/// agent: that requirement names updating the accepted specifications as the fourth of four
/// agent-driven operations and the only true merge among them, then states the division this
/// function sits on — the rules that direct a merge are supplied as instructions, and *what the
/// project implements in code is the checking*. This is the checking.
///
/// One function rather than one per caller. A contributor who runs `sync`, sees an empty absent
/// set, and then meets archival reporting unmerged requirements has met a defect that no test
/// written against either operation alone would catch — the same shape as the version gate that
/// validated its own invented convention for eleven archives without ever refusing anything.
///
/// **Each requirement is judged by the change state the change declares on it**, as
/// [`ContractEffect`] reads it. An identifier the accepted specifications hold says only that a
/// requirement carrying it exists:
///
/// - one the change adds or modifies has reached the contract where an accepted requirement holds
///   its identifier with the content the change states, which [`untaken_content`] compares;
/// - one the change withdraws has reached it where no accepted specification holds its
///   identifier, so a withdrawal the merge applied is reached and never absent;
/// - one the change carries forward is compared by identifier, because another change may have
///   modified its text since, and that change's work is not this one's.
///
/// An absent identifier is a finding and never an error. A requirement whose work went another way
/// is legitimately absent forever, and only an agent knows which case this is.
///
/// **A renamed requirement is compared by its heading as well.** A rename keeps the identifier, so
/// a comparison of identifiers alone reports a rename the accepted specifications never took as
/// reached. So where the change's requirement carries a `renamed-from` paragraph and its identifier
/// is held, the two heading names are compared as strings. Only a stated rename is compared: a
/// heading that differs with no rename stated is an unstated rename or an edit made to the
/// accepted contract directly, and neither is this change's to report.
fn merge_state(project: &ProjectRoot, specification: &htmlspec_parse::Projection) -> MergeState {
    let stated: Vec<String> = specification
        .model
        .requirements
        .iter()
        .filter_map(|requirement| requirement.id.clone())
        .collect();
    if stated.is_empty() {
        // Nothing stated is nothing to compare. Reading every accepted specification to
        // establish that would be work with one possible outcome.
        return MergeState::default();
    }

    // Every accepted requirement, by identifier, in the order the specifications are discovered.
    // An identifier two accepted specifications both hold is a fault in those specifications, and
    // a rename or a requirement's content is taken if either holds it.
    let held = accepted_requirements(project);

    let mut state = MergeState {
        stated,
        ..MergeState::default()
    };
    for requirement in &specification.model.requirements {
        let Some(id) = &requirement.id else {
            continue;
        };
        let effect = ContractEffect::of_requirement(requirement);
        // An identifier no accepted specification holds is absent, and never also untaken work:
        // one finding per requirement, and the absent one is the larger fact. A withdrawal is the
        // exception, because its absence is what the change asks for.
        let Some(accepted) = held.get(id) else {
            match effect {
                ContractEffect::Withdraws => state.reached.push(id.clone()),
                ContractEffect::Establishes | ContractEffect::CarriesForward => {
                    state.absent.push(id.clone())
                }
            }
            continue;
        };
        let headings: Vec<Option<HeadingName>> = accepted
            .iter()
            .map(|held| held.requirement.heading_name())
            .collect();
        match effect {
            ContractEffect::Withdraws => state.untaken_removals.push(id.clone()),
            ContractEffect::Establishes => state.place(
                id,
                untaken_rename(requirement, id, &headings),
                untaken_content(requirement, id, accepted),
            ),
            ContractEffect::CarriesForward => {
                state.place(id, untaken_rename(requirement, id, &headings), None);
            }
        }
    }
    state
}

impl MergeState {
    /// Places a requirement whose identifier is held: in the untaken pair, in either set or both,
    /// or in `reached` where neither its rename nor its content is untaken.
    fn place(&mut self, id: &str, rename: Option<UntakenRename>, content: Option<UntakenContent>) {
        if rename.is_none() && content.is_none() {
            self.reached.push(id.to_owned());
        }
        self.untaken_renames.extend(rename);
        self.untaken_content.extend(content);
    }
}

/// The content `requirement` states that no accepted requirement carrying `id` holds.
///
/// `None` where one of them holds it. Otherwise the difference from the first, in the order the
/// accepted specifications are discovered, which is how an identifier two of them hold is reported.
/// [`compare_content`] decides both, and this names what it found as the report names it.
fn untaken_content(
    requirement: &Requirement,
    id: &str,
    accepted: Holders<'_>,
) -> Option<UntakenContent> {
    match compare_content(requirement, accepted) {
        ContentComparison::Held(_) => None,
        ContentComparison::Differs(held, difference) => UntakenContent::new(
            id,
            difference.first_differing_statement,
            scenarios_named(&requirement.scenarios, &difference.scenarios_not_held),
            scenarios_named(&held.requirement.scenarios, &difference.scenarios_not_kept),
        ),
    }
}

/// What the comparison finds for a requirement whose content it compares, against the accepted
/// requirements carrying its identifier.
pub(crate) enum ContentComparison<'a> {
    /// This accepted requirement holds the content the change states.
    Held(&'a Held),
    /// None of them does. The difference is from this one, the first.
    Differs(&'a Held, ContentDifference),
}

/// Where a stated requirement's content and an accepted requirement's differ, by position.
///
/// Positions rather than names, so `sync` names each scenario as its document names it and the view
/// reads the scenarios themselves. Never empty: [`content_difference`] gives `None` where nothing
/// differs.
pub(crate) struct ContentDifference {
    /// The first statement position, counted from one, at which the two differ.
    pub first_differing_statement: Option<std::num::NonZeroUsize>,
    /// Each scenario the change keeps whose text the accepted requirement does not hold, by its
    /// position among the stated requirement's scenarios.
    pub scenarios_not_held: Vec<usize>,
    /// Each scenario the accepted requirement holds whose text the change does not keep, by its
    /// position among the accepted requirement's scenarios.
    pub scenarios_not_kept: Vec<usize>,
}

/// The comparison of `requirement`'s content with the accepted requirements in `accepted`.
///
/// The one place the comparison chooses between two accepted requirements carrying one identifier:
/// one that holds the content where there is one, and otherwise the first.
pub(crate) fn compare_content<'a>(
    requirement: &Requirement,
    accepted: Holders<'a>,
) -> ContentComparison<'a> {
    let first = accepted.first();
    let Some(difference) = content_difference(requirement, &first.requirement) else {
        return ContentComparison::Held(first);
    };
    match accepted
        .iter()
        .skip(1)
        .find(|held| content_difference(requirement, &held.requirement).is_none())
    {
        Some(held) => ContentComparison::Held(held),
        None => ContentComparison::Differs(first, difference),
    }
}

/// Where the content `requirement` states and the content `held` holds differ, if they do.
///
/// A requirement's content is its statements, in order, and its scenarios, less those the change
/// marks withdrawn, as a collection. Each is compared as projected text after whitespace
/// normalization, which is an exact comparison and not a reading for meaning. Notes, the
/// `renamed-from` paragraph, the requirement's own heading, and every unit's change state are not
/// content: the first two are not fields a merge applies, the heading is the rename comparison's,
/// and a change state differs between a change's specification and an accepted one by design.
fn content_difference(requirement: &Requirement, held: &Requirement) -> Option<ContentDifference> {
    let first_differing_statement = first_differing_statement(
        &normalized_all(&requirement.statements),
        &normalized_all(&held.statements),
    );

    // Matched as a multiset: each scenario the change keeps takes one accepted scenario with the
    // same text, and what is left on either side is the difference. Order is not compared, because
    // a scenario is a unit a reader checks on its own.
    let mut unmatched: Vec<(usize, ScenarioText)> = held
        .scenarios
        .iter()
        .enumerate()
        .map(|(position, scenario)| (position, ScenarioText::of(scenario)))
        .collect();
    let mut scenarios_not_held = Vec::new();
    for (position, scenario) in requirement.scenarios.iter().enumerate() {
        if ContractEffect::of(&scenario.change) == ContractEffect::Withdraws {
            continue;
        }
        let text = ScenarioText::of(scenario);
        match unmatched.iter().position(|(_, accepted)| *accepted == text) {
            Some(matched) => {
                unmatched.remove(matched);
            }
            None => scenarios_not_held.push(position),
        }
    }
    let scenarios_not_kept: Vec<usize> = unmatched
        .into_iter()
        .map(|(position, _)| position)
        .collect();

    if first_differing_statement.is_none()
        && scenarios_not_held.is_empty()
        && scenarios_not_kept.is_empty()
    {
        return None;
    }
    Some(ContentDifference {
        first_differing_statement,
        scenarios_not_held,
        scenarios_not_kept,
    })
}

/// How a report names the scenarios at `positions` among `scenarios`.
fn scenarios_named(scenarios: &[Scenario], positions: &[usize]) -> Vec<String> {
    positions
        .iter()
        .filter_map(|&position| {
            scenarios
                .get(position)
                .map(|scenario| scenario_named(scenario, position))
        })
        .collect()
}

/// The first statement position, counted from one, at which `stated` and `held` differ.
///
/// Where one has fewer paragraphs and they agree up to there, it is the first position the shorter
/// one lacks. `None` where they agree.
fn first_differing_statement(stated: &[String], held: &[String]) -> Option<std::num::NonZeroUsize> {
    let position = stated
        .iter()
        .zip(held)
        .position(|(stated, held)| stated != held)
        .or_else(|| (stated.len() != held.len()).then(|| stated.len().min(held.len())))?;
    Some(std::num::NonZeroUsize::MIN.saturating_add(position))
}

/// A scenario's content: its heading name and its entries, each in order.
///
/// The comparison's one notion of a scenario's text, which the view also reads to find a withdrawn
/// scenario's counterpart where no identifier does.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ScenarioText {
    heading: Option<HeadingName>,
    given: Vec<String>,
    when: Vec<String>,
    then: Vec<String>,
}

impl ScenarioText {
    pub(crate) fn of(scenario: &Scenario) -> Self {
        Self {
            heading: scenario
                .title
                .as_deref()
                .map(|title| HeadingName::of(title, scenario.id.as_deref())),
            given: normalized_all(&scenario.given),
            when: normalized_all(&scenario.when),
            then: normalized_all(&scenario.then),
        }
    }
}

/// How a report names `scenario`, the scenario at `position` in its requirement: by the identifier
/// its own document gives it, by its heading where it has none, and by its position where it has
/// neither.
fn scenario_named(scenario: &Scenario, position: usize) -> String {
    scenario
        .id
        .clone()
        .or_else(|| {
            scenario
                .title
                .as_deref()
                .map(|title| HeadingName::of(title, None).as_str().to_owned())
        })
        .unwrap_or_else(|| format!("scenario {}", position + 1))
}

/// Each text with every run of whitespace collapsed to one space.
pub(crate) fn normalized_all(texts: &[String]) -> Vec<String> {
    texts
        .iter()
        .map(|text| text.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

/// The rename `requirement` states that no accepted requirement carrying `id` has taken.
///
/// `None` where the requirement states no rename, where it has no heading to compare, or where
/// an accepted requirement already carries its heading — the last being OpenSpec's "source gone,
/// target present", a rename applied before archival, which is the ordinary case and not a
/// conflict.
fn untaken_rename(
    requirement: &Requirement,
    id: &str,
    accepted: &[Option<HeadingName>],
) -> Option<UntakenRename> {
    requirement.previous_heading()?;
    let heading = requirement.heading_name()?;
    if accepted.iter().flatten().any(|held| *held == heading) {
        return None;
    }
    Some(UntakenRename {
        id: id.to_owned(),
        heading,
        accepted: accepted.iter().flatten().next().cloned(),
    })
}

/// Reports the findings, naming each one.
///
/// Named individually rather than counted. A contributor can check three identifiers in under a
/// minute and cannot check "3 requirements", and a prompt answered yes every time is a prompt
/// nobody reads.
fn report_findings<O: Write, E: Write>(findings: &Findings, streams: &mut Streams<O, E>) {
    if !findings.open_tasks.is_empty() {
        let _ = streams.progress(&format!(
            "note: {} task(s) are not done or dropped: {}",
            findings.open_tasks.len(),
            findings.open_tasks.join(", ")
        ));
    }
    if !findings.merge.absent.is_empty() {
        // The operation is named, because there now is one. Until 0.7.0 this note told a
        // contributor to apply the change's contract and no command did that, which is the
        // fault `sync` was added to repair — leaving the instruction unnamed here would keep
        // the reader doing the same search through the command surface.
        let _ = streams.progress(&format!(
            "note: {} requirement(s) this change states appear in no accepted specification: \
             {}. This change has not been synced. Run `htmlspec sync --change {}` to see what \
             remains, apply the change's contract, or confirm each one was abandoned",
            findings.merge.absent.len(),
            findings.merge.absent.join(", "),
            findings.change,
        ));
    }
    if !findings.merge.untaken_renames.is_empty() {
        // Its own note rather than a clause of the one above, because the work differs: nothing
        // is missing from the accepted specifications, and the requirement there needs its
        // heading changed rather than a requirement merged in.
        let _ = streams.progress(&format!(
            "note: {} requirement(s) this change renames still carry another heading in the \
             accepted specifications: {}. This change has not been synced. Run `htmlspec sync \
             --change {}` to see what remains, and give each accepted requirement the heading \
             this change states, keeping its identifier",
            findings.merge.untaken_renames.len(),
            findings
                .merge
                .untaken_renames
                .iter()
                .map(UntakenRename::described)
                .collect::<Vec<_>>()
                .join(", "),
            findings.change,
        ));
    }
    if !findings.merge.untaken_content.is_empty() {
        // Its own note, for the reason the rename's is one: nothing is missing, and the work is
        // to edit a requirement the accepted specifications already hold. It names where each
        // differs and never what either side says.
        let _ = streams.progress(&format!(
            "note: {} requirement(s) this change adds or modifies hold other content in the \
             accepted specifications: {}. This change has not been synced. Run `htmlspec sync \
             --change {}` to see what remains, and give each accepted requirement the statements \
             and scenarios this change states",
            findings.merge.untaken_content.len(),
            findings
                .merge
                .untaken_content
                .iter()
                .map(UntakenContent::described)
                .collect::<Vec<_>>()
                .join(", "),
            findings.change,
        ));
    }
    if !findings.merge.untaken_removals.is_empty() {
        // And its own again, because the work is a deletion, which no other note asks for.
        let _ = streams.progress(&format!(
            "note: {} requirement(s) this change withdraws are still held in the accepted \
             specifications: {}. This change has not been synced. Run `htmlspec sync --change {}` \
             to see what remains, and delete each from the accepted specification that holds it",
            findings.merge.untaken_removals.len(),
            findings.merge.untaken_removals.join(", "),
            findings.change,
        ));
    }
    if let Some(follows) = &findings.unresolved_predecessor {
        // The two unresolved outcomes get different sentences, because they ask for different
        // work. Naming an unmatched predecessor as unfinished would send a contributor looking
        // for work that may already be done under another name.
        let _ = streams.progress(&match follows.resolution {
            crate::sequence::Resolution::Unmatched => format!(
                "note: this change follows {}, and no active or archived change carries that \
                 name. It was renamed, or the name is wrong — correct the declaration rather \
                 than waiting on it",
                follows.predecessor
            ),
            _ => format!(
                "note: this change follows {}, which is not archived. Archive that one first, \
                 or state that this change no longer follows it",
                follows.predecessor
            ),
        });
    }
    if !findings.cycle.is_empty() {
        // A contradiction rather than an ordering violation, and reported as one. The program
        // names the changes involved and chooses no order between them: the contributor who
        // wrote both declarations is the one who can say which is wrong.
        let _ = streams.progress(&format!(
            "note: the changes {} each follow one another, so the ordering they state is a \
             contradiction. No order holds among them until one of those declarations changes",
            findings.cycle.join(", ")
        ));
    }
}

/// Asks whether to archive over the findings.
///
/// `None` where nothing can be asked: no terminal, or a structured run whose standard output
/// is promised to the payload alone. This function decides nothing — it carries an answer back
/// to [`decision`], which is where the deciding is and where the tests are.
fn ask<O: Write, E: Write>(findings: &Findings, streams: &mut Streams<O, E>) -> Option<bool> {
    if !crate::prompt::can_confirm(streams.form().is_structured()) {
        return None;
    }
    Some(crate::prompt::confirm(&question_for(findings)).unwrap_or(false))
}

/// The question archival asks about what it found.
///
/// A pure function of the findings, so the wording is testable without a terminal — the same
/// reason [`decision`] is separated from the prompt that feeds it.
///
/// **An unsynced change gets a question about syncing.** "Archive anyway?" invites a yes, and
/// answering it archives a change whose contract never reached the project's specifications.
/// Naming the operation the contributor would otherwise run first makes declining the useful
/// answer rather than the timid one.
fn question_for(findings: &Findings) -> String {
    let count = findings.open_tasks.len()
        + findings.merge.absent.len()
        + findings.merge.untaken_renames.len()
        + findings.merge.untaken_content.len()
        + findings.merge.untaken_removals.len()
        + usize::from(findings.out_of_order())
        + usize::from(!findings.cycle.is_empty());
    // The predecessor is named ahead of the sync, because it is the finding that cannot be
    // repaired after the fact. An unsynced contract can be applied the moment afterwards; a
    // sequence archived out of order is a sequence whose argument was already lost.
    if let Some(follows) = &findings.unresolved_predecessor {
        return format!(
            "This change follows {}, which is not archived. Archive anyway, over {count} \
             finding(s)? Answering no leaves everything as it is, so you can archive {} first",
            follows.predecessor, follows.predecessor,
        );
    }
    if findings.needs_sync() {
        return format!(
            "This change has not been synced. Archive anyway, over {count} finding(s)? \
             Answering no leaves everything as it is, so you can run `htmlspec sync --change \
             {}` first",
            findings.change,
        );
    }
    format!("Archive anyway, over {count} finding(s)?")
}

/// Moves the change into the archive and records the version it shipped in.
fn perform<O: Write, E: Write>(
    project: &ProjectRoot,
    directory: &Path,
    streams: &mut Streams<O, E>,
    findings: &Findings,
    staged: &StagedReleases,
) -> Outcome {
    let name = directory
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let target = project.archive().join(archived_name(&name, &today()));

    if target.exists() {
        let _ = streams.failure(&format!(
            "{} already exists. A collision means this change was archived already, or that \
             two changes share a name on one day — both want you rather than a suffix",
            target.display()
        ));
        return Outcome::OperationalFailure;
    }

    if let Err(error) = std::fs::create_dir_all(project.archive()) {
        let _ = streams.failure(&format!(
            "cannot create {}: {error}",
            project.archive().display()
        ));
        return Outcome::OperationalFailure;
    }

    // **A staged publication left behind by an interrupted one is refused rather than overwritten.**
    // The inventory and the recorded version are written into the change directory so that the rename
    // carries them, which means a crash between staging and moving leaves both files in an *active*
    // change — a place this program never writes them. Finding one there says the last publication did
    // not finish, and the honest answer is to stop: overwriting would publish a release nobody
    // compared, and quietly removing it would destroy the only evidence of what the interrupted run
    // intended. Both files are named, because the fix is to look at them.
    for leftover in [
        crate::runbook_release::INVENTORY_FILE,
        crate::project::RECORDED_VERSION,
    ] {
        let path = directory.join(leftover);
        if path.exists() {
            let _ = streams.failure(&format!(
                "{} already holds a {}, which this operation writes into a change only when it is \
                 about to archive it. An interrupted publication is the way one gets here, and \
                 archival will not overwrite it: look at the file, remove it if the run it belongs \
                 to is abandoned, and run this again",
                directory.display(),
                leftover
            ));
            return Outcome::OperationalFailure;
        }
    }
    // A staged document is the same evidence for the rewrite: archival writes one into a change only
    // when it is about to move it.
    if let Some(leftover) = staged_documents(directory).first() {
        let _ = streams.failure(&format!(
            "{} already holds {}, a document this operation stages in a change only when it is about \
             to archive it. An interrupted archival is the way one gets here, and archival will not \
             overwrite it: look at the file, remove it if the run it belongs to is abandoned, and \
             run this again",
            directory.display(),
            leftover.display()
        ));
        return Outcome::OperationalFailure;
    }

    // Each of the change's documents, rewritten so its references reach from the archive what they
    // reached from the change. Computed before anything is written, so a document that cannot be read
    // refuses the archival while there is still nothing to undo.
    let rewrites = match rewritten_documents(directory, &target) {
        Ok(rewrites) => rewrites,
        Err(report) => {
            let _ = streams.failure(&format!("{report}. Nothing moved"));
            return Outcome::OperationalFailure;
        }
    };

    // The journal is written before any of it, and names every file this transaction owns. An
    // interruption after this point is recoverable by reading it; an interruption before it left
    // nothing to recover, which is the property that makes the ordering safe.
    let inventory_document = staged.inventory.render();
    let declared = crate::declaration::read(&project.declaration())
        .map(|declaration| declaration.version.to_string())
        .unwrap_or_default();
    let version_document = format!("{declared}\n");
    // Absolute before anything is journaled: the change can be named relative to the directory this
    // command runs in, and recovery may run from another one.
    let change = match std::path::absolute(directory) {
        Ok(change) => change,
        Err(error) => {
            let _ = streams.failure(&format!(
                "cannot resolve {} to an absolute path for the journal: {error}. Nothing moved",
                directory.display()
            ));
            return Outcome::OperationalFailure;
        }
    };
    let mut writes = vec![
        (
            change.join(crate::runbook_release::INVENTORY_FILE),
            &inventory_document,
        ),
        (
            change.join(crate::project::RECORDED_VERSION),
            &version_document,
        ),
    ];
    let marker = project.runbook_releases();
    let marker_document = staged.adoption.map(|adoption| adoption.render());
    if let Some(document) = &marker_document {
        writes.push((marker.clone(), document));
    }
    let journal = Journal::new(
        change,
        target.clone(),
        writes
            .iter()
            .map(|(path, document)| {
                (
                    path.clone(),
                    crate::runbook_release::ContentDigest::of(document.as_bytes()),
                )
            })
            .collect(),
        rewrites
            .iter()
            .map(|rewrite| Replacement {
                document: rewrite.document.clone(),
                digest: crate::runbook_release::ContentDigest::of(rewrite.source.as_bytes()),
            })
            .collect(),
    )
    .and_then(|journal| journal.publish(project).map(|()| journal));
    let journal = match journal {
        Ok(journal) => journal,
        Err(report) => {
            let _ = streams.failure(&format!(
            "cannot record what this archival is about to write: {report}. Nothing moved, and this \
             will not publish without a journal — an interrupted publication nothing can account \
             for is the state the journal exists to prevent"
        ));
            return Outcome::OperationalFailure;
        }
    };

    // The marker is written where the journal says it will be, so recovery can tell an interrupted
    // publication from somebody else's file.
    let staged_paths: Vec<PathBuf> = journal
        .writes
        .iter()
        .map(|(path, _)| path.clone())
        .collect();
    let staged_bytes: Vec<&str> = writes
        .iter()
        .map(|(_, document)| document.as_str())
        .collect();

    // The runbook inventory travels with the change rather than being written after it, and that is
    // the ordering the guarantee rests on. `project-bootstrap` requires publication to happen "in the
    // same archival transaction as the change and project version", and a rename is the only atomic
    // step available here: a file written into the change directory before the move arrives in the
    // archive with it, and there is no moment at which the archive exists holding a change but no
    // inventory.
    // Every file the journal named, written where the journal said. A failure here removes the ones
    // already written and clears the journal, so a change directory this operation could not archive
    // looks exactly as it did before: leaving a staged inventory behind would make the retry publish
    // an inventory nobody compared, and would leave a file in an active change that belongs to an
    // archive.
    let mut written: Vec<PathBuf> = Vec::new();
    for (path, document) in staged_paths.iter().zip(&staged_bytes) {
        // The marker lives at the planning root rather than inside the change, and it is published
        // only where nothing is published yet: a rename cannot carry it, and `create_new` is what
        // makes "never advanced on a later release" a property of the write.
        let result = if path == &marker {
            publish_without_replacement(path, document)
        } else {
            std::fs::write(path, document)
                .map_err(|error| format!("{} could not be written: {error}", path.display()))
        };
        if let Err(report) = result {
            for done in &written {
                let _ = std::fs::remove_file(done);
            }
            journal.clear(project);
            let _ = streams.failure(&format!(
                "{report}. Nothing moved, and what this archival had staged is removed"
            ));
            return Outcome::OperationalFailure;
        }
        written.push(path.clone());
    }
    // Each rewritten document is staged beside the one it replaces, and the original is not touched
    // until the change has moved: a refusal or an interruption before the move removes the staged copy
    // and leaves the document exactly as it was.
    for rewrite in &rewrites {
        let path = rewrite.document.staged_in(&journal.change);
        if let Err(report) = write_new(&path, &rewrite.source) {
            for done in &written {
                let _ = std::fs::remove_file(done);
            }
            journal.clear(project);
            let _ = streams.failure(&format!(
                "{report}. Nothing moved, and what this archival had staged is removed"
            ));
            return Outcome::OperationalFailure;
        }
        written.push(path);
    }

    // A rename within one filesystem, so the change is either where it was or where it is
    // going and never in both places or neither. This is the commit point: everything before it
    // writes into the active change and is undone on failure, and everything after it writes into an
    // archive that already holds the change.
    if let Err(error) = std::fs::rename(directory, &target) {
        for path in &written {
            let _ = std::fs::remove_file(path);
        }
        journal.clear(project);
        let _ = streams.failure(&format!(
            "cannot archive {} to {}: {error}. Nothing moved, and what this archival had staged is \
             removed",
            directory.display(),
            target.display()
        ));
        return Outcome::OperationalFailure;
    }

    // The move carried the staged files into the archive. Each rewritten document now takes its
    // document's place, inside an archive that already holds the change, so a failure here leaves an
    // archived change and a journal that lets the next run finish the replacements.
    for rewrite in &rewrites {
        let copy = rewrite.document.staged_in(&target);
        let document = rewrite.document.in_directory(&target);
        if let Err(error) = std::fs::rename(&copy, &document) {
            let _ = streams.failure(&format!(
                "archived to {}, but {} could not take the place of {}: {error}. The journal at {} \
                 is kept, so running `htmlspec archive` again completes it",
                target.display(),
                copy.display(),
                document.display(),
                journal_path(project).display()
            ));
            return Outcome::OperationalFailure;
        }
    }

    // Cleared here rather than earlier: until the rename succeeds and every document is in place the
    // transaction is not complete, and a journal removed too soon is an interruption nothing can
    // account for.
    journal.clear(project);

    let _ = streams.report(&format!(
        "archived to {} at version {declared}\n",
        target.display()
    ));
    if !rewrites.is_empty() {
        let _ = streams.report(&format!(
            "  re-expressed {} reference(s) in {} document(s), so each reaches from the archive what \
             it reached from the change\n",
            rewrites.iter().map(|rewrite| rewrite.references).sum::<usize>(),
            rewrites.len()
        ));
    }
    if !staged.inventory.is_empty() || staged.adoption.is_some() {
        let _ = streams.report(&format!(
            "  published {} runbook release(s) in {}{}\n",
            staged.inventory.len(),
            crate::runbook_release::INVENTORY_FILE,
            match staged.adoption {
                Some(adoption) => format!(
                    ", and adopted publication at project version {} in {}",
                    adoption.version,
                    crate::runbook_release::ADOPTION_FILE
                ),
                None => String::new(),
            }
        ));
    }
    emit_archive_payload(streams, findings, true, Some(&target.to_string_lossy()));
    Outcome::Success
}

/// One of a change's documents, rewritten for the archive.
struct RewrittenDocument {
    /// The document it replaces.
    document: DocumentName,
    /// Its source, with each reference re-expressed.
    source: String,
    /// How many references were re-expressed, for the report.
    references: usize,
}

/// Each of the change's documents whose references would reach other files from `target`, rewritten
/// so they reach the same ones.
///
/// The change's documents are the HTML files directly within its directory, as document-schema's
/// "Cross-references address units by identifier" names them. Evidence is beneath the directory and
/// is never read: artifact-kinds' "A change's evidence is kept in the change and is not one of its
/// documents" forbids rewriting it.
///
/// **Both places are resolved through the filesystem first**, because the references are compared
/// path against path. A change named as `/tmp/…` and an archive found from a working directory that
/// the system reports as `/private/tmp/…` are one place spelled two ways, and relating them lexically
/// would write a reference that climbs to the root. The archive exists by now; the archived change
/// does not, so it is placed in the resolved archive by name.
fn rewritten_documents(directory: &Path, target: &Path) -> Result<Vec<RewrittenDocument>, String> {
    let (Some(archive), Some(name)) = (target.parent(), target.file_name()) else {
        return Err(format!("{} names no archived change", target.display()));
    };
    let moved = htmlspec_format::Move::between(
        &crate::project::canonical(directory),
        &crate::project::canonical(archive).join(name),
    )
    .map_err(|htmlspec_format::NotAbsolute(path)| {
        format!(
            "cannot resolve {} to an absolute path, so its references cannot be re-expressed for \
             the archive",
            path.display()
        )
    })?;

    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("{} could not be read: {error}", directory.display()))?;
    let mut documents: Vec<(DocumentName, PathBuf)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "html")
        })
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            Some((DocumentName::parse(name).ok()?, path))
        })
        .collect();
    documents.sort_by(|one, other| one.1.cmp(&other.1));

    let mut rewrites = Vec::new();
    for (document, path) in documents {
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("{} could not be read: {error}", path.display()))?;
        let relocated = htmlspec_format::relocate(&source, &moved);
        if !relocated.is_unchanged() {
            rewrites.push(RewrittenDocument {
                document,
                references: relocated.rewritten.len(),
                source: relocated.source,
            });
        }
    }
    Ok(rewrites)
}

/// Every staged document in `directory`, which only an interrupted archival leaves there.
fn staged_documents(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut staged: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.len() > 1 + STAGED_SUFFIX.len()
                && name.starts_with('.')
                && name.ends_with(STAGED_SUFFIX)
        })
        .map(|entry| entry.path())
        .collect();
    staged.sort();
    staged
}

/// Writes `contents` to `path`, which must not exist, and leaves nothing behind where it fails.
///
/// Not atomic: a run killed while writing leaves part of `contents` at `path`. That is safe for a
/// staged document because the journal is published first, and recovery removes a staged copy in a
/// change that never moved whatever it holds ([`recover_publication`]).
fn write_new(path: &Path, contents: &str) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{} could not be written: {error}", path.display()))?;
    file.write_all(contents.as_bytes()).map_err(|error| {
        let _ = std::fs::remove_file(path);
        format!("{} could not be written: {error}", path.display())
    })
}

/// Writes a file that must not replace one, and reports why it could not.
///
/// **A link rather than a rename, and the reason is the same one a run's record gives.** `rename`
/// replaces its destination, so a marker written over an existing one would silently move the
/// beginning of the recorded history — and `project-bootstrap` requires that the marker is "never
/// advanced on later releases". `link(2)` fails where the name is taken, which makes "publish only
/// where nothing is published" a property of the call rather than of a check performed before it.
///
/// The staging file is a [`crate::runs::Temporary`]: created exclusively under a unique name, so a
/// file somebody else left beside the destination is never overwritten or removed, and two writers
/// cannot stage through one name. It removes itself on every path out of this function.
fn publish_without_replacement(path: &Path, contents: &str) -> Result<(), String> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let label = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temporary = crate::runs::Temporary::in_directory(directory, &label)?;
    if let Err(error) = std::fs::write(temporary.path(), contents) {
        return Err(format!(
            "{} could not be written: {error}",
            temporary.path().display()
        ));
    }
    match std::fs::hard_link(temporary.path(), path) {
        Ok(()) => {
            let _ = temporary.remove();
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(format!(
            "{} already exists, and this publication never replaces one: it names the first \
                 project version subject to runbook inventories, so overwriting it would move the \
                 beginning of the recorded history",
            path.display()
        )),
        Err(error) => Err(format!(
            "{} could not be published: {error}",
            path.display()
        )),
    }
}

/// The archived directory's name: the date, then the change's name.
///
/// A change already named with a leading date keeps the one it has. Archiving twice must not
/// produce `2026-08-30-2026-08-29-a-change`, and the second date would be the wrong one anyway.
fn archived_name(name: &str, today: &str) -> String {
    if leads_with_a_date(name) {
        return name.to_owned();
    }
    format!("{today}-{name}")
}

/// Whether a name begins `YYYY-MM-DD-`.
pub fn leads_with_a_date(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() < 11 {
        return false;
    }
    let digits = |range: std::ops::Range<usize>| bytes[range].iter().all(u8::is_ascii_digit);
    digits(0..4)
        && bytes[4] == b'-'
        && digits(5..7)
        && bytes[7] == b'-'
        && digits(8..10)
        && bytes[10] == b'-'
}

/// Today, as `YYYY-MM-DD` in UTC.
///
/// Computed from the civil-date algorithm rather than taken from a date library, because the
/// crate has no other need for one and this is the whole of what it would be used for.
fn today() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default();
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Howard Hinnant's `civil_from_days`, for days since the Unix epoch.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The structured payload for an archival run.
fn emit_archive_payload<O: Write, E: Write>(
    streams: &mut Streams<O, E>,
    findings: &Findings,
    moved: bool,
    target: Option<&str>,
) {
    if !streams.form().is_structured() {
        return;
    }
    let payload = serde_json::json!({
        "moved": moved,
        "archivedTo": target,
        "openTasks": findings.open_tasks,
        "unmergedRequirements": findings.merge.absent,
        // Each naming the identifier and both headings. A rename's identifier is held, so it is
        // not among the unmerged requirements above, and it is still a reason `needsSync` is true.
        "untakenRenames": findings
            .merge
            .untaken_renames
            .iter()
            .map(UntakenRename::payload)
            .collect::<Vec<_>>(),
        // The same two sets `sync` carries, in the same shape, so an agent reading either payload
        // reads one comparison. Each is a reason `needsSync` is true.
        "untakenContent": findings
            .merge
            .untaken_content
            .iter()
            .map(UntakenContent::payload)
            .collect::<Vec<_>>(),
        "untakenRemovals": findings.merge.untaken_removals,
        // Stated rather than left to be inferred from the array above. An agent driving
        // archival cannot answer the prompt a contributor gets, so the condition that prompt
        // asks about has to be a field it can read.
        "needsSync": findings.needs_sync(),
        // The same reasoning, for the same reason: an agent archiving out of order cannot be
        // asked about it, so the condition is a value beside the findings rather than something
        // to infer from the object below. `null` where nothing was found, since "follows nothing"
        // and "follows something that is done" are both silence.
        "outOfOrder": findings.out_of_order(),
        "unresolvedPredecessor": findings.unresolved_predecessor.as_ref().map(|follows| {
            serde_json::json!({
                "change": follows.predecessor,
                "resolution": follows.resolution.label(),
            })
        }),
        "followsCycle": findings.cycle,
    });
    if let Ok(json) = serde_json::to_string_pretty(&payload) {
        let _ = streams.payload(&json);
    }
}

/// Creates a whole change directory and the documents that stage needs.
///
/// A change is a directory of kinded documents rather than one file carrying every
/// stage, so creating one is a directory operation. Each document declares its kind
/// and each validates immediately, which is checked here rather than promised.
///
/// **The location is not an argument.** A change goes where its project keeps changes,
/// and the resolved path is reported so the contributor knows where it went. Accepting a
/// path would let a change be created outside `changes/`, and a project root that does
/// not determine where changes live is not doing the job the root exists for.
pub fn new_change<O: Write, E: Write>(
    name: &str,
    subject: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let project = match ProjectRoot::resolve(Path::new(".")) {
        Ok(project) => project,
        Err(error) => {
            let _ = streams.failure(&format!(
                "{error}. A change belongs to a project, because that is where its \
                 documents find the asset store they render against"
            ));
            return Outcome::OperationalFailure;
        }
    };
    new_change_in(&project, name, subject, streams)
}

/// Creates a change in a named project.
///
/// Split from [`new_change`] so the project is an argument rather than something read
/// from the process. A test that had to change the working directory to name its project
/// would race every other test in the binary, and the resolution is already covered
/// where it lives.
fn new_change_in<O: Write, E: Write>(
    project: &ProjectRoot,
    name: &str,
    subject: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    // The name is checked before anything is composed from it. **This is where a measured wedge
    // came from**: `htmlspec new change ../runbooks/sneaky` created a change directory inside
    // `runbooks/`, which then listed as a runbook, was invisible to `htmlspec list`, and was
    // refused by every change-taking operation. The check lives on the constructor rather than
    // here, because this was the third call site at which the same invariant was re-litigated.
    let directory = match project.named_change(name) {
        Ok(directory) => directory,
        Err(error) => {
            let _ = streams.failure(&error.to_string());
            return Outcome::OperationalFailure;
        }
    };
    // An existing directory is refused whole. Writing into one would mean deciding
    // what to do about a proposal already there, and `new` does not overwrite a
    // document — so a partial change directory would be half created and half
    // refused, which is worse than not starting.
    if directory.exists() {
        let _ = streams.failure(&format!(
            "{} already exists; new will not write into an existing change directory",
            directory.display()
        ));
        return Outcome::OperationalFailure;
    }

    // The name is the subject when none is given, which is the common case: a change
    // named `add-dark-mode` is about adding dark mode.
    let subject = subject.unwrap_or(name);
    let assets_root = project.htmlspec();

    for (file, kind) in CHANGE_DOCUMENTS {
        // The proposal takes the change's subject unqualified, because it is the
        // artifact the change is named for. The later stages qualify it, so three
        // browser tabs are distinguishable by their titles.
        let title = if *kind == htmlspec_assets::DEFAULT_KIND {
            subject.to_owned()
        } else {
            format!("{subject}: {kind}")
        };
        let outcome = write_document(
            &directory.join(file),
            &title,
            kind,
            Some(&assets_root),
            streams,
        );
        if outcome != Outcome::Success {
            return outcome;
        }
    }

    let _ = streams.report(&format!(
        "{}: created a change with {} document(s). The specification is produced at \
         archival, from the completed change.\n",
        directory.display(),
        CHANGE_DOCUMENTS.len()
    ));
    Outcome::Success
}

/// The version a runbook `new runbook` writes is released at, until its first change.
const FIRST_RUNBOOK_VERSION: &str = "0.1.0";

/// Creates a runbook: its specification, its tasks document, its `VERSION`, and its changelog.
///
/// The change's design, decision 7. The project is resolved from the working directory, as
/// [`new_change`] resolves it, because the runbook's location is the project's to decide, and the
/// name goes through [`ProjectRoot::named_runbook`], which refuses anything but a plain name.
///
/// Every unit the two documents hold is `unchanged` and every task `todo`: runbook-authority's "The
/// canonical procedure carries no completion state" requires it of a runbook, and this operation is
/// the second exception that requirement names beside the one that starts a run. The templates'
/// example identifiers and placeholder summaries are reported by validation, as every template's are.
pub fn new_runbook<O: Write, E: Write>(
    name: &str,
    subject: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let project = match ProjectRoot::resolve(Path::new(".")) {
        Ok(project) => project,
        Err(error) => {
            let _ = streams.failure(&format!(
                "{error}. A runbook belongs to a project, because that is where its documents find \
                 the asset store they render against and where its runs are recorded"
            ));
            return Outcome::OperationalFailure;
        }
    };
    new_runbook_in(&project, name, subject, streams)
}

/// Creates a runbook in a named project, for the reason [`new_change_in`] takes its project.
fn new_runbook_in<O: Write, E: Write>(
    project: &ProjectRoot,
    name: &str,
    subject: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let directory = match project.new_runbook(name) {
        Ok(directory) => directory,
        Err(error) => {
            let _ = streams.failure(&error.to_string());
            return Outcome::OperationalFailure;
        }
    };
    // Refused whole, for the reason `new change` refuses an existing directory: a runbook half
    // written into a directory that held something is half created and half refused. Asked of the
    // entry itself rather than of what it points at: `exists` follows a link, so a dangling one read
    // as nothing there, and the cleanup below once removed the contributor's link.
    if std::fs::symlink_metadata(&directory).is_ok() {
        let _ = streams.failure(&format!(
            "{} already exists; new will not write into an existing runbook directory",
            directory.display()
        ));
        return Outcome::OperationalFailure;
    }

    let subject = subject.unwrap_or(name);

    // **All four files, or none.** Both documents are rendered and validated before the runbook's
    // directory exists, against the store they will render with, and a failure after that removes
    // the directory. PR #80's review found the earlier order, writing each document and then
    // validating it, left a directory holding one document after the second failed, and that
    // directory was a runbook nothing could run or create again.
    let assets = project.assets();
    let Some(reference) = relative_reference(&directory, &assets) else {
        let _ = streams.failure(&format!(
            "cannot express {} relative to {}",
            assets.display(),
            directory.display()
        ));
        return Outcome::OperationalFailure;
    };
    if let Err(error) = establish_assets(&assets, streams) {
        let _ = streams.failure(&error);
        return Outcome::OperationalFailure;
    }
    let mut files: Vec<(PathBuf, String)> = Vec::new();
    for template in htmlspec_assets::RUNBOOK_TEMPLATES {
        let path = directory.join(template.file);
        let document = htmlspec_assets::render_runbook(template, subject, &reference);
        let report = validate_source(&document, &path, &mut Reach::default(), Strictness::Default);
        if !report.is_accepted() {
            let _ = streams.failure(&format!(
                "the runbook's {} would not validate, so nothing was written:\n{}",
                template.file,
                htmlspec_validate::render_human(&report)
            ));
            return Outcome::OperationalFailure;
        }
        files.push((path, document));
    }
    // The pair a carried runbook identifies its release by: runbook-authority's "A runbook carries
    // an independently checked release version". Without both, `preflight` and `run` refuse it.
    files.push((
        directory.join(crate::runbook_release::VERSION_FILE),
        format!("{FIRST_RUNBOOK_VERSION}\n"),
    ));
    files.push((
        directory.join(crate::runbook_release::CHANGELOG_FILE),
        first_runbook_changelog(),
    ));

    // The runbook's own directory is made with `create_dir`, which fails rather than reusing an
    // entry that appeared since the check above, so a directory removed below is one this run made.
    if let Err(error) =
        std::fs::create_dir_all(project.runbooks()).and_then(|()| std::fs::create_dir(&directory))
    {
        let _ = streams.failure(&format!(
            "cannot create {}: {error}. Nothing was written",
            directory.display()
        ));
        return Outcome::OperationalFailure;
    }
    let written = files
        .iter()
        .try_for_each(|(path, contents)| std::fs::write(path, contents));
    if let Err(error) = written {
        let _ = std::fs::remove_dir_all(&directory);
        let _ = streams.failure(&format!(
            "cannot write {}: {error}. Nothing was left behind",
            directory.display()
        ));
        return Outcome::OperationalFailure;
    }
    for template in htmlspec_assets::RUNBOOK_TEMPLATES {
        let _ = streams.report(&format!(
            "{}: created a {}\n",
            directory.join(template.file).display(),
            template.kind
        ));
    }

    let _ = streams.report(&format!(
        "{}: created a runbook at {FIRST_RUNBOOK_VERSION}. Replace its example content, check it \
         with `htmlspec preflight {name}`, and start a run with `htmlspec run {name} --as \
         <record>`.\n",
        directory.display()
    ));
    Outcome::Success
}

/// The changelog a new runbook starts with: what a release is, and a heading for the first one.
fn first_runbook_changelog() -> String {
    format!(
        "# Changelog\n\n\
         What this runbook's releases are. A release is the pair of canonical documents, \
         `specification.html` and `tasks.html`, as they stood when the project archived them, and \
         a version identifies one such pair. A run's record is history rather than procedure, so \
         adding or editing one is not a release.\n\n\
         ## {FIRST_RUNBOOK_VERSION}\n\n\
         The first release of this runbook.\n"
    )
}

/// What a bootstrap does, as plain data.
///
/// This is the seam the design insists on. A prompt's whole job is producing one of
/// these, and [`bootstrap`] consumes one and never reads a keypress — so every case
/// worth testing is a constructed value rather than a simulated terminal. Without the
/// split, the interactive path is the one part of the program with no test, and it is
/// also the first part a new contributor meets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    /// Which tools receive instructions.
    ///
    /// An empty vector means no tools, which is a legitimate answer. That differs from
    /// never having asked, and the difference is carried by whether a `Selection` exists
    /// at all rather than by a sentinel inside one.
    pub tools: Vec<ToolId>,
    /// The project's name, or `None` to derive it from the directory.
    pub name: Option<String>,
}

/// Establishes a project: its directories, its one asset store, its declaration, and the
/// workflow instructions for the tools selected.
///
/// Takes no terminal. Safe to re-run: an established project has its instructions
/// refreshed and its specifications, changes, and archive left alone.
pub fn bootstrap<O: Write, E: Write>(
    root: &Path,
    selection: &Selection,
    streams: &mut Streams<O, E>,
) -> Outcome {
    report_banner(streams);

    // Extend mode is decided before anything is created, because `establish` creates the
    // directory that would answer the question.
    let extending = ProjectRoot::exists(root);

    let project = match ProjectRoot::establish(root) {
        Ok(project) => project,
        Err(error) => {
            let _ = streams.failure(&format!(
                "cannot establish a project in {}: {error}",
                root.display()
            ));
            return Outcome::OperationalFailure;
        }
    };

    if extending {
        let _ = streams.progress(&format!(
            "extending the project in {}",
            project.path().display()
        ));
    } else {
        let _ = streams.report(&format!(
            "{}: created the project structure\n",
            project.htmlspec().display()
        ));
    }

    // One store, at the project root. Every document the project holds references this
    // one, which is what keeps a project to a single copy of its runtime.
    if let Err(error) = establish_assets(&project.assets(), streams) {
        let _ = streams.failure(&error);
        return Outcome::OperationalFailure;
    }
    // An existing store is checked against the pinned runtime rather than overwritten.
    // Bytes that differ from the pin are a fact the contributor needs, and replacing them
    // silently would hide it.
    report_store_drift(&project, streams);

    let name = selection.name.clone().unwrap_or_else(|| {
        project
            .path()
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("htmlspec-project"))
    });
    if let Err(error) = write_declaration(&project, &name, streams) {
        let _ = streams.failure(&error);
        return Outcome::OperationalFailure;
    }

    let installed = match install_instructions(&project, &selection.tools, streams) {
        Ok(installed) => installed,
        Err(error) => {
            let _ = streams.failure(&error);
            return Outcome::OperationalFailure;
        }
    };

    report_next_command(&selection.tools, &installed, streams);
    Outcome::Success
}

/// Prints the banner, when a terminal is there to print it to.
///
/// Hand-rolled ANSI with no dependency, because the whole requirement is a few escape
/// codes and a dependency for that would be surface with no payoff.
///
/// Suppressed when standard output is not a terminal, when `NO_COLOR` is set, and when the
/// terminal is too narrow to hold it. Each is a case where the banner is noise rather than
/// welcome: a pipe gets decoration in its data, a contributor who asked for no color asked
/// for this too, and a narrow terminal wraps it into nonsense.
fn report_banner<O: Write, E: Write>(streams: &mut Streams<O, E>) {
    use std::io::IsTerminal;

    if !std::io::stdout().is_terminal() || streams.form().is_structured() {
        return;
    }
    // Presence is what counts, not the value: an empty `NO_COLOR` still means no color.
    if std::env::var_os("NO_COLOR").is_some() {
        return;
    }
    // 44 columns holds the widest line with room to spare. Below that it wraps.
    if terminal_width().is_some_and(|width| width < 44) {
        return;
    }

    const DIM: &str = "\x1b[2m";
    const BOLD: &str = "\x1b[1m";
    const RESET: &str = "\x1b[0m";
    let _ = streams.report(&format!(
        "{BOLD}htmlspec{RESET} {DIM}{VERSION}{RESET}\n\
         {DIM}specifications you read in a browser{RESET}\n\n"
    ));
}

/// The terminal's width in columns, when it can be determined.
///
/// Read from `COLUMNS` rather than through an `ioctl`, which would need `unsafe` — and the
/// workspace forbids it. An absent value means "unknown", and the banner prints: a
/// contributor with an unusual terminal setup should see the tool work, not have a cosmetic
/// check silently withhold output.
fn terminal_width() -> Option<usize> {
    std::env::var("COLUMNS").ok()?.trim().parse().ok()
}

/// Names the literal command to run next, spelled for the tools that were selected.
///
/// **One line per distinct spelling**, labelled with the tools it applies to. The
/// spelling differs per tool — `/htmlspec:new` for one, `$htmlspec-new` for another — and
/// a single canonical form would be wrong for most of them. Printing a command no selected
/// tool answers to teaches a command that does not work, which is why this runs after the
/// selection is known rather than as part of a banner.
///
/// The workflow named is the one the table flags as the lifecycle's entry point, not the
/// one that ends it. A bootstrap is a first run, and naming archival there describes work
/// the contributor has not started. Reading the flag rather than naming a workflow here
/// keeps the message following the table when the table changes.
fn report_next_command<O: Write, E: Write>(
    tools: &[ToolId],
    installed: &[(ToolId, crate::install::Installed)],
    streams: &mut Streams<O, E>,
) {
    let _ = streams.report("\nthe project is ready.\n");
    let _ = streams.report(
        "  Create your first change:  htmlspec new change <name>\n\
         \x20 Then open its proposal in a browser and tell your agent what to change.\n",
    );

    // Only tools that actually received something can be told what to type. A tool that
    // was selected and whose write failed is reported elsewhere; it is not given an
    // invocation here.
    let served: Vec<ToolId> = installed
        .iter()
        .filter(|(_, written)| !written.is_empty())
        .map(|(id, _)| *id)
        .collect();
    if served.is_empty() {
        // No tools, so no command to name. Reporting one anyway would name a command
        // nothing answers to, which is exactly the failure this function exists to avoid.
        if !tools.is_empty() {
            let _ = streams.report(
                "  No workflow instructions were installed, so each stage is a manual step \
                 for now.\n",
            );
        }
        return;
    }

    // The table decides which workflow begins the lifecycle. Absent one, there is no
    // workflow to name, and naming an arbitrary row would teach a command that is not the
    // one to start with.
    let Some(entry) = htmlspec_assets::instructions::entry_point() else {
        return;
    };

    // Grouped by spelling, so tools that agree share a line.
    let mut spellings: Vec<(String, Vec<&'static str>)> = Vec::new();
    for id in &served {
        let tool = id.tool();
        let spelling = tool.invocation(entry.name);
        match spellings.iter_mut().find(|(known, _)| *known == spelling) {
            Some((_, names)) => names.push(tool.name),
            None => spellings.push((spelling, vec![tool.name])),
        }
    }

    let _ = streams.report("\nStart a change with your agent:\n");
    for (spelling, names) in &spellings {
        if spellings.len() == 1 {
            let _ = streams.report(&format!("  {spelling}\n"));
        } else {
            let _ = streams.report(&format!("  {spelling}  ({})\n", names.join(", ")));
        }
    }

    // Archival follows, and it is a sentence rather than the headline: a contributor who
    // has just bootstrapped has nothing to archive yet.
    let _ = streams.report(&format!(
        "\n{} workflow(s) were installed, covering the lifecycle from exploring an idea \
         to archiving the result.\n",
        WORKFLOWS.len()
    ));
}

/// Installs instructions for each selected tool, reporting what each received.
fn install_instructions<O: Write, E: Write>(
    project: &ProjectRoot,
    tools: &[ToolId],
    streams: &mut Streams<O, E>,
) -> Result<Vec<(ToolId, crate::install::Installed)>, String> {
    let mut installed = Vec::new();
    for id in tools {
        let tool = id.tool();
        let written = crate::install::install(project.path(), tool, VERSION, streams)?;
        let _ = streams.report(&format!(
            "{}: {} file(s) in {}\n",
            tool.name,
            written.len(),
            tool.directory
        ));
        installed.push((*id, written));
    }
    Ok(installed)
}

/// Reports stored assets that differ from the pinned runtime.
///
/// Advisory rather than fatal: the project still works, and the contributor may have
/// edited a stylesheet deliberately. What must not happen is `init` overwriting the edit
/// without saying so.
fn report_store_drift<O: Write, E: Write>(project: &ProjectRoot, streams: &mut Streams<O, E>) {
    let store = project.assets();
    for asset in htmlspec_assets::all_assets() {
        let path = store.join(asset.path);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        if htmlspec_assets::sha256_hex(&bytes) != asset.sha256() {
            let _ = streams.progress(&format!(
                "note: {} differs from the pinned runtime, and was left as it is",
                path.display()
            ));
        }
    }
}

/// Establishes a project from command-line arguments.
///
/// The thin half: it turns flags into a [`Selection`] and hands off. Tool selection is
/// the flag when one was given, and the detected tools otherwise — the prompt that would
/// ask arrives in its own module, and until then a detected default is the answer a
/// contributor would most likely give.
pub fn init<O: Write, E: Write>(
    root: Option<&Path>,
    name: Option<&str>,
    tools: Option<&str>,
    withdrawn: Option<&Path>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    // The withdrawn spelling. `init <path>` created a document until this release, so a
    // bare usage error would leave a reader who used it yesterday guessing. clap cannot
    // say this for us: the argument parses fine and the problem is what it used to mean.
    if let Some(path) = withdrawn {
        let kind = htmlspec_assets::DEFAULT_KIND;
        let _ = streams.failure(&format!(
            "init no longer takes a path: its subject is the project, not a document. To \
             create one document, run `htmlspec new document {kind} {}`. To create a \
             change, run `htmlspec new change <name>`. To set up this project, run \
             `htmlspec init` with no arguments",
            path.display()
        ));
        return Outcome::OperationalFailure;
    }

    let root = root.unwrap_or_else(|| Path::new("."));

    // Parsed before anything is written, so a misspelled tool does not leave half a
    // project behind.
    let tools = match tools {
        Some(value) => match crate::tools::parse_tools(value) {
            Ok(tools) => tools,
            Err(error) => {
                let _ = streams.failure(&error.to_string());
                return Outcome::OperationalFailure;
            }
        },
        // No flag, so the question has not been answered. Ask when a terminal is there to
        // ask on, and fall back to what was detected when it is not — an `init` that blocked
        // for a keypress inside an automated job would never finish.
        None if crate::prompt::can_prompt(false, streams.form().is_structured()) => {
            match crate::prompt::select_tools(root, name) {
                Ok(selection) => return bootstrap(root, &selection, streams),
                Err(crate::prompt::Cancelled) => {
                    let _ = streams.progress(
                        "cancelled; nothing was written. Run `htmlspec init --tools none` to \
                         establish the project without installing instructions",
                    );
                    return Outcome::Cancelled;
                }
            }
        }
        None => {
            let detected = crate::tools::detect(root);
            if detected.is_empty() {
                let _ = streams.progress(
                    "no AI coding tool directories were detected, so no instructions were \
                     installed. Pass --tools to name them",
                );
            } else {
                let names: Vec<&str> = detected.iter().map(|id| id.tool().name).collect();
                let _ = streams.progress(&format!("detected: {}", names.join(", ")));
            }
            detected
        }
    };

    bootstrap(
        root,
        &Selection {
            tools,
            name: name.map(str::to_owned),
        },
        streams,
    )
}

/// Writes the project's declaration, unless it already has one.
///
/// Never overwrites. A contributor who edited the version or the pinned toolchain meant
/// it, and `init` is safe to re-run precisely because re-running cannot undo that.
///
/// The starting version is `0.0.0`, which is what makes the archival version comparison
/// total: a project always has a version on record, so there is no absent-previous-value
/// case for the gate to special-case — a branch that would run once per project and be
/// tested once.
fn write_declaration<O: Write, E: Write>(
    project: &ProjectRoot,
    name: &str,
    streams: &mut Streams<O, E>,
) -> Result<(), String> {
    let path = project.declaration();
    if path.exists() {
        let _ = streams.progress(&format!(
            "{} already exists, so it was left as it is",
            path.display()
        ));
        return Ok(());
    }

    std::fs::write(&path, render_declaration(name))
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    let _ = streams.report(&format!(
        "{}: declared the project at version {INITIAL_VERSION}\n",
        path.display()
    ));
    Ok(())
}

/// The version gate: `Some(outcome)` when archival must not proceed.
///
/// Compares the declared version against the last archived one and refuses unless the
/// declared version is **ordered after** it. An ordering rather than a test for difference,
/// because a difference test accepts a downgrade — archiving at `0.1.0` after `0.2.0`
/// differs, so it would pass.
///
/// Three refusals, reported distinctly because they have different causes: the version has
/// not moved (work not yet done), the version went backwards (a mistake), and the version
/// cannot be ordered (a gate that cannot compare its inputs is not a gate).
fn check_version<O: Write, E: Write>(streams: &mut Streams<O, E>) -> Option<Outcome> {
    let project = match ProjectRoot::resolve(Path::new(".")) {
        Ok(project) => project,
        Err(error) => {
            let _ = streams.failure(&format!(
                "{error}. Archival compares the project's declared version against the last \
                 archived one, so it needs the project"
            ));
            return Some(Outcome::OperationalFailure);
        }
    };

    let declaration = match crate::declaration::read(&project.declaration()) {
        Ok(declaration) => declaration,
        Err(error) => {
            let _ = streams.failure(&error.to_string());
            return Some(Outcome::OperationalFailure);
        }
    };

    let previous = last_archived_version(&project);
    if declaration.version > previous {
        let _ = streams.progress(&format!(
            "version {} follows {previous}",
            declaration.version
        ));
        return None;
    }

    if declaration.version == previous {
        let _ = streams.failure(&format!(
            "the declared version is still {previous}, which is the last archived version. \
             Raise `version` in {} before archiving, so the accepted contract carries a \
             history rather than a sequence of undated merges. Advancing it is your work — \
             this command compares and never edits",
            crate::project::DECLARATION
        ));
    } else {
        let _ = streams.failure(&format!(
            "the declared version {} is ordered *before* the last archived version \
             {previous}, so archiving it would move the project backwards. This is a \
             different problem from a version that has not moved: check whether {} was \
             edited by mistake",
            declaration.version,
            crate::project::DECLARATION
        ));
    }
    Some(Outcome::OperationalFailure)
}

/// The version of the most recently archived change.
///
/// **Total.** With nothing archived the answer is [`INITIAL_VERSION`], not an absent value,
/// which is the whole reason `init` writes `0.0.0`: a project always has a version on
/// record, so the first archival is the same comparison as the thousandth. An
/// absent-previous-version branch would run once per project and therefore be tested once.
///
/// An archived change records the version it shipped in inside a [`RECORDED_VERSION`] file,
/// which is the form `toolchain-packaging`'s release-hygiene requirement asks for and the form
/// `scripts/check-release` already reads. A change recording no version is skipped rather than
/// failing the run — an archive predating the convention should not block archival — and the
/// greatest version found wins, so the answer does not depend on directory order.
///
/// **Not the directory name.** Until `finish-the-archival` this scanned the archive directory's
/// name, expecting `2026-08-14-0.2.0-add-dark-mode`. Nothing ever wrote such a name, because
/// archival did not create archive directories at all — it checked and left the change where it
/// was. So the scan found no version in any project that has ever existed, `max()` ran over an
/// empty iterator, and the gate compared every declaration against `0.0.0`. It never refused
/// anything. A gate derived from a convention nothing writes is not a gate, which is why this
/// reads the file the project actually records.
fn last_archived_version(project: &ProjectRoot) -> crate::version::Version {
    let initial =
        crate::version::Version::parse(INITIAL_VERSION).unwrap_or_else(|_| unreachable!());

    let Ok(entries) = std::fs::read_dir(project.archive()) else {
        return initial;
    };
    entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| recorded_version(&entry.path()))
        .max()
        .unwrap_or(initial)
}

/// The version an archived change records, if it records one.
///
/// An absent file, an unreadable one, and contents that do not parse are all `None`. This feeds
/// a `max`, so a change recording no version must not lower the answer, and none of the three
/// cases can be distinguished from the others in a way that would change what the gate should
/// compare against.
///
/// Reporting a malformed `VERSION` belongs to `scripts/check-release`, which names the change
/// that carries it. That is a report a contributor can act on. Refusing here would block
/// archival over a file that has nothing to do with the change being archived.
fn recorded_version(directory: &Path) -> Option<crate::version::Version> {
    let text = std::fs::read_to_string(directory.join(crate::project::RECORDED_VERSION)).ok()?;
    crate::version::Version::parse(text.trim()).ok()
}

/// The text of a project's declaration.
///
/// Separate from writing it so the reader in [`crate::declaration`] can round-trip against
/// exactly what `init` produces. Two halves of one file format drift apart unless something
/// checks them against each other, and `the_declaration_init_writes_is_readable` is that
/// check.
pub fn render_declaration(name: &str) -> String {
    format!(
        "# This project's identity and the toolchain it expects.\n\
         #\n\
         # `htmlspec archive` reads `version` and refuses unless it is ordered after the\n\
         # last archived version. Advancing it is your work, or your agent's; the program\n\
         # only compares. Nix rather than another format because `htmlspec` is a flake\n\
         # input with a hash, which is a pin the build already knows how to resolve.\n\
         {{\n\
         \x20 name = \"{name}\";\n\
         \x20 version = \"{INITIAL_VERSION}\";\n\
         \x20 # The toolchain this project expects. Replace `null` with a pinned flake\n\
         \x20 # reference to hold every machine to one binary.\n\
         \x20 htmlspec = null;\n\
         }}\n"
    )
}

/// The version a project starts at.
///
/// Ordered before every version a project can later declare, which is the property that
/// makes the archival gate total. See [`write_declaration`].
pub const INITIAL_VERSION: &str = "0.0.0";

/// Writes every shared asset that is not already present.
///
/// Existing files are left alone rather than overwritten: a project may have edited
/// its stylesheet, and `init` creating a document is not permission to revert that.
fn establish_assets<O: Write, E: Write>(
    directory: &Path,
    streams: &mut Streams<O, E>,
) -> Result<(), String> {
    let mut written = 0usize;
    for asset in htmlspec_assets::all_assets() {
        let path = directory.join(asset.path);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, asset.bytes)
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        written += 1;
    }
    if written > 0 {
        let _ = streams.progress(&format!(
            "established {written} shared asset(s) in {}",
            directory.display()
        ));
    }
    Ok(())
}

/// A reference from `from` to `to`, as a relative path using forward slashes.
///
/// Returns `None` when the two share no ancestor this function can walk between,
/// which the caller reports rather than falling back to an absolute path.
fn relative_reference(from: &Path, to: &Path) -> Option<String> {
    let from = normalize(from);
    let to = normalize(to);

    if let Ok(suffix) = to.strip_prefix(&from) {
        return Some(join(suffix.components()));
    }

    // Walk up from `from` until `to` is beneath it, counting the steps.
    let mut ascent = 0;
    let mut base = from.as_path();
    while let Some(parent) = base.parent() {
        ascent += 1;
        if let Ok(suffix) = to.strip_prefix(parent) {
            let mut reference = String::new();
            for _ in 0..ascent {
                reference.push_str("../");
            }
            reference.push_str(&join(suffix.components()));
            return Some(reference);
        }
        base = parent;
    }
    None
}

/// Absolutizes a path without touching the filesystem, and resolves its `.` and `..`
/// components lexically.
///
/// `..` is resolved rather than kept, so that `htmlspec/../docs` is not taken for a directory
/// beneath `htmlspec/`: [`project_store_root`] decides by a prefix comparison, and a kept `..`
/// would leave the prefix in place while the path leaves the directory. A `..` at the root stays
/// at the root, as the filesystem has it.
fn normalize(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    normalized
}

/// Joins components with forward slashes, which is what an HTML reference uses on
/// every platform.
fn join(components: std::path::Components<'_>) -> String {
    components
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<String>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::{decision, Decision, Findings};

    fn with_findings() -> Findings {
        Findings {
            change: String::from("a-change"),
            open_tasks: vec![String::from("task-one")],
            // Spread rather than listed, so a finding added later does not have to be
            // restated in every constructed value that does not care about it.
            ..Findings::default()
        }
    }

    /// Findings whose change states a requirement no accepted specification holds.
    fn unsynced_findings() -> Findings {
        Findings {
            change: String::from("a-change"),
            merge: super::MergeState {
                absent: vec![String::from("req-gone")],
                ..super::MergeState::default()
            },
            ..Findings::default()
        }
    }

    /// Findings whose only unapplied work is a rename.
    fn renamed_findings() -> Findings {
        Findings {
            change: String::from("a-change"),
            merge: super::MergeState {
                untaken_renames: vec![super::UntakenRename {
                    id: String::from("req-renamed"),
                    heading: htmlspec_parse::model::HeadingName::of("The new name", None),
                    accepted: Some(htmlspec_parse::model::HeadingName::of("The old name", None)),
                }],
                ..super::MergeState::default()
            },
            ..Findings::default()
        }
    }

    /// The toolchain-packaging scenario "A change whose only unapplied work is a rename", as the
    /// findings see it: every identifier is held, and the change has still not been synced.
    #[test]
    fn a_change_whose_only_unapplied_work_is_a_rename_is_unsynced() {
        let findings = renamed_findings();
        assert!(findings.merge.absent.is_empty());
        assert!(findings.needs_sync());
        assert!(!findings.is_empty(), "an untaken rename is a finding");
        assert_eq!(
            decision(&findings, false, None),
            Decision::Ask,
            "archival proceeded over an untaken rename without asking"
        );
        let question = question_for(&findings);
        assert!(question.contains("has not been synced"), "{question}");
        assert!(
            question.contains("htmlspec sync --change a-change"),
            "{question}"
        );
        assert!(question.contains("1 finding(s)"), "{question}");
    }

    /// An unmerged requirement is what "has not been synced" means.
    ///
    /// Archival already computed this: the finding and `sync`'s answer come from `merge_state`,
    /// so the two cannot disagree about whether a change needs syncing.
    #[test]
    fn an_unmerged_requirement_is_an_unsynced_change() {
        assert!(unsynced_findings().needs_sync());
        assert!(
            !with_findings().needs_sync(),
            "an open task is not an unsynced contract"
        );
        assert!(!Findings::default().needs_sync());
    }

    /// The question names `sync` when the change has not been synced.
    ///
    /// "Archive anyway?" invites a yes. A contributor who is about to rewrite the project's
    /// contract should be told which operation they are skipping, and told that declining costs
    /// them nothing.
    #[test]
    fn the_question_names_sync_for_an_unsynced_change() {
        let question = question_for(&unsynced_findings());
        assert!(question.contains("has not been synced"), "{question}");
        assert!(
            question.contains("htmlspec sync --change a-change"),
            "the question does not name the command that addresses it: {question}"
        );
        assert!(
            question.contains("Answering no leaves everything as it is"),
            "declining is not offered as the safe answer: {question}"
        );
    }

    /// A change that was synced is asked the plain question, with no mention of syncing.
    ///
    /// The warning has to be absent when it does not apply, or it becomes noise a reader learns
    /// to skip — and the one time it mattered would be skipped with it.
    #[test]
    fn the_question_does_not_mention_sync_when_the_contract_reached_the_specifications() {
        let question = question_for(&with_findings());
        assert!(!question.contains("sync"), "{question}");
        assert_eq!(question, "Archive anyway, over 1 finding(s)?");
    }

    /// Both kinds of finding are counted in the question, whichever wording it takes.
    #[test]
    fn the_question_counts_every_finding() {
        let both = Findings {
            change: String::from("a-change"),
            open_tasks: vec![String::from("task-one"), String::from("task-two")],
            merge: super::MergeState {
                absent: vec![String::from("req-gone")],
                ..super::MergeState::default()
            },
            ..Findings::default()
        };
        let question = question_for(&both);
        assert!(question.contains("3 finding(s)"), "{question}");
        assert!(question.contains("has not been synced"), "{question}");
    }

    /// Nothing found means nothing to ask about.
    #[test]
    fn no_findings_proceeds_without_asking() {
        assert_eq!(
            decision(&Findings::default(), false, None),
            Decision::Proceed
        );
    }

    /// `--yes` answers in advance, which is how an agent drives the operation.
    #[test]
    fn assuming_yes_proceeds_over_findings() {
        assert_eq!(decision(&with_findings(), true, None), Decision::Proceed);
    }

    /// Findings and no answer yet: ask.
    #[test]
    fn findings_with_no_answer_yet_ask() {
        assert_eq!(decision(&with_findings(), false, None), Decision::Ask);
    }

    /// A contributor who says yes gets the archival they asked for.
    #[test]
    fn an_accepted_offer_proceeds() {
        assert_eq!(
            decision(&with_findings(), false, Some(true)),
            Decision::Proceed
        );
    }

    /// A contributor who says no stops the operation. This is the path that produces
    /// `Cancelled`, and it is the one an interactive test could not reach.
    #[test]
    fn a_declined_offer_stops() {
        assert_eq!(
            decision(&with_findings(), false, Some(false)),
            Decision::Stop
        );
    }

    /// The decision is a function of its inputs and remembers nothing.
    ///
    /// This is what makes the no-one-to-ask case work. `ask` returns `None` where nothing can
    /// answer, so the second call sees exactly what the first saw and returns `Ask` again. The
    /// caller treats an `Ask` it has already tried to answer as a stop, and reports it
    /// differently from a declined offer because the causes differ: one is a person saying no,
    /// the other is nobody being there. Neither archives.
    #[test]
    fn the_decision_does_not_remember_being_called() {
        assert_eq!(decision(&with_findings(), false, None), Decision::Ask);
        assert_eq!(decision(&with_findings(), false, None), Decision::Ask);
    }

    // -------------------------------------------------------------------------
    // Archiving before the change this one follows
    // -------------------------------------------------------------------------

    /// Findings for a change whose stated predecessor is in `resolution`.
    fn following_findings(resolution: crate::sequence::Resolution) -> Findings {
        Findings {
            change: String::from("the-successor"),
            unresolved_predecessor: Some(crate::sequence::Follows {
                predecessor: String::from("the-contract"),
                resolution,
            }),
            ..Findings::default()
        }
    }

    /// An unresolved predecessor is a finding, and an archived one is not.
    ///
    /// Task 4.6's first half at the predicate. The absence matters as much as the presence: a
    /// report that fired on the in-order case would fire on every correct archival.
    #[test]
    fn only_an_unresolved_predecessor_is_out_of_order() {
        assert!(following_findings(crate::sequence::Resolution::Active).out_of_order());
        assert!(following_findings(crate::sequence::Resolution::Unmatched).out_of_order());
        // A change stating no predecessor, and a change whose predecessor is done, are both the
        // absence of this finding — `findings_for` filters a resolved one out before it arrives.
        assert!(!Findings::default().out_of_order());
        assert!(
            !with_findings().out_of_order(),
            "an open task is not an ordering violation"
        );
    }

    /// Task 4.4: the question names the predecessor.
    ///
    /// A pure function of the findings, so the wording is checked with no terminal — the same
    /// reason `decision` is separate from the prompt that feeds it.
    #[test]
    fn the_question_names_the_predecessor() {
        let question = question_for(&following_findings(crate::sequence::Resolution::Active));
        assert!(
            question.contains("follows the-contract"),
            "the question does not name the predecessor: {question}"
        );
        assert!(
            question.contains("which is not archived"),
            "the question does not say what is wrong: {question}"
        );
        assert!(
            question.contains("archive the-contract first"),
            "the question does not name the work that resolves it: {question}"
        );
        assert!(
            question.contains("Answering no leaves everything as it is"),
            "declining is not offered as the safe answer: {question}"
        );
    }

    /// The predecessor is named ahead of the sync when both are found.
    ///
    /// An unsynced contract can be applied a moment later. A sequence archived out of order is a
    /// sequence whose argument is already lost, so it is the sentence the contributor reads.
    #[test]
    fn the_predecessor_is_named_ahead_of_the_sync() {
        let both = Findings {
            change: String::from("the-successor"),
            open_tasks: vec![String::from("task-one")],
            merge: super::MergeState {
                absent: vec![String::from("req-gone")],
                ..super::MergeState::default()
            },
            unresolved_predecessor: Some(crate::sequence::Follows {
                predecessor: String::from("the-contract"),
                resolution: crate::sequence::Resolution::Active,
            }),
            cycle: Vec::new(),
        };
        let question = question_for(&both);
        assert!(question.contains("follows the-contract"), "{question}");
        // Every finding is counted whichever wording the question takes: two here plus the
        // predecessor.
        assert!(question.contains("3 finding(s)"), "{question}");
    }

    /// A cycle is counted as a finding and asked about.
    #[test]
    fn a_cycle_is_a_finding() {
        let cyclic = Findings {
            change: String::from("first"),
            cycle: vec![String::from("first"), String::from("second")],
            ..Findings::default()
        };
        assert!(!cyclic.is_empty());
        assert!(
            !cyclic.out_of_order(),
            "a cycle is a contradiction rather than an ordering violation"
        );
        assert!(question_for(&cyclic).contains("1 finding(s)"));
    }

    /// Task 4.7: archival proceeds on an affirmative answer and on `--yes`.
    ///
    /// An abandoned sequence must remain archivable. Its first change is dropped and its second
    /// is completed on its own merits, and a gate would leave a contributor editing a declaration
    /// to state a history that did not happen.
    #[test]
    fn an_ordering_finding_is_asked_about_and_never_refused() {
        let findings = following_findings(crate::sequence::Resolution::Active);

        assert_eq!(decision(&findings, false, None), Decision::Ask);
        assert_eq!(
            decision(&findings, false, Some(true)),
            Decision::Proceed,
            "an affirmative answer must archive: a contributor who knows the sequence changed \
             is the authority on it"
        );
        assert_eq!(
            decision(&findings, true, None),
            Decision::Proceed,
            "--yes must archive, so an agent can drive an abandoned sequence to completion"
        );
        // And declining stops, leaving the project as it was.
        assert_eq!(decision(&findings, false, Some(false)), Decision::Stop);
    }

    /// The two unresolved outcomes get different reports.
    ///
    /// A predecessor that was renamed and one that is merely unfinished are different
    /// situations, and a report that conflated them would send a contributor looking for work
    /// already done under another name.
    #[test]
    fn an_unmatched_predecessor_is_reported_as_a_correction() {
        let (_, _, waiting) = run(Form::Human, |streams| {
            report_findings(
                &following_findings(crate::sequence::Resolution::Active),
                streams,
            );
            Outcome::Success
        });
        let (_, _, misnamed) = run(Form::Human, |streams| {
            report_findings(
                &following_findings(crate::sequence::Resolution::Unmatched),
                streams,
            );
            Outcome::Success
        });

        assert!(waiting.contains("which is not archived"), "{waiting}");
        assert!(
            waiting.contains("Archive that one first"),
            "the report does not name the work that resolves it: {waiting}"
        );

        assert!(
            misnamed.contains("no active or archived change carries that name"),
            "{misnamed}"
        );
        assert!(
            misnamed.contains("correct the declaration rather than waiting on it"),
            "an unmatched name is reported as something to wait for: {misnamed}"
        );
        assert_ne!(waiting, misnamed);
    }

    /// Task 4.6: nothing is reported when the predecessor is archived, or when none is stated.
    ///
    /// A report that fires either way is one a reader learns to skip.
    #[test]
    fn nothing_is_reported_about_ordering_when_there_is_nothing_to_report() {
        for findings in [Findings::default(), with_findings()] {
            let (_, _, err) = run(Form::Human, |streams| {
                report_findings(&findings, streams);
                Outcome::Success
            });
            assert!(
                !err.contains("follows"),
                "ordering was reported for findings that state none: {err}"
            );
        }
    }

    use super::*;
    use crate::output::Form;

    /// A temporary directory that removes itself.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("htmlspec-cmd-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            Self(path)
        }

        fn write(&self, relative: &str, contents: &str) -> PathBuf {
            let path = self.0.join(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("the parent exists");
            }
            std::fs::write(&path, contents).expect("the file is written");
            path
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A journal is refused, built or read, if any path in it is relative.
    ///
    /// Before this held, a change named relative to the working directory was journaled as given,
    /// and a recovery run from another directory resolved it somewhere else.
    #[test]
    fn a_journal_holds_only_absolute_paths() {
        let digest = crate::runbook_release::ContentDigest::of(b"x");
        let absolute = PathBuf::from("/work/project/htmlspec/changes/a");
        let relative = PathBuf::from("htmlspec/changes/a");
        assert!(Journal::new(absolute.clone(), absolute.clone(), vec![], vec![]).is_ok());
        for (change, target, writes) in [
            (relative.clone(), absolute.clone(), vec![]),
            (absolute.clone(), relative.clone(), vec![]),
            (
                absolute.clone(),
                absolute.clone(),
                vec![(relative.join("VERSION"), digest.clone())],
            ),
        ] {
            let refused =
                Journal::new(change, target, writes, vec![]).expect_err("a relative journal");
            assert!(refused.contains("not an absolute path"), "{refused}");
        }

        let text = serde_json::json!({
            "format": JOURNAL_FORMAT,
            "change": "htmlspec/changes/a",
            "target": "/work/project/htmlspec/changes/archive/a",
            "writes": [],
            "replacements": [],
        })
        .to_string();
        let refused = Journal::parse(&text).expect_err("a relative journal was read");
        assert!(refused.contains("not an absolute path"), "{refused}");
    }

    /// A journal of the wrong shape is refused with a reason, not read as absent.
    #[test]
    fn a_malformed_journal_names_what_is_wrong() {
        for (text, wanted) in [
            ("{not json", "not JSON"),
            ("{}", "no `format` string"),
            (
                r#"{"format":"something-else/v1","change":"/a","target":"/b","writes":[]}"#,
                "rather than",
            ),
            (
                r#"{"format":"htmlspec-archive-journal/v1","change":"/a","target":"/b"}"#,
                "no `writes` list",
            ),
            (
                r#"{"format":"htmlspec-archive-journal/v1","change":"/a","target":"/b","writes":[{"path":"/a/VERSION","digest":"md5:00"}]}"#,
                "no usable digest",
            ),
        ] {
            let problem = Journal::parse(text).expect_err("read as a journal");
            assert!(problem.contains(wanted), "{text:?} reported {problem:?}");
        }
    }

    /// A v2 journal's replacements name documents in the change, by name, and nothing else.
    ///
    /// A replacement is applied inside whichever of the change and the archive holds the change, so
    /// a name that reached out of it would let a journal replace a file somewhere else.
    #[test]
    fn a_journal_replaces_only_documents_in_the_change() {
        let digest = format!("sha256:{}", "0".repeat(64));
        let journal = |replacements: serde_json::Value| {
            serde_json::json!({
                "format": "htmlspec-archive-journal/v2",
                "change": "/p/htmlspec/changes/a",
                "target": "/p/htmlspec/changes/archive/2026-01-01-a",
                "writes": [],
                "replacements": replacements,
            })
            .to_string()
        };

        let read = Journal::parse(&journal(
            serde_json::json!([{ "document": "proposal.html", "digest": digest }]),
        ))
        .expect("a v2 journal reads");
        assert_eq!(
            read.replacements
                .iter()
                .map(|replacement| replacement.document.staged_in(Path::new("/p/c")))
                .collect::<Vec<_>>(),
            vec![PathBuf::from("/p/c/.proposal.html.archiving")]
        );

        for document in [
            "../proposal.html",
            "evidence/a.html",
            "",
            ".",
            "..",
            "/p/a.html",
        ] {
            let problem = Journal::parse(&journal(
                serde_json::json!([{ "document": document, "digest": digest }]),
            ))
            .expect_err("a replacement outside the change was read");
            assert!(
                problem.contains("not a document in the change"),
                "{document:?} reported {problem:?}"
            );
        }
        let problem =
            Journal::parse(&journal(serde_json::Value::Null).replace(",\"replacements\":null", ""))
                .expect_err("a v2 journal without replacements was read");
        assert!(problem.contains("no `replacements` list"), "{problem}");

        // A v1 journal, written by an earlier program, replaces nothing.
        let earlier = serde_json::json!({
            "format": "htmlspec-archive-journal/v1",
            "change": "/p/htmlspec/changes/a",
            "target": "/p/htmlspec/changes/archive/2026-01-01-a",
            "writes": [],
        })
        .to_string();
        assert!(Journal::parse(&earlier)
            .expect("a v1 journal reads")
            .replacements
            .is_empty());
    }

    /// Publishing the marker stages through a unique file and leaves a neighbour's file alone.
    ///
    /// The staging name used to be fixed — `runbook-releases.publishing` — so a file already at that
    /// name was overwritten and then deleted, and two writers staged through one path.
    #[test]
    fn publishing_without_replacement_stages_through_a_file_it_owns() {
        let scratch = Scratch::new("publish-marker");
        let destination = scratch.0.join("runbook-releases.json");
        let bystander = scratch.write("runbook-releases.publishing", "somebody's\n");

        publish_without_replacement(&destination, "first\n").expect("published");
        assert_eq!(
            std::fs::read_to_string(&destination).expect("readable"),
            "first\n"
        );
        assert_eq!(
            std::fs::read_to_string(&bystander).expect("the bystander survives"),
            "somebody's\n",
            "publication wrote through a name it did not create"
        );

        let refused = publish_without_replacement(&destination, "second\n")
            .expect_err("an existing marker was replaced");
        assert!(refused.contains("already exists"), "{refused}");
        assert_eq!(
            std::fs::read_to_string(&destination).expect("readable"),
            "first\n"
        );

        // Neither the success nor the refusal leaves a staging file behind.
        let mut left: Vec<String> = std::fs::read_dir(&scratch.0)
            .expect("listed")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(
            left,
            ["runbook-releases.json", "runbook-releases.publishing"],
            "a staging file outlived the publication"
        );
    }

    /// Runs a command over capturing streams, returning `(outcome, stdout, stderr)`.
    fn run(
        form: Form,
        act: impl FnOnce(&mut Streams<Vec<u8>, Vec<u8>>) -> Outcome,
    ) -> (Outcome, String, String) {
        let mut streams = Streams::new(form, Vec::new(), Vec::new());
        let outcome = act(&mut streams);
        let (out, err) = streams.into_parts();
        (
            outcome,
            String::from_utf8(out).expect("stdout is UTF-8"),
            String::from_utf8(err).expect("stderr is UTF-8"),
        )
    }

    /// A document that validates, as `init` would write it.
    ///
    /// The `specification` shape rather than `init`'s default, because these tests
    /// need a document carrying references they can break. `init` defaults to a
    /// proposal, which is prose with no unit scaffolding at all — nothing in it to
    /// make dangle.
    fn valid_document() -> String {
        htmlspec_assets::render_kind("specification", "A subject", "assets")
            .expect("the specification template exists")
    }

    /// A project on disk whose accepted specifications hold exactly `accepted`, and one
    /// change whose specification states exactly `stated`.
    ///
    /// The identifiers are rewritten from the template's `req-example`, so both sides are
    /// real documents that validate rather than hand-built markup that might not.
    fn project_stating(
        scratch: &Scratch,
        stated: &[&str],
        accepted: &[&str],
    ) -> (ProjectRoot, PathBuf) {
        let root = scratch.0.join("project");
        let project = ProjectRoot::establish(&root).expect("the project is established");
        for (index, id) in accepted.iter().enumerate() {
            let document = valid_document().replace("req-example", id);
            let path = project.specs().join(format!("accepted-{index}.html"));
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the parent exists");
            std::fs::write(&path, document).expect("the accepted specification is written");
        }
        let directory = project.change("a-change");
        std::fs::create_dir_all(&directory).expect("the change directory exists");
        // One document per stated identifier would be several specifications in one change,
        // which a change does not have. So the identifiers are substituted into the requirement
        // the template ships, and additional ones are appended as copies of that section.
        let mut document = valid_document();
        if let Some((first, rest)) = stated.split_first() {
            document = document.replace("req-example", first);
            for id in rest {
                let section = format!(
                    "<section class=\"requirement\" id=\"{id}\" data-change=\"added\">\n            \
                     <h3>Another requirement <span class=\"hs-id\">{id}</span></h3>\n            \
                     <p>The system SHALL state one obligation per requirement, in a sentence a \
                     reader can verify.</p>\n          </section>\n          "
                );
                let marker = format!("<section class=\"requirement\" id=\"{first}\"");
                document = document.replacen(&marker, &format!("{section}{marker}"), 1);
            }
        }
        std::fs::write(directory.join(CHANGE_SPECIFICATION), document)
            .expect("the change specification is written");
        (project, directory)
    }

    /// The note for an untaken rename names the identifier, both headings, and the operation.
    #[test]
    fn the_reported_rename_names_both_headings_and_the_sync_command() {
        let (_, _, reported) = run(Form::Human, |streams| {
            report_findings(&renamed_findings(), streams);
            Outcome::Success
        });

        for expected in [
            "req-renamed",
            "\"The old name\"",
            "\"The new name\"",
            "has not been synced",
            "htmlspec sync --change a-change",
        ] {
            assert!(
                reported.contains(expected),
                "the note does not name {expected}: {reported}"
            );
        }
    }

    /// The structured payload carries the untaken renames, and says the change needs syncing.
    #[test]
    fn the_payload_carries_each_untaken_rename() {
        let (_, out, _) = run(Form::Structured, |streams| {
            emit_archive_payload(streams, &renamed_findings(), false, None);
            Outcome::Success
        });
        let payload: serde_json::Value = serde_json::from_str(&out).expect("it parses");
        assert_eq!(payload["needsSync"], true, "{payload}");
        assert_eq!(
            payload["untakenRenames"],
            serde_json::json!([{
                "id": "req-renamed",
                "heading": "The new name",
                "acceptedHeading": "The old name",
            }]),
            "{payload}"
        );
        assert_eq!(payload["unmergedRequirements"], serde_json::json!([]));
    }

    /// Findings whose only unapplied work is a word the accepted requirement does not carry.
    fn differing_findings() -> Findings {
        Findings {
            change: String::from("a-change"),
            merge: MergeState {
                untaken_content: vec![untaken("req-x", 1, &[], &[])],
                ..MergeState::default()
            },
            ..Findings::default()
        }
    }

    /// Findings whose only unapplied work is a withdrawal the accepted specifications still hold.
    fn withdrawn_findings() -> Findings {
        Findings {
            change: String::from("a-change"),
            merge: MergeState {
                untaken_removals: vec![String::from("req-y")],
                ..MergeState::default()
            },
            ..Findings::default()
        }
    }

    /// The toolchain-packaging scenario "A change whose only unapplied work is a word", as the
    /// findings see it: every identifier is held, and the change has still not been synced.
    #[test]
    fn a_change_whose_only_unapplied_work_is_a_word_is_unsynced() {
        let findings = differing_findings();
        assert!(findings.merge.absent.is_empty() && findings.merge.untaken_renames.is_empty());
        assert!(findings.needs_sync(), "untaken content was called synced");
        assert!(!findings.is_empty(), "untaken content is a finding");
        assert_eq!(
            decision(&findings, false, None),
            Decision::Ask,
            "archival proceeded over untaken content without asking"
        );
        let question = question_for(&findings);
        assert!(question.contains("has not been synced"), "{question}");
        assert!(question.contains("1 finding(s)"), "{question}");
    }

    /// The toolchain-packaging scenario "A change whose only unapplied work is a withdrawal".
    #[test]
    fn a_change_whose_only_unapplied_work_is_a_withdrawal_is_unsynced() {
        let findings = withdrawn_findings();
        assert!(findings.merge.absent.is_empty() && findings.merge.untaken_renames.is_empty());
        assert!(
            findings.needs_sync(),
            "an untaken withdrawal was called synced"
        );
        assert!(!findings.is_empty(), "an untaken withdrawal is a finding");
        assert_eq!(decision(&findings, false, None), Decision::Ask);
        let question = question_for(&findings);
        assert!(question.contains("has not been synced"), "{question}");
        assert!(question.contains("1 finding(s)"), "{question}");
    }

    /// The toolchain-packaging scenario "The report says where the content differs, not what it
    /// says", over archival's payload built from a real comparison.
    ///
    /// The entry names a paragraph position and scenario identifiers. Every statement and every
    /// scenario's heading and entries, from both sides, is then looked for in the payload's text
    /// and must not be there: a text in a worklist reads as the text to write.
    #[test]
    fn the_payload_carries_where_content_differs_and_no_text() {
        let scratch = Scratch::new("content-payload");
        let accepted = accepted_x();
        let stated = Stated {
            change: "modified",
            statements: &["The system SHALL do x quickly."],
            scenarios: &[
                ("scenario-x-a", "Scenario A", "unchanged"),
                ("scenario-x-b", "Scenario B", "removed"),
                ("scenario-x-c", "Scenario C", "added"),
            ],
            ..accepted_x()
        };
        let (project, directory, projection) = project_holding(
            &scratch,
            &specification_stating(&[accepted]),
            &specification_stating(&[stated]),
        );
        let findings = findings_for(&project, &directory, &projection);

        let (_, out, _) = run(Form::Structured, |streams| {
            emit_archive_payload(streams, &findings, false, None);
            Outcome::Success
        });
        let payload: serde_json::Value = serde_json::from_str(&out).expect("it parses");
        assert_eq!(payload["needsSync"], true, "{payload}");
        assert_eq!(
            payload["untakenContent"],
            serde_json::json!([{
                "id": "req-x",
                "firstDifferingStatement": 1,
                "scenariosNotHeld": ["scenario-x-c"],
                "scenariosNotKept": ["scenario-x-b"],
            }]),
            "{payload}"
        );
        for text in [
            "The system SHALL do x",
            "quickly",
            "Scenario A",
            "Scenario B",
            "Scenario C",
            "happens.",
            "holds.",
        ] {
            assert!(
                !out.contains(text),
                "the payload carries {text:?}, a text from one side of the comparison: {out}"
            );
        }
    }

    /// Every set of work not done counts toward the question's number of findings.
    #[test]
    fn the_question_counts_every_untaken_set() {
        let every = Findings {
            change: String::from("a-change"),
            merge: MergeState {
                absent: vec![String::from("req-gone")],
                untaken_renames: renamed_findings().merge.untaken_renames,
                untaken_content: vec![untaken("req-x", 1, &[], &[])],
                untaken_removals: vec![String::from("req-y")],
                ..MergeState::default()
            },
            ..Findings::default()
        };
        let question = question_for(&every);
        assert!(question.contains("4 finding(s)"), "{question}");
        assert!(question.contains("has not been synced"), "{question}");
    }

    /// The reported note names the operation, because there now is one to name.
    ///
    /// Until 0.7.0 this note said "apply the change's contract" and no command did that. The
    /// repair added `sync`; a note that still does not name it leaves the reader searching the
    /// command surface for an operation whose name they have to guess.
    #[test]
    fn the_reported_finding_names_the_sync_command() {
        let (_, _, reported) = run(Form::Human, |streams| {
            report_findings(&unsynced_findings(), streams);
            Outcome::Success
        });

        assert!(reported.contains("has not been synced"), "{reported}");
        assert!(
            reported.contains("htmlspec sync --change a-change"),
            "the note does not name the command that addresses it: {reported}"
        );
    }

    /// The structured payload states whether the change needs syncing.
    ///
    /// An agent driving archival cannot answer a prompt, so the condition the prompt asks about
    /// has to be a field. Inferring it from an empty array is the kind of arithmetic a caller
    /// gets wrong once and then reports confidently.
    #[test]
    fn the_payload_states_whether_the_change_needs_syncing() {
        let (_, unsynced, _) = run(Form::Structured, |streams| {
            emit_archive_payload(streams, &unsynced_findings(), false, None);
            Outcome::Success
        });
        let payload: serde_json::Value = serde_json::from_str(&unsynced).expect("it parses");
        assert_eq!(payload["needsSync"], true, "{payload}");

        let (_, synced, _) = run(Form::Structured, |streams| {
            emit_archive_payload(streams, &with_findings(), false, None);
            Outcome::Success
        });
        let payload: serde_json::Value = serde_json::from_str(&synced).expect("it parses");
        assert_eq!(payload["needsSync"], false, "{payload}");
    }

    /// A tasks document stating one task per entry in `states`.
    ///
    /// Built from the template `new change` ships rather than from hand-written markup, so what
    /// the counter is tested against is what the tool actually produces. The template states two
    /// tasks — one `todo`, one `dropped` — and both are replaced by the requested set.
    ///
    /// A `None` state produces a task carrying no `data-state` at all, which is how the
    /// undeclared case is reachable: the schema requires a state, and the counter reads a
    /// projection rather than a validated document.
    fn tasks_document(states: &[Option<&str>]) -> String {
        let tasks: Vec<(Option<&str>, Account)> = states
            .iter()
            .map(|state| (*state, Account::AsWritten))
            .collect();
        tasks_document_of(&tasks)
    }

    /// Whether a task in a test document states a deviation.
    ///
    /// A sum type rather than a `bool` beside the state, so a call site says which account each
    /// task gives rather than passing a flag a reader has to look up.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Account {
        /// No deviation paragraph: the task's words are the account of it.
        AsWritten,
        /// One `data-field="deviation"` paragraph, whatever the task's state.
        Deviating,
    }

    /// A tasks document stating one task per entry, each in its state and with its account.
    ///
    /// [`tasks_document`] is this with every task [`Account::AsWritten`]. A deviation is written
    /// on whatever state the entry names, because the counter reads unvalidated projections and
    /// has to be tested against a deviation the validator would refuse.
    fn tasks_document_of(tasks: &[(Option<&str>, Account)]) -> String {
        let template = htmlspec_assets::render_kind("tasks", "A change", "assets")
            .expect("the tasks template exists");
        let opening = template
            .find("<li class=\"task\"")
            .expect("the template states a task");
        let closing = template.rfind("</li>").expect("the task list closes") + "</li>".len();
        let mut written = String::new();
        for (index, (state, account)) in tasks.iter().enumerate() {
            let attribute = match state {
                Some(state) => format!(" data-state=\"{state}\""),
                None => String::new(),
            };
            // `dropped` requires a reason, and every other state takes a note. Emitting the
            // wrong one would produce a document the schema rejects for a reason unrelated to
            // what is being tested.
            let paragraph = if *state == Some("dropped") {
                "<p data-field=\"reason\">It was superseded by another task.</p>"
            } else {
                "<p data-field=\"note\">The work this task states.</p>"
            };
            let deviation = match account {
                Account::AsWritten => "",
                Account::Deviating => {
                    "\n     <p data-field=\"deviation\">It was done another way.</p>"
                }
            };
            written.push_str(&format!(
                "<li class=\"task\" id=\"task-{index}\" data-change=\"added\"{attribute}>\n     \
                 <h3>Task {index} <span class=\"hs-id\">task-{index}</span></h3>\n     \
                 {paragraph}{deviation}\n            </li>\n            "
            ));
        }
        format!("{}{written}{}", &template[..opening], &template[closing..])
    }

    /// A change directory whose tasks document states `states`, inside an established project.
    fn change_with_tasks(scratch: &Scratch, states: &[Option<&str>]) -> (ProjectRoot, PathBuf) {
        let tasks: Vec<(Option<&str>, Account)> = states
            .iter()
            .map(|state| (*state, Account::AsWritten))
            .collect();
        change_with_accounts(scratch, &tasks)
    }

    /// A change directory whose tasks document states `tasks`, inside an established project.
    fn change_with_accounts(
        scratch: &Scratch,
        tasks: &[(Option<&str>, Account)],
    ) -> (ProjectRoot, PathBuf) {
        let root = scratch.0.join("project");
        let project = ProjectRoot::establish(&root).expect("the project is established");
        let directory = project.change("a-change");
        std::fs::create_dir_all(&directory).expect("the change directory exists");
        std::fs::write(directory.join("tasks.html"), tasks_document_of(tasks))
            .expect("the tasks document is written");
        (project, directory)
    }

    /// Every task state the contract defines, counted apart.
    ///
    /// The four states exist so the reports can tell apart situations a fraction collapses. If
    /// the counter folded any two together, the distinction would be gone before any caller saw
    /// it.
    #[test]
    fn each_state_is_counted_apart() {
        let scratch = Scratch::new("completion-states");
        let (_, directory) = change_with_tasks(
            &scratch,
            &[
                Some("todo"),
                Some("todo"),
                Some("in-progress"),
                Some("done"),
                Some("dropped"),
                None,
            ],
        );

        let counted = completion(&directory).expect("the tasks document is present");

        assert_eq!(counted.todo, 2, "{counted:?}");
        assert_eq!(counted.in_progress, 1, "{counted:?}");
        assert_eq!(counted.done, 1, "{counted:?}");
        assert_eq!(counted.dropped, 1, "{counted:?}");
        assert_eq!(counted.undeclared, 1, "{counted:?}");
        assert_eq!(
            counted.total(),
            6,
            "every task is in the total: {counted:?}"
        );
        // Open is everything not `done` and not `dropped`, undeclared included.
        assert_eq!(counted.open.len(), 4, "{counted:?}");
    }

    /// The enumeration and the completion operation report the same numbers.
    ///
    /// This is the accepted scenario `The enumeration and the completion report agree`, checked
    /// rather than asserted, and it is the counterpart of
    /// `archival_and_sync_name_the_same_absent_requirements` for the counts. Two counts of one
    /// thing drift, and a listing reporting `17/31` while `status` reports `18/31` is wrong in a
    /// way no test written against either alone would catch — the shape of the version gate that
    /// validated its own invented convention across eleven archives without refusing anything.
    #[test]
    fn the_enumeration_and_the_completion_report_agree() {
        let scratch = Scratch::new("completion-agree");
        let (project, directory) = change_with_accounts(
            &scratch,
            &[
                (Some("done"), Account::Deviating),
                (Some("done"), Account::AsWritten),
                (Some("in-progress"), Account::AsWritten),
                (Some("todo"), Account::AsWritten),
                (Some("dropped"), Account::AsWritten),
            ],
        );

        // What the enumeration carries, and what the completion operation derives.
        let carried = completion(&directory).expect("the tasks document is present");
        let from_status_open = open_tasks(&directory);

        assert_eq!(
            carried.open, from_status_open,
            "the enumeration and archival disagree about which tasks are open"
        );
        assert_eq!(
            carried.total().saturating_sub(carried.open.len()),
            carried.resolved(),
            "the number `status` renders is not the number the breakdown states: {carried:?}"
        );
        assert_eq!(carried.total(), 5, "{carried:?}");
        assert_eq!(
            carried.resolved(),
            3,
            "two done and one dropped: {carried:?}"
        );

        // The tasks done differently, as each payload reports them, against the counter.
        let (_, listed, _) = run(Form::Structured, |streams| {
            list_in(&project, false, false, streams)
        });
        let listed: serde_json::Value = serde_json::from_str(&listed).expect("the listing parses");
        let (_, reported, _) = run(Form::Structured, |streams| status_of(&directory, streams));
        let reported: serde_json::Value =
            serde_json::from_str(&reported).expect("the report parses");

        assert_eq!(carried.done_differently, vec!["task-0"], "{carried:?}");
        assert_eq!(
            listed["changes"][0]["doneDifferently"],
            carried.done_differently.len(),
            "the enumeration and the counter disagree about the tasks done differently: {listed}"
        );
        assert_eq!(
            reported["tasks"]["doneDifferently"], listed["changes"][0]["doneDifferently"],
            "the enumeration and the completion report disagree: {reported} against {listed}"
        );
        assert_eq!(
            reported["tasks"]["doneDifferentlyTasks"],
            serde_json::json!(carried.done_differently),
            "the completion report names other tasks than the counter holds: {reported}"
        );
    }

    /// A `done` task stating a deviation is `done`, and the counter also names it apart.
    ///
    /// The accepted scenario "A change with a task done differently": its work happened, so it is
    /// in the numerator and resolves, and the count of tasks done differently is beside the
    /// per-state counts rather than among them.
    #[test]
    fn a_done_task_stating_a_deviation_is_done_and_done_differently() {
        let scratch = Scratch::new("completion-deviation");
        let (_, directory) = change_with_accounts(
            &scratch,
            &[
                (Some("done"), Account::AsWritten),
                (Some("done"), Account::Deviating),
                (Some("dropped"), Account::AsWritten),
                (Some("todo"), Account::AsWritten),
            ],
        );

        let counted = completion(&directory).expect("present");

        assert_eq!(
            counted.done, 2,
            "a deviation leaves a task done: {counted:?}"
        );
        assert_eq!(counted.done_differently, vec!["task-1"], "{counted:?}");
        assert_eq!(counted.total(), 4, "{counted:?}");
        assert_eq!(counted.resolved(), 3, "{counted:?}");
        assert_eq!(counted.open, vec!["task-3"], "{counted:?}");
        assert_eq!(counted.state(), ChangeState::InProgress);
    }

    /// A deviation on a task in any other state is counted under that state only.
    ///
    /// Such a document is invalid, and the counter reads projections rather than validated
    /// documents, so the case is reachable. Counting it as done differently would attribute to the
    /// task a state it never declared, which is the reading the counter already refuses for an
    /// undeclared task.
    #[test]
    fn a_deviation_on_a_task_that_is_not_done_counts_under_its_own_state_only() {
        for state in [Some("todo"), Some("in-progress"), Some("dropped"), None] {
            let scratch = Scratch::new("completion-deviation-elsewhere");
            let (_, directory) = change_with_accounts(&scratch, &[(state, Account::Deviating)]);

            let counted = completion(&directory).expect("present");

            assert!(
                counted.done_differently.is_empty(),
                "{state:?}: a task that is not done was counted as done differently: {counted:?}"
            );
            assert_eq!(counted.done, 0, "{state:?}: {counted:?}");
            assert_eq!(counted.total(), 1, "{state:?}: {counted:?}");
            let own = match state {
                Some("todo") => counted.todo,
                Some("in-progress") => counted.in_progress,
                Some("dropped") => counted.dropped,
                _ => counted.undeclared,
            };
            assert_eq!(own, 1, "{state:?} is not counted under itself: {counted:?}");
        }
    }

    /// Archival's finding is a projection of the same count, not a second traversal.
    ///
    /// The accepted contract requires archival to derive its incomplete-work report from the
    /// completion operation. It holds by construction — one function, and `open_tasks` selects
    /// from it — and this is what would fail if someone gave either its own traversal again.
    #[test]
    fn archival_derives_its_open_tasks_from_the_one_counter() {
        let scratch = Scratch::new("completion-archival");
        let (project, directory) = change_with_tasks(&scratch, &[Some("todo"), Some("done"), None]);
        let source = std::fs::read_to_string(directory.join("tasks.html")).expect("it reads");
        let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);

        let from_counter = completion(&directory).expect("present").open;
        let from_archival = findings_for(&project, &directory, &projection).open_tasks;

        assert_eq!(
            from_counter, from_archival,
            "archival and the counter disagree about what is open"
        );
        // The undeclared task is open: a task nobody has said anything about is not finished.
        assert_eq!(from_archival.len(), 2, "{from_archival:?}");
    }

    /// A change with no tasks document has no completion, rather than a completion of zero.
    ///
    /// `0/0` reads as a change with no work outstanding, which is the opposite of what an
    /// unwritten tasks document means. `None` is what keeps the listing able to say so.
    #[test]
    fn a_change_with_no_tasks_document_has_no_completion() {
        let scratch = Scratch::new("completion-absent");
        let root = scratch.0.join("project");
        let project = ProjectRoot::establish(&root).expect("the project is established");
        let directory = project.change("a-change");
        std::fs::create_dir_all(&directory).expect("the change directory exists");

        assert_eq!(completion(&directory), None);
        // Archival's behavior is unchanged: no tasks document produces no finding.
        assert!(open_tasks(&directory).is_empty());
    }

    /// A project holding several changes, each with the tasks its entry states.
    ///
    /// Named to make the alignment case readable: the names differ in length on purpose.
    fn project_listing(
        scratch: &Scratch,
        changes: &[(&str, Option<&[Option<&str>]>)],
    ) -> ProjectRoot {
        let root = scratch.0.join("project");
        let project = ProjectRoot::establish(&root).expect("the project is established");
        for (name, states) in changes {
            let directory = project.change(name);
            std::fs::create_dir_all(&directory).expect("the change directory exists");
            if let Some(states) = states {
                std::fs::write(directory.join("tasks.html"), tasks_document(states))
                    .expect("the tasks document is written");
            }
        }
        project
    }

    /// Dropping a task moves it out of the numerator's reach and leaves the denominator alone.
    ///
    /// The denominator counts every task the change states, `dropped` included. Excluding them
    /// would let a total shrink as work was abandoned — `18/20` becoming `18/18` with nothing
    /// completed — and a denominator that moves is one nobody can compare two readings of.
    #[test]
    fn dropping_a_task_does_not_shrink_the_total() {
        let scratch = Scratch::new("completion-denominator");
        let before = completion(
            &change_with_tasks(
                &scratch,
                &[Some("done"), Some("done"), Some("todo"), Some("todo")],
            )
            .1,
        )
        .expect("present");

        let after_scratch = Scratch::new("completion-denominator-after");
        let after = completion(
            &change_with_tasks(
                &after_scratch,
                &[Some("done"), Some("done"), Some("dropped"), Some("dropped")],
            )
            .1,
        )
        .expect("present");

        assert_eq!(before.total(), after.total(), "the denominator moved");
        assert_eq!(before.done, after.done, "the numerator moved");
        // What changed is the meaning: the same 2/4 now has nothing outstanding.
        assert_eq!(before.state(), ChangeState::InProgress);
        assert_eq!(
            after.state(),
            ChangeState::Done,
            "a change whose remaining tasks were abandoned is finished: {after:?}"
        );
    }

    /// A change whose every task is `dropped` reads as finished, not as untouched.
    #[test]
    fn a_wholly_abandoned_change_is_not_untouched() {
        let scratch = Scratch::new("completion-abandoned");
        let (_, directory) = change_with_tasks(&scratch, &[Some("dropped"), Some("dropped")]);

        let counted = completion(&directory).expect("present");

        assert_eq!(counted.done, 0, "nothing was completed: {counted:?}");
        assert_eq!(counted.state(), ChangeState::Done, "{counted:?}");
        assert!(
            counted.open.is_empty(),
            "nothing is outstanding: {counted:?}"
        );
    }

    /// Every task `todo` is distinguishable from every task `in-progress`, at the same fraction.
    ///
    /// The accepted scenario `Two changes at the same fraction in different states`. Both render
    /// `0/2`, and the payload separates them.
    #[test]
    fn the_same_fraction_in_two_states_is_distinguishable() {
        let untouched = Scratch::new("completion-untouched");
        let started = Scratch::new("completion-started");
        let idle = completion(&change_with_tasks(&untouched, &[Some("todo"), Some("todo")]).1)
            .expect("present");
        let busy =
            completion(&change_with_tasks(&started, &[Some("in-progress"), Some("in-progress")]).1)
                .expect("present");

        assert_eq!(idle.done, busy.done, "both render the same fraction");
        assert_eq!(idle.total(), busy.total());
        assert_ne!(
            idle.state(),
            busy.state(),
            "the two are indistinguishable: {idle:?} against {busy:?}"
        );
        assert_eq!(idle.state(), ChangeState::Todo);
        assert_eq!(busy.state(), ChangeState::InProgress);
    }

    /// The payload omits the counts for a change with no tasks document rather than zeroing them.
    #[test]
    fn the_payload_omits_an_absent_completion() {
        let scratch = Scratch::new("list-absent");
        let root = project_listing(
            &scratch,
            &[
                ("has-tasks", Some(&[Some("done"), Some("todo")])),
                ("no-tasks", None),
            ],
        );
        let (outcome, out, _) = run(Form::Structured, |streams| {
            list_in(&root, false, false, streams)
        });

        assert_eq!(outcome, Outcome::Success);
        let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
        let changes = payload["changes"].as_array().expect("an array");
        let absent = changes
            .iter()
            .find(|entry| entry["name"] == "no-tasks")
            .expect("the change is enumerated");
        assert!(
            absent.get("total").is_none() && absent.get("tasks").is_none(),
            "the counts are present for a change with no tasks document: {absent}"
        );
        let present = changes
            .iter()
            .find(|entry| entry["name"] == "has-tasks")
            .expect("the change is enumerated");
        assert_eq!(present["total"], 2, "{present}");
        assert_eq!(present["tasks"]["done"], 1, "{present}");
    }

    /// The human report aligns its columns across names of differing length.
    #[test]
    fn the_report_aligns_across_names_of_differing_length() {
        let scratch = Scratch::new("list-aligned");
        let root = project_listing(
            &scratch,
            &[
                ("a-considerably-longer-change-name", Some(&[Some("done")])),
                ("short", Some(&[Some("todo")])),
            ],
        );
        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, false, streams));

        assert_eq!(outcome, Outcome::Success);
        // The fraction begins at the same column on every row, which is what makes two changes
        // comparable at a glance.
        let columns: Vec<usize> = out
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.find('/').expect("a fraction on every row"))
            .collect();
        assert_eq!(columns.len(), 2, "both changes are listed: {out:?}");
        assert_eq!(columns[0], columns[1], "the fractions are ragged: {out:?}");
        // Order is by name and does not follow progress: the done change sorts first here
        // because of its name, not because it is finished.
        let names: Vec<&str> = out
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.split_whitespace().next().expect("a name"))
            .collect();
        assert_eq!(names, vec!["a-considerably-longer-change-name", "short"]);
    }

    /// Writes `tasks` as the tasks document of `change`, which `project_listing` made.
    fn rewrite_tasks(project: &ProjectRoot, change: &str, tasks: &[(Option<&str>, Account)]) {
        std::fs::write(
            project.change(change).join("tasks.html"),
            tasks_document_of(tasks),
        )
        .expect("the tasks document is written");
    }

    /// The measured change: four tasks, one done as written, one done differently, one dropped,
    /// and one still to do.
    const MEASURED: &[(Option<&str>, Account)] = &[
        (Some("done"), Account::AsWritten),
        (Some("done"), Account::Deviating),
        (Some("dropped"), Account::AsWritten),
        (Some("todo"), Account::AsWritten),
    ];

    /// The payload carries the count beside `total`, outside `tasks`, and present when zero.
    ///
    /// The accepted scenario "A deviation leaves the per-state counts whole": `tasks` holds one
    /// count per state, and those sum to `total`. A subset count among them would make a
    /// consumer's sum exceed the total.
    #[test]
    fn the_payload_carries_the_tasks_done_differently_beside_the_per_state_counts() {
        let scratch = Scratch::new("list-deviation-payload");
        let root = project_listing(
            &scratch,
            &[
                ("measure-a-deviation", Some(&[Some("todo")])),
                ("as-written", Some(&[Some("done"), Some("todo")])),
                ("no-tasks", None),
            ],
        );
        rewrite_tasks(&root, "measure-a-deviation", MEASURED);

        let (outcome, out, _) = run(Form::Structured, |streams| {
            list_in(&root, false, false, streams)
        });
        assert_eq!(outcome, Outcome::Success);
        let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
        let entry = |name: &str| -> serde_json::Value {
            payload["changes"]
                .as_array()
                .expect("an array")
                .iter()
                .find(|entry| entry["name"] == name)
                .expect("the change is enumerated")
                .clone()
        };

        let measured = entry("measure-a-deviation");
        assert_eq!(measured["doneDifferently"], 1, "{measured}");
        assert_eq!(measured["tasks"]["done"], 2, "{measured}");
        assert!(
            measured["tasks"].get("doneDifferently").is_none(),
            "the count sits among the per-state counts: {measured}"
        );
        let summed: u64 = measured["tasks"]
            .as_object()
            .expect("the counts are an object")
            .values()
            .map(|count| count.as_u64().expect("a count"))
            .sum();
        assert_eq!(
            Some(summed),
            measured["total"].as_u64(),
            "the per-state counts no longer sum to the total: {measured}"
        );

        assert_eq!(
            entry("as-written")["doneDifferently"],
            0,
            "a payload states the fact, zero included: {}",
            entry("as-written")
        );
        assert!(
            entry("no-tasks").get("doneDifferently").is_none(),
            "a change with no tasks document carries a count: {}",
            entry("no-tasks")
        );
    }

    /// The row shows the count after the state, and the columns stay aligned around it.
    ///
    /// The accepted scenario "A change with a task done differently", in the human form. The
    /// fraction keeps every `done` task in its numerator, and the predecessor moves right on every
    /// row by the same amount.
    #[test]
    fn the_row_shows_the_tasks_done_differently_after_the_state() {
        let scratch = Scratch::new("list-deviation-row");
        let root = project_listing(
            &scratch,
            &[
                ("measure-a-deviation", Some(&[Some("todo")])),
                ("the-contract", Some(&[Some("todo")])),
                ("waiting", Some(&[Some("todo")])),
            ],
        );
        rewrite_tasks(&root, "measure-a-deviation", MEASURED);
        declare_follows(&root, "waiting", "the-contract");

        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, false, streams));

        assert_eq!(outcome, Outcome::Success);
        assert_eq!(
            out,
            "measure-a-deviation  2/4  in-progress  1 done differently\n\
             the-contract         0/1  todo\n\
             waiting              0/1  todo                             \
             follows the-contract (active)\n",
            "the listing is not the one the design states"
        );
    }

    /// A listing in which no task was done differently prints exactly what it printed before the
    /// count existed.
    ///
    /// The accepted scenario "A change whose every task was done as written". A figure reading
    /// zero on nearly every row teaches a reader to pass over it, so nothing is shown, and the
    /// column is not reserved either: the predecessor stays where it was.
    #[test]
    fn a_listing_with_no_task_done_differently_prints_what_it_printed_before() {
        let scratch = Scratch::new("list-deviation-none");
        let root = project_listing(
            &scratch,
            &[
                ("measure-a-deviation", Some(&[Some("todo")])),
                ("the-contract", Some(&[Some("todo")])),
                ("waiting", Some(&[Some("todo")])),
            ],
        );
        let as_written: Vec<(Option<&str>, Account)> = MEASURED
            .iter()
            .map(|(state, _)| (*state, Account::AsWritten))
            .collect();
        rewrite_tasks(&root, "measure-a-deviation", &as_written);
        declare_follows(&root, "waiting", "the-contract");

        let (_, out, _) = run(Form::Human, |streams| list_in(&root, false, false, streams));

        assert_eq!(
            out,
            "measure-a-deviation  2/4  in-progress\n\
             the-contract         0/1  todo\n\
             waiting              0/1  todo         follows the-contract (active)\n",
            "a listing with nothing done differently changed"
        );
    }

    /// The completion report counts and names the tasks done differently.
    ///
    /// The accepted scenario "A change with a task done differently", for the completion
    /// operation. The task is counted as complete, and the report names it, because for such a
    /// task the deviation and not its words is the claim a reviewer checks.
    #[test]
    fn the_completion_report_names_the_tasks_done_differently() {
        let scratch = Scratch::new("status-deviation");
        let (_, directory) = change_with_accounts(&scratch, MEASURED);

        let (outcome, out, _) = run(Form::Structured, |streams| status_of(&directory, streams));
        assert_eq!(outcome, Outcome::Success);
        let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
        assert_eq!(
            payload["tasks"],
            serde_json::json!({
                "total": 4,
                "open": 1,
                "openTasks": ["task-3"],
                "doneDifferently": 1,
                "doneDifferentlyTasks": ["task-1"],
            }),
            "{payload}"
        );

        let (_, human, _) = run(Form::Human, |streams| status_of(&directory, streams));
        assert!(
            human.ends_with("  3 of 4 task(s) done or dropped\n  1 done differently: task-1\n"),
            "{human:?}"
        );
    }

    /// A change whose every task was done as written reports a zero count and prints no line.
    #[test]
    fn the_completion_report_of_work_done_as_written_names_none() {
        let scratch = Scratch::new("status-as-written");
        let (_, directory) = change_with_tasks(&scratch, &[Some("done"), Some("todo")]);

        let (_, out, _) = run(Form::Structured, |streams| status_of(&directory, streams));
        let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
        assert_eq!(payload["tasks"]["doneDifferently"], 0, "{payload}");
        assert_eq!(
            payload["tasks"]["doneDifferentlyTasks"],
            serde_json::json!([]),
            "{payload}"
        );

        let (_, human, _) = run(Form::Human, |streams| status_of(&directory, streams));
        assert!(
            human.ends_with("  1 of 2 task(s) done or dropped\n"),
            "a report with nothing done differently changed: {human:?}"
        );
    }

    /// A project with no changes says so, rather than printing an empty table.
    #[test]
    fn an_empty_project_reports_no_active_changes() {
        let scratch = Scratch::new("list-empty");
        let root = project_listing(&scratch, &[]);
        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, false, streams));

        assert_eq!(outcome, Outcome::Success);
        assert_eq!(out, "no active changes\n");
    }

    // -------------------------------------------------------------------------
    // The stated sequence, in the listing
    // -------------------------------------------------------------------------

    /// States that `change` follows `predecessor`.
    fn declare_follows(project: &ProjectRoot, change: &str, predecessor: &str) {
        std::fs::write(
            project.change(change).join(crate::project::FOLLOWS),
            format!("{predecessor}\n"),
        )
        .expect("the declaration is written");
    }

    /// Archives `name` under the dated directory archival would give it.
    fn archive_as(project: &ProjectRoot, dated: &str) {
        std::fs::create_dir_all(project.archive().join(dated)).expect("the archived change exists");
    }

    /// The payload carries each change's predecessor and whether it resolved.
    ///
    /// The accepted scenarios `A change whose predecessor is unfinished` and `A change whose
    /// predecessor is done`. Both arms are checked in one project, because the point is that the
    /// listing separates them.
    #[test]
    fn the_payload_carries_the_predecessor_and_its_resolution() {
        let scratch = Scratch::new("list-follows");
        let root = project_listing(
            &scratch,
            &[
                ("waiting", Some(&[Some("todo")])),
                ("ready", Some(&[Some("done")])),
                ("solitary", Some(&[Some("todo")])),
                ("the-contract", Some(&[Some("todo")])),
            ],
        );
        declare_follows(&root, "waiting", "the-contract");
        declare_follows(&root, "ready", "an-archived-change");
        archive_as(&root, "2026-09-02-an-archived-change");

        let (outcome, out, _) = run(Form::Structured, |streams| {
            list_in(&root, false, false, streams)
        });
        assert_eq!(outcome, Outcome::Success);
        let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
        let entry = |name: &str| -> serde_json::Value {
            payload["changes"]
                .as_array()
                .expect("an array")
                .iter()
                .find(|entry| entry["name"] == name)
                .expect("the change is enumerated")
                .clone()
        };

        let waiting = entry("waiting");
        assert_eq!(waiting["follows"]["change"], "the-contract");
        assert_eq!(waiting["follows"]["resolved"], false);
        assert_eq!(waiting["follows"]["resolution"], "active");

        let ready = entry("ready");
        assert_eq!(ready["follows"]["change"], "an-archived-change");
        assert_eq!(ready["follows"]["resolved"], true);
        assert_eq!(ready["follows"]["resolution"], "archived");

        // A change stating no predecessor omits the field rather than carrying a null, the same
        // way an absent completion is omitted rather than zeroed.
        assert!(
            entry("solitary").get("follows").is_none(),
            "a change that follows nothing carries a predecessor: {}",
            entry("solitary")
        );
    }

    /// An unmatched name is distinguishable from an unfinished predecessor in the payload.
    ///
    /// The two call for different work — a correction and a wait — so a caller must be able to
    /// tell them apart without reading a report.
    #[test]
    fn the_payload_separates_an_unmatched_name_from_an_unfinished_predecessor() {
        let scratch = Scratch::new("list-unmatched");
        let root = project_listing(
            &scratch,
            &[
                ("misnamed", Some(&[Some("todo")])),
                ("waiting", Some(&[Some("todo")])),
                ("the-contract", Some(&[Some("todo")])),
            ],
        );
        declare_follows(&root, "misnamed", "a-change-that-never-existed");
        declare_follows(&root, "waiting", "the-contract");

        let (_, out, _) = run(Form::Structured, |streams| {
            list_in(&root, false, false, streams)
        });
        let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
        let resolution = |name: &str| -> String {
            payload["changes"]
                .as_array()
                .expect("an array")
                .iter()
                .find(|entry| entry["name"] == name)
                .expect("enumerated")["follows"]["resolution"]
                .as_str()
                .expect("a resolution")
                .to_owned()
        };

        assert_eq!(resolution("misnamed"), "unmatched");
        assert_eq!(resolution("waiting"), "active");
        assert_ne!(
            resolution("misnamed"),
            resolution("waiting"),
            "a wrong name and an unfinished predecessor are indistinguishable"
        );
    }

    /// Task 4.2: the columns still line up when some changes state a predecessor and others
    /// do not.
    ///
    /// The alignment the completion columns established was a stated requirement one change ago,
    /// and this is the case that would break it: a fact printed on some rows and not others.
    #[test]
    fn the_columns_line_up_when_only_some_changes_follow_another() {
        let scratch = Scratch::new("list-follows-aligned");
        let root = project_listing(
            &scratch,
            &[
                ("a-considerably-longer-change-name", Some(&[Some("done")])),
                ("middle", Some(&[Some("todo"), Some("done")])),
                ("short", Some(&[Some("todo")])),
                ("zzz-no-tasks", None),
            ],
        );
        // Two of the four follow something, and the two differ in resolution.
        declare_follows(&root, "middle", "a-considerably-longer-change-name");
        declare_follows(&root, "short", "an-archived-change");
        archive_as(&root, "2026-09-02-an-archived-change");

        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, false, streams));
        assert_eq!(outcome, Outcome::Success);

        let rows: Vec<&str> = out.lines().filter(|line| !line.is_empty()).collect();
        assert_eq!(rows.len(), 4, "every change is listed: {out:?}");

        // The fraction column is where it was: one column for every row that has a fraction,
        // and the change with no tasks says so in the same column.
        let fractions: Vec<usize> = rows
            .iter()
            .filter(|row| row.contains('/'))
            .map(|row| row.find('/').expect("a fraction"))
            .collect();
        assert!(
            fractions.windows(2).all(|pair| pair[0] == pair[1]),
            "the fractions are ragged: {out:?}"
        );

        // And the predecessors begin at one column, on the rows that have one.
        let followed: Vec<usize> = rows.iter().filter_map(|row| row.find("follows ")).collect();
        assert_eq!(followed.len(), 2, "two changes follow another: {out:?}");
        assert_eq!(
            followed[0], followed[1],
            "the predecessors are ragged: {out:?}"
        );

        // The unresolved one is marked and the resolved one is not, because the listing exists
        // for selection and an open predecessor is the reason not to select.
        let middle = rows
            .iter()
            .find(|row| row.starts_with("middle"))
            .expect("the row is present");
        assert!(middle.contains("(active)"), "{middle:?}");
        let short = rows
            .iter()
            .find(|row| row.starts_with("short"))
            .expect("the row is present");
        assert!(
            short.contains("follows an-archived-change") && !short.contains('('),
            "an archived predecessor is reported as resolved and unqualified: {short:?}"
        );

        // No row carries trailing padding, which is what the state column's widening could
        // otherwise leave behind on a change that follows nothing.
        for row in &rows {
            assert_eq!(*row, row.trim_end(), "trailing padding on {row:?}");
        }
    }

    /// The order is by name and does not follow the stated sequence.
    ///
    /// The accepted scenario `Order does not follow the stated sequence`. The declarations here
    /// state the reverse of name order on purpose: `alpha` follows `zulu`, so a topological
    /// listing would put `zulu` first.
    #[test]
    fn the_order_does_not_follow_the_stated_sequence() {
        let scratch = Scratch::new("list-follows-order");
        let root = project_listing(
            &scratch,
            &[
                ("alpha", Some(&[Some("todo")])),
                ("zulu", Some(&[Some("todo")])),
            ],
        );
        declare_follows(&root, "alpha", "zulu");

        let (_, out, _) = run(Form::Human, |streams| list_in(&root, false, false, streams));
        let names: Vec<&str> = out
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.split_whitespace().next().expect("a name"))
            .collect();
        assert_eq!(
            names,
            vec!["alpha", "zulu"],
            "the listing reordered itself to follow the sequence it reports: {out:?}"
        );
    }

    // -------------------------------------------------------------------------
    // The runbook enumeration
    // -------------------------------------------------------------------------

    /// A project holding the runbooks each entry names, with the documents it lists.
    ///
    /// The documents are written from the real templates through `render_kind`, which is the code
    /// `htmlspec new document` uses. A hand-written stand-in would let a template change go
    /// unnoticed here, and the whole claim being tested is that a runbook's documents are
    /// ordinary documents of existing kinds.
    fn project_with_runbooks(scratch: &Scratch, runbooks: &[(&str, &[&str])]) -> ProjectRoot {
        let root = scratch.0.join("project");
        let project = ProjectRoot::establish(&root).expect("the project is established");
        // The store as well as the directories. `establish` makes the places and
        // `establish_assets` fills the store, which is the division `bootstrap` already has —
        // and a runbook's documents reference the store, so a project without one would fail
        // these for a reason that is not what they test.
        let (_, _, _) = run(Form::Human, |streams| {
            establish_assets(&project.assets(), streams).expect("the store is established");
            Outcome::Success
        });
        for (name, kinds) in runbooks {
            let directory = project.runbook(name);
            std::fs::create_dir_all(&directory).expect("the runbook directory exists");
            for kind in *kinds {
                let file = RUNBOOK_DOCUMENTS
                    .iter()
                    .find(|(_file, declared)| declared == kind)
                    .map(|(file, _declared)| *file)
                    .expect("the kind is one a runbook holds");
                let rendered = htmlspec_assets::render_kind(kind, name, "../../assets")
                    .expect("the template for the kind exists");
                std::fs::write(directory.join(file), rendered).expect("the document is written");
            }
        }
        project
    }

    /// The enumeration names each runbook once and states which documents it holds.
    ///
    /// **One entry per runbook, which is the property that separates this from the `--specs`
    /// arm.** That arm walks for documents, so applied to runbooks it would list `release`'s two
    /// files as two rows. The `release` entry here holds both documents on purpose: a walk would
    /// produce three rows for these two runbooks and this asserts two.
    #[test]
    fn the_enumeration_names_each_runbook_once() {
        let scratch = Scratch::new("runbooks-listed");
        let root = project_with_runbooks(
            &scratch,
            &[
                ("chocolate-chip-cookies", &["tasks"]),
                ("release", &["specification", "tasks"]),
            ],
        );

        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, true, streams));

        assert_eq!(outcome, Outcome::Success);
        // A runbook's row starts at the margin. The lines indented beneath it state its
        // prerequisites, or that its declarations cannot be read, and are not runbooks.
        let rows: Vec<&str> = out
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with(' '))
            .collect();
        assert_eq!(rows.len(), 2, "one row per runbook: {out:?}");
        let names: Vec<&str> = rows
            .iter()
            .map(|row| row.split_whitespace().next().expect("a name"))
            .collect();
        assert_eq!(names, vec!["chocolate-chip-cookies", "release"]);
        assert!(
            rows[1].contains("specification, tasks"),
            "the row does not state both documents: {:?}",
            rows[1]
        );
        assert!(
            rows[0].contains("tasks") && !rows[0].contains("specification"),
            "the row states a document the runbook does not hold: {:?}",
            rows[0]
        );
        for row in &rows {
            assert_eq!(*row, row.trim_end(), "trailing padding on {row:?}");
        }
    }

    /// A runbook directory holding neither document says so rather than printing a blank column.
    ///
    /// The same treatment `no tasks` gets for a change: a gap reads as a rendering fault, and
    /// this is a fact about the runbook. It is also the fact a selector most needs — a runbook
    /// with no procedure is not one to run.
    #[test]
    fn a_runbook_with_no_documents_says_so() {
        let scratch = Scratch::new("runbooks-empty-directory");
        let root = project_with_runbooks(&scratch, &[("started", &[])]);

        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, true, streams));

        assert_eq!(outcome, Outcome::Success);
        assert!(
            out.contains("no documents"),
            "the row leaves the column blank: {out:?}"
        );
    }

    /// A project with no runbooks reports that, through the path the other subjects use.
    #[test]
    fn a_project_with_no_runbooks_reports_none() {
        let scratch = Scratch::new("runbooks-none");
        let root = project_with_runbooks(&scratch, &[]);

        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, true, streams));

        assert_eq!(outcome, Outcome::Success);
        assert_eq!(out, "no active runbooks\n");
    }

    /// The enumeration reads the place rather than a list, so a directory created by hand appears.
    ///
    /// The accepted scenario `An enumeration nobody maintains`. Written by creating the directory
    /// directly rather than through any operation, because that is the case: nothing registers a
    /// runbook, and a maintained registry is the shape `project-bootstrap` refuses.
    #[test]
    fn a_runbook_added_by_hand_is_enumerated() {
        let scratch = Scratch::new("runbooks-unmaintained");
        let root = project_with_runbooks(&scratch, &[]);
        std::fs::create_dir_all(root.runbook("added-by-hand")).expect("the directory exists");

        let (_, out, _) = run(Form::Human, |streams| list_in(&root, false, true, streams));

        assert!(
            out.contains("added-by-hand"),
            "a runbook nothing registered is not enumerated: {out:?}"
        );
    }

    /// A loose file beside the runbooks is not a runbook.
    ///
    /// **Added because mutation testing found the gap rather than because the case looked
    /// likely.** Removing the `is_dir` filter from the enumeration failed none of the other
    /// eight tests: every one of them creates only directories under `runbooks/`, so the filter
    /// was unguarded and would have been deleted by a later change with the suite green. A
    /// runbook is a directory holding two documents, and a stray `README.md` or a `.DS_Store`
    /// there is not one.
    #[test]
    fn a_loose_file_beside_the_runbooks_is_not_one() {
        let scratch = Scratch::new("runbooks-loose-file");
        let root = project_with_runbooks(&scratch, &[("release", &["tasks"])]);
        std::fs::write(root.runbooks().join("NOTES.md"), "not a runbook")
            .expect("the loose file is written");

        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, true, streams));

        assert_eq!(outcome, Outcome::Success);
        // A runbook's row starts at the margin. The lines indented beneath it state its
        // prerequisites, or that its declarations cannot be read, and are not runbooks.
        let rows: Vec<&str> = out
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with(' '))
            .collect();
        assert_eq!(rows.len(), 1, "the loose file was enumerated: {out:?}");
        assert!(
            !out.contains("NOTES.md"),
            "a file is presented as a runbook: {out:?}"
        );
    }

    /// A project bootstrapped before runbooks existed enumerates none rather than failing.
    #[test]
    fn an_absent_runbooks_directory_enumerates_none() {
        let scratch = Scratch::new("runbooks-absent");
        let root = project_with_runbooks(&scratch, &[]);
        std::fs::remove_dir_all(root.runbooks()).expect("the directory is removed");

        let (outcome, out, _) = run(Form::Human, |streams| list_in(&root, false, true, streams));

        assert_eq!(outcome, Outcome::Success);
        assert_eq!(out, "no active runbooks\n");
    }

    /// The payload is keyed by the word the flag uses, with the documents as an array.
    #[test]
    fn the_payload_states_each_runbooks_documents() {
        let scratch = Scratch::new("runbooks-payload");
        let root = project_with_runbooks(
            &scratch,
            &[("release", &["specification", "tasks"]), ("started", &[])],
        );

        let (outcome, out, _) = run(Form::Structured, |streams| {
            list_in(&root, false, true, streams)
        });

        assert_eq!(outcome, Outcome::Success);
        let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
        let runbooks = payload["runbooks"].as_array().expect("an array");
        assert_eq!(runbooks.len(), 2, "{payload}");
        let release = runbooks
            .iter()
            .find(|entry| entry["name"] == "release")
            .expect("the runbook is enumerated");
        assert_eq!(
            release["documents"],
            serde_json::json!(["specification", "tasks"]),
            "{release}"
        );
        // An empty array rather than an omitted field, which is the opposite of the change
        // payload's treatment of an absent completion — and deliberately so. There, omission
        // distinguishes an unwritten document from finished work. Here, "this runbook holds no
        // documents" is the fact itself, so a caller reads it rather than checking for absence.
        let started = runbooks
            .iter()
            .find(|entry| entry["name"] == "started")
            .expect("the runbook is enumerated");
        assert_eq!(started["documents"], serde_json::json!([]), "{started}");
    }

    /// A runbook's documents are accepted by the ordinary validation path, with no rule added.
    ///
    /// **This is the claim the whole change rests on**, asserted here rather than only measured
    /// by hand: a runbook's documents are documents of existing kinds, so nothing in the
    /// validator knows or needs to know that they belong to a runbook.
    #[test]
    fn a_runbooks_documents_validate_as_documents_of_their_kinds() {
        let scratch = Scratch::new("runbooks-validate");
        let root = project_with_runbooks(&scratch, &[("release", &["specification", "tasks"])]);

        for (file, kind) in RUNBOOK_DOCUMENTS {
            let path = root.runbook("release").join(file);
            let source = std::fs::read_to_string(&path).expect("the document reads");
            // `store_for` walks upward from the document, which is exactly what the `validate`
            // command does. **That the walk finds the project's one store from two levels down
            // is the property being asserted** — it is why a runbook needs no store of its own
            // and why `asset/duplicate` has nothing to report about one.
            let store = store_for(&path);
            assert!(
                store.is_present(),
                "the walk from a runbook's document does not reach the project's store"
            );
            // `validate_source` rather than a reassembled pipeline: it is the function the
            // `validate` command runs, so this asserts against the path a contributor invokes.
            let report =
                validate_source(&source, &path, &mut Reach::default(), Strictness::Default);
            assert!(
                report.is_accepted(),
                "a runbook's {kind} document is not accepted: {}",
                htmlspec_validate::render_human(&report)
            );
            // And under `--strict`, because a runbook's documents are validated by the ordinary
            // path and strict is part of it. A `review/*` warning is the accepted case and
            // cannot be promoted, so this asserts acceptance rather than silence.
            let strict = validate_source(&source, &path, &mut Reach::default(), Strictness::Strict);
            assert!(
                strict.is_accepted(),
                "a runbook's {kind} document is not accepted under --strict: {}",
                htmlspec_validate::render_human(&strict)
            );
        }
    }

    /// The forced split, asserted at the rule that forces it.
    ///
    /// `artifact-kinds` states that a specification SHALL NOT carry task units and that this is
    /// the only placement rule in the vocabulary. **That is why a runbook is two documents rather
    /// than one**, so this asserts the rule rather than trusting the prose: a procedure written
    /// into the canonical document is refused as an error, and a `tasks` document with no steps
    /// is reported too.
    #[test]
    fn the_two_document_split_is_forced_by_validation() {
        let scratch = Scratch::new("runbooks-split");
        let root = project_with_runbooks(&scratch, &[("release", &["specification"])]);
        let path = root.runbook("release").join(CHANGE_SPECIFICATION);
        let accepted = std::fs::read_to_string(&path).expect("the document reads");

        // The procedure moved into the canonical document, which is what the split prevents.
        let tasks = htmlspec_assets::render_kind("tasks", "release", "../../assets")
            .expect("the tasks template exists");
        let opening = tasks
            .find("<li class=\"task\"")
            .expect("the template states a task");
        let closing = tasks.rfind("</li>").expect("the task list closes") + "</li>".len();
        let step = &tasks[opening..closing];
        let drifted = accepted.replacen("<main", &format!("<ul>{step}</ul><main"), 1);

        let report = validate_source(&drifted, &path, &mut Reach::default(), Strictness::Default);
        assert!(
            !report.is_accepted(),
            "a specification carrying the procedure is accepted, so nothing forces the split"
        );
        let rendered = htmlspec_validate::render_human(&report);
        assert!(
            rendered.contains("kind/task-in-specification"),
            "the refusal is for some other reason: {rendered}"
        );
    }

    /// **A runbook is refused by the resolver every change-taking operation shares.**
    ///
    /// The regression test for a measured data loss: `htmlspec archive htmlspec/runbooks/release
    /// --yes` moved the runbook into `changes/archive/2026-09-17-release` and left `runbooks/`
    /// empty. `runbook-authority` states that a procedure run repeatedly is not a change and does
    /// not archive, and nothing checked it.
    ///
    /// **The full clearing shape is what makes this test able to fail.** Four unrelated gates
    /// refuse before archival moves anything — the version gate, the absent-`specification.html`
    /// gate, the example-identifier gate, and the placeholder-summary gate — so a runbook missing
    /// any of them is refused for the wrong reason and the test would pass against the defect.
    /// This resolver-level test needs the first three; `tests/archive.rs` drives the whole command
    /// and needs the fourth as well, which is where the count was corrected from three.
    ///
    /// Asserted at [`resolve_change`] rather than at `archive`, because `archive` resolves its
    /// project from the process's current directory and a test that changed directories would
    /// race every other test in this binary. The resolver is where the guard is and where every
    /// caller passes through; `tests/archive.rs` covers the whole command from the outside.
    #[test]
    fn a_runbook_is_refused_where_a_change_is_expected() {
        let scratch = Scratch::new("runbooks-not-a-change");
        let root = project_with_runbooks(&scratch, &[("release", &["specification", "tasks"])]);
        // The example identifiers replaced, so the example-content gate has nothing to refuse.
        // Without this the archival path is blocked by a gate unrelated to runbooks.
        for (file, _kind) in RUNBOOK_DOCUMENTS {
            let path = root.runbook("release").join(file);
            let document = std::fs::read_to_string(&path).expect("the document reads");
            std::fs::write(&path, document.replace("-example", "-real"))
                .expect("the document is rewritten");
        }
        // And the version advanced past the initial one, so the version gate is satisfied too.
        std::fs::write(
            root.declaration(),
            render_declaration("scratch").replace("0.0.0", "0.1.0"),
        )
        .expect("the declaration is written");

        let directory = root.runbook("release");
        for refusal in [Refusal::Archival, Refusal::Comparison, Refusal::Report] {
            // The guard directly, with the scratch project passed in. **`resolve_change` is no
            // longer the right subject for a unit test, and that is the fix working**: it resolves
            // the project from the process's current directory, which is the whole point — the
            // guard judges against the project the caller is operating in rather than one derived
            // from the argument. A unit test would have to change directories to reach it, so
            // `tests/archive.rs` drives the real command instead.
            let report = refuse_a_runbook(&root, &directory, refusal)
                .expect_err("a runbook resolved as a change, which archival would destroy");
            assert!(
                report.contains("is a runbook, not a change"),
                "the refusal does not name what the directory is: {report}"
            );
            // Not refused by one of the three gates that happen to fire on a half-built runbook.
            // Those would pass this test against the defect, which is the whole reason the
            // clearing shape above exists.
            for wrong in ["version", "specification.html", "example", "placeholder"] {
                assert!(
                    !report.contains(wrong),
                    "refused for {wrong:?} rather than for being a runbook: {report}"
                );
            }
            // And it says where to go, per this project's register on reports.
            assert!(
                report.contains("htmlspec list --runbooks"),
                "the refusal leaves the caller nowhere to go: {report}"
            );
        }
    }

    /// Each operation states its own reason, because a shared one would be false for two of them.
    ///
    /// `archive` would destroy the runbook, `sync` would direct a merge the contract forbids, and
    /// `status` would report it against a shape it does not have. A refusal whose stated reason a
    /// contributor can check and find wrong makes the refusal itself look unreliable.
    #[test]
    fn each_operation_refuses_a_runbook_for_its_own_reason() {
        assert!(Refusal::Archival.reason().contains("not archived"));
        assert!(Refusal::Comparison
            .reason()
            .contains("accepted specifications"));
        assert!(Refusal::Report.reason().contains("two documents"));
        // "not archived" is the archival reason and is false for the other two, which is the
        // defect this enum replaced — the first fix said it for all three.
        assert!(!Refusal::Comparison.reason().contains("not archived"));
        assert!(!Refusal::Report.reason().contains("not archived"));
        // The view directs no merge, so the comparison's reason, which is about requirements to
        // merge, would be false for it.
        assert!(Refusal::View
            .reason()
            .contains("no accepted specification holds"));
        assert!(!Refusal::View.reason().contains("merge"));
        assert!(!Refusal::View.reason().contains("not archived"));
    }

    /// A path reaching the runbooks directory by traversal is refused too.
    ///
    /// **This is the case the first version of the guard let through, and it destroyed the
    /// runbook with the guard in place.** `Path::components` drops `.` and keeps `..`, so
    /// `htmlspec/changes/../runbooks/release` compared against `htmlspec/runbooks` found no
    /// prefix. It is reachable in practice: `project.change("../runbooks/release")` composes
    /// exactly that, which is the `--store` arm's only route.
    #[test]
    fn a_runbook_reached_by_traversal_is_refused() {
        let scratch = Scratch::new("runbooks-traversal");
        let root = project_with_runbooks(&scratch, &[("release", &["tasks"])]);

        let traversed = root.change("../runbooks/release");
        assert!(
            traversed.is_dir(),
            "the traversal does not reach the runbook"
        );
        let report = refuse_a_runbook(&root, &traversed, Refusal::Archival)
            .expect_err("a runbook reached by traversal resolved as a change");
        assert!(report.contains("is a runbook, not a change"), "{report}");
    }

    /// A change is still resolved, so the guard refuses runbooks rather than directories.
    #[test]
    fn a_change_beside_a_runbook_still_resolves() {
        let scratch = Scratch::new("runbooks-change-unaffected");
        let root = project_with_runbooks(&scratch, &[("release", &["tasks"])]);
        let change = root.change("a-real-change");
        std::fs::create_dir_all(&change).expect("the change directory exists");

        assert_eq!(
            refuse_a_runbook(&root, &change, Refusal::Archival),
            Ok(()),
            "the guard refuses a change directory as well as a runbook"
        );
    }

    /// Naming a subject leaves the other two subjects' output exactly as it was.
    ///
    /// The regression this change most plausibly introduces: `list` and `list --specs` are what
    /// every existing caller runs, and a third arm added to the enumeration must not move
    /// either. Asserted by running all three against one project.
    #[test]
    fn the_flag_selects_a_subject_without_moving_the_others() {
        let scratch = Scratch::new("runbooks-subject");
        let root = project_listing(&scratch, &[("a-change", Some(&[Some("todo")]))]);
        std::fs::create_dir_all(root.runbook("release")).expect("the runbook exists");

        let (_, changes, _) = run(Form::Human, |streams| list_in(&root, false, false, streams));
        let (_, specs, _) = run(Form::Human, |streams| list_in(&root, true, false, streams));
        let (_, runbooks, _) = run(Form::Human, |streams| list_in(&root, false, true, streams));

        assert!(
            changes.contains("a-change") && !changes.contains("release"),
            "the default listing changed: {changes:?}"
        );
        assert_eq!(
            specs, "no active specifications\n",
            "the specifications listing changed: {specs:?}"
        );
        assert!(
            runbooks.contains("release") && !runbooks.contains("a-change"),
            "the runbook listing carries a change: {runbooks:?}"
        );
    }

    /// Both flags together resolves to one subject rather than to a state meaning nothing.
    #[test]
    fn both_flags_resolve_to_the_runbooks() {
        assert_eq!(Subject::of(false, false), Subject::Changes);
        assert_eq!(Subject::of(true, false), Subject::Specifications);
        assert_eq!(Subject::of(false, true), Subject::Runbooks);
        assert_eq!(Subject::of(true, true), Subject::Runbooks);
    }

    /// A specification stating each `(identifier, heading, previous heading)` in `requirements`.
    ///
    /// Built from the template, as [`project_stating`] builds its documents. The first entry takes
    /// over the template's requirement and the others are copies inserted before it. A requirement
    /// with a previous heading carries a `renamed-from` paragraph and `data-change="modified"`,
    /// which is how a change renaming and rewriting it states it.
    fn specification_with(requirements: &[(&str, &str, Option<&str>)]) -> String {
        let mut document = valid_document();
        let Some(((first, _, _), rest)) = requirements.split_first() else {
            return document;
        };
        document = document.replace("req-example", first);
        document = document.replacen(
            &format!("<h3>The example requirement <span class=\"hs-id\">{first}</span></h3>"),
            &format!("<h3>{{heading:{first}}} <span class=\"hs-id\">{first}</span></h3>{{rename:{first}}}"),
            1,
        );
        for (id, _, _) in rest {
            let section = format!(
                "<section class=\"requirement\" id=\"{id}\" data-change=\"added\">\n            \
                 <h3>{{heading:{id}}} <span class=\"hs-id\">{id}</span></h3>{{rename:{id}}}\n            \
                 <p>The system SHALL state one obligation per requirement, in a sentence a \
                 reader can verify.</p>\n          </section>\n          "
            );
            let marker = format!("<section class=\"requirement\" id=\"{first}\"");
            document = document.replacen(&marker, &format!("{section}{marker}"), 1);
        }
        for (id, heading, previous) in requirements {
            document = document.replace(&format!("{{heading:{id}}}"), heading);
            let rename = previous
                .map(|previous| {
                    format!("\n            <p data-field=\"renamed-from\">{previous}</p>")
                })
                .unwrap_or_default();
            document = document.replace(&format!("{{rename:{id}}}"), &rename);
            if previous.is_some() {
                document = document.replacen(
                    &format!("id=\"{id}\" data-change=\"added\""),
                    &format!("id=\"{id}\" data-change=\"modified\""),
                    1,
                );
            }
        }
        document
    }

    /// A project whose one accepted specification is `accepted`, and one change whose
    /// specification is `stated`, returning the projected change specification.
    fn project_holding(
        scratch: &Scratch,
        accepted: &str,
        stated: &str,
    ) -> (ProjectRoot, PathBuf, htmlspec_parse::Projection) {
        let root = scratch.0.join("project");
        let project = ProjectRoot::establish(&root).expect("the project is established");
        std::fs::write(project.specs().join("accepted.html"), accepted)
            .expect("the accepted specification is written");
        let directory = project.change("a-change");
        std::fs::create_dir_all(&directory).expect("the change directory exists");
        std::fs::write(directory.join(CHANGE_SPECIFICATION), stated)
            .expect("the change specification is written");
        let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(stated).document);
        (project, directory, projection)
    }

    /// The toolchain-packaging scenario "A rename the accepted specifications have not taken".
    #[test]
    fn a_rename_the_accepted_specifications_have_not_taken_is_not_reached() {
        let scratch = Scratch::new("rename-untaken");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_with(&[("req-x", "The old name", None)]),
            &specification_with(&[("req-x", "The new name", Some("The old name"))]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.stated, vec![String::from("req-x")]);
        assert!(
            state.reached.is_empty(),
            "an untaken rename was reached: {state:?}"
        );
        assert!(state.absent.is_empty(), "{state:?}");
        assert_eq!(
            state.untaken_renames,
            vec![UntakenRename {
                id: String::from("req-x"),
                heading: HeadingName::of("The new name", None),
                accepted: Some(HeadingName::of("The old name", None)),
            }]
        );
    }

    /// The toolchain-packaging scenario "A rename the accepted specifications have taken", which is
    /// OpenSpec's "source gone, target present": the rename was applied before archival.
    #[test]
    fn a_rename_the_accepted_specifications_have_taken_is_reached_and_nothing_else() {
        let scratch = Scratch::new("rename-taken");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_with(&[("req-x", "The new name", None)]),
            &specification_with(&[("req-x", "The new name", Some("The old name"))]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.reached, vec![String::from("req-x")]);
        assert!(state.absent.is_empty(), "{state:?}");
        assert!(state.untaken_renames.is_empty(), "{state:?}");
    }

    /// The toolchain-packaging scenario "A renamed requirement whose identifier is not held".
    #[test]
    fn a_renamed_requirement_whose_identifier_is_not_held_is_absent_only() {
        let scratch = Scratch::new("rename-absent");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_with(&[("req-other", "The old name", None)]),
            &specification_with(&[("req-x", "The new name", Some("The old name"))]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.absent, vec![String::from("req-x")]);
        assert!(
            state.untaken_renames.is_empty(),
            "one requirement was reported twice: {state:?}"
        );
        assert!(state.reached.is_empty(), "{state:?}");
    }

    /// A heading that differs where the change states no rename is not this check's subject.
    ///
    /// Design decision 4's rejected alternative: comparing every heading would report an edit made
    /// to the accepted contract directly, or a rename nobody stated, as this change's work.
    #[test]
    fn a_heading_that_differs_with_no_rename_stated_is_reached() {
        let scratch = Scratch::new("rename-unstated");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_with(&[("req-x", "The old name", None)]),
            &specification_with(&[("req-x", "The new name", None)]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.reached, vec![String::from("req-x")]);
        assert!(state.untaken_renames.is_empty(), "{state:?}");
    }

    /// Heading names, not titles: an accepted specification written without the identifier span
    /// holds the same heading as a change written with it.
    #[test]
    fn the_identifier_shown_beside_a_heading_is_not_a_rename() {
        let scratch = Scratch::new("rename-span");
        let accepted = specification_with(&[("req-x", "The new name", None)])
            .replace(" <span class=\"hs-id\">req-x</span>", "");
        assert!(!accepted.contains("<span class=\"hs-id\">req-x</span></h3>"));
        let (project, _, projection) = project_holding(
            &scratch,
            &accepted,
            &specification_with(&[("req-x", "The new name", Some("The old name"))]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.reached, vec![String::from("req-x")], "{state:?}");
        assert!(state.untaken_renames.is_empty(), "{state:?}");
    }

    /// A requirement as a fixture states it, with every part the content comparison reads.
    ///
    /// A builder beside [`specification_with`] rather than a wider tuple for it, so the fixtures
    /// that helper already builds stay byte for byte as they were.
    struct Stated<'a> {
        id: &'a str,
        /// The `data-change` the requirement carries.
        change: &'a str,
        heading: &'a str,
        renamed_from: Option<&'a str>,
        statements: &'a [&'a str],
        /// Each scenario as `(identifier, heading, change state)`. Its entries are written from
        /// its heading, so two scenarios with one heading have one text.
        scenarios: &'a [(&'a str, &'a str, &'a str)],
        notes: &'a [&'a str],
    }

    impl<'a> Stated<'a> {
        /// `id`, marked `change`, with one statement and no scenario, rename, or note.
        fn plain(id: &'a str, change: &'a str) -> Self {
            Self {
                id,
                change,
                heading: "The requirement",
                renamed_from: None,
                statements: &["The system SHALL do it."],
                scenarios: &[],
                notes: &[],
            }
        }

        fn html(&self) -> String {
            let Self {
                id,
                change,
                heading,
                ..
            } = self;
            let rename = self
                .renamed_from
                .map(|previous| format!("<p data-field=\"renamed-from\">{previous}</p>"))
                .unwrap_or_default();
            let statements: String = self
                .statements
                .iter()
                .map(|statement| format!("<p>{statement}</p>"))
                .collect();
            let notes: String = self
                .notes
                .iter()
                .map(|note| format!("<p data-field=\"note\">{note}</p>"))
                .collect();
            let scenarios: String = self
                .scenarios
                .iter()
                .map(|(scenario, title, state)| {
                    format!(
                        "<article class=\"scenario\" id=\"{scenario}\" data-change=\"{state}\">\
                         <h4>{title}</h4><dl><dt>When</dt><dd>{title} happens.</dd><dt>Then</dt>\
                         <dd>{title} holds.</dd></dl></article>"
                    )
                })
                .collect();
            format!(
                "<section class=\"requirement\" id=\"{id}\" data-change=\"{change}\"><h3>{heading} \
                 <span class=\"hs-id\">{id}</span></h3>{rename}{statements}{notes}{scenarios}\
                 </section>\n          "
            )
        }
    }

    /// A specification stating exactly `requirements`, in place of the template's example one.
    fn specification_stating(requirements: &[Stated]) -> String {
        let document = valid_document();
        let start = document
            .find("<section class=\"requirement\" id=\"req-example\"")
            .expect("the template states an example requirement");
        let end = start
            + document[start..]
                .find("\n        </section>\n        <section id=\"hs-decisions\">")
                .expect("the requirements section closes before the decisions");
        let stated: String = requirements.iter().map(Stated::html).collect();
        // The template's evidence and verification name its example requirement, so they name
        // the first stated one instead, and the document keeps no reference to a unit it lacks.
        let first = requirements
            .first()
            .map_or("req-example", |stated| stated.id);
        format!("{}{stated}{}", &document[..start], &document[end..]).replace(
            "data-refs=\"req-example\"",
            &format!("data-refs=\"{first}\""),
        )
    }

    /// Design decision 1's three cases, over the schema's four change states.
    ///
    /// The states are read from the schema by stem rather than written as literals in the rule, so
    /// this pins what each one becomes. A value the schema does not hold is carried forward, which
    /// is how every requirement was judged before the comparison read content.
    #[test]
    fn every_change_state_is_judged_one_way() {
        let judged: Vec<(String, ContractEffect)> =
            htmlspec_validate::schema_facts::enum_values("definitions/changeState")
                .into_iter()
                .map(|state| {
                    let effect = ContractEffect::of(&state);
                    (state, effect)
                })
                .collect();
        assert_eq!(
            judged,
            vec![
                (String::from("added"), ContractEffect::Establishes),
                (String::from("modified"), ContractEffect::Establishes),
                (String::from("removed"), ContractEffect::Withdraws),
                (String::from("unchanged"), ContractEffect::CarriesForward),
            ],
            "the schema's change states are not judged as design decision 1 states"
        );
        assert_eq!(
            ContractEffect::of("pending"),
            ContractEffect::CarriesForward
        );
    }

    /// The accepted `req-x` the content tests start from: one statement, scenarios A and B.
    fn accepted_x() -> Stated<'static> {
        Stated {
            heading: "The x requirement",
            statements: &["The system SHALL do x."],
            scenarios: &[
                ("scenario-x-a", "Scenario A", "unchanged"),
                ("scenario-x-b", "Scenario B", "unchanged"),
            ],
            ..Stated::plain("req-x", "unchanged")
        }
    }

    /// The content entry for `id`, built the one way an entry can be.
    fn untaken(
        id: &str,
        first_differing_statement: usize,
        not_held: &[&str],
        not_kept: &[&str],
    ) -> UntakenContent {
        UntakenContent::new(
            id,
            std::num::NonZeroUsize::new(first_differing_statement),
            not_held.iter().map(|name| (*name).to_owned()).collect(),
            not_kept.iter().map(|name| (*name).to_owned()).collect(),
        )
        .expect("the fixture names a difference")
    }

    /// toolchain-packaging's "The view and the reporting operation agree", over every kind of
    /// requirement whose content the comparison compares.
    ///
    /// The view calls the comparison rather than making one of its own, and this holds it there:
    /// it shows a requirement as differing exactly where `sync` reports content not taken, its
    /// first paragraph not held by both is the one `sync` names, and the scenarios it shows on each
    /// side are the ones `sync` names on each side.
    #[test]
    fn the_view_and_sync_agree_about_what_differs() {
        let scratch = Scratch::new("view-parity");
        let accepted = specification_stating(&[
            Stated {
                heading: "Taken",
                ..Stated::plain("req-taken", "unchanged")
            },
            Stated {
                heading: "Statement",
                statements: &["The system SHALL do it.", "It SHALL do it once."],
                ..Stated::plain("req-statement", "unchanged")
            },
            Stated {
                heading: "Scenario",
                scenarios: &[
                    ("scenario-s-a", "Scenario A", "unchanged"),
                    ("scenario-s-b", "Scenario B", "unchanged"),
                ],
                ..Stated::plain("req-scenario", "unchanged")
            },
            Stated {
                heading: "Withdrawn",
                scenarios: &[
                    ("scenario-w-a", "Scenario WA", "unchanged"),
                    ("scenario-w-b", "Scenario WB", "unchanged"),
                ],
                ..Stated::plain("req-withdrawn", "unchanged")
            },
            Stated {
                heading: "Added",
                statements: &["The system SHALL do something else."],
                ..Stated::plain("req-added", "unchanged")
            },
            Stated {
                heading: "Carried",
                scenarios: &[("scenario-c-a", "Scenario CA", "unchanged")],
                ..Stated::plain("req-carried", "unchanged")
            },
        ]);
        let stated = specification_stating(&[
            Stated {
                heading: "Taken",
                ..Stated::plain("req-taken", "modified")
            },
            Stated {
                heading: "Statement",
                statements: &["The system SHALL do it.", "It SHALL do it twice."],
                ..Stated::plain("req-statement", "modified")
            },
            Stated {
                heading: "Scenario",
                scenarios: &[
                    ("scenario-s-a", "Scenario A", "unchanged"),
                    ("scenario-s-b", "Scenario B altered", "modified"),
                ],
                ..Stated::plain("req-scenario", "modified")
            },
            Stated {
                heading: "Withdrawn",
                scenarios: &[
                    ("scenario-w-a", "Scenario WA", "unchanged"),
                    ("scenario-w-b", "Scenario WB", "removed"),
                ],
                ..Stated::plain("req-withdrawn", "modified")
            },
            Stated {
                heading: "Added",
                ..Stated::plain("req-added", "added")
            },
            Stated {
                heading: "Carried",
                scenarios: &[("scenario-c-a", "Scenario CA altered", "modified")],
                ..Stated::plain("req-carried", "unchanged")
            },
        ]);
        let (project, _, projection) = project_holding(&scratch, &accepted, &stated);
        let state = merge_state(&project, &projection);
        let document = projection
            .document
            .as_ref()
            .expect("the projection keeps its tree");
        let view = crate::show::view(
            "a-change",
            &projection.model,
            &htmlspec_parse::marked_units(document),
            &accepted_requirements(&project),
        );

        let differing: std::collections::BTreeSet<&str> = view
            .requirements
            .iter()
            .filter(|shown| matches!(shown.shown, crate::show::Shown::Differs(..)))
            .map(|shown| shown.id.as_str())
            .collect();
        let untaken: std::collections::BTreeSet<&str> = state
            .untaken_content
            .iter()
            .map(|content| content.id.as_str())
            .collect();
        assert_eq!(
            untaken,
            [
                "req-added",
                "req-carried",
                "req-scenario",
                "req-statement",
                "req-withdrawn"
            ]
            .into_iter()
            .collect(),
            "the fixture no longer covers every case: {state:?}"
        );
        assert_eq!(differing, untaken);

        for content in &state.untaken_content {
            let payload = content.payload();
            let Some(crate::show::Shown::Differs(_, comparison)) = view
                .requirements
                .iter()
                .find(|shown| shown.id == content.id)
                .map(|shown| &shown.shown)
            else {
                panic!("{} is not shown as differing", content.id);
            };

            let first_not_held = comparison
                .statements
                .paragraphs
                .iter()
                .find_map(|paragraph| match paragraph {
                    crate::show::Paragraph::Both { .. } => None,
                    crate::show::Paragraph::Pair { position, .. }
                    | crate::show::Paragraph::Change { position, .. } => Some(position.get()),
                    crate::show::Paragraph::Accepted { accepted, .. } => Some(accepted.get()),
                });
            assert_eq!(
                first_not_held.map(serde_json::Value::from),
                payload["firstDifferingStatement"]
                    .as_u64()
                    .map(serde_json::Value::from),
                "{}",
                content.id
            );

            let mut change_side = Vec::new();
            let mut accepted_side = Vec::new();
            for scenario in &comparison.scenarios {
                match scenario {
                    crate::show::ScenarioShown::Both { .. } => {}
                    crate::show::ScenarioShown::Pair(pair) => {
                        change_side.push(pair.id.clone());
                        accepted_side.push(pair.id.clone());
                    }
                    crate::show::ScenarioShown::Change { scenario, .. } => {
                        change_side.extend(scenario.id.clone());
                    }
                    crate::show::ScenarioShown::Withdrawn { id, still_held, .. } => {
                        if *still_held {
                            accepted_side.extend(id.clone());
                        }
                    }
                    crate::show::ScenarioShown::Accepted(scenario) => {
                        accepted_side.extend(scenario.id.clone());
                    }
                }
            }
            let named = |key: &str| -> Vec<String> {
                let mut names: Vec<String> = payload[key]
                    .as_array()
                    .expect("a list")
                    .iter()
                    .filter_map(|name| name.as_str().map(str::to_owned))
                    .collect();
                names.sort();
                names
            };
            change_side.sort();
            accepted_side.sort();
            assert_eq!(change_side, named("scenariosNotHeld"), "{}", content.id);
            assert_eq!(accepted_side, named("scenariosNotKept"), "{}", content.id);
        }

        assert!(
            matches!(
                view.requirements
                    .iter()
                    .find(|shown| shown.id == "req-taken")
                    .map(|shown| &shown.shown),
                Some(crate::show::Shown::HeldAsStated(_))
            ),
            "the modification taken is not held as stated: {view:#?}"
        );
    }

    /// The toolchain-packaging scenario "A word the accepted specifications have not taken".
    #[test]
    fn a_word_the_accepted_specifications_have_not_taken_is_not_reached() {
        let scratch = Scratch::new("content-word");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[accepted_x()]),
            &specification_stating(&[Stated {
                change: "modified",
                statements: &["The system SHALL do x quickly."],
                ..accepted_x()
            }]),
        );

        let state = merge_state(&project, &projection);

        assert!(
            state.reached.is_empty(),
            "an untaken word was reached: {state:?}"
        );
        assert!(state.absent.is_empty(), "{state:?}");
        assert_eq!(state.untaken_content, vec![untaken("req-x", 1, &[], &[])]);
        assert!(!state.reached_in_full());
    }

    /// The toolchain-packaging scenario "A scenario the accepted specifications have not taken".
    #[test]
    fn a_scenario_the_accepted_specifications_have_not_taken_is_named_by_its_identifier() {
        let scratch = Scratch::new("content-scenario");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[accepted_x()]),
            &specification_stating(&[Stated {
                change: "modified",
                scenarios: &[
                    ("scenario-x-a", "Scenario A", "unchanged"),
                    ("scenario-x-b", "Scenario B", "unchanged"),
                    ("scenario-x-c", "Scenario C", "added"),
                ],
                ..accepted_x()
            }]),
        );

        let state = merge_state(&project, &projection);

        assert!(state.reached.is_empty(), "{state:?}");
        assert_eq!(
            state.untaken_content,
            vec![untaken("req-x", 0, &["scenario-x-c"], &[])]
        );
    }

    /// The toolchain-packaging scenario "A withdrawn scenario the accepted requirement still
    /// holds". It is named as the accepted specification names it.
    #[test]
    fn a_withdrawn_scenario_the_accepted_requirement_still_holds_is_not_kept() {
        let scratch = Scratch::new("content-withdrawn-scenario");
        let accepted = Stated {
            scenarios: &[
                ("scenario-x-a", "Scenario A", "unchanged"),
                ("accepted-b", "Scenario B", "unchanged"),
            ],
            ..accepted_x()
        };
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[accepted]),
            &specification_stating(&[Stated {
                change: "modified",
                scenarios: &[
                    ("scenario-x-a", "Scenario A", "unchanged"),
                    ("scenario-x-b", "Scenario B", "removed"),
                ],
                ..accepted_x()
            }]),
        );

        let state = merge_state(&project, &projection);

        assert!(state.reached.is_empty(), "{state:?}");
        assert_eq!(
            state.untaken_content,
            vec![untaken("req-x", 0, &[], &["accepted-b"])]
        );
    }

    /// The toolchain-packaging scenario "Statements in another order".
    #[test]
    fn statements_in_another_order_are_not_taken() {
        let scratch = Scratch::new("content-order");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated {
                statements: &["The system SHALL do x.", "That is the whole of it."],
                ..accepted_x()
            }]),
            &specification_stating(&[Stated {
                change: "modified",
                statements: &["That is the whole of it.", "The system SHALL do x."],
                ..accepted_x()
            }]),
        );

        let state = merge_state(&project, &projection);

        assert!(state.reached.is_empty(), "{state:?}");
        assert_eq!(state.untaken_content, vec![untaken("req-x", 1, &[], &[])]);
    }

    /// The toolchain-packaging scenario "An added requirement merged with other content".
    #[test]
    fn an_added_requirement_is_compared_as_a_modified_one_is() {
        let scratch = Scratch::new("content-added");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated {
                scenarios: &[("scenario-new-a", "Scenario A", "unchanged")],
                ..Stated::plain("req-new", "unchanged")
            }]),
            &specification_stating(&[Stated {
                scenarios: &[
                    ("scenario-new-a", "Scenario A", "added"),
                    ("scenario-new-b", "Scenario B", "added"),
                ],
                ..Stated::plain("req-new", "added")
            }]),
        );

        let state = merge_state(&project, &projection);

        assert!(state.reached.is_empty(), "{state:?}");
        assert_eq!(
            state.untaken_content,
            vec![untaken("req-new", 0, &["scenario-new-b"], &[])]
        );
    }

    /// The toolchain-packaging scenario "A removal the accepted specifications have taken".
    #[test]
    fn a_removal_the_merge_applied_is_reached() {
        let scratch = Scratch::new("removal-taken");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated::plain("req-other", "unchanged")]),
            &specification_stating(&[Stated::plain("req-y", "removed")]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.reached, vec![String::from("req-y")], "{state:?}");
        assert!(
            state.absent.is_empty(),
            "a correctly applied withdrawal was reported absent: {state:?}"
        );
        assert!(state.untaken_removals.is_empty(), "{state:?}");
        assert!(state.reached_in_full());
    }

    /// The toolchain-packaging scenario "A removal the accepted specifications have not taken".
    #[test]
    fn a_removal_the_merge_did_not_apply_is_an_untaken_removal() {
        let scratch = Scratch::new("removal-untaken");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated::plain("req-y", "unchanged")]),
            &specification_stating(&[Stated::plain("req-y", "removed")]),
        );

        let state = merge_state(&project, &projection);

        assert!(
            state.reached.is_empty(),
            "an unapplied withdrawal was reached: {state:?}"
        );
        assert!(state.absent.is_empty(), "{state:?}");
        assert_eq!(state.untaken_removals, vec![String::from("req-y")]);
    }

    /// The toolchain-packaging scenario "A rename and a modification, neither taken".
    #[test]
    fn a_rename_and_a_modification_neither_taken_are_both_reported() {
        let scratch = Scratch::new("rename-and-content");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated {
                heading: "The old name",
                ..accepted_x()
            }]),
            &specification_stating(&[Stated {
                change: "modified",
                heading: "The new name",
                renamed_from: Some("The old name"),
                statements: &["The system SHALL do x quickly."],
                ..accepted_x()
            }]),
        );

        let state = merge_state(&project, &projection);

        assert!(state.reached.is_empty(), "{state:?}");
        assert_eq!(
            state
                .untaken_renames
                .iter()
                .map(|rename| rename.id.as_str())
                .collect::<Vec<_>>(),
            vec!["req-x"],
            "{state:?}"
        );
        assert_eq!(state.untaken_content, vec![untaken("req-x", 1, &[], &[])]);
    }

    /// The toolchain-packaging scenario "A modification the accepted specifications have taken",
    /// as a guard against a comparison stricter than the requirement: scenarios in another order,
    /// a note the change does not state, and a scenario the change withdraws and the merge
    /// deleted are all a modification taken.
    #[test]
    fn scenario_order_and_notes_do_not_decide_reaching() {
        let scratch = Scratch::new("content-taken");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated {
                statements: &["The system SHALL do x quickly."],
                scenarios: &[
                    ("scenario-x-c", "Scenario C", "unchanged"),
                    ("scenario-x-a", "Scenario A", "unchanged"),
                ],
                notes: &["An accepted note the change never stated."],
                ..accepted_x()
            }]),
            &specification_stating(&[Stated {
                change: "modified",
                statements: &["The system SHALL do x quickly."],
                scenarios: &[
                    ("scenario-x-a", "Scenario A", "unchanged"),
                    ("scenario-x-b", "Scenario B", "removed"),
                    ("scenario-x-c", "Scenario C", "added"),
                ],
                ..accepted_x()
            }]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.reached, vec![String::from("req-x")], "{state:?}");
        assert!(state.untaken_content.is_empty(), "{state:?}");
        assert!(state.reached_in_full());
    }

    /// The toolchain-packaging scenario "A requirement carried forward is compared by its
    /// identifier": another change modified it since, and that is not this change's work.
    #[test]
    fn a_requirement_carried_forward_is_compared_by_its_identifier() {
        let scratch = Scratch::new("content-carried");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated {
                statements: &["The system SHALL do z, as a later change words it."],
                ..Stated::plain("req-z", "unchanged")
            }]),
            &specification_stating(&[Stated {
                statements: &["The system SHALL do z."],
                ..Stated::plain("req-z", "unchanged")
            }]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.reached, vec![String::from("req-z")], "{state:?}");
        assert!(state.untaken_content.is_empty(), "{state:?}");
    }

    /// The toolchain-packaging scenario "A scenario changed inside a requirement marked
    /// unchanged". The change states content inside it, so it is compared as a modified
    /// requirement is, and a scenario it adds that the merge never took is not reached.
    #[test]
    fn a_scenario_changed_inside_an_unchanged_requirement_is_compared() {
        let scratch = Scratch::new("content-scenario-in-unchanged");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[accepted_x()]),
            &specification_stating(&[Stated {
                scenarios: &[
                    ("scenario-x-a", "Scenario A", "unchanged"),
                    ("scenario-x-b", "Scenario B", "unchanged"),
                    ("scenario-x-c", "Scenario C", "added"),
                ],
                ..accepted_x()
            }]),
        );

        let state = merge_state(&project, &projection);

        assert!(
            state.reached.is_empty(),
            "a scenario added inside an unchanged requirement was reached unmerged: {state:?}"
        );
        assert_eq!(
            state.untaken_content,
            vec![untaken("req-x", 0, &["scenario-x-c"], &[])]
        );
    }

    /// A requirement the change modifies whose identifier is not held is absent, and only that:
    /// there is no accepted content to compare its content with.
    #[test]
    fn a_modified_requirement_whose_identifier_is_not_held_is_absent_only() {
        let scratch = Scratch::new("content-absent");
        let (project, _, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated::plain("req-other", "unchanged")]),
            &specification_stating(&[Stated {
                change: "modified",
                statements: &["The system SHALL do x quickly."],
                ..accepted_x()
            }]),
        );

        let state = merge_state(&project, &projection);

        assert_eq!(state.absent, vec![String::from("req-x")], "{state:?}");
        assert!(
            state.untaken_content.is_empty(),
            "one requirement was reported twice: {state:?}"
        );
        assert!(state.reached.is_empty(), "{state:?}");
    }

    /// The fixture design decision 4's partition is stated over: one requirement in each place a
    /// stated identifier can land, and one in both sets of the untaken pair.
    fn partition_fixture(scratch: &Scratch) -> (ProjectRoot, PathBuf, htmlspec_parse::Projection) {
        const WELL: &[&str] = &["The system SHALL do it well."];
        project_holding(
            scratch,
            &specification_stating(&[
                Stated::plain("req-here", "unchanged"),
                Stated {
                    heading: "The old name",
                    ..Stated::plain("req-renamed", "unchanged")
                },
                Stated {
                    heading: "Applied",
                    ..Stated::plain("req-applied", "unchanged")
                },
                Stated {
                    statements: WELL,
                    ..Stated::plain("req-taken", "unchanged")
                },
                Stated::plain("req-untaken", "unchanged"),
                Stated::plain("req-kept", "unchanged"),
                Stated {
                    statements: &["A later change's wording."],
                    ..Stated::plain("req-carried", "unchanged")
                },
                Stated {
                    heading: "Old both",
                    ..Stated::plain("req-both", "unchanged")
                },
            ]),
            &specification_stating(&[
                Stated::plain("req-here", "unchanged"),
                Stated {
                    heading: "The new name",
                    renamed_from: Some("The old name"),
                    ..Stated::plain("req-renamed", "modified")
                },
                Stated {
                    heading: "Applied",
                    renamed_from: Some("Before"),
                    ..Stated::plain("req-applied", "modified")
                },
                Stated::plain("req-gone", "added"),
                Stated {
                    statements: WELL,
                    ..Stated::plain("req-taken", "modified")
                },
                Stated {
                    statements: WELL,
                    ..Stated::plain("req-untaken", "modified")
                },
                Stated::plain("req-withdrawn", "removed"),
                Stated::plain("req-kept", "removed"),
                Stated {
                    statements: &["This change's wording."],
                    ..Stated::plain("req-carried", "unchanged")
                },
                Stated {
                    heading: "New both",
                    renamed_from: Some("Old both"),
                    statements: WELL,
                    ..Stated::plain("req-both", "modified")
                },
            ]),
        )
    }

    /// Every stated identifier is in exactly one of four places, as design decision 4 states:
    /// `reached`, `absent`, the untaken removals, or the untaken pair. Within the pair it is in
    /// the renames, the content, or both.
    #[test]
    fn every_stated_requirement_is_in_exactly_one_set() {
        let scratch = Scratch::new("partition");
        let (project, _, projection) = partition_fixture(&scratch);

        let state = merge_state(&project, &projection);

        let renamed: std::collections::BTreeSet<&str> = state
            .untaken_renames
            .iter()
            .map(|rename| rename.id.as_str())
            .collect();
        let differing: std::collections::BTreeSet<&str> = state
            .untaken_content
            .iter()
            .map(|content| content.id.as_str())
            .collect();
        let pair: std::collections::BTreeSet<&str> = renamed.union(&differing).copied().collect();
        let mut placed: Vec<&str> = state
            .reached
            .iter()
            .chain(&state.absent)
            .chain(&state.untaken_removals)
            .map(String::as_str)
            .chain(pair.iter().copied())
            .collect();
        placed.sort_unstable();
        let mut stated: Vec<&str> = state.stated.iter().map(String::as_str).collect();
        stated.sort_unstable();
        assert_eq!(placed, stated, "{state:?}");

        let mut reached: Vec<&str> = state.reached.iter().map(String::as_str).collect();
        reached.sort_unstable();
        assert_eq!(
            reached,
            vec![
                "req-applied",
                "req-carried",
                "req-here",
                "req-taken",
                "req-withdrawn"
            ],
            "{state:?}"
        );
        assert_eq!(state.absent, vec![String::from("req-gone")], "{state:?}");
        assert_eq!(
            state.untaken_removals,
            vec![String::from("req-kept")],
            "{state:?}"
        );
        assert_eq!(
            renamed.into_iter().collect::<Vec<_>>(),
            vec!["req-both", "req-renamed"]
        );
        assert_eq!(
            differing.into_iter().collect::<Vec<_>>(),
            vec!["req-both", "req-untaken"]
        );
    }

    /// Task 2.3: `The operation and archival agree about what did not arrive`, over every set of
    /// work not done. Archival's findings are built from `merge_state`, and this is what fails if
    /// they stop being.
    #[test]
    fn archival_and_sync_name_the_same_untaken_work() {
        let scratch = Scratch::new("untaken-agree");
        let (project, directory, projection) = partition_fixture(&scratch);

        let from_sync = merge_state(&project, &projection);
        let from_archival = findings_for(&project, &directory, &projection);

        assert_eq!(
            from_sync.absent, from_archival.merge.absent,
            "the two operations disagree about what is unmerged"
        );
        assert_eq!(
            from_sync.untaken_renames, from_archival.merge.untaken_renames,
            "the two operations disagree about which renames are untaken"
        );
        assert_eq!(
            from_sync.untaken_content, from_archival.merge.untaken_content,
            "the two operations disagree about which content is untaken"
        );
        assert_eq!(
            from_sync.untaken_removals, from_archival.merge.untaken_removals,
            "the two operations disagree about which removals are untaken"
        );
        for (set, count) in [
            ("absent", from_sync.absent.len()),
            ("untaken renames", from_sync.untaken_renames.len()),
            ("untaken content", from_sync.untaken_content.len()),
            ("untaken removals", from_sync.untaken_removals.len()),
        ] {
            assert!(
                count > 0,
                "the fixture puts nothing in {set}, so agreement there is empty"
            );
        }
        assert!(from_archival.needs_sync());
    }

    /// Stores `bytes` under `name` at its address, in the store whose parent is `holder`.
    fn store_in(holder: &Path, name: &str, bytes: &[u8]) {
        let path = holder.join("assets").join(htmlspec_assets::stored_path(
            &htmlspec_assets::sha256_hex(bytes),
            name,
        ));
        std::fs::create_dir_all(path.parent().expect("an address directory")).expect("created");
        std::fs::write(&path, bytes).expect("the stored asset is written");
    }

    /// Task 2.2: `assets` and the promotion check reach the same verdict on the same tree.
    ///
    /// The two made the comparison separately, and each could be changed without the other. The
    /// tree carries each case the partition decides: a store inside the project, a second copy
    /// inside it beside a change's documents, and a fixture's store outside it holding the same
    /// bytes. Asserted with and without the inside copy, so agreement on a clean tree and on a
    /// faulty one are both pinned.
    #[test]
    fn the_asset_check_and_promotion_give_one_verdict() {
        for with_a_second_store_inside in [true, false] {
            let scratch = Scratch::new(if with_a_second_store_inside {
                "topology-agree-faulty"
            } else {
                "topology-agree-clean"
            });
            let project = ProjectRoot::establish(&scratch.0).expect("the project is established");
            let bytes = b"one runtime file";
            store_in(&project.htmlspec(), "runtime.js", bytes);
            store_in(&scratch.0.join("fixtures/a-fixture"), "runtime.js", bytes);
            if with_a_second_store_inside {
                store_in(&project.change("a-change"), "runtime.js", bytes);
            }

            let (outcome, out, _) = run(Form::Structured, |streams| {
                assets(Some(project.path()), streams)
            });
            let payload: serde_json::Value = serde_json::from_str(&out).expect("it parses");
            let mut from_assets: Vec<String> = payload["diagnostics"]
                .as_array()
                .expect("a diagnostics list")
                .iter()
                .map(|d| d["message"].as_str().expect("a message").to_owned())
                .collect();

            let (_, promotion) = {
                let mut streams = Streams::new(Form::Human, Vec::new(), Vec::new());
                conformance(project.path(), &mut streams)
            };
            let mut from_promotion: Vec<String> = promotion
                .iter()
                .filter_map(|line| line.strip_prefix("assets: ").map(str::to_owned))
                .collect();

            from_assets.sort();
            from_promotion.sort();
            assert_eq!(
                from_assets, from_promotion,
                "the two checks disagree about one tree"
            );
            assert_eq!(
                outcome == Outcome::Success,
                !with_a_second_store_inside,
                "the verdict is not the one the partition decides: {payload}"
            );
        }
    }

    /// The comparison separates what reached the contract from what did not.
    #[test]
    fn the_comparison_partitions_what_the_change_states() {
        let scratch = Scratch::new("merge-state");
        let (project, directory) =
            project_stating(&scratch, &["req-here", "req-gone"], &["req-here"]);
        let source = std::fs::read_to_string(directory.join(CHANGE_SPECIFICATION))
            .expect("the specification reads");
        let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);

        let state = merge_state(&project, &projection);

        assert_eq!(
            state.stated.len(),
            2,
            "both identifiers are stated: {state:?}"
        );
        assert_eq!(state.reached, vec![String::from("req-here")]);
        assert_eq!(state.absent, vec![String::from("req-gone")]);
    }

    /// Archival and `sync` report the same identifiers, because they are one function.
    ///
    /// This is the accepted scenario `The operation and archival agree`, checked rather than
    /// asserted. Two counts of one thing drift, and the drift would be invisible to a test
    /// written against either operation alone — the same shape as the version gate that
    /// validated its own invented convention across eleven archives without refusing anything.
    #[test]
    fn archival_and_sync_name_the_same_absent_requirements() {
        let scratch = Scratch::new("merge-agree");
        // A withdrawal the merge applied is beside the requirement nobody merged, so parity also
        // shows it absent from neither operation's report.
        let (project, directory, projection) = project_holding(
            &scratch,
            &specification_stating(&[Stated::plain("req-here", "unchanged")]),
            &specification_stating(&[
                Stated::plain("req-here", "unchanged"),
                Stated::plain("req-gone", "added"),
                Stated::plain("req-withdrawn", "removed"),
            ]),
        );

        let from_sync = merge_state(&project, &projection).absent;
        let from_archival = findings_for(&project, &directory, &projection).merge.absent;

        assert_eq!(
            from_sync, from_archival,
            "the two operations disagree about what is unmerged"
        );
        assert_eq!(from_sync, vec![String::from("req-gone")]);
    }

    /// A change stating nothing compares to nothing, and does not read the project to say so.
    #[test]
    fn a_change_stating_no_identifiers_has_nothing_absent() {
        let scratch = Scratch::new("merge-empty");
        let (project, _) = project_stating(&scratch, &[], &["req-here"]);
        let projection = htmlspec_parse::project(
            &htmlspec_parse::parse_document(&valid_document().replace(r#" id="req-example""#, ""))
                .document,
        );

        let state = merge_state(&project, &projection);

        assert!(state.stated.is_empty(), "nothing is stated: {state:?}");
        assert!(state.absent.is_empty());
        assert!(state.reached.is_empty());
    }

    /// `sync` never writes an accepted specification, whatever it finds.
    ///
    /// The prohibition is stated in the requirement because the name is inherited from OpenSpec
    /// and reads like an operation that writes. A requirement stated and not checked is a
    /// requirement that holds until someone changes the code.
    #[test]
    fn sync_does_not_write_the_accepted_specifications() {
        let scratch = Scratch::new("sync-readonly");
        let (project, _) = project_stating(&scratch, &["req-here", "req-gone"], &["req-here"]);

        let before = accepted_bytes(&project);
        let (outcome, _, _) = run(Form::Human, |streams| {
            sync("a-change", Some("__scratch__"), streams)
        });
        let after = accepted_bytes(&project);

        // The store id does not resolve, so this exercises the refusal path rather than the
        // comparison. Both must leave the contract alone, which is what is asserted.
        assert_eq!(outcome, Outcome::OperationalFailure);
        assert_eq!(before, after, "an accepted specification changed");
    }

    /// Every accepted specification's bytes, for comparing a directory against itself.
    fn accepted_bytes(project: &ProjectRoot) -> Vec<(PathBuf, Vec<u8>)> {
        let mut found = Vec::new();
        if let Ok(entries) = std::fs::read_dir(project.specs()) {
            for entry in entries.flatten() {
                if let Ok(bytes) = std::fs::read(entry.path()) {
                    found.push((entry.path(), bytes));
                }
            }
        }
        found.sort();
        found
    }

    /// A document with a dangling reference, which is a document violation.
    ///
    /// One defect per document, and it is a reference rather than a missing kind:
    /// the tests below count `refs/unresolved` diagnostics, so the fault has to be
    /// the one they name.
    fn invalid_document() -> String {
        valid_document().replace(r#"data-refs="req-example""#, r#"data-refs="req-absent""#)
    }

    #[test]
    fn a_clean_document_validates_and_exits_zero() {
        let scratch = Scratch::new("validate-clean");
        let path = scratch.write("doc.html", &valid_document());
        let (outcome, _, _) = run(Form::Human, |s| {
            validate(Some(&path), Strictness::Default, s)
        });
        assert_eq!(outcome, Outcome::Success);
    }

    #[test]
    fn an_invalid_document_exits_with_the_document_violation_code() {
        let scratch = Scratch::new("validate-invalid");
        let path = scratch.write("doc.html", &invalid_document());
        let (outcome, out, _) = run(Form::Human, |s| {
            validate(Some(&path), Strictness::Default, s)
        });
        assert_eq!(outcome, Outcome::DocumentViolation);
        assert!(out.contains("refs/unresolved"), "{out}");
        assert!(out.contains("req-absent"), "{out}");
    }

    /// A missing path is an operational failure, distinct from a document violation.
    #[test]
    fn a_missing_path_is_an_operational_failure() {
        let (outcome, out, err) = run(Form::Human, |s| {
            validate(
                Some(Path::new("/nonexistent/x.html")),
                Strictness::Default,
                s,
            )
        });
        assert_eq!(outcome, Outcome::OperationalFailure);
        assert!(out.is_empty(), "a failure reached stdout: {out}");
        assert!(err.contains("no such file or directory"), "{err}");
    }

    /// Whole-project validation reports every document rather than stopping at the
    /// first failure.
    #[test]
    fn a_project_run_reports_every_document() {
        let scratch = Scratch::new("validate-project");
        scratch.write("a.html", &invalid_document());
        scratch.write("b.html", &valid_document());
        scratch.write("c.html", &invalid_document());

        let (outcome, out, _) = run(Form::Human, |s| {
            validate(Some(&scratch.0), Strictness::Default, s)
        });
        assert_eq!(outcome, Outcome::DocumentViolation);
        for name in ["a.html", "b.html", "c.html"] {
            assert!(
                out.contains(name),
                "{name} is missing from the report:\n{out}"
            );
        }
        // The specification template's evidence and verification each reference the
        // requirement, so each invalid document reports two unresolved references.
        // Two rather than three: the task that made the third left with the artifact
        // kinds, because a specification must not carry one.
        assert_eq!(out.matches("refs/unresolved").count(), 4, "{out}");
        // And the run-level summary totals them rather than reporting one document.
        assert!(out.contains("3 document(s): 4 error(s)"), "{out}");
    }

    /// The structured payload parses cleanly and progress is present, which is the
    /// scenario the specs name.
    #[test]
    fn the_structured_payload_parses_and_progress_is_on_standard_error() {
        let scratch = Scratch::new("validate-structured");
        scratch.write("a.html", &invalid_document());
        scratch.write("b.html", &valid_document());

        let (outcome, out, err) = run(Form::Structured, |s| {
            validate(Some(&scratch.0), Strictness::Default, s)
        });
        assert_eq!(outcome, Outcome::DocumentViolation);

        let parsed: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
        assert_eq!(parsed["accepted"], serde_json::Value::Bool(false));
        assert_eq!(parsed["documents"], serde_json::json!(2));
        assert_eq!(
            parsed["results"]
                .as_array()
                .expect("results is an array")
                .len(),
            2
        );
        // The stable rule identifier reaches the payload.
        assert!(out.contains("refs/unresolved"), "{out}");

        assert!(err.contains("validating"), "progress is missing: {err}");
    }

    /// Check mode names the document, modifies nothing, and exits nonzero.
    #[test]
    fn check_mode_reports_without_writing() {
        let scratch = Scratch::new("format-check");
        let unformatted = "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body><p>Body.</p></body></html>\n";
        let path = scratch.write("doc.html", unformatted);

        let (outcome, out, _) = run(Form::Human, |s| {
            format(
                Some(&path),
                FormatMode::Check,
                FallbackRequest::Leave,
                AssetRequest::Keep,
                s,
            )
        });
        assert_eq!(outcome, Outcome::DocumentViolation);
        assert!(out.contains("not canonically formatted"), "{out}");
        assert!(out.contains("doc.html"), "{out}");
        assert_eq!(
            std::fs::read_to_string(&path).expect("readable"),
            unformatted,
            "check mode modified the file"
        );
    }

    #[test]
    fn check_mode_on_a_canonical_document_exits_zero() {
        let scratch = Scratch::new("format-check-clean");
        let path = scratch.write("doc.html", &valid_document());
        let (outcome, _, _) = run(Form::Human, |s| {
            format(
                Some(&path),
                FormatMode::Check,
                FallbackRequest::Leave,
                AssetRequest::Keep,
                s,
            )
        });
        assert_eq!(outcome, Outcome::Success);
    }

    /// Write mode rewrites what is not canonical and leaves what is byte-identical.
    #[test]
    fn write_mode_rewrites_only_what_is_not_canonical() {
        let scratch = Scratch::new("format-write");
        let messy = scratch.write(
            "messy.html",
            "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body><p>Body.</p></body></html>\n",
        );
        let clean_before = valid_document();
        let clean = scratch.write("clean.html", &clean_before);

        let (outcome, _, _) = run(Form::Human, |s| {
            format(
                Some(&scratch.0),
                FormatMode::Write,
                FallbackRequest::Leave,
                AssetRequest::Keep,
                s,
            )
        });
        assert_eq!(outcome, Outcome::Success);

        let rewritten = std::fs::read_to_string(&messy).expect("readable");
        assert!(
            rewritten.starts_with("<!DOCTYPE html>\n<html lang=\"en\">\n  <head>"),
            "{rewritten}"
        );
        // And the result is canonical, so a second run changes nothing.
        assert!(
            htmlspec_format::check(&rewritten)
                .expect("formats")
                .is_canonical
        );
        assert_eq!(
            std::fs::read_to_string(&clean).expect("readable"),
            clean_before,
            "an already-canonical document was rewritten"
        );
    }

    /// An ambiguous document is refused with a reason and never silently repaired.
    #[test]
    fn an_ambiguous_document_is_refused_rather_than_repaired() {
        let scratch = Scratch::new("format-ambiguous");
        let ambiguous = "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
             <body><p><strong>text</p></strong></body></html>\n";
        let path = scratch.write("doc.html", ambiguous);

        let (outcome, _, err) = run(Form::Human, |s| {
            format(
                Some(&path),
                FormatMode::Write,
                FallbackRequest::Leave,
                AssetRequest::Keep,
                s,
            )
        });
        assert_eq!(outcome, Outcome::OperationalFailure);
        assert!(err.contains("refusing to format"), "{err}");
        assert_eq!(
            std::fs::read_to_string(&path).expect("readable"),
            ambiguous,
            "an ambiguous document was rewritten"
        );
    }

    /// Task 8.5: every kind `init` can produce validates immediately.
    ///
    /// The promise held for one template before the artifact kinds and has to hold for
    /// three. A kind whose output did not validate would make `init`'s promise false
    /// for that kind alone, which is worse than failing for all of them — a
    /// contributor would meet it only when they reached that stage.
    ///
    /// Driven through `init` itself rather than through `render_kind`, so it covers
    /// what a contributor invokes: the assets `init` establishes, the relative
    /// reference it computes, and the store the document is checked against.
    #[test]
    fn every_kind_init_can_produce_validates_immediately() {
        for template in htmlspec_assets::TEMPLATES {
            let scratch = Scratch::new(&format!("init-kind-{}", template.kind));
            let path = scratch.0.join(format!("{}.html", template.kind));

            let (outcome, out, err) = run(Form::Human, |s| {
                write_document(&path, "A subject", template.kind, None, s)
            });
            assert_eq!(
                outcome,
                Outcome::Success,
                "init failed for the {} kind:\n{out}{err}",
                template.kind
            );
            assert!(
                out.contains(&format!("created a {}", template.kind)),
                "{out}"
            );

            let source = std::fs::read_to_string(&path).expect("the document was written");
            // The document declares the kind that was asked for, which is what makes
            // the shape and the declaration agree.
            let projected =
                htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);
            assert_eq!(
                projected.model.kind.as_deref(),
                Some(template.kind),
                "init wrote a document declaring a different kind than requested"
            );
            // And it validates against the store init just established, under strict
            // mode as well: a project running CI with warnings fatal can adopt this
            // output unchanged.
            for strictness in [Strictness::Default, Strictness::Strict] {
                let report = validate_source(&source, &path, &mut Reach::default(), strictness);
                assert!(
                    report.is_accepted(),
                    "the {} kind does not validate under {strictness:?}:\n{}",
                    template.kind,
                    htmlspec_validate::render_human(&report)
                );
            }
        }
    }

    /// A kind outside the vocabulary is refused before anything is written.
    ///
    /// Before anything is written matters: a refusal that had already established the
    /// shared assets would leave a directory a contributor did not ask for, from a
    /// command that failed.
    #[test]
    fn init_refuses_a_kind_outside_the_vocabulary() {
        let scratch = Scratch::new("init-unknown-kind");
        let path = scratch.0.join("doc.html");

        let (outcome, _, err) = run(Form::Human, |s| {
            write_document(&path, "A document", "rumination", None, s)
        });
        assert_eq!(outcome, Outcome::OperationalFailure);
        assert!(err.contains("rumination is not an artifact kind"), "{err}");
        // The message lists what is available, so the next attempt succeeds.
        for kind in htmlspec_assets::TEMPLATES.iter().map(|t| t.kind) {
            assert!(
                err.contains(kind),
                "the failure does not name {kind}: {err}"
            );
        }
        assert!(!path.exists(), "init wrote a document it refused to create");
        assert!(
            !scratch.0.join("assets").exists(),
            "init established assets for a document it refused to create"
        );
    }

    /// `new change` creates a change directory whose documents each declare their kind
    /// and each validate immediately.
    #[test]
    fn new_change_creates_a_valid_change_directory() {
        let scratch = Scratch::new("new-change");
        let project = ProjectRoot::establish(&scratch.0).expect("the project is established");
        let directory = project.change("add-a-dark-theme");

        let (outcome, out, err) = run(Form::Human, |s| {
            new_change_in(&project, "add-a-dark-theme", Some("Add a dark theme"), s)
        });
        assert_eq!(outcome, Outcome::Success, "{out}{err}");

        for (name, kind) in CHANGE_DOCUMENTS {
            let path = directory.join(name);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{name} was not written: {e}"));
            let projected =
                htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);
            assert_eq!(
                projected.model.kind.as_deref(),
                Some(*kind),
                "{name} does not declare the {kind} kind"
            );
            // The page shows it too, so a reviewer opening the file sees which
            // artifact it is without reading source.
            assert!(
                source.contains(&format!(r#"<span class="hs-kind">{kind}</span>"#)),
                "{name} does not show its kind on the page"
            );
            let report =
                validate_source(&source, &path, &mut Reach::default(), Strictness::Default);
            assert!(
                report.is_accepted(),
                "{name} does not validate:\n{}",
                htmlspec_validate::render_human(&report)
            );
        }

        // No specification. It states the contract the change established, and at
        // creation nothing has been established — it is produced at archival.
        assert!(
            !directory.join("specification.html").exists(),
            "new change wrote a specification, which is produced at archival from \
             the completed change rather than offered as a form to fill in"
        );
        assert!(out.contains("produced at archival"), "{out}");

        // The store is the *project's*, and the change has none of its own. This is the
        // inversion the change exists for: a store inside the change directory is the
        // arrangement that cost 6 MB per change and hid itself from the asset check.
        assert!(
            !directory.join(htmlspec_assets::store::DIRECTORY).exists(),
            "the change established its own store instead of referencing the project's"
        );
        let store = Store::discover(&project.htmlspec());
        assert!(store.is_present(), "the project has no asset store");
        assert!(
            store.duplicates().is_empty(),
            "the project holds duplicated asset bytes: {:#?}",
            store.duplicates()
        );

        // And the documents reach it, which is what makes the shared store usable: an
        // ascending relative reference, resolved against the project's store.
        let source = std::fs::read_to_string(directory.join("proposal.html")).expect("written");
        assert!(
            source.contains(r#"href="../../assets/"#),
            "the proposal does not reference the project's store relatively"
        );
    }

    /// An existing change directory is refused whole rather than written into.
    #[test]
    fn new_change_refuses_an_existing_directory() {
        let scratch = Scratch::new("new-change-existing");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        std::fs::create_dir_all(project.change("already-here")).expect("the directory is created");

        let (outcome, _, err) = run(Form::Human, |s| {
            new_change_in(&project, "already-here", None, s)
        });
        assert_eq!(outcome, Outcome::OperationalFailure);
        assert!(err.contains("already exists"), "{err}");
    }

    /// `init` creates a document that validates, and establishes the assets.
    #[test]
    fn init_creates_a_validating_document_and_establishes_the_assets() {
        let scratch = Scratch::new("init-fresh");
        let path = scratch.0.join("proposal.html");

        let (outcome, out, _) = run(Form::Human, |s| {
            write_document(
                &path,
                "A new proposal",
                htmlspec_assets::DEFAULT_KIND,
                None,
                s,
            )
        });
        assert_eq!(outcome, Outcome::Success, "{out}");
        assert!(out.contains("created"), "{out}");

        // The document validates, which init already checked, asserted here so a
        // change to init's own check cannot quietly remove the guarantee.
        let source = std::fs::read_to_string(&path).expect("the document was written");
        assert!(
            validate_source(&source, &path, &mut Reach::default(), Strictness::Default)
                .is_accepted()
        );

        // Every shared asset is present, so the document renders offline with no
        // further step.
        for asset in htmlspec_assets::all_assets() {
            let established = scratch.0.join("assets").join(asset.path);
            assert!(
                established.exists(),
                "init did not establish {}",
                asset.path
            );
            assert_eq!(
                std::fs::read(&established).expect("readable"),
                asset.bytes,
                "{} was written with different bytes",
                asset.path
            );
        }
    }

    /// An existing asset is left alone: a project may have edited its stylesheet.
    #[test]
    fn init_does_not_overwrite_an_existing_asset() {
        let scratch = Scratch::new("init-existing-asset");
        let edited = "/* a project's own edit */\n";
        scratch.write("assets/htmlspec.css", edited);

        let path = scratch.0.join("doc.html");
        let (outcome, _, _) = run(Form::Human, |s| {
            write_document(&path, "Subject", htmlspec_assets::DEFAULT_KIND, None, s)
        });
        assert_eq!(outcome, Outcome::Success);
        assert_eq!(
            std::fs::read_to_string(scratch.0.join("assets/htmlspec.css")).expect("readable"),
            edited,
            "init overwrote a project's edited stylesheet"
        );
    }

    /// Task 6.4: `init` cannot emit a reference whose hash disagrees with the bytes it
    /// establishes.
    ///
    /// The defect class this whole change exists to eliminate, asserted at the point
    /// where a document is born rather than inferred from the template's correctness.
    /// Every reference is resolved against the store `init` just wrote, through the same
    /// public API the validator uses — so a reference that named the wrong bytes would
    /// fail here even if the document still validated for some other reason.
    #[test]
    fn init_cannot_emit_a_reference_that_disagrees_with_the_stored_bytes() {
        let scratch = Scratch::new("init-provenance");
        let path = scratch.0.join("proposal.html");
        let (outcome, _, _) = run(Form::Human, |s| {
            write_document(&path, "A subject", htmlspec_assets::DEFAULT_KIND, None, s)
        });
        assert_eq!(outcome, Outcome::Success);

        let source = std::fs::read_to_string(&path).expect("the document was written");
        let store = Store::discover(&scratch.0);
        assert!(store.is_present(), "init did not establish a store");

        // Every asset reference the document carries, resolved against those bytes.
        let mut checked = 0usize;
        for reference in asset_references(&source) {
            checked += 1;
            assert_eq!(
                store.resolve(&reference),
                htmlspec_assets::Resolution::Match,
                "init emitted {reference}, which does not name the bytes it stored"
            );
        }
        assert!(
            checked >= 3,
            "expected the two stylesheets and the script, saw {checked} reference(s)"
        );
    }

    /// A destination spelled through `..` is placed where it lands, not where it passes: a document
    /// at `htmlspec/../docs/` is in the project's root, outside `htmlspec/`, so it is given a store
    /// of its own rather than the project's.
    #[test]
    fn a_destination_spelled_through_a_parent_is_placed_where_it_lands() {
        let scratch = Scratch::new("store-root-parent");
        std::fs::create_dir_all(scratch.0.join("htmlspec/changes")).expect("a project");
        let beneath = scratch.0.join("htmlspec/changes/one/proposal.html");
        let left = scratch.0.join("htmlspec/../docs/proposal.html");
        assert_eq!(
            project_store_root(&beneath),
            Some(normalize(&scratch.0.join("htmlspec")))
        );
        assert_eq!(project_store_root(&left), None);
    }

    /// A destination that names the project's directory another way, here through a link at the
    /// level below the project's root, uses the project's store, in the destination's spelling.
    /// The code review of pull request #56 found the comparison made by spelling, which a
    /// filesystem that ignores case defeats for `HTMLSPEC/` exactly as a link does.
    #[cfg(unix)]
    #[test]
    fn another_name_for_the_projects_directory_uses_its_store() {
        let scratch = Scratch::new("store-root-alias");
        std::fs::create_dir_all(scratch.0.join("htmlspec/changes")).expect("a project");
        std::os::unix::fs::symlink(scratch.0.join("htmlspec"), scratch.0.join("alias"))
            .expect("the link is made");
        let through = scratch.0.join("alias/changes/one/proposal.html");
        assert_eq!(
            project_store_root(&through),
            Some(normalize(&scratch.0.join("alias")))
        );
        // A directory beside the project's own, of another name, is not it.
        std::fs::create_dir_all(scratch.0.join("docs")).expect("another directory");
        assert_eq!(project_store_root(&scratch.0.join("docs/notes.html")), None);
    }

    /// On a filesystem that ignores case, the review's own case: `HTMLSPEC/` is the project's
    /// directory. Where the filesystem respects case there is no such directory, and nothing to test.
    #[test]
    fn the_projects_directory_in_another_case_uses_its_store_where_case_is_ignored() {
        let scratch = Scratch::new("store-root-case");
        std::fs::create_dir_all(scratch.0.join("htmlspec/changes")).expect("a project");
        if !scratch.0.join("HTMLSPEC").is_dir() {
            return;
        }
        let shouted = scratch.0.join("HTMLSPEC/changes/one/proposal.html");
        assert_eq!(
            project_store_root(&shouted),
            Some(normalize(&scratch.0.join("HTMLSPEC")))
        );
    }

    /// `normalize` resolves `.` and `..` without touching the filesystem, and a `..` at the root
    /// stays there.
    #[test]
    fn normalize_resolves_dot_components_lexically() {
        assert_eq!(normalize(Path::new("/a/./b/../c")), PathBuf::from("/a/c"));
        assert_eq!(normalize(Path::new("/../a")), PathBuf::from("/a"));
    }

    /// Task 6.2: an already-stored asset is reused rather than copied again.
    ///
    /// Checked by counting the files under the store, because "reuse" is only observable
    /// as the absence of a second copy. A second `init` into the same project must add
    /// documents and no asset bytes.
    #[test]
    fn init_reuses_an_already_stored_asset_rather_than_writing_a_second_copy() {
        let scratch = Scratch::new("init-reuse");
        let first = scratch.0.join("one.html");
        let (outcome, _, _) = run(Form::Human, |s| {
            write_document(&first, "One", htmlspec_assets::DEFAULT_KIND, None, s)
        });
        assert_eq!(outcome, Outcome::Success);

        let after_first = stored_files(&scratch.0.join("assets"));
        assert!(!after_first.is_empty(), "no assets were established");

        // A second document in the same project.
        let second = scratch.0.join("two.html");
        let (outcome, _, _) = run(Form::Human, |s| {
            write_document(&second, "Two", htmlspec_assets::DEFAULT_KIND, None, s)
        });
        assert_eq!(outcome, Outcome::Success);

        assert_eq!(
            stored_files(&scratch.0.join("assets")),
            after_first,
            "a second init wrote another copy of the runtime instead of sharing it"
        );
        // And the project reports no duplication, which is the property that matters.
        let store = Store::discover(&scratch.0);
        assert!(
            store.duplicates().is_empty(),
            "init produced duplicated asset bytes: {:#?}",
            store.duplicates()
        );
    }

    /// Task 6.3: a document initialized into a project with no assets renders offline
    /// with no further step.
    ///
    /// "Renders offline" is asserted as the property that makes it true: every reference
    /// the document carries resolves to a file that exists on disk, relative to the
    /// document itself.
    #[test]
    fn a_document_initialized_into_an_empty_project_renders_offline() {
        let scratch = Scratch::new("init-offline");
        let path = scratch.0.join("proposal.html");
        let (outcome, _, _) = run(Form::Human, |s| {
            write_document(&path, "Subject", htmlspec_assets::DEFAULT_KIND, None, s)
        });
        assert_eq!(outcome, Outcome::Success);

        let source = std::fs::read_to_string(&path).expect("written");
        let directory = path.parent().expect("a parent");
        let mut resolved = 0usize;
        for reference in asset_references(&source) {
            let on_disk = directory.join(&reference);
            assert!(
                on_disk.exists(),
                "{reference} does not resolve on disk, so the document does not render \
                 offline"
            );
            resolved += 1;
        }
        assert!(resolved >= 3, "only {resolved} reference(s) were checked");
    }

    /// Every `assets/`-relative reference in a document, in document order.
    fn asset_references(source: &str) -> Vec<String> {
        let mut found = Vec::new();
        for attribute in ["href=\"", "src=\""] {
            let mut rest = source;
            while let Some(at) = rest.find(attribute) {
                rest = &rest[at + attribute.len()..];
                let Some(end) = rest.find('"') else { break };
                let reference = &rest[..end];
                if reference.contains("assets/") {
                    found.push(reference.to_owned());
                }
            }
        }
        found
    }

    /// Every file beneath `directory`, as sorted paths relative to it.
    fn stored_files(directory: &Path) -> Vec<PathBuf> {
        fn walk(at: &Path, root: &Path, into: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(at) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, root, into);
                } else {
                    into.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
                }
            }
        }
        let mut found = Vec::new();
        walk(directory, directory, &mut found);
        found.sort();
        found
    }

    /// `init` refuses to overwrite a document.
    #[test]
    fn init_refuses_to_overwrite_an_existing_document() {
        let scratch = Scratch::new("init-existing-doc");
        let original = "original content";
        let path = scratch.write("doc.html", original);

        let (outcome, _, err) = run(Form::Human, |s| {
            write_document(&path, "Subject", htmlspec_assets::DEFAULT_KIND, None, s)
        });
        assert_eq!(outcome, Outcome::OperationalFailure);
        assert!(err.contains("already exists"), "{err}");
        assert_eq!(std::fs::read_to_string(&path).expect("readable"), original);
    }

    /// A document in a subdirectory references the assets by a relative path that
    /// resolves, which is what keeps it offline.
    #[test]
    fn a_nested_document_references_the_assets_relatively() {
        let scratch = Scratch::new("init-nested");
        let path = scratch.0.join("changes/proposal.html");

        let (outcome, _, _) = run(Form::Human, |s| {
            write_document(
                &path,
                "Nested",
                htmlspec_assets::DEFAULT_KIND,
                Some(&scratch.0),
                s,
            )
        });
        assert_eq!(outcome, Outcome::Success);

        // The stylesheet is referenced at its content address, so the expected path is
        // derived rather than spelled out: a literal hash here would go stale the first
        // time the stylesheet changed, and a test asserting a stale path is worse than
        // no test.
        let stylesheet = htmlspec_assets::OWN
            .iter()
            .find(|asset| asset.name() == "htmlspec.css")
            .expect("the stylesheet is embedded");
        let reference = format!("../assets/{}", stylesheet.path);

        let source = std::fs::read_to_string(&path).expect("written");
        assert!(
            source.contains(&format!(r#"href="{reference}""#)),
            "the nested document does not reference the assets relatively:\n{source}"
        );
        // And the reference resolves on disk, which is the property that keeps the
        // document offline.
        assert!(
            path.parent()
                .expect("has a parent")
                .join(&reference)
                .exists(),
            "the relative reference does not resolve"
        );
    }

    #[test]
    fn a_relative_reference_is_computed_for_both_directions() {
        let scratch = Scratch::new("relative");
        let root = scratch.0.clone();
        assert_eq!(
            relative_reference(&root, &root.join("assets")).as_deref(),
            Some("assets")
        );
        assert_eq!(
            relative_reference(&root.join("changes"), &root.join("assets")).as_deref(),
            Some("../assets")
        );
        assert_eq!(
            relative_reference(&root.join("a/b/c"), &root.join("assets")).as_deref(),
            Some("../../../assets")
        );
    }

    /// A document deep in a project finds the store above it.
    ///
    /// The walk is upward, so a document several directories down is still inside its
    /// project — which is what makes the check independent of how the tool was invoked.
    #[test]
    fn a_nested_document_finds_the_store_above_it() {
        let scratch = Scratch::new("store-upward");
        std::fs::create_dir_all(scratch.0.join("assets/sha256-0123456789abcdef"))
            .expect("a store at the project root");
        let deep = scratch.write("changes/nested/doc.html", "<!doctype html>\n");

        assert!(
            store_for(&deep).is_present(),
            "a document below the project root did not find the store above it"
        );
    }

    /// A document with no store anywhere above it has none, which is what scopes the
    /// asset rules out for a conformance fixture.
    #[test]
    fn a_document_with_no_store_above_it_has_none() {
        let scratch = Scratch::new("store-absent");
        let path = scratch.write("doc.html", "<!doctype html>\n");
        // The scratch directory is under the system temporary directory, which has no
        // `assets/` at any level, so the walk reaches the root and finds nothing.
        assert!(
            !store_for(&path).is_present(),
            "a store was found where none exists, which would bring the asset rules into \
             scope for a document that claims no provenance"
        );
    }

    /// The nearest store wins: a document beside its own `assets/` resolves against
    /// that one rather than a project's.
    #[test]
    fn the_nearest_store_is_the_one_found() {
        let scratch = Scratch::new("store-nearest");
        std::fs::create_dir_all(scratch.0.join("assets/sha256-aaaaaaaaaaaaaaaa"))
            .expect("the outer store");
        std::fs::create_dir_all(scratch.0.join("inner/assets/sha256-bbbbbbbbbbbbbbbb"))
            .expect("the inner store");
        let inner = scratch.write("inner/doc.html", "<!doctype html>\n");

        let store = store_for(&inner);
        assert!(store.is_present());
        assert_eq!(
            store.len(),
            0,
            "both scratch stores hold empty address directories, so this only checks the \
             walk stopped at one of them"
        );
        // The distinguishing check: the inner store's root is the inner directory.
        match store {
            Store::Present(contents) => assert!(
                contents.root.ends_with("inner/assets"),
                "the walk passed the nearest store and took an outer one: {:?}",
                contents.root
            ),
            Store::Absent => panic!("a store was expected"),
        }
    }

    /// A survey payload holding `paths` as its entities.
    fn survey_payload(paths: &[&str]) -> String {
        serde_json::to_string(&serde_json::json!({
            "entities": paths
                .iter()
                .map(|path| serde_json::json!({ "path": path, "kind": "change-document" }))
                .collect::<Vec<_>>(),
        }))
        .expect("the payload serializes")
    }

    /// A plan document accounting for each of `entities` with `disposition`.
    fn plan_document(entries: &[(&str, &str, &str)]) -> String {
        let items: String = entries
            .iter()
            .enumerate()
            .map(|(index, (source, destination, disposition))| {
                let reason = if *disposition == "dropped" || *disposition == "folded" {
                    "<p data-field=\"note\">reason: the destination has nowhere to put it</p>"
                } else {
                    ""
                };
                format!(
                    "<li class=\"task\" id=\"task-entity-{index}\" data-change=\"added\" \
                     data-state=\"todo\"><h3>Entity {index}</h3>\
                     <p data-field=\"note\">source: {source}</p>\
                     <p data-field=\"note\">destination: {destination}</p>\
                     <p data-field=\"note\">disposition: {disposition}</p>{reason}</li>"
                )
            })
            .collect();
        format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"0.1.0\" name=\"htmlspec-version\">\
             <meta content=\"tasks\" name=\"htmlspec-kind\">\
             <title>Migration plan</title></head><body><main>{items}</main></body></html>"
        )
    }

    /// A complete plan passes, and the report says the check judged no disposition.
    #[test]
    fn promotion_accepts_a_plan_that_accounts_for_every_entity() {
        let scratch = Scratch::new("promote-complete");
        let plan = scratch.write(
            "plan.html",
            &plan_document(&[
                (
                    "openspec/specs/a/spec.md",
                    "htmlspec/specs/a/spec.html",
                    "converted",
                ),
                ("openspec/config.yaml", "none", "dropped"),
            ]),
        );
        let survey = scratch.write(
            "survey.json",
            &survey_payload(&["openspec/specs/a/spec.md", "openspec/config.yaml"]),
        );

        let (outcome, out, err) = run(Form::Human, |streams| {
            migrate_promote(&plan, &survey, None, None, PromoteMode::Perform, streams)
        });
        assert_eq!(outcome, Outcome::Success, "{out}{err}");
        assert!(
            out.contains("every surveyed entity has a disposition"),
            "{out}"
        );
        // And the report says plainly what the check did not do, because a passing coverage
        // check is easy to read as approval of the plan.
        assert!(err.contains("is right"), "{err}");
    }

    /// A plan missing entities refuses with **every** one named, and exits with the code for a
    /// document that fails a check rather than for an operation that could not run — the plan is
    /// readable and the operation ran.
    #[test]
    fn promotion_refuses_and_names_every_unaccounted_entity() {
        let scratch = Scratch::new("promote-incomplete");
        let plan = scratch.write(
            "plan.html",
            &plan_document(&[(
                "openspec/specs/a/spec.md",
                "htmlspec/specs/a/spec.html",
                "converted",
            )]),
        );
        let missing = [
            "openspec/changes/archive/x/VERSION",
            "openspec/changes/archive/x/.openspec.yaml",
            "openspec/changes/archive/x/documents/spec.html",
        ];
        let survey = scratch.write(
            "survey.json",
            &survey_payload(
                &[&["openspec/specs/a/spec.md"][..], &missing[..]]
                    .concat()
                    .to_vec(),
            ),
        );

        let (outcome, out, err) = run(Form::Human, |streams| {
            migrate_promote(&plan, &survey, None, None, PromoteMode::Perform, streams)
        });
        assert_eq!(outcome, Outcome::DocumentViolation, "{out}{err}");
        for entity in missing {
            assert!(out.contains(entity), "the report omits {entity}:\n{out}");
        }
        assert!(err.contains("promotion does not proceed"), "{err}");
    }

    /// The structured form carries the same verdict in its exit code, so a job does not have to
    /// parse the payload to learn whether promotion may proceed.
    #[test]
    fn the_structured_promotion_payload_carries_the_verdict() {
        let scratch = Scratch::new("promote-json");
        let plan = scratch.write(
            "plan.html",
            &plan_document(&[(
                "openspec/specs/a/spec.md",
                "htmlspec/specs/a/spec.html",
                "converted",
            )]),
        );
        let survey = scratch.write(
            "survey.json",
            &survey_payload(&["openspec/specs/a/spec.md", "openspec/config.yaml"]),
        );

        let (outcome, out, _) = run(Form::Structured, |streams| {
            migrate_promote(&plan, &survey, None, None, PromoteMode::Perform, streams)
        });
        assert_eq!(outcome, Outcome::DocumentViolation);
        let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
        assert_eq!(payload["covered"], serde_json::Value::Bool(false));
        assert_eq!(payload["surveyed"], 2);
        assert_eq!(payload["planned"], 1);
        assert_eq!(
            payload["unaccounted"],
            serde_json::json!(["openspec/config.yaml"])
        );
    }

    /// A plan that cannot be read is an operational failure rather than a coverage failure.
    /// Reporting every entity as unaccounted would send a contributor to the source when the
    /// fault is the file they named.
    #[test]
    fn a_plan_that_cannot_be_read_is_an_operational_failure() {
        let scratch = Scratch::new("promote-unreadable");
        let survey = scratch.write("survey.json", &survey_payload(&["openspec/config.yaml"]));
        let (outcome, out, err) = run(Form::Human, |streams| {
            migrate_promote(
                &scratch.0.join("absent.html"),
                &survey,
                None,
                None,
                PromoteMode::Perform,
                streams,
            )
        });
        assert_eq!(outcome, Outcome::OperationalFailure, "{out}");
        assert!(err.contains("could not be read"), "{err}");
    }

    /// A file that is not a survey payload is refused, naming the command that produces one.
    #[test]
    fn a_survey_file_that_is_not_a_payload_is_refused() {
        let scratch = Scratch::new("promote-not-a-payload");
        let plan = scratch.write(
            "plan.html",
            &plan_document(&[("openspec/config.yaml", "none", "dropped")]),
        );
        let survey = scratch.write("survey.json", "{\"counts\":{}}");
        let (outcome, _, err) = run(Form::Human, |streams| {
            migrate_promote(&plan, &survey, None, None, PromoteMode::Perform, streams)
        });
        assert_eq!(outcome, Outcome::OperationalFailure);
        assert!(err.contains("migrate survey --json"), "{err}");
    }

    /// The survey's human report names each change whose configuration states a predecessor, with
    /// the change it follows, and says so where none does.
    ///
    /// Each becomes a `FOLLOWS` declaration in the migrated project, written from the survey, so a
    /// contributor reading the report before the plan sees the names the plan will carry.
    #[test]
    fn the_survey_report_names_each_change_that_states_a_predecessor() {
        let scratch = Scratch::new("survey-predecessors");
        scratch.write(
            "openspec/changes/archive/2026-09-26-log-a-runbook-run/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-09-17\nfollows: establish-a-runbook\n",
        );
        scratch.write(
            "openspec/changes/archive/2026-09-18-establish-a-runbook/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-09-16\n",
        );
        scratch.write(
            "openspec/changes/archive/2026-08-30-finish-the-archival/proposal.md",
            "## Why\n",
        );
        let (outcome, out, _) = run(Form::Human, |streams| {
            migrate_survey(Some(&scratch.0), streams)
        });
        assert_eq!(outcome, Outcome::Success);
        let line = out
            .lines()
            .find(|line| line.contains("state a predecessor"))
            .unwrap_or_else(|| panic!("the report names no predecessor:\n{out}"));
        assert!(
            line.ends_with(": 2026-09-26-log-a-runbook-run follows establish-a-runbook"),
            "the report does not name exactly the one change that states a predecessor: {line}"
        );

        let none = Scratch::new("survey-no-predecessors");
        none.write(
            "openspec/changes/archive/2026-09-18-establish-a-runbook/.openspec.yaml",
            "schema: spec-driven\n",
        );
        let (_, out, _) = run(Form::Human, |streams| {
            migrate_survey(Some(&none.0), streams)
        });
        assert!(
            out.contains("no change's configuration states a predecessor"),
            "{out}"
        );
    }
}
