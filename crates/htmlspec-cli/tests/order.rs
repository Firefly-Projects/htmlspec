//! What must run first, as `htmlspec list --runbooks`, the bare `run` listing, and
//! `htmlspec list --runbooks --graph` show it.
//!
//! The change `require-a-run-of-another-runbook`, group 3. Each runbook's run checks, and the state
//! of each now, read from the same records and declarations a start reads. Driven through the built
//! binary, because what is under test is what a person or an agent reads.
//!
//! The runbooks are generic and made with `htmlspec new runbook`: `backup` before `migrate`, and
//! `create-account`, `provision-database`, and `deploy-service` in a chain.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// A temporary directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-order-{label}-{}", std::process::id()));
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

/// A project, with its own registry directory so no test writes into the runner's configuration.
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

    /// A runbook made by `new runbook`.
    fn add(&self, name: &str) {
        let (code, out, err) = self.run(&["new", "runbook", name]);
        assert_eq!(code, 0, "new runbook {name} failed:\n{out}{err}");
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
            declaration
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('"', "&quot;")
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

    /// Declares that `runbook` requires a passed run of `required`.
    fn require(&self, runbook: &str, id: &str, required: &str) {
        self.declare(
            runbook,
            id,
            &format!("{{\"kind\":\"run\",\"runbook\":\"{required}\"}}"),
        );
    }

    /// Sets a runbook's release, with the changelog heading a release needs.
    fn release(&self, runbook: &str, version: &str) {
        std::fs::write(
            self.runbook(runbook).join("VERSION"),
            format!("{version}\n"),
        )
        .expect("the version is written");
        let changelog = self.runbook(runbook).join("CHANGELOG.md");
        let source = std::fs::read_to_string(&changelog).expect("the changelog reads");
        std::fs::write(
            &changelog,
            source.replacen(
                "## 0.1.0",
                &format!("## {version}\n\nThe release these tests run.\n\n## 0.1.0"),
                1,
            ),
        )
        .expect("the changelog is written");
    }

    fn record(&self, runbook: &str, record: &str) -> PathBuf {
        self.runbook(runbook)
            .join("runs")
            .join(format!("{record}.html"))
    }

    /// Starts a run and settles it as finished and passed.
    fn passed_run(&self, runbook: &str, record: &str) {
        let (code, out, err) = self.run(&["run", runbook, "--as", record]);
        assert_eq!(code, 0, "run {runbook} failed:\n{out}{err}");
        let path = self.record(runbook, record);
        let source = std::fs::read_to_string(&path).expect("the record reads");
        std::fs::write(
            &path,
            source
                .replace("data-state=\"todo\"", "data-state=\"done\"")
                .replace("data-outcome=\"not-run\"", "data-outcome=\"passed\""),
        )
        .expect("the record is finished");
    }

    /// Starts a run and settles only its first task, so it is still under way.
    fn open_run(&self, runbook: &str, record: &str) {
        let (code, out, err) = self.run(&["run", runbook, "--as", record]);
        assert_eq!(code, 0, "run {runbook} failed:\n{out}{err}");
        let path = self.record(runbook, record);
        let source = std::fs::read_to_string(&path).expect("the record reads");
        std::fs::write(
            &path,
            source.replacen("data-state=\"todo\"", "data-state=\"done\"", 1),
        )
        .expect("the record is advanced");
    }

    fn json(&self, arguments: &[&str]) -> serde_json::Value {
        let mut all = vec!["--json"];
        all.extend_from_slice(arguments);
        let (code, out, err) = self.run(&all);
        assert_eq!(code, 0, "{arguments:?} failed:\n{out}{err}");
        serde_json::from_str(&out).expect("the payload is JSON")
    }

    fn graph(&self) -> String {
        let (code, out, err) = self.run(&["list", "--runbooks", "--graph"]);
        assert_eq!(code, 0, "the graph failed:\n{out}{err}");
        assert!(err.is_empty(), "the graph wrote to standard error: {err}");
        out
    }
}

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

/// The design's example project: `backup` at 1.4.2 with one passed run, `migrate` requiring it at
/// 1.4.0 or later, and the account chain with no runs.
fn the_example(scratch: &Scratch, name: &str, creation: &[&str]) -> Project {
    let project = Project::new(scratch, name);
    for runbook in creation {
        project.add(runbook);
    }
    project.declare(
        "migrate",
        "after-backup",
        "{\"kind\":\"run\",\"runbook\":\"backup\",\"atLeast\":\"1.4.0\"}",
    );
    project.require("provision-database", "after-account", "create-account");
    project.require("deploy-service", "after-database", "provision-database");
    project.release("backup", "1.4.2");
    project.passed_run("backup", "nightly");
    project
}

const THE_EXAMPLE: &[&str] = &[
    "backup",
    "migrate",
    "create-account",
    "provision-database",
    "deploy-service",
];

/// The graph of the design's example, exactly as decision 8 prints it.
#[test]
fn the_graph_of_the_example_is_the_designs_byte_for_byte() {
    let scratch = Scratch::new("example");
    let project = the_example(&scratch, "p", THE_EXAMPLE);
    assert_eq!(
        project.graph(),
        "flowchart LR\n  r1[\"backup\"]\n  r2[\"create-account\"]\n  r3[\"deploy-service\"]\n  \
         r4[\"migrate\"]\n  r5[\"provision-database\"]\n  r1 -->|\"met, at least 1.4.0\"| r4\n  \
         r2 -->|\"unmet\"| r5\n  r5 -->|\"unmet\"| r3\n  linkStyle 0 \
         stroke:#2e6b3a,stroke-width:2px\n  linkStyle 1,2 \
         stroke:#a33333,stroke-width:2px,stroke-dasharray:5 4\n"
    );
}

/// The same project gives the same bytes on every run, and whatever order its runbook directories
/// were created in.
#[test]
fn the_graph_is_the_same_bytes_every_run_and_every_creation_order() {
    let scratch = Scratch::new("deterministic");
    let forward = the_example(&scratch, "forward", THE_EXAMPLE);
    let mut reversed: Vec<&str> = THE_EXAMPLE.to_vec();
    reversed.reverse();
    let backward = the_example(&scratch, "backward", &reversed);
    let first = forward.graph();
    assert_eq!(first, forward.graph(), "two runs differ");
    assert_eq!(first, backward.graph(), "two creation orders differ");
    assert_eq!(
        forward.json(&["list", "--runbooks"]),
        backward.json(&["list", "--runbooks"]),
        "two creation orders give different listings"
    );
}

/// `--graph` is refused without `--runbooks`, and with `--json` the payload is the listing's.
#[test]
fn graph_is_refused_alone_and_changes_nothing_under_json() {
    let scratch = Scratch::new("refusals");
    let project = the_example(&scratch, "p", THE_EXAMPLE);
    for arguments in [
        &["list", "--graph"][..],
        &["list", "--specs", "--graph"][..],
        &["get", "list", "--graph"][..],
    ] {
        let (code, out, err) = project.run(arguments);
        assert_eq!(code, 2, "{arguments:?} was not refused:\n{out}{err}");
        assert!(
            out.is_empty(),
            "{arguments:?} wrote to standard output: {out}"
        );
        assert!(err.contains("--runbooks"), "{arguments:?}: {err}");
    }
    assert_eq!(
        project.json(&["list", "--runbooks", "--graph"]),
        project.json(&["list", "--runbooks"])
    );
    let (code, spelled, err) = project.run(&["get", "list", "--runbooks", "--graph"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        spelled,
        project.graph(),
        "the request-verb spelling draws another graph"
    );
}

/// Every state, a malformed and an unreadable specification, and two separate cycles, in the
/// payload and in the rows.
#[test]
fn the_listing_shows_each_prerequisites_state() {
    let scratch = Scratch::new("states");
    let project = Project::new(&scratch, "p");
    for runbook in [
        "backup",
        "migrate",
        "create-account",
        "provision-database",
        "lookup",
        "loop",
        "ping",
        "pong",
        "east",
        "west",
        "broken",
        "gone",
        "reach",
    ] {
        project.add(runbook);
    }
    project.passed_run("backup", "nightly");
    project.passed_run("backup", "weekly");
    // A backup still under way beside the passed ones: the met prerequisite names it.
    project.open_run("backup", "tonight");
    // Passed runs of a runbook that will require itself and of one end of a cycle, made before the
    // declarations, since a start refuses a contract that leads back to its own runbook. The
    // listing must not count them: no record decides a self-reference or a cycle.
    project.passed_run("loop", "done");
    project.passed_run("pong", "done");
    project.open_run("pong", "open");
    project.require("migrate", "after-backup", "backup");
    project.open_run("create-account", "first");
    project.require("provision-database", "after-account", "create-account");
    project.require("lookup", "after-nowhere", "nowhere");
    project.require("loop", "after-itself", "loop");
    project.require("ping", "after-pong", "pong");
    project.require("pong", "after-ping", "ping");
    project.require("east", "after-west", "west");
    project.require("west", "after-east", "east");
    project.declare(
        "broken",
        "after-backup",
        "{\"kind\":\"run\",\"runbook\":\"backup\",\"since\":\"1.0.0\"}",
    );
    std::fs::remove_file(project.runbook("gone").join("specification.html"))
        .expect("the specification is removed");
    project.require("reach", "after-broken", "broken");

    let listed = project.json(&["list", "--runbooks"]);
    let entry = |name: &str| -> serde_json::Value {
        listed["runbooks"]
            .as_array()
            .expect("runbooks")
            .iter()
            .find(|entry| entry["name"] == name)
            .unwrap_or_else(|| panic!("{name} is not listed: {listed:#}"))
            .clone()
    };
    let state =
        |name: &str| -> serde_json::Value { entry(name)["prerequisites"][0]["state"].clone() };

    assert_eq!(state("migrate"), "met", "{listed:#}");
    assert_eq!(
        entry("migrate")["prerequisites"][0]["satisfiedBy"],
        serde_json::json!(["nightly", "weekly"])
    );
    assert_eq!(entry("migrate")["prerequisites"][0]["satisfying"], 2);
    assert_eq!(entry("migrate")["prerequisitesMet"], true);
    assert_eq!(state("provision-database"), "unmet");
    assert_eq!(
        entry("provision-database")["prerequisites"][0]["openRuns"],
        serde_json::json!(["first"])
    );
    assert_eq!(
        entry("migrate")["prerequisites"][0]["openRuns"],
        serde_json::json!(["tonight"])
    );
    assert_eq!(entry("provision-database")["prerequisitesMet"], false);
    assert_eq!(state("lookup"), "missing");
    assert_eq!(state("loop"), "self");
    assert_eq!(state("ping"), "cycle");
    assert_eq!(state("pong"), "cycle");
    assert_eq!(state("reach"), "unavailable");
    // Preflight reports the same check with the same word, so the listing and a start agree.
    let (code, out, err) = project.run(&["--json", "preflight", "reach"]);
    assert_eq!(
        code, 1,
        "an unavailable check leaves reach not ready:\n{out}{err}"
    );
    let preflight: serde_json::Value = serde_json::from_str(&out).expect("the payload is JSON");
    assert_eq!(
        preflight["checks"][0]["run"]["state"], "unavailable",
        "{preflight:#}"
    );
    assert_eq!(entry("reach")["prerequisitesMet"], false);
    // Only met and unmet are decided by records, so only they name any: a self-reference, a cycle,
    // a missing runbook, and an unavailable one carry none, even where the named runbook has passed and
    // open runs, as `loop` and `pong` do here.
    for name in ["loop", "ping", "lookup", "reach"] {
        let prerequisite = &entry(name)["prerequisites"][0];
        assert_eq!(
            prerequisite["satisfiedBy"],
            serde_json::json!([]),
            "{name}: {listed:#}"
        );
        assert_eq!(prerequisite["satisfying"], 0, "{name}: {listed:#}");
        assert_eq!(
            prerequisite["openRuns"],
            serde_json::json!([]),
            "{name}: {listed:#}"
        );
    }
    assert_eq!(
        entry("backup")["prerequisitesMet"],
        true,
        "a runbook requiring nothing is met"
    );
    for (name, declarations) in [("broken", "malformed"), ("gone", "unreadable")] {
        assert_eq!(entry(name)["declarations"], declarations, "{listed:#}");
        assert_eq!(entry(name)["prerequisitesMet"], false, "{listed:#}");
        assert_eq!(
            entry(name)["prerequisites"],
            serde_json::json!([]),
            "{listed:#}"
        );
    }
    assert_eq!(
        listed["prerequisiteCycles"],
        serde_json::json!([["east", "west"], ["loop"], ["ping", "pong"]]),
        "{listed:#}"
    );

    let (code, rows, err) = project.run(&["list", "--runbooks"]);
    assert_eq!(code, 0, "{err}");
    for expected in [
        "  requires backup: met by nightly, weekly, 1 run still open\n",
        "  requires create-account: unmet, 1 run still open\n",
        "  requires nowhere: missing, this project has no runbook of that name\n",
        "  requires loop: self, the runbook requires itself\n",
        "  requires pong: cycle, it leads back to this runbook\n",
        "  requires broken: unavailable, a runbook it reaches has declarations that cannot be read\n",
        "  declarations cannot be read: ",
    ] {
        assert!(rows.contains(expected), "missing {expected:?} in:\n{rows}");
    }
}

/// A bare `run` shows the same prerequisite rows, a cycle and a missing runbook included, before
/// refusing as a usage error.
#[test]
fn the_bare_run_listing_shows_what_can_start() {
    let scratch = Scratch::new("bare");
    let project = Project::new(&scratch, "p");
    for runbook in ["backup", "migrate", "lookup", "ping", "pong"] {
        project.add(runbook);
    }
    project.require("migrate", "after-backup", "backup");
    project.require("lookup", "after-nowhere", "nowhere");
    project.require("ping", "after-pong", "pong");
    project.require("pong", "after-ping", "ping");
    let (code, out, err) = project.run(&["run"]);
    assert_eq!(code, 2, "{out}{err}");
    for expected in [
        "    requires backup: unmet, no finished and passed run\n",
        "    requires nowhere: missing, this project has no runbook of that name\n",
        "    requires pong: cycle, it leads back to this runbook\n",
        "    requires ping: cycle, it leads back to this runbook\n",
    ] {
        assert!(err.contains(expected), "missing {expected:?} in:\n{err}");
    }
    let (_, rows, _) = project.run(&["list", "--runbooks"]);
    let indented: Vec<String> = rows.lines().map(|line| format!("  {line}")).collect();
    for line in indented {
        assert!(
            err.contains(&line),
            "the bare run listing lacks {line:?}:\n{err}"
        );
    }
}

/// The graph's edge cases: an isolated runbook, a missing one, a self-reference, a cycle,
/// declarations that cannot be read, and two declarations of one prerequisite.
#[test]
fn the_graph_draws_each_edge_case() {
    let scratch = Scratch::new("cases");
    let project = Project::new(&scratch, "p");
    for runbook in [
        "alone", "ask", "loop", "ping", "pong", "broken", "backup", "migrate",
    ] {
        project.add(runbook);
    }
    project.require("ask", "after-nowhere", "nowhere");
    project.require("loop", "after-itself", "loop");
    project.require("ping", "after-pong", "pong");
    project.require("pong", "after-ping", "ping");
    project.declare(
        "broken",
        "after-backup",
        "{\"kind\":\"run\",\"runbook\":\"backup\",\"since\":\"1.0.0\"}",
    );
    project.declare(
        "migrate",
        "after-backup-new",
        "{\"kind\":\"run\",\"runbook\":\"backup\",\"atLeast\":\"0.1.0\"}",
    );
    project.declare(
        "migrate",
        "after-backup-newer",
        "{\"kind\":\"run\",\"runbook\":\"backup\",\"atLeast\":\"0.3.0\"}",
    );
    project.passed_run("backup", "nightly");
    assert_eq!(
        project.graph(),
        "flowchart LR\n  r1[\"alone\"]\n  r2[\"ask\"]\n  r3[\"backup\"]\n  \
         r4[\"broken (declarations cannot be read)\"]\n  r5[\"loop\"]\n  r6[\"migrate\"]\n  \
         r7[\"nowhere (no such runbook)\"]\n  r8[\"ping\"]\n  r9[\"pong\"]\n  \
         r3 -->|\"unmet, at least 0.1.0 and 0.3.0\"| r6\n  r5 -->|\"requires itself\"| r5\n  \
         r7 -->|\"unmet\"| r2\n  r8 -->|\"cycle\"| r9\n  r9 -->|\"cycle\"| r8\n  \
         classDef missing stroke-dasharray:5 4\n  \
         classDef malformed stroke:#a33333,stroke-width:2px\n  class r7 missing\n  \
         class r4 malformed\n  \
         linkStyle 0,2 stroke:#a33333,stroke-width:2px,stroke-dasharray:5 4\n  \
         linkStyle 1,3,4 stroke:#7a3fa3,stroke-width:3px\n"
    );
}

/// A project with no runbooks draws the header alone.
#[test]
fn an_empty_project_draws_the_header_alone() {
    let scratch = Scratch::new("empty");
    let project = Project::new(&scratch, "p");
    assert_eq!(project.graph(), "flowchart LR\n");
    let listed = project.json(&["list", "--runbooks"]);
    assert_eq!(listed["prerequisiteCycles"], serde_json::json!([]));
}

/// Names holding characters Mermaid, HTML, or a terminal would read reach the graph as entity codes
/// and code points: no raw one, and no line break.
#[test]
fn hostile_names_reach_the_graph_escaped() {
    let scratch = Scratch::new("hostile");
    let project = Project::new(&scratch, "p");
    project.add("deploy");
    let hostile = [
        "q\\\"x",
        "h#x",
        "`b`",
        "l<t",
        "a&b",
        "e\\u001bx",
        "n\\nl",
        "r\\u202eo",
        // A Mermaid directive, which `%%` begins even inside a quoted label, and a bare `%%`.
        "%%{init: {\\\"theme\\\":\\\"dark\\\"}}%%",
        "x %%",
        // The characters Mermaid carries an entity code through its parser in.
        "\u{FB02}\u{B0}x\u{B6}\u{DF}",
    ];
    for (index, name) in hostile.iter().enumerate() {
        project.declare(
            "deploy",
            &format!("after-{index}"),
            &format!("{{\"kind\":\"run\",\"runbook\":\"{name}\"}}"),
        );
    }
    let drawn = project.graph();
    let body: Vec<&str> = drawn.lines().collect();
    for line in &body {
        assert!(!line.contains('<'), "a raw < in {line:?}");
        assert!(!line.contains('&'), "a raw & in {line:?}");
        assert!(!line.contains('`'), "a raw backtick in {line:?}");
        assert!(!line.contains('\u{1b}'), "a raw escape in {line:?}");
        assert!(!line.contains('\u{202e}'), "a raw override in {line:?}");
        assert!(!line.contains('%'), "a raw % in {line:?}");
        for placeholder in ['\u{FB02}', '\u{B0}', '\u{B6}', '\u{DF}'] {
            assert!(
                !line.contains(placeholder),
                "Mermaid's placeholder {placeholder:?} in {line:?}"
            );
        }
        // Every quote is a label's own delimiter: one pair per node or edge label.
        assert_eq!(line.matches('"').count() % 2, 0, "an odd quote in {line:?}");
    }
    for expected in [
        "[\"q#quot;x (no such runbook)\"]",
        "[\"h#35;x (no such runbook)\"]",
        "[\"#96;b#96; (no such runbook)\"]",
        "[\"l#lt;t (no such runbook)\"]",
        "[\"a#amp;b (no such runbook)\"]",
        "[\"e#lt;U+001B#gt;x (no such runbook)\"]",
        "[\"n#lt;U+000A#gt;l (no such runbook)\"]",
        "[\"r#lt;U+202E#gt;o (no such runbook)\"]",
        "[\"#37;#37;{init: {#quot;theme#quot;:#quot;dark#quot;}}#37;#37; (no such runbook)\"]",
        "[\"x #37;#37; (no such runbook)\"]",
        "[\"#lt;U+FB02#gt;#lt;U+00B0#gt;x#lt;U+00B6#gt;#lt;U+00DF#gt; (no such runbook)\"]",
    ] {
        assert!(drawn.contains(expected), "missing {expected} in:\n{drawn}");
    }
    assert!(
        !drawn.contains("%%"),
        "a directive reached the graph:\n{drawn}"
    );
    // One line per node and edge, so no name broke a line: the header, the runbook and one node per
    // name, one edge per name, the missing class's two lines, and one link style.
    let names = hostile.len();
    assert_eq!(body.len(), 1 + (1 + names) + names + 2 + 1, "{drawn}");
    let (_, rows, _) = project.run(&["list", "--runbooks"]);
    assert!(
        !rows.contains('\u{1b}'),
        "a raw escape in the rows:\n{rows}"
    );
    assert!(rows.contains("requires e\\u{1b}x: missing"), "{rows}");
}

/// The graph pasted into a document's `<pre class="mermaid">` validates, and formats canonically.
#[test]
fn a_pasted_graph_validates_and_formats_canonically() {
    let scratch = Scratch::new("pasted");
    let project = the_example(&scratch, "p", THE_EXAMPLE);
    project.require("migrate", "after-quirk", "`quirk` & <co>");
    let drawn = project.graph();
    let tasks = project.runbook("backup").join("tasks.html");
    let source = std::fs::read_to_string(&tasks).expect("the tasks document reads");
    let opening = source
        .find("<pre class=\"mermaid\">")
        .expect("the runbook template draws a figure")
        + "<pre class=\"mermaid\">".len();
    let closing = opening + source[opening..].find("</pre>").expect("the figure closes");
    let pasted = format!(
        "{}{}{}",
        &source[..opening],
        drawn.trim_end(),
        &source[closing..]
    );
    std::fs::write(&tasks, pasted).expect("the graph is pasted");
    let path = tasks.to_string_lossy().into_owned();
    let (code, out, err) = project.run(&["validate", &path]);
    assert_eq!(code, 0, "the pasted graph does not validate:\n{out}{err}");
    let (code, out, err) = project.run(&["format", &path]);
    assert_eq!(code, 0, "the pasted graph does not format:\n{out}{err}");
    let (code, out, err) = project.run(&["format", "--check", &path]);
    assert_eq!(code, 0, "the formatted graph is not canonical:\n{out}{err}");
    let formatted = std::fs::read_to_string(&tasks).expect("the tasks document reads");
    assert!(
        formatted.contains("r2 --&gt;|\"met, at least 1.4.0\"| r5"),
        "the arrows are not written as &gt;:\n{formatted}"
    );
}
