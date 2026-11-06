use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;
use std::sync::Arc;

/// Complete, uncached folders and definition references before pruning validation.
pub(crate) struct AzureDevOpsBuildFolderSnapshot {
    pub(crate) folders: Vec<Arc<AzureDevOpsBuildFolder>>,
    pub(crate) definitions: Vec<Arc<AzureDevOpsBuildDefinitionReference>>,
}
