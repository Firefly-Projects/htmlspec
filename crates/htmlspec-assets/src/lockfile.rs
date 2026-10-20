//! The asset lockfile: what was vendored, from where, and with what contents.
//!
//! The lockfile is the answer to the obvious objection against committing
//! third-party build output. It records enough to re-derive every committed byte
//! from its published source, so the committed copy is auditable rather than
//! trusted.
//!
//! The types below deliberately reject an incomplete entry at parse time: a
//! library with no hash, or a file with no source member, would be a dependency
//! nothing pins, and a lockfile that admitted one would be worse than no lockfile
//! because it would look like a guarantee.

use serde::Deserialize;

/// The parsed `assets/vendor.lock.json`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Lockfile {
    /// Prose explaining the file's purpose, for a reader who opens it directly.
    #[serde(default)]
    pub comment: Vec<String>,
    /// This project's own assets, which have no upstream to re-fetch.
    ///
    /// Recorded because they need addresses by the same rule as the vendored ones,
    /// and because an unpinned stylesheet is how a stale copy shipped undetected.
    #[serde(default)]
    pub own: Vec<OwnFile>,
    /// Earlier revisions of this project's own assets, still committed at their addresses.
    ///
    /// A document keeps the revision it was authored against, so a revision this repository's
    /// documents name stays where it is when a newer one replaces it. Recorded because committed
    /// bytes nothing pins are how a stale stylesheet shipped undetected once, and kept apart from
    /// [`Self::own`] because a retained revision is neither embedded nor stored by `init`.
    #[serde(default)]
    pub retained: Vec<RetainedFile>,
    /// Every vendored library.
    pub libraries: Vec<Library>,
}

/// A superseded revision of one of this project's own assets.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct RetainedFile {
    /// The own asset's file name, which this is an earlier revision of.
    pub path: String,
    /// The revision's SHA-256, as lowercase hex, which is also its address.
    pub sha256: String,
    /// Why the revision is kept.
    #[serde(default)]
    pub note: String,
}

/// One of this project's own assets.
///
/// No archive and no archive member: there is no upstream to re-extract from, so the
/// repository's history is its provenance. The hash still pins the bytes, which is
/// what gives the asset an address.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct OwnFile {
    /// The file's name within `assets/`.
    pub path: String,
    /// The file's SHA-256, as lowercase hex.
    pub sha256: String,
    /// Whether the file is stored at a content address in a project.
    ///
    /// False for the template, which is the document `init` produces rather than an
    /// asset copied beside it. Defaults to true, because every other own asset is
    /// part of the shared runtime.
    #[serde(default = "default_true")]
    pub stored: bool,
    /// Anything a reader needs to know about this specific file.
    #[serde(default)]
    pub note: String,
}

fn default_true() -> bool {
    true
}

/// One vendored library.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Library {
    /// The library's published name.
    pub name: String,
    /// The exact version vendored.
    pub version: String,
    /// Why this project vendors it, so a reader can judge whether it is still
    /// needed rather than inferring that from its presence.
    #[serde(default)]
    pub purpose: String,
    /// The published archive the bytes came from.
    pub archive: Archive,
    /// Each file extracted from the archive and committed.
    pub files: Vec<VendoredFile>,
}

/// The published archive a library was vendored from.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Archive {
    /// The URL the archive was fetched from.
    pub url: String,
    /// The archive's SHA-256, as lowercase hex.
    pub sha256: String,
}

/// One file extracted from an archive and committed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct VendoredFile {
    /// The path inside the project's `assets/` directory.
    pub path: String,
    /// The path inside the published archive the file was extracted from.
    ///
    /// Recorded so the Nix check can re-extract exactly this file rather than
    /// guessing which of an archive's builds was committed.
    pub archive_member: String,
    /// The file's SHA-256, as lowercase hex.
    pub sha256: String,
    /// Anything a reader needs to know about this specific file.
    #[serde(default)]
    pub note: String,
}

/// The lockfile text, embedded so the check needs no source checkout.
pub const TEXT: &str = include_str!("../assets/vendor.lock.json");

/// The parsed lockfile.
///
/// Parsed once. A malformed lockfile is a build-time defect rather than a runtime
/// condition — it is committed alongside the assets it pins — so this panics with
/// the parse error rather than returning a `Result` every caller would unwrap.
pub fn lockfile() -> &'static Lockfile {
    static PARSED: std::sync::OnceLock<Lockfile> = std::sync::OnceLock::new();
    PARSED.get_or_init(|| {
        serde_json::from_str(TEXT).unwrap_or_else(|e| {
            panic!("assets/vendor.lock.json is malformed, so no asset is pinned: {e}")
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_committed_lockfile_parses() {
        let lockfile = lockfile();
        assert!(!lockfile.libraries.is_empty());
        assert!(
            !lockfile.comment.is_empty(),
            "the lockfile should explain itself to a reader who opens it directly"
        );
    }

    /// An entry missing a hash must not parse. A lockfile that admitted one would
    /// look like a guarantee while pinning nothing.
    #[test]
    fn a_library_without_a_hash_is_rejected() {
        let text = r#"{
            "libraries": [
                {
                    "name": "x",
                    "version": "1.0.0",
                    "archive": { "url": "https://example.test/x-1.0.0.tgz" },
                    "files": []
                }
            ]
        }"#;
        assert!(serde_json::from_str::<Lockfile>(text).is_err());
    }

    #[test]
    fn a_file_without_its_archive_member_is_rejected() {
        let text = r#"{
            "libraries": [
                {
                    "name": "x",
                    "version": "1.0.0",
                    "archive": { "url": "https://example.test/x.tgz", "sha256": "00" },
                    "files": [ { "path": "vendor/x.js", "sha256": "00" } ]
                }
            ]
        }"#;
        assert!(
            serde_json::from_str::<Lockfile>(text).is_err(),
            "without the archive member, a re-fetch cannot know which file was committed"
        );
    }

    #[test]
    fn a_complete_entry_parses() {
        let text = r#"{
            "libraries": [
                {
                    "name": "x",
                    "version": "1.0.0",
                    "archive": { "url": "https://example.test/x.tgz", "sha256": "aa" },
                    "files": [
                        {
                            "path": "vendor/x.js",
                            "archive_member": "package/dist/x.js",
                            "sha256": "bb"
                        }
                    ]
                }
            ]
        }"#;
        let parsed: Lockfile = serde_json::from_str(text).expect("a complete entry parses");
        assert_eq!(
            parsed.libraries[0].files[0].archive_member,
            "package/dist/x.js"
        );
        // The optional fields default rather than failing.
        assert!(parsed.libraries[0].purpose.is_empty());
        assert!(parsed.libraries[0].files[0].note.is_empty());
    }
}
