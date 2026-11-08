//! `cargo xtask pack-release TARGET BINARY OUT`: pack a built `htmlspec` binary as a release
//! tarball; and `cargo xtask pack-release --sums OUT`: write the release's `SHA256SUMS`.
//!
//! # What a tarball holds
//!
//! `htmlspec-X.Y.Z-<target>.tar.gz` holds one directory of the same name, and in it the binary, the
//! root `LICENSE`, `THIRD-PARTY-LICENSES` from `crates/htmlspec-assets`, and `CRATE-LICENSES` from
//! `crates/htmlspec-cli`. The binary embeds the vendored Mermaid and Vega libraries and is built
//! from the Rust crates `CRATE-LICENSES` credits, and their licenses ask that their notices travel
//! with the software, so the texts go beside the binary rather than only into the repository.
//!
//! The version is the one `[workspace.package]` declares, as Cargo read it when it built this tool,
//! [`facts::release_version`], the same one the release workflow's verify job has already compared
//! with the tag, so the tarball's name, the binary's `--version`, and the tag agree. The release
//! workflow packs in its sums job, which installs no toolchain, with the tool its verify job built
//! from the same commit.
//!
//! # Why the bytes are fixed
//!
//! Every entry has the same owner, mode, and time, and the gzip header carries no time or name. So
//! packing the same binary twice gives the same tarball and the same digest, and `SHA256SUMS`
//! describes the binary rather than the moment it was packed. The binary's mode is set in its
//! entry, not read from the file, because a binary a workflow artifact carried has lost its
//! executable bit.
//!
//! # `SHA256SUMS`
//!
//! One line for each release tarball in OUT, in `sha256sum`'s format, in the order the release's
//! facts list the targets. A name that is not a release tarball's, for a target a release builds,
//! is refused, and so are tarballs of two versions. A target with no tarball is named on standard
//! error, and the file is still written, because a pull request builds the Linux targets alone.
//!
//! It reads the license texts in the directory it runs in, the repository's root. Exit codes: 0
//! with the path written on standard output, 1 when OUT holds what is not a release's tarballs, and
//! 2 when a file cannot be read or written, or the binary is not a file.

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use flate2::write::GzEncoder;
use flate2::{Compression, GzBuilder};
use htmlspec_cli::version::ReleaseVersion;
use sha2::{Digest, Sha256};

use crate::facts::{self, Facts, MacosJobs, Target};
use crate::render_formula::tarball_name;
use crate::report::{Exit, Report};

/// The file `--sums` writes.
pub const SUMS: &str = "SHA256SUMS";

/// The license texts that travel beside the binary: each one's name in the tarball, and where it
/// is read from, from the repository's root.
pub const NOTICES: [(&str, &str); 3] = [
    ("LICENSE", "LICENSE"),
    (
        "THIRD-PARTY-LICENSES",
        "crates/htmlspec-assets/THIRD-PARTY-LICENSES",
    ),
    ("CRATE-LICENSES", "crates/htmlspec-cli/CRATE-LICENSES"),
];

/// The mode an entry of a tarball is given, whatever the file's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// 755: the binary, and the directory that holds it.
    Executable,
    /// 644: each license text.
    Readable,
}

impl Mode {
    /// The permission bits the entry's header carries.
    pub fn bits(self) -> u32 {
        match self {
            Self::Executable => 0o755,
            Self::Readable => 0o644,
        }
    }
}

/// One file of a tarball: its name within the tarball's directory, its bytes, and its mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub bytes: Vec<u8>,
    pub mode: Mode,
}

/// What the packer packs beside the binary, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notices(Vec<Member>);

impl Notices {
    /// The license texts, read from the repository's root, `root`.
    pub fn read(root: &Path) -> Result<Self, FileFault> {
        NOTICES
            .iter()
            .map(|(name, path)| {
                std::fs::read(root.join(path))
                    .map(|bytes| Member {
                        name: (*name).to_owned(),
                        bytes,
                        mode: Mode::Readable,
                    })
                    .map_err(|error| FileFault::new(&root.join(path), &error))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

/// A file or directory the packer could not read or write, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileFault {
    pub path: PathBuf,
    pub kind: std::io::ErrorKind,
}

impl FileFault {
    fn new(path: &Path, error: &std::io::Error) -> Self {
        Self {
            path: path.to_path_buf(),
            kind: error.kind(),
        }
    }
}

impl fmt::Display for FileFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path.display(), self.kind)
    }
}

/// Why a tarball was not packed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackFault {
    /// A file the tarball holds could not be read.
    Read(FileFault),
    /// The archive could not be made: the tar or gzip encoder refused, as it said.
    Archive { said: String },
    /// OUT, or the tarball in it, could not be written.
    Write(FileFault),
}

impl fmt::Display for PackFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(fault) => write!(formatter, "cannot read {fault}"),
            Self::Archive { said } => write!(formatter, "the archive could not be made: {said}"),
            Self::Write(fault) => write!(formatter, "cannot write {fault}"),
        }
    }
}

/// The tarball's name, and the directory within it: `htmlspec-X.Y.Z-TARGET`.
pub fn stem(version: ReleaseVersion, target: Target) -> String {
    format!("htmlspec-{version}-{target}")
}

/// A header for one entry, with its owner, its time, and its mode fixed.
fn header(path: &str, size: u64, mode: Mode, kind: tar::EntryType) -> std::io::Result<tar::Header> {
    let mut header = tar::Header::new_ustar();
    header.set_path(path)?;
    header.set_entry_type(kind);
    header.set_size(size);
    header.set_mode(mode.bits());
    header.set_mtime(0);
    header.set_uid(0);
    header.set_gid(0);
    header.set_username("")?;
    header.set_groupname("")?;
    // The device numbers are left empty, as they are for a file that is no device.
    header.set_cksum();
    Ok(header)
}

/// The tarball's bytes: the directory, the binary, and the license texts, as a tar archive
/// compressed with a gzip header that holds no time or name. It reads and writes no file, so a
/// failure is the encoders' own.
pub fn tarball(
    version: ReleaseVersion,
    target: Target,
    binary: Vec<u8>,
    notices: &Notices,
) -> std::io::Result<Vec<u8>> {
    let stem = stem(version, target);
    let mut members = vec![Member {
        name: String::from("htmlspec"),
        bytes: binary,
        mode: Mode::Executable,
    }];
    members.extend(notices.0.iter().cloned());
    let mut archive = tar::Builder::new(Vec::new());
    archive.append(
        &header(
            &format!("{stem}/"),
            0,
            Mode::Executable,
            tar::EntryType::Directory,
        )?,
        std::io::empty(),
    )?;
    for member in &members {
        archive.append(
            &header(
                &format!("{stem}/{}", member.name),
                member.bytes.len() as u64,
                member.mode,
                tar::EntryType::Regular,
            )?,
            member.bytes.as_slice(),
        )?;
    }
    let tar = archive.into_inner()?;
    let mut compressed: GzEncoder<Vec<u8>> = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::best());
    compressed.write_all(&tar)?;
    compressed.finish()
}

/// Packs `binary`, built for `target`, into `out`, and gives the tarball's path.
pub fn pack(
    version: ReleaseVersion,
    target: Target,
    binary: &Path,
    notices: &Notices,
    out: &Path,
) -> Result<PathBuf, PackFault> {
    let bytes =
        std::fs::read(binary).map_err(|error| PackFault::Read(FileFault::new(binary, &error)))?;
    let packed = tarball(version, target, bytes, notices).map_err(|error| PackFault::Archive {
        said: error.to_string(),
    })?;
    std::fs::create_dir_all(out).map_err(|error| PackFault::Write(FileFault::new(out, &error)))?;
    let destination = out.join(format!("{}.tar.gz", stem(version, target)));
    std::fs::write(&destination, packed)
        .map_err(|error| PackFault::Write(FileFault::new(&destination, &error)))?;
    Ok(destination)
}

/// Why OUT does not hold one release's tarballs, or its sums could not be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SumsFault {
    /// A file named like a tarball that is not a release tarball for a target a release builds.
    NotATarball { name: String },
    /// Tarballs of two versions.
    Versions {
        out: PathBuf,
        first: ReleaseVersion,
        second: ReleaseVersion,
    },
    /// No release tarball at all.
    None { out: PathBuf },
    /// OUT, an entry of it, or a tarball could not be read.
    Read(FileFault),
    /// `SHA256SUMS` could not be written.
    Write(FileFault),
}

impl SumsFault {
    /// Whether the fault is in reading or writing a file, rather than in what OUT holds.
    pub fn could_not_ask(&self) -> bool {
        matches!(self, Self::Read(_) | Self::Write(_))
    }
}

impl fmt::Display for SumsFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotATarball { name } => write!(
                formatter,
                "{name} is not a release tarball for a known target"
            ),
            Self::Versions { out, first, second } => write!(
                formatter,
                "{} holds tarballs of {first} and {second}",
                out.display()
            ),
            Self::None { out } => write!(formatter, "{} holds no release tarball", out.display()),
            Self::Read(fault) => write!(formatter, "cannot read {fault}"),
            Self::Write(fault) => write!(formatter, "cannot write {fault}"),
        }
    }
}

/// What `--sums` wrote: the file, and the targets it has no tarball for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SumsWritten {
    pub path: PathBuf,
    pub missing: Vec<Target>,
}

/// Writes `SHA256SUMS` over every release tarball in `out`, one per target, in `targets`' order.
/// An entry of OUT that cannot be read stops it, rather than being passed over.
pub fn write_sums(out: &Path, targets: &[Target]) -> Result<SumsWritten, SumsFault> {
    let unread = |error: std::io::Error| SumsFault::Read(FileFault::new(out, &error));
    let mut names: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(out).map_err(unread)? {
        let entry = entry.map_err(unread)?;
        if let Some(name) = entry.file_name().to_str() {
            if name.starts_with("htmlspec-") && name.ends_with(".tar.gz") {
                names.push(name.to_owned());
            }
        }
    }
    names.sort();
    let mut version: Option<ReleaseVersion> = None;
    let mut found: Vec<(Target, String)> = Vec::new();
    for name in names {
        let read = tarball_name(&name).and_then(|(named, triple)| {
            Target::from_triple(triple)
                .filter(|target| targets.contains(target))
                .map(|target| (named, target))
        });
        let Some((named, target)) = read else {
            return Err(SumsFault::NotATarball { name });
        };
        if let Some(first) = version.filter(|first| *first != named) {
            return Err(SumsFault::Versions {
                out: out.to_path_buf(),
                first,
                second: named,
            });
        }
        version = Some(named);
        found.push((target, name));
    }
    if found.is_empty() {
        return Err(SumsFault::None {
            out: out.to_path_buf(),
        });
    }
    let mut lines = String::new();
    let mut missing = Vec::new();
    for target in targets {
        let Some((_, name)) = found.iter().find(|(found, _)| found == target) else {
            missing.push(*target);
            continue;
        };
        let path = out.join(name);
        let bytes =
            std::fs::read(&path).map_err(|error| SumsFault::Read(FileFault::new(&path, &error)))?;
        let digest: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        lines.push_str(&format!("{digest}  {name}\n"));
    }
    let path = out.join(SUMS);
    std::fs::write(&path, lines)
        .map_err(|error| SumsFault::Write(FileFault::new(&path, &error)))?;
    Ok(SumsWritten { path, missing })
}

/// What the packer was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Job {
    Pack {
        target: Target,
        binary: PathBuf,
        out: PathBuf,
    },
    Sums {
        out: PathBuf,
    },
}

/// The job's outcome, as the run ends.
pub fn report(job: &Job) -> Report {
    let facts = match Facts::read() {
        Ok(facts) => facts,
        Err(fault) => {
            return Report::could_not_ask(&format!(
                "pack-release: xtask/release-facts.json {fault}"
            ))
        }
    };
    match job {
        Job::Pack {
            target,
            binary,
            out,
        } => {
            if !binary.is_file() {
                return Report::could_not_ask(&format!(
                    "pack-release: {} is not a file",
                    binary.display()
                ));
            }
            let version = match facts::release_version() {
                Ok(version) => version,
                Err(fault) => {
                    return Report::could_not_ask(&format!(
                        "pack-release: the tools were built at a version that is not a release's: \
                         {fault}"
                    ))
                }
            };
            let packed = Notices::read(Path::new("."))
                .map_err(PackFault::Read)
                .and_then(|notices| pack(version, *target, binary, &notices, out));
            match packed {
                Ok(path) => Report {
                    exit: Exit::Clean,
                    out: vec![path.display().to_string()],
                    err: Vec::new(),
                },
                Err(fault) => Report::could_not_ask(&format!("pack-release: {fault}")),
            }
        }
        Job::Sums { out } => match write_sums(out, &facts.targets(MacosJobs::Run)) {
            Ok(written) => Report {
                exit: Exit::Clean,
                out: vec![written.path.display().to_string()],
                err: if written.missing.is_empty() {
                    Vec::new()
                } else {
                    vec![format!(
                        "pack-release: no tarball for {}",
                        written
                            .missing
                            .iter()
                            .map(|target| target.triple())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )]
                },
            },
            Err(fault) if fault.could_not_ask() => {
                Report::could_not_ask(&format!("pack-release: {fault}"))
            }
            Err(fault) => Report {
                exit: Exit::Faults,
                out: Vec::new(),
                err: vec![format!("pack-release: {fault}")],
            },
        },
    }
}

/// Packs a tarball, or writes `SHA256SUMS`.
pub fn run(job: &Job) -> ExitCode {
    report(job).print()
}

#[cfg(test)]
mod tests;
