use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantId;
use facet::Facet;
use itertools::Itertools;

/// Display fields for a tracked tenant, with its exact ID for navigation.
#[derive(Facet)]
pub(crate) struct AzureTrackedTenantRow {
    pub tenant_id: AzureTenantId,
    pub aliases: String,
    pub default: Option<&'static str>,
}

impl AzureTrackedTenantRow {
    pub fn new(tenant_id: AzureTenantId, aliases: &[AzureTenantAlias], default: bool) -> Self {
        Self {
            tenant_id,
            aliases: aliases.iter().map(|alias| alias.as_str()).join(", "),
            default: default.then_some("Default"),
        }
    }
}
