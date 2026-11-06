use crate::AzureDevOpsBuildFolderPruneSkipReason;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use facet::Facet;

/// A planned folder deletion prevented by a fresh inventory check.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
pub struct AzureDevOpsBuildFolderPruneSkip {
    pub path: AzureDevOpsBuildFolderPath,
    pub reason: AzureDevOpsBuildFolderPruneSkipReason,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderPruneSkip);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderPruneSkip);
