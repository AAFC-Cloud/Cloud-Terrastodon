use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

/// Owns the cache location and, for a temporary root, its cleanup guard.
#[derive(Debug)]
pub(crate) enum WorkCacheRoot {
    Path(PathBuf),
    Temporary(TempDir),
}

impl WorkCacheRoot {
    pub fn path(&self) -> &Path {
        match self {
            Self::Path(path) => path,
            Self::Temporary(directory) => directory.path(),
        }
    }
}
