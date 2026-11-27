use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantId;
use facet::Facet;
use itertools::Itertools;

/// Display fields for a tracked tenant, with its exact ID for navigation.
#[derive(Facet)]
pub(crate) struct AzureTrackedTenantRow {
    pub tenant_id: AzureTenantId,
    pub aliases: String,
    pub default_tenant: Option<&'static str>,
    pub default_account: Option<CloudTerrastodonAccountAlias>,
}

impl AzureTrackedTenantRow {
    pub fn new(
        tenant_id: AzureTenantId,
        aliases: &[AzureTenantAlias],
        is_default: bool,
        default_account: Option<CloudTerrastodonAccountAlias>,
    ) -> Self {
        Self {
            tenant_id,
            aliases: aliases.iter().map(|alias| alias.as_str()).join(", "),
            default_tenant: is_default.then_some("Default"),
            default_account,
        }
    }
}
