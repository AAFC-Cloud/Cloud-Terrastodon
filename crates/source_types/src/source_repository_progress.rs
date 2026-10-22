use std::path::PathBuf;

/// File-count snapshot used to rank repositories while an inventory is running.
///
/// Live progress omits completed history and remote metadata so refreshing the
/// display never copies potentially large author inventories.
#[derive(Debug, Clone)]
pub struct SourceRepositoryProgress {
    pub path: PathBuf,
    pub total_files: u64,
    pub matched_files: u64,
}
