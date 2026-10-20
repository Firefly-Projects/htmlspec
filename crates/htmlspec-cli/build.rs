//! Records the revision the binary is built from, for `--version`.
//!
//! The point is that a diagnostic in a bug report identifies a specific build rather
//! than a release line, so a maintainer can check out exactly what the reporter ran.
//!
//! The revision is taken from the first of these that has one:
//!
//! 1. `HTMLSPEC_REVISION`, set by a packaging system that knows the revision without a
//!    `.git` directory. `flake.nix` sets it from the flake's own revision, and the release
//!    workflow from the commit it builds.
//! 2. `.cargo_vcs_info.json`, which cargo writes into every packaged crate with the commit it
//!    was packaged from. So a build from crates.io names the release's commit.
//! 3. Git, for a build in a checkout.
//!
//! The packaged record comes before Git because a packaged crate's source can sit inside an
//! unrelated repository: `cargo install` unpacks it under the cargo home, and a home directory
//! kept in Git would otherwise lend the build that repository's commit. A checkout holds no
//! `.cargo_vcs_info.json`, because cargo writes it only into the package, so the order changes
//! nothing there.
//!
//! When none has a revision, the binary reports `unknown`. That is deliberate: failing the
//! build because Git is absent would make the packaging requirement unmeetable, and an
//! honest "unknown" is more useful than a fabricated hash.
//!
//! The variable is *always* set, even to `unknown`, so the binary can read it with a
//! plain `env!`. Leaving it unset would push the absent case into the source, where
//! `option_env!` cannot express it: `env!` in a match arm is expanded whichever arm is
//! taken, so the build would fail exactly when Git was missing.

use std::path::Path;
use std::process::Command;

/// The length a commit hash is shortened to, whichever source names it.
const SHORT_HASH: usize = 12;

/// The file cargo writes into a packaged crate, naming the commit it was packaged from.
const VCS_INFO: &str = ".cargo_vcs_info.json";

fn main() {
    println!("cargo:rerun-if-env-changed=HTMLSPEC_REVISION");

    // An externally supplied revision wins, so a packaging system that knows the
    // revision without a `.git` directory can state it.
    if let Ok(supplied) = std::env::var("HTMLSPEC_REVISION") {
        if !supplied.trim().is_empty() {
            println!("cargo:rustc-env=HTMLSPEC_REVISION={}", supplied.trim());
            return;
        }
    }

    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest directory");
    let vcs_info = Path::new(&manifest).join(VCS_INFO);
    if vcs_info.is_file() {
        println!("cargo:rerun-if-changed={}", vcs_info.display());
        let revision = packaged_revision(&vcs_info).unwrap_or_else(|| String::from("unknown"));
        println!("cargo:rustc-env=HTMLSPEC_REVISION={revision}");
        return;
    }

    // Rerun when the checked-out commit moves. Without this the recorded revision
    // would be whatever it was when the crate last compiled, which is worse than
    // reporting nothing: it would be confidently wrong.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs");

    let revision = git_revision().unwrap_or_else(|| String::from("unknown"));
    println!("cargo:rustc-env=HTMLSPEC_REVISION={revision}");
}

/// The commit a packaged crate records, shortened, with a `-dirty` marker when cargo
/// packaged it from a tree with uncommitted changes.
///
/// cargo writes `{"git": {"sha1": "<hash>", "dirty": true}, "path_in_vcs": "<path>"}`, and
/// writes `dirty` only when it is true. A record without a whole hexadecimal hash yields
/// nothing, so the build reports `unknown` rather than whatever the field held.
fn packaged_revision(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let record: serde_json::Value = serde_json::from_str(&text).ok()?;
    let git = record.get("git")?;
    let hash = git.get("sha1")?.as_str()?;
    if hash.len() < SHORT_HASH || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let short = &hash[..SHORT_HASH];
    let dirty = git.get("dirty").and_then(serde_json::Value::as_bool) == Some(true);
    Some(if dirty {
        format!("{short}-dirty")
    } else {
        short.to_owned()
    })
}

/// The short commit hash, with a `-dirty` marker when the tree has uncommitted
/// changes.
///
/// The marker matters more than the hash: a build from a modified tree is not the
/// revision it names, and a version that hid that would send a maintainer to the
/// wrong source.
fn git_revision() -> Option<String> {
    let hash = run(&["rev-parse", &format!("--short={SHORT_HASH}"), "HEAD"])?;
    let dirty = run(&["status", "--porcelain"]).is_some_and(|out| !out.is_empty());
    Some(if dirty { format!("{hash}-dirty") } else { hash })
}

fn run(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_owned())
}
