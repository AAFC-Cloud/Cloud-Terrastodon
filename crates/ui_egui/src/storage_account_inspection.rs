use cloud_terrastodon_azure::AzureStorageAccount;
use cloud_terrastodon_azure::AzureStorageAccountGetRequest;
use cloud_terrastodon_command::app_work::Loadable;
use std::sync::Arc;

/// The last storage account request and response, shared by all tabs for that ID.
pub(crate) struct StorageAccountInspection {
    pub request: AzureStorageAccountGetRequest<'static>,
    pub response: Loadable<Arc<AzureStorageAccount>>,
}
