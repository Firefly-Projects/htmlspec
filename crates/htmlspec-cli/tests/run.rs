//! Starting a run of a runbook, and reporting what its runs recorded.
//!
//! Driven through the built binary rather than through the functions, and the reason is the same one
//! `archive.rs` gives for its runbook tests: the things under test here are **a file appearing where
//! it should** and **a file not appearing where it should not**, and an exit status alone does not
//! distinguish a refusal from a near miss.
//!
//! The load-bearing test in this file is [`a_records_directory_is_not_archivable`]. This change adds
//! a directory inside the tree the previous change's guard protects, and that guard's history is two
//! shipped data losses — so the claim that `starts_with` covers a descendant is asserted here with a
//! record actually on disk rather than reasoned about.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The binary under test, built by cargo before the test runs.
fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// A temporary directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-run-{label}-{}", std::process::id()));
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

fn run_in(directory: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    support::held(arguments, &stdout);
    (
        output.status.code().expect("the process exited normally"),
        stdout,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// A project holding a runbook whose specification states two requirements of its runs.
///
/// Built from the real templates through `new document`, so a template change reaches these tests
/// rather than being shadowed by a hand-written sample — the discipline `commands.rs`'s own tests
/// use. The example identifiers are then replaced, because `review/example-content` reports a
/// surviving one and a test asserting "no diagnostics" would fail on the template rather than on
/// this change.
///
/// **The second requirement is added rather than taken from the template**, because the template
/// writes one and a rule reporting "which requirement is short" cannot be shown to name the right
/// one from a set of size one.
fn a_project_with_a_runbook(scratch: &Scratch) -> (PathBuf, PathBuf) {
    let (code, out, err) = run_in(&scratch.0, &["init", "--tools", "none"]);
    assert_eq!(code, 0, "init failed:\n{out}{err}");
    let project = scratch.0.clone();
    let runbook = project.join("htmlspec").join("runbooks").join("release");
    std::fs::create_dir_all(&runbook).expect("the runbook directory exists");

    for (kind, file) in [
        ("specification", "specification.html"),
        ("tasks", "tasks.html"),
    ] {
        let path = runbook.join(file);
        let (code, out, err) = run_in(
            &project,
            &[
                "new",
                "document",
                kind,
                &path.to_string_lossy(),
                "--subject",
                "Releasing the project",
            ],
        );
        assert_eq!(code, 0, "new document failed:\n{out}{err}");
        let document = std::fs::read_to_string(&path).expect("the document reads");
        let mut document = document
            .replace("-example", "-real")
            .replace(
                "Replace this paragraph with what",
                "This document states what",
            )
            // A runbook's units are `unchanged`, which `runbook-authority` requires and the
            // templates cannot know. Without this the runbook itself reports
            // `review/stale-verification`, which would confound a test about the record.
            .replace("data-change=\"added\"", "data-change=\"unchanged\"");
        if file == "specification.html" {
            document = document.replace(
                "        </section>\n      </main>",
                "          <section class=\"requirement\" id=\"req-second\" \
                 data-change=\"unchanged\">\n\
                 \x20           <h3>The second obligation <span class=\"hs-id\">req-second</span>\
                 </h3>\n\
                 \x20           <p>A run of this procedure MUST record the second value too.</p>\n\
                 \x20           <article class=\"scenario\" id=\"scenario-second\" \
                 data-change=\"unchanged\">\n\
                 \x20             <h4>The second value recorded</h4>\n\
                 \x20             <dl>\n\
                 \x20               <dt>When</dt>\n\
                 \x20               <dd>A run reaches the step that produces it.</dd>\n\
                 \x20               <dt>Then</dt>\n\
                 \x20               <dd>The run records what it read.</dd>\n\
                 \x20             </dl>\n\
                 \x20           </article>\n\
                 \x20         </section>\n        </section>\n      </main>",
            );
        }
        std::fs::write(&path, document).expect("rewritten");
    }

    // Release metadata, which starting a run now requires. Written here rather than in each test
    // because it is a property of a runbook that can be run at all, and not of any one case: a
    // fixture without it exercises the migration refusal instead of whatever the test is about.
    // `carry-a-runbook-to-another-machine` made this intentional — a record that cannot say which
    // release of the procedure it ran cannot answer the question a record exists for.
    std::fs::write(runbook.join("VERSION"), "0.1.0\n").expect("the version is written");
    std::fs::write(
        runbook.join("CHANGELOG.md"),
        "# Changelog\n\n## 0.1.0\n\nThe first release of this procedure.\n",
    )
    .expect("the changelog is written");

    // The runbook itself must be clean, or a test asserting the *record* is clean is asserting
    // something about the fixture instead.
    let (code, out, err) = run_in(&project, &["validate", &runbook.to_string_lossy()]);
    assert_eq!(
        code, 0,
        "the fixture runbook does not validate, so nothing below is about the record:\n{out}{err}"
    );

    (project, runbook)
}

/// Where a runbook's records live, which no test composes for itself.
fn records(runbook: &Path) -> PathBuf {
    runbook.join("runs")
}

// -----------------------------------------------------------------------------
// The record a run starts with
// -----------------------------------------------------------------------------

/// A started run writes a record that validates and is canonical, immediately.
///
/// **Both, and under both strictnesses.** `document-template` requires what this program creates to
/// be valid, and the four constraints the design rests on are each a diagnostic — so a record that
/// validated by default and failed under `--strict` would mean one of them was mis-implemented in a
/// way only strict mode sees.
#[test]
fn a_started_run_writes_a_record_that_validates_and_is_canonical() {
    let scratch = Scratch::new("starts");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "0-14-0"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let record = records(&runbook).join("0-14-0.html");
    assert!(
        record.is_file(),
        "no record at {}; the run left nothing, which is the failure a run record exists to \
         prevent",
        record.display()
    );
    assert!(
        out.contains(&record.display().to_string()),
        "the report does not name where the record landed:\n{out}"
    );

    for arguments in [
        vec!["validate", &*record.to_string_lossy()],
        vec!["validate", "--strict", &record.to_string_lossy()],
    ] {
        let (code, out, err) = run_in(&project, &arguments);
        assert_eq!(
            code, 0,
            "the record this operation wrote does not validate under {arguments:?}:\n{out}{err}"
        );
        assert!(
            out.contains("no diagnostics"),
            "the record reports a diagnostic under {arguments:?}:\n{out}"
        );
    }

    let (code, out, err) = run_in(&project, &["format", "--check", &record.to_string_lossy()]);
    assert_eq!(code, 0, "the record is not canonical:\n{out}{err}");
}

/// A new record declares the current contract version, in its head and in its masthead.
///
/// It declared `0.1.0` in both until `1.0.0`, which understated a document the current program
/// writes. Read from the record itself, so a head and a masthead that disagreed would fail here
/// whichever one moved.
#[test]
fn a_new_record_declares_the_current_contract_version_in_its_head_and_its_masthead() {
    let scratch = Scratch::new("declares");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    let (code, out, err) = run_in(&project, &["run", "release", "--as", "0-14-0"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let source = std::fs::read_to_string(records(&runbook).join("0-14-0.html")).expect("readable");
    let version = htmlspec_schema::CONTRACT_VERSION;
    assert!(
        source.contains(&format!(
            r#"<meta content="{version}" name="htmlspec-version">"#
        )),
        "the record's head does not declare {version}:\n{source}"
    );
    let masthead = source
        .split_once("<dt>Contract</dt>")
        .map(|(_, after)| after)
        .expect("the record's masthead names its contract version");
    let named = masthead
        .trim_start()
        .strip_prefix("<dd>")
        .and_then(|rest| rest.split_once("</dd>"))
        .map(|(named, _)| named)
        .expect("the contract version is a <dd>");
    assert_eq!(
        named, version,
        "the record's masthead names another version"
    );
}

/// The record carries the procedure, the copied requirements, and their inline markup.
///
/// **The inline markup is the assertion worth having.** The projection flattens a unit's text to
/// strings, so a record composed from the model would lose the `<code>` and `<strong>` a runbook's
/// prose is full of — and it would still validate, so no other test here would notice. The record
/// exists to say which version of the procedure ran, and a paraphrase of that procedure does not.
#[test]
fn a_record_carries_the_procedure_and_the_copied_requirements() {
    let scratch = Scratch::new("carries");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    // A distinctive piece of inline markup in the procedure, which the record must preserve.
    let procedure = runbook.join("tasks.html");
    let source = std::fs::read_to_string(&procedure).expect("the procedure reads");
    std::fs::write(
        &procedure,
        source.replace(
            "State the work in a sentence",
            "Run <code>scripts/check</code> and <strong>read every line</strong>. State the work \
             in a sentence",
        ),
    )
    .expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "batch-a"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let written =
        std::fs::read_to_string(records(&runbook).join("batch-a.html")).expect("the record reads");

    assert!(
        written.contains("<code>scripts/check</code>")
            && written.contains("<strong>read every line</strong>"),
        "the record lost the procedure's inline markup, so it paraphrases the procedure that ran \
         rather than carrying it"
    );
    assert!(
        written.contains("id=\"task-real\""),
        "the record does not carry the runbook's step"
    );
    for requirement in ["req-real", "req-second"] {
        assert!(
            written.contains(&format!("id=\"{requirement}\"")),
            "the record does not carry {requirement}. A reference from a runbook's documents \
             resolves within the document that makes it, so a record that pointed outward would \
             fail refs/unresolved"
        );
    }
    assert!(
        written.contains("MUST"),
        "the copied requirement lost its modal, which draws schema/weak-modal"
    );
    assert!(
        written.contains("class=\"scenario\""),
        "the copied requirement lost its scenarios, which is a schema error"
    );
    assert!(
        !written.contains("data-change=\"added\""),
        "the record declares a unit as added, which draws review/stale-verification: the \
         obligation existed before the run started"
    );
}

/// A runbook whose own units are `added` still produces a record whose units are `unchanged`.
///
/// **This test exists because the obvious one passed for the wrong reason.** Mutating
/// `settle_change_states` into a no-op left all fifteen other tests in this file green, because
/// [`a_project_with_a_runbook`] rewrites the template's `added` into `unchanged` — so the copy had
/// nothing to settle and an assertion that the record carries no `added` was asserting a property of
/// the fixture. That is the same shape the previous change recorded: a test that cannot fail for the
/// reason it was written is worse than no test.
///
/// The input is not contrived. `htmlspec new document` writes `added` on every unit and cannot know
/// it is writing a runbook, which `runbook-authority` states explicitly — so a runbook authored by
/// hand *is* the runbook that carries `added`, and it is the input the copy has to fix.
///
/// The warning it prevents is `review/stale-verification`: the record's verification names a
/// requirement, so a requirement copied as `added` makes that verification attest to an obligation
/// declared not to exist yet. Asserted through validation rather than only through the markup,
/// because the warning is the consequence the attribute exists for.
#[test]
fn a_record_declares_its_copied_units_unchanged_whatever_the_runbook_declares() {
    let scratch = Scratch::new("settles");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    // The runbook put back into the state the template writes, which is what an author authoring by
    // hand has.
    for file in ["specification.html", "tasks.html"] {
        let path = runbook.join(file);
        let source = std::fs::read_to_string(&path).expect("the document reads");
        std::fs::write(
            &path,
            source.replace("data-change=\"unchanged\"", "data-change=\"added\""),
        )
        .expect("rewritten");
    }

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let path = records(&runbook).join("first.html");
    let written = std::fs::read_to_string(&path).expect("the record reads");
    assert!(
        !written.contains("data-change=\"added\""),
        "the record copied the runbook's `added` state, so its verifications attest to obligations \
         the record itself declares do not exist yet"
    );
    // Every descendant, not only each unit's root. A scenario left `added` inside a requirement
    // settled to `unchanged` is the half-fixed state the previous change shipped and a review
    // caught.
    assert_eq!(
        written.matches("data-change=\"unchanged\"").count(),
        written.matches("data-change=").count(),
        "some unit in the record declares a change state that is not `unchanged`:\n{written}"
    );

    let (code, out, err) = run_in(&project, &["validate", &path.to_string_lossy()]);
    assert_eq!(code, 0, "the record does not validate:\n{out}{err}");
    assert!(
        !out.contains("review/stale-verification"),
        "the record attests to an obligation it declares has moved, which is the warning the \
         change state exists to prevent:\n{out}"
    );
}

/// Every copied requirement gets a verification, written as not yet run.
///
/// `not-run` rather than absent, so the record *enumerates* what it owes. The conformance rule then
/// reports each one as unrecorded, which is true of a run that has only just started — and
/// [`a_just_started_run_has_recorded_nothing`] is where that pairing is asserted.
#[test]
fn a_record_enumerates_what_the_run_owes() {
    let scratch = Scratch::new("enumerates");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");
    assert!(
        out.contains("2 requirement(s) to record"),
        "the report does not say how many requirements this run owes:\n{out}"
    );

    let written =
        std::fs::read_to_string(records(&runbook).join("first.html")).expect("the record reads");
    for requirement in ["req-real", "req-second"] {
        assert!(
            written.contains(&format!("data-refs=\"{requirement}\"")),
            "no verification names {requirement}, so the record does not enumerate what it owes"
        );
    }
    assert_eq!(
        written.matches("data-outcome=\"not-run\"").count(),
        2,
        "the record does not write one not-yet-run entry per requirement:\n{written}"
    );
}

// -----------------------------------------------------------------------------
// The conformance report
// -----------------------------------------------------------------------------

/// A run that has only just started has recorded nothing, and saying so does not gate it.
///
/// The pairing that makes the `review/*` severity necessary rather than convenient: the operation
/// that starts a run leaves every requirement unrecorded, so a rule that gated would fail every run
/// at the moment it began.
#[test]
fn a_just_started_run_has_recorded_nothing() {
    let scratch = Scratch::new("just-started");
    let (project, _runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(
        code, 0,
        "the conformance report gated a run that has just started, which would fail every run at \
         the moment it began:\n{out}{err}"
    );
    for requirement in ["req-real", "req-second"] {
        assert!(
            out.contains(requirement),
            "the report does not name {requirement} as unrecorded:\n{out}"
        );
    }
    assert!(
        out.contains("first.html"),
        "the report does not name which run is short:\n{out}"
    );
}

/// A run that recorded its outcomes reports nothing.
///
/// The accepting direction, and it is what shows the rule is not merely reporting every run. Without
/// it, a rule that always fired would pass every other test in this file.
#[test]
fn a_run_that_recorded_its_outcomes_reports_nothing() {
    let scratch = Scratch::new("recorded");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    // What an operator does as the run proceeds: replace the outcome and the method. Textual here
    // because that is what editing the document is.
    let path = records(&runbook).join("first.html");
    let written = std::fs::read_to_string(&path).expect("the record reads");
    let written = written
        .replace("data-outcome=\"not-run\"", "data-outcome=\"passed\"")
        .replace(
            "Not run yet. State how this requirement was checked",
            "Advanced the version first and confirmed the installed stage passed. State how this \
             requirement was checked",
        );
    std::fs::write(&path, written).expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(code, 0, "the report gated:\n{out}{err}");
    assert!(
        out.contains("records what the runbook states"),
        "a run that recorded every outcome was reported as short:\n{out}"
    );
    assert!(
        !out.contains("review/unrecorded-requirement"),
        "a conforming run drew a diagnostic:\n{out}"
    );

    // And the edited record still validates, which is the property that makes a run resumable: a
    // record edited after it was written is the ordinary case rather than a repair.
    let (code, out, err) = run_in(&project, &["validate", &path.to_string_lossy()]);
    assert_eq!(code, 0, "the edited record does not validate:\n{out}{err}");
}

/// A recorded failure is a correct record of a failure, not a document fault.
///
/// The obligation is that the run recorded an outcome, not that the outcome was good. A rule that
/// reported `failed` would teach an operator to record `passed`, which is the opposite of what a run
/// record is for.
#[test]
fn a_run_that_recorded_a_failure_reports_nothing() {
    let scratch = Scratch::new("failed");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let path = records(&runbook).join("first.html");
    let written = std::fs::read_to_string(&path).expect("the record reads");
    std::fs::write(
        &path,
        written
            .replace("data-outcome=\"not-run\"", "data-outcome=\"failed\"")
            .replace(
                "Not run yet. State how this requirement was checked",
                "Regenerated before advancing the version, and the installed stage went red. State \
                 how this requirement was checked",
            ),
    )
    .expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(code, 0, "the report gated:\n{out}{err}");
    assert!(
        !out.contains("review/unrecorded-requirement"),
        "a run that honestly recorded a failure was reported as having recorded nothing:\n{out}"
    );
}

/// A recorded partial check is a recorded outcome, the twin of a recorded failure.
///
/// `record-a-partial-verification`: a check that reached part of a requirement ran, so the run
/// recorded an outcome for it, and the record states what the check did not reach in an
/// `unchecked` paragraph. The record still validates, because `partial` with that paragraph is a
/// complete record.
#[test]
fn a_run_that_recorded_a_partial_check_reports_nothing() {
    let scratch = Scratch::new("partial");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let path = records(&runbook).join("first.html");
    let written = std::fs::read_to_string(&path).expect("the record reads");
    let rewritten = written
        .replace("data-outcome=\"not-run\"", "data-outcome=\"partial\"")
        .replace(
            "Not run yet. State how this requirement was checked",
            "Regenerated before advancing the version, and the installed stage passed. State how \
             this requirement was checked",
        )
        .replace(
            "and set the outcome accordingly.</p>",
            "and set the outcome accordingly.</p>\n\
             \x20           <p data-field=\"unchecked\">The release stage, which the run did not \
             reach.</p>",
        );
    assert_eq!(
        rewritten.matches("data-field=\"unchecked\"").count(),
        2,
        "the record no longer carries one verification per requirement as this test expects:\n\
         {rewritten}"
    );
    std::fs::write(&path, rewritten).expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(code, 0, "the report gated:\n{out}{err}");
    assert!(
        !out.contains("review/unrecorded-requirement"),
        "a run that recorded a partial check was reported as having recorded nothing:\n{out}"
    );

    let (code, out, err) = run_in(&project, &["validate", &path.to_string_lossy()]);
    assert_eq!(
        code, 0,
        "a record of a partial check does not validate:\n{out}{err}"
    );
}

/// A runbook nobody has run reports nothing, and says so distinctly.
#[test]
fn a_runbook_with_no_runs_reports_nothing() {
    let scratch = Scratch::new("no-runs");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("no runs on record"),
        "a runbook with no runs is not distinguished from one whose runs all conform:\n{out}"
    );
    assert!(
        !records(&runbook).exists(),
        "reporting on a runbook created its records directory; --check writes nothing"
    );
}

// -----------------------------------------------------------------------------
// The guard, with a record on disk
// -----------------------------------------------------------------------------

/// A runbook's records are not archivable, resolvable as a change, or destroyable.
///
/// **This change adds a directory inside the tree the previous change's guard protects, and that
/// guard's history is two shipped data losses.** Its evidence recorded "Safe — `starts_with` covers
/// descendants" about a `runs/` directory that did not exist yet; this asserts it with a record
/// actually on disk, through every route, and checks the files afterwards.
///
/// The surviving files are what matters. Exit status alone does not distinguish a refusal from a
/// near miss, which is the lesson `archive.rs` records from the first data loss.
#[test]
fn a_records_directory_is_not_archivable() {
    let scratch = Scratch::new("guard");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");
    let record = records(&runbook).join("first.html");

    // The version advanced, so the version-ordering gate does not refuse before the guard is
    // reached. A probe that bounced off an unrelated gate would pass against no guard at all,
    // which is the measured trap the previous change recorded.
    let declaration = project.join("spec.nix");
    let source = std::fs::read_to_string(&declaration).expect("the declaration reads");
    std::fs::write(&declaration, source.replace("0.0.0", "0.1.0")).expect("written");

    // **The runbook first, and the ordering is the whole point.** `runs/` holds no
    // `specification.html`, so the absent-specification gate refuses it before the runbook guard is
    // reached — measured: with the guard neutered, pointing archival at `runs/` still exits non-zero
    // saying `has no specification.html`. That is the trap the previous change recorded, where a
    // probe bounced off an unrelated gate and looked safe.
    //
    // The *runbook* holds both documents and real content, so it clears every gate and reaches the
    // move — taking `runs/` with it, because archival renames the whole directory. So the runbook is
    // the route that can actually destroy a record, and it is checked first and checked for the file
    // surviving rather than only for the message.
    let directory = records(&runbook);
    let relative = "htmlspec/runbooks/release/runs";
    for (label, target) in [
        (
            "the runbook holding the records",
            String::from("htmlspec/runbooks/release"),
        ),
        ("the records by relative path", relative.to_owned()),
        (
            "the records by absolute path",
            directory.to_string_lossy().into_owned(),
        ),
        (
            "the records by traversal",
            String::from("htmlspec/changes/../runbooks/release/runs"),
        ),
    ] {
        let (code, out, err) = run_in(&project, &["archive", &target, "--yes"]);
        assert_ne!(code, 0, "archival accepted {label}:\n{out}{err}");
        assert!(
            !out.contains("archived to"),
            "archival moved {label}:\n{out}"
        );
        // Checked after *each* route rather than once at the end, so a route that destroys the
        // record fails naming itself rather than leaving a later route to report the damage.
        assert!(
            record.is_file(),
            "the run's record is gone after archival was pointed at {label}: a route past the \
             guard destroyed a run's history"
        );
        assert!(
            err.contains("is a runbook, not a change"),
            "the refusal of {label} does not name what the directory is, so the refusal came from \
             some other gate and this assertion says nothing about the guard:\n{out}{err}"
        );
    }

    // The two read-only operations that share the resolver, each refusing for its own reason.
    for (subcommand, reason) in [
        ("status", "two documents whose steps stay unstarted"),
        ("sync", "requires of a run"),
    ] {
        let (code, out, err) = run_in(&project, &[subcommand, "--change", relative]);
        assert_ne!(
            code, 0,
            "{subcommand} reported on a runbook's records:\n{out}{err}"
        );
        assert!(
            err.contains(reason),
            "{subcommand} refused for the wrong reason, which makes the refusal look \
             unreliable:\n{out}{err}"
        );
    }

    assert!(
        record.is_file(),
        "the run's record is gone: something destroyed a runbook's history"
    );
    for file in ["specification.html", "tasks.html"] {
        assert!(
            runbook.join(file).is_file(),
            "{file} is gone: a route past the guard destroyed the runbook"
        );
    }
    let archive = project.join("htmlspec").join("changes").join("archive");
    let archived: Vec<PathBuf> = std::fs::read_dir(&archive)
        .expect("the archive is readable")
        .flatten()
        .map(|entry| entry.path())
        .collect();
    assert!(
        archived.is_empty(),
        "something reached the archive: {archived:?}"
    );
}

/// A nested project or repository inside the records directory does not disable the guard.
///
/// The two bypasses the previous change measured, re-applied one directory further down. Neither is
/// contrived: a runbook documenting how to bootstrap a project has a nested `htmlspec/`, and its
/// run records would sit beside that.
#[test]
fn a_nested_project_or_repository_does_not_disable_the_guard() {
    let scratch = Scratch::new("bypass");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");
    let record = records(&runbook).join("first.html");

    let declaration = project.join("spec.nix");
    let source = std::fs::read_to_string(&declaration).expect("the declaration reads");
    std::fs::write(&declaration, source.replace("0.0.0", "0.1.0")).expect("written");

    // **Pointed at the runbook rather than at `runs/`, for the reason
    // `a_records_directory_is_not_archivable` states.** `runs/` bounces off the
    // absent-specification gate whatever the guard does, so a probe aimed there would pass against
    // no guard at all. The runbook clears every gate and reaches the move, and the nested directory
    // is planted inside `runs/` — which is where a run record's neighbour would be.
    for nested in ["htmlspec/runbooks", ".git"] {
        let planted = records(&runbook).join(nested);
        std::fs::create_dir_all(&planted).expect("the nested directory exists");

        let (code, out, err) = run_in(&project, &["archive", "htmlspec/runbooks/release", "--yes"]);
        assert_ne!(
            code, 0,
            "a nested {nested} inside the records directory let archival accept the runbook:\n\
             {out}{err}"
        );
        assert!(
            !out.contains("archived to"),
            "archival moved the runbook with a nested {nested} inside its records:\n{out}"
        );
        assert!(
            record.is_file(),
            "a nested {nested} let archival destroy a run's record"
        );
        assert!(
            runbook.join("tasks.html").is_file(),
            "a nested {nested} let archival destroy the runbook"
        );
        assert!(
            err.contains("is a runbook, not a change"),
            "the refusal with a nested {nested} does not name what the directory is, so it came \
             from another gate:\n{out}{err}"
        );

        let top = records(&runbook).join(nested.split('/').next().expect("a first component"));
        std::fs::remove_dir_all(&top).expect("the nested directory is removed");
    }
}

/// A record's name is refused where it is a path rather than a name.
///
/// The name reaches a `join`, so `--as ../../../sneaky` would write a document outside the project.
/// `ProjectRoot::is_a_name` is the constructor the previous change moved this invariant into after
/// re-litigating it at three call sites, and this asserts the new caller uses it.
#[test]
fn a_record_name_carrying_a_path_is_refused() {
    let scratch = Scratch::new("names");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    for given in ["../../../sneaky", "runs/sneaky", "..", ".", "a\\b"] {
        let (code, out, err) = run_in(&project, &["run", "release", "--as", given]);
        assert_ne!(
            code, 0,
            "{given:?} was accepted as a record name:\n{out}{err}"
        );
        assert!(
            err.contains("is a path rather than a record name"),
            "the refusal of {given:?} does not name the problem:\n{out}{err}"
        );
    }

    // Nothing was written anywhere, which is the property the refusal exists for: a name refused
    // after a directory was created would have left the traversal's effect behind.
    assert!(
        !records(&runbook).exists(),
        "a refused name still created the records directory"
    );
    assert!(
        !scratch.0.join("sneaky.html").exists()
            && !scratch.0.join("..").join("sneaky.html").exists(),
        "a traversal escaped the project"
    );
}

/// Starting a run over an existing record is refused, and the record is untouched.
///
/// A run interrupted and continued is that record edited rather than a second one, so this operation
/// starts a run and never resumes one. Overwriting would destroy the history the record holds, which
/// is the family of defect this whole line of work exists around.
#[test]
fn starting_a_run_does_not_overwrite_a_record() {
    let scratch = Scratch::new("overwrite");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    // An edit standing in for the work a run does, so the assertion below is about *this* content
    // surviving rather than about a file of the right name existing.
    let path = records(&runbook).join("first.html");
    let written = std::fs::read_to_string(&path).expect("the record reads");
    let marked = written.replace("data-state=\"todo\"", "data-state=\"done\"");
    std::fs::write(&path, &marked).expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_ne!(
        code, 0,
        "starting a run over an existing record succeeded:\n{out}{err}"
    );
    assert!(
        err.contains("already exists"),
        "the refusal does not say the record is there:\n{out}{err}"
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("the record reads"),
        marked,
        "the record was rewritten, so a second run destroyed the first run's work"
    );
}

/// A runbook with no procedure has nothing to run, and a name that is not a runbook is reported.
#[test]
fn a_run_of_something_that_is_not_a_runbook_is_reported() {
    let scratch = Scratch::new("absent");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "cookies", "--as", "batch-a"]);
    assert_ne!(code, 0, "a run of an absent runbook succeeded:\n{out}{err}");
    assert!(
        err.contains("no runbook named cookies") && err.contains("list --runbooks"),
        "the report does not say what is absent or how to see what is there:\n{out}{err}"
    );

    std::fs::remove_file(runbook.join("tasks.html")).expect("the procedure is removed");
    let (code, out, err) = run_in(&project, &["run", "release", "--as", "batch-a"]);
    assert_ne!(
        code, 0,
        "a run of a runbook with no procedure succeeded:\n{out}{err}"
    );
    assert!(
        err.contains("tasks.html"),
        "the report does not name the document a run copies:\n{out}{err}"
    );
    assert!(
        !records(&runbook).exists(),
        "a refused run created the records directory"
    );
}

/// The two modes are exclusive, and neither is defaulted.
///
/// `--as` has no default on purpose: a date collides when a procedure runs twice in a day, and a
/// sequence number derives an ordering from a directory's contents — which `runbook-authority`
/// forbids for a runbook's name. The report says so rather than inventing one.
#[test]
fn the_two_modes_are_exclusive_and_the_record_name_is_not_defaulted() {
    let scratch = Scratch::new("modes");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release"]);
    assert_ne!(code, 0, "a run with no name succeeded:\n{out}{err}");
    assert!(
        err.contains("--as") && err.contains("--check"),
        "the report names neither the missing flag nor the other mode:\n{out}{err}"
    );

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "x", "--check"]);
    assert_ne!(code, 0, "both modes at once succeeded:\n{out}{err}");
    assert!(
        err.contains("takes no --as"),
        "the report does not say which flag is surplus:\n{out}{err}"
    );

    assert!(
        !records(&runbook).exists(),
        "a usage error created the records directory"
    );
}

/// The structured payload carries what the human report carries.
#[test]
fn the_payload_names_the_record_and_the_diagnostics() {
    let scratch = Scratch::new("payload");
    let (project, _runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["--json", "run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
    assert_eq!(payload["runbook"], "release");
    assert_eq!(payload["record"], "first");
    assert_eq!(payload["recording"], 2);
    assert!(
        payload["path"]
            .as_str()
            .expect("a path")
            .ends_with("first.html"),
        "the payload does not say where the record landed: {payload}"
    );

    let (code, out, err) = run_in(&project, &["--json", "run", "release", "--check"]);
    assert_eq!(code, 0, "the report gated:\n{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload parses");
    assert_eq!(
        payload["required"]
            .as_array()
            .expect("the requirements")
            .len(),
        2
    );
    assert_eq!(
        payload["diagnostics"]
            .as_array()
            .expect("the diagnostics")
            .len(),
        2,
        "the payload does not carry one diagnostic per unrecorded requirement: {payload}"
    );
    assert_eq!(
        payload["diagnostics"][0]["rule"],
        "review/unrecorded-requirement"
    );
    assert_eq!(payload["diagnostics"][0]["severity"], "warning");
}

/// A runbook stating nothing about its runs has runs nothing checks.
///
/// **Stated by a specification holding no requirement, rather than by no specification at all, and
/// that is a change this repository's own contract made.** `carry-a-runbook-to-another-machine`
/// turned an absent specification into a migration refusal for a *new start*, because the read that
/// treated absent as empty was the same read that would treat unreadable as empty — and an unreadable
/// specification is a machine contract bypassed silently. The subject of this test is what `--check`
/// reports, which is unchanged, so it reaches that subject through a runbook that can still be run.
#[test]
fn a_runbook_stating_nothing_has_runs_nothing_checks() {
    let scratch = Scratch::new("silent");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    // A specification that states nothing of its runs, which is the form a runbook takes before its
    // author has written what its runs must record.
    strip_requirements(&runbook);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(
        code, 0,
        "a run of a runbook stating nothing was refused; a procedure with no stated contract is \
         still worth recording:\n{out}{err}"
    );
    let record = records(&runbook).join("first.html");
    let (code, out, err) = run_in(&project, &["validate", &record.to_string_lossy()]);
    assert_eq!(code, 0, "the record does not validate:\n{out}{err}");

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("states no requirement"),
        "a runbook stating nothing is not distinguished from one whose runs conform:\n{out}"
    );
}

/// Removes every unit from a fixture runbook's specification that states or attests to an obligation.
///
/// Leaves a valid document stating nothing, which is the case a runbook is in before its author has
/// written what its runs must record. **The verifications and the evidence go with the
/// requirements**, because a reference resolves within one document: excising `req-real` and leaving
/// the verification naming it would produce `refs/unresolved`, and the test below would be about the
/// fixture rather than about a runbook that states nothing.
fn strip_requirements(runbook: &Path) {
    let path = runbook.join("specification.html");
    let source = std::fs::read_to_string(&path).expect("the specification reads");
    let stripped = ["requirement", "verification", "evidence"]
        .iter()
        .fold(source, |source, class| remove_units(&source, class));
    std::fs::write(&path, stripped).expect("rewritten");

    // Whatever the excision produced has to still be a document this program accepts, or the test
    // below is about the fixture. Canonical too, since the excision moves bytes the formatter would
    // otherwise put back.
    let project = runbook
        .ancestors()
        .nth(3)
        .expect("the project root")
        .to_owned();
    let (code, out, err) = run_in(&project, &["format", &path.to_string_lossy()]);
    assert_eq!(
        code, 0,
        "the stripped specification will not format:\n{out}{err}"
    );
    let (code, out, err) = run_in(&project, &["validate", &path.to_string_lossy()]);
    assert_eq!(
        code, 0,
        "stripping the requirements left a document that does not validate:\n{out}{err}"
    );
}

/// Removes every unit of one class, and the sections that held only them.
///
/// A unit is a `<section>` carrying the class, and none of these classes nests a `<section>` of its
/// own — a requirement's scenarios are `<article>` elements — so the matching close tag is the first
/// one after the open.
fn remove_units(source: &str, class: &str) -> String {
    let opener = format!("<section class=\"{class}\"");
    let mut out = String::new();
    let mut rest = source;
    while let Some(at) = rest.find(&opener) {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        let close = after
            .find("</section>")
            .map(|found| found + "</section>".len())
            .unwrap_or(after.len());
        rest = &after[close..];
    }
    out.push_str(rest);
    out
}

// -----------------------------------------------------------------------------
// What reaches the filesystem, and what does not
// -----------------------------------------------------------------------------

/// A symbolic link at the record's path does not become a route out of the project.
///
/// **The assertion is where the bytes went, not the exit code.** Measured against the version using
/// `Path::exists`: with `runs/x.html` a broken symlink to `../../../../../ESCAPED.html`, the run
/// exited **0**, reported `started a run of rb`, and wrote 19574 bytes outside the project — so a test
/// reading the exit status alone would have passed.
///
/// `Path::exists` follows symlinks and reports `false` for a broken one, which this project already
/// learned when the archival guard failed open a third time. The lesson was stated in `commands.rs`
/// and not applied here.
#[test]
fn a_symlink_at_the_record_path_does_not_escape_the_project() {
    let scratch = Scratch::new("symlink");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    // Outside the scratch directory entirely, so "did it escape" is unambiguous.
    let outside =
        std::env::temp_dir().join(format!("htmlspec-escaped-{}.html", std::process::id()));
    let _ = std::fs::remove_file(&outside);

    std::fs::create_dir_all(records(&runbook)).expect("the records directory exists");
    let link = records(&runbook).join("x.html");
    std::os::unix::fs::symlink(&outside, &link).expect("the symlink is created");

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "x"]);

    assert!(
        !outside.exists(),
        "the run wrote through a symlink to {}, outside the project entirely",
        outside.display()
    );
    assert_ne!(code, 0, "a run through a symlink succeeded:\n{out}{err}");
    assert!(
        err.contains("symbolic link"),
        "the refusal does not name what is there, so a contributor cannot tell it from an existing \
         record:\n{out}{err}"
    );
    // The link itself is untouched: this operation refuses rather than tidying up after somebody.
    assert!(std::fs::symlink_metadata(&link)
        .expect("the link is still there")
        .file_type()
        .is_symlink());
    let _ = std::fs::remove_file(&outside);
}

/// A record that does not validate is never written, so a failed run leaves nothing behind.
///
/// **Three defects in one ordering, and the assertion for each is on disk or on the stream.** The
/// record used to be written before it was validated, so: an invalid record sat on disk, the
/// existing-record refusal then fired on the retry and wedged the run with no CLI route back, and the
/// report announced success before the validation that failed.
#[test]
fn a_record_that_does_not_validate_is_not_written_and_does_not_wedge_the_run() {
    let scratch = Scratch::new("unwritten");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    // A requirement with no identifier, which is a fault in the *runbook* and the likeliest of the
    // cases that produce an invalid record.
    let path = runbook.join("specification.html");
    let source = std::fs::read_to_string(&path).expect("the specification reads");
    std::fs::write(
        &path,
        source.replace(
            "<section class=\"requirement\" id=\"req-real\" data-change=\"unchanged\">",
            "<section class=\"requirement\" data-change=\"unchanged\">",
        ),
    )
    .expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_ne!(code, 0, "an invalid record was accepted:\n{out}{err}");
    assert!(
        !records(&runbook).join("first.html").exists(),
        "an invalid record was left on disk, which both wedges the retry and leaves a document \
         somebody can open"
    );
    // The report names the runbook's own document as the fault, because that is where it is and it is
    // what the contributor can fix. The previous wording blamed the copy.
    //
    // **The refusal now arrives earlier than this test used to meet it**, and that is the point of
    // validating the snapshot before composing from it: the fault is in the runbook's specification,
    // so it is reported against that document rather than against a record that was never written.
    // What the test asserts is unchanged — nothing on disk, and the same name works once the runbook
    // is fixed.
    assert!(
        err.contains("specification.html does not validate"),
        "the report does not name the runbook's document, so a contributor looks in the wrong \
         place:\n{out}{err}"
    );

    // The wedge: the same name works once the runbook is fixed.
    std::fs::write(&path, &source).expect("the specification is restored");
    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(
        code, 0,
        "the failed run wedged the name, so there is no route back through the CLI:\n{out}{err}"
    );
    assert!(records(&runbook).join("first.html").is_file());
}

/// The `--json` payload is not emitted for a run that fails.
///
/// A payload saying `started a run` on a process exiting 2 is a payload an automated caller believes.
#[test]
fn the_payload_is_not_emitted_for_a_run_that_fails() {
    let scratch = Scratch::new("payload-failure");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let path = runbook.join("specification.html");
    let source = std::fs::read_to_string(&path).expect("the specification reads");
    std::fs::write(
        &path,
        source.replace(
            "<section class=\"requirement\" id=\"req-real\" data-change=\"unchanged\">",
            "<section class=\"requirement\" data-change=\"unchanged\">",
        ),
    )
    .expect("rewritten");

    let (code, out, err) = run_in(&project, &["--json", "run", "release", "--as", "first"]);
    assert_ne!(code, 0, "an invalid record was accepted:\n{out}{err}");
    assert!(
        out.trim().is_empty(),
        "a payload was emitted for a run that exited {code}:\n{out}"
    );
}

/// A record that cannot be read is reported, not counted as absent.
///
/// Byte-identical output for "nobody has run this" and "one record exists and I could not read it" is
/// the conflation, and this code distinguishes those cases three lines away from where it was losing
/// the distinction.
#[test]
fn a_record_that_cannot_be_read_is_not_an_absent_record() {
    let scratch = Scratch::new("unreadable");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let path = records(&runbook).join("first.html");
    let mut permissions = std::fs::metadata(&path)
        .expect("the record's metadata reads")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o000);
    std::fs::set_permissions(&path, permissions).expect("the record is made unreadable");

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);

    // Restored before asserting, so a failure does not leave the scratch directory undeletable.
    let mut permissions = std::fs::metadata(&path)
        .expect("the record's metadata reads")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o644);
    let _ = std::fs::set_permissions(&path, permissions);

    assert_eq!(code, 0, "{out}{err}");
    let whole = format!("{out}{err}");
    assert!(
        whole.contains("cannot read the record first.html"),
        "the unreadable record is not named:\n{whole}"
    );
    assert!(
        !whole.contains("has no runs on record"),
        "a record that could not be read was reported as no runs at all, which is a different \
         fact:\n{whole}"
    );
}

/// A record whose markup needs error recovery is not read as if it had one meaning.
///
/// The recovery gate `compose` applies on the write path, applied on the read path — where it was
/// missing. A document needing recovery has more than one reading, and `--check` picked one silently.
#[test]
fn a_record_whose_markup_needs_recovery_is_excluded_from_the_report() {
    let scratch = Scratch::new("recovery");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let path = records(&runbook).join("first.html");
    let source = std::fs::read_to_string(&path).expect("the record reads");
    // A stray close tag, which is the shape the parser recovers from rather than refuses.
    std::fs::write(&path, source.replace("</main>", "</section></main>")).expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(code, 0, "{out}{err}");
    let whole = format!("{out}{err}");
    assert!(
        whole.contains("requires error recovery"),
        "a record with more than one reading was read as if it had one:\n{whole}"
    );
}

/// The record carries the procedure's prose, not only its task units.
///
/// Measured against this repository's **real** release runbook, because the dropped content was the
/// section stating that the procedure needs `python3` and a repository that is not already an
/// html-spec project — a hazard this project discovered by violating it.
#[test]
fn the_record_carries_the_procedures_prose() {
    let scratch = Scratch::new("prose");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    // A section of prose beside the tasks, which is the shape the release runbook has.
    let path = runbook.join("tasks.html");
    let source = std::fs::read_to_string(&path).expect("the procedure reads");
    std::fs::write(
        &path,
        source.replace(
            "        <section id=\"hs-tasks\">",
            "        <section id=\"hs-what-this-consumes\">\n\
             \x20         <h2>What this procedure consumes</h2>\n\
             \x20         <p><code>python3</code>, and a repository that is not already an \
             html-spec project.</p>\n\
             \x20       </section>\n\
             \x20       <section id=\"hs-tasks\">",
        ),
    )
    .expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let written =
        std::fs::read_to_string(records(&runbook).join("first.html")).expect("the record reads");
    assert!(
        written.contains("hs-what-this-consumes")
            && written.contains("<code>python3</code>")
            && written.contains("not already an html-spec project"),
        "the record dropped the prose the procedure states its prerequisites in, so it cannot say \
         what the run consumed"
    );

    // And it still validates and formats, which is what makes carrying the prose safe rather than
    // merely more faithful.
    for arguments in [
        vec!["validate", "htmlspec/runbooks/release/runs/first.html"],
        vec![
            "validate",
            "--strict",
            "htmlspec/runbooks/release/runs/first.html",
        ],
        vec![
            "format",
            "--check",
            "htmlspec/runbooks/release/runs/first.html",
        ],
    ] {
        let (code, out, err) = run_in(&project, &arguments);
        assert_eq!(code, 0, "the record fails {arguments:?}:\n{out}{err}");
    }
}

/// The record carries the fallback stylesheet once, in its head, and stays valid and canonical.
///
/// A record sits three folders below the store, which is where Safari refuses a stylesheet opened
/// from disk. document-template's "A document stays readable when its stylesheet cannot load" places
/// the fallback in every document the toolchain writes, and a record is one. Once, even when the
/// runbook's own procedure carries one, because the record copies the procedure's `main` and never
/// its head.
#[test]
fn the_record_carries_the_fallback_stylesheet_once() {
    let scratch = Scratch::new("fallback");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let written =
        std::fs::read_to_string(records(&runbook).join("first.html")).expect("the record reads");
    let element = htmlspec_assets::fallback::ELEMENT;
    assert_eq!(
        written.matches(element).count(),
        1,
        "the record does not carry the fallback stylesheet exactly once:\n{written}"
    );
    let head_end = written.find("</head>").expect("a record has a head");
    assert!(
        written.find(element).is_some_and(|at| at < head_end),
        "the record carries the fallback outside its head"
    );

    for arguments in [
        vec![
            "validate",
            "--strict",
            "htmlspec/runbooks/release/runs/first.html",
        ],
        vec![
            "format",
            "--check",
            "htmlspec/runbooks/release/runs/first.html",
        ],
    ] {
        let (code, out, err) = run_in(&project, &arguments);
        assert_eq!(code, 0, "the record fails {arguments:?}:\n{out}{err}");
    }
}

/// A withdrawn requirement is carried by neither the record nor the report.
#[test]
fn a_withdrawn_requirement_is_neither_copied_nor_demanded() {
    let scratch = Scratch::new("withdrawn");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let path = runbook.join("specification.html");
    let source = std::fs::read_to_string(&path).expect("the specification reads");
    std::fs::write(
        &path,
        source.replace(
            "<section class=\"requirement\" id=\"req-second\" data-change=\"unchanged\">",
            "<section class=\"requirement\" id=\"req-second\" data-change=\"removed\">",
        ),
    )
    .expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let written =
        std::fs::read_to_string(records(&runbook).join("first.html")).expect("the record reads");
    assert!(
        !written.contains("req-second"),
        "the record copied a withdrawn requirement, which no reference may target"
    );
    assert!(
        written.contains("req-real"),
        "the record dropped the requirement that still applies"
    );

    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !out.contains("req-second"),
        "the report demands an outcome for an obligation that no longer applies:\n{out}"
    );
}

/// A runbook that states nothing produces a summary that does not count to zero.
#[test]
fn the_records_summary_does_not_describe_an_absent_section() {
    let scratch = Scratch::new("summary");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    // A specification stating nothing rather than no specification, because starting a run now needs
    // the document: see `a_runbook_stating_nothing_has_runs_nothing_checks`.
    strip_requirements(&runbook);

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "starting a run failed:\n{out}{err}");

    let written =
        std::fs::read_to_string(records(&runbook).join("first.html")).expect("the record reads");
    assert!(
        !written.contains(" 0 requirement"),
        "the record's summary counts to zero about a section that is not there, and that string is \
         the document's projected summary"
    );
    assert!(
        written.contains("states no requirement of its runs"),
        "the summary does not say why there is nothing to attest to"
    );
}

// -----------------------------------------------------------------------------
// Starting a run is gated on this machine.
//
// Driven through the built binary for the reason the rest of this file gives: the subject is a file
// appearing where it should and *not* appearing where it should not, and an exit status alone does
// not distinguish a refusal from a near miss.
// -----------------------------------------------------------------------------

/// Gives a fixture runbook one machine check that this machine satisfies, and one it does not.
///
/// Both are `state` checks, and that is deliberate: a check about a tool would depend on what the
/// machine running the suite happens to have installed, and `runbook-portability` requires the
/// opposite. `outside-project` is satisfiable from a scratch directory that is not a project, and
/// unsatisfiable from one that is — so the same declaration form produces both answers from a
/// directory the test controls.
fn declare_machine_checks(runbook: &Path, ids: &[&str]) {
    let path = runbook.join("specification.html");
    let source = std::fs::read_to_string(&path).expect("the specification reads");
    let mut units = String::new();
    for id in ids {
        units.push_str(&format!(
            "          <section class=\"requirement\" id=\"req-{id}\" data-change=\"unchanged\">\n\
             \x20           <h3>The {id} prerequisite <span class=\"hs-id\">req-{id}</span></h3>\n\
             \x20           <p>A run MUST satisfy the {id} condition before it starts.</p>\n\
             \x20           <article class=\"scenario\" id=\"scenario-{id}\" \
             data-change=\"unchanged\">\n\
             \x20             <h4>The {id} condition holds</h4>\n\
             \x20             <dl>\n\
             \x20               <dt>When</dt>\n\
             \x20               <dd>A run starts.</dd>\n\
             \x20               <dt>Then</dt>\n\
             \x20               <dd>The run records that it was checked.</dd>\n\
             \x20             </dl>\n\
             \x20           </article>\n\
             \x20           <p data-field=\"note\">preflight/v1 \
             {{&quot;kind&quot;:&quot;state&quot;,&quot;check&quot;:&quot;outside-project&quot;}}</p>\n\
             \x20         </section>\n",
        ));
    }
    let written = source.replace(
        "        </section>\n      </main>",
        &format!("{units}        </section>\n      </main>"),
    );
    assert_ne!(written, source, "the fixture has no requirements section");
    std::fs::write(&path, written).expect("rewritten");

    // Canonical, because a test that later asserts nothing was written would otherwise be measuring
    // the fixture's own indentation. `format` is run through the binary this file already drives.
    let project = runbook
        .ancestors()
        .nth(3)
        .expect("the project root")
        .to_owned();
    let (code, out, err) = run_in(&project, &["format", &path.to_string_lossy()]);
    assert_eq!(code, 0, "the declaration does not format:\n{out}{err}");
    let (code, out, err) = run_in(&project, &["validate", &path.to_string_lossy()]);
    assert_eq!(
        code, 0,
        "the fixture's declaration left a document that does not validate:\n{out}{err}"
    );
}

/// A run starts where every declared machine check passes, and the record carries the evidence.
///
/// **The load-bearing assertion is what the record says about the two kinds of requirement.** A
/// machine check that passed is recorded as passed with what was observed; a requirement with no
/// declaration stays `not-run`, because passing preflight does not discharge a logging obligation —
/// it records the outcome of one of them.
#[test]
fn a_run_starts_where_its_machine_checks_pass_and_records_the_evidence() {
    let scratch = Scratch::new("preflight-passes");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    declare_machine_checks(&runbook, &["machine"]);

    // The scratch directory holds a project, so `outside-project` would fail there. The run is
    // started from a sibling that is not inside one, which is the state the declaration asks for.
    let outside = scratch.0.join("outside");
    std::fs::create_dir_all(outside.join(".git")).expect("a repository boundary, not a project");

    // The registry is a scratch directory for every store test here, for the reason
    // `run_with_registry` gives: the real one lives under `XDG_CONFIG_HOME`, so a test using the
    // ambient environment would write into the config directory of whoever ran the suite, and would
    // fail outright in a build sandbox where `$HOME` is read-only.
    let config = scratch.0.join("config");
    std::fs::create_dir_all(&config).expect("the registry's home exists");
    let (code, out, err) = run_with_registry(
        &outside,
        &config,
        &["--store", "unused", "run", "release", "--as", "x"],
    );
    assert_ne!(code, 0, "an unregistered store was accepted:\n{out}{err}");

    // Registered so the runbook is reachable while the working directory stays outside the project.
    let (code, out, err) = run_with_registry(
        &project,
        &config,
        &["store", "add", "fixture", &project.to_string_lossy()],
    );
    assert_eq!(code, 0, "the store was not registered:\n{out}{err}");
    let (code, out, err) = run_with_registry(
        &outside,
        &config,
        &["--store", "fixture", "run", "release", "--as", "first"],
    );
    assert_eq!(
        code, 0,
        "a run whose every machine check passes was refused. The store holds the documents and the \
         working directory is outside a project:\n{out}{err}"
    );
    let _ = run_with_registry(&project, &config, &["store", "remove", "fixture"]);

    let record = records(&runbook).join("first.html");
    assert!(record.is_file(), "no record was written:\n{out}{err}");
    let written = std::fs::read_to_string(&record).expect("the record reads");

    // The passed machine check carries its observation, and the outcome says it passed.
    assert!(
        written.contains("data-outcome=\"passed\""),
        "the passed machine check was not recorded as passed:\n{written}"
    );
    assert!(
        written.contains("preflight release"),
        "the record does not say how the prerequisite was checked:\n{written}"
    );
    // A requirement with no declaration still owes its outcome.
    assert!(
        written.contains("data-outcome=\"not-run\""),
        "a logging requirement with no machine check was recorded as though it had been:\n{written}"
    );
    assert!(
        written.matches("data-outcome=\"passed\"").count() == 1,
        "more than the one declared machine check was recorded as passed:\n{written}"
    );

    // The release version is in the readable prose and not in the schema-version meta tag, which
    // identifies the document schema and would be read as something else by every consumer of it.
    assert!(
        written.contains("0.1.0 release"),
        "the record does not name the release it ran:\n{written}"
    );
    // The tag declares the contract version, which differs from the runbook's 0.1.0 release since
    // the contract reached 1.0.0, so this now tells the two apart. Until then both were 0.1.0.
    assert!(
        written.contains(&format!(
            "<meta content=\"{}\" name=\"htmlspec-version\">",
            htmlspec_schema::CONTRACT_VERSION
        )),
        "the schema version moved:\n{written}"
    );
    assert!(
        !written.contains("<meta content=\"0.1.0\" name=\"htmlspec-version\">"),
        "the runbook's release reached the schema-version tag:\n{written}"
    );

    // And the record is still a valid, canonical document, which is the invariant the copy exists
    // to preserve.
    let (code, out, err) = run_in(
        &project,
        &["validate", "--strict", &record.to_string_lossy()],
    );
    assert_eq!(code, 0, "the record does not validate:\n{out}{err}");
    let (code, out, err) = run_in(&project, &["format", "--check", &record.to_string_lossy()]);
    assert_eq!(code, 0, "the record is not canonical:\n{out}{err}");
}

/// A run does not start where a machine check fails, and nothing is created.
///
/// **`runs/` not being created is half the assertion.** `runbook-portability` requires that no record
/// *or new records directory* is created, and a directory left behind by a refused start is a place
/// the project did not have before with nothing in it.
#[test]
fn a_run_does_not_start_where_a_machine_check_fails() {
    let scratch = Scratch::new("preflight-fails");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    declare_machine_checks(&runbook, &["machine"]);
    assert!(
        !records(&runbook).exists(),
        "the fixture already has a records directory, so the assertion below proves nothing"
    );

    // Started from inside the project, which is exactly the state `outside-project` refuses: this is
    // the bootstrap hazard, and the refusal is what stops the procedure acting on a project the
    // operator did not choose.
    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 1, "an unmet prerequisite did not exit 1:\n{out}{err}");
    assert!(
        !records(&runbook).join("first.html").exists(),
        "a record was created for a run that was refused"
    );
    assert!(
        !records(&runbook).exists(),
        "a refused start left a records directory behind"
    );
    let reported = format!("{out}{err}");
    assert!(
        reported.contains("req-machine"),
        "the refusal does not name the requirement that failed:\n{reported}"
    );
    assert!(
        reported.contains("no record was created"),
        "the refusal does not say what was not done:\n{reported}"
    );
    assert!(
        reported.starts_with("not ready:"),
        "the refusal does not lead with readiness:\n{reported}"
    );
    // A remedy is named, and it is not an offer to install anything.
    assert!(
        reported.contains("remedy:"),
        "the refusal names no way forward:\n{reported}"
    );
    assert!(
        !reported.contains("installing it for you"),
        "the report claims a remediation this program does not perform:\n{reported}"
    );

    // The canonical procedure is untouched, which is the other half of "nothing was written".
    let (code, out, err) = run_in(&project, &["format", "--check", &runbook.to_string_lossy()]);
    assert_eq!(
        code, 0,
        "a refused start left the runbook non-canonical, so it wrote to it:\n{out}{err}"
    );
}

/// A malformed declaration is an operational failure, not an unmet prerequisite.
///
/// The distinction `toolchain-packaging` draws: exit 2 says this program could not read the contract,
/// exit 1 says it read it and the machine fell short. Folding them together would tell an author to
/// install something when what they have to do is fix a declaration.
#[test]
fn a_malformed_declaration_is_an_operational_failure_and_not_an_unmet_prerequisite() {
    let scratch = Scratch::new("preflight-malformed");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    let path = runbook.join("specification.html");
    let source = std::fs::read_to_string(&path).expect("the specification reads");
    std::fs::write(
        &path,
        source.replace(
            "        </section>\n      </main>",
            "          <section class=\"requirement\" id=\"req-broken\" data-change=\"unchanged\">\n\
             \x20           <h3>A broken declaration <span class=\"hs-id\">req-broken</span></h3>\n\
             \x20           <p>A run MUST have this checked.</p>\n\
             \x20           <article class=\"scenario\" id=\"scenario-broken\" \
             data-change=\"unchanged\">\n\
             \x20             <h4>The declaration</h4>\n\
             \x20             <dl>\n\
             \x20               <dt>When</dt>\n\
             \x20               <dd>A run starts.</dd>\n\
             \x20               <dt>Then</dt>\n\
             \x20               <dd>It is refused.</dd>\n\
             \x20             </dl>\n\
             \x20           </article>\n\
             \x20           <p data-field=\"note\">preflight/v1 \
             {&quot;kind&quot;:&quot;command&quot;,&quot;run&quot;:&quot;cargo build&quot;}</p>\n\
             \x20         </section>\n        </section>\n      </main>",
        ),
    )
    .expect("rewritten");

    for arguments in [
        vec!["run", "release", "--as", "first"],
        vec!["preflight", "release"],
    ] {
        let (code, out, err) = run_in(&project, &arguments);
        assert_eq!(
            code, 2,
            "{arguments:?} did not report a malformed declaration as an operational \
             failure:\n{out}{err}"
        );
        assert!(
            err.contains("req-broken"),
            "{arguments:?} does not name the requirement that declared it:\n{err}"
        );
        assert!(
            !records(&runbook).join("first.html").exists(),
            "{arguments:?} created a record for a contract it could not read"
        );
    }
}

/// A runbook with no version declaration refuses a new start and says how to give it one.
///
/// The migration case: a runbook written before this obligation has no `VERSION`, and an operator
/// meeting that has to be told what to write rather than being told that something is wrong.
#[test]
fn an_unversioned_runbook_refuses_a_new_start_with_migration_guidance() {
    let scratch = Scratch::new("unversioned");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    std::fs::remove_file(runbook.join("VERSION")).expect("removed");

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 2, "an unversioned runbook started a run:\n{out}{err}");
    assert!(
        err.contains("VERSION") && err.contains("CHANGELOG.md"),
        "the refusal does not say what to write:\n{err}"
    );
    assert!(
        !records(&runbook).join("first.html").exists(),
        "a record was written for a runbook that cannot name its release"
    );

    // And preflight refuses it the same way, because both operations need the version to report
    // which release they checked.
    let (code, out, err) = run_in(&project, &["preflight", "release"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(err.contains("VERSION"), "{err}");

    // Reviewing what the runbook's runs recorded is untouched: an older record stays readable on a
    // machine that has nothing and in a runbook that declares nothing.
    let (code, out, err) = run_in(&project, &["run", "release", "--check"]);
    assert_eq!(
        code, 0,
        "reviewing an unversioned runbook's runs was refused:\n{out}{err}"
    );
}

/// A changelog that does not head the declared version refuses a new start.
#[test]
fn a_version_its_changelog_does_not_head_refuses_a_new_start() {
    let scratch = Scratch::new("unheaded");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    std::fs::write(
        runbook.join("CHANGELOG.md"),
        "# Changelog\n\n## 0.9.0\n\nSome other release.\n",
    )
    .expect("rewritten");

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains("no heading naming it"),
        "the refusal does not say what is missing:\n{err}"
    );
    assert!(!records(&runbook).join("first.html").exists());
}

/// A runbook declaring no machine check starts, and says that nothing was established.
///
/// Zero checks is legal. What is not acceptable is a report that reads as though prerequisites were
/// established, which is why the wording is asserted rather than the exit code alone.
#[test]
fn a_runbook_declaring_no_machine_check_starts_and_claims_nothing() {
    let scratch = Scratch::new("no-checks");
    let (project, runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["preflight", "release"]);
    assert_eq!(
        code, 0,
        "a runbook declaring nothing was refused:\n{out}{err}"
    );
    assert!(
        out.starts_with("ready:"),
        "the report does not lead with readiness:\n{out}"
    );
    assert!(
        out.contains("declares no machine check"),
        "a zero-check report does not say that nothing was established:\n{out}"
    );

    let (code, out, err) = run_in(&project, &["run", "release", "--as", "first"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("0 of 0 check(s) passed before it started"),
        "the start report does not say how many checks passed:\n{out}"
    );
    assert!(records(&runbook).join("first.html").is_file());
}

/// Preflight writes nothing at all, whatever it concludes.
#[test]
fn preflight_writes_nothing_whatever_it_concludes() {
    let scratch = Scratch::new("preflight-read-only");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    declare_machine_checks(&runbook, &["machine"]);

    let before: Vec<(PathBuf, u64)> = walk(&project);
    // From inside the project, so the check fails and the report is the refusal path.
    let (code, out, err) = run_in(&project, &["preflight", "release"]);
    assert_eq!(code, 1, "{out}{err}");
    assert_eq!(
        before,
        walk(&project),
        "preflight changed something in the project while reporting that it could not start"
    );
}

/// Every file beneath `root` with its length, for a before-and-after comparison.
fn walk(root: &Path) -> Vec<(PathBuf, u64)> {
    let mut found = Vec::new();
    collect(root, &mut found);
    found.sort();
    found
}

fn collect(directory: &Path, into: &mut Vec<(PathBuf, u64)>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, into);
        } else {
            let length = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            into.push((path, length));
        }
    }
}

/// Preflight in JSON mode is one parseable report on standard output, with no prompt.
///
/// The scenario `toolchain-packaging` names: an agent inspecting readiness pipes standard output
/// straight into a parser, so a probe's own text landing there would produce malformed input rather
/// than an empty stream and a nonzero exit.
#[test]
fn preflight_in_json_mode_is_one_parseable_report() {
    let scratch = Scratch::new("preflight-json");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    declare_machine_checks(&runbook, &["machine", "another"]);

    // From inside the project, so both checks fail and the payload carries two findings.
    let (code, out, err) = run_in(&project, &["--json", "preflight", "release"]);
    assert_eq!(code, 1, "an unmet prerequisite did not exit 1:\n{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|error| {
        panic!("standard output is not one JSON document ({error}):\n{out}\nstderr:\n{err}")
    });
    assert_eq!(payload["ready"], serde_json::json!(false), "{payload}");
    assert_eq!(payload["declared"], serde_json::json!(2), "{payload}");
    assert_eq!(
        payload["runbook"],
        serde_json::json!("release"),
        "{payload}"
    );
    assert_eq!(payload["version"], serde_json::json!("0.1.0"), "{payload}");
    // Both locations, because `--store` makes them different places and a worktree check judges one.
    assert!(
        payload["source"]
            .as_str()
            .is_some_and(|source| source.contains("runbooks/release")),
        "{payload}"
    );
    assert!(payload["workdir"].as_str().is_some(), "{payload}");

    let checks = payload["checks"].as_array().expect("the checks");
    assert_eq!(checks.len(), 2, "{payload}");
    assert_eq!(
        checks
            .iter()
            .filter_map(|check| check["requirement"].as_str())
            .collect::<Vec<_>>(),
        vec!["req-machine", "req-another"],
        "the results do not follow document order: {payload}"
    );
    for check in checks {
        for field in ["requirement", "expected", "observed", "outcome", "remedy"] {
            assert!(
                check.get(field).is_some(),
                "a check carries no {field}: {payload}"
            );
        }
    }
    // Nothing was written while producing it.
    assert!(!records(&runbook).exists());
}

/// Preflight reports on the invocation's working directory and not on the store's.
///
/// The scenario the specs name: a stored runbook checked from a separate target checkout judges the
/// checkout, and the report identifies both places.
#[test]
fn preflight_checks_the_working_directory_and_not_the_store() {
    let scratch = Scratch::new("store-and-workdir");
    let (project, runbook) = a_project_with_a_runbook(&scratch);
    declare_machine_checks(&runbook, &["machine"]);

    let config = scratch.0.join("config");
    std::fs::create_dir_all(&config).expect("the registry's home exists");
    let (code, out, err) = run_with_registry(
        &project,
        &config,
        &["store", "add", "fixture", &project.to_string_lossy()],
    );
    assert_eq!(code, 0, "{out}{err}");

    // A target directory that is not inside a project: the check passes there and fails at the store.
    let target = scratch.0.join("target");
    std::fs::create_dir_all(target.join(".git")).expect("a repository boundary, not a project");
    let (code, out, err) = run_with_registry(
        &target,
        &config,
        &["--store", "fixture", "preflight", "release"],
    );
    assert_eq!(
        code, 0,
        "preflight judged the store's directory rather than the working one:\n{out}{err}"
    );
    let _ = run_with_registry(&project, &config, &["store", "remove", "fixture"]);
    let canonical_target = target.canonicalize().expect("the target resolves");
    let canonical_store = project.canonicalize().expect("the store resolves");
    assert!(
        out.contains(&canonical_target.display().to_string()),
        "the report does not name the working directory it checked:\n{out}"
    );
    assert!(
        out.contains(&canonical_store.display().to_string()),
        "the report does not name where the documents were read from:\n{out}"
    );
}

/// A runbook that is not there, and a name that is a path, are both refused before anything is read.
#[test]
fn preflight_refuses_a_name_that_is_not_a_runbook() {
    let scratch = Scratch::new("preflight-names");
    let (project, _runbook) = a_project_with_a_runbook(&scratch);

    let (code, out, err) = run_in(&project, &["preflight", "cookies"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains("no runbook named cookies") && err.contains("list --runbooks"),
        "the report does not say what to do instead:\n{err}"
    );

    let (code, out, err) = run_in(&project, &["preflight", "../escape"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains("is a path rather than a runbook name"),
        "a path was accepted as a name:\n{err}"
    );

    // `run` and its `--check` take the name through the same constructor, which refuses the shapes
    // of a path and nothing 1.0 ran: `a..b` is looked for, and reported as not there.
    for arguments in [
        &["run", "a/b", "--as", "first"][..],
        &["run", "../escape", "--check"],
    ] {
        let (code, out, err) = run_in(&project, arguments);
        assert_eq!(code, 2, "{arguments:?}: {out}{err}");
        assert!(
            err.contains("is a path rather than a runbook name"),
            "{arguments:?} accepted a path as a name:\n{err}"
        );
    }
    let (code, out, err) = run_in(&project, &["run", "a..b", "--check"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains("no runbook named a..b"),
        "a name 1.0 accepted was refused as a name:\n{err}"
    );
}

/// Runs the binary with the store registry pointed at a scratch directory.
///
/// **Every test that registers a store must use this.** The registry lives under `XDG_CONFIG_HOME`, so
/// a test that ran with the ambient environment would register entries in the config directory of
/// whoever ran the suite — and in a build sandbox, where `$HOME` is read-only, would fail outright
/// rather than test anything. Both outcomes are a test that does not mean the same thing on two
/// machines.
fn run_with_registry(directory: &Path, config: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .env("XDG_CONFIG_HOME", config)
        .env("HOME", config)
        .args(arguments)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    support::held(arguments, &stdout);
    (
        output.status.code().expect("the process exited normally"),
        stdout,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}
