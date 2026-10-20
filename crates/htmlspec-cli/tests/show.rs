//! `htmlspec show <change> --diff`, driven through the built binary.
//!
//! toolchain-packaging's "The command surface shows a modified requirement beside the accepted
//! one". Every test runs in a project built by `init --tools none` and `new change`, and every run
//! of the view is checked to leave each file in the project byte for byte as it was.

mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

const SUCCESS: i32 = 0;
const DOCUMENT_VIOLATION: i32 = 1;
const OPERATIONAL_FAILURE: i32 = 2;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// Runs the binary from inside `directory`, with the store registry under `config`.
///
/// Every run points the registry at the scratch directory, so no test reads or writes the
/// registrations of whoever runs the suite.
fn run_in(directory: &Path, config: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .env("XDG_CONFIG_HOME", config)
        .args(arguments)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    support::held(arguments, &stdout);
    (
        output.status.code().expect("the process exited normally"),
        stdout,
        String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    )
}

/// A temporary directory that removes itself, holding a project and a store registry.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-show-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("project")).expect("the scratch directory is created");
        Self(path)
    }

    fn project(&self) -> PathBuf {
        self.0.join("project")
    }

    fn config(&self) -> PathBuf {
        self.0.join("config")
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.project().join(relative)
    }

    /// Runs the binary from the project's root.
    fn run(&self, arguments: &[&str]) -> (i32, String, String) {
        run_in(&self.project(), &self.config(), arguments)
    }

    /// Runs the view from `directory`, and asserts that no file in the project changed.
    ///
    /// The toolchain-packaging scenario "The view writes nothing", held for every run of the view
    /// in this file, whatever its outcome.
    fn show_from(&self, directory: &Path, arguments: &[&str]) -> (i32, String, String) {
        let before = snapshot(&self.project());
        let ran = run_in(directory, &self.config(), arguments);
        assert!(
            before == snapshot(&self.project()),
            "{arguments:?} changed a file in the project:\nstdout: {}\nstderr: {}",
            ran.1,
            ran.2
        );
        ran
    }

    fn show(&self, arguments: &[&str]) -> (i32, String, String) {
        self.show_from(&self.project(), arguments)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Every file under `root`, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the directory reads") {
            let path = entry.expect("the entry reads").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let bytes = std::fs::read(&path).expect("the file reads");
                files.insert(path, bytes);
            }
        }
    }
    files
}

const CHANGE_SPECIFICATION: &str = "htmlspec/changes/a-change/specification.html";
const ACCEPTED: &str = "htmlspec/specs/surface/spec.html";

const SPELLINGS: &str = "The spellings are therefore fixed rather than left to an implementer: \
                         <code>validate</code>, <code>archive</code>, <code>init</code>, \
                         <code>new</code>, <code>list</code>, <code>status</code>, \
                         <code>instructions</code>, and <code>store</code> name the operations \
                         OpenSpec also provides.";
const SPELLINGS_WITH_SYNC: &str = "The spellings are therefore fixed rather than left to an \
                                   implementer: <code>validate</code>, <code>archive</code>, \
                                   <code>init</code>, <code>new</code>, <code>list</code>, \
                                   <code>status</code>, <code>sync</code>, \
                                   <code>instructions</code>, and <code>store</code> name the \
                                   operations OpenSpec also provides.";

/// The requirement the one-word case modifies, with `spellings` as its second paragraph, marked
/// `change` where it is given one.
fn the_names(change: Option<&str>, spellings: &str) -> String {
    let state = change.map_or(String::new(), |state| format!(" data-change=\"{state}\""));
    let scenario_state = change.map_or("", |_| " data-change=\"unchanged\"");
    format!(
        r#"<section class="requirement" id="req-the-names"{state}>
            <h3>The command surface names its operations as OpenSpec names them <span class="hs-id">req-the-names</span></h3>
            <p><strong>Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it.</strong> This holds for the subcommand and for the options that select what it acts on.</p>
            <p>{spellings}</p>
            <article class="scenario" id="scenario-the-names-are-found"{scenario_state}>
              <h4>The names are found</h4>
              <dl>
                <dt>When</dt>
                <dd>A contributor runs the operation.</dd>
                <dt>Then</dt>
                <dd>It is found under the name the requirement fixes.</dd>
              </dl>
            </article>
          </section>"#
    )
}

/// A requirement the change adds, which no accepted specification holds.
const THE_REPORT: &str = r#"<section class="requirement" id="req-the-report" data-change="added">
            <h3>The command surface reports whether a change reached the accepted contract <span class="hs-id">req-the-report</span></h3>
            <p>The command surface SHALL provide an operation that reports whether a change reached the accepted contract.</p>
            <article class="scenario" id="scenario-the-report-is-made" data-change="added">
              <h4>The report is made</h4>
              <dl>
                <dt>When</dt>
                <dd>A contributor runs the operation.</dd>
                <dt>Then</dt>
                <dd>It reports each requirement.</dd>
              </dl>
            </article>
          </section>"#;

/// Writes a specification at `relative` with `new document`, holding `requirements` in place of
/// the template's example content.
fn specification(scratch: &Scratch, relative: &str, requirements: &str) {
    let (code, out, err) = scratch.run(&[
        "new",
        "document",
        "specification",
        relative,
        "--subject",
        "The command surface",
    ]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let path = scratch.path(relative);
    let text = std::fs::read_to_string(&path).expect("the specification reads");

    let open = "<main class=\"hs-main\" id=\"hs-content\">";
    let start = text.find(open).expect("the template has its main") + open.len();
    let end = text.find("</main>").expect("the main closes");
    let units = format!(
        "\n        <section id=\"hs-requirements\">\n          <h2>Requirements</h2>\n          \
         {requirements}\n        </section>\n      "
    );
    let mut text = format!("{}{units}{}", &text[..start], &text[end..]);

    let placeholder = "<p>Replace this paragraph with what this document is for.";
    let start = text
        .find(placeholder)
        .expect("the template has its summary");
    let end = start + text[start..].find("</p>").expect("the summary closes") + "</p>".len();
    text.replace_range(
        start..end,
        "<p>What the command surface offers, and under which names.</p>",
    );

    // The contents list names only the sections the document still holds.
    let text: String = text
        .split_inclusive('\n')
        .filter(|line| {
            ![
                "#hs-decisions",
                "#hs-risks",
                "#hs-evidence",
                "#hs-verification",
            ]
            .iter()
            .any(|section| line.contains(&format!("<li><a href=\"{section}\">")))
        })
        .collect();
    std::fs::write(&path, text).expect("the specification is written");
}

/// A project holding `a-change`, whose specification adds `sync` to the accepted list of
/// spellings and adds one requirement the accepted specification does not hold.
fn the_one_word_case(label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    let (code, out, err) = scratch.run(&["init", "--tools", "none"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let (code, out, err) = scratch.run(&["new", "change", "a-change"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    specification(&scratch, ACCEPTED, &the_names(None, SPELLINGS));
    specification(
        &scratch,
        CHANGE_SPECIFICATION,
        &format!(
            "{}\n          {THE_REPORT}",
            the_names(Some("modified"), SPELLINGS_WITH_SYNC)
        ),
    );
    let (code, out, err) = scratch.run(&["validate", CHANGE_SPECIFICATION]);
    assert_eq!(
        code, SUCCESS,
        "the fixture does not validate:\n{out}\n{err}"
    );
    scratch
}

/// The toolchain-packaging scenario "A one-word modification", in human form.
#[test]
fn the_one_word_case_shows_one_pair_with_the_word_marked() {
    let scratch = the_one_word_case("one-word");
    let (code, out, err) = scratch.show(&["show", "a-change", "--diff"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    // The first two lines say what the view is and explain the marks, so they name both.
    let body: String = out.lines().skip(2).collect::<Vec<_>>().join("\n");
    assert_eq!(body.matches("{+").count(), 1, "one word is marked:\n{out}");
    assert!(body.contains("{+sync,+}"), "{out}");
    assert!(
        !body.contains("[-"),
        "nothing is held only by the accepted side:\n{out}"
    );
    assert!(out.contains("the accepted specification holds:"), "{out}");
    assert!(out.contains("the change states:"), "{out}");
    assert!(out.contains(&format!("compared with {ACCEPTED}")), "{out}");
    assert!(
        out.contains("1 of 2 paragraphs held by both"),
        "the paragraph held by both is not counted:\n{out}"
    );
    assert!(out.contains("req-the-report, added"), "{out}");
    assert!(out.contains("held by no accepted specification"), "{out}");
}

/// The one-word case as a payload: the scenario "A structured view".
#[test]
fn the_one_word_case_as_a_payload() {
    let scratch = the_one_word_case("one-word-json");
    let (code, out, err) = scratch.show(&["--json", "show", "a-change", "--diff"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let payload: Value = serde_json::from_str(&out).expect("the payload parses");
    assert_eq!(payload["change"], "a-change");

    let names = &payload["requirements"][0];
    assert_eq!(names["id"], "req-the-names");
    assert_eq!(names["declared"], "modified");
    let comparison = &names["comparison"];
    assert_eq!(comparison["shown"], "differs");
    assert_eq!(comparison["acceptedIn"], ACCEPTED);
    assert_eq!(comparison["statements"]["marks"], "comparison");
    let paragraphs = comparison["statements"]["paragraphs"]
        .as_array()
        .expect("a list of paragraphs");
    assert_eq!(paragraphs.len(), 2, "{paragraphs:#?}");
    assert_eq!(paragraphs[0]["kind"], "both");
    let pair = &paragraphs[1];
    assert_eq!(pair["kind"], "pair");
    assert_eq!(
        (pair["position"].as_u64(), pair["acceptedPosition"].as_u64()),
        (Some(2), Some(2))
    );
    let only = |side: &str| -> Vec<String> {
        pair[side]
            .as_array()
            .expect("a list of runs")
            .iter()
            .filter(|run| run["onlyThisSide"] == Value::Bool(true))
            .map(|run| run["text"].as_str().expect("text").trim().to_owned())
            .collect()
    };
    assert_eq!(only("changeStates"), vec![String::from("sync,")]);
    assert!(only("acceptedHolds").is_empty());
    assert_eq!(comparison["scenarios"][0]["kind"], "both");

    let report = &payload["requirements"][1];
    assert_eq!(report["id"], "req-the-report");
    assert_eq!(report["declared"], "added");
    assert_eq!(report["comparison"]["shown"], "held-by-none");
}

/// The scenario "The view writes nothing", across every outcome. Each run below is checked by
/// [`Scratch::show_from`]; this test gathers the outcomes the others take one at a time.
#[test]
fn the_view_writes_nothing() {
    let scratch = the_one_word_case("writes-nothing");
    // Each outcome's code as well, so that the view is known to have reached the outcome named
    // and a program that refused everything could not pass.
    for (arguments, expected) in [
        (&["show", "a-change", "--diff"][..], SUCCESS),
        (&["--json", "show", "a-change", "--diff"], SUCCESS),
        (&["show", "htmlspec/changes/a-change", "--diff"], SUCCESS),
        (&["show", "a-change"], OPERATIONAL_FAILURE),
        (&["show", "no-such-change", "--diff"], OPERATIONAL_FAILURE),
    ] {
        let (code, out, err) = scratch.show(arguments);
        assert_eq!(
            code, expected,
            "{arguments:?}\nstdout: {out}\nstderr: {err}"
        );
    }
}

/// The scenario "Showing a change without a view".
#[test]
fn showing_a_change_without_a_view_names_diff() {
    let scratch = the_one_word_case("no-view");
    for arguments in [&["show", "a-change"][..], &["--json", "show", "a-change"]] {
        let (code, out, err) = scratch.show(arguments);
        assert_eq!(
            code, OPERATIONAL_FAILURE,
            "{arguments:?}\nstdout: {out}\nstderr: {err}"
        );
        assert!(out.is_empty(), "{arguments:?} printed: {out}");
        assert!(
            err.contains("--diff"),
            "{arguments:?} does not name --diff: {err}"
        );
    }
}

/// The scenario "An archived change", by its directory and through a registered store.
#[test]
fn an_archived_change_is_refused() {
    let scratch = the_one_word_case("archived");
    std::fs::create_dir_all(scratch.path("htmlspec/changes/archive")).expect("the archive exists");
    std::fs::rename(
        scratch.path("htmlspec/changes/a-change"),
        scratch.path("htmlspec/changes/archive/2026-10-07-a-change"),
    )
    .expect("the change moves into the archive");

    let (code, out, err) = scratch.show(&[
        "show",
        "htmlspec/changes/archive/2026-10-07-a-change",
        "--diff",
    ]);
    assert_eq!(code, OPERATIONAL_FAILURE, "stdout: {out}\nstderr: {err}");
    assert!(out.is_empty(), "{out}");
    assert!(err.contains("is an archived change"), "{err}");
    assert!(
        err.contains("reversed"),
        "the refusal does not say why: {err}"
    );

    let project = scratch.project();
    let (code, out, err) = scratch.run(&["store", "add", "surface", &project.to_string_lossy()]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let (code, out, err) = scratch.show_from(
        &scratch.0,
        &[
            "--store",
            "surface",
            "show",
            "archive/2026-10-07-a-change",
            "--diff",
        ],
    );
    assert_eq!(code, OPERATIONAL_FAILURE, "stdout: {out}\nstderr: {err}");
    assert!(err.contains("is an archived change"), "{err}");
}

/// The scenario "An invalid specification".
#[test]
fn an_invalid_specification_is_refused() {
    let scratch = the_one_word_case("invalid");
    let path = scratch.path(CHANGE_SPECIFICATION);
    let text = std::fs::read_to_string(&path).expect("the specification reads");
    std::fs::write(
        &path,
        text.replacen("data-change=\"modified\"", "data-change=\"sideways\"", 1),
    )
    .expect("the specification is written");

    let (code, out, err) = scratch.show(&["show", "a-change", "--diff"]);
    assert_eq!(code, DOCUMENT_VIOLATION, "stdout: {out}\nstderr: {err}");
    assert!(
        !out.contains("the change states:"),
        "a comparison was shown: {out}"
    );
    assert!(
        err.contains("does not run while the specification reports an error"),
        "{err}"
    );
}

/// A change that cannot be resolved, and a runbook, are each refused as `sync` refuses them.
#[test]
fn an_unresolvable_change_and_a_runbook_are_refused() {
    let scratch = the_one_word_case("unresolvable");
    let (code, out, err) = scratch.show(&["show", "no-such-change", "--diff"]);
    assert_eq!(code, OPERATIONAL_FAILURE, "stdout: {out}\nstderr: {err}");
    assert!(err.contains("no change named no-such-change"), "{err}");

    let runbook = "htmlspec/runbooks/release";
    for (kind, file) in [
        ("specification", "specification.html"),
        ("tasks", "tasks.html"),
    ] {
        let (code, out, err) = scratch.run(&[
            "new",
            "document",
            kind,
            &format!("{runbook}/{file}"),
            "--subject",
            "Releasing the project",
        ]);
        assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    }
    let (code, out, err) = scratch.show(&["show", runbook, "--diff"]);
    assert_eq!(code, OPERATIONAL_FAILURE, "stdout: {out}\nstderr: {err}");
    assert!(err.contains("is a runbook, not a change"), "{err}");
    assert!(
        err.contains("no accepted specification holds"),
        "the refusal does not give the view's reason: {err}"
    );
}

/// Under `--json`, standard output holds the payload and nothing else.
#[test]
fn the_payload_is_alone_on_standard_output() {
    let scratch = the_one_word_case("payload-alone");
    let (code, out, err) = scratch.show(&["--json", "show", "a-change", "--diff"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let mut values = serde_json::Deserializer::from_str(&out).into_iter::<Value>();
    assert!(values.next().is_some_and(|value| value.is_ok()), "{out}");
    assert!(
        values.next().is_none(),
        "more than one value on standard output: {out}"
    );
}

/// The scenario "Nothing reads as merge text", in both forms.
#[test]
fn nothing_reads_as_merge_text() {
    let scratch = the_one_word_case("not-merge-text");
    let (code, out, err) = scratch.show(&["show", "a-change", "--diff"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    for line in out.lines() {
        for header in ["@@", "---", "+++"] {
            assert!(!line.starts_with(header), "a patch header: {line}");
        }
        let first = line.trim_start().chars().next();
        assert!(
            !matches!(first, Some('+' | '-')),
            "a line marked whole as added or removed: {line}"
        );
    }

    let (code, out, err) = scratch.show(&["--json", "show", "a-change", "--diff"]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let payload: Value = serde_json::from_str(&out).expect("the payload parses");
    let mut keys = Vec::new();
    collect_keys(&payload, &mut keys);
    for key in keys {
        let lower = key.to_lowercase();
        for forbidden in ["result", "merge", "replace", "apply", "patch", "diff"] {
            assert!(
                !lower.contains(forbidden),
                "the payload key {key} is named for a {forbidden}"
            );
        }
    }
}

fn collect_keys(value: &Value, into: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (key, inner) in object {
                into.push(key.clone());
                collect_keys(inner, into);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_keys(item, into);
            }
        }
        _ => {}
    }
}

/// The scenario "A registered standalone project": the same view, through `--store`, from outside
/// the project.
#[test]
fn the_view_through_a_registered_store_is_the_same() {
    let scratch = the_one_word_case("store");
    let (code, here, err) = scratch.show(&["--json", "show", "a-change", "--diff"]);
    assert_eq!(code, SUCCESS, "stdout: {here}\nstderr: {err}");

    let project = scratch.project();
    let (code, out, err) = scratch.run(&["store", "add", "surface", &project.to_string_lossy()]);
    assert_eq!(code, SUCCESS, "stdout: {out}\nstderr: {err}");
    let (code, there, err) = scratch.show_from(
        &scratch.0,
        &["--json", "--store", "surface", "show", "a-change", "--diff"],
    );
    assert_eq!(code, SUCCESS, "stdout: {there}\nstderr: {err}");
    assert_eq!(
        serde_json::from_str::<Value>(&here).expect("parses"),
        serde_json::from_str::<Value>(&there).expect("parses")
    );
}
