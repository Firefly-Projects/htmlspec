//! The facts a release states once, and `cargo xtask facts`, which prints them.
//!
//! They live in `xtask/release-facts.json`: the four targets a release builds, each with the
//! GitHub-hosted runner it builds on, in the order `SHA256SUMS` lists them; the public repository,
//! the only one crates are published from; the one User-Agent every request a release makes
//! carries; and the version at or below which the release record lets archived changes share one.
//! Data rather than code, because the release workflow decides which builds run before anything is
//! built, so it reads them where no build has happened, and because a fact about this repository
//! belongs to it rather than to a crate it publishes. The release workflow's verify job reads the
//! build matrix and the User-Agent from the file with `jq`, before it installs a toolchain.
//!
//! This module reads the file into typed values, compiled in: a [`Target`] for each target, so a
//! target the code does not know is refused when the file is read, a [`UserAgent`] that can only
//! hold what a header may, and the [`PublicRepository`]. [`Facts::read`] refuses a file that names a
//! target twice or leaves one out, so the file and [`Target`] cannot drift apart.
//!
//! One more fact is stated once, and not in the file: the version a release of this commit names,
//! which `[workspace.package]` declares. [`release_version`] gives it as Cargo read it when it
//! built the tools, so the jobs that pack the tarballs and render the formula, which install no
//! toolchain, know it without reading a manifest.

use std::fmt;
use std::process::ExitCode;
use std::str::FromStr;

use clap::Subcommand;
use serde::Deserialize;

use htmlspec_cli::version::{ProjectVersion, ReleaseVersion, ReleaseVersionError};

use crate::report::{Exit, Report};
use crate::workspace::RepositoryAddress;

/// The file, as the tools crate was compiled with it.
const FILE: &str = include_str!("../release-facts.json");

/// What `cargo xtask facts` prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Subcommand)]
pub enum Fact {
    /// The User-Agent every request a release makes carries.
    UserAgent,
    /// The targets a release builds, one per line, in the order `SHA256SUMS` lists them.
    Targets {
        /// Leave out the targets that build on macOS.
        #[arg(long)]
        without_macos: bool,
    },
    /// The release workflow's build matrix, as one line of JSON: each target with its runner.
    Matrix {
        /// Leave out the targets that build on macOS.
        #[arg(long)]
        without_macos: bool,
    },
    /// The version a release of this commit names: the one `[workspace.package]` declares, as
    /// Cargo read it when it built the tools.
    Version,
}

/// The version a release of this commit names, the one `[workspace.package]` declares.
///
/// The tools crate takes its version from the workspace, `version.workspace = true`, so Cargo read
/// this from the manifest when it built the tools, and no line of the manifest is read here. That
/// is what lets the release workflow's sums job, which installs no toolchain and so has no cargo to
/// ask, name its tarballs: the tool it runs was built by the verify job of the same run, from the
/// same commit. `this_version_is_the_one_the_workspace_declares` holds it to what `cargo metadata`
/// reads. A version that is not a release's, such as one with a pre-release part, is refused here
/// as the release check refuses it.
pub fn release_version() -> Result<ReleaseVersion, ReleaseVersionError> {
    ReleaseVersion::parse(env!("CARGO_PKG_VERSION"))
}

/// Prints one fact. Exits 2 when the facts file cannot be read, which is a fault in this
/// repository, since the file is compiled in.
pub fn run(fact: Fact) -> ExitCode {
    report(fact).print()
}

/// What asking for one fact prints, and how the run ends.
fn report(fact: Fact) -> Report {
    let printed = match fact {
        Fact::Version => release_version()
            .map(|version| vec![version.to_string()])
            .map_err(|fault| {
                format!("facts: the tools were built at a version that is not a release's: {fault}")
            }),
        Fact::UserAgent => from_file(|facts| vec![facts.user_agent().to_string()]),
        Fact::Targets { without_macos } => from_file(|facts| {
            facts
                .targets(MacosJobs::from_flag(without_macos))
                .into_iter()
                .map(|target| target.to_string())
                .collect()
        }),
        Fact::Matrix { without_macos } => {
            from_file(|facts| vec![facts.matrix(MacosJobs::from_flag(without_macos))])
        }
    };
    match printed {
        Ok(out) => Report {
            exit: Exit::Clean,
            out,
            err: Vec::new(),
        },
        Err(line) => Report {
            exit: Exit::CouldNotAsk,
            out: Vec::new(),
            err: vec![line],
        },
    }
}

/// What `print` makes of the facts file, or why the file could not be read.
fn from_file(print: impl FnOnce(&Facts) -> Vec<String>) -> Result<Vec<String>, String> {
    Facts::read()
        .map(|facts| print(&facts))
        .map_err(|fault| format!("facts: xtask/release-facts.json {fault}"))
}

/// A target a release builds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Aarch64AppleDarwin,
    X8664AppleDarwin,
    X8664UnknownLinuxMusl,
    Aarch64UnknownLinuxMusl,
}

impl Target {
    /// Every target the code knows, which the file must name once each.
    pub const ALL: [Self; 4] = [
        Self::Aarch64AppleDarwin,
        Self::X8664AppleDarwin,
        Self::X8664UnknownLinuxMusl,
        Self::Aarch64UnknownLinuxMusl,
    ];

    /// The target's triple, as `rustup` and `cargo build --target` name it.
    pub fn triple(self) -> &'static str {
        match self {
            Self::Aarch64AppleDarwin => "aarch64-apple-darwin",
            Self::X8664AppleDarwin => "x86_64-apple-darwin",
            Self::X8664UnknownLinuxMusl => "x86_64-unknown-linux-musl",
            Self::Aarch64UnknownLinuxMusl => "aarch64-unknown-linux-musl",
        }
    }

    /// Whether the target builds on macOS. In a private repository a macOS minute costs as much as
    /// ten Linux minutes, so the release workflow builds these, and checks the formula, only in the
    /// public repository or on a pull request labelled `release-check`.
    pub fn builds_on_macos(self) -> bool {
        matches!(self, Self::Aarch64AppleDarwin | Self::X8664AppleDarwin)
    }

    /// The target this triple names, if a release builds it.
    pub fn from_triple(triple: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|target| target.triple() == triple)
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.triple())
    }
}

/// A target named on a command line, by its triple.
impl FromStr for Target {
    type Err = UnknownTarget;

    fn from_str(triple: &str) -> Result<Self, Self::Err> {
        Self::from_triple(triple).ok_or_else(|| UnknownTarget(triple.to_owned()))
    }
}

/// A triple that names no target a release builds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownTarget(pub String);

impl fmt::Display for UnknownTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let known: Vec<&str> = Target::ALL.into_iter().map(Target::triple).collect();
        write!(
            formatter,
            "{:?} is no target a release builds, which are {}",
            self.0,
            known.join(", ")
        )
    }
}

impl std::error::Error for UnknownTarget {}

/// Whether a run of the release workflow runs its macOS jobs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacosJobs {
    Run,
    Skipped,
}

impl MacosJobs {
    fn from_flag(without_macos: bool) -> Self {
        if without_macos {
            Self::Skipped
        } else {
            Self::Run
        }
    }
}

/// A GitHub-hosted runner's label: letters, digits, `.`, `_`, and `-`, as every label GitHub gives
/// is, so it is written into the matrix's JSON as it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Runner(String);

impl Runner {
    fn parse(label: String) -> Result<Self, FactsFault> {
        let allowed = |character: char| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        };
        if label.is_empty() || !label.chars().all(allowed) {
            return Err(FactsFault::Runner(label));
        }
        Ok(Self(label))
    }
}

/// The one User-Agent every request a release makes carries: the project and its public repository,
/// and nothing that names a person or a machine. Printable ASCII and nothing else, so it is a valid
/// header value and cannot carry a line break into a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserAgent(String);

impl UserAgent {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserAgent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// The public repository, the only repository crates are published from: always an address on
/// GitHub, read as its owner and its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicRepository(RepositoryAddress);

impl PublicRepository {
    /// Whether `address` names this repository: on GitHub, by the same owner and name.
    pub fn is(&self, address: &RepositoryAddress) -> bool {
        self.0 == *address
    }
}

impl fmt::Display for PublicRepository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// One build of a release: a target, and the runner it builds on.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Build {
    target: Target,
    runner: Runner,
}

/// The facts, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    public_repository: PublicRepository,
    user_agent: UserAgent,
    /// Every [`Target`] once, in the file's order.
    builds: Vec<Build>,
    /// The version at or below which this repository archived several changes as one release.
    ///
    /// toolchain-packaging's "An archived change records the version it shipped in" requires
    /// recorded versions to be distinct, and provides for a project that archived several changes
    /// under one version before that obligation existed: its changelog carries one baseline entry
    /// naming them, and the suite treats the baseline as one release, until the version advances
    /// past it. Seven of this repository's changes record 0.1.0. A fact about this repository,
    /// which the release check exempts from distinctness, and which `tests/migrate.rs` holds the
    /// migration's report of a shared version to, reading this file by its path.
    shared_release_baseline: ProjectVersion,
}

/// Why the facts file could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactsFault {
    /// Not the facts' shape, from this line and column of the file on.
    NotTheShape {
        line: usize,
        column: usize,
    },
    UnknownTarget(String),
    RepeatedTarget(Target),
    MissingTarget(Target),
    Runner(String),
    UserAgent(String),
    PublicRepository(String),
    SharedReleaseBaseline(ReleaseVersionError),
}

impl fmt::Display for FactsFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotTheShape { line, column } => write!(
                formatter,
                "is not the facts' shape, at line {line}, column {column}"
            ),
            Self::UnknownTarget(triple) => {
                write!(
                    formatter,
                    "names {triple:?}, which is no target the tools know"
                )
            }
            Self::RepeatedTarget(target) => write!(formatter, "names {target} twice"),
            Self::MissingTarget(target) => {
                write!(formatter, "does not name {target}, which a release builds")
            }
            Self::Runner(runner) => write!(
                formatter,
                "names the runner {runner:?}, not a runner's label"
            ),
            Self::UserAgent(agent) => write!(
                formatter,
                "names the User-Agent {agent:?}, which is not printable ASCII alone"
            ),
            Self::PublicRepository(address) => write!(
                formatter,
                "names the public repository {address:?}, not an address on GitHub"
            ),
            Self::SharedReleaseBaseline(reason) => {
                write!(formatter, "names a shared release baseline that {reason}")
            }
        }
    }
}

/// The file's shape, before any of it is trusted.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Raw {
    public_repository: String,
    user_agent: String,
    targets: Vec<RawBuild>,
    shared_release_baseline: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBuild {
    target: String,
    runner: String,
}

impl Facts {
    /// The facts file the tools crate was compiled with.
    pub fn read() -> Result<Self, FactsFault> {
        Self::parse(FILE)
    }

    fn parse(text: &str) -> Result<Self, FactsFault> {
        let raw: Raw = serde_json::from_str(text).map_err(|error| FactsFault::NotTheShape {
            line: error.line(),
            column: error.column(),
        })?;
        let mut builds: Vec<Build> = Vec::new();
        for entry in raw.targets {
            let target = Target::from_triple(&entry.target)
                .ok_or_else(|| FactsFault::UnknownTarget(entry.target.clone()))?;
            if builds.iter().any(|build| build.target == target) {
                return Err(FactsFault::RepeatedTarget(target));
            }
            builds.push(Build {
                target,
                runner: Runner::parse(entry.runner)?,
            });
        }
        if let Some(missing) = Target::ALL
            .into_iter()
            .find(|target| !builds.iter().any(|build| build.target == *target))
        {
            return Err(FactsFault::MissingTarget(missing));
        }
        if raw.user_agent.is_empty()
            || raw
                .user_agent
                .chars()
                .any(|character| !(' '..='~').contains(&character))
        {
            return Err(FactsFault::UserAgent(raw.user_agent));
        }
        let public_repository = RepositoryAddress::parse(&raw.public_repository);
        if !matches!(public_repository, RepositoryAddress::GitHub { .. }) {
            return Err(FactsFault::PublicRepository(raw.public_repository));
        }
        let shared_release_baseline = ReleaseVersion::parse(&raw.shared_release_baseline)
            .map_err(FactsFault::SharedReleaseBaseline)?
            .project();
        Ok(Self {
            public_repository: PublicRepository(public_repository),
            user_agent: UserAgent(raw.user_agent),
            builds,
            shared_release_baseline,
        })
    }

    /// The version at or below which this repository's archived changes may share one.
    pub fn shared_release_baseline(&self) -> ProjectVersion {
        self.shared_release_baseline
    }

    pub fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }

    pub fn public_repository(&self) -> &PublicRepository {
        &self.public_repository
    }

    /// The targets a run builds, in the file's order: all four, or all but the macOS ones.
    pub fn targets(&self, macos: MacosJobs) -> Vec<Target> {
        self.builds(macos).map(|build| build.target).collect()
    }

    /// The build matrix of a run, one entry per target it builds, with its runner, as one line of
    /// JSON. Written out rather than serialized: a triple and a [`Runner`] hold nothing JSON escapes.
    pub fn matrix(&self, macos: MacosJobs) -> String {
        let entries: Vec<String> = self
            .builds(macos)
            .map(|build| {
                format!(
                    r#"{{"target":"{}","runner":"{}"}}"#,
                    build.target.triple(),
                    build.runner.0
                )
            })
            .collect();
        format!(r#"{{"include":[{}]}}"#, entries.join(","))
    }

    fn builds(&self, macos: MacosJobs) -> impl Iterator<Item = &Build> {
        self.builds
            .iter()
            .filter(move |build| macos == MacosJobs::Run || !build.target.builds_on_macos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The matrix the workflow wrote out before it read one, spelled out rather than read from the
    /// code under test.
    const ALL: &str = concat!(
        r#"{"include":["#,
        r#"{"target":"aarch64-apple-darwin","runner":"macos-15"},"#,
        r#"{"target":"x86_64-apple-darwin","runner":"macos-15-intel"},"#,
        r#"{"target":"x86_64-unknown-linux-musl","runner":"ubuntu-24.04"},"#,
        r#"{"target":"aarch64-unknown-linux-musl","runner":"ubuntu-24.04-arm"}"#,
        "]}"
    );
    const LINUX: &str = concat!(
        r#"{"include":["#,
        r#"{"target":"x86_64-unknown-linux-musl","runner":"ubuntu-24.04"},"#,
        r#"{"target":"aarch64-unknown-linux-musl","runner":"ubuntu-24.04-arm"}"#,
        "]}"
    );

    fn facts() -> Facts {
        Facts::read().expect("the facts file reads")
    }

    fn triples(targets: Vec<Target>) -> Vec<&'static str> {
        targets.into_iter().map(Target::triple).collect()
    }

    #[test]
    fn a_run_with_the_macos_jobs_builds_all_four_targets() {
        let facts = facts();
        assert_eq!(facts.matrix(MacosJobs::Run), ALL);
        assert_eq!(
            triples(facts.targets(MacosJobs::Run)),
            [
                "aarch64-apple-darwin",
                "x86_64-apple-darwin",
                "x86_64-unknown-linux-musl",
                "aarch64-unknown-linux-musl",
            ]
        );
    }

    #[test]
    fn a_run_without_them_builds_the_two_linux_targets_and_nothing_on_macos() {
        let facts = facts();
        assert_eq!(facts.matrix(MacosJobs::Skipped), LINUX);
        assert_eq!(
            triples(facts.targets(MacosJobs::Skipped)),
            ["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"]
        );
    }

    /// Spelled out, as the matrix is: this is what every request must carry, and a test that read it
    /// from the code under test would pass whatever that code said.
    #[test]
    fn the_user_agent_names_the_project_and_its_public_repository_and_nothing_else() {
        let facts = facts();
        assert_eq!(
            facts.user_agent().to_string(),
            "htmlspec-release (https://github.com/Firefly-Projects/htmlspec)"
        );
        assert_eq!(
            facts.public_repository.to_string(),
            "https://github.com/Firefly-Projects/htmlspec"
        );
    }

    fn with(targets: &str, user_agent: &str) -> Result<Facts, FactsFault> {
        Facts::parse(&format!(
            r#"{{"publicRepository":"https://github.com/someone/somewhere","userAgent":{user_agent},"targets":[{targets}],"sharedReleaseBaseline":"0.1.0"}}"#
        ))
    }

    const FOUR: &str = concat!(
        r#"{"target":"aarch64-apple-darwin","runner":"a"},"#,
        r#"{"target":"x86_64-apple-darwin","runner":"b"},"#,
        r#"{"target":"x86_64-unknown-linux-musl","runner":"c"},"#,
        r#"{"target":"aarch64-unknown-linux-musl","runner":"d"}"#
    );

    #[test]
    fn a_file_naming_every_target_once_reads() {
        assert!(with(FOUR, r#""agent (x)""#).is_ok());
    }

    #[test]
    fn a_target_the_tools_do_not_know_is_refused() {
        let targets = format!(r#"{FOUR},{{"target":"riscv64gc-unknown-linux-gnu","runner":"e"}}"#);
        assert_eq!(
            with(&targets, r#""agent""#),
            Err(FactsFault::UnknownTarget(String::from(
                "riscv64gc-unknown-linux-gnu"
            )))
        );
    }

    #[test]
    fn a_target_named_twice_or_left_out_is_refused() {
        let twice = format!(r#"{FOUR},{{"target":"x86_64-apple-darwin","runner":"e"}}"#);
        assert_eq!(
            with(&twice, r#""agent""#),
            Err(FactsFault::RepeatedTarget(Target::X8664AppleDarwin))
        );
        let three = FOUR
            .rsplit_once(",{")
            .map(|(three, _)| three)
            .unwrap_or(FOUR);
        assert_eq!(
            with(three, r#""agent""#),
            Err(FactsFault::MissingTarget(Target::Aarch64UnknownLinuxMusl))
        );
    }

    #[test]
    fn a_user_agent_that_could_break_a_header_is_refused() {
        for agent in [r#""""#, r#""agent\r\nX-Account: someone""#, r#""agént""#] {
            assert!(
                matches!(with(FOUR, agent), Err(FactsFault::UserAgent(_))),
                "{agent}"
            );
        }
    }

    #[test]
    fn a_runner_label_the_matrix_would_have_to_escape_is_refused() {
        for label in [r#""mac\"os""#, r#""macos 15""#, r#""""#] {
            let targets = FOUR.replacen(r#""runner":"a""#, &format!(r#""runner":{label}"#), 1);
            assert!(
                matches!(with(&targets, r#""agent""#), Err(FactsFault::Runner(_))),
                "{label}"
            );
        }
    }

    /// The baseline is a release's version as written, and this repository's is 0.1.0.
    #[test]
    fn the_shared_release_baseline_is_a_release_version() {
        let facts = Facts::read().expect("the facts file reads");
        assert_eq!(facts.shared_release_baseline().to_string(), "0.1.0");
        for baseline in ["0.1", "01.0.0", "0.1.0.0"] {
            let text = format!(
                r#"{{"publicRepository":"https://github.com/a/b","userAgent":"a","targets":[{FOUR}],"sharedReleaseBaseline":"{baseline}"}}"#
            );
            assert!(
                matches!(
                    Facts::parse(&text),
                    Err(FactsFault::SharedReleaseBaseline(_))
                ),
                "{baseline}"
            );
        }
    }

    /// The version the tools were built at is the one `cargo metadata` reads for the crates a
    /// release publishes, so the tarballs a tool built from this commit names carry the version
    /// the release check holds the tag to.
    #[test]
    fn this_version_is_the_one_the_workspace_declares() {
        let workspace = crate::workspace::Workspace::here().expect("the workspace reads");
        let published: Vec<&str> = workspace
            .members
            .iter()
            .filter(|member| member.publishable)
            .map(|member| member.version.as_str())
            .collect();
        let version = release_version().expect("the tools were built at a release's version");
        assert!(!published.is_empty());
        for declared in published {
            assert_eq!(version.to_string(), declared);
        }
        assert_eq!(
            report(Fact::Version),
            Report {
                exit: Exit::Clean,
                out: vec![version.to_string()],
                err: Vec::new(),
            }
        );
    }

    #[test]
    fn a_target_is_named_by_its_triple_and_nothing_else() {
        for target in Target::ALL {
            assert_eq!(target.triple().parse(), Ok(target));
        }
        for triple in [
            "riscv64gc-unknown-linux-gnu",
            "x86_64-unknown-linux-gnu",
            "",
            "X86_64-apple-darwin",
        ] {
            assert_eq!(
                triple.parse::<Target>(),
                Err(UnknownTarget(String::from(triple)))
            );
        }
    }

    /// What each fact prints, exit 0, as the release workflow reads it.
    #[test]
    fn each_fact_prints_what_the_workflow_reads() {
        assert_eq!(
            report(Fact::Targets {
                without_macos: true
            })
            .out,
            ["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"]
        );
        assert_eq!(
            report(Fact::Matrix {
                without_macos: false
            })
            .out,
            [ALL]
        );
        let agent = report(Fact::UserAgent);
        assert_eq!(agent.exit, Exit::Clean);
        assert_eq!(
            agent.out,
            ["htmlspec-release (https://github.com/Firefly-Projects/htmlspec)"]
        );
    }

    #[test]
    fn a_field_the_facts_do_not_have_is_refused() {
        let text = format!(
            r#"{{"publicRepository":"https://github.com/a/b","userAgent":"a","targets":[{FOUR}],"sharedReleaseBaseline":"0.1.0","crates":[]}}"#
        );
        assert!(matches!(
            Facts::parse(&text),
            Err(FactsFault::NotTheShape { .. })
        ));
    }
}
