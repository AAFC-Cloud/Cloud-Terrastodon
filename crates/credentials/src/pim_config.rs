use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::{AzureTenantId, EntraApplicationClientId};
use cloud_terrastodon_config::Config;
use std::collections::HashMap;

/// Non-sensitive configuration for the Cloud Terrastodon PIM integration.
#[derive(Debug, Default, Arbitrary, facet::Facet, Clone, PartialEq)]
pub struct PimConfig {
    /// PIM application client ids keyed by Entra tenant id.
    pub client_ids: HashMap<AzureTenantId, EntraApplicationClientId>,
}

impl PimConfig {
    pub fn client_id(&self, tenant_id: &AzureTenantId) -> Option<EntraApplicationClientId> {
        self.client_ids.get(tenant_id).copied()
    }

    pub fn set_client_id(
        &mut self,
        tenant_id: &AzureTenantId,
        client_id: EntraApplicationClientId,
    ) {
        self.client_ids.insert(*tenant_id, client_id);
    }
}

#[async_trait::async_trait]
impl Config for PimConfig {
    const FILE_SLUG: &'static str = "pim";
}

cloud_terrastodon_registry::register_thing!(PimConfig);
cloud_terrastodon_registry::register_arbitrary!(PimConfig);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_tenant_keyed_client_ids() -> eyre::Result<()> {
        let tenant_id = AzureTenantId::new(cloud_terrastodon_azure_types::uuid::Uuid::nil());
        let client_id =
            EntraApplicationClientId::new(cloud_terrastodon_azure_types::uuid::Uuid::from_u128(1));
        let mut config = PimConfig::default();
        config.set_client_id(&tenant_id, client_id);

        let json = facet_json::to_string(&config)?;
        let decoded = facet_json::from_str::<PimConfig>(&json)?;

        assert_eq!(decoded, config);
        Ok(())
    }
}
