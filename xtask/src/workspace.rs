//! The workspace as `cargo metadata` reads it, and the crates it publishes, in the order they
//! publish.
//!
//! Cargo already knows the members, what each may be published to, and what each depends on, so
//! the tools ask it rather than keep a list: `cargo metadata --format-version 1 --no-deps`, read
//! into the typed model below. The crates a release publishes are the members whose `publish` is
//! not `false`, and they publish in an order derived from their dependencies on one another, so a
//! crate never reaches crates.io before one it names. A new publishable member joins that order by
//! its manifest alone, and a member marked `publish = false` is left out.
//!
//! `--no-deps` because nothing here needs the resolved graph, and without it Cargo would read every
//! dependency's manifest, which a fresh checkout has not fetched. `--offline` because a reading
//! that needs nothing from a registry should not be able to reach one. The one reader that needs
//! the resolved graph, the notices', reads it through [`graph`].

pub mod graph;

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use htmlspec_cli::version::ReleaseVersion;
use serde::Deserialize;

/// A package's name, as Cargo and the crate index spell it: ASCII letters, digits, `-`, and `_`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateName(String);

impl CrateName {
    pub fn parse(name: &str) -> Option<Self> {
        let allowed =
            |character: char| character.is_ascii_alphanumeric() || matches!(character, '-' | '_');
        (!name.is_empty() && name.chars().all(allowed)).then(|| Self(name.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CrateName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A package's version, as Cargo reports it and the crate index records it: one word of printable
/// ASCII.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageVersion(String);

impl PackageVersion {
    pub fn parse(version: &str) -> Option<Self> {
        (!version.is_empty()
            && version
                .chars()
                .all(|character| character.is_ascii_graphic()))
        .then(|| Self(version.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// The repository a manifest names: one on GitHub, read as its owner and its name, or any other
/// address as written. Two addresses are the same repository only when both are on GitHub with the
/// same owner and name, spelled the same way.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RepositoryAddress {
    GitHub { owner: String, name: String },
    Other(String),
}

impl RepositoryAddress {
    pub fn parse(address: &str) -> Self {
        let segment = |text: &str| {
            !text.is_empty()
                && text
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
        };
        address
            .strip_prefix("https://github.com/")
            .and_then(|path| path.split_once('/'))
            .filter(|(owner, name)| segment(owner) && segment(name))
            .map_or_else(
                || Self::Other(address.to_owned()),
                |(owner, name)| Self::GitHub {
                    owner: owner.to_owned(),
                    name: name.to_owned(),
                },
            )
    }
}

impl fmt::Display for RepositoryAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitHub { owner, name } => write!(formatter, "https://github.com/{owner}/{name}"),
            Self::Other(address) => formatter.write_str(address),
        }
    }
}

/// What kind of dependency a manifest declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyKind {
    Normal,
    Development,
    Build,
}

impl fmt::Display for DependencyKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Normal => "depends on",
            Self::Development => "has a development dependency on",
            Self::Build => "has a build dependency on",
        })
    }
}

/// The versions a dependency accepts, read from the requirement `cargo metadata` writes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Requirement {
    /// `*`: the manifest names no version, as for a dependency named by path alone.
    Any,
    /// `=X.Y.Z`: exactly one release's version, the pin the workspace's crates give one another.
    Exactly(ReleaseVersion),
    /// Any other requirement, as Cargo writes it, which pins no one release.
    Other(String),
}

impl Requirement {
    /// Reads a requirement as `cargo metadata` writes it.
    pub fn parse(requirement: &str) -> Self {
        if requirement == "*" {
            return Self::Any;
        }
        requirement
            .strip_prefix('=')
            .and_then(|version| ReleaseVersion::parse(version).ok())
            .map_or_else(|| Self::Other(requirement.to_owned()), Self::Exactly)
    }

    /// Whether the manifest names no version.
    pub fn names_none(&self) -> bool {
        *self == Self::Any
    }

    /// Whether this accepts exactly `version` and nothing else.
    pub fn pins_exactly(&self, version: ReleaseVersion) -> bool {
        *self == Self::Exactly(version)
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => formatter.write_str("*"),
            Self::Exactly(version) => write!(formatter, "={version}"),
            Self::Other(requirement) => formatter.write_str(requirement),
        }
    }
}

/// One dependency a member's manifest declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub name: CrateName,
    pub kind: DependencyKind,
    pub requirement: Requirement,
    /// Whether the published manifest names it, so crates.io must hold it when the crate that names
    /// it arrives. Every dependency does but a development dependency named by path alone, with no
    /// version: `cargo publish` strips that one from the manifest it uploads.
    pub reaches_the_registry: bool,
}

/// One member of the workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub name: CrateName,
    pub version: PackageVersion,
    /// The directory its manifest is in, which is all `cargo publish` uploads of it.
    pub directory: PathBuf,
    /// Whether the manifest lets the member be published at all: false for `publish = false`.
    pub publishable: bool,
    /// The manifest's `repository`, where it names one.
    pub repository: Option<RepositoryAddress>,
    pub dependencies: Vec<Dependency>,
}

/// The workspace's members, as `cargo metadata` names them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    pub root: PathBuf,
    pub members: Vec<Member>,
}

/// Why the workspace could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataFault {
    /// The current directory, where a tool asks from, could not be read.
    NoDirectory(std::io::ErrorKind),
    /// `cargo` could not be started.
    NotRun(std::io::ErrorKind),
    /// `cargo metadata` exited unsuccessfully, with what cargo said.
    Failed { stderr: String },
    /// Its output was not the shape this reader knows, from this line and column of it on.
    NotTheShape { line: usize, column: usize },
    /// A name Cargo reported that no crate can have.
    Name(String),
    /// A version that is not one word.
    Version { name: CrateName, version: String },
    /// A dependency kind Cargo does not have.
    Kind { name: CrateName, kind: String },
    /// The resolved graph names a package its list of packages does not hold.
    Unresolved(String),
    /// The output holds no resolved graph, which a reading without `--no-deps` always has.
    NoResolve,
}

impl fmt::Display for MetadataFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDirectory(kind) => {
                write!(formatter, "cannot read the current directory: {kind}")
            }
            Self::NotRun(kind) => write!(formatter, "cargo metadata could not be run: {kind}"),
            Self::Failed { stderr } => write!(formatter, "cargo metadata failed: {stderr}"),
            Self::NotTheShape { line, column } => write!(
                formatter,
                "cargo metadata wrote what this reader does not know, at line {line}, column \
                 {column}"
            ),
            Self::Name(name) => write!(formatter, "cargo metadata names the package {name:?}"),
            Self::Version { name, version } => {
                write!(
                    formatter,
                    "cargo metadata gives {name} the version {version:?}"
                )
            }
            Self::Kind { name, kind } => write!(
                formatter,
                "cargo metadata gives a dependency of {name} the kind {kind:?}"
            ),
            Self::Unresolved(id) => write!(
                formatter,
                "cargo metadata's resolved graph names {id:?}, which it lists no package for"
            ),
            Self::NoResolve => formatter.write_str("cargo metadata wrote no resolved graph"),
        }
    }
}

/// Why the workspace names no one version a release can carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UndeclaredVersion {
    /// No member may be published, so none declares a release's version.
    NoPublishedCrate,
    /// The published crates are at more than one version.
    Versions(Vec<PackageVersion>),
    /// Their one version is not written as a release writes it.
    NotARelease {
        root: PathBuf,
        reason: htmlspec_cli::version::ReleaseVersionError,
    },
}

impl fmt::Display for UndeclaredVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPublishedCrate => formatter
                .write_str("the workspace publishes no crate, so it declares no version to check"),
            Self::Versions(versions) => write!(
                formatter,
                "the published crates are at more than one version, {}, and a release has one",
                versions
                    .iter()
                    .map(PackageVersion::as_str)
                    .collect::<Vec<_>>()
                    .join(" and ")
            ),
            Self::NotARelease { root, reason } => write!(
                formatter,
                "no parseable version in {}: {reason}",
                root.join("Cargo.toml").display()
            ),
        }
    }
}

/// A publishable member whose published manifest would name a member marked `publish = false`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unpublishable {
    pub dependent: CrateName,
    pub kind: DependencyKind,
    pub dependency: CrateName,
}

/// Why the publishing order could not be derived.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderFault {
    /// These publishable members depend on one another in a cycle, so none of them can be first.
    Cycle(Vec<CrateName>),
    /// Publishable members that depend on unpublishable ones. crates.io would refuse each of them,
    /// after the crates before it in the order were published, and a published version cannot be
    /// taken back, so a release with any of these does not start.
    DependsOnUnpublishable(Vec<Unpublishable>),
}

impl fmt::Display for OrderFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cycle(names) => write!(
                formatter,
                "{} depend on one another in a cycle, so no order publishes each after what it \
                 depends on",
                names
                    .iter()
                    .map(CrateName::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::DependsOnUnpublishable(pairs) => {
                let said: Vec<String> = pairs
                    .iter()
                    .map(|pair| format!("{} {} {}", pair.dependent, pair.kind, pair.dependency))
                    .collect();
                write!(
                    formatter,
                    "{}, which is marked publish = false. crates.io would refuse what depends on it \
                     after publishing what comes before, and a published crate cannot be taken \
                     back, so nothing is published",
                    said.join(", and ")
                )
            }
        }
    }
}

#[derive(Deserialize)]
struct RawMetadata {
    packages: Vec<RawPackage>,
    workspace_members: Vec<String>,
    workspace_root: PathBuf,
}

#[derive(Deserialize)]
struct RawPackage {
    name: String,
    version: String,
    id: String,
    /// `null` for a package that may be published anywhere, the list of registries otherwise, and
    /// the empty list for `publish = false`.
    publish: Option<Vec<String>>,
    repository: Option<String>,
    manifest_path: PathBuf,
    dependencies: Vec<RawDependency>,
}

#[derive(Deserialize)]
struct RawDependency {
    name: String,
    kind: Option<String>,
    /// The version requirement, `*` where the manifest names none.
    req: String,
    /// The local path, for a dependency named by path.
    #[serde(default)]
    path: Option<PathBuf>,
}

/// The cargo that runs this tool, which `cargo xtask` names in `CARGO`, so the workspace is read,
/// relocked, and built by the toolchain the repository pins, or else the first `cargo` on `PATH`.
pub fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

impl Workspace {
    /// Asks [`cargo`] for the workspace the current directory belongs to, which is where every
    /// tool runs from.
    pub fn here() -> Result<Self, MetadataFault> {
        let directory =
            std::env::current_dir().map_err(|error| MetadataFault::NoDirectory(error.kind()))?;
        Self::read(&cargo(), &directory)
    }

    /// Asks `cargo` for the workspace rooted at, or containing, `directory`.
    pub fn read(cargo: &OsStr, directory: &Path) -> Result<Self, MetadataFault> {
        let output = Command::new(cargo)
            .args([
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--locked",
                "--offline",
            ])
            .current_dir(directory)
            .output()
            .map_err(|error| MetadataFault::NotRun(error.kind()))?;
        if !output.status.success() {
            return Err(MetadataFault::Failed {
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        Self::parse(&String::from_utf8_lossy(&output.stdout))
    }

    /// Reads `cargo metadata`'s output.
    pub fn parse(json: &str) -> Result<Self, MetadataFault> {
        let raw: RawMetadata =
            serde_json::from_str(json).map_err(|error| MetadataFault::NotTheShape {
                line: error.line(),
                column: error.column(),
            })?;
        let mut members = Vec::new();
        for package in raw.packages {
            if !raw.workspace_members.contains(&package.id) {
                continue;
            }
            let name = CrateName::parse(&package.name)
                .ok_or_else(|| MetadataFault::Name(package.name.clone()))?;
            let version =
                PackageVersion::parse(&package.version).ok_or_else(|| MetadataFault::Version {
                    name: name.clone(),
                    version: package.version.clone(),
                })?;
            let mut dependencies = Vec::new();
            for dependency in package.dependencies {
                let kind = match dependency.kind.as_deref() {
                    None => DependencyKind::Normal,
                    Some("dev") => DependencyKind::Development,
                    Some("build") => DependencyKind::Build,
                    Some(other) => {
                        return Err(MetadataFault::Kind {
                            name,
                            kind: other.to_owned(),
                        })
                    }
                };
                let requirement = Requirement::parse(&dependency.req);
                let path_alone = dependency.path.is_some() && requirement.names_none();
                dependencies.push(Dependency {
                    name: CrateName::parse(&dependency.name)
                        .ok_or(MetadataFault::Name(dependency.name))?,
                    kind,
                    requirement,
                    reaches_the_registry: !(kind == DependencyKind::Development && path_alone),
                });
            }
            members.push(Member {
                name,
                version,
                directory: package
                    .manifest_path
                    .parent()
                    .map_or_else(PathBuf::new, Path::to_path_buf),
                publishable: package
                    .publish
                    .is_none_or(|registries| !registries.is_empty()),
                repository: package.repository.as_deref().map(RepositoryAddress::parse),
                dependencies,
            });
        }
        Ok(Self {
            root: raw.workspace_root,
            members,
        })
    }

    /// The version a release of this workspace names: the one version its published crates
    /// declare, which `[workspace.package]` gives each of them, read as a release writes it.
    ///
    /// The release check asks it of this repository, and the private working repository's export
    /// asks it of the tree it exports, whose commit and pull request name it.
    pub fn release_version(&self) -> Result<ReleaseVersion, UndeclaredVersion> {
        let versions: BTreeSet<&PackageVersion> = self
            .members
            .iter()
            .filter(|member| member.publishable)
            .map(|member| &member.version)
            .collect();
        let versions: Vec<&PackageVersion> = versions.into_iter().collect();
        match versions.as_slice() {
            [] => Err(UndeclaredVersion::NoPublishedCrate),
            [version] => ReleaseVersion::parse(version.as_str()).map_err(|reason| {
                UndeclaredVersion::NotARelease {
                    root: self.root.clone(),
                    reason,
                }
            }),
            many => Err(UndeclaredVersion::Versions(
                many.iter().copied().cloned().collect(),
            )),
        }
    }

    /// The members a release publishes, each after every other one it depends on.
    ///
    /// Only the dependencies the published manifest names order anything, as
    /// [`Dependency::reaches_the_registry`] says. A dependency on a registry crate orders nothing,
    /// because crates.io holds it already. One on a member marked `publish = false` refuses the
    /// whole order, because crates.io would refuse the crate that names it after the crates before
    /// it were published, and those cannot be taken back. Where more than one crate could come
    /// next, the first by name does, so the order is the same on every run.
    pub fn publishing_order(&self) -> Result<Vec<&Member>, OrderFault> {
        let publishable: Vec<&Member> = self
            .members
            .iter()
            .filter(|member| member.publishable)
            .collect();
        let unpublishable: BTreeSet<&CrateName> = self
            .members
            .iter()
            .filter(|member| !member.publishable)
            .map(|member| &member.name)
            .collect();
        let mut refused: Vec<Unpublishable> = publishable
            .iter()
            .flat_map(|member| {
                member
                    .dependencies
                    .iter()
                    .filter(|dependency| {
                        dependency.reaches_the_registry && unpublishable.contains(&dependency.name)
                    })
                    .map(|dependency| Unpublishable {
                        dependent: member.name.clone(),
                        kind: dependency.kind,
                        dependency: dependency.name.clone(),
                    })
            })
            .collect();
        if !refused.is_empty() {
            refused.sort();
            refused.dedup();
            return Err(OrderFault::DependsOnUnpublishable(refused));
        }
        let names: BTreeSet<&CrateName> = publishable.iter().map(|member| &member.name).collect();
        let mut order: Vec<&Member> = Vec::new();
        let mut placed: BTreeSet<&CrateName> = BTreeSet::new();
        while order.len() < publishable.len() {
            let next = publishable
                .iter()
                .filter(|member| !placed.contains(&member.name))
                .filter(|member| {
                    member.dependencies.iter().all(|dependency| {
                        !dependency.reaches_the_registry
                            || dependency.name == member.name
                            || !names.contains(&dependency.name)
                            || placed.contains(&dependency.name)
                    })
                })
                .min_by(|one, other| one.name.cmp(&other.name));
            let Some(next) = next else {
                let mut stuck: Vec<CrateName> = publishable
                    .iter()
                    .filter(|member| !placed.contains(&member.name))
                    .map(|member| member.name.clone())
                    .collect();
                stuck.sort();
                return Err(OrderFault::Cycle(stuck));
            };
            placed.insert(&next.name);
            order.push(next);
        }
        Ok(order)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// One member, as `cargo metadata` writes it: a name, its dependencies on other members with
    /// their kinds, and whether it publishes.
    pub(crate) struct Sketch<'a> {
        pub name: &'a str,
        pub publish: bool,
        pub dependencies: &'a [(&'a str, Option<&'a str>)],
    }

    /// The kind a sketch gives a development dependency named by path alone, with no version.
    const DEV_BY_PATH_ALONE: Option<&str> = Some("dev by path alone");

    /// `cargo metadata`'s output for a workspace of `members`, at `version`, naming `repository`,
    /// with one dependency on a registry crate each, which orders nothing. Each dependency on a
    /// member carries the version pin `workspace = true` gives it, but one of
    /// [`DEV_BY_PATH_ALONE`], which carries none.
    pub(crate) fn metadata(members: &[Sketch<'_>], version: &str, repository: &str) -> String {
        let packages: Vec<serde_json::Value> = members
            .iter()
            .map(|member| {
                let mut dependencies: Vec<serde_json::Value> = member
                    .dependencies
                    .iter()
                    .map(|(name, kind)| {
                        let (kind, requirement) = if *kind == DEV_BY_PATH_ALONE {
                            (Some("dev"), String::from("*"))
                        } else {
                            (*kind, format!("={version}"))
                        };
                        serde_json::json!({
                            "name": name, "kind": kind, "req": requirement,
                            "source": null, "target": null, "path": format!("/w/crates/{name}"),
                        })
                    })
                    .collect();
                dependencies.push(serde_json::json!({
                    "name": "serde", "kind": null, "req": "1.0",
                    "source": "registry+https://github.com/rust-lang/crates.io-index", "target": null,
                }));
                serde_json::json!({
                    "name": member.name,
                    "version": version,
                    "id": format!("path+file:///w/crates/{}#{version}", member.name),
                    "manifest_path": format!("/w/crates/{}/Cargo.toml", member.name),
                    "publish": if member.publish { serde_json::Value::Null } else { serde_json::json!([]) },
                    "repository": repository,
                    "dependencies": dependencies,
                })
            })
            .collect();
        let ids: Vec<serde_json::Value> = packages
            .iter()
            .map(|package| package["id"].clone())
            .collect();
        serde_json::json!({
            "packages": packages,
            "workspace_members": ids,
            "workspace_default_members": [],
            "resolve": null,
            "target_directory": "/w/target",
            "version": 1,
            "workspace_root": "/w",
            "metadata": null,
        })
        .to_string()
    }

    fn order(members: &[Sketch<'_>]) -> Result<Vec<String>, OrderFault> {
        let workspace = Workspace::parse(&metadata(members, "1.2.3", "https://github.com/a/b"))
            .expect("the sketch reads");
        workspace.publishing_order().map(|order| {
            order
                .into_iter()
                .map(|member| member.name.as_str().to_owned())
                .collect()
        })
    }

    /// The six crates of this workspace, as their manifests declare them today.
    pub(crate) const THIS_WORKSPACE: [Sketch<'static>; 8] = [
        Sketch {
            name: "htmlspec-assets",
            publish: true,
            dependencies: &[
                ("htmlspec-parse", Some("dev")),
                ("htmlspec-schema", Some("dev")),
            ],
        },
        Sketch {
            name: "htmlspec-parse",
            publish: true,
            dependencies: &[("htmlspec-schema", None)],
        },
        Sketch {
            name: "htmlspec-schema",
            publish: true,
            dependencies: &[],
        },
        Sketch {
            name: "htmlspec-cli",
            publish: true,
            dependencies: &[
                ("htmlspec-assets", None),
                ("htmlspec-format", None),
                ("htmlspec-parse", None),
                ("htmlspec-schema", None),
                ("htmlspec-validate", None),
            ],
        },
        Sketch {
            name: "htmlspec-format",
            publish: true,
            dependencies: &[
                ("htmlspec-assets", None),
                ("htmlspec-parse", None),
                ("htmlspec-schema", Some("dev")),
            ],
        },
        Sketch {
            name: "htmlspec-validate",
            publish: true,
            dependencies: &[
                ("htmlspec-assets", None),
                ("htmlspec-parse", None),
                ("htmlspec-schema", None),
                ("htmlspec-format", Some("dev")),
            ],
        },
        Sketch {
            name: "htmlspec-conformance",
            publish: false,
            dependencies: &[("htmlspec-validate", Some("dev"))],
        },
        Sketch {
            name: "htmlspec-xtask",
            publish: false,
            dependencies: &[],
        },
    ];

    /// The order the Python release facts kept by hand, as a list of the six crates, before the
    /// published crates were derived from the workspace.
    const CRATES: [&str; 6] = [
        "htmlspec-schema",
        "htmlspec-parse",
        "htmlspec-assets",
        "htmlspec-format",
        "htmlspec-validate",
        "htmlspec-cli",
    ];

    #[test]
    fn a_member_marked_unpublished_is_left_out() {
        let found = order(&THIS_WORKSPACE).expect("an order");
        assert_eq!(found, CRATES);
        assert!(!found.iter().any(|name| name == "htmlspec-conformance"));
        assert!(!found.iter().any(|name| name == "htmlspec-xtask"));
    }

    #[test]
    fn a_new_publishable_member_joins_the_order_after_its_dependencies() {
        let mut members: Vec<Sketch<'_>> = THIS_WORKSPACE.into_iter().collect();
        // Named to sort first, so only its dependencies can hold it back.
        members.push(Sketch {
            name: "htmlspec-aaa",
            publish: true,
            dependencies: &[
                ("htmlspec-format", None),
                ("htmlspec-schema", Some("build")),
            ],
        });
        let found = order(&members).expect("an order");
        let position = |name: &str| found.iter().position(|found| found == name);
        assert_eq!(found.len(), 7);
        assert!(
            position("htmlspec-aaa") > position("htmlspec-format"),
            "{found:?}"
        );
        assert!(
            position("htmlspec-aaa") > position("htmlspec-schema"),
            "{found:?}"
        );
        assert_eq!(position("htmlspec-aaa"), Some(4), "{found:?}");
    }

    #[test]
    fn a_cycle_is_refused() {
        let members = [
            Sketch {
                name: "one",
                publish: true,
                dependencies: &[("two", None)],
            },
            Sketch {
                name: "two",
                publish: true,
                dependencies: &[("one", Some("dev"))],
            },
            Sketch {
                name: "three",
                publish: true,
                dependencies: &[],
            },
        ];
        assert_eq!(
            order(&members),
            Err(OrderFault::Cycle(vec![
                CrateName(String::from("one")),
                CrateName(String::from("two"))
            ]))
        );
    }

    /// `cargo publish` strips a development dependency named by path alone from the manifest it
    /// uploads, so crates.io never sees it: it orders nothing, makes no cycle, and may name a member
    /// that is never published.
    #[test]
    fn a_development_dependency_named_by_path_alone_orders_nothing() {
        let members = [
            Sketch {
                name: "one",
                publish: true,
                dependencies: &[("two", None)],
            },
            Sketch {
                name: "two",
                publish: true,
                dependencies: &[("one", DEV_BY_PATH_ALONE), ("harness", DEV_BY_PATH_ALONE)],
            },
            Sketch {
                name: "harness",
                publish: false,
                dependencies: &[],
            },
        ];
        assert_eq!(
            order(&members),
            Ok(vec![String::from("two"), String::from("one")])
        );
    }

    /// The case that cannot be taken back. With `htmlspec-format` marked `publish = false`, a run
    /// would publish schema, parse, and assets, and then crates.io would refuse validate. So the
    /// order is refused, naming every crate that names it, by every kind of dependency that reaches
    /// the registry.
    #[test]
    fn a_crate_that_depends_on_one_marked_unpublished_is_refused() {
        let members: Vec<Sketch<'_>> = THIS_WORKSPACE
            .into_iter()
            .map(|member| Sketch {
                publish: member.publish && member.name != "htmlspec-format",
                ..member
            })
            .collect();
        let name = |text: &str| CrateName(String::from(text));
        assert_eq!(
            order(&members),
            Err(OrderFault::DependsOnUnpublishable(vec![
                Unpublishable {
                    dependent: name("htmlspec-cli"),
                    kind: DependencyKind::Normal,
                    dependency: name("htmlspec-format"),
                },
                Unpublishable {
                    dependent: name("htmlspec-validate"),
                    kind: DependencyKind::Development,
                    dependency: name("htmlspec-format"),
                },
            ]))
        );
        let refused = order(&members).err().map(|fault| fault.to_string());
        assert_eq!(
            refused.as_deref(),
            Some(
                "htmlspec-cli depends on htmlspec-format, and htmlspec-validate has a development \
                 dependency on htmlspec-format, which is marked publish = false. crates.io would \
                 refuse what depends on it after publishing what comes before, and a published \
                 crate cannot be taken back, so nothing is published"
            )
        );
    }

    #[test]
    fn a_repository_on_github_is_read_as_its_owner_and_name() {
        assert_eq!(
            RepositoryAddress::parse("https://github.com/Firefly-Projects/htmlspec"),
            RepositoryAddress::GitHub {
                owner: String::from("Firefly-Projects"),
                name: String::from("htmlspec"),
            }
        );
        for other in [
            "https://github.com/Firefly-Projects/htmlspec/",
            "https://github.com/Firefly-Projects",
            "http://github.com/Firefly-Projects/htmlspec",
            "https://gitlab.com/Firefly-Projects/htmlspec",
            "https://github.com/Firefly-Projects/html spec",
        ] {
            assert_eq!(
                RepositoryAddress::parse(other),
                RepositoryAddress::Other(String::from(other)),
                "{other}"
            );
        }
    }

    /// The order the hand-kept list held, derived from this repository's own manifests, so a new
    /// dependency that changes it shows here rather than at a release.
    #[test]
    fn this_workspace_publishes_the_six_crates_in_the_order_the_list_kept() {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let workspace = Workspace::read(&cargo, &root).expect("this workspace reads");
        let found: Vec<&str> = workspace
            .publishing_order()
            .expect("an order")
            .into_iter()
            .map(|member| member.name.as_str())
            .collect();
        assert_eq!(found, CRATES);
    }

    #[test]
    fn a_name_or_kind_cargo_does_not_have_is_refused() {
        let text = metadata(
            &[Sketch {
                name: "one",
                publish: true,
                dependencies: &[("two", Some("optional"))],
            }],
            "1.0.0",
            "https://github.com/a/b",
        );
        assert!(matches!(
            Workspace::parse(&text),
            Err(MetadataFault::Kind { .. })
        ));
        let text = metadata(
            &[Sketch {
                name: "one two",
                publish: true,
                dependencies: &[],
            }],
            "1.0.0",
            "https://github.com/a/b",
        );
        assert!(matches!(
            Workspace::parse(&text),
            Err(MetadataFault::Name(_))
        ));
    }
}
