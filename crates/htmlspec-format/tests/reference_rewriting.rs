//! Tasks 5.2 through 5.4: reference rewriting, pinned from outside the crate.
//!
//! The property that matters most here is a *refusal*. A formatter that rewrote every
//! asset reference to the current address would silently re-point an archived document
//! at newer bytes, changing what a reviewed artifact renders with. That is a worse
//! defect than the stale reference this change exists to catch, because the document
//! still renders and nothing looks wrong.
//!
//! So the rewrite fills only what is absent. These tests drive the public API and assert
//! the refusal on a real document with a real store beside it, which is the arrangement
//! a contributor is actually in.

use std::path::PathBuf;

use htmlspec_assets::{address, sha256_hex, Store};
use htmlspec_format::{format, rewrite, KeptReason, ReferenceChange};

/// A scratch project with a real store, removed when the test ends.
struct Project(PathBuf);

impl Project {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "htmlspec-reffixture-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("assets")).expect("the scratch store");
        Self(path)
    }

    /// Stores the shipped revision of `name` at its own address.
    fn store_current(&self, name: &str) -> String {
        let asset = htmlspec_assets::all_assets()
            .into_iter()
            .find(|a| a.name() == name)
            .expect("an embedded asset");
        self.write(asset.path, asset.bytes);
        format!("assets/{}", asset.path)
    }

    /// Stores arbitrary bytes under `name`, at their own address.
    fn store_bytes(&self, name: &str, bytes: &[u8]) -> String {
        let stored = address::stored_path(&sha256_hex(bytes), name);
        self.write(&stored, bytes);
        format!("assets/{stored}")
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.0.join("assets").join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory");
        std::fs::write(&path, bytes).expect("written");
    }

    fn store(&self) -> Store {
        Store::discover(&self.0)
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A canonical document referencing `stylesheet`.
fn document(stylesheet: &str) -> String {
    let source = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\
         <meta charset=\"utf-8\">\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>An archived document</title>\
         <link rel=\"stylesheet\" href=\"{stylesheet}\">\
         </head>\n<body>\n<p>Body.</p>\n</body>\n</html>\n"
    );
    format(&source).expect("the fixture formats")
}

/// Task 5.2: a formatter run cannot silently re-point an archived document.
///
/// The fixture is the real situation. An archived document names the stylesheet revision
/// it was reviewed against, the project has since stored a newer one, and both are
/// present. The reference must survive untouched.
#[test]
fn a_formatter_run_does_not_repoint_an_archived_document_at_newer_bytes() {
    let project = Project::new("archived");
    // The revision the archived document was reviewed against.
    let reviewed = project.store_bytes("htmlspec.css", b"/* the reviewed revision */\n");
    // And the current one, which the project has since adopted.
    let current = project.store_current("htmlspec.css");
    assert_ne!(reviewed, current, "the two revisions must differ");

    let archived = document(&reviewed);
    let outcome = rewrite(&archived, &project.store(), "assets");

    assert_eq!(
        outcome.rewritten, archived,
        "the formatter re-pointed an archived document at newer bytes"
    );
    assert!(
        !outcome.rewritten.contains(&current),
        "the current address leaked into the archived document"
    );
    assert!(outcome.is_unchanged());

    // And the refusal is reported rather than silent, so a contributor expecting a
    // rewrite learns why it did not happen.
    let kept: Vec<&ReferenceChange> = outcome
        .changes
        .iter()
        .filter(|change| {
            matches!(
                change,
                ReferenceChange::Kept {
                    reason: KeptReason::NamesPresentBytes,
                    ..
                }
            )
        })
        .collect();
    assert_eq!(kept.len(), 1, "{:#?}", outcome.changes);
}

/// The migration case: an unaddressed reference is filled.
///
/// The other half of the behaviour. If this failed, the rewrite would be a no-op and the
/// refusal above would be vacuous.
#[test]
fn an_unaddressed_reference_is_filled_from_the_store() {
    let project = Project::new("migrate");
    let current = project.store_current("htmlspec.css");

    let outcome = rewrite(&document("assets/htmlspec.css"), &project.store(), "assets");
    assert!(
        outcome.rewritten.contains(&current),
        "the reference was not filled: {}",
        outcome.rewritten
    );
    assert_eq!(outcome.filled().count(), 1, "{:#?}", outcome.changes);
}

/// Task 5.3: formatting stays idempotent under the rewrite.
///
/// Checked as the composition a caller performs — rewrite, then format — and run twice.
/// Either order must reach a fixed point, because a contributor runs the tool repeatedly.
#[test]
fn rewriting_and_formatting_reach_a_fixed_point() {
    let project = Project::new("fixed-point");
    project.store_current("htmlspec.css");
    let store = project.store();

    let once = format(&rewrite(&document("assets/htmlspec.css"), &store, "assets").rewritten)
        .expect("the first pass formats");
    let twice =
        format(&rewrite(&once, &store, "assets").rewritten).expect("the second pass formats");
    assert_eq!(twice, once, "the composition is not a fixed point");

    // And a third, since a fixed point claimed from two passes is weaker than one
    // claimed from three.
    let thrice =
        format(&rewrite(&twice, &store, "assets").rewritten).expect("the third pass formats");
    assert_eq!(thrice, twice);
}

/// The rewrite is a fixed point on its own, independent of the serializer.
#[test]
fn the_rewrite_alone_is_a_fixed_point() {
    let project = Project::new("rewrite-fixed");
    project.store_current("htmlspec.css");
    let store = project.store();

    let once = rewrite(&document("assets/htmlspec.css"), &store, "assets");
    let twice = rewrite(&once.rewritten, &store, "assets");
    assert_eq!(twice.rewritten, once.rewritten);
    assert!(
        twice.is_unchanged(),
        "a second rewrite changed something: {:#?}",
        twice.changes
    );
}

/// Task 5.4: the rewrite does not widen a diff beyond the reference.
///
/// Asserted on an already-canonical document, because that is the situation the
/// diff-churn requirement is about: a contributor formats a formatted document and
/// reviews the diff.
#[test]
fn rewriting_a_reference_changes_only_that_line() {
    let project = Project::new("no-churn");
    project.store_current("htmlspec.css");

    let before = document("assets/htmlspec.css");
    let after = rewrite(&before, &project.store(), "assets").rewritten;
    assert_ne!(
        after, before,
        "nothing was rewritten, so this proves nothing"
    );

    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    assert_eq!(
        before_lines.len(),
        after_lines.len(),
        "the rewrite changed the line count"
    );

    let differing: Vec<usize> = (0..before_lines.len())
        .filter(|i| before_lines[*i] != after_lines[*i])
        .collect();
    assert_eq!(
        differing.len(),
        1,
        "the rewrite touched {} lines:\n{}",
        differing.len(),
        differing
            .iter()
            .map(|i| format!("- {}\n+ {}", before_lines[*i], after_lines[*i]))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        after_lines[differing[0]].contains("sha256-"),
        "the changed line is not the reference: {}",
        after_lines[differing[0]]
    );
}

/// The rewritten document is still canonical, so a rewrite does not create formatting
/// work.
#[test]
fn a_rewritten_document_is_still_canonical() {
    let project = Project::new("still-canonical");
    project.store_current("htmlspec.css");

    let rewritten = rewrite(&document("assets/htmlspec.css"), &project.store(), "assets").rewritten;
    assert_eq!(
        format(&rewritten).expect("formats"),
        rewritten,
        "the rewrite left the document needing reformatting"
    );
}

/// A filled reference resolves on disk from the document's own directory.
///
/// The regression this exists for: the rewrite once preserved the prefix the old reference
/// carried, which is wrong whenever the document's references are wrong — the case a
/// rewrite exists to repair. The archived document in this project referenced
/// `assets/htmlspec.css` while its real store sat four levels up, and preserving the
/// prefix produced seven addresses that were individually correct and collectively
/// unreachable. The document named the right bytes and still rendered unstyled.
///
/// So the assertion is not "the address is right" but "the browser finds the file".
#[test]
fn a_filled_reference_resolves_on_disk_from_a_nested_document() {
    let project = Project::new("nested-resolves");
    project.store_current("htmlspec.css");

    // A document several levels below the store, referencing the assets as though they
    // sat beside it — which is exactly the archived document's state.
    let nested = project.0.join("changes/archive/one/documents");
    std::fs::create_dir_all(&nested).expect("the nested directory");
    let document_path = nested.join("archived.html");
    std::fs::write(&document_path, document("assets/htmlspec.css")).expect("written");

    let source = std::fs::read_to_string(&document_path).expect("readable");
    let outcome = rewrite(&source, &project.store(), "../../../../assets");
    assert_eq!(outcome.filled().count(), 1, "{:#?}", outcome.changes);

    for reference in outcome
        .rewritten
        .split('"')
        .filter(|part| part.contains("assets/") && part.contains("sha256-"))
    {
        assert!(
            nested.join(reference).exists(),
            "{reference} does not resolve from the document's directory, so the document \
             renders unstyled while naming the correct bytes"
        );
    }
}

/// A document validated with no store beside it is left exactly as it was.
///
/// This is what keeps the conformance fixtures intact: their references are symbolic and
/// a rewrite pass over them must not change a byte.
#[test]
fn a_document_with_no_store_is_untouched() {
    let source = document("assets/htmlspec.css");
    let outcome = rewrite(&source, &Store::Absent, "assets");
    assert_eq!(outcome.rewritten, source);
    assert!(outcome.changes.is_empty());
}

/// A canonical document whose own stylesheet reference is unaddressed, with `body` as its
/// content.
///
/// The `<link>` is the control: it names bytes the store does not hold, so it is filled.
/// Whatever `body` holds is the subject, and the assertion is that it survives.
fn document_with_body(body: &str) -> String {
    let source = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\
         <meta charset=\"utf-8\">\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>A document that writes about markup</title>\
         <link rel=\"stylesheet\" href=\"assets/htmlspec.css\">\
         </head>\n<body>\n{body}\n</body>\n</html>\n"
    );
    format(&source).expect("the fixture formats")
}

/// Rewrites `body`'s document in a project holding the shipped stylesheet, and asserts that
/// the document's own `<link>` was the only thing filled and that `body` survived byte for
/// byte.
fn assert_only_the_link_is_filled(label: &str, body: &str) {
    let project = Project::new(label);
    let current = project.store_current("htmlspec.css");

    let source = document_with_body(body);
    assert!(
        source.contains(body),
        "the fixture is not canonical as written, so it cannot pin the words: {source}"
    );
    let outcome = rewrite(&source, &project.store(), "assets");

    assert!(
        outcome.rewritten.contains(body),
        "the author's words were rewritten:\n{}",
        outcome.rewritten
    );
    assert!(
        outcome.rewritten.contains(&format!("href=\"{current}\"")),
        "the document's own stylesheet reference was not filled, so this proves nothing: {}",
        outcome.rewritten
    );
    assert_eq!(
        outcome.filled().count(),
        1,
        "more than the document's own reference was filled: {:#?}",
        outcome.changes
    );
}

/// Prose that quotes an attribute is content, not a reference.
///
/// The defect this change exists for. The formatter found `href="` and `src="` anywhere in
/// the source, so an author who wrote about a template's markup had their words changed to
/// a hashed path, and `format --check` failed until the words were wrong.
#[test]
fn prose_quoting_an_asset_attribute_is_left_as_written() {
    assert_only_the_link_is_filled(
        "prose-code",
        "<p>A template links its stylesheet with <code>href=\"assets/htmlspec.css\"</code>, \
         and a script with <code>src=\"assets/htmlspec.js\"</code>.</p>",
    );
}

/// An escaped example is text, however much it looks like a tag.
#[test]
fn an_escaped_example_of_a_link_is_left_as_written() {
    assert_only_the_link_is_filled(
        "prose-pre",
        "<pre><code>&lt;link rel=\"stylesheet\" href=\"assets/htmlspec-print.css\"&gt;</code></pre>",
    );
}

/// A hyperlink is a citation, not something the document loads.
///
/// The asset rules never judge an `<a href>`, so a formatter that filled one would be
/// rewriting a value the validator does not consider a reference.
#[test]
fn a_hyperlink_to_an_asset_is_left_as_written() {
    assert_only_the_link_is_filled(
        "prose-anchor",
        "<p>The <a href=\"assets/htmlspec.css\">stylesheet</a> is linked for reading.</p>",
    );
}
