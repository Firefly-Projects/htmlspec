//! Which asset stores are a project's, and the duplicate check made over them.
//!
//! document-validation's "A project must not store one asset's bytes more than once" takes a
//! project as its subject, and a project's stores are the ones within its own `htmlspec/`
//! directory. A repository holding a project can hold other stores that are not the project's:
//! a conformance fixture that is a project of its own, a toolchain's source copy of the
//! runtime, a vendored copy another tool reads. Bytes those share with the project's are not
//! the project storing its bytes twice.
//!
//! Before this module, `assets` compared every store beneath the resolved project's *path*,
//! which in a repository is the repository root. Creating an empty `htmlspec/` in this
//! repository reported nine duplicates, none of them the project's. Promotion made the same
//! comparison in a second copy of the loop. Both now call [`check`], so a staged project and a
//! promoted one are judged by one function.

use std::path::{Path, PathBuf};

use htmlspec_assets::Store;
use htmlspec_validate::{rules, Diagnostic};

use crate::project::{canonical, ProjectRoot};

/// The stores in view, split by whether they lie within the project's own directory.
///
/// Two lists rather than a flag per store, because the two are treated differently and a
/// caller that reports them has to name both: a contributor whose duplicate is no longer
/// reported should be able to see that the store was checked on its own, rather than wonder
/// whether it was checked at all.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Partition {
    /// Stores within `htmlspec/`, compared with one another.
    pub project: Vec<PathBuf>,
    /// Every other store, each checked on its own and compared with nothing.
    pub others: Vec<PathBuf>,
}

/// Splits `stores` into the project's and the others.
///
/// A store is the project's when its directory — the parent of its `assets/`, as
/// [`crate::discover::stores`] returns it — is `project.htmlspec()` or lies beneath it. The
/// test is on canonical paths, so a project reached through a symbolic link and a store
/// discovered through its target are still one project's. `starts_with` is component-wise, so
/// `htmlspec-old/` is not taken for a directory beneath `htmlspec/`. With no project in view,
/// every store is another's, which is the arrangement the check already had for that case.
///
/// Order within each list is the order of `stores`.
pub fn partition(project: Option<&ProjectRoot>, stores: Vec<PathBuf>) -> Partition {
    let Some(project) = project else {
        return Partition {
            project: Vec::new(),
            others: stores,
        };
    };
    let extent = canonical(&project.htmlspec());
    let (project, others) = stores
        .into_iter()
        .partition(|store| canonical(store).starts_with(&extent));
    Partition { project, others }
}

/// What the duplicate check found among the stores beneath one root.
#[derive(Debug)]
pub struct Topology {
    /// Which stores were the project's and which were checked on their own.
    pub partition: Partition,
    /// How many assets the stores hold, together.
    pub stored: usize,
    /// Every duplication found: within any one store, and across the project's stores.
    pub diagnostics: Vec<Diagnostic>,
}

/// Checks every store beneath `root` for duplicated bytes.
///
/// Each store is checked on its own, wherever it lives, so a store holding the same bytes
/// twice is reported even outside the project. Only the project's stores are compared with
/// one another. Diagnostics follow the stores' discovery order, then the comparison across
/// the project's stores, which is the order the check had before the partition existed.
pub fn check(project: Option<&ProjectRoot>, root: &Path) -> Topology {
    let partition = partition(project, crate::discover::stores(root));
    let mut stored = 0usize;
    let mut diagnostics = Vec::new();
    let mut located: Vec<(String, String)> = Vec::new();

    let mut discovered: Vec<&PathBuf> = partition.project.iter().chain(&partition.others).collect();
    discovered.sort();
    for directory in discovered {
        let store = Store::discover(directory);
        stored += store.len();
        diagnostics.extend(rules::topology::duplicates(&store));
        if partition.project.contains(directory) {
            located.extend(store.located_digests());
        }
    }
    diagnostics.extend(rules::topology::duplicates_across(&located));

    Topology {
        partition,
        stored,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary directory that removes itself.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-topology-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            Self(path)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn holding_a_store(at: &Path) -> PathBuf {
        std::fs::create_dir_all(at.join("assets")).expect("the store is created");
        at.to_owned()
    }

    /// The project's own store sits at `htmlspec/assets`, so its directory is `htmlspec/`
    /// itself — the boundary of the extent, which a strict "beneath" would have excluded.
    #[test]
    fn the_projects_own_store_is_the_projects() {
        let scratch = Scratch::new("own");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        let stores = vec![holding_a_store(&project.htmlspec())];

        let split = partition(Some(&project), stores.clone());

        assert_eq!(split.project, stores);
        assert!(split.others.is_empty());
    }

    /// Every case the design names, in one tree: the project's store, a store beside a change's
    /// documents, the repository root's store, a fixture's store, and a sibling whose name
    /// merely begins with the project directory's.
    #[test]
    fn a_store_is_the_projects_exactly_when_it_lies_within_htmlspec() {
        let scratch = Scratch::new("extent");
        let project = ProjectRoot::establish(&scratch.0).expect("established");
        let own = holding_a_store(&project.htmlspec());
        let beside_a_change = holding_a_store(&project.change("a-change"));
        let root = holding_a_store(&scratch.0);
        let fixture = holding_a_store(&scratch.0.join("fixtures/a-fixture"));
        let lookalike = holding_a_store(&scratch.0.join("htmlspec-old"));

        let split = partition(
            Some(&project),
            vec![
                root.clone(),
                fixture.clone(),
                own.clone(),
                beside_a_change.clone(),
                lookalike.clone(),
            ],
        );

        assert_eq!(split.project, vec![own, beside_a_change]);
        assert_eq!(split.others, vec![root, fixture, lookalike]);
    }

    /// A project reached through a symbolic link owns the stores discovered through the link's
    /// target, and the reverse. Path strings differ and the directories do not.
    #[cfg(unix)]
    #[test]
    fn a_project_reached_through_a_symbolic_link_keeps_its_stores() {
        let scratch = Scratch::new("symlink");
        let real = scratch.0.join("real");
        std::fs::create_dir_all(&real).expect("created");
        let project = ProjectRoot::establish(&real).expect("established");
        let own = holding_a_store(&project.htmlspec());
        let outside = holding_a_store(&real.join("fixtures/a-fixture"));

        let link = scratch.0.join("link");
        std::os::unix::fs::symlink(&real, &link).expect("the link is created");
        let through_link = ProjectRoot::resolve(&link).expect("the link resolves to a project");

        let split = partition(Some(&through_link), vec![own.clone(), outside.clone()]);
        assert_eq!(split.project, vec![own.clone()]);
        assert_eq!(split.others, vec![outside.clone()]);

        let linked_own = link.join("htmlspec");
        let split = partition(Some(&project), vec![linked_own.clone(), outside.clone()]);
        assert_eq!(split.project, vec![linked_own]);
        assert_eq!(split.others, vec![outside]);
    }

    /// With no project in view, every store is checked on its own, as before the partition.
    #[test]
    fn with_no_project_every_store_is_anothers() {
        let scratch = Scratch::new("none");
        let stores = vec![
            holding_a_store(&scratch.0.join("one")),
            holding_a_store(&scratch.0.join("two")),
        ];

        let split = partition(None, stores.clone());

        assert!(split.project.is_empty());
        assert_eq!(split.others, stores);
    }
}
