use eyre::Result;
use eyre::ensure;
use std::path::Component;
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
    pub fn validate_path(path: &Path) -> Result<()> {
        ensure!(path.is_absolute(), "cache root must be absolute: {path:?}");
        ensure!(
            !path
                .components()
                .any(|component| component == Component::ParentDir),
            "cache root must not contain unresolved parent traversal: {path:?}"
        );
        Ok(())
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Path(path) => path,
            Self::Temporary(directory) => directory.path(),
        }
    }
}
