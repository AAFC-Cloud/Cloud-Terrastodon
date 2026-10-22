use crate::SourceHuntIssue;
use crate::SourceRepositorySummary;
use crate::source_path_proxy::SourcePathProxy;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Completed local inventory, including any paths that could not be read.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
pub struct SourceHuntReport {
    #[facet(proxy = SourcePathProxy)]
    pub root: PathBuf,
    /// Every repository, or repositories matching at least one selected extension.
    pub repositories: Vec<SourceRepositorySummary>,
    /// Counts across every scanned file, including files outside repositories.
    /// The reserved `.` key represents extensionless files.
    pub extension_counts: BTreeMap<String, u64>,
    pub files_scanned: u64,
    pub files_outside_repositories: u64,
    pub directories_scanned: u64,
    /// Nonempty when the result is partial because local paths could not be read.
    pub errors: Vec<SourceHuntIssue>,
}

cloud_terrastodon_registry::register_thing!(SourceHuntReport);
cloud_terrastodon_registry::register_arbitrary!(SourceHuntReport);
