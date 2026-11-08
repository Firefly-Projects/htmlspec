//! A project version, parsed once and ordered componentwise.
//!
//! Three dot-separated numbers. Parsed at the boundary rather than compared as text,
//! because a string comparison orders `0.10.0` before `0.9.0` — and a version gate that
//! got that backwards would refuse the tenth minor release of a project and accept a
//! downgrade to the ninth.
//!
//! The gate this feeds is an **ordering**, not a test for difference. A difference test
//! accepts a downgrade: archiving at `0.1.0` after `0.2.0` differs, so it would pass. That
//! distinction is why this type exists rather than a `String` comparison at the call site.

use std::cmp::Ordering;

/// A version that can be ordered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

/// Why a version could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VersionError {
    /// Not three components.
    Shape {
        /// How many components were found.
        found: usize,
    },
    /// A component that is not a number.
    Component {
        /// The component as written.
        text: String,
    },
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shape { found } => write!(
                f,
                "a version is three dot-separated numbers like 0.1.0, and this has {found} \
                 component(s)"
            ),
            Self::Component { text } => {
                write!(f, "{text:?} is not a number, so it cannot be ordered")
            }
        }
    }
}

impl std::error::Error for VersionError {}

impl Version {
    /// Reads a version.
    pub fn parse(text: &str) -> Result<Self, VersionError> {
        let parts: Vec<&str> = text.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::Shape { found: parts.len() });
        }
        let mut numbers = [0u64; 3];
        for (slot, part) in numbers.iter_mut().zip(&parts) {
            *slot = part.trim().parse().map_err(|_| VersionError::Component {
                text: (*part).to_owned(),
            })?;
        }
        Ok(Self {
            major: numbers[0],
            minor: numbers[1],
            patch: numbers[2],
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Ord for Version {
    /// Componentwise, most significant first.
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A **runbook's** release version.
///
/// A distinct type from [`ProjectVersion`] rather than a second use of [`Version`], and the reason
/// is that the two are never comparable. `runbook-authority` states that a runbook's version
/// "SHALL identify the runbook's release independently of the project's version" and that
/// distinctness is per runbook — so `0.1.0` for a runbook and `0.14.0` for the project are two
/// facts about two things, and a signature taking `Version` cannot say which one it was handed.
/// Comparing them would compile and would be meaningless.
///
/// Wraps [`Version`] rather than restating its parser, per the design: one ordering rule, three
/// meanings.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RunbookVersion(Version);

/// The **project's** declared version, as the release bookkeeping reads it.
///
/// Distinct from [`RunbookVersion`] for the same reason. This is the version an archive's
/// `VERSION` records and the one release snapshots are ordered by.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProjectVersion(Version);

impl RunbookVersion {
    /// Reads a runbook version.
    pub fn parse(text: &str) -> Result<Self, VersionError> {
        Version::parse(text).map(Self)
    }

    /// The version every runbook starts below, which is also the one no release may carry.
    ///
    /// `runbook-authority`: "A release version SHALL be greater than `0.0.0`." Spelled through the
    /// shared parser rather than as a literal triple, so the value and its shape cannot disagree.
    pub const fn unreleased() -> Self {
        Self(Version {
            major: 0,
            minor: 0,
            patch: 0,
        })
    }

    /// Whether this version can identify a release at all.
    ///
    /// The whole of the gate `project-bootstrap` states for a first release: a runbook with no
    /// prior snapshot must declare a version above [`Self::unreleased`].
    pub fn is_released(self) -> bool {
        self > Self::unreleased()
    }

    /// The major component, which a run check's `atLeast` compares within.
    ///
    /// The change `require-a-run-of-another-runbook` (its design's decision 4): a release counts
    /// only inside the bound's major, because a major release is one that breaks what came before.
    pub const fn major(self) -> u64 {
        self.0.major
    }
}

impl std::fmt::Display for RunbookVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ProjectVersion {
    /// Reads a project version.
    pub fn parse(text: &str) -> Result<Self, VersionError> {
        Version::parse(text).map(Self)
    }
}

impl std::fmt::Display for ProjectVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A **tool's** reported or declared version.
///
/// A third meaning over the same three numbers, and a distinct type for the same reason as the
/// other two: a tool's version is compared against a declaration, never against a project's release
/// or a runbook's. Keeping them apart is what lets `crate::preflight` compare numerically without
/// a reader wondering which of the three a function was handed.
///
/// The declared and the observed forms are one type, which is the point: `runbook-portability`
/// requires the comparison to be between two versions of the same kind, and a report naming an
/// expected and an observed value reads both from here.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ToolVersion(Version);

impl ToolVersion {
    /// Reads a tool version, as exactly three numeric components.
    ///
    /// `3.13` is refused rather than widened to `3.13.0`: a declaration is what the runbook's author
    /// wrote, and guessing the third component would check a condition nobody stated.
    pub fn parse(text: &str) -> Result<Self, VersionError> {
        Version::parse(text).map(Self)
    }
}

impl std::fmt::Display for ToolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Version> for ProjectVersion {
    fn from(version: Version) -> Self {
        Self(version)
    }
}

/// A meaning a version read from text can take: a bare [`Version`], a [`RunbookVersion`], or a
/// [`ProjectVersion`].
///
/// Each reads by [`Version::parse`], so a reading generic over the kind, such as
/// `runbook_release::changelog_versions`, finds the same versions whichever meaning its caller
/// asks for, and the caller still says which meaning it holds. Sealed, so no other crate adds a
/// kind that reads by another grammar: the four grammars the release tools once had were four
/// kinds of this.
pub trait VersionKind: sealed::Sealed + Sized {
    /// Reads a version of this kind.
    fn read(text: &str) -> Result<Self, VersionError>;
}

impl VersionKind for Version {
    fn read(text: &str) -> Result<Self, VersionError> {
        Self::parse(text)
    }
}

impl VersionKind for RunbookVersion {
    fn read(text: &str) -> Result<Self, VersionError> {
        Self::parse(text)
    }
}

impl VersionKind for ProjectVersion {
    fn read(text: &str) -> Result<Self, VersionError> {
        Self::parse(text)
    }
}

mod sealed {
    /// What only this module's kinds of version are.
    pub trait Sealed {}
    impl Sealed for super::Version {}
    impl Sealed for super::RunbookVersion {}
    impl Sealed for super::ProjectVersion {}
}

/// The project's version as a **release** names it: in a tag, a crate, a tarball, and a formula.
///
/// A [`ProjectVersion`] whose text is exactly its own canonical rendering: three numbers with no
/// leading zero, no sign, and no space, and nothing before or after them. The release tools take
/// one wherever a release's version is given to them, in an argument or a tag, and for the version
/// the published crates declare, because a tag, a crate, and a formula must all name the same
/// string, and a version that renders other than it was written is one the release could not
/// carry. [`Version::parse`] is more forgiving, trimming spaces and reading `01` as `1`, and stays
/// so wherever a version is read from a file the program already read in 1.x: a `spec.nix`, a
/// runbook's `VERSION`, and an archived change's `VERSION`, which the release check reads as a
/// [`ProjectVersion`], as archival does.
///
/// The one grammar is still [`Version::parse`]'s. This type refuses what it reads but would render
/// differently, and adds no rule of its own.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ReleaseVersion(ProjectVersion);

/// Why text is not a release's version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseVersionError {
    /// Not a version at all, by [`Version::parse`].
    NotAVersion {
        /// The text as given.
        text: String,
        /// Why the one grammar refused it.
        reason: VersionError,
    },
    /// A version, written otherwise than it renders.
    NotCanonical {
        /// The text as given.
        text: String,
        /// The version as a release names it.
        canonical: ProjectVersion,
    },
}

impl std::fmt::Display for ReleaseVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAVersion { text, reason } => write!(
                f,
                "{text:?} is not a release version: {reason}. Give three numbers, as in 0.33.0"
            ),
            Self::NotCanonical { text, canonical } => write!(
                f,
                "{text:?} is not a release version as written: a release names it {canonical}, \
                 three numbers with no leading zero, sign, or space"
            ),
        }
    }
}

impl std::error::Error for ReleaseVersionError {}

impl ReleaseVersion {
    /// Reads a release's version, refusing any text that is not its own canonical rendering.
    pub fn parse(text: &str) -> Result<Self, ReleaseVersionError> {
        let version = Version::parse(text).map_err(|reason| ReleaseVersionError::NotAVersion {
            text: text.to_owned(),
            reason,
        })?;
        if version.to_string() == text {
            Ok(Self(ProjectVersion(version)))
        } else {
            Err(ReleaseVersionError::NotCanonical {
                text: text.to_owned(),
                canonical: ProjectVersion(version),
            })
        }
    }

    /// The project version this release is, to compare with what an archive or a changelog
    /// records.
    pub const fn project(self) -> ProjectVersion {
        self.0
    }
}

impl std::fmt::Display for ReleaseVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ReleaseVersion {
    type Err = ReleaseVersionError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).unwrap_or_else(|error| panic!("{text} does not parse: {error}"))
    }

    /// The defect a string comparison has, asserted directly.
    #[test]
    fn ten_is_ordered_after_nine() {
        assert!(
            v("0.10.0") > v("0.9.0"),
            "componentwise ordering failed where a string comparison would too"
        );
        // And the string comparison really is wrong, which is why this type exists.
        assert!(
            "0.10.0" < "0.9.0",
            "the premise of this test no longer holds"
        );
    }

    #[test]
    fn ordering_is_componentwise_and_most_significant_first() {
        assert!(v("1.0.0") > v("0.99.99"));
        assert!(v("0.2.0") > v("0.1.99"));
        assert!(v("0.0.2") > v("0.0.1"));
        assert_eq!(v("1.2.3"), v("1.2.3"));
    }

    /// `0.0.0` is what makes the archival comparison total: it is ordered before every
    /// version a project can later declare, so there is never an absent previous value.
    #[test]
    fn the_initial_version_is_ordered_before_every_later_one() {
        let initial = v(crate::commands::INITIAL_VERSION);
        for later in ["0.0.1", "0.1.0", "1.0.0", "99.99.99"] {
            assert!(
                v(later) > initial,
                "{later} is not ordered after the initial version"
            );
        }
        assert_eq!(initial, v("0.0.0"));
    }

    #[test]
    fn a_version_round_trips_through_its_text() {
        for text in ["0.0.0", "1.2.3", "0.10.0", "12.0.7"] {
            assert_eq!(v(text).to_string(), text);
        }
    }

    #[test]
    fn a_version_that_is_not_three_numbers_is_rejected() {
        for text in ["1.0", "1", "1.0.0.0", ""] {
            assert!(
                matches!(Version::parse(text), Err(VersionError::Shape { .. })),
                "{text:?} was accepted"
            );
        }
        for text in ["one.0.0", "1.x.0", "1.0.beta"] {
            assert!(
                matches!(Version::parse(text), Err(VersionError::Component { .. })),
                "{text:?} was accepted"
            );
        }
    }

    /// A rejection says what a version looks like, so a contributor can fix it without
    /// reading the source.
    #[test]
    fn a_rejection_describes_the_shape_it_wanted() {
        let error = Version::parse("1.0").expect_err("rejected");
        assert!(error.to_string().contains("0.1.0"), "{error}");
    }

    /// A runbook version orders like the version it wraps, and `0.0.0` is the one it may not be.
    #[test]
    fn a_runbook_version_orders_and_knows_what_a_release_is() {
        assert!(
            RunbookVersion::parse("0.10.0").expect("parses")
                > RunbookVersion::parse("0.9.0").expect("parses")
        );
        assert!(
            !RunbookVersion::unreleased().is_released(),
            "0.0.0 identified a release, which `runbook-authority` forbids"
        );
        for text in ["0.0.1", "0.1.0", "1.0.0"] {
            assert!(
                RunbookVersion::parse(text).expect("parses").is_released(),
                "{text} should identify a release"
            );
        }
        assert_eq!(
            RunbookVersion::parse("0.1.0").expect("parses").to_string(),
            "0.1.0"
        );
    }

    /// Two runbooks may each be at one version, so the type carries no cross-runbook comparison.
    ///
    /// Asserted as a property of the parse rather than of a comparison: `runbook-authority`
    /// requires that "two different runbooks can each be at `0.1.0`", and nothing in the type
    /// knows which runbook a version belongs to, so there is no way for it to object.
    #[test]
    fn two_runbooks_sharing_a_version_are_both_valid() {
        let one = RunbookVersion::parse("0.1.0").expect("parses");
        let another = RunbookVersion::parse("0.1.0").expect("parses");
        assert_eq!(one, another);
        assert!(one.is_released() && another.is_released());
    }

    /// A malformed runbook version reports the shared parser's reason rather than a second one.
    #[test]
    fn a_runbook_version_rejects_what_a_version_rejects() {
        for text in ["1.0", "one.0.0", ""] {
            assert!(
                RunbookVersion::parse(text).is_err(),
                "{text:?} was accepted as a runbook version"
            );
        }
    }

    /// A tool version is three numeric components, ordered numerically, and is neither of the other
    /// two meanings.
    ///
    /// `3.13` is refused rather than widened: a declaration is what an author wrote, and inventing
    /// the third component would check a condition nobody stated.
    #[test]
    fn a_tool_version_is_three_numeric_components_ordered_numerically() {
        assert!(ToolVersion::parse("3.13.2").is_ok());
        for text in ["3.13", "3", "3.13.2.1", ""] {
            assert!(
                ToolVersion::parse(text).is_err(),
                "{text:?} was accepted as a tool version"
            );
        }
        assert!(
            ToolVersion::parse("3.13.2").expect("parses")
                > ToolVersion::parse("3.9.0").expect("parses"),
            "tool versions are not ordered numerically"
        );
        assert_eq!(
            ToolVersion::parse("3.13.2").expect("parses").to_string(),
            "3.13.2"
        );
    }

    /// A release's version is the one grammar's, refused wherever its text is not its rendering.
    ///
    /// The inputs on which the release tools' four grammars disagreed before they became one:
    ///
    /// | Input        | check-release | set-version | release-notes | `Version::parse` | here    |
    /// | ------------ | ------------- | ----------- | ------------- | --------------- | ------- |
    /// | `0.1`        | 0.1           | refused     | refused       | refused         | refused |
    /// | `0.1.0.0`    | 0.1.0.0       | refused     | refused       | refused         | refused |
    /// | `01.2.3`     | 1.2.3         | refused     | `01.2.3`      | 1.2.3           | refused |
    /// | `1. 2.3`     | refused       | refused     | refused       | 1.2.3           | refused |
    /// | `1.2.3`      | 1.2.3         | 1.2.3       | `1.2.3`       | 1.2.3           | 1.2.3   |
    ///
    /// release-notes compared its argument with a heading as text, so its `01.2.3` named no
    /// heading that check-release read as 1.2.3. The rest are the ones found while porting them.
    #[test]
    fn a_release_version_is_its_own_canonical_rendering() {
        assert_eq!(
            ReleaseVersion::parse("1.2.3").map(|version| version.to_string()),
            Ok(String::from("1.2.3"))
        );
        assert_eq!(
            ReleaseVersion::parse("1.2.3").map(ReleaseVersion::project),
            Ok(ProjectVersion::parse("1.2.3").expect("parses"))
        );
        for (text, found) in [("0.1", 2), ("0.1.0.0", 4)] {
            assert_eq!(
                ReleaseVersion::parse(text),
                Err(ReleaseVersionError::NotAVersion {
                    text: text.to_owned(),
                    reason: VersionError::Shape { found }
                }),
                "{text:?}"
            );
        }
        // What `Version::parse` reads as 1.2.3 but writes otherwise: a leading zero and a space
        // the explainer found, and a sign, a space around it, and a line's end found in the port.
        // `u64`'s own parser takes a `+`, which no grammar before the port did.
        for text in ["01.2.3", "1. 2.3", "+1.2.3", " 1.2.3", "1.2.3\n"] {
            assert_eq!(
                ReleaseVersion::parse(text),
                Err(ReleaseVersionError::NotCanonical {
                    text: text.to_owned(),
                    canonical: ProjectVersion::parse("1.2.3").expect("parses"),
                }),
                "{text:?}"
            );
        }
        // Python's `\d` matched every script's decimal digits and its `int` read them, and its
        // numbers had no ceiling. The one grammar reads ASCII digits into a `u64`.
        for text in ["١.٢.٣", "１.２.３", "18446744073709551616.0.0"] {
            assert!(
                matches!(
                    ReleaseVersion::parse(text),
                    Err(ReleaseVersionError::NotAVersion {
                        reason: VersionError::Component { .. },
                        ..
                    })
                ),
                "{text:?} was read as a release version"
            );
        }
    }

    /// What the program accepts in a `spec.nix` or a runbook's `VERSION` did not change when the
    /// release tools took a stricter type: the forgiving inputs still read.
    #[test]
    fn the_one_grammar_still_forgives_what_it_forgave() {
        for text in ["01.2.3", "1. 2.3", " 1.2.3\n"] {
            assert_eq!(Version::parse(text), Version::parse("1.2.3"), "{text:?}");
            assert_eq!(
                RunbookVersion::parse(text).map(|version| version.to_string()),
                Ok(String::from("1.2.3")),
                "{text:?}"
            );
        }
    }

    /// A rejection of a release version says what one looks like.
    #[test]
    fn a_release_version_rejection_says_what_to_give() {
        let shape = ReleaseVersion::parse("0.1").expect_err("refused");
        assert!(shape.to_string().contains("as in 0.33.0"), "{shape}");
        let written = ReleaseVersion::parse("01.2.3").expect_err("refused");
        assert!(written.to_string().contains("names it 1.2.3"), "{written}");
    }

    /// A project version is the same three numbers, and is not a runbook version.
    ///
    /// The two types are deliberately not interconvertible: `runbook-authority` states a runbook's
    /// version is independent of the project's, so a function handed one cannot silently compare
    /// it against the other. `From<Version>` exists for [`ProjectVersion`] because an archive's
    /// recorded version arrives as one, and there is no such conversion for [`RunbookVersion`].
    #[test]
    fn a_project_version_is_not_a_runbook_version() {
        let project = ProjectVersion::parse("0.14.0").expect("parses");
        assert_eq!(project.to_string(), "0.14.0");
        assert_eq!(
            ProjectVersion::from(v("0.14.0")),
            project,
            "an archive's recorded version does not arrive as a project version"
        );
        // Ordering is the project's own, used to order release snapshots.
        assert!(project > ProjectVersion::parse("0.9.0").expect("parses"));
    }
}
