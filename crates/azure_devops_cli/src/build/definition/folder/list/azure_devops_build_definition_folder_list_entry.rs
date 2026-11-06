use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolder;
use std::sync::Arc;

/// A folder and the build definitions directly assigned to it in CLI output.
///
/// Folder metadata retains its existing JSON shape, with a `definitions` array
/// added for text, JSON, and Facet Pretty to share the same grouped inventory.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildDefinitionFolderListEntry {
    #[facet(flatten)]
    pub folder: Arc<AzureDevOpsBuildFolder>,
    pub definitions: Vec<Arc<AzureDevOpsBuildDefinitionReference>>,
}
