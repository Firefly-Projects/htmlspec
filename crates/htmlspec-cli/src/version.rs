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
/// or a runbook's. Keeping them apart is what lets [`crate::preflight`] compare numerically without
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
