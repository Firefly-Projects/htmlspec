//! The dependency graph `cargo metadata` resolves for one target: every package a build of the
//! workspace for that target reaches, and how each depends on the others.
//!
//! `cargo xtask crate-notices` reads it once for each target a release builds, because the crates
//! the binary is built from are the ones its build reaches there, and a crate only another platform
//! builds is not one of them. It is read with `cargo metadata --format-version 1 --locked
//! --filter-platform TARGET`: without `--no-deps`, because the resolved graph is the point, and
//! without `--offline`, because the notices read each crate's license files from its source, which
//! cargo fetches on a checkout that never built. On one that did, and in the Nix sandbox, which
//! vendors every source, it reaches no network.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use super::{CrateName, DependencyKind, MetadataFault, PackageVersion};
use crate::facts::Target;

/// A package as `cargo metadata` identifies it, which tells apart two packages of one name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageId(String);

/// One package the graph reaches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub name: CrateName,
    pub version: PackageVersion,
    /// The manifest's `license`, as written, where it states one.
    pub license: Option<String>,
    pub authors: Vec<String>,
    /// The manifest's `repository`, as written, where it states one.
    pub repository: Option<String>,
    /// The directory its manifest is in, which holds its source and its license files.
    pub directory: PathBuf,
}

/// One package another depends on, and every kind of dependency by which it does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub to: PackageId,
    pub kinds: BTreeSet<DependencyKind>,
}

/// The packages a build for one target reaches, the workspace's members among them, and the
/// dependencies between them. Every edge names a package the graph holds: [`Graph::parse`]
/// refuses output where one does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph {
    /// The workspace's root, where its manifest is.
    pub root: PathBuf,
    packages: BTreeMap<PackageId, Package>,
    members: BTreeSet<PackageId>,
    edges: BTreeMap<PackageId, Vec<Edge>>,
}

#[derive(Deserialize)]
struct RawMetadata {
    packages: Vec<RawPackage>,
    workspace_members: Vec<String>,
    workspace_root: PathBuf,
    resolve: Option<RawResolve>,
}

#[derive(Deserialize)]
struct RawPackage {
    id: String,
    name: String,
    version: String,
    license: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    repository: Option<String>,
    manifest_path: PathBuf,
}

#[derive(Deserialize)]
struct RawResolve {
    nodes: Vec<RawNode>,
}

#[derive(Deserialize)]
struct RawNode {
    id: String,
    deps: Vec<RawDep>,
}

#[derive(Deserialize)]
struct RawDep {
    pkg: String,
    dep_kinds: Vec<RawDepKind>,
}

#[derive(Deserialize)]
struct RawDepKind {
    kind: Option<String>,
}

impl Graph {
    /// Asks `cargo` for the graph of the workspace rooted at, or containing, `directory`, as a
    /// build for `target` resolves it, with the lockfile held as it is.
    pub fn read(cargo: &OsStr, directory: &Path, target: Target) -> Result<Self, MetadataFault> {
        let output = Command::new(cargo)
            .args(["metadata", "--format-version", "1", "--locked"])
            .args(["--filter-platform", target.triple()])
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

    /// Reads `cargo metadata`'s output, resolve and all.
    pub fn parse(json: &str) -> Result<Self, MetadataFault> {
        let raw: RawMetadata =
            serde_json::from_str(json).map_err(|error| MetadataFault::NotTheShape {
                line: error.line(),
                column: error.column(),
            })?;
        let mut packages = BTreeMap::new();
        for package in raw.packages {
            let name = CrateName::parse(&package.name)
                .ok_or_else(|| MetadataFault::Name(package.name.clone()))?;
            let version =
                PackageVersion::parse(&package.version).ok_or_else(|| MetadataFault::Version {
                    name: name.clone(),
                    version: package.version.clone(),
                })?;
            packages.insert(
                PackageId(package.id),
                Package {
                    name,
                    version,
                    license: package.license,
                    authors: package.authors,
                    repository: package.repository,
                    directory: package
                        .manifest_path
                        .parent()
                        .map_or_else(PathBuf::new, Path::to_path_buf),
                },
            );
        }
        let known = |id: String| {
            let id = PackageId(id);
            if packages.contains_key(&id) {
                Ok(id)
            } else {
                Err(MetadataFault::Unresolved(id.0))
            }
        };
        let members = raw
            .workspace_members
            .into_iter()
            .map(known)
            .collect::<Result<BTreeSet<_>, _>>()?;
        let resolve = raw.resolve.ok_or(MetadataFault::NoResolve)?;
        let mut edges = BTreeMap::new();
        for node in resolve.nodes {
            let from = known(node.id)?;
            let from_name = packages
                .get(&from)
                .map(|package| package.name.clone())
                .ok_or_else(|| MetadataFault::Unresolved(from.0.clone()))?;
            let mut out = Vec::new();
            for dependency in node.deps {
                let to = known(dependency.pkg)?;
                let mut kinds = BTreeSet::new();
                for kind in dependency.dep_kinds {
                    kinds.insert(match kind.kind.as_deref() {
                        None => DependencyKind::Normal,
                        Some("dev") => DependencyKind::Development,
                        Some("build") => DependencyKind::Build,
                        Some(other) => {
                            return Err(MetadataFault::Kind {
                                name: from_name,
                                kind: other.to_owned(),
                            })
                        }
                    });
                }
                out.push(Edge { to, kinds });
            }
            edges.insert(from, out);
        }
        Ok(Self {
            root: raw.workspace_root,
            packages,
            members,
            edges,
        })
    }

    /// The package `id` names. Every id the graph gives out names one.
    pub fn package(&self, id: &PackageId) -> Option<&Package> {
        self.packages.get(id)
    }

    /// The workspace's members, by id.
    pub fn members(&self) -> impl Iterator<Item = (&PackageId, &Package)> {
        self.members
            .iter()
            .filter_map(|id| self.packages.get(id).map(|package| (id, package)))
    }

    /// Whether `id` is one of the workspace's members.
    pub fn is_member(&self, id: &PackageId) -> bool {
        self.members.contains(id)
    }

    /// What `id` depends on, and how.
    pub fn edges(&self, id: &PackageId) -> &[Edge] {
        self.edges.get(id).map_or(&[], Vec::as_slice)
    }

    /// Every package reachable from `starts` by dependencies of `kind`, the starts included.
    pub fn closure(
        &self,
        starts: impl IntoIterator<Item = PackageId>,
        kind: DependencyKind,
    ) -> BTreeSet<PackageId> {
        let mut seen = BTreeSet::new();
        let mut stack: Vec<PackageId> = starts.into_iter().collect();
        while let Some(current) = stack.pop() {
            if seen.contains(&current) {
                continue;
            }
            stack.extend(
                self.edges(&current)
                    .iter()
                    .filter(|edge| edge.kinds.contains(&kind))
                    .map(|edge| edge.to.clone()),
            );
            seen.insert(current);
        }
        seen
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// `cargo metadata`'s output for a graph: each package's name and the directory its manifest is
    /// in, the members among them, and each edge with its kind, `None` for a normal dependency.
    pub(crate) fn metadata(
        packages: &[(&str, &str)],
        members: &[&str],
        edges: &[(&str, &str, Option<&str>)],
    ) -> String {
        metadata_value(packages, members, edges).to_string()
    }

    fn metadata_value(
        packages: &[(&str, &str)],
        members: &[&str],
        edges: &[(&str, &str, Option<&str>)],
    ) -> serde_json::Value {
        let id = |name: &str| format!("{name}-id");
        let listed: Vec<serde_json::Value> = packages
            .iter()
            .map(|(name, directory)| {
                serde_json::json!({
                    "id": id(name), "name": name, "version": "1.0.0", "license": "MIT",
                    "authors": [], "repository": null,
                    "manifest_path": format!("{directory}/Cargo.toml"),
                })
            })
            .collect();
        let nodes: Vec<serde_json::Value> = packages
            .iter()
            .map(|(name, _)| {
                let deps: Vec<serde_json::Value> = edges
                    .iter()
                    .filter(|(from, _, _)| from == name)
                    .map(|(_, to, kind)| {
                        serde_json::json!({ "pkg": id(to), "dep_kinds": [{ "kind": kind, "target": null }] })
                    })
                    .collect();
                serde_json::json!({ "id": id(name), "deps": deps })
            })
            .collect();
        serde_json::json!({
            "packages": listed,
            "workspace_members": members.iter().map(|name| id(name)).collect::<Vec<_>>(),
            "workspace_root": "/w",
            "resolve": { "nodes": nodes, "root": null },
            "version": 1,
        })
    }

    fn names(graph: &Graph, ids: &BTreeSet<PackageId>) -> Vec<String> {
        ids.iter()
            .filter_map(|id| graph.package(id))
            .map(|package| package.name.as_str().to_owned())
            .collect()
    }

    #[test]
    fn a_closure_follows_one_kind_of_dependency_alone() {
        let graph = Graph::parse(&metadata(
            &[
                ("app", "/w/app"),
                ("lib", "/r/lib"),
                ("probe", "/r/probe"),
                ("test", "/r/test"),
            ],
            &["app"],
            &[
                ("app", "lib", None),
                ("lib", "probe", Some("build")),
                ("app", "test", Some("dev")),
            ],
        ))
        .expect("the graph reads");
        let app = PackageId(String::from("app-id"));
        assert!(graph.is_member(&app));
        assert_eq!(
            names(
                &graph,
                &graph.closure([app.clone()], DependencyKind::Normal)
            ),
            ["app", "lib"]
        );
        assert_eq!(
            names(&graph, &graph.closure([app], DependencyKind::Development)),
            ["app", "test"]
        );
        let lib = graph.package(&PackageId(String::from("lib-id")));
        assert_eq!(
            lib.map(|package| package.directory.clone()),
            Some(PathBuf::from("/r/lib"))
        );
    }

    #[test]
    fn an_edge_to_a_package_it_does_not_list_is_refused() {
        let text = metadata(&[("app", "/w/app")], &["app"], &[("app", "gone", None)]);
        assert_eq!(
            Graph::parse(&text),
            Err(MetadataFault::Unresolved(String::from("gone-id")))
        );
        let text = metadata(
            &[("app", "/w/app")],
            &["app"],
            &[("app", "app", Some("optional"))],
        );
        assert!(matches!(
            Graph::parse(&text),
            Err(MetadataFault::Kind { .. })
        ));
        let mut unresolved = metadata_value(&[("app", "/w/app")], &["app"], &[]);
        unresolved["resolve"] = serde_json::Value::Null;
        assert_eq!(
            Graph::parse(&unresolved.to_string()),
            Err(MetadataFault::NoResolve)
        );
    }
}
