//! The migration plan's shape, verified against the project's own checks.
//!
//! Section 4's first task settles what a plan *is*, and a shape that only the coverage gate can
//! read would be a second document format hiding inside the first. So these tests take a plan
//! written the way the workflow will write it and put it through `validate`, `format --check`, and
//! `migrate promote` in turn — through the compiled program, so a shape that satisfies the gate's
//! reader and fails the validator's cannot pass here.
//!
//! `format --check` is the test worth naming. The formatter reorders attributes and re-indents,
//! and a plan whose per-entity facts lived in attributes rather than in notes would either be
//! rewritten by the formatter or refused by it. Running the check after the format proves the
//! shape is a fixed point rather than something that merely parsed once.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The binary under test.
fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// Runs the binary from inside `directory`, returning its exit code and streams.
fn run(directory: &Path, args: &[&str]) -> (i32, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .args(args)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    support::held(args, &stdout);
    (
        output.status.code().expect("the process exited normally"),
        stdout,
        String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    )
}

/// A temporary directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-plan-{label}-{}", std::process::id()));
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

/// The two plan items, as the workflow writes them.
///
/// One converted and one dropped-with-a-reason, because those are the two shapes the gate treats
/// differently: a drop owes a reason and a conversion does not. The dropped one uses the task's
/// own `data-field="reason"` rather than a note, which is the spelling the validator already
/// requires of a `dropped` task — so the plan does not need a second convention for a fact the
/// document model already carries.
const ITEMS: &str = r#"
            <li class="task" id="task-openspec-specs-project-bootstrap-spec-md" data-change="added" data-state="todo">
              <h3>The project-bootstrap capability's accepted specification <span class="hs-id">task-openspec-specs-project-bootstrap-spec-md</span></h3>
              <p data-field="note">source: openspec/specs/project-bootstrap/spec.md</p>
              <p data-field="note">destination: htmlspec/specs/project-bootstrap/spec.html</p>
              <p data-field="note">disposition: converted</p>
              <p data-field="note">derives: A project has a declared root holding one asset store -&gt; req-a-project-has-a-declared-root-holding-one-asset-store</p>
            </li>
            <li class="task" id="task-openspec-config-yaml" data-change="added" data-state="dropped">
              <h3>The project's configuration <span class="hs-id">task-openspec-config-yaml</span></h3>
              <p data-field="note">source: openspec/config.yaml</p>
              <p data-field="note">destination: none</p>
              <p data-field="note">disposition: dropped</p>
              <p data-field="reason">The destination's declaration carries no authoring rules or project context, so there is nowhere for this content to go. Its context, rules, and operations blocks were commented-out examples in the source, so they were absent rather than dropped.</p>
            </li>
"#;

/// The survey payload those two items were written against.
const PAYLOAD: &str = r#"{"entities":[
  {"path":"openspec/specs/project-bootstrap/spec.md","kind":"accepted-specification"},
  {"path":"openspec/config.yaml","kind":"configuration"}
]}"#;

/// Establishes a project holding a plan at `htmlspec/changes/m/tasks.html`, and a survey payload.
///
/// Built with `htmlspec init` and `htmlspec new document tasks` rather than by hand, so the plan
/// is the document the project's own operations produce with its example units replaced. A
/// hand-built document could satisfy these checks and differ from what the workflow will write.
fn project_with_a_plan(label: &str) -> (Scratch, PathBuf) {
    let scratch = Scratch::new(label);
    let (code, out, err) = run(&scratch.0, &["init", "--tools", "none", "--name", label]);
    assert_eq!(code, 0, "init failed: {out}{err}");

    let relative = "htmlspec/changes/m/tasks.html";
    let (code, out, err) = run(
        &scratch.0,
        &[
            "new",
            "document",
            "tasks",
            relative,
            "--subject",
            "Migration plan",
        ],
    );
    assert_eq!(code, 0, "the tasks document was not created: {out}{err}");

    let path = scratch.0.join(relative);
    let template = std::fs::read_to_string(&path).expect("the template is readable");
    // The template's two example task units, replaced by the plan's own. Located by their
    // identifiers rather than by position, so a template that gains a unit fails here loudly
    // instead of producing a plan with an example task still in it.
    let opening = template
        .find("<li class=\"task\" id=\"task-example\"")
        .expect("the template writes an example task");
    let closing = template
        .rfind("</li>")
        .map(|at| at + "</li>".len())
        .expect("the example tasks close");
    let plan = format!("{}{ITEMS}{}", &template[..opening], &template[closing..]);
    std::fs::write(&path, &plan).expect("the plan is written");
    std::fs::write(scratch.0.join("survey.json"), PAYLOAD).expect("the payload is written");

    (scratch, path)
}

/// The plan validates as a `tasks` document, formats canonically, and passes the coverage gate.
///
/// One test rather than three, because the claim is that a single document satisfies all three at
/// once. Three tests each building their own plan could pass while no one shape satisfied
/// everything.
#[test]
fn the_plans_shape_validates_formats_and_passes_the_gate() {
    let (scratch, path) = project_with_a_plan("shape");
    let relative = "htmlspec/changes/m/tasks.html";

    let (code, out, err) = run(&scratch.0, &["validate", relative]);
    assert_eq!(code, 0, "the plan does not validate: {out}{err}");
    assert!(out.contains("accepted"), "{out}");

    // Formatted, then checked. The formatter is what would rewrite a shape that leaned on
    // attribute order or indentation, so a plan surviving both is a fixed point.
    let (code, out, err) = run(&scratch.0, &["format", relative]);
    assert_eq!(code, 0, "the plan could not be formatted: {out}{err}");
    let (code, out, err) = run(&scratch.0, &["format", "--check", relative]);
    assert_eq!(
        code, 0,
        "the plan is not canonical after formatting: {out}{err}"
    );

    // And the gate reads the same document the formatter just rewrote, so the two readers cannot
    // disagree about what the plan says.
    let (code, out, err) = run(
        &scratch.0,
        &["migrate", "promote", relative, "--survey", "survey.json"],
    );
    assert_eq!(code, 0, "the gate refused a complete plan: {out}{err}");
    assert!(
        out.contains("every surveyed entity has a disposition"),
        "{out}"
    );
    // The report says what the check did *not* do, because a passing coverage check reads as
    // approval of the plan and is not one.
    assert!(err.contains("is right"), "{err}");

    // Formatting did not move the facts out of the model's reach.
    let formatted = std::fs::read_to_string(&path).expect("the formatted plan is readable");
    for note in [
        "source: openspec/config.yaml",
        "destination: none",
        "disposition: dropped",
    ] {
        assert!(formatted.contains(note), "the formatter lost {note}");
    }
}

/// Removing one entity from a complete plan fails the gate, through the real program.
///
/// The unit test pins the comparison and this pins the exit code. A gate returning the right
/// verdict while `main` exited zero would pass every unit test in the crate.
#[test]
fn a_plan_missing_one_entity_refuses_with_the_violation_code() {
    let (scratch, path) = project_with_a_plan("missing");
    let relative = "htmlspec/changes/m/tasks.html";

    let plan = std::fs::read_to_string(&path).expect("readable");
    let opening = plan
        .find("<li class=\"task\" id=\"task-openspec-config-yaml\"")
        .expect("the configuration item is present");
    let closing = plan[opening..]
        .find("</li>")
        .map(|at| opening + at + "</li>".len())
        .expect("it closes");
    std::fs::write(&path, format!("{}{}", &plan[..opening], &plan[closing..]))
        .expect("the shortened plan is written");

    // Still a valid document. The omission is not a document defect, which is exactly why a
    // coverage gate exists rather than a validation rule.
    let (code, out, err) = run(&scratch.0, &["validate", relative]);
    assert_eq!(
        code, 0,
        "the shortened plan should still validate: {out}{err}"
    );

    let (code, out, err) = run(
        &scratch.0,
        &["migrate", "promote", relative, "--survey", "survey.json"],
    );
    assert_eq!(code, 1, "the gate accepted an incomplete plan: {out}{err}");
    assert!(out.contains("openspec/config.yaml"), "{out}");
    assert!(err.contains("promotion does not proceed"), "{err}");
}
