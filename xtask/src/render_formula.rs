//! `cargo xtask render-formula --sums SHA256SUMS`: write the Homebrew formula for a release from
//! that release's `SHA256SUMS`.
//!
//! The formula names one tarball per platform and the digest Homebrew checks it against. A digest
//! copied by hand is one that can disagree with the file, and a formula whose digest disagrees is an
//! install that fails for everyone. So the formula is rendered from the `SHA256SUMS` the release's
//! sums job wrote: `packaging/homebrew/htmlspec.rb.in` holds placeholders, and this fills the
//! version and each target's digest from that file and nothing else.
//!
//! It refuses a `SHA256SUMS` that lacks one of the four targets, names two versions, names a file
//! that is not a release tarball, or names a version that is not a release's as written; a template
//! whose tarballs are not exactly the four targets; and a rendering that leaves any placeholder
//! unfilled.
//!
//! The template's `license` must name what the binary credits: the `license` of
//! `crates/htmlspec-assets`, for the libraries the binary embeds, joined with the licenses
//! `crates/htmlspec-cli/CRATE-LICENSES` names in its header, for the Rust crates it is built from.
//! [`template_faults`] holds that, and `cargo xtask crate-notices --check` asks it, in the suite's
//! `notices` stage, on every pull request and on a tag before anything publishes. The renderer does
//! not ask it again: the assets crate's `license` is stated in a manifest, which only cargo reads,
//! and the jobs that render, the release workflow's sums and formula jobs, install no toolchain.
//!
//! `--base-url` replaces the address the tarballs are fetched from, which is the version's GitHub
//! release download directory unless it is given. `{version}` in it is replaced by the version. The
//! formula check points it at the directory the tarballs were packed into.
//!
//! It reads the template in the directory it runs in, the repository's root. Exit codes: 0 with
//! the formula on standard output, 1 when the sums or the template are refused, and 2 when a file
//! cannot be read.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;
use std::process::ExitCode;

use htmlspec_cli::version::ReleaseVersion;

use crate::facts::{Facts, MacosJobs, Target};
use crate::report::Exit;

/// The template, from the repository's root.
pub const TEMPLATE: &str = "packaging/homebrew/htmlspec.rb.in";

/// The header line of `CRATE-LICENSES` that names every license it credits.
pub const CREDITED_PREFIX: &str = "Credited licenses: ";

/// What every tarball's address in the template begins with, before its target.
const TARBALL: &str = "url \"@BASE_URL@/htmlspec-@VERSION@-";

/// The licenses `CRATE-LICENSES`' header says the binary's Rust crates are credited under, or
/// `None` where it has no such line.
pub fn credited_licenses(crate_licenses: &str) -> Option<Vec<String>> {
    crate_licenses.lines().find_map(|line| {
        line.strip_prefix(CREDITED_PREFIX).map(|names| {
            names
                .split(" AND ")
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .collect()
        })
    })
}

/// A SHA-256 digest, as `sha256sum` writes it: 64 lowercase hexadecimal digits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Digest(String);

impl Digest {
    pub fn parse(text: &str) -> Option<Self> {
        (text.len() == 64
            && text
                .chars()
                .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character)))
        .then(|| Self(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A release's `SHA256SUMS`, read: one version, and a digest for every target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sums {
    pub version: ReleaseVersion,
    pub digests: BTreeMap<Target, Digest>,
}

/// Why a `SHA256SUMS` is not a release's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SumsFault {
    /// A line that is not a digest, two spaces, and a release tarball's name.
    Line { number: usize, line: String },
    /// A tarball for a target a release does not build.
    Unbuilt { target: String },
    /// Tarballs of two versions.
    Versions {
        first: ReleaseVersion,
        second: ReleaseVersion,
    },
    /// Two tarballs for one target.
    Twice(Target),
    /// No tarball for these targets.
    Lacks(Vec<Target>),
}

impl fmt::Display for SumsFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line { number, line } => write!(
                formatter,
                "line {number} of SHA256SUMS is not a digest and a release tarball: {line:?}"
            ),
            Self::Unbuilt { target } => write!(
                formatter,
                "SHA256SUMS names {target}, which a release does not build"
            ),
            Self::Versions { first, second } => {
                write!(formatter, "SHA256SUMS names versions {first} and {second}")
            }
            Self::Twice(target) => write!(formatter, "SHA256SUMS names {target} twice"),
            Self::Lacks(targets) => write!(formatter, "SHA256SUMS lacks {}", listed(targets)),
        }
    }
}

/// `targets` joined as a sentence lists them.
fn listed(targets: &[Target]) -> String {
    targets
        .iter()
        .map(|target| target.triple())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A release tarball's name, `htmlspec-X.Y.Z-TARGET.tar.gz`, as its version and its target's
/// triple, where the triple is lowercase letters, digits, `_`, and `-`.
pub fn tarball_name(name: &str) -> Option<(ReleaseVersion, &str)> {
    let middle = name.strip_prefix("htmlspec-")?.strip_suffix(".tar.gz")?;
    let (version, target) = middle.split_once('-')?;
    let triple = |character: char| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '_' | '-')
    };
    if target.is_empty() || !target.chars().all(triple) {
        return None;
    }
    Some((ReleaseVersion::parse(version).ok()?, target))
}

/// The version and each target's digest, from `SHA256SUMS` in `sha256sum`'s format, which must
/// name every one of `targets`.
pub fn read_sums(text: &str, targets: &[Target]) -> Result<Sums, SumsFault> {
    let mut version: Option<ReleaseVersion> = None;
    let mut digests = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let refused = || SumsFault::Line {
            number: index + 1,
            line: line.to_owned(),
        };
        let (digest, name) = match trimmed.split_at_checked(64) {
            Some((digest, rest)) => (digest, rest.strip_prefix("  ").ok_or_else(refused)?),
            None => return Err(refused()),
        };
        let digest = Digest::parse(digest).ok_or_else(refused)?;
        let (named, triple) = tarball_name(name).ok_or_else(refused)?;
        let target = Target::from_triple(triple)
            .filter(|target| targets.contains(target))
            .ok_or_else(|| SumsFault::Unbuilt {
                target: triple.to_owned(),
            })?;
        if let Some(first) = version.filter(|first| *first != named) {
            return Err(SumsFault::Versions {
                first,
                second: named,
            });
        }
        if digests.contains_key(&target) {
            return Err(SumsFault::Twice(target));
        }
        version = Some(named);
        digests.insert(target, digest);
    }
    let missing: Vec<Target> = targets
        .iter()
        .copied()
        .filter(|target| !digests.contains_key(target))
        .collect();
    match version {
        Some(version) if missing.is_empty() => Ok(Sums { version, digests }),
        _ => Err(SumsFault::Lacks(missing)),
    }
}

/// Where the template disagrees with the four targets, or with what the binary credits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateFault {
    /// Its tarballs are for these targets, and not exactly the ones a release builds.
    Tarballs {
        named: Vec<String>,
        built: Vec<Target>,
    },
    /// `htmlspec-assets` declares no license, or one not joined by `AND` alone.
    AssetsLicense(Option<String>),
    /// `CRATE-LICENSES` names no credited licenses.
    NoCredited,
    /// Its `license all_of:` names other licenses than the binary credits.
    License {
        stated: Vec<String>,
        wanted: Vec<String>,
        declared: String,
        credited: Vec<String>,
    },
}

/// A list as the Python tools wrote one, `['MIT', 'ISC']`, so a reader of the old message reads
/// the same one.
fn quoted(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| format!("'{item}'")).collect();
    format!("[{}]", quoted.join(", "))
}

impl fmt::Display for TemplateFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tarballs { named, built } => write!(
                formatter,
                "the template fetches tarballs for {}, and a release builds {}",
                if named.is_empty() {
                    String::from("no target")
                } else {
                    named.join(", ")
                },
                listed(built)
            ),
            Self::AssetsLicense(declared) => write!(
                formatter,
                "crates/htmlspec-assets/Cargo.toml declares {}, and this tool compares only \
                 licenses joined by AND, which is what the formula's all_of states",
                declared.as_ref().map_or_else(
                    || String::from("no license"),
                    |license| format!("license {license:?}")
                )
            ),
            Self::NoCredited => write!(
                formatter,
                "crates/htmlspec-cli/CRATE-LICENSES has no '{}' line; run cargo xtask \
                 crate-notices to write it",
                CREDITED_PREFIX.trim()
            ),
            Self::License {
                stated,
                wanted,
                declared,
                credited,
            } => write!(
                formatter,
                "the template's license is all_of {}, and the binary credits {}: \
                 crates/htmlspec-assets/Cargo.toml declares {declared} for what it embeds, and \
                 CRATE-LICENSES credits {} for the crates it is built from",
                quoted(stated),
                quoted(wanted),
                credited.join(" AND ")
            ),
        }
    }
}

/// The targets the template's tarball addresses name, in the order it names them.
fn template_targets(template: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find(TARBALL) {
        rest = &rest[start + TARBALL.len()..];
        let length = rest
            .find(|character: char| {
                !(character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || matches!(character, '_' | '-'))
            })
            .unwrap_or(rest.len());
        if length > 0 && rest[length..].starts_with(".tar.gz\"") {
            found.push(rest[..length].to_owned());
        }
    }
    found
}

/// The licenses the template's `license all_of: [...]` line states, in its order: each quoted
/// string between its brackets.
fn template_license(template: &str) -> Vec<String> {
    let inside = template.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix("license all_of: [")?;
        let (inside, after) = rest.split_once(']')?;
        after.trim().is_empty().then_some(inside)
    });
    let mut stated = Vec::new();
    let mut rest = inside.unwrap_or_default();
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        match after.find('"') {
            Some(0) => rest = after,
            Some(close) => {
                stated.push(after[..close].to_owned());
                rest = &after[close + 1..];
            }
            None => break,
        }
    }
    stated
}

/// Whether the template's tarballs are exactly the targets a release builds.
pub fn tarball_fault(template: &str, targets: &[Target]) -> Option<TemplateFault> {
    let named = template_targets(template);
    let mut sorted_named = named.clone();
    sorted_named.sort();
    let mut built: Vec<String> = targets.iter().map(|target| target.to_string()).collect();
    built.sort();
    (sorted_named != built).then(|| TemplateFault::Tarballs {
        named,
        built: targets.to_vec(),
    })
}

/// Where the template disagrees with the four targets, or with what the binary credits: the
/// license `htmlspec-assets` declares for what it embeds, and the licenses `crate_licenses`, the
/// text of `CRATE-LICENSES`, credits for the crates it is built from.
pub fn template_faults(
    template: &str,
    assets_license: Option<&str>,
    crate_licenses: &str,
    targets: &[Target],
) -> Vec<TemplateFault> {
    let mut faults: Vec<TemplateFault> = tarball_fault(template, targets).into_iter().collect();
    let declared = match assets_license {
        Some(declared) if !declared.contains('(') && !declared.contains(" OR ") => declared,
        other => {
            faults.push(TemplateFault::AssetsLicense(other.map(str::to_owned)));
            return faults;
        }
    };
    let Some(credited) = credited_licenses(crate_licenses) else {
        faults.push(TemplateFault::NoCredited);
        return faults;
    };
    let wanted: BTreeSet<String> = declared
        .split(" AND ")
        .map(str::to_owned)
        .chain(credited.iter().cloned())
        .collect();
    let wanted: Vec<String> = wanted.into_iter().collect();
    let stated = template_license(template);
    let mut sorted_stated = stated.clone();
    sorted_stated.sort();
    if sorted_stated != wanted {
        faults.push(TemplateFault::License {
            stated,
            wanted,
            declared: declared.to_owned(),
            credited,
        });
    }
    faults
}

/// Where the tarballs are fetched from: an address, in which `{version}` stands for the version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseUrl(String);

impl BaseUrl {
    pub fn new(address: String) -> Self {
        Self(address)
    }

    /// The version's download directory on the public repository's GitHub releases.
    pub fn releases(facts: &Facts) -> Self {
        Self(format!(
            "{}/releases/download/v{{version}}",
            facts.public_repository()
        ))
    }

    fn for_version(&self, version: ReleaseVersion) -> String {
        self.0
            .replace("{version}", &version.to_string())
            .trim_end_matches('/')
            .to_owned()
    }
}

/// Why no formula was rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Template(TemplateFault),
    Sums(SumsFault),
    /// The rendering leaves these placeholders unfilled.
    Unfilled(Vec<String>),
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Template(fault) => fault.fmt(formatter),
            Self::Sums(fault) => fault.fmt(formatter),
            Self::Unfilled(left) => {
                write!(
                    formatter,
                    "the template leaves {} unfilled",
                    left.join(", ")
                )
            }
        }
    }
}

/// The placeholders `@NAME@` left in `text`, NAME being capitals, digits, and `_`, each once,
/// sorted.
fn placeholders(text: &str) -> Vec<String> {
    let mut found = BTreeSet::new();
    let mut rest = text;
    while let Some(open) = rest.find('@') {
        let after = &rest[open + 1..];
        let length = after
            .find(|character: char| {
                !(character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_')
            })
            .unwrap_or(after.len());
        if length > 0 && after[length..].starts_with('@') {
            found.insert(format!("@{}@", &after[..length]));
            rest = &after[length + 1..];
        } else {
            rest = after;
        }
    }
    found.into_iter().collect()
}

/// The placeholder a target's digest fills.
fn digest_placeholder(target: Target) -> String {
    format!(
        "@SHA256_{}@",
        target.triple().to_ascii_uppercase().replace('-', "_")
    )
}

/// The formula for `sums`, fetching from `base_url`.
pub fn render(template: &str, sums: &Sums, base_url: &BaseUrl) -> Result<String, Refusal> {
    let mut text = template
        .replace("@BASE_URL@", &base_url.for_version(sums.version))
        .replace("@VERSION@", &sums.version.to_string());
    for (target, digest) in &sums.digests {
        text = text.replace(&digest_placeholder(*target), digest.as_str());
    }
    let left = placeholders(&text);
    if left.is_empty() {
        Ok(text)
    } else {
        Err(Refusal::Unfilled(left))
    }
}

/// The formula from the template's text and a `SHA256SUMS`'s, for the targets a release builds.
pub fn formula(
    template: &str,
    sums: &str,
    base_url: &BaseUrl,
    targets: &[Target],
) -> Result<String, Refusal> {
    if let Some(fault) = tarball_fault(template, targets) {
        return Err(Refusal::Template(fault));
    }
    let sums = read_sums(sums, targets).map_err(Refusal::Sums)?;
    render(template, &sums, base_url)
}

/// Why the formula's inputs could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable {
    pub path: String,
    pub kind: std::io::ErrorKind,
}

impl fmt::Display for Unreadable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cannot read {}: {}", self.path, self.kind)
    }
}

/// Why rendering stopped: the inputs could not be read, or the rendering was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stopped {
    CouldNotAsk(String),
    Refused(Refusal),
}

impl Stopped {
    /// How the run ends: 1 for a refusal, 2 when the inputs could not be read.
    pub fn exit(&self) -> Exit {
        match self {
            Self::CouldNotAsk(_) => Exit::CouldNotAsk,
            Self::Refused(_) => Exit::Faults,
        }
    }
}

impl fmt::Display for Stopped {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CouldNotAsk(reason) => formatter.write_str(reason),
            Self::Refused(refusal) => refusal.fmt(formatter),
        }
    }
}

/// The formula from the template in the current directory and the `SHA256SUMS` at `sums`.
pub fn from_files(sums: &Path, base_url: Option<BaseUrl>) -> Result<String, Stopped> {
    let facts = Facts::read()
        .map_err(|fault| Stopped::CouldNotAsk(format!("xtask/release-facts.json {fault}")))?;
    let read = |path: &Path| {
        std::fs::read_to_string(path).map_err(|error| {
            Stopped::CouldNotAsk(
                Unreadable {
                    path: path.display().to_string(),
                    kind: error.kind(),
                }
                .to_string(),
            )
        })
    };
    let template = read(Path::new(TEMPLATE))?;
    let sums = read(sums)?;
    let base_url = base_url.unwrap_or_else(|| BaseUrl::releases(&facts));
    formula(&template, &sums, &base_url, &facts.targets(MacosJobs::Run)).map_err(Stopped::Refused)
}

/// Prints the formula for the `SHA256SUMS` at `sums`.
pub fn run(sums: &Path, base_url: Option<String>) -> ExitCode {
    match from_files(sums, base_url.map(BaseUrl::new)) {
        Ok(formula) => {
            // Written as it is, so the formula's bytes are the template's, filled.
            print!("{formula}");
            ExitCode::SUCCESS
        }
        Err(stopped) => {
            eprintln!("render-formula: {stopped}");
            ExitCode::from(stopped.exit().code())
        }
    }
}

#[cfg(test)]
mod tests;
