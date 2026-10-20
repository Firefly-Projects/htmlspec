//! Records the pinned and resolved `html5ever` versions so a test can compare
//! them against the version the parse-error enumeration was reviewed against.
//!
//! Cargo exposes no dependency versions to a build script, so both values are
//! read from the workspace manifest and lockfile. Absence is reported as the
//! literal `unknown`, which fails the tripwire test rather than passing it.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .to_path_buf()
}

/// Reads the exact requirement from `[workspace.dependencies] html5ever = "=x.y.z"`.
fn pinned_version(manifest: &str) -> Option<String> {
    manifest
        .lines()
        .find_map(|line| line.strip_prefix("html5ever = \"="))
        .and_then(|rest| rest.split('"').next())
        .map(str::to_owned)
}

/// Reads the resolved version from the lockfile's `html5ever` package entry.
fn locked_version(lock: &str) -> Option<String> {
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "name = \"html5ever\"" {
            return lines
                .next()
                .and_then(|v| v.trim().strip_prefix("version = \""))
                .and_then(|v| v.split('"').next())
                .map(str::to_owned);
        }
    }
    None
}

fn main() {
    let root = workspace_root();
    let manifest_path = root.join("Cargo.toml");
    let lock_path = root.join("Cargo.lock");

    let pinned = std::fs::read_to_string(&manifest_path)
        .ok()
        .as_deref()
        .and_then(pinned_version)
        .unwrap_or_else(|| "unknown".to_owned());
    let locked = std::fs::read_to_string(&lock_path)
        .ok()
        .as_deref()
        .and_then(locked_version)
        .unwrap_or_else(|| "unknown".to_owned());

    println!("cargo:rustc-env=HTMLSPEC_PINNED_HTML5EVER_VERSION={pinned}");
    println!("cargo:rustc-env=HTMLSPEC_LOCKED_HTML5EVER_VERSION={locked}");
    println!("cargo:rerun-if-changed={}", manifest_path.display());
    println!("cargo:rerun-if-changed={}", lock_path.display());
    println!("cargo:rerun-if-changed=build.rs");
}
