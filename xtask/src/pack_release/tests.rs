//! The packer's cases: what a tarball holds, that its bytes are fixed, and what `SHA256SUMS` says.

use std::io::Read;

use super::*;
use crate::scratch::Scratch;

fn version(text: &str) -> ReleaseVersion {
    ReleaseVersion::parse(text).expect("a release version")
}

fn notices() -> Notices {
    Notices(vec![
        Member {
            name: String::from("LICENSE"),
            bytes: b"the project's license\n".to_vec(),
            mode: Mode::Readable,
        },
        Member {
            name: String::from("THIRD-PARTY-LICENSES"),
            bytes: b"the embedded libraries' notices\n".to_vec(),
            mode: Mode::Readable,
        },
        Member {
            name: String::from("CRATE-LICENSES"),
            bytes: b"the crates' notices\n".to_vec(),
            mode: Mode::Readable,
        },
    ])
}

/// One entry of a tarball, as a reader of it sees it.
#[derive(Debug, PartialEq, Eq)]
struct Seen {
    path: String,
    kind: tar::EntryType,
    mode: u32,
    uid: u64,
    gid: u64,
    username: Vec<u8>,
    groupname: Vec<u8>,
    mtime: u64,
    bytes: Vec<u8>,
}

fn entries(tarball: &[u8]) -> Vec<Seen> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball));
    let mut seen = Vec::new();
    for entry in archive.entries().expect("the archive reads") {
        let mut entry = entry.expect("the entry reads");
        let header = entry.header().clone();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).expect("its bytes read");
        seen.push(Seen {
            path: String::from_utf8_lossy(&header.path_bytes()).into_owned(),
            kind: header.entry_type(),
            mode: header.mode().expect("a mode"),
            uid: header.uid().expect("an owner"),
            gid: header.gid().expect("a group"),
            username: header
                .username_bytes()
                .map(<[u8]>::to_vec)
                .unwrap_or_default(),
            groupname: header
                .groupname_bytes()
                .map(<[u8]>::to_vec)
                .unwrap_or_default(),
            mtime: header.mtime().expect("a time"),
            bytes,
        });
    }
    seen
}

#[test]
fn a_tarball_holds_one_directory_with_the_binary_and_its_license_texts() {
    let packed = tarball(
        version("1.2.3"),
        Target::X8664UnknownLinuxMusl,
        b"a binary".to_vec(),
        &notices(),
    )
    .expect("it packs");
    let seen = entries(&packed);
    let stem = "htmlspec-1.2.3-x86_64-unknown-linux-musl";
    let expected: Vec<(String, tar::EntryType, u32, &[u8])> = vec![
        (format!("{stem}/"), tar::EntryType::Directory, 0o755, b""),
        (
            format!("{stem}/htmlspec"),
            tar::EntryType::Regular,
            0o755,
            b"a binary",
        ),
        (
            format!("{stem}/LICENSE"),
            tar::EntryType::Regular,
            0o644,
            b"the project's license\n",
        ),
        (
            format!("{stem}/THIRD-PARTY-LICENSES"),
            tar::EntryType::Regular,
            0o644,
            b"the embedded libraries' notices\n",
        ),
        (
            format!("{stem}/CRATE-LICENSES"),
            tar::EntryType::Regular,
            0o644,
            b"the crates' notices\n",
        ),
    ];
    assert_eq!(seen.len(), expected.len());
    for (entry, (path, kind, mode, bytes)) in seen.iter().zip(expected) {
        assert_eq!(entry.path, path);
        assert_eq!(entry.kind, kind, "{path}");
        assert_eq!(entry.mode, mode, "{path}");
        assert_eq!((entry.uid, entry.gid, entry.mtime), (0, 0, 0), "{path}");
        assert!(
            entry.username.is_empty() && entry.groupname.is_empty(),
            "{path}"
        );
        assert_eq!(entry.bytes, bytes, "{path}");
    }
}

/// The same inputs pack to the same bytes, and the gzip header holds no time and no name: the
/// magic, deflate, no flags, a time of zero, the best compression, and no operating system named.
#[test]
fn the_same_binary_packs_to_the_same_bytes() {
    let pack = || {
        tarball(
            version("1.2.3"),
            Target::Aarch64AppleDarwin,
            b"a binary".to_vec(),
            &notices(),
        )
        .expect("it packs")
    };
    let (one, other) = (pack(), pack());
    assert_eq!(one, other);
    assert_eq!(
        one.get(..10),
        Some(&[0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0x02, 0xff][..])
    );
}

/// A binary a workflow artifact carried has lost its executable bit, and its entry has it anyway.
#[test]
fn the_binary_s_mode_is_set_in_its_entry_whatever_the_file_s() {
    let scratch = Scratch::new("pack");
    scratch.write("htmlspec", "a binary without its executable bit");
    let path = pack(
        version("1.2.3"),
        Target::Aarch64UnknownLinuxMusl,
        &scratch.path().join("htmlspec"),
        &notices(),
        &scratch.path().join("dist"),
    )
    .expect("it packs");
    assert_eq!(
        path,
        scratch
            .path()
            .join("dist/htmlspec-1.2.3-aarch64-unknown-linux-musl.tar.gz")
    );
    let seen = entries(&std::fs::read(&path).expect("the tarball reads"));
    assert_eq!(seen.get(1).map(|entry| entry.mode), Some(0o755));
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// `sha256sum`'s format, in the facts' order of the targets, and the targets with no tarball named.
#[test]
fn sums_name_each_tarball_in_the_targets_order_as_sha256sum_does() {
    let scratch = Scratch::new("sums");
    for target in ["aarch64-unknown-linux-musl", "x86_64-unknown-linux-musl"] {
        scratch.write(&format!("htmlspec-1.2.3-{target}.tar.gz"), target);
    }
    scratch.write("notes.txt", "not a tarball, and not read");
    let written = write_sums(scratch.path(), &Target::ALL).expect("the sums are written");
    assert_eq!(written.path, scratch.path().join("SHA256SUMS"));
    assert_eq!(
        written.missing,
        [Target::Aarch64AppleDarwin, Target::X8664AppleDarwin]
    );
    assert_eq!(
        scratch.read("SHA256SUMS"),
        format!(
            "{}  htmlspec-1.2.3-x86_64-unknown-linux-musl.tar.gz\n{}  htmlspec-1.2.3-aarch64-unknown-linux-musl.tar.gz\n",
            digest(b"x86_64-unknown-linux-musl"),
            digest(b"aarch64-unknown-linux-musl")
        )
    );
}

/// A packing that fails names what failed: the binary it could not read, or OUT it could not
/// write, each by its own path, and never one for the other.
#[test]
fn a_failure_names_the_file_that_failed() {
    let scratch = Scratch::new("pack-faults");
    let missing = scratch.path().join("no-binary");
    assert_eq!(
        pack(
            version("1.2.3"),
            Target::X8664UnknownLinuxMusl,
            &missing,
            &notices(),
            &scratch.path().join("dist"),
        ),
        Err(PackFault::Read(FileFault {
            path: missing,
            kind: std::io::ErrorKind::NotFound,
        }))
    );
    scratch.write("htmlspec", "a binary");
    scratch.write("occupied", "a file where OUT would be");
    let refused = pack(
        version("1.2.3"),
        Target::X8664UnknownLinuxMusl,
        &scratch.path().join("htmlspec"),
        &notices(),
        &scratch.path().join("occupied"),
    );
    assert!(
        matches!(&refused, Err(PackFault::Write(FileFault { path, .. })) if *path == scratch.path().join("occupied")),
        "{refused:?}"
    );
    assert_eq!(Mode::Executable.bits(), 0o755);
    assert_eq!(Mode::Readable.bits(), 0o644);
}

/// An OUT that cannot be listed is a read that failed, and exits 2; it is not passed over.
#[test]
fn an_out_that_cannot_be_read_is_named() {
    let scratch = Scratch::new("sums-unread");
    scratch.write("not-a-directory", "x");
    let refused = write_sums(&scratch.path().join("not-a-directory"), &Target::ALL);
    assert!(
        matches!(&refused, Err(SumsFault::Read(FileFault { path, .. })) if *path == scratch.path().join("not-a-directory")),
        "{refused:?}"
    );
    assert!(refused.as_ref().is_err_and(SumsFault::could_not_ask));
}

#[test]
fn sums_over_what_is_not_one_release_s_tarballs_are_refused() {
    let scratch = Scratch::new("sums-refused");
    assert_eq!(
        write_sums(scratch.path(), &Target::ALL),
        Err(SumsFault::None {
            out: scratch.path().to_path_buf()
        })
    );
    scratch.write("htmlspec-1.2.3-x86_64-unknown-linux-musl.tar.gz", "x");
    scratch.write("htmlspec-1.2.4-aarch64-unknown-linux-musl.tar.gz", "a");
    assert!(matches!(
        write_sums(scratch.path(), &Target::ALL),
        Err(SumsFault::Versions { .. })
    ));
    scratch.remove("htmlspec-1.2.4-aarch64-unknown-linux-musl.tar.gz");
    scratch.write("htmlspec-1.2.3-riscv64gc-unknown-linux-gnu.tar.gz", "r");
    assert_eq!(
        write_sums(scratch.path(), &Target::ALL),
        Err(SumsFault::NotATarball {
            name: String::from("htmlspec-1.2.3-riscv64gc-unknown-linux-gnu.tar.gz")
        })
    );
}

/// The path written on standard output, exit 0; what is not a release's tarballs, exit 1; what
/// cannot be read, exit 2.
#[test]
fn each_ending_has_its_exit_code() {
    let scratch = Scratch::new("pack-exits");
    scratch.write("htmlspec-1.2.3-x86_64-unknown-linux-musl.tar.gz", "x");
    let written = report(&Job::Sums {
        out: scratch.path().to_path_buf(),
    });
    assert_eq!(written.exit, Exit::Clean);
    assert_eq!(
        written.out,
        [scratch.path().join("SHA256SUMS").display().to_string()]
    );
    assert_eq!(
        written.err,
        ["pack-release: no tarball for aarch64-apple-darwin, x86_64-apple-darwin, aarch64-unknown-linux-musl"]
    );
    scratch.write("htmlspec-1.2.3-riscv64gc-unknown-linux-gnu.tar.gz", "r");
    assert_eq!(
        report(&Job::Sums {
            out: scratch.path().to_path_buf()
        })
        .exit,
        Exit::Faults
    );
    assert_eq!(
        report(&Job::Sums {
            out: scratch.path().join("absent")
        })
        .exit,
        Exit::CouldNotAsk
    );
    assert_eq!(
        report(&Job::Pack {
            target: Target::X8664UnknownLinuxMusl,
            binary: scratch.path().join("no-binary"),
            out: scratch.path().join("dist"),
        })
        .exit,
        Exit::CouldNotAsk
    );
}
