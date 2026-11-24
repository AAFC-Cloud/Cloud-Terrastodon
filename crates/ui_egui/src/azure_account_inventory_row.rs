use crate::azure_account_details::AzureAccountDetails;
use crate::azure_account_registration_form::source_label;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use facet::Facet;

/// Searchable account registration metadata and the identity used to open its details.
#[derive(Facet)]
pub(crate) struct AzureAccountInventoryRow {
    pub id: CloudTerrastodonAccountId,
    pub alias: CloudTerrastodonAccountAlias,
    pub scope: String,
    pub preferred_source: &'static str,
    pub session: Option<&'static str>,
    pub default_account: &'static str,
}

impl AzureAccountInventoryRow {
    pub fn new(account: &CloudTerrastodonAccountProfile, is_default: bool) -> Self {
        Self {
            id: account.id,
            alias: account.alias.clone(),
            scope: AzureAccountDetails::namespace_label(&account.namespace),
            preferred_source: source_label(account.preferred_source),
            session: account
                .registration(account.preferred_source)
                .ok()
                .map(|registration| AzureAccountDetails::session_label(registration.session)),
            default_account: if is_default { "Yes" } else { "No" },
        }
    }
}
