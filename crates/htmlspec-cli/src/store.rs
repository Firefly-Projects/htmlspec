//! Standalone projects registered on this machine.
//!
//! A store is an HTMLSpec project held in its own repository, registered under an identifier so
//! that operations elsewhere can act on it. The registrations are **machine-level and live
//! outside every project**, which is the whole point: a project that recorded where other
//! projects live would make one repository's contents depend on another's location, and a
//! checkout on a second machine would carry paths that do not exist there.
//!
//! # What this module does not do
//!
//! It does not fall back. Where a registration names a location that no longer holds a project,
//! every operation reports that and acts on nothing. Falling back to the working directory would
//! mean acting on a different project than the one named, and it would look like success — which
//! is the failure the reporting requirement exists to prevent.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::project::ProjectRoot;

/// Where the registrations live.
///
/// `XDG_CONFIG_HOME` where set, and `~/.config` otherwise. A config directory rather than a data
/// one: this is a small map a person edits and reads, not state the program accumulates.
pub fn registry_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("htmlspec").join("stores.json"))
}

/// Every registration, by identifier.
///
/// A missing or unreadable registry is an empty map rather than an error. Nothing has been
/// registered yet is the ordinary first state, and it is not distinguishable from a registry
/// nobody has created in any way that would change what a caller does.
pub fn read() -> BTreeMap<String, PathBuf> {
    let Some(path) = registry_path() else {
        return BTreeMap::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    serde_json::from_str::<BTreeMap<String, PathBuf>>(&text).unwrap_or_default()
}

/// Writes the registrations, creating the directory if it is absent.
pub fn write(registrations: &BTreeMap<String, PathBuf>) -> std::io::Result<()> {
    let path = registry_path().ok_or_else(|| {
        std::io::Error::other("no configuration directory: set XDG_CONFIG_HOME or HOME")
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(registrations).map_err(std::io::Error::other)?;
    std::fs::write(path, format!("{json}\n"))
}

/// Why a store selection could not be resolved.
pub enum Unresolvable {
    /// No registration of that identifier.
    Unregistered { id: String, known: Vec<String> },
    /// A registration whose location no longer holds a project.
    Gone { id: String, path: PathBuf },
}

impl std::fmt::Display for Unresolvable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unregistered { id, known } if known.is_empty() => write!(
                f,
                "no store registered as {id}, and nothing is registered on this machine. \
                 Register one with `htmlspec store add <id> <path>`"
            ),
            Self::Unregistered { id, known } => write!(
                f,
                "no store registered as {id}. Registered: {}",
                known.join(", ")
            ),
            Self::Gone { id, path } => write!(
                f,
                "the store {id} is registered at {}, which no longer holds a project. \
                 Re-register it or remove it — this does not fall back to the project you are \
                 standing in, because acting on a different project than the one you named \
                 would look like success",
                path.display()
            ),
        }
    }
}

/// The project a selection names.
///
/// `None` selects the project containing the working directory, which is the unscoped case.
pub fn resolve(selected: Option<&str>) -> Result<Option<ProjectRoot>, Unresolvable> {
    let Some(id) = selected else {
        return Ok(None);
    };
    let registrations = read();
    let Some(path) = registrations.get(id) else {
        return Err(Unresolvable::Unregistered {
            id: id.to_owned(),
            known: registrations.keys().cloned().collect(),
        });
    };
    ProjectRoot::resolve(path)
        .map(Some)
        .map_err(|_| Unresolvable::Gone {
            id: id.to_owned(),
            path: path.clone(),
        })
}

/// The project an operation acts on, given a selection.
///
/// Split from [`resolve`] so the unscoped fallback is in one place and reads as the deliberate
/// case rather than as an `unwrap_or_else` at each call site.
pub fn project_for(selected: Option<&str>) -> Result<ProjectRoot, String> {
    match resolve(selected) {
        Ok(Some(project)) => Ok(project),
        Ok(None) => ProjectRoot::resolve(Path::new(".")).map_err(|error| {
            format!("{error}. Name a registered store with --store, or stand in a project")
        }),
        Err(unresolvable) => Err(unresolvable.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An unregistered identifier names what is registered, so a contributor who mistyped sees
    /// the list they were choosing from.
    #[test]
    fn an_unregistered_id_names_what_is_registered() {
        let reported = Unresolvable::Unregistered {
            id: String::from("wrong"),
            known: vec![String::from("specs"), String::from("other")],
        }
        .to_string();
        assert!(reported.contains("wrong"), "{reported}");
        assert!(reported.contains("specs, other"), "{reported}");
    }

    /// With nothing registered, the report says so and names the operation that registers one.
    /// "Registered: " followed by nothing is the failure this avoids.
    #[test]
    fn an_empty_registry_says_so() {
        let reported = Unresolvable::Unregistered {
            id: String::from("wrong"),
            known: Vec::new(),
        }
        .to_string();
        assert!(reported.contains("nothing is registered"), "{reported}");
        assert!(reported.contains("store add"), "{reported}");
    }

    /// A registration whose project is gone says it does not fall back, because falling back is
    /// the plausible-looking wrong behavior.
    #[test]
    fn a_gone_store_says_it_does_not_fall_back() {
        let reported = Unresolvable::Gone {
            id: String::from("moved"),
            path: PathBuf::from("/nowhere"),
        }
        .to_string();
        assert!(reported.contains("does not fall back"), "{reported}");
        assert!(reported.contains("/nowhere"), "{reported}");
    }
}
