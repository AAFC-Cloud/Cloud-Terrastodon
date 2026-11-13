use crate::AzureStorageAccountActiveDirectoryProperties;
use crate::AzureStorageAccountDefaultSharePermission;
use crate::AzureStorageAccountDirectoryService;
use crate::AzureStorageAccountSmbOAuthSettings;
use arbitrary::Arbitrary;
use facet::Facet;

/// Azure Files directory and share authentication settings.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#azurefilesidentitybasedauthentication).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountAzureFilesIdentityBasedAuthentication {
    pub directory_service_options: AzureStorageAccountDirectoryService,
    pub active_directory_properties: Option<AzureStorageAccountActiveDirectoryProperties>,
    pub default_share_permission: Option<AzureStorageAccountDefaultSharePermission>,
    #[facet(rename = "smbOAuthSettings")]
    pub smb_oauth_settings: Option<AzureStorageAccountSmbOAuthSettings>,
}

cloud_terrastodon_registry::register_thing!(
    AzureStorageAccountAzureFilesIdentityBasedAuthentication
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountAzureFilesIdentityBasedAuthentication
);
