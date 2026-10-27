//! A runbook that requires a finished, passed run of another runbook before its own run starts.
//!
//! The change `require-a-run-of-another-runbook`: a `preflight/v3` declaration of kind `run`, checked
//! by `htmlspec preflight` and by `htmlspec run --as`, which refuses an unmet one before anything is
//! created. Driven through the built binary, because what is under test is a record appearing or
//! not appearing, and the report an operator reads.
//!
//! The runbooks are made with `htmlspec new runbook`, so a template change reaches these tests, and
//! each one is generic: `backup` before `migrate`, and `create-account`, `provision-database`, and
//! `deploy-service` in a chain.

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

/// Runs the binary in `directory`, with a registry of its own under `config`.
///
/// The registry is a scratch directory for every test here, so no test writes into the
/// configuration directory of whoever runs the suite.
fn run_in(directory: &Path, config: &Path, arguments: &[&str]) -> (i32, String, String) {
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

/// A project, with its own registry directory.
struct Project {
    root: PathBuf,
    config: PathBuf,
}

impl Project {
    fn new(scratch: &Scratch, name: &str) -> Self {
        let root = scratch.0.join(name);
        let config = scratch.0.join(format!("{name}-config"));
        std::fs::create_dir_all(&root).expect("the project directory exists");
        std::fs::create_dir_all(&config).expect("the registry's home exists");
        let project = Self { root, config };
        let (code, out, err) = project.run(&["init", "--tools", "none"]);
        assert_eq!(code, 0, "init failed:\n{out}{err}");
        project
    }

    fn run(&self, arguments: &[&str]) -> (i32, String, String) {
        run_in(&self.root, &self.config, arguments)
    }

    fn runbook(&self, name: &str) -> PathBuf {
        self.root.join("htmlspec").join("runbooks").join(name)
    }

    /// A runbook made by `new runbook`, whose specification declares a run check of each
    /// `(runbook, atLeast)` in `requires`, written as `notes` exactly.
    fn add_runbook(&self, name: &str, requires: &[&str]) {
        let (code, out, err) = self.run(&["new", "runbook", name]);
        assert_eq!(code, 0, "new runbook {name} failed:\n{out}{err}");
        for (index, declaration) in requires.iter().enumerate() {
            self.declare(name, &format!("after-{index}"), declaration);
        }
    }

    /// Adds a requirement holding the note `preflight/v3 {declaration}` to `runbook`'s
    /// specification, formatted canonically.
    fn declare(&self, runbook: &str, id: &str, declaration: &str) {
        let path = self.runbook(runbook).join("specification.html");
        let source = std::fs::read_to_string(&path).expect("the specification reads");
        let unit = format!(
            "          <section class=\"requirement\" id=\"req-{id}\" data-change=\"unchanged\">\n\
             \x20           <h3>The {id} prerequisite <span class=\"hs-id\">req-{id}</span></h3>\n\
             \x20           <p>A run MUST follow the run this declares.</p>\n\
             \x20           <article class=\"scenario\" id=\"scenario-{id}\" \
             data-change=\"unchanged\">\n\
             \x20             <h4>The prerequisite ran</h4>\n\
             \x20             <dl>\n\
             \x20               <dt>When</dt>\n\
             \x20               <dd>A run starts.</dd>\n\
             \x20               <dt>Then</dt>\n\
             \x20               <dd>The run it requires has finished and passed.</dd>\n\
             \x20             </dl>\n\
             \x20           </article>\n\
             \x20           <p data-field=\"note\">preflight/v3 {}</p>\n\
             \x20         </section>\n",
            declaration.replace('"', "&quot;")
        );
        let written = source.replace(
            "        </section>\n      </main>",
            &format!("{unit}        </section>\n      </main>"),
        );
        assert_ne!(
            written, source,
            "the specification has no requirements section"
        );
        std::fs::write(&path, written).expect("rewritten");
        let (code, out, err) = self.run(&["format", &path.to_string_lossy()]);
        assert_eq!(code, 0, "the declaration does not format:\n{out}{err}");
    }

    /// Starts a run of `runbook` under `record`, returning the exit code and both streams.
    fn start(&self, runbook: &str, record: &str) -> (i32, String, String) {
        self.run(&["run", runbook, "--as", record])
    }

    fn record(&self, runbook: &str, record: &str) -> PathBuf {
        self.runbook(runbook)
            .join("runs")
            .join(format!("{record}.html"))
    }

    /// Settles every task of a record as done and every verification as passed: a finished run
    /// that passed.
    fn finish(&self, runbook: &str, record: &str) {
        let path = self.record(runbook, record);
        let source = std::fs::read_to_string(&path).expect("the record reads");
        let finished = source
            .replace("data-state=\"todo\"", "data-state=\"done\"")
            .replace("data-state=\"in-progress\"", "data-state=\"done\"")
            .replace("data-outcome=\"not-run\"", "data-outcome=\"passed\"");
        std::fs::write(&path, finished).expect("the record is finished");
    }

    /// Settles the record's first task as done and leaves the rest open.
    fn advance(&self, runbook: &str, record: &str) {
        let path = self.record(runbook, record);
        let source = std::fs::read_to_string(&path).expect("the record reads");
        let advanced = source.replacen("data-state=\"todo\"", "data-state=\"done\"", 1);
        std::fs::write(&path, advanced).expect("the record is advanced");
    }
}

/// The declaration requiring a passed run of `runbook`.
fn requires(runbook: &str) -> String {
    format!("{{\"kind\":\"run\",\"runbook\":\"{runbook}\"}}")
}

/// The JSON payload's check entries.
fn checks(payload: &str) -> Vec<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_str(payload).expect("the payload is JSON");
    value["checks"].as_array().cloned().unwrap_or_default()
}

/// Migrate waits for a finished, passed backup: refused with no record, refused with an open one,
/// and started after a finished one, with the new record naming the backup that satisfied it.
#[test]
fn migrate_starts_only_after_a_finished_passed_backup() {
    let scratch = Scratch::new("backup-migrate");
    let project = Project::new(&scratch, "project");
    project.add_runbook("backup", &[]);
    project.add_runbook("migrate", &[&requires("backup")]);

    // No backup run on record.
    let (code, out, err) = project.start("migrate", "m1");
    assert_eq!(code, 1, "migrate started with no backup run:\n{out}{err}");
    assert!(
        out.contains("`backup` has no runs on record"),
        "the refusal does not say why:\n{out}"
    );
    assert!(
        !project.runbook("migrate").join("runs").exists(),
        "a refused start created the runs directory"
    );

    // A backup run started and not finished.
    let (code, out, err) = project.start("backup", "b1");
    assert_eq!(code, 0, "backup did not start:\n{out}{err}");
    let (code, out, _) = project.start("migrate", "m1");
    assert_eq!(code, 1, "migrate started beside an open backup:\n{out}");
    assert!(out.contains("b1 has no task done"), "{out}");
    project.advance("backup", "b1");
    let (code, out, _) = project.start("migrate", "m1");
    assert_eq!(code, 1, "migrate started beside an open backup:\n{out}");
    assert!(out.contains("b1 is still open, with 1 of"), "{out}");
    assert!(out.contains("Still under way: b1 (1 of"), "{out}");
    assert!(!project.record("migrate", "m1").exists());

    // The same report from preflight, and in JSON.
    let (code, out, _) = project.run(&["--json", "preflight", "migrate"]);
    assert_eq!(code, 1, "{out}");
    let run = checks(&out)
        .into_iter()
        .find_map(|check| check.get("run").cloned())
        .expect("the run check carries its run object");
    assert_eq!(run["state"], "unmet");
    assert_eq!(run["openRuns"], serde_json::json!(["b1"]));
    assert_eq!(run["notSatisfying"][0]["record"], "b1");

    // Finished and passed: migrate starts, and its record names the backup run and its release.
    project.finish("backup", "b1");
    let (code, out, err) = project.start("migrate", "m1");
    assert_eq!(
        code, 0,
        "migrate did not start after a passed backup:\n{out}{err}"
    );
    let record = project.record("migrate", "m1");
    let written = std::fs::read_to_string(&record).expect("the record reads");
    assert!(
        written.contains(
            "1 finished, passed run(s) of `backup` meet it: b1 (0.1.0). Repeat it with that \
             command, which reads this project's runs afresh"
        ),
        "the record does not name the backup that satisfied it:\n{written}"
    );
    assert!(
        written.contains("<meta content=\"0.1.0\" name=\"htmlspec-runbook-release\">"),
        "the record does not state its release in canonical form:\n{written}"
    );
    let (code, out, err) = project.run(&["validate", "--strict", &record.to_string_lossy()]);
    assert_eq!(code, 0, "the record does not validate:\n{out}{err}");
    let (code, out, err) = project.run(&["format", "--check", &record.to_string_lossy()]);
    assert_eq!(code, 0, "the record is not canonical:\n{out}{err}");

    let (code, out, _) = project.run(&["--json", "preflight", "migrate"]);
    assert_eq!(code, 0, "{out}");
    let run = checks(&out)
        .into_iter()
        .find_map(|check| check.get("run").cloned())
        .expect("the run check carries its run object");
    assert_eq!(run["state"], "met");
    assert_eq!(run["satisfiedBy"], serde_json::json!(["b1"]));
    assert_eq!(run["satisfying"], 1);
}

/// A record with a failed outcome does not satisfy it, and the refusal names the requirement.
#[test]
fn a_backup_that_recorded_a_failure_does_not_satisfy_it() {
    let scratch = Scratch::new("failed");
    let project = Project::new(&scratch, "project");
    project.add_runbook("backup", &[]);
    project.add_runbook("migrate", &[&requires("backup")]);
    let (code, out, err) = project.start("backup", "b1");
    assert_eq!(code, 0, "{out}{err}");
    project.finish("backup", "b1");
    let path = project.record("backup", "b1");
    let source = std::fs::read_to_string(&path).expect("reads");
    std::fs::write(
        &path,
        source.replacen("data-outcome=\"passed\"", "data-outcome=\"failed\"", 1),
    )
    .expect("written");
    let (code, out, _) = project.start("migrate", "m1");
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("b1 recorded failed for req-"), "{out}");
}

/// `atLeast` compares the release a record states, within one major.
#[test]
fn at_least_refuses_a_run_of_an_older_release() {
    let scratch = Scratch::new("at-least");
    let project = Project::new(&scratch, "project");
    project.add_runbook("backup", &[]);
    project.add_runbook(
        "migrate",
        &["{\"kind\":\"run\",\"runbook\":\"backup\",\"atLeast\":\"0.2.0\"}"],
    );
    let (code, out, err) = project.start("backup", "b1");
    assert_eq!(code, 0, "{out}{err}");
    project.finish("backup", "b1");
    let (code, out, _) = project.start("migrate", "m1");
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("b1 ran release 0.1.0, below 0.2.0"), "{out}");
}

/// Three runbooks in a chain: each declares only its own prerequisite, and each start waits for
/// the one before it.
#[test]
fn a_chain_of_three_starts_in_order() {
    let scratch = Scratch::new("chain");
    let project = Project::new(&scratch, "project");
    project.add_runbook("create-account", &[]);
    project.add_runbook("provision-database", &[&requires("create-account")]);
    project.add_runbook("deploy-service", &[&requires("provision-database")]);

    let (code, out, _) = project.start("deploy-service", "d1");
    assert_eq!(code, 1, "{out}");
    let (code, out, _) = project.start("provision-database", "p1");
    assert_eq!(code, 1, "{out}");

    let (code, out, err) = project.start("create-account", "a1");
    assert_eq!(code, 0, "{out}{err}");
    project.finish("create-account", "a1");
    let (code, out, err) = project.start("provision-database", "p1");
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, _) = project.start("deploy-service", "d1");
    assert_eq!(
        code, 1,
        "deploy-service started before provision-database finished:\n{out}"
    );
    project.finish("provision-database", "p1");
    let (code, out, err) = project.start("deploy-service", "d1");
    assert_eq!(code, 0, "{out}{err}");
}

/// A self-reference, and a cycle of two and of three through the runbook, are invalid contracts:
/// exit 2, the requirement and the cycle named in order, and nothing created.
#[test]
fn a_self_reference_or_a_cycle_through_the_runbook_is_an_invalid_contract() {
    let scratch = Scratch::new("cycles");
    let project = Project::new(&scratch, "project");
    project.add_runbook("migrate", &[&requires("migrate")]);
    let (code, out, err) = project.start("migrate", "m1");
    assert_eq!(
        code, 2,
        "a self-reference was not refused as invalid:\n{out}{err}"
    );
    assert!(err.contains("req-after-0"), "{err}");
    assert!(err.contains("the runbook that declares it"), "{err}");
    assert!(out.is_empty(), "an invalid contract wrote a report:\n{out}");
    assert!(!project.runbook("migrate").join("runs").exists());

    // Two runbooks requiring each other.
    project.add_runbook("left", &[&requires("right")]);
    project.add_runbook("right", &[&requires("left")]);
    let (code, _, err) = project.run(&["preflight", "left"]);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("`left` requires a run of `right` requires a run of `left`"),
        "{err}"
    );

    // Three, through the runbook.
    project.add_runbook("one", &[&requires("two")]);
    project.add_runbook("two", &[&requires("three")]);
    project.add_runbook("three", &[&requires("one")]);
    let (code, _, err) = project.start("two", "t1");
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains(
            "`two` requires a run of `three` requires a run of `one` requires a run of `two`"
        ),
        "{err}"
    );
}

/// A cycle elsewhere in the project, which the runbook does not reach, is not its fault.
#[test]
fn a_cycle_elsewhere_does_not_refuse_a_runbook_outside_it() {
    let scratch = Scratch::new("cycle-elsewhere");
    let project = Project::new(&scratch, "project");
    project.add_runbook("left", &[&requires("right")]);
    project.add_runbook("right", &[&requires("left")]);
    project.add_runbook("backup", &[]);
    project.add_runbook("migrate", &[&requires("backup")]);
    let (code, out, err) = project.run(&["preflight", "migrate"]);
    assert_eq!(code, 1, "the cycle elsewhere refused migrate:\n{out}{err}");
}

/// A specification reached on the way that cannot be read makes the check unavailable, naming it.
#[test]
fn an_unreadable_specification_on_the_way_makes_the_check_unavailable() {
    let scratch = Scratch::new("unreadable");
    let project = Project::new(&scratch, "project");
    project.add_runbook("helper", &[]);
    project.add_runbook("backup", &[&requires("helper")]);
    project.add_runbook("migrate", &[&requires("backup")]);
    std::fs::write(
        project.runbook("helper").join("specification.html"),
        "<p>not a specification",
    )
    .expect("broken");
    let (code, out, _) = project.run(&["--json", "preflight", "migrate"]);
    assert_eq!(code, 1, "{out}");
    let check = checks(&out)
        .into_iter()
        .find(|check| check.get("run").is_some())
        .expect("the run check is reported");
    assert_eq!(check["outcome"], "unavailable");
    assert_eq!(check["run"]["state"], "unavailable");
    // Running `backup` again would not help, so the remedy is the observation's own.
    assert_eq!(
        check["remedy"],
        "make the specification of `helper` read and validate: run `htmlspec validate` on it"
    );
    assert!(
        check["observed"]
            .as_str()
            .unwrap_or_default()
            .contains("`helper`, reached through `backup`'s run checks"),
        "{check}"
    );
}

/// A name matches a runbook byte for byte on every filesystem: `Backup` beside a `backup`
/// directory names no runbook, and a self-reference spelled in another case is missing, not a
/// self-reference.
#[test]
fn a_name_matches_byte_for_byte_whatever_the_filesystem() {
    let scratch = Scratch::new("case");
    let project = Project::new(&scratch, "project");
    project.add_runbook("backup", &[]);
    project.add_runbook("migrate", &[&requires("Backup"), &requires("MIGRATE")]);
    let (code, out, err) = project.run(&["preflight", "migrate"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(
        out.contains("this project has no runbook named `Backup`"),
        "{out}"
    );
    assert!(
        out.contains("this project has no runbook named `MIGRATE`"),
        "{out}"
    );
    // A missing runbook cannot be run to completion, so its remedy says to add it or correct the
    // name.
    assert!(
        out.contains("remedy: add a runbook named `Backup` to this project"),
        "{out}"
    );
}

/// A runbook named on the command line in another case, which a filesystem that folds case finds,
/// is known by its directory's name: a self-reference or a cycle through it is refused exactly as
/// under its own spelling, and a run it starts is recorded under that name.
///
/// Skipped where the filesystem tells names apart by case, because there the other spelling reaches
/// no directory and the command line's lookup already refuses it.
#[test]
fn a_runbook_typed_in_another_case_is_known_by_its_directory_name() {
    let scratch = Scratch::new("typed-case");
    let project = Project::new(&scratch, "project");
    project.add_runbook("migrate", &[&requires("migrate")]);
    if !project.runbook("MIGRATE").is_dir() {
        eprintln!("skipped: this filesystem tells names apart by case");
        return;
    }

    let (code, out, err) = project.run(&["preflight", "MIGRATE"]);
    assert_eq!(
        code, 2,
        "a self-reference typed in another case was not refused:\n{out}{err}"
    );
    assert!(
        err.contains("the run check names `migrate`, the runbook that declares it"),
        "{err}"
    );

    project.add_runbook("one", &[&requires("two")]);
    project.add_runbook("two", &[&requires("three")]);
    project.add_runbook("three", &[&requires("one")]);
    let (code, out, err) = project.start("TWO", "t1");
    assert_eq!(
        code, 2,
        "a cycle typed in another case was not refused:\n{out}{err}"
    );
    assert!(
        err.contains(
            "`two` requires a run of `three` requires a run of `one` requires a run of `two`"
        ),
        "{err}"
    );
    assert!(!project.runbook("two").join("runs").exists());

    project.add_runbook("backup", &[]);
    let (code, out, err) = project.run(&["--json", "run", "BACKUP", "--as", "b1"]);
    assert_eq!(code, 0, "{out}{err}");
    let payload: serde_json::Value = serde_json::from_str(&out).expect("the payload is JSON");
    assert_eq!(payload["runbook"], "backup");
    assert!(project.record("backup", "b1").is_file());
}

/// A record under `runs/` that is a symbolic link to another project's passed record does not
/// satisfy a run check: only this project's runs count. It is named with its own reason, and once a
/// real run passes here, the check is met.
#[cfg(unix)]
#[test]
fn a_linked_record_from_another_project_does_not_satisfy_it() {
    let scratch = Scratch::new("linked-record");
    let elsewhere = Project::new(&scratch, "elsewhere");
    elsewhere.add_runbook("backup", &[]);
    let (code, out, err) = elsewhere.start("backup", "b1");
    assert_eq!(code, 0, "{out}{err}");
    elsewhere.finish("backup", "b1");

    let project = Project::new(&scratch, "project");
    project.add_runbook("backup", &[]);
    project.add_runbook("migrate", &[&requires("backup")]);
    let runs = project.runbook("backup").join("runs");
    std::fs::create_dir_all(&runs).expect("the runs folder");
    std::os::unix::fs::symlink(elsewhere.record("backup", "b1"), runs.join("borrowed.html"))
        .expect("a link to another project's record");

    let (code, out, err) = project.run(&["preflight", "migrate"]);
    assert_eq!(code, 1, "a linked record satisfied the check:\n{out}{err}");
    assert!(out.contains("borrowed is a link"), "{out}");

    let (code, out, err) = project.start("backup", "b1");
    assert_eq!(code, 0, "{out}{err}");
    project.finish("backup", "b1");
    let (code, out, err) = project.run(&["preflight", "migrate"]);
    assert_eq!(code, 0, "a passed run here did not meet it:\n{out}{err}");
}

/// Every name 1.1.1 ran still runs. A runbook with a symbolic link beside it is listed under two
/// names for one directory, and 1.1.1 started it under its own name, the link's, and either one in
/// another case. Each still starts, known by the name it folds to.
#[cfg(unix)]
#[test]
fn a_runbook_with_a_link_beside_it_still_runs_under_every_name_1_1_ran() {
    let scratch = Scratch::new("linked-names");
    let project = Project::new(&scratch, "project");
    project.add_runbook("outside", &[]);
    std::os::unix::fs::symlink("outside", project.runbook("outside-link")).expect("a link");
    let folds = project.runbook("OUTSIDE").is_dir();
    let names: &[&str] = if folds {
        &["outside", "outside-link", "OUTSIDE", "Outside-Link"]
    } else {
        &["outside", "outside-link"]
    };
    for typed in names {
        let (code, out, err) = project.run(&["preflight", typed]);
        assert_eq!(code, 0, "{typed} no longer runs:\n{out}{err}");
    }
    if folds {
        let (_, out, _) = project.run(&["preflight", "OUTSIDE"]);
        assert!(out.starts_with("ready: outside can start here"), "{out}");
    }
}

/// A report that is not ready still names a run under way beside a met run check: the check passed
/// on an older finished run, and the operator trying again should see the one in progress.
#[test]
fn a_report_that_is_not_ready_names_a_met_checks_open_run() {
    let scratch = Scratch::new("not-ready-open");
    let project = Project::new(&scratch, "project");
    project.add_runbook("backup", &[]);
    project.add_runbook(
        "migrate",
        &[
            &requires("backup"),
            "{\"kind\":\"tool\",\"program\":\"htmlspec-no-such-tool\",\"version\":\"1.0.0\"}",
        ],
    );
    let (code, out, err) = project.start("backup", "b1");
    assert_eq!(code, 0, "{out}{err}");
    project.finish("backup", "b1");
    let (code, out, err) = project.start("backup", "b2");
    assert_eq!(code, 0, "{out}{err}");

    let (code, out, err) = project.run(&["preflight", "migrate"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(
        out.contains("1 met run check(s) have a run still under way:"),
        "{out}"
    );
    assert!(
        out.contains("meet it: b1 (0.1.0). Still under way: b2 (0 of"),
        "{out}"
    );
}

/// A runbook name and a record name holding an escape sequence reach the report by code point.
#[test]
fn runbook_and_record_names_reach_the_report_escaped() {
    let scratch = Scratch::new("escapes");
    let project = Project::new(&scratch, "project");
    project.add_runbook("backup", &[]);
    project.add_runbook(
        "migrate",
        &[
            &requires("backup"),
            "{\"kind\":\"run\",\"runbook\":\"back\\u001b[2Jup\"}",
        ],
    );
    let runs = project.runbook("backup").join("runs");
    std::fs::create_dir_all(&runs).expect("the runs directory exists");
    std::fs::write(runs.join("odd\u{1b}[31m.html"), "").expect("a stray record");
    let (code, out, err) = project.run(&["preflight", "migrate"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(
        !out.contains('\u{1b}'),
        "a raw escape reached the report:\n{out:?}"
    );
    assert!(out.contains("back<U+001B>[2Jup"), "{out}");
    assert!(out.contains("odd<U+001B>[31m"), "{out}");
}

/// A real runbook directory and real records whose names hold an escape sequence and a
/// right-to-left override reach every report by code point: the check met, the run under way, the
/// header of the runbook's own preflight, and the method a new record writes.
///
/// The runs are made under plain names and renamed after: a record composed under a name holding a
/// control character would itself be refused, and a directory or a file can still be given any
/// name afterwards.
#[test]
fn a_real_runbook_and_records_named_with_escapes_reach_every_report_escaped() {
    let scratch = Scratch::new("escaped-names");
    let project = Project::new(&scratch, "project");
    project.add_runbook("backup", &[]);
    let (code, out, err) = project.start("backup", "b1");
    assert_eq!(code, 0, "{out}{err}");
    project.finish("backup", "b1");
    let (code, out, err) = project.start("backup", "b2");
    assert_eq!(code, 0, "{out}{err}");

    let runbook = "back\u{1b}[2Jup\u{202e}";
    let runs = project.runbook("backup").join("runs");
    std::fs::rename(
        runs.join("b1.html"),
        runs.join("night\u{1b}[31mly\u{202e}.html"),
    )
    .expect("the finished record is renamed");
    std::fs::rename(
        runs.join("b2.html"),
        runs.join("open\u{1b}[1m\u{202e}.html"),
    )
    .expect("the open record is renamed");
    std::fs::rename(project.runbook("backup"), project.runbook(runbook))
        .expect("the runbook is renamed");
    project.add_runbook(
        "migrate",
        &["{\"kind\":\"run\",\"runbook\":\"back\\u001b[2Jup\\u202e\"}"],
    );

    let raw = |text: &str| text.contains('\u{1b}') || text.contains('\u{202e}');
    let (code, out, err) = project.run(&["preflight", "migrate"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !raw(&out) && !raw(&err),
        "a raw character reached the report:\n{out:?}{err:?}"
    );
    assert!(
        out.contains(
            "of `back<U+001B>[2Jup<U+202E>` meet it: night<U+001B>[31mly<U+202E> (0.1.0). \
             Still under way: open<U+001B>[1m<U+202E> (0 of"
        ),
        "{out}"
    );

    let (code, out, err) = project.run(&["preflight", runbook]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(!raw(&out) && !raw(&err), "{out:?}{err:?}");
    assert!(
        out.starts_with("ready: back<U+001B>[2Jup<U+202E> can start here"),
        "{out}"
    );

    let (code, out, err) = project.start("migrate", "m1");
    assert_eq!(code, 0, "{out}{err}");
    assert!(!raw(&out) && !raw(&err), "{out:?}{err:?}");
    let record = project.record("migrate", "m1");
    let written = std::fs::read_to_string(&record).expect("the record reads");
    assert!(!raw(&written), "a raw character reached the record");
    assert!(
        written.contains("meet it: night&lt;U+001B&gt;[31mly&lt;U+202E&gt; (0.1.0)"),
        "{written}"
    );
    let (code, out, err) = project.run(&["validate", "--strict", &record.to_string_lossy()]);
    assert_eq!(code, 0, "{out}{err}");
}

/// Starting a run of a runbook whose directory name holds an escape sequence composes a record the
/// formatter refuses, because the name reaches its markup. The refusal quotes the parser's
/// recoveries, and each quotes the character it stopped at, so the recovery's message names it by
/// its code point instead of writing it to the terminal. Before 1.2.0 the message wrote it raw.
#[test]
fn a_parse_recovery_quoting_a_names_escape_reaches_the_terminal_escaped() {
    let scratch = Scratch::new("recovery-escaped");
    let project = Project::new(&scratch, "project");
    project.add_runbook("plain", &[]);
    let runbook = "cy\u{1b}[31mc";
    std::fs::rename(project.runbook("plain"), project.runbook(runbook))
        .expect("the runbook is renamed");

    let (code, out, err) = project.start(runbook, "first");
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        !out.contains('\u{1b}') && !err.contains('\u{1b}'),
        "a raw escape reached the terminal:\n{out:?}{err:?}"
    );
    assert!(
        err.contains("parse/bad-character: Bad character <U+001B>"),
        "{err}"
    );
}

/// A carried runbook's prerequisite is checked in the receiving project: unmet where that project
/// has no backup, unmet where it has no passed run, and met once it has one. A carried runbook with
/// no run check still starts in a project with no records.
#[test]
fn a_carried_runbook_is_checked_against_the_receiving_project() {
    let scratch = Scratch::new("carried");
    let source = Project::new(&scratch, "source");
    source.add_runbook("migrate", &[&requires("backup")]);
    source.add_runbook("backup", &[]);
    let (code, out, err) = source.start("backup", "b1");
    assert_eq!(code, 0, "{out}{err}");
    source.finish("backup", "b1");

    // The receiving project holds a copy of migrate, and no backup.
    let receiving = Project::new(&scratch, "receiving");
    copy(&source.runbook("migrate"), &receiving.runbook("migrate"));
    let elsewhere = scratch.0.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("a working directory");
    let store = |arguments: &[&str]| run_in(&elsewhere, &receiving.config, arguments);
    let (code, out, err) = store(&[
        "store",
        "add",
        "receiving",
        &receiving.root.to_string_lossy(),
    ]);
    assert_eq!(code, 0, "{out}{err}");

    let (code, out, err) = store(&["--store", "receiving", "preflight", "migrate"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(
        out.contains("this project has no runbook named `backup`"),
        "{out}"
    );

    // A carried backup with no run check starts in the receiving project, which has no records.
    copy(&source.runbook("backup"), &receiving.runbook("backup"));
    let _ = std::fs::remove_dir_all(receiving.runbook("backup").join("runs"));
    let (code, out, err) = store(&["--store", "receiving", "preflight", "migrate"]);
    assert_eq!(
        code, 1,
        "the source project's run counted elsewhere:\n{out}{err}"
    );
    assert!(out.contains("`backup` has no runs on record"), "{out}");
    let (code, out, err) = store(&["--store", "receiving", "run", "backup", "--as", "here"]);
    assert_eq!(code, 0, "{out}{err}");
    receiving.finish("backup", "here");
    let (code, out, err) = store(&["--store", "receiving", "preflight", "migrate"]);
    assert_eq!(code, 0, "{out}{err}");
    let _ = store(&["store", "remove", "receiving"]);
}

/// Copies a runbook's directory, documents and records alike.
fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the destination exists");
    for entry in std::fs::read_dir(from).expect("the source reads").flatten() {
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copied");
        }
    }
}
