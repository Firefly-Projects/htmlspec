//! The template `init` writes must produce a document that validates immediately.
//!
//! This is the acceptance signal for group 6: a template that looked right and
//! failed validation would make `init`'s promise false, and a contributor's first
//! experience of the toolchain would be a document the toolchain rejects.
//!
//! The test lives in the validator's crate rather than the assets crate because it
//! needs the whole rule set, and it drives it through the public API — the same
//! entry point the CLI uses — so it tests the contract rather than internals.

use htmlspec_format::format;
use htmlspec_parse::{parse_document, project};
use htmlspec_validate::{engine, rules, Report, Strictness};

/// Validates one document through the public API.
fn validate(source: &str, strictness: Strictness) -> Report {
    let outcome = parse_document(source);
    let projection = project(&outcome.document);
    engine::run(
        &outcome,
        &projection,
        &htmlspec_assets::Store::Absent,
        &htmlspec_validate::Scope::loose(),
        rules::ALL,
        strictness,
    )
}

/// The rendered template, as `init` writes it for a document beside `assets/`.
fn rendered() -> String {
    htmlspec_assets::render_template("A new proposal", "assets")
}

fn describe(report: &Report) -> String {
    let mut out = String::new();
    if let Some(curtailment) = &report.curtailed {
        out.push_str(&format!("curtailed: {curtailment}\n"));
    }
    for diagnostic in &report.diagnostics {
        out.push_str(&format!("  {diagnostic}\n"));
    }
    out
}

/// The template's own promise: the document `init` produces passes validation with
/// nothing to fix.
#[test]
fn the_rendered_template_validates() {
    let report = validate(&rendered(), Strictness::Default);
    assert!(
        report.is_accepted(),
        "the template init writes does not validate:\n{}",
        describe(&report)
    );
}

/// And it passes under strict validation, so a project that runs continuous
/// integration with warnings fatal can adopt `init`'s output unchanged.
///
/// A `review/*` warning on a fresh document would be the likely failure: every unit
/// the template writes is marked `added`, which is a material change state. That the
/// template survives strict mode means the template's own cross-references are
/// consistent, not that the review rule is asleep.
#[test]
fn the_rendered_template_validates_under_strict_mode() {
    let report = validate(&rendered(), Strictness::Strict);
    assert!(
        report.is_accepted(),
        "the template does not survive strict validation:\n{}",
        describe(&report)
    );
}

/// The template is already canonically formatted, so `init` followed immediately by
/// `format --check` succeeds. A template that needed formatting would mean every new
/// document arrived dirty.
///
/// **Every kind, not only the default.** The fallback stylesheet arrives through a placeholder each
/// template carries, and its text is canonical only because it carries its own indentation: the
/// formatter keeps a `<style>` element's text byte for byte. A template whose placeholder sat at the
/// wrong depth would write a document `format --check` rejects, and checking one kind would not see
/// it in the other three.
#[test]
fn the_rendered_template_is_already_canonical() {
    for template in htmlspec_assets::TEMPLATES {
        let document = htmlspec_assets::render_kind(template.kind, "A new document", "assets")
            .expect("every template renders");
        let outcome = htmlspec_format::check(&document).expect("the template formats");
        assert!(
            outcome.is_canonical,
            "the {} template init writes is not canonically formatted. Canonical form:\n{}",
            template.kind, outcome.canonical
        );
    }
}

/// Every rendered template carries the fallback stylesheet exactly once, in its head.
///
/// document-template's "A document stays readable when its stylesheet cannot load" places it in
/// every document the toolchain writes. Once, because a second copy would be a second place to drift,
/// and in the head, because a `<style>` element elsewhere is not where a reader of the source looks.
#[test]
fn every_rendered_template_carries_the_fallback_once_in_its_head() {
    for template in htmlspec_assets::TEMPLATES {
        let document = htmlspec_assets::render_kind(template.kind, "A new document", "assets")
            .expect("every template renders");
        assert_eq!(
            document.matches(htmlspec_assets::fallback::ELEMENT).count(),
            1,
            "the {} template does not carry the fallback exactly once",
            template.kind
        );
        let head_end = document.find("</head>").expect("a document has a head");
        let at = document
            .find(htmlspec_assets::fallback::ELEMENT)
            .expect("counted above");
        assert!(
            at < head_end,
            "the {} template carries the fallback outside its head",
            template.kind
        );
        assert!(
            !document.contains(htmlspec_assets::FALLBACK_PLACEHOLDER),
            "the {} template leaves its fallback placeholder unsubstituted",
            template.kind
        );
    }
}

/// Formatting the template does not break it, which the two properties above imply
/// separately but not together.
#[test]
fn the_formatted_template_still_validates() {
    let formatted = format(&rendered()).expect("the template formats");
    let report = validate(&formatted, Strictness::Strict);
    assert!(
        report.is_accepted(),
        "the formatted template does not validate:\n{}",
        describe(&report)
    );
}

/// Every unit type the schema defines appears in some template `init` can write.
///
/// The templates are a contributor's first sight of the vocabulary, so an omitted
/// unit type is a unit type nobody discovers. Read from the schema rather than
/// listed, so a new unit type fails here until a template shows it.
///
/// The assertion is over the **union** of the templates rather than over any one of
/// them, and the union is what keeps the property. A specification must not carry a
/// task unit, so no single document can demonstrate all seven — the task lives in
/// the `tasks` template and the other six in the `specification` template.
///
/// Weakening this to "the specification template shows six of the seven" was the
/// available alternative and it is worse: it turns a schema-derived check into a
/// hardcoded exemption for one unit, and an eighth unit type added to the schema
/// would then pass unnoticed. The union costs a few lines and still fails on a unit
/// no template demonstrates.
#[test]
fn the_templates_demonstrate_every_unit_type_the_schema_defines() {
    let rendered: Vec<(&str, String)> = htmlspec_assets::TEMPLATES
        .iter()
        .map(|template| {
            (
                template.kind,
                htmlspec_assets::render_kind(template.kind, "A subject", "assets")
                    .expect("a declared template renders"),
            )
        })
        .collect();

    for (unit, prefix) in htmlspec_schema::facts::identifier_prefixes() {
        let demonstrated_by: Vec<&str> = rendered
            .iter()
            .filter(|(_, document)| {
                document.contains(&format!(r#"class="{unit}""#))
                    && document.contains(&format!(r#"id="{prefix}"#))
            })
            .map(|(kind, _)| *kind)
            .collect();
        assert!(
            !demonstrated_by.is_empty(),
            "no template demonstrates the {unit} unit with a {prefix} identifier, so \
             a contributor never sees it"
        );
    }
}

/// The task unit is demonstrated by the `tasks` template and by no other.
///
/// The specific placement, asserted separately from the union above, because the
/// union alone would pass if the task appeared in the specification template — which
/// is the one document the vocabulary forbids it in.
#[test]
fn the_task_unit_is_demonstrated_only_by_the_tasks_template() {
    for template in htmlspec_assets::TEMPLATES {
        let document = htmlspec_assets::render_kind(template.kind, "A subject", "assets")
            .expect("a declared template renders");
        let carries_a_task = document.contains(r#"class="task""#);
        assert_eq!(
            carries_a_task,
            template.kind == "tasks",
            "the {} template {} a task unit",
            template.kind,
            if carries_a_task {
                "carries"
            } else {
                "does not carry"
            }
        );
    }
}

/// The proposal template carries no unit scaffolding at all.
///
/// The asymmetry is the requirement rather than a stylistic choice. A proposal
/// arriving full of empty requirement blocks would push its author toward stating a
/// contract at the stage meant for arguing about one, and the emptiest possible
/// starting point is the correct one for a document whose structure is the author's
/// to choose.
#[test]
fn the_proposal_template_carries_no_unit_scaffolding() {
    let document = htmlspec_assets::render_kind("proposal", "A proposal", "assets")
        .expect("the proposal template renders");
    for (unit, prefix) in htmlspec_schema::facts::identifier_prefixes() {
        assert!(
            !document.contains(&format!(r#"class="{unit}""#)),
            "the proposal template scaffolds a {unit} unit, which pushes its author \
             toward stating a contract at the stage meant for arguing about one"
        );
        assert!(
            !document.contains(&format!(r#"id="{prefix}"#)),
            "the proposal template carries a {prefix} identifier"
        );
    }
    // And it is still a readable page rather than an empty file.
    assert!(document.contains("<h2>Why</h2>"), "{document}");
}

/// Every kind `init` can produce validates immediately, and under strict mode.
///
/// The promise held for one template before this change and has to hold for three.
/// A kind whose template does not validate would make `init`'s promise false for
/// that kind alone, which is worse than failing for all of them: a contributor
/// would meet it only when they reached that stage.
#[test]
fn every_kind_init_can_produce_validates_immediately() {
    for template in htmlspec_assets::TEMPLATES {
        let document = htmlspec_assets::render_kind(template.kind, "A subject", "assets")
            .expect("a declared template renders");
        for strictness in [Strictness::Default, Strictness::Strict] {
            let report = validate(&document, strictness);
            assert!(
                report.is_accepted(),
                "the {} template does not validate under {strictness:?}:\n{}",
                template.kind,
                describe(&report)
            );
        }
        // And each is canonically formatted, so `init` followed by `format --check`
        // succeeds for every kind rather than only the default one.
        let outcome = htmlspec_format::check(&document).expect("the template formats");
        assert!(
            outcome.is_canonical,
            "the {} template is not canonically formatted. Canonical form:\n{}",
            template.kind, outcome.canonical
        );
    }
}

/// Every template's navigation resolves without JavaScript: each entry is a plain
/// anchor to a fragment the document declares.
///
/// `refs/broken-link` would catch an unresolvable one, and this states the stronger
/// property the requirement asks for — that the navigation is authored markup rather
/// than something the enhancement layer builds.
///
/// A template with *no* navigation satisfies this trivially, and the proposal
/// template is that case: it has no unit sections to navigate between, so offering a
/// contents list of one entry would be scaffolding for its own sake. The assertion is
/// therefore about the navigation a template has rather than about having one, and
/// `the_proposal_template_carries_no_unit_scaffolding` pins the absence deliberately.
#[test]
fn every_templates_navigation_resolves_without_scripting() {
    let mut checked = 0;
    for template in htmlspec_assets::TEMPLATES {
        let document = htmlspec_assets::render_kind(template.kind, "A subject", "assets")
            .expect("a declared template renders");
        let Some(navigation) = document
            .split_once("<nav")
            .and_then(|(_, rest)| rest.split_once("</nav>"))
            .map(|(inside, _)| inside)
        else {
            continue;
        };
        checked += 1;

        let mut entries = 0;
        let mut rest = navigation;
        while let Some(at) = rest.find("href=\"#") {
            rest = &rest[at + "href=\"#".len()..];
            let Some(end) = rest.find('"') else { break };
            let target = &rest[..end];
            entries += 1;
            assert!(
                document.contains(&format!(r#"id="{target}""#)),
                "the {} template's navigation links to #{target}, which the document \
                 does not declare",
                template.kind
            );
        }
        assert!(
            entries > 0,
            "the {} template has a navigation element listing nothing",
            template.kind
        );
    }
    assert!(
        checked >= 2,
        "only {checked} template(s) have navigation; the specification and tasks \
         templates both do, so a count below two means one lost its contents list"
    );
}

/// The document declares its identity in machine-readable form *and* shows it to a
/// reader, which are two requirements and not one.
#[test]
fn the_template_declares_its_identity_in_both_forms() {
    let document = rendered();
    let version =
        htmlspec_schema::facts::contract_version().expect("the schema declares a version");

    // Machine-readable: a consumer determines the contract version before parsing
    // the content. Asserted through the projection rather than by matching markup,
    // because the canonical form fixes attribute order and a test that spelled the
    // tag out would be asserting the formatter's ordering rather than the template's
    // declaration.
    let declared = project(&parse_document(&document).document)
        .model
        .schema_version;
    assert_eq!(
        declared.as_deref(),
        Some(version.as_str()),
        "the template does not declare the contract version"
    );

    // Visible: the projected model carries the kind, and the page shows it, so a
    // proposal under discussion is not mistaken for an accepted specification.
    let projection = project(&parse_document(&document).document);
    let kind = projection
        .model
        .kind
        .expect("the document declares an artifact kind");
    assert!(
        document.contains(&format!(r#"<span class="hs-kind">{kind}</span>"#)),
        "the kind {kind} is not shown on the page"
    );
}

/// The template references only project-local assets, which is what makes it render
/// offline. The safety rules assert this too; this test names the reason so a failure
/// here reads as "the template reached outward" rather than as a generic rule
/// violation.
#[test]
fn the_template_references_only_project_local_assets() {
    let report = validate(&rendered(), Strictness::Default);
    let safety: Vec<String> = report
        .diagnostics
        .iter()
        .filter(|d| d.rule.namespace() == htmlspec_validate::Namespace::Safety)
        .map(|d| d.to_string())
        .collect();
    assert!(
        safety.is_empty(),
        "the template trips a safety rule:\n{}",
        safety.join("\n")
    );
}
