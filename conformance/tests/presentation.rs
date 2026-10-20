//! The stylesheet must serve every conforming document, not only the template's.
//!
//! This file exists because four presentation defects reached a reviewer's screen
//! that the rest of the suite could not see. The suite proves a document *validates*;
//! nothing proved it *renders correctly*, and one of the four silently violated a
//! stated requirement rather than merely looking wrong.
//!
//! These checks read the stylesheet's source rather than a rendered page. That is a
//! real limitation and it is the deliberate trade: computing styles needs a headless
//! browser, which the design keeps out of scope, and both defects worth catching turn
//! out to be *selector-coverage* questions — which headings a rule names, which
//! elements a rule is keyed on — that the source answers exactly. A cascade or
//! layout defect would not be caught here, and `evidence/browser-verification.md`
//! records what was verified in a real browser instead.
//!
//! The two properties, and the defect each is here for:
//!
//! 1. **A change state is never carried by hue alone.** The badge rules covered `h2`
//!    and `h3`; a scenario's heading is an `h4`. Scenarios therefore had no text
//!    label, and dropping their border — which the nesting fix required — would have
//!    left hue as the only channel. The specs forbid exactly that.
//!
//! 2. **A document with no `hs-*` class still gets a page frame.** The frame was
//!    keyed on `.hs-page`. The contract requires no class at all, so the three
//!    hand-authored documents rendered as full-window text against the viewport edge.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The project's stylesheet, read from the embedded asset table.
///
/// By file name rather than by path. The stylesheet now lives at its content address, so
/// a literal path would name a directory that changes whenever the bytes do — and these
/// checks are about the stylesheet's *content*, which is exactly what the address encodes.
/// The embedded bytes are the committed bytes, which
/// `every_own_asset_matches_its_recorded_hash` verifies against the lockfile.
fn stylesheet() -> String {
    let asset = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == "htmlspec.css")
        .expect("the stylesheet is embedded");
    String::from_utf8(asset.bytes.to_vec()).expect("the stylesheet is UTF-8")
}

/// Every document this project ships: the template `init` writes, the hand-authored
/// dogfood documents, and the converted capability spec.
///
/// Read from disk rather than listed, so a document added to the project is held to
/// these properties without an edit here.
fn shipped_documents() -> Vec<(String, String)> {
    let root = repo_root();
    let mut found = Vec::new();

    found.push((
        String::from("template.html"),
        htmlspec_assets::render_template("A subject", "assets"),
    ));

    // Documents live under a change while it is active and under the archive once it
    // is archived, so both are read. A path that named only the active location went
    // stale the day the change was archived, and the count assertion below was the only
    // thing that noticed.
    //
    // **Finding none is a failure, not an empty loop.** With `openspec/`, where these were
    // read from until 0.30.1, moved aside, the checks still passed reading four fewer
    // documents, because the count assertion below is satisfied by the template and the
    // dogfood documents alone.
    let changes = change_document_directories(&root);
    assert!(
        !changes.is_empty(),
        "no change holds a documents directory where these checks look, so they read no \
         change's documents. The directories `change_document_directories` names have moved"
    );
    let mut directories = vec![root.join("fixtures/dogfood")];
    directories.extend(changes);

    for path in directories {
        let Ok(entries) = std::fs::read_dir(&path) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "html"))
            .collect();
        paths.sort();
        for path in paths {
            let name = path
                .file_name()
                .expect("named file")
                .to_string_lossy()
                .into_owned();
            found.push((
                name,
                std::fs::read_to_string(&path).expect("the document is readable"),
            ));
        }
    }

    assert!(
        found.len() >= 5,
        "expected the templates, six dogfood documents, and the converted spec; found {}",
        found.len()
    );
    found
}

/// Every `evidence/documents/` directory belonging to a change, active or archived.
///
/// Where migration put the hand-converted documents a change kept beside its Markdown, byte for
/// byte. Discovered rather than listed, so archiving a change does not silently drop its
/// documents out of these checks.
fn change_document_directories(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for changes in [
        root.join("htmlspec/changes"),
        root.join("htmlspec/changes/archive"),
    ] {
        let Ok(entries) = std::fs::read_dir(&changes) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path().join("evidence/documents"))
            .filter(|path| path.is_dir())
            .collect();
        paths.sort();
        found.extend(paths);
    }
    found
}

/// The heading levels that immediately follow an element carrying a state attribute,
/// across every shipped document.
///
/// This is the set the badge rules must cover. Derived from the documents rather than
/// hard-coded, because the `h4` defect was precisely a hard-coded assumption — the
/// rules named the levels someone had thought of, and a real document used another.
fn heading_levels_carrying_state() -> BTreeSet<String> {
    let mut levels = BTreeSet::new();
    for (name, source) in shipped_documents() {
        for attribute in STATE_ATTRIBUTES {
            let needle = format!("data-{attribute}=\"");
            let mut rest = source.as_str();
            while let Some(at) = rest.find(&needle) {
                rest = &rest[at + needle.len()..];
                // The unit's own heading is the first one after its opening tag. A
                // window is enough: a heading further away belongs to a nested unit.
                let window = &rest[..rest.len().min(600)];
                if let Some(level) = first_heading(window) {
                    levels.insert(level);
                } else {
                    // A state attribute with no heading nearby would mean a unit with
                    // no visible name, which the schema already rejects. Worth
                    // knowing about rather than passing over.
                    panic!("{name}: a data-{attribute} element has no heading near it");
                }
            }
        }
    }
    assert!(
        !levels.is_empty(),
        "no shipped document carries a state attribute, so this test proves nothing"
    );
    levels
}

/// The tag name of the first `h1`–`h6` opening tag in `text`.
fn first_heading(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    for (at, _) in text.char_indices() {
        if bytes[at] != b'<' {
            continue;
        }
        let rest = &text[at + 1..];
        let mut chars = rest.chars();
        if chars.next() != Some('h') {
            continue;
        }
        let Some(digit) = chars.next() else { continue };
        if !('1'..='6').contains(&digit) {
            continue;
        }
        // The character after the level must end the tag name, or `<hgroup>` would
        // read as `h1` through the letter that follows.
        if chars.next().is_some_and(|c| c.is_ascii_alphanumeric()) {
            continue;
        }
        return Some(format!("h{digit}"));
    }
    None
}

/// The attributes that carry a change state, a lifecycle state, or an outcome.
///
/// These are the values the visual system must convey without relying on hue.
const STATE_ATTRIBUTES: &[&str] = &["change", "state", "status", "outcome"];

/// Property 1: a state is never carried by hue alone.
///
/// For every heading level a real document uses on a state-bearing element, the
/// stylesheet must set generated content — the text label — for that level. This is
/// the check that would have caught the `h4` gap: scenarios use `h4`, the badge rules
/// named only `h2` and `h3`, so a scenario's state had no text channel at all.
#[test]
fn every_heading_level_that_carries_a_state_gets_a_text_label() {
    let css = stylesheet();
    let mut missing = Vec::new();

    for level in heading_levels_carrying_state() {
        for attribute in STATE_ATTRIBUTES {
            // The badge selector for this attribute and this heading level. Matched
            // on the selector text, which is what decides whether the rule applies.
            let selector = format!("> {level}::after");
            let mentions_attribute = css
                .lines()
                .filter(|line| line.contains(&format!("[data-{attribute}")))
                .any(|line| line.contains(&selector));
            if !mentions_attribute {
                missing.push(format!("data-{attribute} on a {level}"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "these state attributes have no text label at the heading level a real document \
         uses, so the state would be carried by hue alone:\n  {}\n\nAdd `> {}::after` to the \
         badge selectors in assets/htmlspec.css. The specs require that change state be \
         distinguishable without relying on hue.",
        missing.join("\n  "),
        heading_levels_carrying_state()
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join("/")
    );
}

/// Every state value the schema enumerates has a label, not just every heading level.
///
/// The check above asks whether the *level* is covered; this asks whether the *value*
/// is. A state added to the schema with no `content` rule would render a bordered
/// empty box — visibly wrong, and wrong in a way nobody would attribute to a missing
/// stylesheet rule.
#[test]
fn every_enumerated_state_value_has_a_label() {
    let css = stylesheet();
    let paths = [
        ("change", "definitions/changeState"),
        ("state", "definitions/task/properties/state"),
        ("status", "definitions/decision/properties/status"),
        ("outcome", "definitions/verification/properties/outcome"),
    ];

    let mut missing = Vec::new();
    for (attribute, path) in paths {
        let values = htmlspec_schema::facts::enum_values(path);
        assert!(
            !values.is_empty(),
            "the schema enumerates no values at {path}, so this test covers nothing"
        );
        for value in values {
            // `unchanged` is the default and deliberately unlabelled: badging every
            // untouched unit would make the badge meaningless.
            if value == "unchanged" {
                continue;
            }
            let selector = format!("[data-{attribute}=\"{value}\"]");
            if !css.contains(&selector) {
                missing.push(format!("{selector} has no rule"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "the schema enumerates state values the stylesheet does not label:\n  {}",
        missing.join("\n  ")
    );
}

/// Property 2: a document with no `hs-*` class still gets a page frame.
///
/// The contract requires no class: the projection ignores every class token beyond the
/// reserved seven and treats `header`, `nav`, and `main` as presentational. So a
/// hand-authored document, or one from an independent implementation, carries no
/// wrapper — and a frame keyed on `.hs-page` leaves it with full-window text flush
/// against the viewport edge, which is how this was found.
#[test]
fn the_page_frame_does_not_require_a_wrapper_class() {
    let css = stylesheet();

    // The frame's own declarations must be reachable through `body`, not only through
    // the wrapper. Checked by finding the rule that sets the page width and asserting
    // its selector list names `body` on its own.
    let framed_by_body = css
        .split('}')
        .filter(|rule| rule.contains("max-width: calc(var(--hs-nav-width)"))
        .any(|rule| {
            let selector = rule.split('{').next().unwrap_or_default();
            selector
                .split(',')
                .any(|part| part.trim().trim_end_matches(|c: char| c.is_whitespace()) == "body")
        });

    assert!(
        framed_by_body,
        "the page frame is keyed on a wrapper class only. A conforming document carries no \
         hs-* class, so it would render as full-window text against the viewport edge. Add \
         `body` to the selector list of the rule setting the page width in \
         assets/htmlspec.css."
    );
}

/// The three hand-authored documents genuinely have no `hs-*` class.
///
/// This is what makes the property above load-bearing rather than theoretical. If
/// every shipped document used the template's wrapper, keying the frame on the class
/// would be harmless and the test would be guarding nothing.
#[test]
fn the_hand_authored_documents_carry_no_presentation_classes() {
    let root = repo_root().join("fixtures/dogfood");
    let mut checked = 0;
    for entry in std::fs::read_dir(&root).expect("the dogfood fixtures exist") {
        let path = entry.expect("readable entry").path();
        if path.extension().is_none_or(|e| e != "html") {
            continue;
        }
        checked += 1;
        let source = std::fs::read_to_string(&path).expect("readable");
        assert!(
            !source.contains("hs-page"),
            "{} carries an hs-* wrapper; the frame property is then untested by it",
            path.display()
        );
    }
    assert_eq!(checked, 6, "expected the six hand-authored documents");
}

/// A scenario must not repeat its parent's left rule.
///
/// The defect this is for: a scenario carries `data-change`, the state rules set a
/// 4px colored border on anything that does, and the result was two parallel bars of
/// identical weight and hue — a blockquote inside a blockquote, carrying no
/// information the indent did not already carry.
#[test]
fn a_scenario_does_not_repeat_its_parents_left_rule() {
    let css = stylesheet();
    let rule = css
        .split('}')
        .find(|rule| {
            let selector = rule.split('{').next().unwrap_or_default();
            selector.lines().any(|l| l.trim() == ".scenario")
        })
        .expect("the stylesheet styles .scenario");

    assert!(
        rule.contains("border-left: none"),
        "a scenario sets a left border, which duplicates the bar of the unit it sits \
         inside. The state rules would also override a thinner one, so the border must be \
         removed with `!important`:\n{rule}"
    );
    assert!(
        rule.contains("!important"),
        "the scenario's border removal will be overridden by the change-state rules, which \
         set a 4px colored border on any element carrying data-change:\n{rule}"
    );
}

/// The print stylesheet, read from the embedded asset table for the reason [`stylesheet`] is.
fn print_stylesheet() -> String {
    let asset = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == "htmlspec-print.css")
        .expect("the print stylesheet is embedded");
    String::from_utf8(asset.bytes.to_vec()).expect("the print stylesheet is UTF-8")
}

/// The body of every rule whose selector list names `selector` exactly, joined.
///
/// Comments are removed first, because a comment above a rule sits in the same text as its
/// selector list once the source is split on braces, and a comma inside one splits it wrongly.
fn rules_for(css: &str, selector: &str) -> String {
    let mut uncommented = String::new();
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        uncommented.push_str(&rest[..start]);
        rest = rest[start..]
            .find("*/")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    uncommented.push_str(rest);
    uncommented
        .split('}')
        .filter(|rule| {
            rule.split('{')
                .next()
                .unwrap_or_default()
                .split(',')
                .any(|part| part.trim() == selector)
        })
        .map(|rule| rule.split('{').nth(1).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

/// A renamed requirement's previous heading is labelled, on screen and on paper.
///
/// The `renamed-from` paragraph is the previous heading as plain text, which reads with styles
/// off. With styles on, a reader should be told what the line is rather than infer it from its
/// position under the heading, and the label is generated content for the reason the
/// change-state label is: the fact already lives in `data-field`, and a label written into each
/// document would be a second place for it to disagree. The print stylesheet applies on top of
/// the screen one, so it has to carry the label to paper as well as the screen does, in ink.
#[test]
fn a_renamed_requirements_previous_heading_is_labelled() {
    const PARAGRAPH: &str = ".requirement [data-field=\"renamed-from\"]";
    let css = stylesheet();

    let before = rules_for(&css, &format!("{PARAGRAPH}::before"));
    assert!(
        before.contains("content:") && before.contains("Renamed from"),
        "no rule labels a renamed-from paragraph as the heading a requirement replaced:\n{before}"
    );
    assert!(
        before.contains("open-quote")
            && rules_for(&css, &format!("{PARAGRAPH}::after")).contains("close-quote"),
        "the previous heading is not quoted, so it reads as a sentence the requirement states"
    );
    let paragraph = rules_for(&css, PARAGRAPH);
    assert!(
        paragraph.contains("quotes:"),
        "the paragraph names no quotation marks, so open-quote falls back to the user agent's:\n{paragraph}"
    );
    assert!(
        !paragraph.contains("display: none") && !paragraph.contains("visibility: hidden"),
        "the previous heading is hidden, which puts content in the markup out of a reader's reach"
    );

    let print = rules_for(&print_stylesheet(), PARAGRAPH);
    assert!(
        print.contains("color:"),
        "the print stylesheet does not settle the previous heading's ink:\n{print}"
    );
}

/// The words a change inserted and deleted are drawn by shape and announced, never by hue alone.
///
/// `mark-the-words-a-modification-changed` permits `ins` and `del` in a modified unit of a
/// change's specification. The browser underlines `ins`, and this stylesheet underlines every
/// link and tells the two apart by the link's color alone, which document-template's "The visual
/// system favors comprehension over decoration" forbids as the only signal. So `ins` takes a
/// double underline, a shape no link has. `del` states its line-through rather than relying on the
/// browser's default. Most screen readers announce neither element, so each carries a start and an
/// end marker in generated content, which a screen reader reads and the page does not draw. And
/// neither is hidden: a reviewer has to see what a change deleted.
#[test]
fn an_edits_words_are_drawn_by_shape_and_announced() {
    let css = stylesheet();

    let ins = rules_for(&css, "ins");
    assert!(
        ins.contains("double"),
        "an inserted word is not drawn with a double underline, so it looks like a link:\n{ins}"
    );
    let links = rules_for(&css, "a");
    assert!(
        !links.is_empty() && !links.contains("double"),
        "a link is drawn with a double underline too, so the shape tells nothing apart:\n{links}"
    );

    let del = rules_for(&css, "del");
    assert!(
        del.contains("line-through"),
        "a deleted word's line-through is left to the browser's default:\n{del}"
    );

    for marker in ["ins::before", "ins::after", "del::before", "del::after"] {
        assert!(
            rules_for(&css, marker).contains("content:"),
            "{marker} carries no generated text, so a screen reader announces no edit"
        );
    }

    for element in [&ins, &del] {
        assert!(
            !element.contains("display: none") && !element.contains("visibility: hidden"),
            "an edit's words are hidden, which puts them out of a reviewer's reach:\n{element}"
        );
    }
}
