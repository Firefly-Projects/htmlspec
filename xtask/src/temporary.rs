//! A scratch directory a tool makes for one run and removes after it.
//!
//! It is made new, under the system's temporary directory, readable by its owner alone, and never
//! one the tool finds there: a name another process already holds is passed over for the next, so
//! nothing another user left in a shared temporary directory is read or written.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tells apart the directories one process makes.
static MADE: AtomicUsize = AtomicUsize::new(0);

/// A directory made for this run, removed when it is dropped.
#[derive(Debug)]
pub struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    /// Makes a new directory, named for `label`, this process, and a count.
    pub fn new(label: &str) -> std::io::Result<Self> {
        let temporary = std::env::temp_dir();
        loop {
            let path = temporary.join(format!(
                "htmlspec-{label}-{}-{}",
                std::process::id(),
                MADE.fetch_add(1, Ordering::Relaxed)
            ));
            let mut builder = std::fs::DirBuilder::new();
            #[cfg(unix)]
            std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
            match builder.create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_is_new_its_owner_s_alone_and_gone_when_dropped() {
        let first = TemporaryDirectory::new("test").expect("a directory is made");
        let second = TemporaryDirectory::new("test").expect("a second is made");
        assert_ne!(first.path(), second.path());
        assert!(first.path().is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(first.path())
                .expect("it is read")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o700);
        }
        let path = first.path().to_path_buf();
        std::fs::write(path.join("file"), "x").expect("a file is written in it");
        drop(first);
        assert!(!path.exists());
    }
}
