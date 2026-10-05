use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;

/// Complete, uncached folders and definition references before pruning validation.
pub(crate) struct AzureDevOpsBuildFolderSnapshot {
    pub(crate) folders: Vec<AzureDevOpsBuildFolder>,
    pub(crate) definitions: Vec<AzureDevOpsBuildDefinitionReference>,
}
