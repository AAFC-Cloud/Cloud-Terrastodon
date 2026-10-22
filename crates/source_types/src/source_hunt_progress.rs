use crate::SourceRepositoryProgress;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// A bounded UI snapshot of an inventory in progress.
///
/// The request emits snapshots at most about ten times a second. Repository
/// snapshots contain only the ten highest current matching file counts.
#[derive(Debug, Clone, Default)]
pub struct SourceHuntProgress {
    pub directories_scanned: u64,
    pub files_scanned: u64,
    pub repositories_found: usize,
    pub active_paths: Vec<PathBuf>,
    /// Global suffix counts; the reserved `.` key represents extensionless files.
    pub extension_counts: BTreeMap<String, u64>,
    pub top_repositories: Vec<SourceRepositoryProgress>,
    pub errors: usize,
}
