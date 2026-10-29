//! How far each run of a runbook has come: `htmlspec run <runbook> --status [<record>]`.
//!
//! The change `show-how-far-a-run-has-come`. Driven through the built binary, because what is under
//! test is the report an operator reads and the payload a script reads, and that the report agrees
//! with preflight about every record. Every payload goes through `support::held`, so it is held to
//! `run`'s published schema.
//!
//! The runbooks are made with `htmlspec new runbook`, whose procedure has two steps and whose
//! specification states one requirement, `req-example`, and the runs with `htmlspec run --as`. A
//! record a test needs in a shape no run reaches is written directly.

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
            std::env::temp_dir().join(format!("htmlspec-progress-{label}-{}", std::process::id()));
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

/// A project, with a registry directory of its own so no test writes into the configuration of
/// whoever runs the suite.
struct Project {
    root: PathBuf,
    config: PathBuf,
}

impl Project {
    fn new(scratch: &Scratch) -> Self {
        let root = scratch.0.join("project");
        let config = scratch.0.join("config");
        std::fs::create_dir_all(&root).expect("the project directory exists");
        std::fs::create_dir_all(&config).expect("the registry's home exists");
        let project = Self { root, config };
        project.ok(&["init", "--tools", "none"]);
        project
    }

    /// Runs the binary in the project, holding any payload to its schema.
    fn run(&self, arguments: &[&str]) -> (i32, String, String) {
        let output = Command::new(binary())
            .current_dir(&self.root)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("HOME", &self.config)
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

    /// Runs an operation that must succeed, and returns its standard output.
    fn ok(&self, arguments: &[&str]) -> String {
        let (code, out, err) = self.run(arguments);
        assert_eq!(code, 0, "{arguments:?} failed:\n{out}{err}");
        out
    }

    /// The payload of `arguments` under `--json`.
    fn payload(&self, arguments: &[&str]) -> serde_json::Value {
        let mut structured = vec!["--json"];
        structured.extend_from_slice(arguments);
        serde_json::from_str(&self.ok(&structured)).expect("the payload is JSON")
    }

    fn runbook(&self, name: &str) -> PathBuf {
        self.root.join("htmlspec").join("runbooks").join(name)
    }

    fn runs(&self, runbook: &str) -> PathBuf {
        self.runbook(runbook).join("runs")
    }

    fn record(&self, runbook: &str, record: &str) -> PathBuf {
        self.runs(runbook).join(format!("{record}.html"))
    }

    /// Starts a run of `runbook` as `record`, and rewrites its record with `edit`.
    fn started(&self, runbook: &str, record: &str, edit: impl FnOnce(String) -> String) {
        self.ok(&["run", runbook, "--as", record]);
        let path = self.record(runbook, record);
        let source = std::fs::read_to_string(&path).expect("the record reads");
        std::fs::write(&path, edit(source)).expect("the record is rewritten");
    }

    /// Writes `source` under `runbook`'s runs as `file`.
    fn write(&self, runbook: &str, file: &str, source: &str) {
        let runs = self.runs(runbook);
        std::fs::create_dir_all(&runs).expect("the runs folder");
        std::fs::write(runs.join(file), source).expect("written");
    }
}

/// The first `count` steps of a record set to `state`.
fn steps(source: &str, state: &str, count: usize) -> String {
    source.replacen(
        "data-state=\"todo\"",
        &format!("data-state=\"{state}\""),
        count,
    )
}

/// A record's one outcome set to `outcome`.
fn outcome(source: &str, outcome: &str) -> String {
    source.replace(
        "data-outcome=\"not-run\"",
        &format!("data-outcome=\"{outcome}\""),
    )
}

/// A record whose first step is dropped, with `reason` where one is given.
fn dropped_first(source: &str, reason: Option<&str>) -> String {
    let mut source = steps(source, "dropped", 1);
    if let Some(reason) = reason {
        let step = source
            .find("data-state=\"dropped\"")
            .expect("a dropped step");
        let heading = step + source[step..].find("</h3>").expect("its heading") + "</h3>".len();
        source.insert_str(heading, &format!("<p data-field=\"reason\">{reason}</p>"));
    }
    source
}

/// A tasks document holding a step in each of `states`, and nothing else, which no run writes.
fn handmade(states: &[&str]) -> String {
    let tasks: String = states
        .iter()
        .enumerate()
        .map(|(index, state)| {
            format!(
                "<li class=\"task\" id=\"task-{index}\" data-change=\"unchanged\" \
                 data-state=\"{state}\"><h3>Step {index} <span class=\"hs-id\">task-{index}</span>\
                 </h3></li>"
            )
        })
        .collect();
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta content=\"1.0.0\" name=\"htmlspec-version\">\
         <meta content=\"tasks\" name=\"htmlspec-kind\"><title>A run</title></head>\
         <body><main><ul>{tasks}</ul></main></body></html>"
    )
}

/// Twelve records of `backup`, one of each kind a run can leave: open in four ways, passed twice,
/// failed, every step dropped, a mix of every state, no steps, an empty file, and a record whose
/// markup needs recovery.
fn every_kind_of_run(project: &Project) {
    project.started("backup", "a-started", |source| source);
    project.started("backup", "b-in-progress", |source| {
        steps(&source, "in-progress", 1)
    });
    project.started("backup", "c-both-in-progress", |source| {
        steps(&source, "in-progress", 2)
    });
    project.started("backup", "d-half-done", |source| steps(&source, "done", 1));
    project.started("backup", "e-passed", |source| {
        outcome(&steps(&source, "done", 2), "passed")
    });
    project.started("backup", "f-dropped", |source| {
        outcome(
            &steps(&dropped_first(&source, Some("Not needed.")), "done", 1),
            "passed",
        )
    });
    project.started("backup", "g-failed", |source| {
        outcome(&steps(&source, "done", 2), "failed")
    });
    project.started("backup", "h-all-dropped", |source| {
        outcome(&steps(&source, "dropped", 2), "passed")
    });
    project.write(
        "backup",
        "i-mixed.html",
        &handmade(&["done", "in-progress", "todo", "dropped", "sideways"]),
    );
    project.write("backup", "j-no-steps.html", &handmade(&[]));
    project.write("backup", "k-empty.html", "");
    project.write(
        "backup",
        "l-recovering.html",
        "<p>an <b>unclosed paragraph</p>",
    );
}

/// A record with a verification beside the others that references no requirement, and failed.
fn stray_verification(source: &str) -> String {
    let opening = source
        .find("<section class=\"verification\"")
        .expect("a verification");
    let after =
        opening + source[opening..].find("</section>").expect("it closes") + "</section>".len();
    format!(
        "{}<section class=\"verification\" id=\"verify-extra\" data-change=\"unchanged\" \
         data-outcome=\"failed\"><h3>Also checked</h3><p data-field=\"method\">Looked.</p>\
         </section>{}",
        &source[..after],
        &source[after..]
    )
}

/// A record whose one requirement is marked withdrawn, so it owes no outcome for it.
fn withdrawn_requirement(source: &str) -> String {
    let unit = source
        .find("<section class=\"requirement\"")
        .expect("a requirement");
    let change = unit
        + source[unit..]
            .find("data-change=\"unchanged\"")
            .expect("its change state");
    format!(
        "{}data-change=\"removed\"{}",
        &source[..change],
        &source[change + "data-change=\"unchanged\"".len()..]
    )
}

/// Every state a run can be in, one line each, in name order, and the same bytes every time.
#[test]
fn each_run_gets_one_line_in_name_order() {
    let scratch = Scratch::new("every-state");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);
    every_kind_of_run(&project);

    let (code, out, err) = project.run(&["run", "backup", "--status"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(
        out,
        "backup has 12 runs, in name order:\n\
         \x20 a-started           0/2 done, step 1 next\n\
         \x20 b-in-progress       0/2 done, step 1 in progress\n\
         \x20 c-both-in-progress  0/2 done, steps 1 and 2 in progress\n\
         \x20 d-half-done         1/2 done, step 2 next\n\
         \x20 e-passed            2/2 done, finished and passed\n\
         \x20 f-dropped           1/2 done, 1 dropped, finished and passed\n\
         \x20 g-failed            2/2 done, finished, not passed: recorded failed for \
         req-example\n\
         \x20 h-all-dropped       0/2 done, 2 dropped, finished, not passed: has no task done, \
         so it records no work\n\
         \x20 i-mixed             1/5 done, 1 dropped, step 2 in progress\n\
         \x20 j-no-steps          0/0 done, finished, not passed: has no task done, so it \
         records no work\n\
         \x20 k-empty             not read: is not a run's record: not a tasks document\n\
         \x20 l-recovering        not read: needs error recovery to parse, so it has more than \
         one reading and none is chosen\n"
    );
    assert!(
        err.contains(
            "To see one run's steps and outcomes: `htmlspec run backup --status <record>`"
        ),
        "{err}"
    );
    // The same project reads the same way every time.
    let (_, again, _) = project.run(&["run", "backup", "--status"]);
    assert_eq!(again, out);
    let first = project.ok(&["--json", "run", "backup", "--status"]);
    let second = project.ok(&["--json", "run", "backup", "--status"]);
    assert_eq!(first, second);

    let payload: serde_json::Value = serde_json::from_str(&first).expect("JSON");
    let states: Vec<(&str, &str)> = payload["records"]
        .as_array()
        .expect("records")
        .iter()
        .map(|record| {
            (
                record["record"].as_str().expect("a name"),
                record["state"].as_str().expect("a state"),
            )
        })
        .collect();
    assert_eq!(
        states,
        [
            ("a-started", "open"),
            ("b-in-progress", "open"),
            ("c-both-in-progress", "open"),
            ("d-half-done", "open"),
            ("e-passed", "passed"),
            ("f-dropped", "passed"),
            ("g-failed", "not-passed"),
            ("h-all-dropped", "not-passed"),
            ("i-mixed", "open"),
            ("j-no-steps", "not-passed"),
            ("k-empty", "unreadable"),
            ("l-recovering", "unreadable"),
        ]
    );
    assert_eq!(payload["runbook"], "backup");
}

/// One run: every step with its state, a dropped step's reason, each requirement's outcome, the
/// release the record states, and the verdict, in text and in the payload.
#[test]
fn one_run_shows_its_steps_outcomes_release_and_verdict() {
    let scratch = Scratch::new("one-run");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);
    project.started("backup", "b1", |source| {
        steps(
            &dropped_first(&source, Some("Not needed this time.")),
            "in-progress",
            1,
        )
    });
    project.started("backup", "b2", |source| {
        outcome(&steps(&dropped_first(&source, None), "done", 1), "passed")
    });

    let out = project.ok(&["run", "backup", "--status", "b1"]);
    assert_eq!(
        out,
        "backup run b1\n\
         Steps: 0 of 2 done, 1 dropped\n\
         \x20 1. dropped      The first step\n\
         \x20                 reason: Not needed this time.\n\
         \x20 2. in progress  The second step\n\
         Requirements: 1 of 1 have an outcome\n\
         \x20 not-run  req-example\n\
         Runbook release: 0.1.0\n\
         Finished and passed: no, it has no task done, so it records no work\n"
    );
    let out = project.ok(&["post", "run", "backup", "--status", "b2"]);
    assert!(
        out.contains("                 reason: none given\n"),
        "{out}"
    );
    assert!(out.ends_with("Finished and passed: yes\n"), "{out}");

    let payload = project.payload(&["run", "backup", "--status", "b1"]);
    assert_eq!(
        payload,
        serde_json::json!({
            "runbook": "backup",
            "records": [{
                "record": "b1",
                "state": "open",
                "reason": "has no task done, so it records no work",
                "done": 0,
                "dropped": 1,
                "total": 2,
                "steps": [
                    payload["records"][0]["steps"][0].clone(),
                    payload["records"][0]["steps"][1].clone(),
                ],
                "requirements": [{ "id": "req-example", "outcomes": ["not-run"] }],
                "otherOutcomes": [],
                "release": { "stated": "one", "version": "0.1.0" },
            }],
        })
    );
    let first = &payload["records"][0]["steps"][0];
    assert_eq!(first["number"], 1);
    assert_eq!(first["heading"], "The first step");
    assert_eq!(first["state"], "dropped");
    assert_eq!(first["reason"], "Not needed this time.");
    assert!(first["id"].is_string(), "{first}");
    assert_eq!(payload["records"][0]["steps"][1]["state"], "in-progress");
}

/// Each outcome a requirement can record is shown as recorded, and only `passed` passes.
#[test]
fn every_outcome_is_shown_as_recorded() {
    let scratch = Scratch::new("outcomes");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);
    for recorded in ["passed", "failed", "blocked", "partial", "not-run"] {
        project.started("backup", recorded, |source| {
            outcome(&steps(&source, "done", 2), recorded)
        });
        let out = project.ok(&["run", "backup", "--status", recorded]);
        assert!(
            out.contains(&format!("  {recorded}  req-example\n")),
            "{out}"
        );
        let verdict = if recorded == "passed" {
            String::from("Finished and passed: yes\n")
        } else {
            format!("Finished and passed: no, it recorded {recorded} for req-example\n")
        };
        assert!(out.ends_with(&verdict), "{out}");
    }
    // A requirement with no verification at all has no outcome.
    project.started("backup", "unrecorded", |source| {
        let opening = source
            .find("<section class=\"verification\"")
            .expect("a verification");
        let closing =
            opening + source[opening..].find("</section>").expect("it closes") + "</section>".len();
        steps(
            &format!("{}{}", &source[..opening], &source[closing..]),
            "done",
            2,
        )
    });
    let out = project.ok(&["run", "backup", "--status", "unrecorded"]);
    assert!(
        out.contains("Requirements: 0 of 1 have an outcome\n  no outcome  req-example\n"),
        "{out}"
    );
    assert!(
        out.ends_with("Finished and passed: no, it records no outcome for req-example\n"),
        "{out}"
    );
}

/// The report and preflight read the same records the same way: what preflight counts is
/// `passed` here, and what it names as open is `open` here.
#[test]
fn the_report_and_preflight_agree_about_every_record() {
    let scratch = Scratch::new("agree");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);
    project.ok(&["new", "runbook", "migrate"]);
    let specification = project.runbook("migrate").join("specification.html");
    let source = std::fs::read_to_string(&specification).expect("the specification reads");
    let unit = "          <section class=\"requirement\" id=\"req-after-backup\" \
                data-change=\"unchanged\">\n\
         \x20           <h3>After a backup <span class=\"hs-id\">req-after-backup</span></h3>\n\
         \x20           <p>A run MUST follow a backup.</p>\n\
         \x20           <article class=\"scenario\" id=\"scenario-after-backup\" \
         data-change=\"unchanged\">\n\
         \x20             <h4>The backup ran</h4>\n\
         \x20             <dl>\n\
         \x20               <dt>When</dt>\n\
         \x20               <dd>A run starts.</dd>\n\
         \x20               <dt>Then</dt>\n\
         \x20               <dd>A backup has finished and passed.</dd>\n\
         \x20             </dl>\n\
         \x20           </article>\n\
         \x20           <p data-field=\"note\">preflight/v3 {&quot;kind&quot;:&quot;run&quot;,\
         &quot;runbook&quot;:&quot;backup&quot;}</p>\n\
         \x20         </section>\n";
    let written = source.replace(
        "        </section>\n      </main>",
        &format!("{unit}        </section>\n      </main>"),
    );
    assert_ne!(written, source);
    std::fs::write(&specification, written).expect("rewritten");
    project.ok(&["format", &specification.to_string_lossy()]);

    // Every kind of run, and two whose verdict comes from a verification that references no
    // requirement the record owes: a stray one, and one under a withdrawn requirement. A report that
    // judged only the requirements it lists would call both passed.
    every_kind_of_run(&project);
    project.started("backup", "m-stray", |source| {
        stray_verification(&outcome(&steps(&source, "done", 2), "passed"))
    });
    project.started("backup", "n-withdrawn", |source| {
        withdrawn_requirement(&outcome(&steps(&source, "done", 2), "failed"))
    });

    let check = || {
        let (_, preflight, _) = project.run(&["--json", "preflight", "migrate"]);
        let preflight: serde_json::Value = serde_json::from_str(&preflight).expect("JSON");
        preflight["checks"]
            .as_array()
            .expect("checks")
            .iter()
            .find_map(|check| check.get("run"))
            .expect("the run check")
            .clone()
    };
    let named = |status: &serde_json::Value, state: &str| -> Vec<String> {
        status["records"]
            .as_array()
            .expect("records")
            .iter()
            .filter(|record| record["state"] == state)
            .map(|record| record["record"].as_str().expect("a name").to_owned())
            .collect()
    };

    // Met: the records preflight counts are the passed ones here, and its open runs the open ones.
    let run = check();
    let status = project.payload(&["run", "backup", "--status"]);
    assert_eq!(run["state"], "met");
    assert_eq!(named(&status, "passed"), ["e-passed", "f-dropped"]);
    assert_eq!(
        run["satisfiedBy"],
        serde_json::json!(named(&status, "passed"))
    );
    assert_eq!(run["satisfying"], 2);
    assert_eq!(run["openRuns"], serde_json::json!(named(&status, "open")));

    // Unmet, once the passed runs are gone: preflight names every other record with the reason the
    // report gives it, the unreadable ones included.
    std::fs::remove_file(project.record("backup", "e-passed")).expect("removed");
    std::fs::remove_file(project.record("backup", "f-dropped")).expect("removed");
    let run = check();
    let status = project.payload(&["run", "backup", "--status"]);
    assert_eq!(run["state"], "unmet");
    let refused: Vec<(&serde_json::Value, &serde_json::Value)> = run["notSatisfying"]
        .as_array()
        .expect("not satisfying")
        .iter()
        .map(|entry| (&entry["record"], &entry["reason"]))
        .collect();
    let reported: Vec<(&serde_json::Value, &serde_json::Value)> = status["records"]
        .as_array()
        .expect("records")
        .iter()
        .map(|record| (&record["record"], &record["reason"]))
        .collect();
    assert_eq!(refused, reported);
    assert_eq!(reported.len(), 12, "{status:#}");

    // The verification a verdict names is one the view shows.
    let stray = project.ok(&["run", "backup", "--status", "m-stray"]);
    assert!(
        stray.contains("  passed  req-example\nOther outcomes:\n  failed  verify-extra\n"),
        "{stray}"
    );
    assert!(
        stray.ends_with("Finished and passed: no, it recorded failed for verify-extra\n"),
        "{stray}"
    );
    let withdrawn = project.ok(&["run", "backup", "--status", "n-withdrawn"]);
    assert!(
        withdrawn.contains("Requirements: none\nOther outcomes:\n  failed  req-example\n"),
        "{withdrawn}"
    );
    assert!(
        withdrawn.ends_with("Finished and passed: no, it recorded failed for req-example\n"),
        "{withdrawn}"
    );
}

/// A runbook with no runs says so, a runbook or record that is not there is refused with exit 2,
/// and a record is matched by its name alone.
#[test]
fn what_is_not_there_is_named() {
    let scratch = Scratch::new("absent");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);

    let out = project.ok(&["run", "backup", "--status"]);
    assert_eq!(
        out,
        "backup has no runs on record. Start one with `htmlspec run backup --as <record>`\n"
    );
    let payload = project.payload(&["run", "backup", "--status"]);
    assert_eq!(
        payload,
        serde_json::json!({ "runbook": "backup", "records": [] })
    );

    let (code, out, err) = project.run(&["run", "nowhere", "--status"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(err.contains("no runbook named nowhere in"), "{err}");
    assert!(out.is_empty(), "{out}");

    let (code, out, err) = project.run(&["--json", "run", "backup", "--status", "b1"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(out.is_empty(), "a refusal wrote a payload: {out}");
    assert!(
        err.contains("backup has no run recorded as b1, and no runs on record"),
        "{err}"
    );

    project.started("backup", "b1", |source| source);
    project.started("backup", "b2", |source| source);
    for (typed, expected) in [
        (
            "b3",
            "backup has no run recorded as b3. Its records: b1, b2",
        ),
        (
            "b1.html",
            "A record is named without its .html: `htmlspec run backup --status b1`",
        ),
        ("../b1", "backup has no run recorded as ../b1"),
        // Matched byte for byte, so another case finds nothing even where the filesystem folds it.
        ("B1", "backup has no run recorded as B1"),
    ] {
        let (code, out, err) = project.run(&["run", "backup", "--status", typed]);
        assert_eq!(code, 2, "{typed}: {out}{err}");
        assert!(err.contains(expected), "{typed}: {err}");
    }
}

/// `--status` takes neither `--as` nor `--check`, a bare `run` names it, and the value never takes
/// the runbook's place.
#[test]
fn the_modes_are_exclusive_and_named() {
    let scratch = Scratch::new("modes");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);

    let (code, out, err) = project.run(&["run", "backup", "--status", "--as", "b1"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains("--status reports on the runs a runbook already has"),
        "{err}"
    );
    assert!(
        !project.runs("backup").exists(),
        "a refused pairing started a run"
    );

    let (code, out, err) = project.run(&["run", "backup", "--status", "--check"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains("--check and --status are two reports"),
        "{err}"
    );

    let (code, _, err) = project.run(&["run", "backup"]);
    assert_eq!(code, 2);
    assert!(
        err.contains("or --status for how far each has come"),
        "{err}"
    );

    // `run --status backup` reads `backup` as the record, and so names no runbook. The listing it
    // gives spells the command.
    let (code, out, err) = project.run(&["run", "--status", "backup"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(
        err.contains("see how far its runs have come with `htmlspec run <runbook> --status`"),
        "{err}"
    );
}

/// A runbook typed in another case, which a filesystem that folds case finds, reports under its
/// listed name, as `run` and `preflight` name it.
#[test]
fn a_runbook_typed_in_another_case_reports_under_its_name() {
    let scratch = Scratch::new("case");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);
    project.started("backup", "b1", |source| source);
    if !project.runbook("BACKUP").is_dir() {
        eprintln!("skipped: this filesystem tells names apart by case");
        return;
    }
    let out = project.ok(&["run", "BACKUP", "--status"]);
    assert!(
        out.starts_with("backup has 1 run, in name order:\n"),
        "{out}"
    );
    let payload = project.payload(&["run", "BACKUP", "--status", "b1"]);
    assert_eq!(payload["runbook"], "backup");
}

/// A record that is a link, or resolves outside the runbook's runs, is not read as one of its
/// records, as the run check does not count it.
#[cfg(unix)]
#[test]
fn a_linked_record_is_not_read() {
    let scratch = Scratch::new("link");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);
    project.started("backup", "b1", |source| {
        outcome(&steps(&source, "done", 2), "passed")
    });
    std::os::unix::fs::symlink(
        project.record("backup", "b1"),
        project.runs("backup").join("borrowed.html"),
    )
    .expect("a link");
    let out = project.ok(&["run", "backup", "--status", "borrowed"]);
    assert!(
        out.contains(
            "Finished and passed: no, it is a link, or resolves outside this runbook's runs"
        ),
        "{out}"
    );
    let payload = project.payload(&["run", "backup", "--status", "borrowed"]);
    assert_eq!(payload["records"][0]["state"], "unreadable");
}

/// A runbook, a record, a heading, a reason, and an outcome holding an escape sequence or a
/// right-to-left override reach the terminal by code point.
///
/// The run is made under plain names and renamed after, because a record composed under a name
/// holding a control character would be refused.
#[test]
fn every_name_and_text_reaches_the_terminal_escaped() {
    let scratch = Scratch::new("escapes");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);
    project.started("backup", "b1", |source| {
        outcome(
            &dropped_first(&source, Some("because \u{202E}reasons")),
            "pa\u{202E}ssed",
        )
        .replace("The second step", "The \u{202E}second step")
    });
    std::fs::rename(
        project.record("backup", "b1"),
        project.runs("backup").join("b1\u{1b}[31m.html"),
    )
    .expect("the record is renamed");
    let hostile = "back\u{1b}[2Jup";
    std::fs::rename(project.runbook("backup"), project.runbook(hostile))
        .expect("the runbook is renamed");

    let mut seen = String::new();
    for arguments in [
        vec!["run", hostile, "--status"],
        vec!["run", hostile, "--status", "b1\u{1b}[31m"],
        vec!["run", hostile, "--status", "b9\u{1b}[31m"],
    ] {
        let (_, out, err) = project.run(&arguments);
        seen.push_str(&out);
        seen.push_str(&err);
    }
    assert!(
        !seen.contains('\u{1b}'),
        "a raw escape reached the terminal:\n{seen:?}"
    );
    assert!(
        !seen.contains('\u{202E}'),
        "a raw override reached the terminal:\n{seen:?}"
    );
    for shown in [
        "back<U+001B>[2Jup has 1 run",
        "b1<U+001B>[31m",
        "because <U+202E>reasons",
        "pa<U+202E>ssed",
        "The <U+202E>second step",
        "no run recorded as b9<U+001B>[31m",
    ] {
        assert!(seen.contains(shown), "{shown} is missing:\n{seen}");
    }
    // The payload carries the names as they are, because JSON escapes them.
    let payload = project.payload(&["run", hostile, "--status"]);
    assert_eq!(payload["runbook"], hostile);
    assert_eq!(payload["records"][0]["record"], "b1\u{1b}[31m");
}

/// The report writes nothing, whatever it finds.
#[test]
fn the_report_writes_nothing() {
    let scratch = Scratch::new("writes-nothing");
    let project = Project::new(&scratch);
    project.ok(&["new", "runbook", "backup"]);
    project.started("backup", "b1", |source| steps(&source, "done", 1));
    project.write("backup", "stray.html", "");
    let before = walk(&project.root);
    project.ok(&["run", "backup", "--status"]);
    project.ok(&["run", "backup", "--status", "b1"]);
    project.ok(&["--json", "run", "backup", "--status"]);
    assert_eq!(walk(&project.root), before);
}

/// Every file under `root` with its contents, in path order.
fn walk(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable").flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let contents = std::fs::read(&path).expect("readable");
                found.push((path, contents));
            }
        }
    }
    found.sort();
    found
}
