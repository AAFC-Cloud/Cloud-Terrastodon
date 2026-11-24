use cloud_terrastodon_azure::AzureStorageAccountBlobContainer;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerPublicAccess;
use facet::Facet;
use teamy_tracing::gui::escape_display;

/// The searchable summary of a container in the management-plane inventory.
#[derive(Debug, Facet)]
pub(crate) struct StorageExplorerContainerRow {
    pub id: AzureStorageAccountBlobContainerId,
    pub name: String,
    pub last_modified: String,
    pub anonymous_access: String,
    pub lease_state: String,
}

impl StorageExplorerContainerRow {
    pub fn new(container: &AzureStorageAccountBlobContainer) -> Self {
        let properties = &container.properties;
        Self {
            id: container.id.clone(),
            name: escape_display(container.name.as_str()),
            last_modified: properties
                .last_modified_time
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            anonymous_access: match &properties.public_access {
                AzureStorageAccountBlobContainerPublicAccess::None => "Private".into(),
                other => escape_display(&String::from(other)),
            },
            lease_state: escape_display(&String::from(&properties.lease_state)),
        }
    }
}
