//! A project's asset store: what bytes are present, and at which addresses.
//!
//! The store is the subject of every asset rule. A document's reference names an
//! address, and the store either holds matching bytes at it or does not — so the
//! rules read this rather than the filesystem directly, and a rule cannot accidentally
//! depend on a path that only exists on one machine.
//!
//! **A missing store is not an empty store.** A conformance fixture references
//! `assets/htmlspec.css` and no such file has ever sat beside it: the reference is
//! symbolic, present so the parser sees a realistic document. Resolving references
//! against a store that is absent would reject every fixture in the suite, and those
//! inputs are the pin for every rule this project has. So [`Store::discover`]
//! distinguishes the two cases, and the rules fire only on [`Store::Present`].
//!
//! The scope is the store's presence and nothing else. A document cannot declare
//! itself exempt — a rule a document can switch off protects nothing, and the rule
//! exists because a real archived page shipped against a stylesheet it did not match.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use crate::{address, sha256_hex};

/// The directory a project keeps its assets in, relative to the project root.
pub const DIRECTORY: &str = "assets";

/// Whether a project has an asset store, and what it holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Store {
    /// No `assets/` directory, so there is nothing to resolve against.
    ///
    /// Distinct from a present-but-empty store: a document here is not misreporting
    /// its provenance, because no provenance is claimed.
    Absent,
    /// A store, with every addressed asset it holds.
    Present(Contents),
}

/// The addressed assets a store holds.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Contents {
    /// The store's root directory.
    pub root: PathBuf,
    /// Every addressed asset, keyed by its stored path (`sha256-<16 hex>/<name>`).
    entries: BTreeMap<String, Entry>,
    /// Every file under the store root that is *not* at an address.
    ///
    /// Kept because the duplication this rule exists to catch is unaddressed by
    /// nature: an addressed copy cannot duplicate another addressed copy, since
    /// identical bytes take the same address and land on the same path. The real
    /// duplication is a copy sitting outside the scheme — a legacy `vendor/` tree, or
    /// a second `assets/` directory beside a document.
    unaddressed: BTreeMap<String, Entry>,
    /// Where the store is seen from, where it is seen from one document.
    ///
    /// `None` for a store seen from nowhere, which resolves a reference by its address alone. The
    /// formatter's store is one, and so is any store a caller has not placed a document against.
    seen_from: Option<Viewpoint>,
}

/// A document's directory and its store's directory, each absolute, with `.` and `..` resolved
/// without reading the filesystem.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Viewpoint {
    /// The document's directory, as its components from the root.
    document: Vec<String>,
    /// The store's directory, the same way.
    store: Vec<String>,
}

impl Viewpoint {
    /// The directory `written` names from the document, as a browser resolves the directory part of
    /// a relative URL, or `None` where it begins with a scheme and names no path from the document.
    ///
    /// `..` at the root stays at the root, and a path from the root starts there.
    fn reached_by(&self, written: &str) -> Option<Vec<String>> {
        if has_scheme(written) {
            return None;
        }
        let mut components = if written.starts_with('/') {
            Vec::new()
        } else {
            self.document.clone()
        };
        for segment in written.split('/') {
            match segment {
                "" | "." => {}
                ".." => {
                    components.pop();
                }
                name => components.push(name.to_owned()),
            }
        }
        Some(components)
    }

    /// The path from the document's directory to the store's, as a reference would spell it.
    fn store_from_document(&self) -> String {
        let shared = self
            .document
            .iter()
            .zip(&self.store)
            .take_while(|(one, other)| one == other)
            .count();
        let mut path = "../".repeat(self.document.len() - shared);
        for directory in self.store.iter().skip(shared) {
            path.push_str(directory);
            path.push('/');
        }
        if path.is_empty() {
            path.push_str("./");
        }
        path
    }
}

/// `path` as its components from the root, made absolute against the working directory where it is
/// relative, and with `.` and `..` resolved lexically.
fn absolute_components(path: &Path) -> Vec<String> {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_owned());
    let mut components = Vec::new();
    for component in absolute.components() {
        match component {
            Component::Normal(name) => components.push(name.to_string_lossy().into_owned()),
            Component::ParentDir => {
                components.pop();
            }
            Component::CurDir | Component::RootDir => {}
            Component::Prefix(prefix) => {
                components.push(prefix.as_os_str().to_string_lossy().into_owned());
            }
        }
    }
    components
}

/// Whether a path begins with a URL scheme, such as `https:`.
fn has_scheme(path: &str) -> bool {
    let Some((scheme, _)) = path.split_once(':') else {
        return false;
    };
    let mut characters = scheme.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters.all(|rest| rest.is_ascii_alphanumeric() || matches!(rest, '+' | '-' | '.'))
}

/// One stored asset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    /// The full lowercase-hex digest of the bytes actually present.
    pub sha256: String,
    /// How many bytes are present, for a report that names a size.
    pub len: usize,
}

/// Two locations hold identical bytes, so the project stores one asset twice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Duplicate {
    /// The shared digest.
    pub sha256: String,
    /// Every stored path holding these bytes, sorted.
    pub locations: Vec<String>,
}

/// What a reference resolves to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    /// The store holds bytes at this address whose digest matches it.
    Match,
    /// Nothing is stored at the address.
    Absent,
    /// Bytes are stored at the address and their digest disagrees with it.
    ///
    /// A different fault from [`Self::Absent`], with a different cause: absent means
    /// the bytes were never written, and a mismatch means something replaced them.
    Mismatch {
        /// The digest the address names, truncated as addresses are.
        addressed: String,
        /// The digest the stored bytes actually produce.
        found: String,
    },
    /// The reference names an address, and its path leads to a directory that is not the store.
    ///
    /// Only a store seen from a document ([`Store::seen_from`]) answers this, because only it
    /// knows where a document's path leads. Distinct from [`Self::Absent`] because the address may
    /// well be stored: what is wrong is the path in front of it, which a browser follows to another
    /// directory. Both are unresolved, and the repair differs.
    Elsewhere {
        /// The directory the reference's path names, as the reference writes it, ending in `/`.
        leads_to: String,
        /// The path from the document to the store, as a reference would write it.
        store: String,
    },
    /// The reference does not name an address, so this rule has nothing to say.
    ///
    /// A reference to a document, an anchor, or an unaddressed path. Reported as its
    /// own case rather than as a violation, because not every reference is an asset.
    NotAddressed,
}

impl Store {
    /// Finds the store for the project rooted at `root`.
    ///
    /// Reads the `assets/` directory one level deep for hash-named directories, which
    /// is the layout the addressing scheme produces. An unreadable directory yields
    /// [`Store::Absent`] rather than an error: the caller is a validation rule, and a
    /// rule that failed the run because a directory was unreadable would report a
    /// filesystem problem as a contract violation.
    pub fn discover(root: &Path) -> Self {
        let directory = root.join(DIRECTORY);
        if !directory.is_dir() {
            return Self::Absent;
        }

        let mut contents = Contents {
            root: directory.clone(),
            entries: BTreeMap::new(),
            unaddressed: BTreeMap::new(),
            seen_from: None,
        };

        let Ok(addresses) = std::fs::read_dir(&directory) else {
            return Self::Absent;
        };
        for address_entry in addresses.flatten() {
            let address_path = address_entry.path();
            let address_name = address_entry.file_name().to_string_lossy().into_owned();

            if address_path.is_dir() && is_address(&address_name) {
                let Ok(files) = std::fs::read_dir(&address_path) else {
                    continue;
                };
                for file in files.flatten() {
                    let path = file.path();
                    if !path.is_file() {
                        continue;
                    }
                    let Ok(bytes) = std::fs::read(&path) else {
                        continue;
                    };
                    let name = file.file_name().to_string_lossy().into_owned();
                    contents.entries.insert(
                        format!("{address_name}/{name}"),
                        Entry {
                            sha256: sha256_hex(&bytes),
                            len: bytes.len(),
                        },
                    );
                }
                continue;
            }

            // Everything else under the store root, recorded so duplication can be
            // seen. A legacy `vendor/` tree lives here.
            collect_unaddressed(&address_path, &directory, &mut contents.unaddressed);
        }

        Self::Present(contents)
    }

    /// This store as the document in `directory` sees it.
    ///
    /// A reference then resolves only where its path, followed from `directory` as a browser follows
    /// a relative URL, names the store's own directory, and [`Self::resolve`] answers
    /// [`Resolution::Elsewhere`] for one that names an address through a path leading anywhere else.
    /// So `../../../htmlspec/assets/`, which climbs out of the project's `htmlspec/` and back in,
    /// reaches the store from `htmlspec/changes/<name>/` as `../../assets/` does.
    ///
    /// **Lexical, and never the filesystem's answer.** Both directories are made absolute against
    /// the working directory where they are relative, as the command line's paths are, and their
    /// `.` and `..` are resolved without reading anything. A reference through a link to the store,
    /// or spelled in another case on a filesystem that ignores case, loads the store's bytes on one
    /// machine and nothing on another, and a verdict that followed the filesystem would pass a
    /// document on one and fail it on the other.
    pub fn seen_from(self, directory: &Path) -> Self {
        match self {
            Self::Absent => Self::Absent,
            Self::Present(contents) => {
                let seen_from = Viewpoint {
                    document: absolute_components(directory),
                    store: absolute_components(&contents.root),
                };
                Self::Present(Contents {
                    seen_from: Some(seen_from),
                    ..contents
                })
            }
        }
    }

    /// Whether asset rules apply here.
    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }

    /// Resolves one asset reference.
    ///
    /// `reference` is the path a document names, relative to the document. Only its
    /// last two components matter, because a stored asset is an address and a file
    /// name however deep the document sits.
    pub fn resolve(&self, reference: &str) -> Resolution {
        let Self::Present(contents) = self else {
            // No store, no relationship to violate. The caller checks `is_present`
            // first; this keeps the answer safe if one forgets.
            return Resolution::NotAddressed;
        };

        let Some(stored) = addressed_suffix(reference) else {
            return Resolution::NotAddressed;
        };
        // Seen from a document, the path in front of the address has to reach the store, because
        // that path is what a browser follows. Seen from nowhere, only the address is asked about.
        if let Some(view) = &contents.seen_from {
            let written = directory_of_address(reference);
            if view.reached_by(written).as_ref() != Some(&view.store) {
                return Resolution::Elsewhere {
                    leads_to: if written.is_empty() {
                        String::from("./")
                    } else {
                        format!("{written}/")
                    },
                    store: view.store_from_document(),
                };
            }
        }
        let addressed = stored
            .split('/')
            .next()
            .unwrap_or_default()
            .trim_start_matches(&format!("{}-", address::ALGORITHM))
            .to_owned();

        match contents.entries.get(&stored) {
            None => Resolution::Absent,
            Some(entry) => {
                if entry.sha256.starts_with(&addressed) {
                    Resolution::Match
                } else {
                    Resolution::Mismatch {
                        addressed,
                        found: entry.sha256.clone(),
                    }
                }
            }
        }
    }

    /// Every set of locations holding identical bytes.
    ///
    /// Empty for an absent store: a directory with no store has nothing to duplicate.
    ///
    /// Addressed and unaddressed files are compared together, which is the point. Two
    /// addressed copies cannot duplicate each other — identical bytes take one address
    /// and one path — so a comparison limited to the addressed layout would report
    /// nothing, ever, while the project held two full copies of the runtime. The
    /// duplication that exists is always a file outside the scheme.
    pub fn duplicates(&self) -> Vec<Duplicate> {
        let Self::Present(contents) = self else {
            return Vec::new();
        };

        let mut by_digest: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for (path, entry) in contents.entries.iter().chain(&contents.unaddressed) {
            by_digest
                .entry(entry.sha256.as_str())
                .or_default()
                .push(path.clone());
        }

        by_digest
            .into_iter()
            .filter(|(_, locations)| locations.len() > 1)
            .map(|(sha256, mut locations)| {
                locations.sort();
                Duplicate {
                    sha256: sha256.to_owned(),
                    locations,
                }
            })
            .collect()
    }

    /// Every stored file as `(location, digest)`, addressed and unaddressed alike.
    ///
    /// Exposed so a caller holding *several* stores can compare across them.
    /// [`Self::duplicates`] answers the question within one store, and a project that
    /// grew a second store — the arrangement that gave each change its own copy of the
    /// vendored runtime — has its duplication *between* stores, where no single store
    /// can see it. The location is prefixed with the store's own root so a report names
    /// a path a contributor can act on rather than a suffix repeated in both stores.
    pub fn located_digests(&self) -> Vec<(String, String)> {
        let Self::Present(contents) = self else {
            return Vec::new();
        };
        contents
            .entries
            .iter()
            .chain(&contents.unaddressed)
            .map(|(path, entry)| {
                (
                    contents.root.join(path).display().to_string(),
                    entry.sha256.clone(),
                )
            })
            .collect()
    }

    /// How many assets the store holds.
    pub fn len(&self) -> usize {
        match self {
            Self::Absent => 0,
            Self::Present(contents) => contents.entries.len(),
        }
    }

    /// Whether the store holds nothing.
    ///
    /// True for an absent store and for a present-but-empty one, so callers that only
    /// care about content need not match. Use [`Self::is_present`] to tell them apart.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Records every file beneath `path` that is not at an address.
///
/// Keys are relative to the store root, so a report names `vendor/mermaid.min.js`
/// rather than an absolute path that differs per machine.
fn collect_unaddressed(path: &Path, root: &Path, into: &mut BTreeMap<String, Entry>) {
    if path.is_file() {
        let Ok(bytes) = std::fs::read(path) else {
            return;
        };
        let key = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        into.insert(
            key,
            Entry {
                sha256: sha256_hex(&bytes),
                len: bytes.len(),
            },
        );
        return;
    }
    if !path.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_unaddressed(&entry.path(), root, into);
    }
}

/// Whether a directory name is an address.
fn is_address(name: &str) -> bool {
    let prefix = format!("{}-", address::ALGORITHM);
    let Some(digest) = name.strip_prefix(&prefix) else {
        return false;
    };
    digest.len() == address::ADDRESS_HEX_LEN && digest.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The path in front of an addressed reference's `<address>/<file>`, which is where it says the
/// store is.
///
/// Empty for a reference beside its store's addresses. The query, the fragment, and a trailing slash
/// are left out, as [`addressed_suffix`] leaves them out.
fn directory_of_address(reference: &str) -> &str {
    let reference = reference
        .split(['?', '#'])
        .next()
        .unwrap_or(reference)
        .trim_end_matches('/');
    let mut parts = reference.rsplitn(3, '/');
    let _name = parts.next();
    let _address = parts.next();
    parts.next().unwrap_or_default()
}

/// The `<address>/<file>` suffix of a reference, when it has one.
///
/// A document several directories deep references `../../assets/sha256-…/x.js`, and
/// the store is keyed by the address and the name alone.
fn addressed_suffix(reference: &str) -> Option<String> {
    // Strip a query or fragment: a stylesheet reference may carry neither, but a
    // document is written by hand and a rule should not be defeated by `?v=2`.
    let reference = reference
        .split(['?', '#'])
        .next()
        .unwrap_or(reference)
        .trim_end_matches('/');

    let mut components = reference.rsplit('/');
    let name = components.next()?;
    let address = components.next()?;
    if name.is_empty() || !is_address(address) {
        return None;
    }
    Some(format!("{address}/{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch project, removed when the test ends.
    struct Project(PathBuf);

    impl Project {
        fn new(label: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("htmlspec-store-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch project is created");
            Self(path)
        }

        /// Stores `bytes` at their own address, as the project layout does.
        fn store(&self, name: &str, bytes: &[u8]) -> String {
            let stored = address::stored_path(&sha256_hex(bytes), name);
            let path = self.0.join(DIRECTORY).join(&stored);
            std::fs::create_dir_all(path.parent().expect("an address directory"))
                .expect("the address exists");
            std::fs::write(&path, bytes).expect("the asset is written");
            stored
        }

        /// Writes bytes at an address that does not describe them, which is the
        /// corruption a mismatch reports.
        fn store_wrong(&self, name: &str, addressed: &[u8], actual: &[u8]) -> String {
            let stored = address::stored_path(&sha256_hex(addressed), name);
            let path = self.0.join(DIRECTORY).join(&stored);
            std::fs::create_dir_all(path.parent().expect("an address directory"))
                .expect("the address exists");
            std::fs::write(&path, actual).expect("the asset is written");
            stored
        }
    }

    impl Drop for Project {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The distinction the whole scope rests on: no `assets/` directory means no
    /// store, which is not the same as a store holding nothing.
    #[test]
    fn a_project_without_an_assets_directory_has_no_store() {
        let project = Project::new("absent");
        let store = Store::discover(&project.0);
        assert_eq!(store, Store::Absent);
        assert!(!store.is_present(), "asset rules must not fire here");
        assert!(store.is_empty());
    }

    /// A fixture directory is exactly this case, and rejecting it would reject the
    /// suite that pins every other rule.
    #[test]
    fn a_reference_in_a_project_without_a_store_is_not_a_violation() {
        let project = Project::new("fixture-like");
        let store = Store::discover(&project.0);
        assert!(!store.is_present());
        assert_eq!(
            store.resolve("assets/sha256-0123456789abcdef/htmlspec.css"),
            Resolution::NotAddressed,
            "with no store there is no relationship to violate"
        );
        assert!(store.duplicates().is_empty());
    }

    #[test]
    fn an_empty_store_is_present_and_distinguishable_from_an_absent_one() {
        let project = Project::new("empty");
        std::fs::create_dir_all(project.0.join(DIRECTORY)).expect("an empty store");
        let store = Store::discover(&project.0);
        assert!(
            store.is_present(),
            "the directory exists, so the rules apply"
        );
        assert!(store.is_empty());
    }

    #[test]
    fn a_reference_to_stored_bytes_matching_its_address_resolves() {
        let project = Project::new("match");
        let stored = project.store("htmlspec.css", b"--hs-measure: 36rem;");
        let store = Store::discover(&project.0);
        assert_eq!(store.len(), 1);
        assert_eq!(
            store.resolve(&format!("assets/{stored}")),
            Resolution::Match
        );
        // The same asset, referenced from a document several levels down.
        assert_eq!(
            store.resolve(&format!("../../assets/{stored}")),
            Resolution::Match,
            "depth is the document's business, not the store's"
        );
    }

    #[test]
    fn a_reference_to_bytes_the_store_does_not_hold_is_absent() {
        let project = Project::new("missing");
        project.store("htmlspec.css", b"one revision");
        let store = Store::discover(&project.0);
        let elsewhere = address::stored_path(&sha256_hex(b"another revision"), "htmlspec.css");
        assert_eq!(
            store.resolve(&format!("assets/{elsewhere}")),
            Resolution::Absent
        );
    }

    /// The two faults are distinguishable, which the specs require because their
    /// causes differ: absent bytes were never written, and a mismatch was replaced.
    #[test]
    fn stored_bytes_disagreeing_with_their_address_are_a_mismatch_not_an_absence() {
        let project = Project::new("mismatch");
        let stored = project.store_wrong("htmlspec.css", b"the addressed bytes", b"other bytes");
        let store = Store::discover(&project.0);
        match store.resolve(&format!("assets/{stored}")) {
            Resolution::Mismatch { addressed, found } => {
                assert!(
                    found.starts_with(&sha256_hex(b"other bytes")[..address::ADDRESS_HEX_LEN]),
                    "the report names the digest actually present"
                );
                assert!(
                    !found.starts_with(&addressed),
                    "and it differs from the address"
                );
            }
            other => panic!("expected a mismatch, got {other:?}"),
        }
    }

    /// The defect that started this change, reproduced through the store: a document
    /// authored against one stylesheet revision, stored beside another.
    #[test]
    fn the_stale_stylesheet_defect_is_caught() {
        let project = Project::new("stale");
        // The project stores the current stylesheet, with the skip-link rule.
        project.store("htmlspec.css", b".hs-skip-link { left: -9999px }");
        let store = Store::discover(&project.0);

        // The archived document names the older revision, which has no such rule.
        let older = address::stored_path(&sha256_hex(b"/* no skip link rule */"), "htmlspec.css");
        assert_eq!(
            store.resolve(&format!("assets/{older}")),
            Resolution::Absent,
            "a document naming bytes the project does not store must not pass"
        );
    }

    /// Content addressing makes coexistence the default rather than a special case.
    #[test]
    fn two_revisions_of_one_file_coexist_and_are_not_duplicates() {
        let project = Project::new("revisions");
        let new = project.store("htmlspec.css", b"--hs-measure: 36rem;");
        let old = project.store("htmlspec.css", b"--hs-measure: 34rem;");
        assert_ne!(new, old, "differing bytes take differing addresses");

        let store = Store::discover(&project.0);
        assert_eq!(store.len(), 2);
        assert_eq!(store.resolve(&format!("assets/{new}")), Resolution::Match);
        assert_eq!(store.resolve(&format!("assets/{old}")), Resolution::Match);
        assert!(
            store.duplicates().is_empty(),
            "distinct bytes are not a duplicate, however similar the names"
        );
    }

    #[test]
    fn identical_bytes_under_two_names_are_reported_as_a_duplicate() {
        let project = Project::new("duplicate");
        let bytes = b"the same library";
        project.store("mermaid.min.js", bytes);
        // The same bytes stored again under another name: the copy-per-document
        // arrangement, which is what the duplication rule exists to reject.
        let digest = sha256_hex(bytes);
        let second = format!("{}/copy.js", address::address_dir(&digest));
        let path = project.0.join(DIRECTORY).join(&second);
        std::fs::write(&path, bytes).expect("the second copy is written");

        let store = Store::discover(&project.0);
        let duplicates = store.duplicates();
        assert_eq!(duplicates.len(), 1, "{duplicates:#?}");
        assert_eq!(duplicates[0].sha256, digest);
        assert_eq!(
            duplicates[0].locations.len(),
            2,
            "the report names every location, so a contributor can tell which to remove"
        );
    }

    /// The duplication this project actually has, and the reason `duplicates` compares
    /// unaddressed files too.
    ///
    /// Two addressed copies cannot duplicate each other, because identical bytes take
    /// one address and land on one path. A check limited to the addressed layout would
    /// therefore report nothing while the project held two whole copies of the
    /// runtime — passing for the exact arrangement it exists to reject.
    #[test]
    fn an_unaddressed_copy_of_a_stored_asset_is_a_duplicate() {
        let project = Project::new("legacy-layout");
        let bytes = b"the vendored library";
        let stored = project.store("mermaid.min.js", bytes);

        // The legacy layout: the same bytes at a path outside the scheme.
        let legacy = project.0.join(DIRECTORY).join("vendor");
        std::fs::create_dir_all(&legacy).expect("the legacy directory");
        std::fs::write(legacy.join("mermaid.min.js"), bytes).expect("the legacy copy");

        let store = Store::discover(&project.0);
        let duplicates = store.duplicates();
        assert_eq!(duplicates.len(), 1, "{duplicates:#?}");
        assert_eq!(duplicates[0].locations.len(), 2);
        assert!(
            duplicates[0].locations.contains(&stored),
            "the report names the addressed copy: {:?}",
            duplicates[0].locations
        );
        assert!(
            duplicates[0]
                .locations
                .iter()
                .any(|l| l == "vendor/mermaid.min.js"),
            "and the unaddressed one, by a path that is the same on every machine: {:?}",
            duplicates[0].locations
        );
    }

    /// An unaddressed file that duplicates nothing is not reported. The rule is about
    /// duplication, not about tidiness.
    #[test]
    fn an_unaddressed_file_that_duplicates_nothing_is_not_reported() {
        let project = Project::new("unique-stray");
        project.store("htmlspec.css", b"stored bytes");
        let stray = project.0.join(DIRECTORY).join("notes.txt");
        std::fs::write(stray, b"nothing else has these bytes").expect("written");

        let store = Store::discover(&project.0);
        assert!(store.duplicates().is_empty());
    }

    /// A reference that is not an address is not this rule's business.
    #[test]
    fn an_unaddressed_reference_is_not_resolved() {
        let project = Project::new("unaddressed");
        project.store("htmlspec.css", b"anything");
        let store = Store::discover(&project.0);
        for reference in [
            "#req-theme-selection",
            "assets/htmlspec.css",
            "assets/vendor/mermaid.min.js",
            "other.html",
            "",
        ] {
            assert_eq!(
                store.resolve(reference),
                Resolution::NotAddressed,
                "for {reference:?}"
            );
        }
    }

    /// A directory under `assets/` that is not an address is not part of the store.
    #[test]
    fn a_directory_that_is_not_an_address_is_not_part_of_the_store() {
        let project = Project::new("stray");
        let stray = project.0.join(DIRECTORY).join("vendor");
        std::fs::create_dir_all(&stray).expect("a stray directory");
        std::fs::write(stray.join("mermaid.min.js"), b"legacy layout").expect("written");

        let store = Store::discover(&project.0);
        assert!(store.is_present(), "the assets directory exists");
        assert_eq!(
            store.len(),
            0,
            "only addressed directories are stored assets"
        );
    }

    /// A query string must not defeat the rule.
    #[test]
    fn a_reference_carrying_a_query_still_resolves() {
        let project = Project::new("query");
        let stored = project.store("htmlspec.css", b"bytes");
        let store = Store::discover(&project.0);
        assert_eq!(
            store.resolve(&format!("assets/{stored}?v=2")),
            Resolution::Match
        );
    }

    /// Seen from a document, a reference resolves only through a path that reaches the store.
    ///
    /// The store seen from nowhere answers by address, as above, and that is what let an archived
    /// change's `../../assets/` pass when its store was three levels up.
    #[test]
    fn seen_from_a_document_a_path_that_misses_the_store_leads_elsewhere() {
        let project = Project::new("seen");
        let stored = project.store("htmlspec.css", b"bytes");
        let store = Store::discover(&project.0).seen_from(&project.0.join("a/b/c"));
        assert_eq!(
            store.resolve(&format!("../../assets/{stored}")),
            Resolution::Elsewhere {
                leads_to: String::from("../../assets/"),
                store: String::from("../../../assets/"),
            }
        );
        assert_eq!(
            store.resolve(&format!("../../../assets/{stored}?v=2#x")),
            Resolution::Match
        );
        // A path that reaches the store still answers for the bytes it names.
        assert_eq!(
            store.resolve(&format!(
                "../../../assets/{}",
                address::stored_path(&sha256_hex(b"never stored"), "htmlspec.css")
            )),
            Resolution::Absent
        );
        // A path that names no address is still not this store's business.
        assert_eq!(
            store.resolve("../../proposal.html"),
            Resolution::NotAddressed
        );
    }

    /// A path from the root of the filesystem reaches the store only where it names the store's
    /// directory, and one with a scheme names no path from the document at all.
    #[test]
    fn seen_from_a_document_an_absolute_path_leads_where_it_names() {
        let project = Project::new("seen-absolute");
        let stored = project.store("htmlspec.css", b"bytes");
        let store = Store::discover(&project.0).seen_from(&project.0);
        for reference in [
            format!("/assets/{stored}"),
            format!("https://example.com/assets/{stored}"),
        ] {
            assert!(
                matches!(store.resolve(&reference), Resolution::Elsewhere { .. }),
                "{reference} resolved through a path that does not reach the store"
            );
        }
        let root = std::path::absolute(project.0.join("assets")).expect("absolute");
        assert_eq!(
            store.resolve(&format!("{}/{stored}", root.display())),
            Resolution::Match,
            "a path from the root that names the store's directory reaches it"
        );
    }

    /// A path that climbs out of the directories above the document and back in by their names
    /// lands where it names, as a browser follows it.
    ///
    /// The code review of pull request #58 found these reported as leading elsewhere, because the
    /// path was compared with the store's as text.
    #[test]
    fn seen_from_a_document_a_path_out_and_back_in_reaches_the_store() {
        let project = Project::new("seen-climbs");
        let stored = project.store("htmlspec.css", b"bytes");
        let store = Store::discover(&project.0).seen_from(&project.0.join("a/b"));
        let name = project
            .0
            .file_name()
            .expect("the project has a name")
            .to_string_lossy()
            .into_owned();
        for spelled in [
            format!("../../assets/{stored}"),
            format!("../../../{name}/assets/{stored}"),
            format!("../../a/../assets/{stored}"),
            format!("../../../{name}/a/b/../../assets/{stored}"),
        ] {
            assert_eq!(store.resolve(&spelled), Resolution::Match, "{spelled}");
        }
    }

    /// Whether a path reaches the store is read from the path, never from the filesystem, so a
    /// filesystem that ignores case does not change the answer.
    #[test]
    fn seen_from_a_document_a_path_in_another_case_leads_elsewhere() {
        let project = Project::new("seen-case");
        let stored = project.store("htmlspec.css", b"bytes");
        let store = Store::discover(&project.0).seen_from(&project.0.join("a/b"));
        assert_eq!(
            store.resolve(&format!("../../ASSETS/{stored}")),
            Resolution::Elsewhere {
                leads_to: String::from("../../ASSETS/"),
                store: String::from("../../assets/"),
            }
        );
    }
}
