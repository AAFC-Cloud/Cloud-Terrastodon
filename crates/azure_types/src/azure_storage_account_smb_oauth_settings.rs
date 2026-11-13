use arbitrary::Arbitrary;
use facet::Facet;

/// Whether Azure Files admits managed identities over SMB using OAuth.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#smboauthsettings).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountSmbOAuthSettings {
    #[facet(rename = "isSmbOAuthEnabled")]
    pub is_smb_oauth_enabled: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountSmbOAuthSettings);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountSmbOAuthSettings);
