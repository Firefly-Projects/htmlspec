//! Promotion end to end, through the compiled program.
//!
//! `staging.rs` unit-tests each structural check and `plan.rs` unit-tests the coverage comparison.
//! This file covers what neither can reach: the four stages in order, the exit codes, and the
//! whole-set reporting behavior over a tree big enough for a systematic fault to look systematic.
//!
//! **The last of those is the one this file exists for.** A mapping fault is not one document's
//! defect: a rule that is wrong for one scenario is wrong for every scenario. So the test below
//! breaks one mapping rule across many documents and asserts the report shows a class of failure
//! rather than an instance — which is the behavior the check-last requirement was written for, and
//! therefore the behavior that is checked rather than described.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The exit codes, restated rather than imported. These are a public contract, and a test reading
/// them from the source could not notice them changing.
const SUCCESS: i32 = 0;
const DOCUMENT_VIOLATION: i32 = 1;

/// The binary under test.
fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// Runs the binary from inside `directory`.
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
            std::env::temp_dir().join(format!("htmlspec-promote-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path.canonicalize().expect("the scratch path resolves"))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Every file beneath `directory`, with its bytes, sorted by path.
///
/// The comparison that proves a tree is unchanged. Names alone would miss an edit and lengths
/// alone would miss a swap, so this holds the contents.
fn snapshot(directory: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    let mut pending = vec![directory.to_owned()];
    while let Some(next) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if let Ok(bytes) = std::fs::read(&path) {
                found.push((
                    path.strip_prefix(directory).unwrap_or(&path).to_owned(),
                    bytes,
                ));
            }
        }
    }
    found.sort();
    found
}

/// A migration in progress: a staged project, a plan, a survey payload, and an empty destination.
struct Migration {
    scratch: Scratch,
    staged: PathBuf,
    destination: PathBuf,
}

impl Migration {
    /// Stages a project holding `specifications` accepted specifications and one archived change.
    ///
    /// Built with `htmlspec init` and `htmlspec new document` rather than by hand, so the staged
    /// tree is what the toolchain's own operations produce. A hand-built tree could pass these
    /// checks and differ from what an agent following the workflow will write.
    fn stage(label: &str, specifications: usize) -> Self {
        Self::stage_carrying(label, specifications, &[])
    }

    /// Stages the same project, with `carried` files placed in its archived change and carried by
    /// the plan. Each is a path beneath the archived change and the bytes to put there.
    fn stage_carrying(label: &str, specifications: usize, carried: &[(&str, String)]) -> Self {
        let scratch = Scratch::new(label);
        let staged = scratch.0.join("staging");
        let destination = scratch.0.join("repository");
        std::fs::create_dir_all(&staged).expect("the staging directory");
        std::fs::create_dir_all(&destination).expect("the destination directory");

        let (code, out, err) = run(&staged, &["init", "--tools", "none", "--name", "staged"]);
        assert_eq!(code, SUCCESS, "init failed: {out}{err}");

        let mut sources = Vec::new();
        for index in 0..specifications {
            let relative = format!("htmlspec/specs/capability-{index}/spec.html");
            let (code, out, err) = run(
                &staged,
                &[
                    "new",
                    "document",
                    "specification",
                    &relative,
                    "--subject",
                    &format!("Capability {index}"),
                ],
            );
            assert_eq!(
                code, SUCCESS,
                "the specification was not created: {out}{err}"
            );
            sources.push((
                format!("openspec/specs/capability-{index}/spec.md"),
                relative,
            ));
        }

        // One archived change recording a version, so the version checks have something to read.
        let archive = staged.join("htmlspec/changes/archive/2026-08-10-html-spec");
        std::fs::create_dir_all(&archive).expect("the archived change");
        std::fs::write(archive.join("VERSION"), "0.1.0\n").expect("the recorded version");
        sources.push((
            String::from("openspec/changes/archive/2026-08-10-html-spec/VERSION"),
            String::from("htmlspec/changes/archive/2026-08-10-html-spec/VERSION"),
        ));
        for (relative, bytes) in carried {
            let path = archive.join(relative);
            std::fs::create_dir_all(path.parent().expect("a carried file has a parent"))
                .expect("the carried file's directory");
            std::fs::write(&path, bytes).expect("the carried file");
            sources.push((
                format!("openspec/changes/archive/2026-08-10-html-spec/{relative}"),
                format!("htmlspec/changes/archive/2026-08-10-html-spec/{relative}"),
            ));
        }

        // The declaration equals the greatest archived version, so the next archival must advance.
        let declaration = staged.join("spec.nix");
        let text = std::fs::read_to_string(&declaration).expect("the declaration is readable");
        std::fs::write(
            &declaration,
            text.replace("version = \"0.0.0\"", "version = \"0.1.0\""),
        )
        .expect("the declaration is rewritten");
        std::fs::write(
            staged.join("CHANGELOG.md"),
            "# Changelog\n\n## 0.1.0\n\nThe baseline release.\n",
        )
        .expect("the release record");

        // The plan and the payload, outside the staged tree — the plan lands in the migrated
        // project as the migration's own change, and a plan inside the staging root here would be
        // a stray document the conformance checks would read.
        std::fs::write(
            scratch.0.join("survey.json"),
            serde_json::to_string(&serde_json::json!({
                "entities": sources
                    .iter()
                    .map(|(source, _)| serde_json::json!({ "path": source, "kind": "x" }))
                    .collect::<Vec<_>>(),
            }))
            .expect("the payload serializes"),
        )
        .expect("the payload is written");

        let items: String = sources
            .iter()
            .enumerate()
            .map(|(index, (source, destination))| {
                format!(
                    "<li class=\"task\" id=\"task-entity-{index}\" data-change=\"added\" \
                     data-state=\"todo\"><h3>Entity {index}</h3>\
                     <p data-field=\"note\">source: {source}</p>\
                     <p data-field=\"note\">destination: {destination}</p>\
                     <p data-field=\"note\">disposition: converted</p></li>"
                )
            })
            .collect();
        std::fs::write(
            scratch.0.join("plan.html"),
            format!(
                "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
                 <meta content=\"0.1.0\" name=\"htmlspec-version\">\
                 <meta content=\"tasks\" name=\"htmlspec-kind\">\
                 <title>Migration plan</title></head><body><main>{items}</main></body></html>"
            ),
        )
        .expect("the plan is written");

        Self {
            scratch,
            staged,
            destination,
        }
    }

    /// Runs `migrate promote` with `extra` appended.
    fn promote(&self, extra: &[&str]) -> (i32, String, String) {
        let mut args = vec![
            "migrate",
            "promote",
            "plan.html",
            "--survey",
            "survey.json",
            "--staged",
            "staging",
            "--into",
            "repository",
        ];
        args.extend_from_slice(extra);
        run(&self.scratch.0, &args)
    }
}

/// The whole promotion runs, and the promoted project is one the toolchain accepts.
#[test]
fn a_complete_migration_promotes_and_the_result_is_a_project() {
    let migration = Migration::stage("complete", 2);

    let (code, out, err) = migration.promote(&[]);
    assert_eq!(code, SUCCESS, "promotion refused: {out}{err}");
    assert!(
        out.contains("promoted: htmlspec, spec.nix, CHANGELOG.md"),
        "{out}"
    );
    // The report says the source survives, because that is what makes a wrong migration
    // recoverable and a contributor will not read the specification to find out.
    assert!(err.contains("source project is untouched"), "{err}");

    // The destination is a project, and the staging location holds none of it.
    for entry in ["htmlspec", "spec.nix", "CHANGELOG.md"] {
        assert!(
            migration.destination.join(entry).exists(),
            "{entry} did not arrive"
        );
        assert!(
            !migration.staged.join(entry).exists(),
            "{entry} is still staged"
        );
    }

    // And the promoted project passes the checks that ran against it in staging, from its new
    // location — which is the whole claim of mirroring the destination's paths.
    let (code, out, err) = run(&migration.destination, &["validate"]);
    assert_eq!(
        code, SUCCESS,
        "the promoted project does not validate: {out}{err}"
    );
    let (code, out, err) = run(&migration.destination, &["assets"]);
    assert_eq!(code, SUCCESS, "the promoted project's assets: {out}{err}");
    let (code, out, err) = run(&migration.destination, &["format", "--check"]);
    assert_eq!(
        code, SUCCESS,
        "the promoted project is not canonical: {out}{err}"
    );
}

/// The version gate refuses in the migrated project without an advance.
///
/// The migration's own claim about history, checked where it matters: the declaration equals the
/// greatest migrated version, so the project's next archival is forced to advance past its real
/// history rather than past a version it has already passed.
#[test]
fn the_migrated_projects_version_gate_refuses_without_an_advance() {
    let migration = Migration::stage("version-gate", 1);
    assert_eq!(migration.promote(&[]).0, SUCCESS);

    let (code, out, err) = run(&migration.destination, &["new", "change", "try"]);
    assert_eq!(code, SUCCESS, "{out}{err}");

    let (code, out, err) = run(&migration.destination, &["archive", "try", "--check"]);
    assert_ne!(
        code, SUCCESS,
        "archival passed without an advance in the migrated project: {out}{err}"
    );
    assert!(err.contains("still 0.1.0"), "{err}");
    // The refusal names the carried version, which is the proof that the gate read the file the
    // migration carried rather than a directory name or an initial value.
    assert!(err.contains("Raise `version`"), "{err}");
}

/// A migration abandoned before promotion leaves the destination byte-for-byte unchanged, and the
/// source too.
///
/// Checked at **each stage boundary** rather than only at the last, because the requirement is
/// about every stage: a coverage failure, a staging fault, a conformance failure, and a clean
/// check-mode run must all leave the destination as it was.
#[test]
fn an_abandoned_migration_leaves_the_destination_unchanged_at_every_stage() {
    let migration = Migration::stage("abandoned", 2);
    let destination_before = snapshot(&migration.destination);
    let staged_before = snapshot(&migration.staged);

    // Stage 4 boundary: the plan does not cover the survey.
    let payload = migration.scratch.0.join("survey.json");
    let text = std::fs::read_to_string(&payload).expect("readable");
    let mut extended: serde_json::Value = serde_json::from_str(&text).expect("parses");
    extended["entities"]
        .as_array_mut()
        .expect("an array")
        .push(serde_json::json!({ "path": "openspec/config.yaml", "kind": "configuration" }));
    std::fs::write(&payload, extended.to_string()).expect("written");

    let (code, out, _) = migration.promote(&[]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}");
    assert!(out.contains("openspec/config.yaml"), "{out}");
    assert_eq!(
        snapshot(&migration.destination),
        destination_before,
        "a coverage failure changed the destination"
    );

    // Restored, so the next boundary is reached for its own reason.
    std::fs::write(&payload, text).expect("restored");

    // Stage 5 boundary: a staging fault. The declaration no longer equals the greatest archived
    // version, which is the fault that would let the migrated project's next archival pass
    // without advancing.
    let declaration = migration.staged.join("spec.nix");
    let declared = std::fs::read_to_string(&declaration).expect("readable");
    std::fs::write(
        &declaration,
        declared.replace("version = \"0.1.0\"", "version = \"0.0.1\""),
    )
    .expect("written");

    let (code, out, _) = migration.promote(&[]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}");
    assert!(out.contains("staging fault"), "{out}");
    assert_eq!(
        snapshot(&migration.destination),
        destination_before,
        "a staging fault changed the destination"
    );
    std::fs::write(&declaration, &declared).expect("restored");

    // Stage 6 boundary: a conformance failure.
    let document = migration
        .staged
        .join("htmlspec/specs/capability-0/spec.html");
    let source = std::fs::read_to_string(&document).expect("readable");
    std::fs::write(
        &document,
        source.replace("id=\"req-example\"", "id=\"REQ-Example\""),
    )
    .expect("written");

    let (code, out, _) = migration.promote(&[]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}");
    assert!(out.contains("conformance diagnostic"), "{out}");
    assert_eq!(
        snapshot(&migration.destination),
        destination_before,
        "a conformance failure changed the destination"
    );
    std::fs::write(&document, source).expect("restored");

    // Stage 7 boundary, declined: every check passes and `--check` moves nothing.
    let (code, out, err) = migration.promote(&["--check"]);
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(err.contains("nothing was moved"), "{err}");
    assert_eq!(
        snapshot(&migration.destination),
        destination_before,
        "check mode changed the destination"
    );
    // And the staged tree is byte-for-byte as it was through all four, so nothing above consumed
    // its input either.
    assert_eq!(
        snapshot(&migration.staged),
        staged_before,
        "the staged tree changed while migration was being checked"
    );
}

/// A systematic mapping fault is reported as a class rather than as one document's defect.
///
/// **The case the check-last requirement was written for.** One mapping rule is broken here — the
/// identifier a requirement's heading derives — and it is broken the same way in every document,
/// which is exactly how a wrong rule behaves. The assertion is not that a diagnostic appears: it is
/// that *every* occurrence appears in one report, so the shape of the fault is visible.
///
/// A run that halted at the first would report one document and leave a contributor repairing it
/// by hand before continuing into the rest — in a tree that is now partly migrated. That is the
/// failure mode, and reporting the whole set is what points at the real repair.
#[test]
fn a_systematic_mapping_fault_is_reported_as_a_whole_set() {
    const DOCUMENTS: usize = 12;
    let migration = Migration::stage("systematic", DOCUMENTS);

    // The broken rule: an identifier that keeps its heading's capitals. Every requirement in every
    // document gets it, because a rule is applied to everything it applies to.
    for index in 0..DOCUMENTS {
        let path = migration
            .staged
            .join(format!("htmlspec/specs/capability-{index}/spec.html"));
        let source = std::fs::read_to_string(&path).expect("readable");
        std::fs::write(
            &path,
            source
                .replace("\"req-example\"", "\"req-Example\"")
                .replace(">req-example<", ">req-Example<"),
        )
        .expect("written");
    }

    let (code, out, err) = migration.promote(&[]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");

    // Every document reports, and the count is stated before the list — because "12 documents
    // report the same diagnostic" is a different finding from "capability-3 is broken", and it is
    // the finding that points at the mapping rule.
    for index in 0..DOCUMENTS {
        assert!(
            out.contains(&format!("capability-{index}/spec.html")),
            "capability-{index} was not reported, so the run stopped early:\n{out}"
        );
    }
    assert!(
        out.contains(&format!("across {DOCUMENTS} staged document")),
        "the report does not state how many documents were checked:\n{out}"
    );
    // And the refusal says why the whole set is shown, so a contributor reading 12 identical
    // diagnostics knows to look for one rule rather than 12 mistakes.
    assert!(err.contains("systematic"), "{err}");

    // The destination is still empty. A systematic fault found at stage 6 leaves nothing behind.
    assert!(
        snapshot(&migration.destination).is_empty(),
        "a refused promotion left something in the destination"
    );
}

/// A stale asset reference is reported as a reference fault, not as a formatting fault.
///
/// **Two causes needing different repairs.** A staged document can differ from its canonical form
/// because its serialization is not canonical, or because it names asset bytes the store does not
/// hold. `htmlspec format` already distinguishes them, and the migration's own check did not — it
/// reported both as "not canonically formatted", which points a contributor at the serializer for a
/// reference fault. A review of the pull request found that, and this pins the repair.
///
/// The distinction matters most here of all places, because a migration produces documents in bulk:
/// a wrong asset prefix is wrong in every document of a kind, so the misdirected diagnostic arrives
/// dozens of times at once.
#[test]
fn a_stale_asset_reference_is_not_reported_as_a_formatting_fault() {
    let migration = Migration::stage("stale-reference", 2);
    let document = migration
        .staged
        .join("htmlspec/specs/capability-0/spec.html");
    let source = std::fs::read_to_string(&document).expect("readable");

    // A reference naming an address the store does not hold, with the document otherwise canonical.
    // The digits are a valid address shape and no stored asset has them. The current address is read
    // from the embedded asset table rather than written here, because it moves whenever the
    // stylesheet's bytes do, and a literal went stale the first time they did.
    let stylesheet = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == "htmlspec.css")
        .expect("the stylesheet is embedded");
    let stale = source.replace(stylesheet.path, "sha256-0123456789abcdef/htmlspec.css");
    assert_ne!(stale, source, "the stylesheet address was not found");
    std::fs::write(&document, &stale).expect("written");

    let (code, out, err) = migration.promote(&[]);
    assert_eq!(code, DOCUMENT_VIOLATION, "{out}{err}");

    // The report names the reference and what it must be, so the repair is the reference rather
    // than the serializer.
    assert!(
        out.contains("names bytes this project does not store"),
        "the stale reference was not reported as one:\n{out}"
    );
    assert!(
        out.contains("htmlspec.css"),
        "the report does not name the asset:\n{out}"
    );
    // And it is not misreported as a formatting fault, which is the defect this pins.
    assert!(
        !out.contains("capability-0/spec.html: not canonically formatted"),
        "a reference fault was reported as a formatting fault:\n{out}"
    );
}

/// A record carried as evidence: the specification template as the program writes it, declaring
/// contract `0.1.0` and naming the stylesheet revisions this repository's own HTML evidence names,
/// whose bytes a freshly established store does not hold.
///
/// The same shape `tests/cli.rs` builds for the commands. Restated because each test file here
/// keeps its own helpers.
fn a_record() -> String {
    let mut document = htmlspec_assets::render_kind("specification", "A record", "assets")
        .expect("the specification template exists");
    let declared = document
        .split("<meta content=\"")
        .find_map(|rest| rest.split_once("\" name=\"htmlspec-version\""))
        .map(|(version, _)| version.to_owned())
        .expect("the template declares its contract version");
    document = document.replace(
        &format!("<meta content=\"{declared}\" name=\"htmlspec-version\""),
        "<meta content=\"0.1.0\" name=\"htmlspec-version\"",
    );
    for (name, superseded) in [
        ("htmlspec.css", "sha256-e4f5de7cecc4f691/htmlspec.css"),
        (
            "htmlspec-print.css",
            "sha256-0350261bb5666167/htmlspec-print.css",
        ),
    ] {
        let current = htmlspec_assets::all_assets()
            .into_iter()
            .find(|asset| asset.name() == name)
            .unwrap_or_else(|| panic!("{name} is embedded"));
        let replaced = document.replace(current.path, superseded);
        assert_ne!(replaced, document, "the template does not reference {name}");
        document = replaced;
    }
    document
}

/// project-migration's "Carried evidence in the destination format does not gate promotion".
///
/// Evidence crosses unchanged, and a document in this format that no longer conforms is preserved
/// rather than carried as a document — both land beneath a change's `evidence/`. Before the
/// exclusion, the conformance check read those records as documents and refused the whole
/// migration on their account: measured against this repository's three HTML evidence files, 13
/// diagnostics and nothing else wrong in the tree.
#[test]
fn carried_evidence_in_this_format_does_not_gate_promotion() {
    let record = a_record();
    let migration = Migration::stage_carrying(
        "carried-evidence",
        1,
        &[
            ("evidence/a-record.html", record.clone()),
            ("evidence/converted/design.html", record.clone()),
        ],
    );

    let (code, out, err) = migration.promote(&["--check"]);
    assert_eq!(
        code, SUCCESS,
        "the check read carried evidence as documents:\n{out}{err}"
    );
    assert!(
        !out.contains("a-record.html") && !out.contains("converted/design.html"),
        "the report names carried evidence:\n{out}"
    );

    let (code, out, err) = migration.promote(&[]);
    assert_eq!(code, SUCCESS, "promotion refused: {out}{err}");
    for relative in ["evidence/a-record.html", "evidence/converted/design.html"] {
        let arrived = migration
            .destination
            .join("htmlspec/changes/archive/2026-08-10-html-spec")
            .join(relative);
        assert_eq!(
            std::fs::read_to_string(&arrived)
                .unwrap_or_else(|error| panic!("{relative} did not arrive: {error}")),
            record,
            "{relative} did not arrive byte-identical"
        );
    }
}

/// The archived change's directory, as [`Migration::stage`] stages it.
const ARCHIVED: &str = "htmlspec/changes/archive/2026-08-10-html-spec";

/// A migrated change's specification and tasks, rendered as `htmlspec new document` writes them,
/// with the specification's requirement renamed to `req-only-in-the-change` and each task naming the
/// identifiers given in `data-refs`.
///
/// Rendered from the templates rather than written by hand, so the documents are the toolchain's own
/// and validate and format as it writes them; only the identifiers and the references are this
/// test's.
fn a_migrated_change(task_refs: [&str; 2]) -> Vec<(&'static str, String)> {
    let assets = "../../../assets";
    let specification = htmlspec_assets::render_kind("specification", "A migrated change", assets)
        .expect("the specification template exists")
        .replace("req-example", "req-only-in-the-change");
    let tasks = htmlspec_assets::render_kind("tasks", "A migrated change", assets)
        .expect("the tasks template exists");
    let [first, second] = task_refs;
    let tasks = tasks
        .replace(
            "<li class=\"task\" id=\"task-example\" data-change=\"added\" data-state=\"todo\">",
            &format!(
                "<li class=\"task\" id=\"task-example\" data-change=\"added\" data-state=\"todo\" \
                 data-refs=\"{first}\">"
            ),
        )
        .replace(
            "<li class=\"task\" id=\"task-dropped-example\" data-change=\"added\" \
             data-state=\"dropped\">",
            &format!(
                "<li class=\"task\" id=\"task-dropped-example\" data-change=\"added\" \
                 data-state=\"dropped\" data-refs=\"{second}\">"
            ),
        );
    assert_eq!(
        tasks.matches("data-refs=").count(),
        2,
        "the tasks template no longer holds the two example tasks this test gives references"
    );
    vec![("specification.html", specification), ("tasks.html", tasks)]
}

/// Renames the staged accepted specification's requirement, so its identifier is one that only an
/// accepted specification declares.
fn rename_the_accepted_requirement(migration: &Migration) {
    let document = migration
        .staged
        .join("htmlspec/specs/capability-0/spec.html");
    let source = std::fs::read_to_string(&document).expect("readable");
    let renamed = source.replace("req-example", "req-only-accepted");
    assert_ne!(
        renamed, source,
        "the accepted specification holds no req-example"
    );
    std::fs::write(&document, renamed).expect("written");
}

/// project-migration's "A unit the author's words name is referenced wherever the migrated project
/// resolves it", scenario "A reference checked before promotion", first half.
///
/// A staged task names, in `data-refs`, a requirement that only its change's staged specification
/// declares, and another names one that only a staged accepted specification declares. Promotion's
/// conformance check resolves both, because the staged project resolves as a project and places each
/// staged document as promotion will (design decision 7). **This pins that behavior rather than
/// driving a fix:** it passed on the first run, with no change to the staging code.
#[test]
fn a_staged_reference_across_a_change_is_checked_as_after_promotion() {
    let migration = Migration::stage_carrying(
        "staged-reference",
        1,
        &a_migrated_change(["req-only-in-the-change", "req-only-accepted"]),
    );
    rename_the_accepted_requirement(&migration);

    let (code, out, err) = migration.promote(&["--check"]);
    assert_eq!(
        code, SUCCESS,
        "a staged reference across a change did not resolve:\n{out}{err}"
    );
    assert!(!out.contains("refs/"), "a reference was reported:\n{out}");
    // Both references are in the tree the check read, so the pass is not an absence of references.
    let tasks = std::fs::read_to_string(migration.staged.join(ARCHIVED).join("tasks.html"))
        .expect("the staged tasks are readable");
    assert!(tasks.contains("data-refs=\"req-only-in-the-change\""));
    assert!(tasks.contains("data-refs=\"req-only-accepted\""));
}

/// The same scenario's second half: a staged reference that resolves nowhere holds promotion back.
///
/// The identifier is one nothing in the staged project declares, so the check reports
/// `refs/unresolved`, exits 1, and moves nothing. Before promotion is the last point a dangling
/// reference can be repaired in the staging location rather than in the contributor's repository.
#[test]
fn a_dangling_staged_reference_holds_promotion_back() {
    let migration = Migration::stage_carrying(
        "dangling-reference",
        1,
        &a_migrated_change(["req-only-in-the-change", "req-nothing-declares"]),
    );
    rename_the_accepted_requirement(&migration);

    let (code, out, err) = migration.promote(&[]);
    assert_eq!(
        code, DOCUMENT_VIOLATION,
        "a dangling staged reference did not hold promotion back:\n{out}{err}"
    );
    assert!(out.contains("refs/unresolved"), "{out}");
    assert!(out.contains("req-nothing-declares"), "{out}");
    // Only the dangling one is reported: the reference to the change's own requirement resolved.
    assert_eq!(out.matches("refs/unresolved").count(), 1, "{out}");
    assert!(
        snapshot(&migration.destination).is_empty(),
        "a refused promotion left something in the destination"
    );
}

/// Validating a document while drafting it is not prohibited.
///
/// The obligation is on where a migration *ends*, never on where it checks. A rule that fired on a
/// drafting validation would forbid the cheapest feedback an agent has, so this runs `validate` and
/// `format --check` against the staged tree mid-migration and asserts both work — and that the
/// later promotion is unaffected by their having run.
#[test]
fn checking_a_document_while_drafting_it_is_permitted() {
    let migration = Migration::stage("drafting", 2);

    // Mid-migration, before promotion, from inside the staged project. This is what an agent does
    // after writing each document.
    let (code, out, err) = run(&migration.staged, &["validate"]);
    assert_eq!(
        code, SUCCESS,
        "validating a staged document was refused: {out}{err}"
    );
    let (code, out, err) = run(
        &migration.staged,
        &["validate", "htmlspec/specs/capability-0/spec.html"],
    );
    assert_eq!(
        code, SUCCESS,
        "validating one staged document was refused: {out}{err}"
    );
    let (code, out, err) = run(&migration.staged, &["format", "--check"]);
    assert_eq!(
        code, SUCCESS,
        "checking a staged document's format was refused: {out}{err}"
    );

    // And promotion still proceeds. Nothing about having checked mid-flight changes the outcome,
    // which is the difference between an obligation about where migration ends and a prohibition
    // on validating.
    let (code, out, err) = migration.promote(&[]);
    assert_eq!(code, SUCCESS, "{out}{err}");
}

/// The coverage check is usable before anything is staged.
///
/// The stage order made reachable rather than merely documented: an agent runs the gate at stage 4,
/// with a plan and a survey and no staged tree, and runs the whole of promotion at stage 7.
#[test]
fn the_coverage_check_runs_before_anything_is_staged() {
    let migration = Migration::stage("coverage-only", 1);
    let (code, out, err) = run(
        &migration.scratch.0,
        &["migrate", "promote", "plan.html", "--survey", "survey.json"],
    );
    assert_eq!(code, SUCCESS, "{out}{err}");
    assert!(
        out.contains("every surveyed entity has a disposition"),
        "{out}"
    );
    // And it says what it did not do, and how to do the rest.
    assert!(err.contains("--staged"), "{err}");
    assert!(
        snapshot(&migration.destination).is_empty(),
        "the coverage check touched the destination"
    );
}
