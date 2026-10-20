//! Writes the conformance fixtures.
//!
//! Run with `cargo run -p htmlspec-conformance --features generate-fixtures --bin generate_fixtures`.
//!
//! The fixtures are generated rather than hand-written for one reason: each invalid
//! fixture must differ from a *conforming* document in exactly one way, or it does not
//! isolate the rule it claims. Hand-writing thirty near-identical documents makes that
//! impossible to verify by reading them, and a fixture that trips three rules while
//! claiming one is worse than no fixture — the harness passes and the coverage is
//! fictional.
//!
//! So each fixture is a stated edit to a known-good base, and the harness then asserts
//! the claimed rule is reported. The generated files are committed, because the
//! fixtures are part of the published contract and a consumer must be able to read
//! them without running this.

use std::path::{Path, PathBuf};

/// A conformance fixture: a name, an edit to the base document, and what it pins.
struct Fixture {
    /// The directory name.
    name: &'static str,
    /// Why this fixture exists, in one line.
    note: &'static str,
    /// The rule identifiers the document violates. Empty means a valid fixture.
    rules: &'static [&'static str],
    /// Whether the document is accepted despite reporting the rules above.
    ///
    /// True only for a fixture pinning a `review/*` warning: such a report is made and
    /// never gates work, so acceptance and the rule list are independent.
    accepted: bool,
    /// The edit that produces the document, applied to the base.
    edit: fn(String) -> String,
    /// The asset store this fixture carries, written into its own directory.
    ///
    /// Empty for all but the `asset/*` fixtures, which is what keeps those rules out of
    /// scope for every other fixture in the suite.
    assets: &'static [StoredAsset],
}

/// The conforming base every fixture is an edit to.
///
/// A document the toolchain produces, with its asset references in the **unaddressed**
/// form. That form is deliberate and load-bearing, so it is spelled out here rather
/// than taken from `render_template`.
///
/// `render_template` now emits content addresses, because a rendered document must name
/// the bytes it renders with. A fixture must not: no `assets/` directory sits beside
/// one, so an addressed reference there would name bytes nothing stores — and while
/// that is inert today (an absent store resolves nothing), it would make the 37
/// existing fixtures depend on the store staying absent rather than on their own
/// content. It would also rewrite all 37 committed inputs, whose exact bytes are what
/// each fixture pins.
///
/// So this base is frozen, and `the_committed_fixtures_match_the_generator` in the
/// harness asserts the 37 are byte-identical to what it produces. The fixtures that
/// *do* need addressed references are the `asset/*` ones, and they carry a real store —
/// see [`store_for_fixture`].
fn base() -> String {
    // The `specification` shape, which is the kind the 39 fixtures below are: they
    // carry requirements, decisions, evidence, risks, and verifications, and none of
    // them is about a task. The two that need a task declare their own kind through
    // their own edit — see `invalid-schema-missing-reason` and
    // `invalid-a11y-script-supplied-content`.
    let rendered = htmlspec_assets::render_kind("specification", "A conformance fixture", "assets")
        .expect("the specification template exists");
    // Rewrite each address back to the bare file name, which is the form the committed
    // fixtures hold. Derived from the asset table rather than hardcoded, so it cannot go
    // stale when the bytes change.
    let mut out = rendered;
    for asset in htmlspec_assets::all_assets() {
        out = out.replace(
            &format!("assets/{}", asset.path),
            &format!("assets/{}", legacy_path(asset.name())),
        );
    }
    // The declared contract version is frozen for the same reason, and for a second one
    // the addresses do not have. The template declares the newest version, and the
    // committed fixtures declare `0.1.0`. A minor bump accepts every document declaring
    // the version before it, so keeping the base at `0.1.0` makes the whole suite a
    // standing check of that promise, rather than rewriting every committed input the
    // day a version is added. A fixture about a newer field declares the newer version
    // through its own edit: see [`declaring_0_7_0`] and [`declaring_the_current_version`].
    let out = as_version(out, &current_version(), FROZEN_VERSION);
    // The template's example risk is frozen too, for both reasons. Since `0.7.0` the
    // template declares the risk's `data-response` and teaches the accepted alternative in
    // its mitigation paragraph, and the committed fixtures hold the risk as it was. Taking
    // the template's would rewrite a paragraph that pins no rule in every committed input
    // that carries the risk, and would put a `0.7.0` attribute in a base declaring `0.1.0`.
    // A fixture about the response makes its own edit: see [`responding`].
    //
    // The fallback stylesheet is frozen out for the same reason. The templates carry it since
    // `0.31.0`, and the committed fixtures were written before it and pin no rule about it.
    // Taking it would add nine lines to every committed input. `format --fallback` and the
    // template tests hold the fallback itself.
    without_the_fallback(with_the_frozen_risk(out))
}

/// The rendered template without the fallback stylesheet's line in its head.
fn without_the_fallback(rendered: String) -> String {
    let line = format!("    {}\n", htmlspec_assets::fallback::ELEMENT);
    assert!(
        rendered.contains(&line),
        "the specification template carries the fallback stylesheet in its head"
    );
    rendered.replacen(&line, "", 1)
}

/// The example risk as every committed fixture holds it, whatever the template now writes.
const FROZEN_RISK: &str = "<section class=\"risk\" id=\"risk-example\" data-change=\"added\">\n\
     \x20           <h3>The example risk <span class=\"hs-id\">risk-example</span></h3>\n\
     \x20           <p>State what could go wrong, in one unmarked paragraph.</p>\n\
     \x20           <p data-field=\"mitigation\">State what holds it down. A risk with no \
     mitigation is a worry.</p>\n\
     \x20         </section>";

/// Replaces the rendered template's example risk, from its opening tag to its closing one,
/// with [`FROZEN_RISK`].
///
/// The whole element is replaced rather than the two edits `0.7.0` made, so that a later
/// edit to the template's risk is frozen out as well, instead of passing silently into
/// every committed input.
fn with_the_frozen_risk(rendered: String) -> String {
    const OPENING: &str = "<section class=\"risk\" id=\"risk-example\"";
    const CLOSING: &str = "</section>";
    let start = rendered
        .find(OPENING)
        .expect("the specification template has an example risk");
    let end = rendered[start..]
        .find(CLOSING)
        .expect("the example risk closes")
        + start
        + CLOSING.len();
    let mut out = rendered;
    out.replace_range(start..end, FROZEN_RISK);
    out
}

/// The contract version every committed fixture declares, unless its edit says otherwise.
const FROZEN_VERSION: &str = "0.1.0";

/// The contract version a new document declares: the one the template writes.
///
/// Read from the rendered template rather than named here, because this binary depends
/// on the assets crate alone and a second statement of the version would go stale the
/// first time it moved.
fn current_version() -> String {
    let rendered = htmlspec_assets::render_kind("specification", "A conformance fixture", "assets")
        .expect("the specification template exists");
    // Anchored on the version's own `name`, and read backwards to its `content`: the
    // template's first meta element is the viewport's, whose `content` comes first too.
    let end = rendered
        .find(r#"" name="htmlspec-version">"#)
        .expect("the template declares its contract version");
    let start = rendered[..end]
        .rfind(r#"content=""#)
        .expect("the version meta element has a content attribute")
        + r#"content=""#.len();
    rendered[start..end].to_owned()
}

/// Redeclares the contract version `from` as `to`, in the metadata and on the page.
///
/// Both places, for the reason [`as_kind`] gives: the masthead shows the version to a
/// reader, and a page contradicting its own metadata is a fixture about that instead.
fn as_version(source: String, from: &str, to: &str) -> String {
    let redeclared = replace(
        source,
        &format!(r#"<meta content="{from}" name="htmlspec-version">"#),
        &format!(r#"<meta content="{to}" name="htmlspec-version">"#),
    );
    replace(
        redeclared,
        &format!("<dd>{from}</dd>"),
        &format!("<dd>{to}</dd>"),
    )
}

/// Redeclares the frozen base at the newest contract version.
///
/// For the one fixture that pins a document declaring the version a new document declares,
/// `valid-declaring-the-current-contract-version`. It moves with the template, so it is the
/// fixture a contract bump regenerates.
fn declaring_the_current_version(source: String) -> String {
    as_version(source, FROZEN_VERSION, &current_version())
}

/// The version the fixtures about `0.2.0` to `0.7.0`'s fields declare: the version they were
/// written at.
///
/// Named here rather than read from the template, unlike [`current_version`]. Those fixtures
/// declared the template's version until `1.0.0`, and `1.0.0` is the `0.7.0` vocabulary
/// unchanged, so following the template would rewrite 20 committed inputs to a version no
/// field of theirs needs. Pinning them keeps them byte for byte and makes each a standing check
/// that a document declaring `0.7.0` keeps its verdict, which is what the contract's versioning
/// promises.
const VOCABULARY_0_7_0: &str = "0.7.0";

/// Redeclares the frozen base at [`VOCABULARY_0_7_0`].
///
/// For a fixture about a field one of `0.2.0` to `0.7.0` added. Its valid twin makes the same
/// edit, so the pair still differs in one way.
fn declaring_0_7_0(source: String) -> String {
    as_version(source, FROZEN_VERSION, VOCABULARY_0_7_0)
}

/// The base's verification recorded as `partial`, at `0.7.0`.
///
/// `unchecked` is written immediately after the verification's method paragraph, and is
/// empty for the fixture that states nothing the check did not reach. The two twins call
/// this with and without it, so they differ in that paragraph alone.
fn partially_verified(source: String, unchecked: &str) -> String {
    const METHOD: &str = "<p data-field=\"method\">State how the requirement was checked, \
                          precisely enough that someone else could repeat it.</p>";
    let partial = replace(
        declaring_0_7_0(source),
        r#"data-outcome="not-run" data-refs="req-example""#,
        r#"data-outcome="partial" data-refs="req-example""#,
    );
    replace(partial, METHOD, &format!("{METHOD}{unchecked}"))
}

/// The base's requirement as a change renaming it would state it.
///
/// Redeclared at `0.7.0`, restated in `change` state under `heading`, and
/// carrying one `renamed-from` paragraph per entry of `paragraphs`, immediately after the
/// heading. The heading keeps the identifier span the template writes, because the rule
/// about a rename to the same name has to be pinned against the heading as a document
/// written from the template carries it.
fn renamed(source: String, change: &str, heading: &str, paragraphs: &[&str]) -> String {
    let current = declaring_0_7_0(source);
    let restated = replace(
        current,
        r#"<section class="requirement" id="req-example" data-change="added">"#,
        &format!(r#"<section class="requirement" id="req-example" data-change="{change}">"#),
    );
    let renames: String = paragraphs
        .iter()
        .map(|text| format!("\n            <p data-field=\"renamed-from\">{text}</p>"))
        .collect();
    replace(
        restated,
        r#"<h3>The example requirement <span class="hs-id">req-example</span></h3>"#,
        &format!(r#"<h3>{heading} <span class="hs-id">req-example</span></h3>{renames}"#),
    )
}

/// The base as a tasks document at `0.7.0`, holding one task in `state`
/// that states a deviation.
///
/// The task's words are kept as written and the deviation sits beside them, which is the shape
/// `apply` teaches. Only `state` differs between the two fixtures that use this.
fn a_task_stating_a_deviation(source: String, state: &str) -> String {
    let tasks = as_kind(declaring_0_7_0(source), "tasks");
    append_to_main(
        tasks,
        &format!(
            "        <section id=\"hs-tasks\">\n\
             \x20         <h2>Tasks</h2>\n\
             \x20         <section class=\"task\" id=\"task-hand-written-list\" \
             data-change=\"added\" data-state=\"{state}\">\n\
             \x20           <h3>Add the subcommand to the hand-written list</h3>\n\
             \x20           <p data-field=\"deviation\">There is no hand-written list. The check \
             resolves each invocation against the command tree, so the subcommand was covered \
             the moment it existed.</p>\n\
             \x20         </section>\n\
             \x20       </section>"
        ),
    )
}

/// The base's risk, as its element opens.
const BASE_RISK_OPENING: &str = r#"<section class="risk" id="risk-example" data-change="added">"#;

/// The base's risk's mitigation paragraph, on its own line.
const BASE_RISK_MITIGATION: &str =
    "\n            <p data-field=\"mitigation\">State what holds it \
                                    down. A risk with no mitigation is a worry.</p>";

/// The base's risk with its mitigation paragraph removed, and nothing else changed.
fn without_the_risks_mitigation(source: String) -> String {
    replace(source, BASE_RISK_MITIGATION, "")
}

/// The base's risk declaring `response` in its `data-response` attribute.
fn responding(source: String, response: &str) -> String {
    let opening = BASE_RISK_OPENING.trim_end_matches('>');
    replace(
        source,
        BASE_RISK_OPENING,
        &format!(r#"{opening} data-response="{response}">"#),
    )
}

/// The pre-addressing path a committed fixture references for `name`.
///
/// The vendored libraries sat under `assets/vendor/`, and this project's own files sat
/// flat in `assets/`. That layout is gone from the project, and it survives here only
/// as the frozen shape of the 37 fixtures.
fn legacy_path(name: &str) -> String {
    let own = ["htmlspec.css", "htmlspec-print.css", "htmlspec.js"];
    if own.contains(&name) {
        name.to_owned()
    } else {
        format!("vendor/{name}")
    }
}

/// A file a fixture stores in its own `assets/` directory.
///
/// A fixture carrying one of these is a fixture *inside a project with an asset store*,
/// which is the only way the `asset/*` rules come into scope. Nothing about the document
/// puts them in scope — that is the point, and the reason the rule cannot be switched
/// off from inside a document.
struct StoredAsset {
    /// The file name, which the address wraps.
    name: &'static str,
    /// The bytes to write.
    bytes: &'static [u8],
    /// The bytes whose address the file is written *at*.
    ///
    /// Equal to `bytes` for an honest store. Different for the mismatch fixture, where
    /// something replaced a stored file in place — the address describes one thing and
    /// the contents are another.
    addressed_as: &'static [u8],
}

/// Replaces `from` with `to`, and panics if `from` is absent.
///
/// The panic is the point. A silent no-op would produce a fixture identical to the
/// base — valid, claiming a rule, and failing the harness with a confusing message
/// rather than here with a clear one.
fn replace(source: String, from: &str, to: &str) -> String {
    assert!(
        source.contains(from),
        "the base document does not contain {from:?}, so this fixture's edit is stale"
    );
    source.replace(from, to)
}

/// Inserts `markup` just before the closing `</main>`.
fn append_to_main(source: String, markup: &str) -> String {
    replace(source, "      </main>", &format!("{markup}\n      </main>"))
}

/// Inserts `markup` into the document head.
fn append_to_head(source: String, markup: &str) -> String {
    replace(source, "  </head>", &format!("{markup}\n  </head>"))
}

/// Redeclares the document's artifact kind, in the metadata and on the page.
///
/// Both places, because the base shows its kind to a reader as well as declaring it
/// to the projection. A fixture that changed only the meta element would render a
/// page contradicting its own model, and the presentation checks read the page.
fn as_kind(source: String, kind: &str) -> String {
    let base_kind = "specification";
    let redeclared = replace(
        source,
        &format!(r#"<meta content="{base_kind}" name="htmlspec-kind">"#),
        &format!(r#"<meta content="{kind}" name="htmlspec-kind">"#),
    );
    replace(
        redeclared,
        &format!(r#"<span class="hs-kind">{base_kind}</span>"#),
        &format!(r#"<span class="hs-kind">{kind}</span>"#),
    )
}

/// Which of the two run-record fixtures to produce.
///
/// A sum type rather than a `bool`, because the two differ in what they demonstrate rather than in a
/// quantity: one shows the record's shape is accepted and the other shows that a record which has
/// recorded nothing is *also* accepted. A `bool` at the call site would name neither.
#[derive(Clone, Copy)]
enum RunRecord {
    /// Every copied requirement has a verification recording an outcome.
    Conforming,
    /// The verification is gone, so the record attests to nothing.
    Short,
}

/// A run's record, edited out of the base.
///
/// Four edits, and each one is a measured constraint rather than a stylistic choice:
///
/// - **The kind becomes `tasks`.** Declared `specification`, the copied step draws
///   `kind/task-in-specification` — an error, once per step.
/// - **A task unit is added**, which is the copied procedure. A `tasks` document with no task unit
///   draws `kind/no-tasks`, and a record of a run that worked no step is not a record.
/// - **Every unit becomes `unchanged`.** The base declares `added`, which is right for a change's
///   specification and wrong here: the obligation a run attests to existed before the run started,
///   and `added` draws `review/stale-verification` on the verification naming it.
/// - **The verification's outcome becomes `passed`.** The base writes `not-run`, which records no
///   outcome — so a fixture claiming to be a conforming record would be one that recorded nothing.
///
/// The requirement is left where the base puts it. A record carries its requirements *copied*, and
/// the base's `req-example` is already a requirement with a scenario and a strong modal, which is
/// exactly what a copy has to be.
fn a_run_record(base: String, which: RunRecord) -> String {
    let record = as_kind(base, "tasks");
    // A change state on every unit, so a `replace_all` reaches the requirement, the scenario, the
    // decision, the evidence, the risk, and the verification together. The previous change shipped
    // this half-done — the specification fixed and the tasks document left carrying `added` — and a
    // review caught it, so it is done in one edit here rather than per unit.
    let record = record.replace("data-change=\"added\"", "data-change=\"unchanged\"");
    let record = append_to_main(
        record,
        "        <section id=\"hs-tasks\">\n\
         \x20         <h2>Tasks</h2>\n\
         \x20         <p>The runbook's procedure, copied as it stood when this run started. \
         Each step's state is this run's to advance.</p>\n\
         \x20         <section class=\"task\" id=\"task-copied-step\" data-change=\"unchanged\" \
         data-state=\"done\">\n\
         \x20           <h3>A step this run worked <span class=\"hs-id\">task-copied-step</span>\
         </h3>\n\
         \x20           <p data-field=\"note\">Copied from the runbook, with the state this run \
         advanced it to.</p>\n\
         \x20         </section>\n\
         \x20       </section>",
    );
    match which {
        RunRecord::Conforming => replace(
            record,
            "data-outcome=\"not-run\"",
            "data-outcome=\"passed\"",
        ),
        RunRecord::Short => {
            // The verification section removed whole, rather than the unit alone. Removing the unit
            // and leaving its wrapper would leave an empty document section, which says the record
            // has a place for outcomes and nothing in it — a different fact from a record with no
            // verification at all, and the harness would be pinning the wrong one.
            let opening = record
                .find("        <section id=\"hs-verification\">")
                .expect("the template has a verification section");
            // **Anchored on the newline, and the unanchored version is a measured defect.**
            // Searching for the eight-space close alone matches *inside* the ten-space close of the
            // verification unit nested within — the ten spaces contain the eight — so the slice
            // ended two characters early and left `>` behind. The result needed error recovery,
            // which curtailed every rule and made the fixture pin a parse failure instead of the
            // silence it exists to pin.
            const END: &str = "\n        </section>\n";
            let closing = record[opening..].find(END).expect("it closes") + END.len();
            let mut out = record.clone();
            out.replace_range(opening..opening + closing, "");
            // And its entry in the contents list, which would otherwise be a fragment link to a
            // section that is gone — `refs/broken-link`, an error, and not what this fixture pins.
            replace(
                out,
                "          <li><a href=\"#hs-verification\">Verification</a></li>\n",
                "",
            )
        }
    }
}

const FIXTURES: &[Fixture] = &[
    // -------------------------------------------------------------------------
    // Valid fixtures
    // -------------------------------------------------------------------------
    Fixture {
        name: "valid-minimal",
        note: "the document init writes, unmodified: every unit type, all conforming",
        rules: &[],
        accepted: false,
        edit: |base| base,
        assets: &[],
    },
    Fixture {
        name: "valid-declaring-the-current-contract-version",
        note: "the base declaring the version a new document declares, so the newest version \
               the schema accepts is pinned by a document and not only by the template",
        rules: &[],
        accepted: false,
        edit: declaring_the_current_version,
        assets: &[],
    },
    Fixture {
        name: "valid-empty-code-example",
        note: "a code element with no children is inert, so safety/live-example must \
               not fire on it",
        rules: &[],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section id=\"hs-empty\">\n\
                 \x20         <h2>An empty example</h2>\n\
                 \x20         <pre><code></code></pre>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-comment-split-example",
        note: "a parser splits a code element's text around a comment; no child is an \
               element, so the example is still inert",
        rules: &[],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section id=\"hs-split\">\n\
                 \x20         <h2>Text split by a comment</h2>\n\
                 \x20         <pre><code>before<!-- a note -->after</code></pre>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-local-asset-reference",
        note: "a relative reference into the project's shared assets is accepted, which \
               is what lets a document render offline",
        rules: &[],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                "    <link href=\"assets/htmlspec.css\" rel=\"stylesheet\">",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-chart-figure",
        note: "a Vega-Lite specification that its own schema accepts, with local data \
               and a visible source",
        rules: &[],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section id=\"hs-chart\">\n\
                 \x20         <h2>A chart</h2>\n\
                 \x20         <figure id=\"figure-chart\">\n\
                 \x20           <figcaption>A bar chart with local data.</figcaption>\n\
                 \x20           <details>\n\
                 \x20             <summary>Chart source</summary>\n\
                 \x20             <pre class=\"vega-lite\">{\n\
                 \x20 \"$schema\": \"https://vega.github.io/schema/vega-lite/v6.json\",\n\
                 \x20 \"data\": {\"values\": [{\"a\": \"x\", \"b\": 1}]},\n\
                 \x20 \"mark\": \"bar\",\n\
                 \x20 \"encoding\": {\"x\": {\"field\": \"a\", \"type\": \"nominal\"}, \
                 \"y\": {\"field\": \"b\", \"type\": \"quantitative\"}}\n\
                 }</pre>\n\
                 \x20           </details>\n\
                 \x20         </figure>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // schema/*
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-schema-wrong-version",
        note: "a document declaring a contract version the schema does not pin; the \
               `const` that type generation drops is enforced at runtime",
        rules: &["schema/invalid"],
        accepted: false,
        edit: |base| replace(base, r#"content="0.1.0""#, r#"content="9.9.9""#),
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-unknown-kind",
        note: "an artifact kind outside the schema's closed enumeration; the set of \
               kinds is closed, so a fifth carries no stated rules",
        rules: &["schema/invalid"],
        accepted: false,
        edit: |base| as_kind(base, "rumination"),
        assets: &[],
    },
    // The other half of section 9.3: an absent kind. Required and undefaulted, so an
    // unmigrated document fails by naming the missing declaration rather than
    // silently becoming whichever kind is most convenient.
    Fixture {
        name: "invalid-schema-absent-kind",
        note: "a document declaring no artifact kind; nothing then determines which \
               units belong in it, and no kind rule applies",
        rules: &["schema/invalid"],
        accepted: false,
        edit: |base| {
            let stripped = replace(
                base,
                "    <meta content=\"specification\" name=\"htmlspec-kind\">\n",
                "",
            );
            replace(
                stripped,
                "          <dt>Kind</dt>\n          <dd><span class=\"hs-kind\">specification\
                 </span></dd>\n",
                "",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-requirement-without-scenario",
        note: "a requirement with no scenario; `minItems: 1` vanishes in type \
               generation and the runtime validator is what enforces it",
        rules: &["schema/invalid"],
        accepted: false,
        edit: |base| {
            let opening = base
                .find("            <article class=\"scenario\"")
                .expect("the template has a scenario");
            let closing = base.find("            </article>").expect("it closes")
                + "            </article>\n".len();
            let mut out = base.clone();
            out.replace_range(opening..closing, "");
            out
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-projection-two-reserved-classes",
        note: "an element carrying two reserved class names; the projection cannot \
               choose one and reports the specification section it violates",
        rules: &["schema/projection"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section class=\"risk decision\" id=\"risk-both\">\n\
                 \x20         <h2>Two reserved classes</h2>\n\
                 \x20         <p>Ambiguous.</p>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-duplicate-id",
        note: "two units sharing one identifier, which JSON Schema cannot express and \
               Rust owns",
        rules: &["schema/duplicate-id"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section class=\"risk\" id=\"req-example\">\n\
                 \x20         <h2>A duplicate identifier</h2>\n\
                 \x20         <p>The identifier is already taken.</p>\n\
                 \x20         <p data-field=\"mitigation\">Rename it.</p>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-prefix-mismatch",
        note: "a risk carrying a requirement's identifier prefix, so the identifier \
               lies about the unit's type",
        rules: &["schema/prefix-mismatch", "schema/invalid"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section class=\"risk\" id=\"req-mislabelled\">\n\
                 \x20         <h2>A mislabelled risk</h2>\n\
                 \x20         <p>The prefix says requirement.</p>\n\
                 \x20         <p data-field=\"mitigation\">Use the risk prefix.</p>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-missing-successor",
        note: "a superseded decision that names no replacement, leaving a reader with \
               no way to find what replaced it",
        rules: &["schema/missing-successor"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                r#"data-change="added" data-status="proposed""#,
                r#"data-change="added" data-status="superseded""#,
            )
        },
        assets: &[],
    },
    // Task 4.2: the one rule a task alone can violate. `schema/missing-reason` reads
    // `task.state == dropped`, and no other unit has a state — so this fixture adds
    // back the task the base lost and declares the kind that permits it. A variant
    // within one base rather than a second base.
    Fixture {
        name: "invalid-schema-missing-reason",
        note: "a dropped task with no reason; a dropped task without one records that \
               something was abandoned and not why",
        rules: &["schema/missing-reason"],
        accepted: false,
        edit: |base| {
            let tasks = as_kind(base, "tasks");
            append_to_main(
                tasks,
                "        <section id=\"hs-tasks\">\n\
                 \x20         <h2>Tasks</h2>\n\
                 \x20         <section class=\"task\" id=\"task-example\" \
                 data-change=\"added\" data-state=\"dropped\">\n\
                 \x20           <h3>The example task</h3>\n\
                 \x20         </section>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    // `schema/deviation-not-done`: a deviation states how finished work departed from a task's
    // words, so only a `done` task may state one. The pair differs in the task's state alone, and
    // both declare `0.7.0`, because `deviation` is a field a 0.x version
    // added. A tasks document, for the reason `invalid-schema-missing-reason` is one.
    Fixture {
        name: "invalid-schema-deviation-not-done",
        note: "a task still to do that states a deviation; nothing is finished yet that could \
               differ from the task's words",
        rules: &["schema/deviation-not-done"],
        accepted: false,
        edit: |base| a_task_stating_a_deviation(base, "todo"),
        assets: &[],
    },
    Fixture {
        name: "valid-task-done-with-a-deviation",
        note: "a done task stating what was done where that departs from its words; the task is \
               done because the work happened, and its words stay as written",
        rules: &[],
        accepted: false,
        edit: |base| a_task_stating_a_deviation(base, "done"),
        assets: &[],
    },
    // `schema/missing-unchecked` reads `verification.outcome == partial` and an empty
    // `unchecked`. The twins declare `0.7.0`, because `partial` and
    // `unchecked` are what a 0.x version added, and they differ only in the `unchecked`
    // paragraph. The base's verification is the one they edit, so no unit is added.
    Fixture {
        name: "invalid-schema-missing-unchecked",
        note: "a partial verification that does not say what its check did not reach; the \
               record claims part of a requirement and leaves the rest unnamed",
        rules: &["schema/missing-unchecked"],
        accepted: false,
        edit: |base| partially_verified(base, ""),
        assets: &[],
    },
    Fixture {
        name: "valid-partial-verification-states-what-it-did-not-check",
        note: "a partial verification naming what its check did not reach in a \
               data-field=\"unchecked\" paragraph",
        rules: &[],
        accepted: false,
        edit: |base| {
            partially_verified(
                base,
                "\n            <p data-field=\"unchecked\">The observable outcome past its \
                 first clause, which the check did not reach.</p>",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // schema/missing-mitigation, and a risk's response
    //
    // Four fixtures, each an edit to the base's one risk and each declaring `0.7.0`, because
    // `data-response` is an attribute that version added. They
    // pair off: the two without a mitigation differ only in whether the risk is accepted,
    // and the two with one differ only in the value of `data-response`.
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-schema-missing-mitigation",
        note: "a risk that declares no response and states no mitigation; a risk with no \
               data-response is mitigated, and a mitigated risk states what holds it down",
        rules: &["schema/missing-mitigation"],
        accepted: false,
        edit: |base| without_the_risks_mitigation(declaring_0_7_0(base)),
        assets: &[],
    },
    Fixture {
        name: "valid-risk-accepted-without-mitigation",
        note: "a risk the change bears as it stands, declared data-response=\"accepted\", with \
               no mitigation; an accepted risk needs none",
        rules: &[],
        accepted: false,
        edit: |base| {
            responding(
                without_the_risks_mitigation(declaring_0_7_0(base)),
                "accepted",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-risk-accepted-with-mitigation",
        note: "an accepted risk that keeps its mitigation paragraph, which then states the part \
               of the risk something does hold down",
        rules: &[],
        accepted: false,
        edit: |base| responding(declaring_0_7_0(base), "accepted"),
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-unknown-response",
        note: "a risk declaring data-response=\"partial\"; a response is mitigated or accepted, \
               and how far a mitigation reaches is stated in its paragraph",
        rules: &["schema/invalid"],
        accepted: false,
        edit: |base| responding(declaring_0_7_0(base), "partial"),
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-weak-modal",
        note: "a requirement stating an obligation with no normative modal, so what is \
               required cannot be determined",
        rules: &["schema/weak-modal"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                "The system SHALL state one obligation per requirement, in a sentence a \
                 reader can verify.",
                "It would probably be nice if the system did something along these lines.",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // schema/rename-*
    //
    // Six rules, each with an invalid fixture and a valid twin that differs from it
    // in the one way the rule turns on. Every one of the twelve declares `0.7.0`, the
    // version they were written at, because `renamed-from` is a field a 0.x version added.
    // -------------------------------------------------------------------------
    Fixture {
        name: "valid-rename-noted-in-another-unit",
        note: "a decision whose note mentions the heading it used to have; prose about a \
               previous name is not a rename, and only a renamed-from paragraph is one",
        rules: &[],
        accepted: false,
        edit: |base| {
            replace(
                declaring_0_7_0(base),
                r#"<p data-field="rationale">"#,
                "<p data-field=\"note\">This decision was titled An earlier decision.</p>\n            \
                 <p data-field=\"rationale\">",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-rename-outside-requirement",
        note: "a decision carrying a renamed-from paragraph; only a requirement can state \
               the heading it replaced",
        rules: &["schema/rename-outside-requirement"],
        accepted: false,
        edit: |base| {
            replace(
                declaring_0_7_0(base),
                r#"<p data-field="rationale">"#,
                "<p data-field=\"renamed-from\">An earlier decision</p>\n            \
                 <p data-field=\"rationale\">",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-rename-of-a-modified-requirement",
        note: "a requirement renamed and rewritten: data-change=\"modified\" with a \
               renamed-from paragraph, which is OpenSpec's RENAMED plus MODIFIED",
        rules: &[],
        accepted: false,
        edit: |base| {
            renamed(
                base,
                "modified",
                "The renamed requirement",
                &["The example requirement"],
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-rename-on-added",
        note: "a renamed-from paragraph on a requirement this change introduces, which had \
               no previous heading",
        rules: &["schema/rename-on-added"],
        accepted: false,
        edit: |base| {
            renamed(
                base,
                "added",
                "The renamed requirement",
                &["The example requirement"],
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-rename-of-an-unchanged-requirement",
        note: "a requirement renamed and otherwise untouched: data-change=\"unchanged\" with \
               a renamed-from paragraph, which is OpenSpec's RENAMED alone",
        rules: &[],
        accepted: false,
        edit: |base| {
            renamed(
                base,
                "unchanged",
                "The renamed requirement",
                &["The example requirement"],
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-rename-on-removed",
        note: "a renamed-from paragraph on a requirement this change withdraws; OpenSpec \
               rejects a rename and a removal of one requirement too. The references to it \
               are removed so the one fault is the rename",
        rules: &["schema/rename-on-removed"],
        accepted: false,
        edit: |base| {
            let withdrawn = renamed(
                base,
                "removed",
                "The renamed requirement",
                &["The example requirement"],
            );
            // A reference to a removed unit is `refs/removed-target`, a second fault.
            // The evidence and the verification name the requirement, so they stop.
            let unreferenced = replace(
                withdrawn,
                r#" data-change="added" data-refs="req-example""#,
                r#" data-change="added""#,
            );
            replace(
                unreferenced,
                r#" data-outcome="not-run" data-refs="req-example""#,
                r#" data-outcome="not-run" data-refs="decision-example""#,
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-rename-stated-once-with-its-history-noted",
        note: "one renamed-from paragraph naming the heading the accepted specification \
               holds, and a note recording an older one; the history is prose, not a second \
               rename",
        rules: &[],
        accepted: false,
        edit: |base| {
            replace(
                renamed(
                    base,
                    "modified",
                    "The renamed requirement",
                    &["The example requirement"],
                ),
                r#"<p data-field="renamed-from">The example requirement</p>"#,
                "<p data-field=\"renamed-from\">The example requirement</p>\n            \
                 <p data-field=\"note\">Before that it was The first example requirement.</p>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-rename-repeated",
        note: "two renamed-from paragraphs on one requirement; one rename replaces one \
               heading",
        rules: &["schema/rename-repeated"],
        accepted: false,
        edit: |base| {
            renamed(
                base,
                "modified",
                "The renamed requirement",
                &["The example requirement", "The first example requirement"],
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-rename-stated-through-inline-markup",
        note: "a renamed-from paragraph whose text is inside inline markup; the text is \
               what projects, and it is not empty",
        rules: &[],
        accepted: false,
        edit: |base| {
            renamed(
                base,
                "modified",
                "The renamed requirement",
                &["<em>The example requirement</em>"],
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-rename-empty",
        note: "a renamed-from paragraph holding only whitespace, so it says the \
               requirement was renamed without saying from what",
        rules: &["schema/rename-empty"],
        accepted: false,
        edit: |base| renamed(base, "modified", "The renamed requirement", &[" "]),
        assets: &[],
    },
    Fixture {
        name: "valid-rename-changing-only-case",
        note: "a rename that changes only the heading's case; OpenSpec allows it, and the \
               comparison is case-sensitive",
        rules: &[],
        accepted: false,
        edit: |base| {
            renamed(
                base,
                "modified",
                "The Example Requirement",
                &["The example requirement"],
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-rename-to-same-name",
        note: "a renamed-from paragraph stating the requirement's own heading; the heading \
               shows its identifier, and the identifier is not part of the name",
        rules: &["schema/rename-to-same-name"],
        accepted: false,
        edit: |base| {
            renamed(
                base,
                "modified",
                "The example requirement",
                &["The example requirement"],
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // A modified requirement
    //
    // artifact-kinds' "A change's specification states a modified requirement whole": every
    // scenario the requirement holds, each in its own change state, and a withdrawn one stated
    // as `removed` rather than left out. The contract already expresses this, so the fixture
    // declares the base's frozen version and pins that it stays valid.
    // -------------------------------------------------------------------------
    Fixture {
        name: "valid-modified-requirement-stated-whole",
        note: "a modified requirement stated whole, with a withdrawn scenario stated as removed",
        rules: &[],
        accepted: false,
        edit: |base| {
            let modified = replace(
                base,
                r#"<section class="requirement" id="req-example" data-change="added">"#,
                r#"<section class="requirement" id="req-example" data-change="modified">"#,
            );
            let carried = replace(
                modified,
                r#"<article class="scenario" id="scenario-example">"#,
                r#"<article class="scenario" id="scenario-example" data-change="unchanged">"#,
            );
            replace(
                carried,
                "            </article>\n          </section>",
                "            </article>\n\
                 \x20           <article class=\"scenario\" id=\"scenario-altered\" \
                 data-change=\"modified\">\n\
                 \x20             <h4>The obligation holds with new wording</h4>\n\
                 \x20             <dl>\n\
                 \x20               <dt>When</dt>\n\
                 \x20               <dd>The thing that happens, as this change words it.</dd>\n\
                 \x20               <dt>Then</dt>\n\
                 \x20               <dd>The observable outcome, as this change words it.</dd>\n\
                 \x20             </dl>\n\
                 \x20           </article>\n\
                 \x20           <article class=\"scenario\" id=\"scenario-introduced\" \
                 data-change=\"added\">\n\
                 \x20             <h4>A case this change introduces</h4>\n\
                 \x20             <dl>\n\
                 \x20               <dt>When</dt>\n\
                 \x20               <dd>A thing no earlier change described happens.</dd>\n\
                 \x20               <dt>Then</dt>\n\
                 \x20               <dd>The outcome this change establishes.</dd>\n\
                 \x20             </dl>\n\
                 \x20           </article>\n\
                 \x20           <article class=\"scenario\" id=\"scenario-withdrawn\" \
                 data-change=\"removed\">\n\
                 \x20             <h4>A case this change withdraws</h4>\n\
                 \x20             <dl>\n\
                 \x20               <dt>When</dt>\n\
                 \x20               <dd>A thing the project no longer describes happens.</dd>\n\
                 \x20               <dt>Then</dt>\n\
                 \x20               <dd>The outcome the project no longer requires.</dd>\n\
                 \x20             </dl>\n\
                 \x20           </article>\n\
                 \x20         </section>",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // schema/change-markup
    //
    // Three fixtures carrying the same `del` and `ins`, which differ only in where the markup
    // sits: in the statement of the base's requirement, marked `added`; in the same statement
    // with the requirement marked `modified`; or in a paragraph between the section heading
    // and the requirement, outside every unit. All three declare the base's frozen `0.1.0`, so
    // the invalid ones are also a standing check that an earlier declared version exempts no
    // document.
    //
    // The markup is permitted only in a modified unit of a change's specification, and a
    // fixture sits in no change, so no fixture can pin the permitted case.
    // `conformance/tests/change_markup.rs` validates the modified one's bytes at every
    // placement instead.
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-schema-change-markup",
        note: "a deleted phrase and an inserted one in the statement of a requirement marked \
               added, which has no accepted text for the marks to edit; the markup is \
               permitted only in a modified unit of a change's specification",
        rules: &["schema/change-markup"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                "The system SHALL state one obligation per requirement",
                &format!("The system SHALL state {CHANGE_MARKUP} per requirement"),
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-schema-change-markup-in-a-modified-unit",
        note: "a deleted phrase and an inserted one in the statement of a requirement marked \
               modified; the markup is permitted only in a change's specification, and a \
               fixture sits in none",
        rules: &["schema/change-markup"],
        accepted: false,
        edit: |base| {
            let modified = replace(
                base,
                r#"<section class="requirement" id="req-example" data-change="added">"#,
                r#"<section class="requirement" id="req-example" data-change="modified">"#,
            );
            replace(
                modified,
                "The system SHALL state one obligation per requirement",
                &format!("The system SHALL state {CHANGE_MARKUP} per requirement"),
            )
        },
        assets: &[],
    },
    Fixture {
        name: "valid-change-markup-outside-a-unit",
        note: "the same deleted and inserted phrases in prose outside every unit, which \
               projects into no unit's field, so a document may show a wording change there",
        rules: &[],
        accepted: false,
        edit: |base| {
            replace(
                base,
                "<h2>Requirements</h2>\n          <section class=\"requirement\"",
                &format!(
                    "<h2>Requirements</h2>\n          <p>Each requirement states \
                     {CHANGE_MARKUP}.</p>\n          <section class=\"requirement\""
                ),
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // kind/*
    //
    // Two rules and no more. The two fixtures below pin them, and the four after
    // them pin the *absence* of every other kind rule — which is the harder half,
    // because a rule that quietly stopped firing passes every test written to
    // assert silence. A positive fixture carrying a would-be violation fails in the
    // other direction: add a placement matrix later and these stop being valid.
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-kind-task-in-specification",
        note: "a specification carrying a task unit; the vocabulary's one placement \
               rule, because a specification records the obligation a task satisfied \
               rather than the work that satisfied it",
        rules: &["kind/task-in-specification"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section class=\"task\" id=\"task-misplaced\" \
                 data-state=\"done\">\n\
                 \x20         <h2>A task in the wrong document</h2>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-kind-no-tasks",
        note: "a tasks document carrying no task unit; the stage exists to enumerate \
               work and this one enumerates none",
        rules: &["kind/no-tasks"],
        accepted: false,
        edit: |base| as_kind(base, "tasks"),
        assets: &[],
    },
    Fixture {
        name: "warning-review-unverified-requirement",
        note: "a specification requirement that no verification names; advisory, so \
               the document is accepted — recording verification is the work in \
               progress rather than a violation",
        rules: &["review/unverified-requirement"],
        accepted: true,
        edit: |base| {
            // The base's verification names the requirement, so removing that
            // reference is what leaves the requirement unattested. The verification
            // stays: a specification with no verification at all would be a
            // different fixture, and this one is about the requirement.
            replace(
                base,
                "<section class=\"verification\" id=\"verify-example\" data-change=\"added\" \
                 data-outcome=\"not-run\" data-refs=\"req-example\">",
                "<section class=\"verification\" id=\"verify-example\" data-change=\"added\" \
                 data-outcome=\"not-run\" data-refs=\"scenario-example\">",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "warning-review-example-content",
        note: "a requirement whose prose is the author's own and whose identifier is \
               still the template's; advisory, because creation writes the template \
               and the result is required to be valid",
        rules: &["review/example-content"],
        accepted: true,
        edit: |base| {
            // The half-replaced case, which the contract names as the likely one rather
            // than the edge one: text is edited while an identifier survives. Rewriting
            // the prose and keeping `req-example` is what makes this fixture pin the
            // check on the *identifier* — a check reading the text would find nothing
            // template-written here and report nothing.
            //
            // The base already carries the identifier untouched, so this fixture is not
            // what makes the rule fire; it is what makes a fixture *declare* it. Every
            // other fixture reports the rule and declares nothing, which is the state
            // that let the rule ship with no fixture pinning it at all.
            replace(
                base,
                "<p>The system SHALL state one obligation per requirement, in a sentence \
                 a reader can verify.</p>",
                "<p>The system SHALL refuse a request whose signature does not verify.</p>",
            )
        },
        assets: &[],
    },
    // Tasks 7.4 and 7.5: the upstream freedom, as *positive* fixtures.
    //
    // Each carries a unit that would be a violation if any placement matrix
    // existed, and each declares `valid: true` with no rules. The suite's existing
    // `every_fixture_is_accepted_exactly_when_it_declares_it_is` does the work, so
    // there is no new harness concept — and if someone later adds a placement rule
    // for these combinations, these fixtures fail.
    Fixture {
        name: "valid-kind-proposal-with-a-requirement",
        note: "a proposal carrying a requirement unit, which is valid: an author \
               arguing for a prospective requirement is doing the proposal's job, and \
               the unit is permitted without being authoritative",
        rules: &[],
        accepted: false,
        edit: |base| as_kind(base, "proposal"),
        assets: &[],
    },
    // A run's record, both ways, which is what makes the shape a contract rather than an
    // implementation detail.
    //
    // **Both are `valid: true` with no rules, and that pairing is the whole point.** The
    // conforming record shows the shape is accepted — a `tasks` document carrying task units,
    // copied requirements, and verifications together — and the short one shows that a record
    // missing its verification is *also* accepted, because no per-document rule can decide it.
    // `review/unverified-requirement` is scoped to the `specification` kind, so this pair is the
    // measurement that motivates a project-level rule: the second fixture is a record that has
    // recorded nothing and that validation has nothing to say about.
    //
    // The rule that *does* decide it is pinned in `conformance/tests/runs.rs`, whose fixtures are
    // runbook-and-records pairs rather than documents. That is `asset/duplicate`'s arrangement, and
    // `UNFIXTURABLE` in the harness records why.
    Fixture {
        name: "valid-runbook-run-record",
        note: "a run's record: a tasks document carrying the copied procedure, the copied \
               requirements it attests to, and a verification recording each outcome — the \
               shape forced by a reference resolving within one document only",
        rules: &[],
        accepted: false,
        edit: |base| a_run_record(base, RunRecord::Conforming),
        assets: &[],
    },
    Fixture {
        name: "valid-runbook-run-record-missing-its-verification",
        note: "the same record with the verification removed: accepted, and reported by \
               nothing, because whether a run recorded what its runbook required is a fact \
               about two documents and no per-document rule can decide it",
        rules: &[],
        accepted: false,
        edit: |base| a_run_record(base, RunRecord::Short),
        assets: &[],
    },
    Fixture {
        name: "valid-kind-design-with-a-task",
        note: "a design carrying a task unit, which is valid: designing is where the \
               shape of the work becomes clear, and the design kind imposes no \
               structure at all",
        rules: &[],
        accepted: false,
        edit: |base| {
            let design = as_kind(base, "design");
            append_to_main(
                design,
                "        <section class=\"task\" id=\"task-in-a-design\" \
                 data-state=\"todo\">\n\
                 \x20         <h2>Work the design made visible</h2>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    // Task 7.6: the boundary of that freedom. Structural freedom is not permission
    // to ship a document that will not open, and the contract rules are not relaxed
    // by kind.
    Fixture {
        name: "invalid-kind-proposal-with-a-remote-script",
        note: "a proposal loading a remote script, which fails exactly as any other \
               kind would; a kind decides how much shape validation requires and \
               never whether the document opens",
        rules: &["safety/remote-resource", "safety/unapproved-script"],
        accepted: false,
        edit: |base| {
            append_to_head(
                as_kind(base, "proposal"),
                "    <script src=\"https://example.test/analytics.js\"></script>",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // refs/*
    // -------------------------------------------------------------------------
    // Task 4.3: fires from the base's *evidence* and *verification*, which carry
    // `data-refs` in the specification template. Verified by building a task-free
    // document and running the binary: this rule needs a reference, not a task.
    Fixture {
        name: "invalid-refs-unresolved",
        note: "a unit referencing an identifier no unit declares",
        rules: &["refs/unresolved"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                r#"data-refs="req-example""#,
                r#"data-refs="req-absent""#,
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-refs-broken-link",
        note: "a fragment link to an element the document does not contain, so a reader \
               following it arrives nowhere",
        rules: &["refs/broken-link"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                r##"<li><a href="#hs-requirements">Requirements</a></li>"##,
                r##"<li><a href="#hs-absent-section">Requirements</a></li>"##,
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-refs-removed-target",
        note: "a reference to a requirement this change withdraws; the reference \
               resolves and points at something being deleted",
        rules: &["refs/removed-target"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                r#"<section class="requirement" id="req-example" data-change="added">"#,
                r#"<section class="requirement" id="req-example" data-change="removed">"#,
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // safety/* — network
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-safety-remote-script",
        note: "a script loaded by absolute URL, which no lockfile pins and which does \
               not render offline",
        rules: &["safety/remote-resource", "safety/unapproved-script"],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                "    <script src=\"https://example.test/analytics.js\"></script>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-safety-remote-stylesheet",
        note: "a stylesheet from a remote origin, which a document cannot render \
               without a network",
        rules: &["safety/remote-resource"],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                "    <link href=\"https://example.test/theme.css\" rel=\"stylesheet\">",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-safety-remote-font",
        note: "a font loaded from a remote origin through an inline style, which the \
               network rules must see inside CSS and not only in an attribute",
        rules: &["safety/remote-resource"],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                "    <style>@font-face { font-family: X; \
                 src: url(https://example.test/x.woff2); }</style>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-safety-concealed-request",
        note: "a prefetch directive, which issues a request with no visible remote \
               reference for a reader to notice",
        rules: &["safety/concealed-request"],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                "    <link href=\"https://example.test/next\" rel=\"prefetch\">",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-safety-remote-chart-data",
        note: "a chart specification loading its data from a remote URL; Vega-Lite \
               permits this by design, so the offline rule is our own",
        rules: &["safety/remote-chart-data"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section id=\"hs-remote-chart\">\n\
                 \x20         <h2>A chart with remote data</h2>\n\
                 \x20         <figure id=\"figure-remote\">\n\
                 \x20           <figcaption>A chart whose data is elsewhere.</figcaption>\n\
                 \x20           <details>\n\
                 \x20             <summary>Chart source</summary>\n\
                 \x20             <pre class=\"vega-lite\">{\n\
                 \x20 \"$schema\": \"https://vega.github.io/schema/vega-lite/v6.json\",\n\
                 \x20 \"data\": {\"url\": \"https://example.test/data.csv\"},\n\
                 \x20 \"mark\": \"bar\"\n\
                 }</pre>\n\
                 \x20           </details>\n\
                 \x20         </figure>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // safety/* — executable content
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-safety-live-example",
        note: "a code element with an element descendant; the example escaped its \
               container and is no longer inert text",
        rules: &["safety/live-example"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section id=\"hs-live\">\n\
                 \x20         <h2>An example that is not inert</h2>\n\
                 \x20         <pre><code>text <b>bold</b> more</code></pre>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-safety-unapproved-script",
        note: "an inline script, which no lockfile can pin and which therefore is not \
               part of the shared runtime",
        rules: &["safety/unapproved-script"],
        accepted: false,
        edit: |base| append_to_head(base, "    <script>window.x = 1;</script>"),
        assets: &[],
    },
    Fixture {
        name: "invalid-safety-event-handler",
        note: "an inline event handler, which executes on interaction and belongs to no \
               pinned runtime",
        rules: &["safety/event-handler"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                r#"<h1>A conformance fixture</h1>"#,
                r#"<h1 onclick="alert(1)">A conformance fixture</h1>"#,
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-safety-executable-url",
        note: "a link whose scheme executes rather than fetches",
        rules: &["safety/executable-url"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                r##"<a class="hs-skip-link" href="#hs-content">Skip to content</a>"##,
                r#"<a class="hs-skip-link" href="javascript:void(0)">Skip to content</a>"#,
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // viz/*
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-viz-chart-not-json",
        note: "a chart source that is not JSON at all, so nothing can be validated \
               against the chart schema",
        rules: &["viz/chart-not-json"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section id=\"hs-bad-json\">\n\
                 \x20         <h2>A malformed chart</h2>\n\
                 \x20         <figure id=\"figure-bad-json\">\n\
                 \x20           <figcaption>Not JSON.</figcaption>\n\
                 \x20           <details>\n\
                 \x20             <summary>Chart source</summary>\n\
                 \x20             <pre class=\"vega-lite\">{not json at all</pre>\n\
                 \x20           </details>\n\
                 \x20         </figure>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-viz-chart-invalid",
        note: "a chart specification Vega-Lite's own schema rejects, caught by real \
               validation rather than a syntax check",
        rules: &["viz/chart-invalid"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section id=\"hs-bad-chart\">\n\
                 \x20         <h2>An invalid chart</h2>\n\
                 \x20         <figure id=\"figure-bad-chart\">\n\
                 \x20           <figcaption>An unknown mark type.</figcaption>\n\
                 \x20           <details>\n\
                 \x20             <summary>Chart source</summary>\n\
                 \x20             <pre class=\"vega-lite\">{\n\
                 \x20 \"$schema\": \"https://vega.github.io/schema/vega-lite/v6.json\",\n\
                 \x20 \"data\": {\"values\": [{\"a\": 1}]},\n\
                 \x20 \"mark\": \"hexagonal-prism\"\n\
                 }</pre>\n\
                 \x20           </details>\n\
                 \x20         </figure>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-viz-diagram-empty",
        note: "an empty diagram source, which renders as nothing and says nothing",
        rules: &["viz/diagram-empty"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                "                <pre class=\"mermaid\">graph LR\n\
                 \x20 A[Authored HTML] --&gt; B[Parse]\n\
                 \x20 B --&gt; C[Projected model]\n\
                 \x20 C --&gt; D[Diagnostics]</pre>",
                "                <pre class=\"mermaid\"></pre>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-viz-diagram-unrecognized",
        note: "a diagram whose leading keyword names no diagram type Mermaid has",
        rules: &["viz/diagram-unrecognized"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                "<pre class=\"mermaid\">graph LR",
                "<pre class=\"mermaid\">sandwich LR",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // a11y/*
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-a11y-figure-without-caption",
        note: "a figure with no caption, leaving a reader who cannot see the rendering \
               with nothing describing it",
        rules: &["a11y/figure-without-caption"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                "              <figcaption>An example diagram. Its source is visible below, \
                 so this figure is readable with scripting disabled.</figcaption>\n",
                "",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-a11y-figure-without-fallback",
        note: "a figure whose source is absent, so a reader with scripting disabled \
               sees an empty region",
        rules: &["a11y/figure-without-fallback"],
        accepted: false,
        // A mermaid figure whose source is whitespace only. Emptying the `pre`
        // entirely would make the figure project as `prose`, which legitimately
        // needs no source, and the rule would correctly not fire — so the fixture
        // must keep the class and lose the content.
        edit: |base| {
            replace(
                base,
                "<pre class=\"mermaid\">graph LR\n\
                 \x20 A[Authored HTML] --&gt; B[Parse]\n\
                 \x20 B --&gt; C[Projected model]\n\
                 \x20 C --&gt; D[Diagnostics]</pre>",
                "<pre class=\"mermaid\">   </pre>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-a11y-hidden-figure-source",
        note: "a figure carrying its source in a script block, which is invisible \
               without JavaScript while appearing to satisfy the readability rule",
        // `type="application/json"` is data rather than an executable script, so the
        // executable-content rule correctly stays silent; only the a11y rule fires.
        rules: &["a11y/hidden-figure-source"],
        accepted: false,
        edit: |base| {
            append_to_main(
                base,
                "        <section id=\"hs-hidden\">\n\
                 \x20         <h2>A hidden source</h2>\n\
                 \x20         <figure id=\"figure-hidden\">\n\
                 \x20           <figcaption>The source is in a script block.</figcaption>\n\
                 \x20           <script type=\"application/json\">{\"mark\": \"bar\"}</script>\n\
                 \x20         </figure>\n\
                 \x20       </section>",
            )
        },
        assets: &[],
    },
    Fixture {
        name: "invalid-a11y-script-supplied-content",
        note: "a unit whose text is absent from the served markup, so a reader with \
               scripting disabled sees an empty unit",
        rules: &["a11y/script-supplied-content"],
        accepted: false,
        edit: |base| {
            // The *verification*, whose `method` field is optional — so a
            // verification stripped of its heading and its method carries no visible
            // text and is otherwise schema-shaped. Evidence would not work: its
            // `summary` is required, so emptying it reports a schema fault at the
            // same location and the fixture would pin two rules for one edit. I
            // tested both before choosing.
            //
            // This was a task before the artifact kinds landed. The base is a
            // `specification` now and carries none, and this fixture is the second
            // one that depended on the task the base lost.
            let emptied = replace(
                base,
                "            <h3>The example verification <span class=\"hs-id\">verify-example\
                 </span></h3>\n",
                "            <h3></h3>\n",
            );
            replace(
                emptied,
                "            <p data-field=\"method\">State how the requirement was checked, \
                 precisely enough that someone else could repeat it.</p>\n",
                "",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // review/* — warnings, so these only fail under strict validation
    // -------------------------------------------------------------------------
    Fixture {
        name: "warning-review-stale-reference",
        note: "a task referencing a requirement this change alters; a review marker, \
               reported and never gating work",
        rules: &["review/stale-reference"],
        accepted: true,
        edit: |base| {
            replace(
                base,
                r#"<section class="requirement" id="req-example" data-change="added">"#,
                r#"<section class="requirement" id="req-example" data-change="modified">"#,
            )
        },
        assets: &[],
    },
    Fixture {
        name: "warning-review-stale-verification",
        note: "a verification recording a pass against a requirement that has since \
               moved, which attests to an obligation nobody made",
        rules: &["review/stale-verification"],
        accepted: true,
        edit: |base| {
            let modified = replace(
                base,
                r#"<section class="requirement" id="req-example" data-change="added">"#,
                r#"<section class="requirement" id="req-example" data-change="modified">"#,
            );
            replace(
                modified,
                r#"data-outcome="not-run""#,
                r#"data-outcome="passed""#,
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // parse/*
    // -------------------------------------------------------------------------
    Fixture {
        name: "invalid-parse-recovered-error",
        note: "misnested elements, whose reading depends on parser error recovery; \
               every recovery is fatal and curtails the run",
        rules: &["parse/recovered-error"],
        accepted: false,
        edit: |base| {
            replace(
                base,
                "<p>Replace this paragraph",
                "<p><strong>Replace this paragraph",
            )
        },
        assets: &[],
    },
    // -------------------------------------------------------------------------
    // asset/*
    //
    // These are the only fixtures that carry an `assets/` directory of their own, and
    // that is what brings the `asset/*` rules into scope for them and leaves them out
    // of scope for the other 37. The scope is decided by the fixture's surroundings,
    // never by anything inside its document — see the module docs of
    // `htmlspec_validate::rules::assets`.
    //
    // The stylesheet bytes below are deliberately tiny and not the real stylesheet. A
    // fixture referencing the project's actual stylesheet would have to be regenerated
    // every time that file changed, and its committed bytes are what it pins.
    // -------------------------------------------------------------------------
    Fixture {
        name: "valid-asset-reference-matches-stored-bytes",
        note: "an addressed reference naming the hash of the bytes stored at it, in a \
               fixture that carries a real store; the accepting side of both asset rules",
        rules: &[],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                &format!(
                    "    <link href=\"assets/{}\" rel=\"stylesheet\">",
                    stored_path_of(FIXTURE_STYLESHEET)
                ),
            )
        },
        assets: &[StoredAsset {
            name: FIXTURE_STYLESHEET_NAME,
            bytes: FIXTURE_STYLESHEET,
            addressed_as: FIXTURE_STYLESHEET,
        }],
    },
    Fixture {
        name: "invalid-asset-unresolved",
        note: "a reference naming bytes the project does not store; the address is a \
               claim about bytes and nothing is stored there",
        rules: &["asset/unresolved"],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                &format!(
                    "    <link href=\"assets/{}\" rel=\"stylesheet\">",
                    stored_path_of(FIXTURE_STYLESHEET_ABSENT)
                ),
            )
        },
        // A store is present and holds a different revision, so the reference resolves
        // to nothing rather than the rule being out of scope.
        assets: &[StoredAsset {
            name: FIXTURE_STYLESHEET_NAME,
            bytes: FIXTURE_STYLESHEET,
            addressed_as: FIXTURE_STYLESHEET,
        }],
    },
    Fixture {
        name: "invalid-asset-mismatch",
        note: "an addressed location whose contents were replaced with different bytes; \
               reported as a mismatch rather than an absence, because the causes differ",
        rules: &["asset/mismatch"],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                &format!(
                    "    <link href=\"assets/{}\" rel=\"stylesheet\">",
                    stored_path_of(FIXTURE_STYLESHEET)
                ),
            )
        },
        // Written at the address of one revision, holding another: the in-place
        // replacement, which renders and looks fine.
        assets: &[StoredAsset {
            name: FIXTURE_STYLESHEET_NAME,
            bytes: FIXTURE_STYLESHEET_REPLACED,
            addressed_as: FIXTURE_STYLESHEET,
        }],
    },
    // A document separated from the project holding its store.
    Fixture {
        name: "invalid-asset-orphaned-ascending-reference",
        note: "an addressed reference that climbs out of the document's own directory with \
               no store in any directory above it; a change directory copied out of its \
               project keeps references to a store that is no longer there, and renders \
               against nothing. Ascent is what distinguishes this from a fixture whose \
               references are symbolic: no reference in this suite ascends",
        rules: &["asset/unresolved"],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                &format!(
                    "    <link href=\"../../../assets/{}\" rel=\"stylesheet\">",
                    stored_path_of(FIXTURE_STYLESHEET)
                ),
            )
        },
        // No store, which is the whole point: with one present the reference would resolve
        // normally. The rule fires because the reference asserts a store above the document
        // and there is none.
        assets: &[],
    },
    // Task 3.8: the defect that prompted this change, as a fixture.
    Fixture {
        name: "invalid-asset-stale-stylesheet-reference",
        note: "a document authored against one stylesheet revision and stored beside a \
               different one, which is the archived-page defect this change exists for",
        rules: &["asset/unresolved"],
        accepted: false,
        edit: |base| {
            append_to_head(
                base,
                &format!(
                    "    <link href=\"assets/{}\" rel=\"stylesheet\">",
                    stored_path_of(STALE_STYLESHEET_OLD)
                ),
            )
        },
        // The project stores the current revision, which has the skip-link rule. The
        // document names the older one, which does not.
        assets: &[StoredAsset {
            name: FIXTURE_STYLESHEET_NAME,
            bytes: STALE_STYLESHEET_CURRENT,
            addressed_as: STALE_STYLESHEET_CURRENT,
        }],
    },
];

/// The file name the `asset/*` fixtures store and reference.
const FIXTURE_STYLESHEET_NAME: &str = "htmlspec.css";

/// The stylesheet revision the `asset/*` fixtures store.
const FIXTURE_STYLESHEET: &[u8] = b"/* a fixture stylesheet */\n";

/// A revision no fixture stores, so a reference to it resolves to nothing.
const FIXTURE_STYLESHEET_ABSENT: &[u8] = b"/* a revision this project does not store */\n";

/// Bytes written over a stored file in place, producing a mismatch.
const FIXTURE_STYLESHEET_REPLACED: &[u8] = b"/* these bytes replaced the stored ones */\n";

/// The current stylesheet in the stale-reference fixture: it has the skip-link rule.
const STALE_STYLESHEET_CURRENT: &[u8] = b".hs-skip-link { left: -9999px }\n";

/// The older revision the archived document names, which has no skip-link rule.
const STALE_STYLESHEET_OLD: &[u8] = b"/* no skip link rule */\n";

/// The change markup every `schema/change-markup` fixture carries.
const CHANGE_MARKUP: &str = "<del>every obligation</del><ins>one obligation</ins>";

/// Fixtures written by hand, which this generator neither writes nor removes.
///
/// None of the four is one stated edit to the base. Two are runbook documents, from the
/// change that carried a runbook to another machine. Two pin how a summary is compared with a
/// template's placeholder, and each replaces the base's example units with units of its own.
/// Their committed bytes are the fixture, and `expected.json` for the unsatisfiable runbook
/// holds its rule list on one line, which this generator would rewrite.
///
/// Before this table, the sweep in [`main`] deleted all four on every run. That was met
/// twice: `the-generator-deletes-two-fixtures-2026-09-18.md` in the runbook-run change found
/// two, and the renamed-requirement change found four and restored them by hand. A list here
/// is what stops a third contributor meeting it.
const HAND_AUTHORED: &[&str] = &[
    "valid-runbook-machine-requirements",
    "valid-runbook-machine-requirements-unsatisfiable",
    "valid-summary-quoting-the-placeholder",
    "warning-review-placeholder-summary",
];

/// The stored path `bytes` would occupy, as a document references it.
///
/// Derived through the same function the store and the Nix check use, so a fixture
/// cannot reference an address the store would not recognize.
fn stored_path_of(bytes: &[u8]) -> String {
    htmlspec_assets::stored_path(&htmlspec_assets::sha256_hex(bytes), FIXTURE_STYLESHEET_NAME)
}

/// Writes a fixture's own asset store, or removes one it no longer declares.
///
/// The removal matters as much as the writing: a fixture that stopped declaring assets
/// but kept the directory would still be *in scope* for the `asset/*` rules, and would
/// pass or fail for a reason no longer visible in this file.
fn write_store(directory: &Path, assets: &[StoredAsset]) {
    let store = directory.join("assets");
    let _ = std::fs::remove_dir_all(&store);
    if assets.is_empty() {
        return;
    }
    for asset in assets {
        let stored = htmlspec_assets::stored_path(
            &htmlspec_assets::sha256_hex(asset.addressed_as),
            asset.name,
        );
        let path = store.join(&stored);
        std::fs::create_dir_all(path.parent().expect("an address directory"))
            .expect("the address directory is created");
        std::fs::write(&path, asset.bytes).expect("the stored asset is written");
    }
}

fn main() {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/conformance");
    std::fs::create_dir_all(&root).expect("the fixture root is created");

    // A name in both tables would be written over a hand-authored fixture, and a name in
    // the hand-authored table with no directory is an entry that keeps nothing.
    for fixture in FIXTURES {
        assert!(
            !HAND_AUTHORED.contains(&fixture.name),
            "{} is both generated and hand-authored, so generating it would overwrite the \
             committed one",
            fixture.name
        );
    }
    for name in HAND_AUTHORED {
        for file in ["input.html", "expected.json"] {
            assert!(
                root.join(name).join(file).is_file(),
                "the hand-authored fixture {name} has no {file}. Restore it, or remove the \
                 fixture from HAND_AUTHORED if it was withdrawn"
            );
        }
    }

    // Remove any fixture no longer declared, so a renamed one does not leave a stale
    // directory the harness would still drive. A hand-authored fixture is declared by
    // its table.
    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path
                .file_name()
                .expect("named")
                .to_string_lossy()
                .into_owned();
            if !FIXTURES.iter().any(|f| f.name == name) && !HAND_AUTHORED.contains(&name.as_str()) {
                println!("removing stale fixture {name}");
                std::fs::remove_dir_all(&path).expect("the stale fixture is removed");
            }
        }
    }

    let base = base();
    for fixture in FIXTURES {
        let directory = root.join(fixture.name);
        std::fs::create_dir_all(&directory).expect("the fixture directory is created");

        let document = (fixture.edit)(base.clone());
        assert!(
            fixture.rules.is_empty() || document != base,
            "{} claims a rule and is identical to the base document",
            fixture.name
        );

        std::fs::write(directory.join("input.html"), &document).expect("input.html is written");
        write_store(&directory, fixture.assets);

        let expected = serde_json::json!({
            "valid": fixture.rules.is_empty() || fixture.accepted,
            "rules": fixture.rules,
            "note": fixture.note,
        });
        let mut text = serde_json::to_string_pretty(&expected).expect("the expectation serializes");
        text.push('\n');
        std::fs::write(directory.join("expected.json"), text).expect("expected.json is written");

        println!("wrote {}", fixture.name);
    }
    println!("\n{} fixtures", FIXTURES.len());
}
