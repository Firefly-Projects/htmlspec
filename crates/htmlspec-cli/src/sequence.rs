//! What a change states about the change it follows, and what that resolves to.
//!
//! # A declaration, never an inference
//!
//! `artifact-kinds` forbids deriving an ordering from a change's name, its creation date, or
//! the requirement identifiers it states, and the prohibition is what shapes this module: the
//! only input is a file the author wrote. Nothing here reads a name for meaning, compares two
//! dates, or looks at a requirement. [`stated`] opens one file and trims it.
//!
//! The name case is the sharpest of the three. `artifact-kinds` states that a change is renamed
//! by moving its directory "because nothing depends on the first name", and an ordering read out
//! of names would make that sentence false.
//!
//! # Why resolution is a sum type and not a boolean
//!
//! A predecessor is archived, active, or matches nothing, and the third differs from the second
//! in what it asks of a contributor: an unfinished predecessor is work to wait for, and an
//! unmatched name is a correction to make. An `Option<bool>` would let a caller read "unmatched"
//! as "unfinished" by writing `!resolved`, and the report a contributor got would send them
//! looking for work that was already done under another name. [`Resolution`] makes that
//! unwritable.
//!
//! # The archive boundary is the whole difficulty
//!
//! Archival moves `a-change` to `archive/2026-09-02-a-change`, so a successor that recorded its
//! predecessor's name cannot find it by equality once it is archived — and archived is precisely
//! the state the successor is waiting for. An equality test would therefore fail in the one case
//! this module exists to detect, and would fail on the *correct* path rather than the incorrect
//! one. The contract states the fix rather than leaving it to an implementer, and
//! [`Resolution::of`] applies it by stripping the leading date with the same test archival uses.

use std::collections::BTreeSet;
use std::path::Path;

use crate::project::ProjectRoot;

/// The change a change states it follows, or `None` where it states none.
///
/// **An absent file and an empty one are one state.** `artifact-kinds` requires that a change
/// stating no predecessor is "complete as it stands" and that an absent declaration "SHALL NOT
/// be reported", so neither case can be an error and the two cannot be distinguishable. Both
/// arrive here as `None`, which leaves no error path for a caller to get wrong.
///
/// Only the first non-empty line is read, and it is trimmed. A change states *at most one*
/// predecessor, so a second line is not a second predecessor — reading it as one would
/// implement a model the contract does not have.
pub fn stated(directory: &Path) -> Option<String> {
    let source = std::fs::read_to_string(directory.join(crate::project::FOLLOWS)).ok()?;
    source
        .lines()
        // A comment lets a contributor say why the change follows what it follows, in the file
        // that states it, without that prose becoming a change name.
        .map(|line| match line.split_once('#') {
            Some((before, _)) => before.trim(),
            None => line.trim(),
        })
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

/// What a stated predecessor turned out to be.
///
/// Three cases because the contract names three outcomes and requires that they "be reported
/// distinctly, because they call for different work — none, waiting, and a correction
/// respectively". Nothing here is optional or nullable: a `Resolution` exists only where a
/// predecessor was stated, so "no predecessor" is the absence of one of these rather than a
/// fourth variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    /// The predecessor is in the archive, under the dated directory named here.
    ///
    /// Carries the directory rather than a bare `true`, so a report can name what it matched.
    Archived {
        /// The archived directory's name, date and all.
        directory: String,
    },
    /// The predecessor is among the project's active changes.
    Active,
    /// No active and no archived change carries that name.
    Unmatched,
}

impl Resolution {
    /// Resolves `predecessor` against everything `project` holds.
    ///
    /// The archive is searched with each directory's leading `YYYY-MM-DD-` removed, using
    /// [`crate::commands::leads_with_a_date`] — the same test archival uses when it decides
    /// whether a name already carries one. A second copy of that test could drift from the
    /// first, and the two would then disagree about exactly the directories whose names were
    /// already dated before archival ran.
    ///
    /// The archive is checked first. A name in both places means a change was archived and a new
    /// change of that name started, and the archived one is the predecessor the successor was
    /// waiting for — reporting `Active` there would report an unfinished predecessor whose work
    /// is done.
    pub fn of(project: &ProjectRoot, predecessor: &str) -> Self {
        if let Some(directory) = archived_as(project, predecessor) {
            return Self::Archived { directory };
        }
        if crate::commands::active_changes(project)
            .iter()
            .any(|name| name == predecessor)
        {
            return Self::Active;
        }
        Self::Unmatched
    }

    /// Whether the predecessor's work is done, which is the only case nothing is reported for.
    ///
    /// A predicate over the sum type rather than a field, so it cannot disagree with the variant
    /// it summarises. Both `Active` and `Unmatched` are unresolved and the two remain
    /// distinguishable from the `Resolution` itself, which is what this being a projection
    /// rather than a replacement preserves.
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Archived { .. })
    }

    /// The word a report and a payload name this outcome by.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Archived { .. } => "archived",
            Self::Active => "active",
            Self::Unmatched => "unmatched",
        }
    }
}

/// The archived directory holding `name`.
///
/// Two forms match, because `archived_name` produces two. A change is archived as
/// `<date>-<name>`, so the date has to come off before the comparison — but a change whose name
/// *already* began with a date keeps the name it had, and this repository's archive holds such a
/// directory (`2026-08-10-html-spec`). For that one the successor's recorded name is the dated
/// string itself, and stripping it would compare `html-spec` against `2026-08-10-html-spec` and
/// miss.
///
/// Accepting both is not a widening of the rule. It is the rule: the archived name of a change
/// is `archived_name(name, date)`, whose two branches are exactly these two forms, so matching
/// against both is matching against what archival can have written.
fn archived_as(project: &ProjectRoot, name: &str) -> Option<String> {
    let entries = std::fs::read_dir(project.archive()).ok()?;
    entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|directory| directory == name || undated(directory) == name)
}

/// A directory name with its leading `YYYY-MM-DD-` removed, where it has one.
///
/// The date test is archival's, not a second copy of it. See [`Resolution::of`].
fn undated(directory: &str) -> &str {
    if crate::commands::leads_with_a_date(directory) {
        // `leads_with_a_date` checked ten ASCII digits and hyphens, so this index is a
        // character boundary.
        return &directory[11..];
    }
    directory
}

/// A change and what it follows, resolved.
///
/// One value per change so the enumeration and archival read the same thing. `predecessor` is
/// `None` for a change that states none, and `resolution` is `Some` exactly when `predecessor`
/// is — the two are kept together rather than derived twice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Follows {
    /// The change stated as the predecessor, as its author wrote it.
    pub predecessor: String,
    /// What that name resolved to.
    pub resolution: Resolution,
}

impl Follows {
    /// What `directory` states, resolved against `project`, or `None` where it states nothing.
    pub fn read(project: &ProjectRoot, directory: &Path) -> Option<Self> {
        let predecessor = stated(directory)?;
        let resolution = Resolution::of(project, &predecessor);
        Some(Self {
            predecessor,
            resolution,
        })
    }
}

/// The changes in a cycle of `follows` declarations reachable from `start`, or `None`.
///
/// **Not a graph algorithm.** With one relation per change there is one edge out of each, so a
/// cycle is found by walking predecessors and noticing a name twice. The walk is bounded by the
/// number of active changes, which is what makes it terminate on a cycle rather than run
/// forever.
///
/// `artifact-kinds` requires that a cycle be "reported as a contradiction" and that the program
/// "SHALL NOT choose an ordering between them", so this returns the names involved and nothing
/// resembling an order — a caller has no ordering to accidentally use. The names are returned
/// sorted for the same reason every other report in this program is sorted: two machines must
/// report one project identically.
///
/// Only active changes are walked. An archived change's work is done, so it cannot be a
/// predecessor anyone is waiting on, and a cycle through the archive is history rather than a
/// contradiction in what remains to be done.
pub fn cycle_through(project: &ProjectRoot, start: &str) -> Option<Vec<String>> {
    let mut walked: Vec<String> = Vec::new();
    let mut at = start.to_owned();

    loop {
        if let Some(from) = walked.iter().position(|name| *name == at) {
            // The names from the repeat onward are the cycle itself; anything before it is the
            // tail that led into one and is not part of the contradiction.
            let mut cycle: Vec<String> = walked.split_off(from);
            cycle.sort();
            cycle.dedup();
            return Some(cycle);
        }
        walked.push(at.clone());

        // A predecessor that is not an active change ends the walk: an archived one is done,
        // and an unmatched one names nothing to continue from. Either way there is no cycle
        // among the changes still being worked on.
        let directory = project.change(&at);
        if !directory.is_dir() {
            return None;
        }
        match stated(&directory) {
            Some(next) => at = next,
            None => return None,
        }
    }
}

/// Every cycle among a project's active changes, each reported once.
///
/// Walking from every change would report a two-change cycle twice, once from each end. The
/// cycles are collected into a set keyed by their sorted membership, so a contradiction is
/// reported as one finding whichever change a reader came at it from.
pub fn cycles(project: &ProjectRoot) -> Vec<Vec<String>> {
    let mut found: BTreeSet<Vec<String>> = BTreeSet::new();
    for name in crate::commands::active_changes(project) {
        if let Some(cycle) = cycle_through(project, &name) {
            found.insert(cycle);
        }
    }
    found.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A temporary directory that removes itself.
    ///
    /// Hand-rolled to match `project.rs` and `discover.rs`, which do the same for the same
    /// reason: a `tempfile` dependency for one directory per test module is more surface than
    /// the lines it replaces.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-sequence-{label}-{}", std::process::id()));
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

    /// A project holding the named active changes, each following what its entry states.
    fn project_with(scratch: &Scratch, changes: &[(&str, Option<&str>)]) -> ProjectRoot {
        let project =
            ProjectRoot::establish(&scratch.0.join("project")).expect("the project is established");
        for (name, follows) in changes {
            let directory = project.change(name);
            std::fs::create_dir_all(&directory).expect("the change directory exists");
            if let Some(follows) = follows {
                std::fs::write(
                    directory.join(crate::project::FOLLOWS),
                    format!("{follows}\n"),
                )
                .expect("the declaration is written");
            }
        }
        project
    }

    /// An archived change, under the dated directory name archival gives it.
    fn archive(project: &ProjectRoot, directory: &str) {
        std::fs::create_dir_all(project.archive().join(directory))
            .expect("the archived change exists");
    }

    /// **The test that fails for a naive equality match.**
    ///
    /// Task 3.4. The predecessor is archived under `2026-09-02-the-contract` and the successor
    /// recorded `the-contract`, which is the name it had while active. String equality against
    /// the archive finds nothing and concludes `Unmatched` — a report telling a contributor
    /// their declaration is wrong, on the correct path, about work that is done.
    #[test]
    fn a_predecessor_archived_under_a_dated_directory_resolves_as_archived() {
        let scratch = Scratch::new("archived");
        let project = project_with(&scratch, &[("the-successor", Some("the-contract"))]);
        archive(&project, "2026-09-02-the-contract");

        let follows = Follows::read(&project, &project.change("the-successor"))
            .expect("the successor states a predecessor");

        assert_eq!(follows.predecessor, "the-contract");
        assert_eq!(
            follows.resolution,
            Resolution::Archived {
                directory: String::from("2026-09-02-the-contract")
            },
            "an equality match would report this as unmatched, which is the failure the \
             date-stripping exists to prevent"
        );
        assert!(follows.resolution.is_resolved());
    }

    /// A change whose name already carried a date is matched as archival left it.
    ///
    /// `archived_name` does not prepend a second date, so the archived directory is the change's
    /// own name — and stripping eleven characters unconditionally would mangle it.
    #[test]
    fn a_predecessor_already_named_with_a_date_still_matches() {
        let scratch = Scratch::new("archived-dated");
        let project = project_with(&scratch, &[("the-successor", Some("2026-08-10-html-spec"))]);
        archive(&project, "2026-08-10-html-spec");

        let follows = Follows::read(&project, &project.change("the-successor")).expect("stated");
        assert!(follows.resolution.is_resolved(), "{:?}", follows.resolution);
    }

    /// Task 3.5: the active and unmatched cases, and that the two are distinguishable from the
    /// resolution alone.
    ///
    /// The second assertion is the one that matters. A contributor waiting on unfinished work
    /// and a contributor holding a wrong name need different reports, and a caller must not be
    /// able to reach one from the other by negating a boolean.
    #[test]
    fn an_active_predecessor_and_an_unmatched_name_are_distinguishable() {
        let scratch = Scratch::new("active-unmatched");
        let project = project_with(
            &scratch,
            &[
                ("the-contract", None),
                ("waiting", Some("the-contract")),
                ("misnamed", Some("a-change-that-never-existed")),
            ],
        );

        let waiting = Follows::read(&project, &project.change("waiting")).expect("stated");
        let misnamed = Follows::read(&project, &project.change("misnamed")).expect("stated");

        assert_eq!(waiting.resolution, Resolution::Active);
        assert_eq!(misnamed.resolution, Resolution::Unmatched);

        // Neither is resolved, and that is the whole of what they share.
        assert!(!waiting.resolution.is_resolved());
        assert!(!misnamed.resolution.is_resolved());
        assert_ne!(
            waiting.resolution, misnamed.resolution,
            "an unfinished predecessor and a wrong name are the same value, so a report cannot \
             separate them"
        );
        assert_ne!(waiting.resolution.label(), misnamed.resolution.label());
    }

    /// Task 3.7: a change stating no predecessor.
    ///
    /// Most changes follow nothing. `artifact-kinds` requires that such a change is complete as
    /// it stands and that nothing about ordering is reported, so this must be `None` and must
    /// never be an error.
    #[test]
    fn a_change_stating_no_predecessor_resolves_to_nothing() {
        let scratch = Scratch::new("none");
        let project = project_with(&scratch, &[("solitary", None)]);

        assert_eq!(stated(&project.change("solitary")), None);
        assert_eq!(Follows::read(&project, &project.change("solitary")), None);
        // And no cycle, which is what keeps the walk from reporting a change against itself.
        assert_eq!(cycle_through(&project, "solitary"), None);
    }

    /// An empty declaration is the same state as an absent one.
    ///
    /// Both mean "no predecessor". A file holding a newline, or spaces, or only a comment, is a
    /// contributor who wrote the file and stated nothing in it.
    #[test]
    fn an_empty_declaration_is_the_same_state_as_an_absent_one() {
        let scratch = Scratch::new("empty");
        let project = project_with(&scratch, &[("a-change", None)]);
        let directory = project.change("a-change");

        for contents in ["", "\n", "   \n\n", "# why this change follows nothing\n"] {
            std::fs::write(directory.join(crate::project::FOLLOWS), contents)
                .expect("the declaration is written");
            assert_eq!(
                stated(&directory),
                None,
                "{contents:?} was read as a predecessor"
            );
        }
    }

    /// The name is read as the author wrote it, and a trailing comment is not part of it.
    #[test]
    fn the_name_is_read_as_written_with_surrounding_noise_removed() {
        let scratch = Scratch::new("trim");
        let project = project_with(&scratch, &[("a-change", None)]);
        let directory = project.change("a-change");

        for contents in [
            "the-contract\n",
            "  the-contract  \n",
            "the-contract  # the ordering is the argument\n",
            "# a leading comment\nthe-contract\n",
            // A second line is not a second predecessor: at most one is carried.
            "the-contract\nanother-change\n",
        ] {
            std::fs::write(directory.join(crate::project::FOLLOWS), contents).expect("written");
            assert_eq!(
                stated(&directory).as_deref(),
                Some("the-contract"),
                "{contents:?}"
            );
        }
    }

    /// Task 3.6: a two-change cycle.
    #[test]
    fn a_two_change_cycle_is_found() {
        let scratch = Scratch::new("cycle-two");
        let project = project_with(
            &scratch,
            &[("first", Some("second")), ("second", Some("first"))],
        );

        assert_eq!(
            cycle_through(&project, "first").as_deref(),
            Some(&[String::from("first"), String::from("second")][..])
        );
        // Reported once, not once per end.
        assert_eq!(
            cycles(&project),
            vec![vec![String::from("first"), String::from("second")]]
        );
    }

    /// Task 3.6: a three-change cycle.
    #[test]
    fn a_three_change_cycle_is_found() {
        let scratch = Scratch::new("cycle-three");
        let project = project_with(
            &scratch,
            &[
                ("alpha", Some("beta")),
                ("beta", Some("gamma")),
                ("gamma", Some("alpha")),
            ],
        );

        let found = cycles(&project);
        assert_eq!(
            found,
            vec![vec![
                String::from("alpha"),
                String::from("beta"),
                String::from("gamma")
            ]],
            "a three-change cycle is one contradiction, reported once"
        );
        // Every member reaches it, and none of them is privileged as its start.
        for name in ["alpha", "beta", "gamma"] {
            assert_eq!(cycle_through(&project, name).map(|c| c.len()), Some(3));
        }
    }

    /// A change following itself is a cycle of one, which is still a contradiction.
    #[test]
    fn a_change_following_itself_is_a_cycle() {
        let scratch = Scratch::new("cycle-self");
        let project = project_with(&scratch, &[("recursive", Some("recursive"))]);

        assert_eq!(
            cycle_through(&project, "recursive").as_deref(),
            Some(&[String::from("recursive")][..])
        );
    }

    /// A chain that leads into a cycle reports the cycle, not the change that led there.
    ///
    /// The tail is not part of the contradiction: `entry` has a coherent predecessor and it is
    /// the two changes behind it whose declarations disagree.
    #[test]
    fn a_chain_into_a_cycle_reports_only_the_cycle() {
        let scratch = Scratch::new("cycle-tail");
        let project = project_with(
            &scratch,
            &[
                ("entry", Some("first")),
                ("first", Some("second")),
                ("second", Some("first")),
            ],
        );

        assert_eq!(
            cycle_through(&project, "entry").as_deref(),
            Some(&[String::from("first"), String::from("second")][..]),
            "the tail that led into the cycle is not part of it"
        );
    }

    /// A long chain with no cycle terminates and reports none.
    #[test]
    fn a_chain_with_no_cycle_reports_none() {
        let scratch = Scratch::new("cycle-chain");
        let project = project_with(
            &scratch,
            &[
                ("fourth", Some("third")),
                ("third", Some("second")),
                ("second", Some("first")),
                ("first", None),
            ],
        );

        assert_eq!(cycle_through(&project, "fourth"), None);
        assert!(cycles(&project).is_empty());
    }

    /// A predecessor that is archived ends the walk rather than continuing through the archive.
    ///
    /// An archived change's work is done, so it is not a predecessor anyone waits on. Walking
    /// into the archive would let history produce a contradiction about work that is finished.
    #[test]
    fn the_walk_stops_at_the_archive() {
        let scratch = Scratch::new("cycle-archived");
        let project = project_with(&scratch, &[("successor", Some("the-contract"))]);
        archive(&project, "2026-09-02-the-contract");

        assert_eq!(cycle_through(&project, "successor"), None);
        assert!(cycles(&project).is_empty());
    }
}
