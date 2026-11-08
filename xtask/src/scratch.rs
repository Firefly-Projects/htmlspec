//! A temporary directory for a test, which removes itself.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tells apart the directories one process's tests make, which run in parallel.
static MADE: AtomicUsize = AtomicUsize::new(0);

/// A temporary directory that removes itself when dropped.
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "htmlspec-xtask-{label}-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Writes `content` at `path`, beneath the directory, making its parents.
    pub fn write(&self, path: &str, content: &str) {
        let file = self.0.join(path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).expect("the parent directory is created");
        }
        std::fs::write(&file, content).expect("the file is written");
    }

    /// Removes the file at `path`, beneath the directory.
    pub fn remove(&self, path: &str) {
        std::fs::remove_file(self.0.join(path)).expect("the file is removed");
    }

    /// The file at `path`, beneath the directory.
    pub fn read(&self, path: &str) -> String {
        std::fs::read_to_string(self.0.join(path)).expect("the file is read")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
