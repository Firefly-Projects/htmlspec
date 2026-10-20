//! The reference-resolution fixtures: a document, where it sits, and what the places it reaches
//! declare.
//!
//! A conformance fixture in `conformance.rs` is one document, and it is loose: it reaches only
//! itself. document-schema's "Cross-references address units by identifier" makes a reference's
//! resolution a fact about **a document and its surroundings**: the same task resolves or not
//! depending on what its change's specification and the accepted specifications declare. So these
//! fixtures are built here, as a placement, a document, and the declarations of each place it
//! reaches, which is the departure `topology.rs` and `runs.rs` make for the same kind of reason.
//!
//! Everything drives the public API only. A scope is built through
//! [`htmlspec_validate::references`]'s public types, and the documents its places declare are
//! parsed and projected exactly as the command line does, with no file read.

use std::path::{Path, PathBuf};

use htmlspec_assets::Store;
use htmlspec_parse::{parse_document, project, Projection};
use htmlspec_validate::references::{Declared, Place, Placement, Scope, Unread};
use htmlspec_validate::{engine, rules, Report, Strictness};

/// A document of `kind` from its template, with every example identifier renamed to end in
/// `suffix`, and each `(from, to)` edit applied after.
fn document(kind: &str, suffix: &str, edits: &[(&str, &str)]) -> String {
    let mut text = htmlspec_assets::render_kind(kind, "A reference fixture", "assets")
        .expect("the template renders")
        .replace("-example", &format!("-{suffix}"));
    for (from, to) in edits {
        assert!(
            text.contains(from),
            "{from:?} is not in the {kind} template"
        );
        text = text.replacen(from, to, 1);
    }
    text
}

/// The task in a tasks document, naming `refs`.
fn task_naming(refs: &str) -> String {
    document(
        "tasks",
        "here",
        &[(
            "id=\"task-here\"",
            &format!("id=\"task-here\" data-refs=\"{refs}\""),
        )],
    )
}

/// A specification declaring `req-{suffix}`, from the template.
fn specification(suffix: &str) -> String {
    document("specification", suffix, &[])
}

/// A specification declaring `req-{suffix}`, marked `removed`.
fn withdrawing(suffix: &str) -> String {
    document(
        "specification",
        suffix,
        &[(
            &format!("id=\"req-{suffix}\" data-change=\"added\""),
            &format!("id=\"req-{suffix}\" data-change=\"removed\""),
        )],
    )
}

fn projected(source: &str) -> Projection {
    project(&parse_document(source).document)
}

/// What the documents of one place declare.
fn declaring(sources: &[String]) -> Declared {
    let projections: Vec<Projection> = sources.iter().map(|source| projected(source)).collect();
    Declared::gather(projections.iter(), Vec::new())
}

/// Validates `source` at `placement`, with `change` and `accepted` as what its change's
/// specification and the accepted specifications declare.
fn validate_at(source: &str, placement: Placement, change: Declared, accepted: Declared) -> Report {
    let mut change = Some(change);
    let mut accepted = Some(accepted);
    let scope = Scope::gather(placement, |place| match place {
        Place::ChangeSpecification => change.take().unwrap_or_default(),
        Place::AcceptedSpecifications => accepted.take().unwrap_or_default(),
    });
    let outcome = parse_document(source);
    let projection = project(&outcome.document);
    engine::run(
        &outcome,
        &projection,
        &Store::Absent,
        &scope,
        rules::ALL,
        Strictness::Default,
    )
}

/// The `refs/*` rules a report names, each once.
fn refs(report: &Report) -> Vec<String> {
    let mut found: Vec<String> = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule.to_string())
        .filter(|rule| rule.starts_with("refs/"))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// The message of the one `refs/unresolved` diagnostic a report holds.
fn unresolved_message(report: &Report) -> String {
    let messages: Vec<&str> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule.to_string() == "refs/unresolved")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages.len(), 1, "{messages:?}");
    messages[0].to_owned()
}

// --- document-schema's "Cross-references address units by identifier" ------------------------

/// "A task references a requirement in its change's specification".
#[test]
fn a_task_resolves_in_its_change_specification() {
    let report = validate_at(
        &task_naming("req-x"),
        Placement::ChangeDocument,
        declaring(&[specification("x")]),
        Declared::default(),
    );
    assert!(refs(&report).is_empty(), "{:?}", report.diagnostics);
}

/// "A task references a requirement only the accepted contract holds".
#[test]
fn a_task_resolves_in_the_accepted_specifications() {
    let report = validate_at(
        &task_naming("req-y"),
        Placement::ChangeDocument,
        declaring(&[specification("x")]),
        declaring(&[specification("y")]),
    );
    assert!(refs(&report).is_empty(), "{:?}", report.diagnostics);
}

/// "A change withdraws a requirement the accepted contract still holds": the change's
/// specification is reached first.
#[test]
fn the_change_specifications_withdrawal_comes_before_the_accepted_unit() {
    let report = validate_at(
        &task_naming("req-y"),
        Placement::ChangeDocument,
        declaring(&[withdrawing("y")]),
        declaring(&[specification("y")]),
    );
    assert_eq!(refs(&report), vec!["refs/removed-target"]);
}

/// "Accepted specifications reference one another".
#[test]
fn an_accepted_specification_resolves_in_another() {
    let source = document(
        "specification",
        "b",
        &[(
            "id=\"evidence-b\" data-change=\"added\" data-refs=\"req-b\"",
            "id=\"evidence-b\" data-change=\"added\" data-refs=\"req-a\"",
        )],
    );
    let report = validate_at(
        &source,
        Placement::AcceptedSpecification,
        Declared::default(),
        declaring(&[specification("a")]),
    );
    assert!(refs(&report).is_empty(), "{:?}", report.diagnostics);
}

/// "An identifier two accepted specifications declare".
#[test]
fn an_identifier_two_accepted_specifications_declare_resolves() {
    let report = validate_at(
        &task_naming("req-x"),
        Placement::ChangeDocument,
        Declared::default(),
        declaring(&[specification("x"), specification("x")]),
    );
    assert!(refs(&report).is_empty(), "{:?}", report.diagnostics);
}

/// "A reference to another change": another change's specification is no place this one reaches,
/// so what it declares is never gathered.
#[test]
fn another_changes_specification_is_not_reached() {
    let report = validate_at(
        &task_naming("req-x"),
        Placement::ChangeDocument,
        Declared::default(),
        Declared::default(),
    );
    assert_eq!(refs(&report), vec!["refs/unresolved"]);
}

/// "A document that reaches only itself": a loose document is not given the places it would reach
/// in a change, whatever they declare.
#[test]
fn a_loose_document_reaches_only_itself() {
    let report = validate_at(
        &task_naming("req-x"),
        Placement::Loose,
        declaring(&[specification("x")]),
        declaring(&[specification("x")]),
    );
    assert_eq!(refs(&report), vec!["refs/unresolved"]);
    assert!(
        unresolved_message(&report).contains("no unit in the document declares that identifier"),
        "a loose document's message changed: {}",
        unresolved_message(&report)
    );
}

/// "A runbook's document names an accepted requirement".
#[test]
fn a_runbook_document_reaches_only_itself() {
    let report = validate_at(
        &task_naming("req-y"),
        Placement::RunbookDocument,
        Declared::default(),
        declaring(&[specification("y")]),
    );
    assert_eq!(refs(&report), vec!["refs/unresolved"]);
}

/// "A fragment link to a unit in another document": a link resolves within its own page, so it is
/// both an unresolved reference and a broken link.
#[test]
fn a_fragment_link_resolves_only_in_its_own_document() {
    let source = document(
        "tasks",
        "here",
        &[(
            "<p data-field=\"note\">",
            "<p data-field=\"note\">Serves <a href=\"#req-x\">the requirement</a>. ",
        )],
    );
    let report = validate_at(
        &source,
        Placement::ChangeDocument,
        declaring(&[specification("x")]),
        Declared::default(),
    );
    assert_eq!(refs(&report), vec!["refs/broken-link", "refs/unresolved"]);
}

// --- document-validation's "Cross-references must resolve" -----------------------------------

/// "A reference resolved by the change's specification": the specification declares the
/// requirement while it is itself invalid, and nothing about it is reported.
#[test]
fn a_reached_specification_is_read_and_not_reported() {
    let invalid = document(
        "specification",
        "x",
        &[(
            "<section class=\"requirement\" id=\"req-x\"",
            "<section class=\"requirement\" id=\"req-without-a-scenario\" data-change=\"added\">\
             <h3>A requirement with no scenario</h3><p>The system SHALL do one more thing.</p>\
             </section><section class=\"requirement\" id=\"req-x\"",
        )],
    );
    let report = validate_at(
        &task_naming("req-x"),
        Placement::ChangeDocument,
        declaring(&[invalid]),
        Declared::default(),
    );
    assert!(report.is_accepted(), "{:?}", report.diagnostics);
    assert!(refs(&report).is_empty(), "{:?}", report.diagnostics);
}

/// "A dangling reference in a change says where it looked".
#[test]
fn a_dangling_reference_names_the_places_searched() {
    let report = validate_at(
        &task_naming("req-nowhere"),
        Placement::ChangeDocument,
        declaring(&[specification("x")]),
        declaring(&[specification("y")]),
    );
    let message = unresolved_message(&report);
    assert!(
        message.contains(
            "no unit in this document, in its change's specification, or in the accepted \
             specifications declares that identifier"
        ),
        "{message}"
    );
}

/// "A reachable document that cannot be read".
#[test]
fn an_unread_document_is_named_and_not_treated_as_empty() {
    let mut change = Some(Declared::gather(
        std::iter::empty(),
        vec![Unread {
            document: String::from("htmlspec/changes/a-change/specification.html"),
            reason: String::from("its markup needs the parser to recover"),
        }],
    ));
    let scope = Scope::gather(Placement::ChangeDocument, |place| match place {
        Place::ChangeSpecification => change.take().unwrap_or_default(),
        Place::AcceptedSpecifications => Declared::default(),
    });
    let source = task_naming("req-x");
    let outcome = parse_document(&source);
    let projection = project(&outcome.document);
    let report = engine::run(
        &outcome,
        &projection,
        &Store::Absent,
        &scope,
        rules::ALL,
        Strictness::Default,
    );
    let message = unresolved_message(&report);
    assert!(
        message.contains("htmlspec/changes/a-change/specification.html")
            && message.contains("could not be read"),
        "{message}"
    );
}

/// "A change with no specification yet": the place declares nothing, and nothing is said about it.
#[test]
fn a_change_with_no_specification_reaches_the_accepted_contract() {
    let report = validate_at(
        &task_naming("req-y"),
        Placement::ChangeDocument,
        Declared::default(),
        declaring(&[specification("y")]),
    );
    assert!(refs(&report).is_empty(), "{:?}", report.diagnostics);
}

/// document-validation's "A design references its change's specification": a kind does not decide
/// what a reference reaches.
#[test]
fn a_design_reaches_what_a_tasks_document_does() {
    let source = document(
        "design",
        "here",
        &[(
            "</main>",
            "<section class=\"decision\" id=\"decision-here\" data-change=\"added\" \
             data-status=\"proposed\" data-refs=\"req-x\"><h3>A decision \
             <span class=\"hs-id\">decision-here</span></h3><p>The design decides this.</p>\
             <p data-field=\"rationale\">Because the requirement asks it.</p></section></main>",
        )],
    );
    let report = validate_at(
        &source,
        Placement::ChangeDocument,
        declaring(&[specification("x")]),
        Declared::default(),
    );
    assert!(refs(&report).is_empty(), "{:?}", report.diagnostics);
}

// --- The property ------------------------------------------------------------------------------

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/conformance")
}

/// Every rule and location a report names, in order.
fn findings(report: &Report) -> Vec<String> {
    let mut found: Vec<String> = report
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{} {:?}", diagnostic.rule, diagnostic.location))
        .collect();
    found.sort();
    found
}

/// The one fixture whose report depends on where it sits, and the rule that makes it so.
///
/// `schema/change-markup` permits `ins` and `del` in a modified unit of a change's specification
/// and nowhere else, so this fixture's markup is refused loose and permitted at
/// [`Placement::ChangeSpecification`]. `change_markup.rs` states its verdict at each placement.
/// Named here so the exception cannot widen without an edit to this test.
const PLACED_FIXTURE: (&str, Placement, &str) = (
    "invalid-schema-change-markup-in-a-modified-unit",
    Placement::ChangeSpecification,
    "schema/change-markup",
);

/// Every committed conformance fixture reports the same rules at the same locations under every
/// placement, with places that declare nothing, as it does loose, with one named exception:
/// [`PLACED_FIXTURE`] at its placement reports what it reports loose, less exactly its
/// diagnostics under the rule that permits it there. A placement alone changes no other result,
/// so the fixtures, which are all loose, cover every placement's behavior where nothing is
/// reached.
#[test]
fn a_placement_whose_places_declare_nothing_changes_no_fixture() {
    let mut fixtures: Vec<PathBuf> = std::fs::read_dir(fixture_root())
        .expect("the fixtures are readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.join("input.html").is_file())
        .collect();
    fixtures.sort();
    assert!(
        fixtures.len() > 50,
        "only {} fixtures found",
        fixtures.len()
    );

    for fixture in fixtures {
        let source = std::fs::read_to_string(fixture.join("input.html")).expect("it reads");
        let store = Store::discover(&fixture);
        let outcome = parse_document(&source);
        let projection = project(&outcome.document);
        let at = |scope: &Scope| {
            engine::run(
                &outcome,
                &projection,
                &store,
                scope,
                rules::ALL,
                Strictness::Default,
            )
        };
        let loose = findings(&at(&Scope::loose()));
        for placement in [
            Placement::ChangeDocument,
            Placement::ChangeSpecification,
            Placement::AcceptedSpecification,
            Placement::RunbookDocument,
        ] {
            let placed = findings(&at(&Scope::gather(placement, |_| Declared::default())));
            let (name, permitted_at, rule) = PLACED_FIXTURE;
            let expected: Vec<String> = if fixture.ends_with(name) && placement == permitted_at {
                loose
                    .iter()
                    .filter(|finding| !finding.starts_with(&format!("{rule} ")))
                    .cloned()
                    .collect()
            } else {
                loose.clone()
            };
            assert_eq!(
                placed,
                expected,
                "{} reports differently at {placement:?} with nothing reached",
                fixture.display()
            );
        }
    }
}
