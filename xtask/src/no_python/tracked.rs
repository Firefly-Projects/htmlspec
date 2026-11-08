//! Which files the stage reads: the repository's tracked files, and nothing else.
//!
//! Two ways find them. In the suite's Nix derivation, `HTMLSPEC_SOURCE_TREE` names the source Nix
//! copied from the repository, which is the tracked files as they were before any stage built
//! anything, and every file under it is read. Everywhere else Git's index says which paths are
//! tracked, and each is read as the working tree holds it, so a staged or edited file is read as it
//! stands and a file the index does not list is not read at all: bytecode, build output, a worktree
//! under `.claude/`, a contributor's scratch file. Where neither way works, reading fails rather
//! than finding nothing.
//!
//! The two name the same paths. In a dirty tree they can differ on a file's mode, because Nix
//! copies the working tree's mode and the index records the staged one. So in a checkout a file
//! is executable when the index or the working tree says so, the second only where Git's
//! `core.fileMode` counts the working tree's modes, and the stage refuses what either would.
//!
//! A link is never followed, and nothing is skipped by its name: `.github/`, `.githooks/`, and
//! `.cargo/` are read like any other directory.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The variable the flake sets, in the suite's derivation alone, to the tracked tree's store path.
pub const SOURCE_TREE: &str = "HTMLSPEC_SOURCE_TREE";

/// A path relative to the repository's root, `/` between its components, as Git's index names one.
///
/// Never empty, never absolute, and never holding an empty, `.`, or `..` component, so two spellings
/// of one file cannot compare unequal and no path leaves the tree.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoPath(String);

impl RepoPath {
    pub fn parse(text: &str) -> Result<Self, PathFault> {
        if text.is_empty() {
            return Err(PathFault::Empty);
        }
        if text.starts_with('/') {
            return Err(PathFault::Absolute);
        }
        if text.contains('\0') {
            return Err(PathFault::Nul);
        }
        if text
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
        {
            return Err(PathFault::Component);
        }
        Ok(Self(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The last component.
    pub fn file_name(&self) -> &str {
        match self.0.rsplit_once('/') {
            Some((_, name)) => name,
            None => &self.0,
        }
    }

    /// Whether the path lies beneath `directory`, a path of whole components such as `.github`.
    pub fn is_under(&self, directory: &str) -> bool {
        self.0
            .strip_prefix(directory)
            .is_some_and(|rest| rest.starts_with('/'))
    }

    /// The components, first to last.
    pub fn components(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.0.split('/')
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a text is not a [`RepoPath`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathFault {
    Empty,
    Absolute,
    Nul,
    Component,
}

impl fmt::Display for PathFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "it is empty",
            Self::Absolute => "it is absolute",
            Self::Nul => "it holds a NUL byte",
            Self::Component => "it holds an empty, `.`, or `..` component",
        })
    }
}

/// What a tracked path holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// A file: its bytes, and whether it is executable.
    File { executable: bool, content: Vec<u8> },
    /// A symbolic link, which the stage never follows, and the path it points to.
    Link { target: Vec<u8> },
    /// A submodule's commit, which holds no file in this repository.
    Submodule,
}

/// Where the tracked files were read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// The tree `HTMLSPEC_SOURCE_TREE` names.
    SourceTree(PathBuf),
    /// Git's index, each listed file read from the working tree at this root.
    Index(PathBuf),
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceTree(path) => {
                write!(f, "the source tree {SOURCE_TREE} names, {}", path.display())
            }
            Self::Index(root) => write!(f, "Git's index of {}", root.display()),
        }
    }
}

/// The tracked files, by path.
#[derive(Debug)]
pub struct FileSet {
    origin: Origin,
    files: BTreeMap<RepoPath, Entry>,
}

impl FileSet {
    pub fn new(origin: Origin, files: BTreeMap<RepoPath, Entry>) -> Self {
        Self { origin, files }
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn get(&self, path: &str) -> Option<&Entry> {
        RepoPath::parse(path)
            .ok()
            .and_then(|path| self.files.get(&path))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&RepoPath, &Entry)> {
        self.files.iter()
    }
}

/// Why the tracked files could not be read.
#[derive(Debug)]
pub enum ReadFault {
    /// Neither a source tree nor a Git repository says which files are tracked.
    Untracked(Untracked),
    /// `HTMLSPEC_SOURCE_TREE` names something that is not a directory.
    NotADirectory(PathBuf),
    /// Git failed, and said this.
    Git { command: &'static str, said: String },
    /// A record of `git ls-files -z --stage` the stage cannot read.
    Record { record: String, why: RecordFault },
    /// A file that is tracked could not be read.
    Io { path: PathBuf, error: io::Error },
    /// A tracked path the stage refuses to read.
    Path { path: String, why: PathRefusal },
}

/// Why nothing says which files are tracked.
#[derive(Debug)]
pub enum Untracked {
    /// `git` could not be started.
    NoGit(io::Error),
    /// The directory is in no Git repository, as `git rev-parse` said.
    NoRepository { directory: PathBuf, said: String },
}

/// What is wrong with a record of the index.
#[derive(Debug, PartialEq, Eq)]
pub enum RecordFault {
    NoTab,
    UnknownMode(String),
}

/// Why a tracked path is refused.
#[derive(Debug, PartialEq, Eq)]
pub enum PathRefusal {
    NotUtf8,
    OutsideTheTree,
    NotAFileLinkOrDirectory,
    Malformed(PathFault),
}

impl fmt::Display for PathRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUtf8 => f.write_str("it is not UTF-8"),
            Self::OutsideTheTree => f.write_str("it is outside the tree"),
            Self::NotAFileLinkOrDirectory => {
                f.write_str("it is neither a file, a link, nor a directory")
            }
            Self::Malformed(fault) => write!(f, "{fault}"),
        }
    }
}

impl fmt::Display for ReadFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Untracked(why) => {
                write!(
                    f,
                    "cannot tell tracked files from anything else: {SOURCE_TREE} is not set, and "
                )?;
                match why {
                    Untracked::NoGit(error) => write!(f, "git could not be started ({error})")?,
                    Untracked::NoRepository { directory, said } => write!(
                        f,
                        "{} is in no Git repository (git rev-parse: {said})",
                        directory.display()
                    )?,
                }
                write!(
                    f,
                    ". Run it from the root of a Git checkout, or inside the suite's derivation, \
                     which sets {SOURCE_TREE}"
                )
            }
            Self::NotADirectory(path) => write!(
                f,
                "{SOURCE_TREE} names {}, which is not a directory",
                path.display()
            ),
            Self::Git { command, said } => write!(f, "`git {command}` failed: {said}"),
            Self::Record { record, why } => {
                let why = match why {
                    RecordFault::NoTab => String::from("it holds no tab"),
                    RecordFault::UnknownMode(mode) => format!("its mode, {mode}, is unknown"),
                };
                write!(
                    f,
                    "a record of the index this stage cannot read, {record:?}: {why}"
                )
            }
            Self::Io { path, error } => write!(f, "cannot read {}: {error}", path.display()),
            Self::Path { path, why } => write!(f, "the tracked path {path:?} is refused: {why}"),
        }
    }
}

/// The tracked files: under `source_tree` when it is set and not empty, and otherwise as Git's
/// index lists them in the repository that holds `directory`.
pub fn read(source_tree: Option<OsString>, directory: &Path) -> Result<FileSet, ReadFault> {
    match source_tree.filter(|value| !value.is_empty()) {
        Some(tree) => read_source_tree(PathBuf::from(tree)),
        None => read_index(directory),
    }
}

fn read_source_tree(tree: PathBuf) -> Result<FileSet, ReadFault> {
    let metadata = std::fs::metadata(&tree).map_err(|error| ReadFault::Io {
        path: tree.clone(),
        error,
    })?;
    if !metadata.is_dir() {
        return Err(ReadFault::NotADirectory(tree));
    }
    let mut files = BTreeMap::new();
    walk(&tree, &tree, &mut files)?;
    Ok(FileSet::new(Origin::SourceTree(tree), files))
}

fn io(path: &Path) -> impl FnOnce(io::Error) -> ReadFault {
    let path = path.to_path_buf();
    move |error| ReadFault::Io { path, error }
}

fn link_target(path: &Path) -> Result<Vec<u8>, ReadFault> {
    use std::os::unix::ffi::OsStrExt;
    let target = std::fs::read_link(path).map_err(io(path))?;
    Ok(target.as_os_str().as_bytes().to_vec())
}

fn walk(
    tree: &Path,
    directory: &Path,
    files: &mut BTreeMap<RepoPath, Entry>,
) -> Result<(), ReadFault> {
    for item in std::fs::read_dir(directory).map_err(io(directory))? {
        let path = item.map_err(io(directory))?.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(io(&path))?;
        let kind = metadata.file_type();
        if kind.is_dir() {
            walk(tree, &path, files)?;
            continue;
        }
        let entry = if kind.is_symlink() {
            Entry::Link {
                target: link_target(&path)?,
            }
        } else if kind.is_file() {
            Entry::File {
                executable: metadata.permissions().mode() & 0o111 != 0,
                content: std::fs::read(&path).map_err(io(&path))?,
            }
        } else {
            return Err(ReadFault::Path {
                path: path.display().to_string(),
                why: PathRefusal::NotAFileLinkOrDirectory,
            });
        };
        files.insert(relative(tree, &path)?, entry);
    }
    Ok(())
}

/// `path`, beneath `tree`, as a [`RepoPath`].
fn relative(tree: &Path, path: &Path) -> Result<RepoPath, ReadFault> {
    let refuse = |why| ReadFault::Path {
        path: path.display().to_string(),
        why,
    };
    let inner = path
        .strip_prefix(tree)
        .map_err(|_| refuse(PathRefusal::OutsideTheTree))?;
    let mut components = Vec::new();
    for component in inner.components() {
        let text = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| refuse(PathRefusal::NotUtf8))?;
        components.push(text);
    }
    RepoPath::parse(&components.join("/")).map_err(|why| refuse(PathRefusal::Malformed(why)))
}

fn read_index(directory: &Path) -> Result<FileSet, ReadFault> {
    let toplevel = git(
        directory,
        &["rev-parse", "--show-toplevel"],
        "rev-parse --show-toplevel",
    )
    .map_err(|fault| match fault {
        GitFault::NotStarted(error) => ReadFault::Untracked(Untracked::NoGit(error)),
        GitFault::Failed { said, .. } => ReadFault::Untracked(Untracked::NoRepository {
            directory: directory.to_path_buf(),
            said,
        }),
    })?;
    let root = PathBuf::from(String::from_utf8_lossy(&toplevel).trim_end_matches('\n'));
    let listing = git(&root, &["ls-files", "-z", "--stage"], "ls-files -z --stage")
        .map_err(GitFault::into_read_fault)?;
    let disk_modes = file_modes_count(&root)?;
    let mut files = BTreeMap::new();
    for record in listing.split(|byte| *byte == 0).filter(|r| !r.is_empty()) {
        let (mode, path) = index_record(record)?;
        let on_disk = root.join(path.as_str());
        let entry = match mode {
            IndexMode::Submodule => Some(Entry::Submodule),
            IndexMode::Link => working_link(&on_disk)?,
            IndexMode::File { executable } => working_file(&on_disk, executable, disk_modes)?,
        };
        if let Some(entry) = entry {
            files.insert(path, entry);
        }
    }
    Ok(FileSet::new(Origin::Index(root), files))
}

/// Whether Git counts the working tree's executable bits, as its `core.fileMode` says. A checkout
/// on a filesystem that keeps no modes, such as a Windows drive under WSL, sets it to false, and
/// every file there reads as executable on disk.
fn file_modes_count(root: &Path) -> Result<bool, ReadFault> {
    match git(
        root,
        &["config", "--bool", "core.fileMode"],
        "config --bool core.fileMode",
    ) {
        Ok(value) => Ok(String::from_utf8_lossy(&value).trim() != "false"),
        // Unset, which `git config` reports by failing: Git's default is to count them.
        Err(GitFault::Failed { .. }) => Ok(true),
        Err(fault) => Err(fault.into_read_fault()),
    }
}

/// A file the index lists, as the working tree holds it, or nothing where the working tree no
/// longer holds it: a deleted file is not read, because nothing of it is there to read. It is
/// executable when the index says so, or when the working tree says so and Git counts the
/// working tree's modes, so a mode changed and not yet staged is read as it stands, as an edit is.
fn working_file(
    path: &Path,
    index_executable: bool,
    disk_modes: bool,
) -> Result<Option<Entry>, ReadFault> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(ReadFault::Io {
                path: path.to_path_buf(),
                error,
            })
        }
    };
    let kind = metadata.file_type();
    if kind.is_symlink() {
        return Ok(Some(Entry::Link {
            target: link_target(path)?,
        }));
    }
    if !kind.is_file() {
        return Ok(None);
    }
    let content = std::fs::read(path).map_err(io(path))?;
    let disk_executable = metadata.permissions().mode() & 0o111 != 0;
    Ok(Some(Entry::File {
        executable: index_executable || (disk_modes && disk_executable),
        content,
    }))
}

/// A link the index lists, as the working tree holds it: a link, or, where the checkout makes no
/// links, a file holding the target.
fn working_link(path: &Path) -> Result<Option<Entry>, ReadFault> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Ok(Some(Entry::Link {
            target: link_target(path)?,
        })),
        Ok(metadata) if metadata.is_file() => Ok(Some(Entry::Link {
            target: std::fs::read(path).map_err(io(path))?,
        })),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ReadFault::Io {
            path: path.to_path_buf(),
            error,
        }),
    }
}

/// What the index records a path as.
#[derive(Debug, PartialEq, Eq)]
enum IndexMode {
    File { executable: bool },
    Link,
    Submodule,
}

/// One record of `git ls-files -z --stage`: `MODE OBJECT STAGE<TAB>PATH`.
fn index_record(record: &[u8]) -> Result<(IndexMode, RepoPath), ReadFault> {
    let text = std::str::from_utf8(record).map_err(|_| ReadFault::Path {
        path: String::from_utf8_lossy(record).into_owned(),
        why: PathRefusal::NotUtf8,
    })?;
    let refuse = |why| ReadFault::Record {
        record: text.to_owned(),
        why,
    };
    let (fields, path) = text
        .split_once('\t')
        .ok_or_else(|| refuse(RecordFault::NoTab))?;
    let mode = match fields.split(' ').next().unwrap_or_default() {
        "100644" => IndexMode::File { executable: false },
        "100755" => IndexMode::File { executable: true },
        "120000" => IndexMode::Link,
        "160000" => IndexMode::Submodule,
        other => return Err(refuse(RecordFault::UnknownMode(other.to_owned()))),
    };
    let path = RepoPath::parse(path).map_err(|why| ReadFault::Path {
        path: path.to_owned(),
        why: PathRefusal::Malformed(why),
    })?;
    Ok((mode, path))
}

/// How a `git` command failed.
enum GitFault {
    NotStarted(io::Error),
    Failed { command: &'static str, said: String },
}

impl GitFault {
    fn into_read_fault(self) -> ReadFault {
        match self {
            Self::NotStarted(error) => ReadFault::Untracked(Untracked::NoGit(error)),
            Self::Failed { command, said } => ReadFault::Git { command, said },
        }
    }
}

/// Runs `git` in `directory` and returns what it printed, or how it failed.
fn git(directory: &Path, arguments: &[&str], command: &'static str) -> Result<Vec<u8>, GitFault> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(arguments)
        .output()
        .map_err(GitFault::NotStarted)?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(GitFault::Failed {
            command,
            said: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A temporary directory that removes itself.
    pub struct Scratch(pub PathBuf);

    impl Scratch {
        pub fn new(label: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("htmlspec-xtask-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory is created");
            Self(path)
        }

        pub fn write(&self, path: &str, content: &str) {
            let file = self.0.join(path);
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent).expect("the parent directory is created");
            }
            std::fs::write(&file, content).expect("the file is written");
        }

        pub fn mark_executable(&self, path: &str) {
            let file = self.0.join(path);
            let mut permissions = std::fs::metadata(&file)
                .expect("the file exists")
                .permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&file, permissions).expect("the mode is set");
        }

        /// Runs git here, isolated from the contributor's and the system's configuration.
        pub fn git(&self, arguments: &[&str]) {
            let status = Command::new("git")
                .arg("-C")
                .arg(&self.0)
                .args(arguments)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env_remove("GIT_DIR")
                .env_remove("GIT_WORK_TREE")
                .env_remove("GIT_INDEX_FILE")
                .status()
                .expect("git starts");
            assert!(status.success(), "git {arguments:?} failed");
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn names(files: &FileSet) -> Vec<&str> {
        files.iter().map(|(path, _)| path.as_str()).collect()
    }

    #[test]
    fn a_path_is_relative_and_has_no_empty_or_dot_components() {
        assert!(RepoPath::parse("scripts/check").is_ok());
        assert!(RepoPath::parse(".github/workflows/check.yml").is_ok());
        assert_eq!(RepoPath::parse(""), Err(PathFault::Empty));
        assert_eq!(RepoPath::parse("/etc/passwd"), Err(PathFault::Absolute));
        assert_eq!(RepoPath::parse("a//b"), Err(PathFault::Component));
        assert_eq!(RepoPath::parse("a/./b"), Err(PathFault::Component));
        assert_eq!(RepoPath::parse("../outside"), Err(PathFault::Component));
        assert_eq!(RepoPath::parse("trailing/"), Err(PathFault::Component));
    }

    #[test]
    fn a_path_knows_its_name_and_its_directories() {
        let path = RepoPath::parse(".github/workflows/check.yml").expect("a path");
        assert_eq!(path.file_name(), "check.yml");
        assert!(path.is_under(".github"));
        assert!(path.is_under(".github/workflows"));
        assert!(!path.is_under(".git"));
        assert!(!path.is_under(".github/workflows/check.yml"));
        assert_eq!(
            RepoPath::parse("LICENSE").expect("a path").file_name(),
            "LICENSE"
        );
    }

    /// The index decides: an untracked file and an ignored bytecode directory are not read, a
    /// staged file is read as the working tree holds it, and the index's mode says whether a file
    /// is executable.
    #[test]
    fn the_index_lists_what_is_read() {
        let scratch = Scratch::new("index");
        scratch.git(&["init", "-q"]);
        scratch.write(".gitignore", "__pycache__/\n");
        scratch.write("scripts/check", "#!/usr/bin/env bash\n");
        scratch.mark_executable("scripts/check");
        scratch.write("notes.md", "first\n");
        scratch.git(&["add", ".gitignore", "notes.md"]);
        scratch.git(&["add", "--chmod=+x", "scripts/check"]);
        scratch.write("notes.md", "edited after staging\n");
        scratch.write("tidy.py", "print('untracked')\n");
        scratch.write("scripts/__pycache__/check.cpython-313.pyc", "bytecode");

        let files = read(None, &scratch.0).expect("the index is read");
        assert_eq!(names(&files), [".gitignore", "notes.md", "scripts/check"]);
        assert_eq!(
            files.get("notes.md"),
            Some(&Entry::File {
                executable: false,
                content: b"edited after staging\n".to_vec()
            })
        );
        assert!(matches!(
            files.get("scripts/check"),
            Some(Entry::File {
                executable: true,
                ..
            })
        ));
        assert!(matches!(files.origin(), Origin::Index(_)));

        scratch.git(&["add", "tidy.py"]);
        let files = read(None, &scratch.0).expect("the index is read");
        assert!(files.get("tidy.py").is_some(), "a staged file is read");
    }

    /// A file the index lists and the working tree has lost is not read, so an allow-list entry
    /// naming it reads as absent before the deletion is staged.
    #[test]
    fn a_file_deleted_from_the_working_tree_is_not_read() {
        let scratch = Scratch::new("deleted");
        scratch.git(&["init", "-q"]);
        scratch.write("kept.md", "kept\n");
        scratch.write("gone.md", "gone\n");
        scratch.git(&["add", "kept.md", "gone.md"]);
        std::fs::remove_file(scratch.0.join("gone.md")).expect("the file is removed");
        let files = read(None, &scratch.0).expect("the index is read");
        assert_eq!(names(&files), ["kept.md"]);
    }

    /// A mode changed on disk and not yet staged is read as it stands, as an edit is, unless Git
    /// counts no modes in this checkout. The index's mode counts either way.
    #[test]
    fn a_file_made_executable_on_disk_reads_as_executable() {
        let scratch = Scratch::new("disk-mode");
        scratch.git(&["init", "-q"]);
        scratch.write("tools/run", "echo\n");
        scratch.write("tools/staged", "echo\n");
        scratch.git(&["add", "tools/run"]);
        scratch.git(&["add", "--chmod=+x", "tools/staged"]);
        scratch.mark_executable("tools/run");
        let executable = |files: &FileSet, path: &str| {
            matches!(
                files.get(path),
                Some(Entry::File {
                    executable: true,
                    ..
                })
            )
        };
        let files = read(None, &scratch.0).expect("the index is read");
        assert!(executable(&files, "tools/run"));
        assert!(executable(&files, "tools/staged"));

        scratch.git(&["config", "core.fileMode", "false"]);
        let files = read(None, &scratch.0).expect("the index is read");
        assert!(!executable(&files, "tools/run"));
        assert!(executable(&files, "tools/staged"));
    }

    /// A link is read as its target, never followed, and a submodule as what it is.
    #[test]
    fn a_link_and_a_submodule_are_read_without_following_either() {
        let scratch = Scratch::new("links");
        scratch.git(&["init", "-q"]);
        std::os::unix::fs::symlink("/usr/bin/python3", scratch.0.join("py")).expect("a link");
        scratch.git(&["add", "py"]);
        let commit = "0123456789abcdef0123456789abcdef01234567";
        scratch.git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{commit},vendor/sub"),
        ]);
        std::fs::create_dir_all(scratch.0.join("vendor/sub")).expect("the submodule's directory");
        let files = read(None, &scratch.0).expect("the index is read");
        assert_eq!(
            files.get("py"),
            Some(&Entry::Link {
                target: b"/usr/bin/python3".to_vec()
            })
        );
        assert_eq!(files.get("vendor/sub"), Some(&Entry::Submodule));
    }

    /// Run from a subdirectory, the stage still reads the whole repository.
    #[test]
    fn the_index_is_read_from_the_root() {
        let scratch = Scratch::new("subdirectory");
        scratch.git(&["init", "-q"]);
        scratch.write("top.md", "top\n");
        scratch.write("crates/inner.md", "inner\n");
        scratch.git(&["add", "top.md", "crates/inner.md"]);
        let files = read(None, &scratch.0.join("crates")).expect("the index is read");
        assert_eq!(names(&files), ["crates/inner.md", "top.md"]);
    }

    #[test]
    fn a_directory_in_no_repository_and_no_source_tree_is_refused() {
        let scratch = Scratch::new("nowhere");
        let fault = read(None, &scratch.0).expect_err("nothing says which files are tracked");
        assert!(matches!(fault, ReadFault::Untracked(_)), "{fault}");
        assert!(
            fault.to_string().contains("cannot tell tracked files"),
            "{fault}"
        );
    }

    /// Under a source tree, every file is read, dot-directories included, and the file's own mode
    /// says whether it is executable. An empty variable is unset.
    #[test]
    fn a_source_tree_is_read_whole() {
        let scratch = Scratch::new("tracked-source-tree");
        scratch.write(".github/workflows/check.yml", "on: push\n");
        scratch.write("scripts/check", "#!/usr/bin/env bash\n");
        scratch.mark_executable("scripts/check");
        std::os::unix::fs::symlink("scripts/check", scratch.0.join("link"))
            .expect("the link is made");

        let files = read(Some(scratch.0.clone().into_os_string()), Path::new("/"))
            .expect("the source tree is read");
        assert_eq!(
            names(&files),
            [".github/workflows/check.yml", "link", "scripts/check"]
        );
        assert_eq!(
            files.get("link"),
            Some(&Entry::Link {
                target: b"scripts/check".to_vec()
            })
        );
        assert!(matches!(
            files.get("scripts/check"),
            Some(Entry::File {
                executable: true,
                ..
            })
        ));
        assert_eq!(files.origin(), &Origin::SourceTree(scratch.0.clone()));

        let fault = read(Some(OsString::new()), &scratch.0).expect_err("an empty value is unset");
        assert!(matches!(fault, ReadFault::Untracked(_)), "{fault}");
    }

    #[test]
    fn a_source_tree_that_is_not_a_directory_is_refused() {
        let scratch = Scratch::new("not-a-tree");
        scratch.write("file", "");
        let fault = read(Some(scratch.0.join("file").into_os_string()), &scratch.0)
            .expect_err("a file is no tree");
        assert!(matches!(fault, ReadFault::NotADirectory(_)), "{fault}");
    }

    #[test]
    fn an_index_record_names_its_mode_and_path() {
        let record = |text: &str| index_record(text.as_bytes()).map_err(|f| f.to_string());
        assert_eq!(
            record("100755 0123 0\tscripts/check").expect("a record").0,
            IndexMode::File { executable: true }
        );
        assert_eq!(
            record("100644 0123 0\tREADME.md").expect("a record").0,
            IndexMode::File { executable: false }
        );
        assert_eq!(
            record("120000 0123 0\tlink").expect("a record").0,
            IndexMode::Link
        );
        assert_eq!(
            record("160000 0123 0\tvendor/sub").expect("a record").0,
            IndexMode::Submodule
        );
        assert!(record("040000 0123 0\tdir").is_err());
        assert!(record("100644 0123 0").is_err());
    }
}
