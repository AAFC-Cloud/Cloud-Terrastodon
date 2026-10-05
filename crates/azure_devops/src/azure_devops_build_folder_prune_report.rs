use crate::AzureDevOpsBuildFolderPruneSkip;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;

/// Planned, deleted, and skipped folders from an empty-folder pruning operation.
/// A dry run reports candidates without deleting them or reserving their state.
///
/// See the [folder-delete API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/delete?view=azure-devops-rest-7.1)
/// for the cascading operation that the pruning checks guard.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
pub struct AzureDevOpsBuildFolderPruneReport {
    pub dry_run: bool,
    pub candidates: Vec<AzureDevOpsBuildFolderPath>,
    pub deleted: Vec<AzureDevOpsBuildFolderPath>,
    pub skipped: Vec<AzureDevOpsBuildFolderPruneSkip>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderPruneReport);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderPruneReport);
