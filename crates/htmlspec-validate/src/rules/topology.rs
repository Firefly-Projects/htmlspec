//! `asset/duplicate`: a project stores each asset's bytes once.
//!
//! This is an asset rule like the two in [`super::assets`], and it is **not** a
//! [`crate::engine::Rule`]. Its subject is a project rather than a document, which the
//! specs state directly: the rule "takes a project as its subject rather than a
//! document, so it SHALL be evaluated where a project is in view".
//!
//! That is a real constraint and not a preference. A project holding one duplicated
//! library would report the same violation once per document if this ran in the
//! per-document pass — twenty documents, twenty identical diagnostics naming the same
//! two paths, none of them the fault of the document being validated. So it is absent
//! from [`super::ALL`] and evaluated once against a store instead. `htmlspec assets`
//! is what evaluates it, and `scripts/check` runs that.
//!
//! The detection itself lives in [`htmlspec_assets::Store::duplicates`], which compares
//! addressed and unaddressed files together. Two addressed copies cannot duplicate each
//! other — identical bytes take one address and land on one path — so a check limited to
//! the addressed layout would report nothing while a project held two whole copies of
//! the runtime.

use htmlspec_assets::Store;

use crate::diagnostic::{Diagnostic, Location, Namespace, RuleId};

/// A project stores one asset's bytes at more than one location.
pub const DUPLICATE: RuleId = RuleId::new(Namespace::Asset, "duplicate");

/// Every duplication violation in `store`.
///
/// Empty for an absent store: a directory with no store has nothing to duplicate, which
/// the specs require explicitly. Also empty for a store whose every asset is distinct,
/// however similar two assets' names or versions are — the rule is about bytes.
///
/// One diagnostic per set of duplicated bytes, naming every location holding them, so a
/// contributor can tell which copy to remove. Reporting only the first would leave them
/// guessing at the rest.
pub fn duplicates(store: &Store) -> Vec<Diagnostic> {
    store
        .duplicates()
        .into_iter()
        .map(|duplicate| {
            Diagnostic::new(
                DUPLICATE,
                Location::default().at_schema_path(format!("/assets/{}", duplicate.sha256)),
                format!(
                    "these {} locations hold identical bytes ({}), so the project stores one \
                     asset more than once: {}. A shared runtime is what lets many documents \
                     render from one copy, and a second copy is the arrangement that let a \
                     document drift from the bytes beside it. Remove all but one",
                    duplicate.locations.len(),
                    &duplicate.sha256[..16],
                    duplicate.locations.join(", ")
                ),
            )
        })
        .collect()
}

/// Every duplication violation *between* stores.
///
/// [`duplicates`] answers the question within one store and cannot answer it across
/// several: two addressed copies in two different stores each sit alone at their own
/// address, so each store reports itself clean while the project holds the bytes twice.
/// That is not a hypothetical arrangement. It is what a project looked like when every
/// change established its own store, measured at 6 MB per change.
///
/// Takes `(location, digest)` pairs already gathered from every store, so the walk stays
/// in the caller and this stays a pure function over what the walk found.
pub fn duplicates_across(located: &[(String, String)]) -> Vec<Diagnostic> {
    let mut by_digest: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    for (location, digest) in located {
        by_digest
            .entry(digest.as_str())
            .or_default()
            .push(location.as_str());
    }

    by_digest
        .into_iter()
        .filter(|(_, locations)| locations.len() > 1)
        .map(|(sha256, mut locations)| {
            locations.sort_unstable();
            Diagnostic::new(
                DUPLICATE,
                Location::default().at_schema_path(format!("/assets/{sha256}")),
                format!(
                    "these {} locations hold identical bytes ({}), in {} separate store(s): {}. \
                     A project stores one copy of its runtime and every document references it, \
                     so a second store is bytes the project pays for twice. Establish one store \
                     at the project root and point the documents at it",
                    locations.len(),
                    &sha256[..16.min(sha256.len())],
                    locations.len(),
                    locations.join(", ")
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use htmlspec_assets::{address, sha256_hex};
    use std::path::PathBuf;

    /// A scratch project, removed when the test ends.
    struct Project(PathBuf);

    impl Project {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-topology-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(path.join("assets")).expect("the scratch store is created");
            Self(path)
        }

        /// Stores `bytes` at their own address, as the project layout does.
        fn store(&self, name: &str, bytes: &[u8]) -> String {
            let stored = address::stored_path(&sha256_hex(bytes), name);
            let path = self.0.join("assets").join(&stored);
            std::fs::create_dir_all(path.parent().expect("an address directory"))
                .expect("the address exists");
            std::fs::write(&path, bytes).expect("the asset is written");
            stored
        }

        /// Writes `bytes` at a path outside the addressing scheme, which is where real
        /// duplication lives.
        fn store_unaddressed(&self, relative: &str, bytes: &[u8]) {
            let path = self.0.join("assets").join(relative);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory");
            std::fs::write(&path, bytes).expect("the copy is written");
        }

        fn store_of(&self) -> Store {
            Store::discover(&self.0)
        }
    }

    impl Drop for Project {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn messages(diagnostics: &[Diagnostic]) -> String {
        diagnostics
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Task 4.1: the duplicate is reported and every location is named.
    #[test]
    fn a_duplicated_library_is_reported_with_every_location() {
        let project = Project::new("duplicate");
        let bytes = b"the same library";
        let addressed = project.store("mermaid.min.js", bytes);
        project.store_unaddressed("vendor/mermaid.min.js", bytes);

        let found = duplicates(&project.store_of());
        assert_eq!(found.len(), 1, "{}", messages(&found));
        assert!(found[0].is_error(), "duplication must fail the project");
        assert!(
            found[0].message.contains(&addressed),
            "the report omits the addressed copy: {}",
            found[0].message
        );
        assert!(
            found[0].message.contains("vendor/mermaid.min.js"),
            "the report omits the unaddressed copy: {}",
            found[0].message
        );
    }

    /// The copy-per-document arrangement, which is what the shared-runtime obligation
    /// exists to reject.
    #[test]
    fn a_copy_per_document_is_reported() {
        let project = Project::new("copy-per-document");
        let bytes = b"the shared runtime";
        project.store("htmlspec.js", bytes);
        project.store_unaddressed("one/htmlspec.js", bytes);
        project.store_unaddressed("two/htmlspec.js", bytes);

        let found = duplicates(&project.store_of());
        assert_eq!(found.len(), 1, "{}", messages(&found));
        // One diagnostic, three locations: the fault is one asset stored thrice.
        assert!(
            found[0].message.contains("3 locations"),
            "the report does not count the copies: {}",
            found[0].message
        );
    }

    /// Task 4.2, first half: distinct bytes are not a duplicate, however similar.
    #[test]
    fn two_revisions_of_one_file_are_not_duplicates() {
        let project = Project::new("revisions");
        let new = project.store("htmlspec.css", b"--hs-measure: 36rem;");
        let old = project.store("htmlspec.css", b"--hs-measure: 34rem;");
        assert_ne!(new, old, "differing bytes take differing addresses");

        let found = duplicates(&project.store_of());
        assert!(
            found.is_empty(),
            "distinct bytes were reported as duplicates: {}",
            messages(&found)
        );
    }

    /// Task 4.2, second half: a directory with no store has nothing to duplicate.
    #[test]
    fn a_directory_with_no_store_reports_nothing() {
        assert!(duplicates(&Store::Absent).is_empty());

        // And a project whose directory simply has no `assets/` in it.
        let empty = std::env::temp_dir().join(format!("htmlspec-nostore-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).expect("the directory");
        assert!(duplicates(&Store::discover(&empty)).is_empty());
        let _ = std::fs::remove_dir_all(&empty);
    }

    /// One store serving many documents is the arrangement the project wants, so it must
    /// report nothing.
    #[test]
    fn one_store_serving_many_documents_reports_nothing() {
        let project = Project::new("shared");
        project.store("htmlspec.css", b"the stylesheet");
        project.store("htmlspec.js", b"the script");
        project.store("mermaid.min.js", b"the diagram library");

        let found = duplicates(&project.store_of());
        assert!(found.is_empty(), "{}", messages(&found));
    }

    /// The rule cannot be downgraded, which it inherits from its namespace rather than
    /// restating.
    #[test]
    fn duplication_is_a_non_downgradable_error() {
        assert_eq!(DUPLICATE.namespace(), Namespace::Asset);
        assert!(!Namespace::Asset.is_downgradable());
    }
}
