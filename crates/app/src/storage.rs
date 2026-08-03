//! Local-disk attachment storage (`provider = "local_disk"`).

use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Local disk storage for uploaded assets.
#[derive(Debug, Clone)]
pub struct LocalStorage {
    root: PathBuf,
}

impl LocalStorage {
    /// Creates storage rooted at `root`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Writes `bytes` under `key`, creating parent directories.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] when the file cannot be written.
    pub fn save(&self, key: &str, bytes: &[u8]) -> Result<(), io::Error> {
        let path = self.root.join(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)
    }

    /// Reads the bytes stored under `key`.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] when the file cannot be read.
    pub fn read(&self, key: &str) -> Result<Vec<u8>, io::Error> {
        fs::read(self.root.join(key))
    }

    /// The root directory path.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}
