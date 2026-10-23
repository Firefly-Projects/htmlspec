//! Rewriting a document's asset references to the addresses of the bytes it renders with.
//!
//! Two sources of truth, and keeping them apart is the whole design of this module.
//!
//! **What to rewrite *to*** comes from the embedded asset table — the runtime this
//! toolchain ships, keyed by file name. That is what `init` writes and what a document
//! renders against, so it is the only defensible target for a fill.
//!
//! **Whether to rewrite at all** comes from the store beside the document. A reference
//! is filled only when the bytes it names are *absent*. A reference that resolves to
//! present bytes is left exactly as it was.
//!
//! That asymmetry is the CAUTION in the design, and it is not a detail. A formatter that
//! rewrote every reference to the current address would silently re-point an archived
//! document at newer bytes — quietly changing what a reviewed artifact renders with,
//! which is a worse defect than the stale reference this change exists to catch. An
//! archived page names the revision it was reviewed against, and that reference is
//! correct. Re-pointing it is an explicit operation, never a side effect of tidying.
//!
//! **Which values are references** is the asset rules' answer, not this module's. A
//! reference is the value of one of [`htmlspec_assets::REFERENCING_ATTRIBUTES`] on an element
//! the parser builds, and the asset rules read the same list. Text never is one: an author who
//! quotes `href="assets/htmlspec.css"` in a `<code>` element, or escapes a `<link>` in an
//! example, has written words, and deterministic-formatting's "Formatting preserves meaning"
//! keeps them. This module once searched the source for `href="` and `src="`, and filled
//! those words, and hyperlinks the asset rules deliberately never judge.
//!
//! The rewrite is textual, on the attribute value alone. A tree-level rewrite would
//! reserialize the document and risk reflowing markup the edit did not touch, which the
//! diff-churn requirement forbids: formatting "SHALL NOT widen a diff beyond the units a
//! content edit touched". So the values are found in the source, by reading its tags as the
//! tokenizer does (the private `tags` module), and a test in the asset rules holds what is
//! found here to what they find in the tree.
//!
//! [`crate::format`] does not call this. Formatting stays a pure function of its source,
//! so its idempotence and its property tests do not acquire a filesystem dependency. The
//! caller composes the two, and [`rewrite`] is itself idempotent — once a reference names
//! present bytes, the absent-only trigger makes a second pass a no-op.

use std::ops::Range;

use htmlspec_assets::{ReferencingAttribute, Resolution, Store};

use crate::tags;

/// What happened to one asset reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceChange {
    /// The reference named bytes the store does not hold, and was rewritten.
    Filled {
        /// The reference as the document carried it.
        was: String,
        /// The reference as it now reads.
        now: String,
        /// The asset's file name, which is what the two forms share.
        name: String,
    },
    /// The reference names bytes the store holds, so it was left alone.
    ///
    /// Reported rather than silently skipped: a contributor who expected a rewrite is
    /// owed the reason it did not happen, and that reason is that the document is
    /// already correct about what it renders with.
    Kept {
        /// The reference, unchanged.
        reference: String,
        /// Why it was kept.
        reason: KeptReason,
    },
}

/// Why a reference was left as it was.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeptReason {
    /// The store holds bytes at the address and their hash matches it.
    ///
    /// The document is right about its own provenance. Re-pointing it at a newer
    /// revision is a decision for a person.
    NamesPresentBytes,
    /// The store holds bytes at the address whose hash disagrees with it.
    ///
    /// Not filled, because filling would erase the evidence. The address records what
    /// the document was authored against, and something replaced the bytes underneath
    /// it — a fault for `asset/mismatch` to report, not for the formatter to paper over.
    NamesReplacedBytes,
    /// The reference names a file the embedded runtime does not have.
    ///
    /// Nothing to fill it with. A project asset outside the shipped runtime, or a typo,
    /// and either way the formatter must not invent an address.
    UnknownAsset,
    /// The reference's path does not reach the store it was resolved against.
    ///
    /// Not filled, because the address may be right and the path wrong, and re-pointing a
    /// reference is an explicit operation rather than a side effect of tidying. Archival is the
    /// operation that moves a document, and it re-expresses the document's references itself.
    LeadsElsewhere,
}

/// The result of rewriting one document's references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewriteOutcome {
    /// The document, with absent references filled.
    pub rewritten: String,
    /// Every reference considered, in document order.
    pub changes: Vec<ReferenceChange>,
}

impl RewriteOutcome {
    /// Whether anything was rewritten.
    pub fn is_unchanged(&self) -> bool {
        !self
            .changes
            .iter()
            .any(|change| matches!(change, ReferenceChange::Filled { .. }))
    }

    /// Every reference that was filled.
    pub fn filled(&self) -> impl Iterator<Item = &ReferenceChange> {
        self.changes
            .iter()
            .filter(|change| matches!(change, ReferenceChange::Filled { .. }))
    }
}

/// Fills every asset reference whose bytes the store does not hold.
///
/// `prefix` is the path from the document to the store's `assets/` directory, without a
/// trailing slash — `"assets"` for a document beside it, `"../../assets"` for one two
/// levels below. A filled reference is built from this rather than from the prefix the
/// old reference carried, because the two are not the same thing and conflating them
/// writes a reference that does not resolve.
///
/// That is not hypothetical. The archived document in this project referenced
/// `assets/htmlspec.css` while its own `assets/` directory was a duplicate copy; once
/// that copy was removed, the store it belongs to sits four levels up. Preserving the
/// old prefix produced seven addresses that were individually correct and collectively
/// unreachable — a document that names the right bytes and still renders unstyled, which
/// is the same failure this change exists to remove wearing a different hat.
///
/// Returns the source unchanged when there is no store: with no bytes beside the
/// document there is nothing to be absent from, and rewriting against an absent store
/// would rewrite a conformance fixture whose exact bytes are what it pins.
pub fn rewrite(source: &str, store: &Store, prefix: &str) -> RewriteOutcome {
    if !store.is_present() {
        return RewriteOutcome {
            rewritten: source.to_owned(),
            changes: Vec::new(),
        };
    }
    let prefix = prefix.trim_end_matches('/');

    let mut out = String::with_capacity(source.len());
    let mut changes = Vec::new();
    let mut copied = 0;

    // Walk the asset references. Textual, so everything outside each value is copied
    // through byte for byte.
    for found in asset_references(source) {
        out.push_str(source.get(copied..found.span.start).unwrap_or_default());
        let reference = found.value;

        match decide(reference, store, prefix) {
            Decision::Fill { replacement, name } => {
                out.push_str(&replacement);
                changes.push(ReferenceChange::Filled {
                    was: reference.to_owned(),
                    now: replacement,
                    name,
                });
            }
            Decision::Keep(reason) => {
                out.push_str(reference);
                changes.push(ReferenceChange::Kept {
                    reference: reference.to_owned(),
                    reason,
                });
            }
            Decision::NotAnAsset => out.push_str(reference),
        }

        copied = found.span.end;
    }
    out.push_str(source.get(copied..).unwrap_or_default());

    RewriteOutcome {
        rewritten: out,
        changes,
    }
}

/// One reference [`upgrade`] moved onto the store's current revision of its asset, or would have.
///
/// `#[non_exhaustive]`, as [`UpgradeOutcome`] is: this crate is published, and a field added in a
/// later minor release must not break a caller that built or destructured one.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct UpgradedReference {
    /// The reference as the document carried it.
    pub was: String,
    /// The reference as it now reads.
    pub now: String,
    /// The asset's file name, which is what the two forms share.
    pub name: String,
}

/// The result of upgrading one document's references.
///
/// `#[non_exhaustive]` because this crate is published on crates.io and a later 1.x release may
/// report more about an upgrade: a field added to an exhaustive struct breaks every caller that
/// destructures it.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct UpgradeOutcome {
    /// The document, with every upgradable reference naming the current revision.
    pub rewritten: String,
    /// Every reference that moved, in document order.
    pub upgraded: Vec<UpgradedReference>,
    /// Every reference left as written because the store holds other bytes at the current
    /// revision's address, in document order. `now` is that address, and the reference was not
    /// moved there: it would name bytes the address does not, which `asset/mismatch` reports.
    pub blocked: Vec<UpgradedReference>,
}

/// Rewrites every asset reference to the current revision of the asset it names, on request.
///
/// **The explicit operation the module's CAUTION reserves.** [`rewrite`] fills only references whose
/// bytes are absent, because re-pointing a reference at newer bytes changes what a document renders
/// with and must never be a side effect of tidying. This is that re-pointing, asked for by name:
/// deterministic-formatting's "Formatting preserves meaning" makes it the third exception, made only
/// on a contributor's request, beside filling an absent reference and adding the fallback
/// stylesheet. The caller decides which documents it may touch: never an archived change's, and
/// never a run's record.
///
/// Each reference whose file name is an asset this binary embeds becomes `<prefix>/<address>/<name>`
/// at the embedded revision's address, so a document moves onto the stylesheet and the scripts the
/// toolchain writes into every new document. Nothing is added: a script the document does not load
/// stays unloaded. Three kinds of reference are left as written:
///
/// - one naming a file the embedded runtime does not have, which has no current revision;
/// - one whose stored bytes disagree with its address, because moving it would erase the evidence
///   that something replaced them, which `asset/mismatch` exists to report;
/// - anything that is not an asset reference at all.
///
/// Nothing is fetched or downloaded. The current revision is the one compiled into this binary, and
/// the caller writes its bytes into the store before the rewritten document is read. With no store
/// beside the document there is no provenance to move, so the source is returned unchanged, as
/// [`rewrite`] returns it.
pub fn upgrade(source: &str, store: &Store, prefix: &str) -> UpgradeOutcome {
    if !store.is_present() {
        return UpgradeOutcome {
            rewritten: source.to_owned(),
            upgraded: Vec::new(),
            blocked: Vec::new(),
        };
    }
    let prefix = prefix.trim_end_matches('/');

    let mut out = String::with_capacity(source.len());
    let mut upgraded = Vec::new();
    let mut blocked = Vec::new();
    let mut copied = 0;
    for found in asset_references(source) {
        out.push_str(source.get(copied..found.span.start).unwrap_or_default());
        let reference = found.value;
        match current_revision(reference, store, prefix) {
            Some((replacement, name)) if replacement != reference => {
                let moved = UpgradedReference {
                    was: reference.to_owned(),
                    now: replacement,
                    name,
                };
                // The current address holding other bytes is a fault in the store, and a reference
                // moved onto it would name bytes it does not render with. Left where it is, and
                // reported, so the fault is found rather than inherited.
                if matches!(store.resolve(&moved.now), Resolution::Mismatch { .. }) {
                    out.push_str(reference);
                    blocked.push(moved);
                } else {
                    out.push_str(&moved.now);
                    upgraded.push(moved);
                }
            }
            _ => out.push_str(reference),
        }
        copied = found.span.end;
    }
    out.push_str(source.get(copied..).unwrap_or_default());

    UpgradeOutcome {
        rewritten: out,
        upgraded,
        blocked,
    }
}

/// The current revision's reference for `reference`, with the asset's name, or `None` where
/// [`upgrade`] leaves it as written.
fn current_revision(reference: &str, store: &Store, prefix: &str) -> Option<(String, String)> {
    if !reference.contains("assets/") {
        return None;
    }
    if matches!(store.resolve(reference), Resolution::Mismatch { .. }) {
        return None;
    }
    let name = file_name_of(reference)?;
    let asset = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == name)?;
    Some((format!("{prefix}/{}", asset.path), name.to_owned()))
}

/// What to do with one reference.
enum Decision {
    Fill { replacement: String, name: String },
    Keep(KeptReason),
    NotAnAsset,
}

fn decide(reference: &str, store: &Store, prefix: &str) -> Decision {
    // A fragment, an absolute URL, or anything not pointing into an assets directory is
    // not this function's business.
    if !reference.contains("assets/") {
        return Decision::NotAnAsset;
    }

    match store.resolve(reference) {
        // Already names bytes that are present: the document is correct, and
        // re-pointing it is an explicit act.
        Resolution::Match => Decision::Keep(KeptReason::NamesPresentBytes),
        // Bytes are there and disagree with the address. Filling would destroy the
        // evidence that something was replaced.
        Resolution::Mismatch { .. } => Decision::Keep(KeptReason::NamesReplacedBytes),
        // Only a store seen from a document answers this, and the path, not the bytes, is what
        // is wrong. The formatter does not move a reference.
        Resolution::Elsewhere { .. } => Decision::Keep(KeptReason::LeadsElsewhere),
        // Addressed and absent, or unaddressed. Both are fillable, and both are filled
        // from the embedded runtime by file name.
        Resolution::Absent | Resolution::NotAddressed => fill(reference, prefix),
    }
}

/// Rewrites one reference to the current address of the asset it names.
///
/// The replacement is `<prefix>/<address>/<name>`, built from the store's location rather
/// than from whatever the old reference pointed at.
fn fill(reference: &str, prefix: &str) -> Decision {
    let Some(name) = file_name_of(reference) else {
        return Decision::NotAnAsset;
    };

    let Some(asset) = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == name)
    else {
        return Decision::Keep(KeptReason::UnknownAsset);
    };

    let replacement = format!("{prefix}/{}", asset.path);
    if replacement == reference {
        // Already the current address at the right prefix, and the store simply does not
        // hold it. Reporting a fill that changed nothing would be a lie.
        return Decision::Keep(KeptReason::NamesPresentBytes);
    }
    Decision::Fill {
        replacement,
        name: name.to_owned(),
    }
}

/// The file name a reference names, which is what survives a rewrite.
///
/// `../assets/sha256-abc/htmlspec.css` and `assets/htmlspec.css` both yield
/// `htmlspec.css`. The prefix and any address are discarded: the caller supplies the
/// prefix and the asset table supplies the address.
fn file_name_of(reference: &str) -> Option<&str> {
    let name = reference.rsplit('/').next()?;
    if name.is_empty() {
        return None;
    }
    Some(name)
}

/// An asset reference as a document's source spells it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReference<'s> {
    /// The element and attribute it is the value of.
    pub on: ReferencingAttribute,
    /// The value, trimmed of surrounding whitespace, as the source spells it.
    pub value: &'s str,
    /// Where `value` sits in the source.
    span: Range<usize>,
}

/// Every asset reference in `source`, in document order.
///
/// A reference is the value of one of [`htmlspec_assets::REFERENCING_ATTRIBUTES`] on a start
/// tag the parser would build into the tree, trimmed, when anything is left after trimming.
/// That is what the asset rules read, in the terms they report it, so a value is considered
/// here exactly when the validator considers it.
///
/// The value is the source's spelling. The tree decodes character references, and this does
/// not, because an asset path holds none and decoding is the parser's work.
pub fn asset_references(source: &str) -> Vec<SourceReference<'_>> {
    tags::start_tags(source)
        .into_iter()
        .flat_map(|tags::StartTag { name, attributes }| {
            attributes.into_iter().filter_map(move |attribute| {
                let on = ReferencingAttribute::find(&name, &attribute.name)?;
                let written = source.get(attribute.value.clone())?;
                let value = written.trim();
                if value.is_empty() {
                    return None;
                }
                let start = attribute.value.start + (written.len() - written.trim_start().len());
                Some(SourceReference {
                    on,
                    value,
                    span: start..start + value.len(),
                })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use htmlspec_assets::{address, sha256_hex};
    use std::path::PathBuf;

    /// A scratch project with a real store.
    struct Project(PathBuf);

    impl Project {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "htmlspec-refrewrite-{label}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(path.join("assets")).expect("the scratch store");
            Self(path)
        }

        /// Stores the embedded asset named `name` at its own address, which is the state
        /// a project is in after `init`.
        fn store_current(&self, name: &str) -> String {
            let asset = htmlspec_assets::all_assets()
                .into_iter()
                .find(|a| a.name() == name)
                .expect("an embedded asset");
            let path = self.0.join("assets").join(asset.path);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the address");
            std::fs::write(&path, asset.bytes).expect("the asset is written");
            format!("assets/{}", asset.path)
        }

        /// Stores arbitrary bytes at their own address under `name`.
        fn store_bytes(&self, name: &str, bytes: &[u8]) -> String {
            let stored = address::stored_path(&sha256_hex(bytes), name);
            let path = self.0.join("assets").join(&stored);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the address");
            std::fs::write(&path, bytes).expect("written");
            format!("assets/{stored}")
        }

        /// Writes bytes at the address of other bytes, which is a mismatch.
        fn store_wrong(&self, name: &str, addressed: &[u8], actual: &[u8]) -> String {
            let stored = address::stored_path(&sha256_hex(addressed), name);
            let path = self.0.join("assets").join(&stored);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the address");
            std::fs::write(&path, actual).expect("written");
            format!("assets/{stored}")
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

    fn document(reference: &str) -> String {
        format!("<link rel=\"stylesheet\" href=\"{reference}\">\n")
    }

    /// Task 5.1, first half: an unaddressed reference is filled, which is the migration.
    #[test]
    fn an_unaddressed_reference_is_filled_with_the_current_address() {
        let project = Project::new("fill-unaddressed");
        let current = project.store_current("htmlspec.css");

        let outcome = rewrite(&document("assets/htmlspec.css"), &project.store(), "assets");
        assert!(
            outcome.rewritten.contains(&current),
            "{}",
            outcome.rewritten
        );
        assert_eq!(outcome.filled().count(), 1, "{:#?}", outcome.changes);
        assert!(!outcome.is_unchanged());
    }

    /// A reference to an address the store does not hold is filled.
    #[test]
    fn a_reference_to_absent_bytes_is_filled() {
        let project = Project::new("fill-absent");
        let current = project.store_current("htmlspec.css");
        let stale = format!(
            "assets/{}",
            address::stored_path(&sha256_hex(b"a revision never stored"), "htmlspec.css")
        );

        let outcome = rewrite(&document(&stale), &project.store(), "assets");
        assert!(
            outcome.rewritten.contains(&current),
            "{}",
            outcome.rewritten
        );
        assert!(
            !outcome.rewritten.contains(&stale),
            "the stale reference survived: {}",
            outcome.rewritten
        );
    }

    /// Task 5.1, load-bearing half: a reference naming present bytes is NOT rewritten.
    ///
    /// This is the CAUTION in the design. An archived document names the revision it was
    /// reviewed against, and a formatter that re-pointed it at newer bytes would change
    /// what a reviewed artifact renders with, silently.
    #[test]
    fn a_reference_naming_present_bytes_is_reported_and_not_rewritten() {
        let project = Project::new("keep-present");
        // The project holds both an older revision and the current one.
        let older = project.store_bytes("htmlspec.css", b"/* the reviewed revision */");
        project.store_current("htmlspec.css");

        let source = document(&older);
        let outcome = rewrite(&source, &project.store(), "assets");
        assert_eq!(
            outcome.rewritten, source,
            "an archived document was silently re-pointed at newer bytes"
        );
        assert!(outcome.is_unchanged());
        assert_eq!(
            outcome.changes,
            vec![ReferenceChange::Kept {
                reference: older,
                reason: KeptReason::NamesPresentBytes,
            }],
            "the reference was skipped without reporting why"
        );
    }

    /// A mismatch is kept, not filled: filling would erase the evidence.
    #[test]
    fn a_reference_to_replaced_bytes_is_kept_so_the_fault_stays_visible() {
        let project = Project::new("keep-mismatch");
        let replaced = project.store_wrong("htmlspec.css", b"the addressed bytes", b"other bytes");

        let source = document(&replaced);
        let outcome = rewrite(&source, &project.store(), "assets");
        assert_eq!(outcome.rewritten, source);
        assert!(matches!(
            outcome.changes.as_slice(),
            [ReferenceChange::Kept {
                reason: KeptReason::NamesReplacedBytes,
                ..
            }]
        ));
    }

    /// A reference to something outside the shipped runtime is not invented.
    #[test]
    fn a_reference_to_an_unknown_asset_is_kept() {
        let project = Project::new("unknown");
        project.store_current("htmlspec.css");

        let source = document("assets/a-projects-own-file.css");
        let outcome = rewrite(&source, &project.store(), "assets");
        assert_eq!(outcome.rewritten, source);
        assert!(matches!(
            outcome.changes.as_slice(),
            [ReferenceChange::Kept {
                reason: KeptReason::UnknownAsset,
                ..
            }]
        ));
    }

    /// With no store there is nothing to be absent from, so nothing is rewritten. This
    /// is what keeps a conformance fixture's bytes intact.
    #[test]
    fn no_store_means_no_rewrite() {
        let source = document("assets/htmlspec.css");
        let outcome = rewrite(&source, &Store::Absent, "assets");
        assert_eq!(outcome.rewritten, source);
        assert!(outcome.changes.is_empty());
        assert!(outcome.is_unchanged());
    }

    /// Task 5.3: the rewrite is idempotent. A second pass changes nothing.
    #[test]
    fn a_second_pass_is_byte_identical() {
        let project = Project::new("idempotent");
        project.store_current("htmlspec.css");

        let once = rewrite(&document("assets/htmlspec.css"), &project.store(), "assets");
        let twice = rewrite(&once.rewritten, &project.store(), "assets");
        assert_eq!(twice.rewritten, once.rewritten);
        assert!(
            twice.is_unchanged(),
            "the second pass rewrote something: {:#?}",
            twice.changes
        );
    }

    /// Task 5.4: only the reference changes. Nothing around it moves.
    #[test]
    fn the_rewrite_touches_nothing_but_the_reference() {
        let project = Project::new("diff-width");
        project.store_current("htmlspec.css");

        // Deliberately awkward surroundings: odd indentation, a comment, preserved
        // content, and an attribute order the canonical form would change. None of it
        // may move, because this function is not the serializer.
        let source = "<!doctype html>\n<html lang=\"en\">\n  <head>\n\
             \x20   <!-- a comment that must survive -->\n\
             \x20       <link href=\"assets/htmlspec.css\" rel=\"stylesheet\">\n\
             \x20 </head>\n  <body>\n<pre>  preserved   spacing  </pre>\n\
             <p>Text with <a href=\"#anchor\">a link</a>.</p>\n</body>\n</html>\n";

        let outcome = rewrite(source, &project.store(), "assets");
        assert!(!outcome.is_unchanged(), "nothing was rewritten");

        // Every line except the one holding the reference is byte-identical.
        let before: Vec<&str> = source.lines().collect();
        let after: Vec<&str> = outcome.rewritten.lines().collect();
        assert_eq!(before.len(), after.len(), "the line count changed");
        let differing: Vec<usize> = (0..before.len())
            .filter(|i| before[*i] != after[*i])
            .collect();
        assert_eq!(
            differing.len(),
            1,
            "the rewrite changed {} lines, not 1:\n{}",
            differing.len(),
            differing
                .iter()
                .map(|i| format!("  {}\n  {}", before[*i], after[*i]))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(before[differing[0]].contains("htmlspec.css"));
        // And the fragment link is untouched.
        assert!(outcome.rewritten.contains("href=\"#anchor\""));
        assert!(outcome.rewritten.contains("preserved   spacing"));
    }

    /// Every reference in a document is considered, not only the first.
    #[test]
    fn every_reference_in_a_document_is_rewritten() {
        let project = Project::new("many");
        for name in ["htmlspec.css", "htmlspec-print.css", "htmlspec.js"] {
            project.store_current(name);
        }

        let source = "<link href=\"assets/htmlspec.css\" rel=\"stylesheet\">\n\
             <link href=\"assets/htmlspec-print.css\" rel=\"stylesheet\" media=\"print\">\n\
             <script src=\"assets/htmlspec.js\"></script>\n";
        let outcome = rewrite(source, &project.store(), "assets");
        assert_eq!(outcome.filled().count(), 3, "{:#?}", outcome.changes);
        assert!(!outcome.rewritten.contains("assets/htmlspec.css\""));
    }

    /// Rewrites `source` in a project holding the shipped runtime, whose references are
    /// therefore fillable from it.
    fn in_a_project(label: &str, source: &str) -> RewriteOutcome {
        let project = Project::new(label);
        for name in ["htmlspec.css", "htmlspec-print.css", "htmlspec.js"] {
            project.store_current(name);
        }
        rewrite(source, &project.store(), "assets")
    }

    /// Asserts that nothing in `source` was considered a reference.
    fn assert_untouched(label: &str, source: &str) {
        let outcome = in_a_project(label, source);
        assert_eq!(
            outcome.rewritten, source,
            "a value that is not a reference was rewritten"
        );
        assert!(
            outcome.changes.is_empty(),
            "a value that is not a reference was reported: {:#?}",
            outcome.changes
        );
    }

    /// Asserts that `source` holds exactly one reference, to `name`, and that it was filled
    /// with the quoting it was written in.
    fn assert_filled(label: &str, source: &str, name: &str) {
        let outcome = in_a_project(label, source);
        let current = htmlspec_assets::all_assets()
            .into_iter()
            .find(|asset| asset.name() == name)
            .expect("an embedded asset")
            .path;
        assert!(
            matches!(
                outcome.changes.as_slice(),
                [ReferenceChange::Filled { name: filled, .. }] if filled == name
            ),
            "expected one fill of {name}: {:#?}",
            outcome.changes
        );
        assert!(
            outcome.rewritten.contains(&format!("assets/{current}")),
            "{}",
            outcome.rewritten
        );
        assert_eq!(
            outcome.rewritten.len() - source.len(),
            current.len() - name.len(),
            "more than the reference's value changed: {}",
            outcome.rewritten
        );
    }

    /// A comment is text to the parser, whatever it holds.
    #[test]
    fn a_reference_inside_a_comment_is_not_a_reference() {
        assert_untouched(
            "comment",
            "<!-- <link rel=\"stylesheet\" href=\"assets/htmlspec.css\"> -->\n",
        );
    }

    /// The contents of a raw-text element are text up to its end tag, and so is a
    /// `<textarea>`'s.
    #[test]
    fn a_reference_inside_a_raw_text_element_is_not_a_reference() {
        assert_untouched(
            "script-contents",
            "<script>const s = '<img src=\"assets/htmlspec.js\">';</script>\n",
        );
        assert_untouched(
            "style-contents",
            "<style>/* <link href=\"assets/htmlspec.css\"> */</style>\n",
        );
        assert_untouched(
            "textarea-contents",
            "<textarea><link href=\"assets/htmlspec.css\"></textarea>\n",
        );
    }

    /// A template's contents are a separate fragment, which the tree the asset rules walk
    /// does not hold.
    #[test]
    fn a_reference_inside_a_template_is_not_a_reference() {
        assert_untouched(
            "template",
            "<template><link rel=\"stylesheet\" href=\"assets/htmlspec.css\"></template>\n",
        );
    }

    /// An attribute is matched by its whole name, not by a suffix of it.
    #[test]
    fn a_data_attribute_ending_in_src_is_not_a_reference() {
        assert_untouched(
            "data-src",
            "<img data-src=\"assets/htmlspec.js\" alt=\"\">\n",
        );
    }

    /// Another attribute's value is text, even when it quotes a reference.
    #[test]
    fn an_attribute_value_quoting_a_reference_is_not_a_reference() {
        assert_untouched(
            "quoted-in-title",
            "<abbr title='written as href=\"assets/htmlspec.css\"'>a link</abbr>\n",
        );
    }

    /// `<object data>` loads a file, and the asset rules judge it.
    #[test]
    fn an_object_data_attribute_is_filled() {
        assert_filled(
            "object-data",
            "<object data=\"assets/htmlspec.js\"></object>\n",
            "htmlspec.js",
        );
    }

    /// The parser reads a value in either quote, or none.
    #[test]
    fn a_value_is_filled_in_any_quoting() {
        assert_filled(
            "single-quoted",
            "<link rel='stylesheet' href='assets/htmlspec.css'>\n",
            "htmlspec.css",
        );
        assert_filled(
            "unquoted",
            "<link rel=stylesheet href=assets/htmlspec.css>\n",
            "htmlspec.css",
        );
    }

    /// The parser lowercases names, so an uppercase attribute is the same attribute.
    #[test]
    fn an_uppercase_attribute_is_filled() {
        assert_filled(
            "uppercase",
            "<SCRIPT SRC=\"assets/htmlspec.js\"></SCRIPT>\n",
            "htmlspec.js",
        );
    }

    /// The parser keeps the first of a repeated attribute and drops the rest, so only the
    /// first is a reference.
    #[test]
    fn a_repeated_attribute_fills_only_its_first_value() {
        let source = "<link href=\"assets/htmlspec.css\" href=\"assets/htmlspec-print.css\" \
                      rel=\"stylesheet\">\n";
        assert_filled("repeated", source, "htmlspec.css");
        assert!(in_a_project("repeated-second", source)
            .rewritten
            .contains("href=\"assets/htmlspec-print.css\""));
    }

    /// The asset rules trim a value before resolving it, so the fill replaces what is between
    /// the spaces and keeps the spaces.
    #[test]
    fn a_padded_value_is_filled_inside_its_padding() {
        let source = "<script src=\"  assets/htmlspec.js \"></script>\n";
        assert_filled("padded", source, "htmlspec.js");
        let rewritten = in_a_project("padded-kept", source).rewritten;
        assert!(
            rewritten.starts_with("<script src=\"  assets/sha256-")
                && rewritten.ends_with("/htmlspec.js \"></script>\n"),
            "{rewritten}"
        );
    }

    /// A `>` inside a quoted value does not end the tag. A substring search was right about
    /// this by accident; a scanner that ended the tag at the `>` would read the reference as
    /// text.
    #[test]
    fn a_closing_bracket_inside_a_quoted_value_does_not_end_the_tag() {
        assert_filled(
            "bracket-in-value",
            "<link title=\"a > b\" href=\"assets/htmlspec.css\" rel=\"stylesheet\">\n",
            "htmlspec.css",
        );
    }

    /// A nested document's relative prefix is preserved.
    #[test]
    fn a_relative_prefix_survives_the_rewrite() {
        let project = Project::new("nested");
        project.store_current("htmlspec.css");

        let outcome = rewrite(
            &document("../../assets/htmlspec.css"),
            &project.store(),
            "../../assets",
        );
        assert!(
            outcome.rewritten.contains("../../assets/sha256-"),
            "the relative prefix was lost: {}",
            outcome.rewritten
        );
    }

    /// `upgrade` moves a reference to an older stored revision onto the current one, which
    /// `rewrite` leaves as written: the third exception to "Formatting preserves meaning", made only
    /// on request.
    #[test]
    fn an_upgrade_moves_a_reference_to_an_older_revision_onto_the_current_one() {
        let project = Project::new("upgrade-older");
        let older = project.store_bytes("htmlspec.css", b"/* an older revision */");
        let current = project.store_current("htmlspec.css");

        let source = document(&older);
        assert_eq!(
            rewrite(&source, &project.store(), "assets").rewritten,
            source,
            "without the request, the older revision must be left as written"
        );

        let outcome = upgrade(&source, &project.store(), "assets");
        assert_eq!(outcome.rewritten, document(&current));
        assert_eq!(
            outcome.upgraded,
            vec![UpgradedReference {
                was: older,
                now: current,
                name: "htmlspec.css".into(),
            }]
        );
    }

    /// A second upgrade changes nothing, so the option is idempotent.
    #[test]
    fn an_upgrade_is_idempotent() {
        let project = Project::new("upgrade-twice");
        let older = project.store_bytes("htmlspec.css", b"/* an older revision */");
        project.store_current("htmlspec.css");

        let once = upgrade(&document(&older), &project.store(), "assets");
        let twice = upgrade(&once.rewritten, &project.store(), "assets");
        assert_eq!(twice.rewritten, once.rewritten);
        assert!(twice.upgraded.is_empty(), "{:#?}", twice.upgraded);
    }

    /// An upgrade adds nothing the document does not load, and leaves what it cannot place alone: an
    /// unknown file, a reference whose stored bytes were replaced, and text outside any reference.
    #[test]
    fn an_upgrade_adds_nothing_and_keeps_what_it_cannot_place() {
        let project = Project::new("upgrade-keeps");
        project.store_current("htmlspec.css");
        let replaced = project.store_wrong("htmlspec-print.css", b"addressed", b"actual");

        let source = format!(
            "{}{}<script src=\"assets/sha256-0000000000000000/not-shipped.js\"></script>\n\
             <p><code>assets/sha256-0000000000000000/htmlspec.css</code></p>\n",
            document("assets/sha256-0000000000000000/htmlspec.css"),
            document(&replaced),
        );
        let outcome = upgrade(&source, &project.store(), "assets");

        assert_eq!(outcome.upgraded.len(), 1, "{:#?}", outcome.upgraded);
        assert_eq!(outcome.upgraded[0].name, "htmlspec.css");
        assert!(
            outcome.rewritten.contains(&replaced),
            "the replaced bytes' evidence moved"
        );
        assert!(outcome.rewritten.contains("not-shipped.js"));
        assert!(
            outcome
                .rewritten
                .contains("<code>assets/sha256-0000000000000000/htmlspec.css</code>"),
            "text quoting a reference was rewritten"
        );
        assert!(
            !outcome.rewritten.contains("mermaid") && !outcome.rewritten.contains("vega"),
            "an upgrade added an asset the document does not load"
        );
    }

    /// A clean reference is not moved onto a current address whose stored bytes were replaced: it is
    /// left as written and reported as blocked, so the store's fault stays the store's.
    #[test]
    fn an_upgrade_does_not_move_a_reference_onto_replaced_bytes() {
        let project = Project::new("upgrade-onto-replaced");
        let older = project.store_bytes("htmlspec.css", b"/* an older revision */");
        let current = htmlspec_assets::all_assets()
            .into_iter()
            .find(|asset| asset.name() == "htmlspec.css")
            .expect("the stylesheet");
        let replaced = project.store_wrong("htmlspec.css", current.bytes, b"/* replaced */");

        let source = document(&older);
        let outcome = upgrade(&source, &project.store(), "assets");
        assert_eq!(
            outcome.rewritten, source,
            "the reference moved onto replaced bytes"
        );
        assert!(outcome.upgraded.is_empty(), "{:#?}", outcome.upgraded);
        assert_eq!(
            outcome.blocked,
            vec![UpgradedReference {
                was: older,
                now: replaced,
                name: "htmlspec.css".into(),
            }]
        );
    }

    /// With no store beside the document there is no provenance to move.
    #[test]
    fn no_store_means_no_upgrade() {
        let source = document("assets/sha256-0000000000000000/htmlspec.css");
        let outcome = upgrade(&source, &Store::Absent, "assets");
        assert_eq!(outcome.rewritten, source);
        assert!(outcome.upgraded.is_empty());
    }
}
