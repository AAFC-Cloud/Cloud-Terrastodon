use crate::SourceRepositoryHistorySummary;
use crate::SourceRepositoryRemote;
use crate::source_path_proxy::SourcePathProxy;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// File inventory attributed to the nearest enclosing Git working tree.
///
/// Counts cover only files below the requested scan root, even when `path`
/// identifies an ancestor repository. Nested repositories own their own files.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
pub struct SourceRepositorySummary {
    #[facet(proxy = SourcePathProxy)]
    pub path: PathBuf,
    pub remotes: Vec<SourceRepositoryRemote>,
    /// Whole-repository history, or unknown if it could not be read completely.
    /// A known empty/unborn repository has `Some` history with no authors.
    pub history: Option<SourceRepositoryHistorySummary>,
    /// All suffixes in this repository; the reserved `.` key represents extensionless files.
    pub extension_counts: BTreeMap<String, u64>,
    pub total_files: u64,
    /// Files with any requested suffix, or every file when no suffix was requested.
    pub matched_files: u64,
}

cloud_terrastodon_registry::register_thing!(SourceRepositorySummary);
cloud_terrastodon_registry::register_arbitrary!(SourceRepositorySummary);
cloud_terrastodon_registry::register_thing!(Vec<SourceRepositorySummary>);
cloud_terrastodon_registry::register_arbitrary!(Vec<SourceRepositorySummary>);
