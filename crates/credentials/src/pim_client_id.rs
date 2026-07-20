use crate::PimConfig;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_config::Config;
use eyre::Result;
use eyre::bail;

const PIM_CLIENT_ID_ENV: &str = "CLOUD_TERRASTODON_PIM_CLIENT_ID";

pub async fn pim_client_id(tenant_id: &AzureTenantId) -> Result<EntraApplicationClientId> {
    if let Ok(client_id) = std::env::var(PIM_CLIENT_ID_ENV) {
        if client_id.trim().is_empty() {
            bail!("{PIM_CLIENT_ID_ENV} must not be empty");
        }
        return Ok(client_id.parse()?);
    }

    let config = PimConfig::load().await?;
    if let Some(client_id) = config.client_id(tenant_id) {
        return Ok(client_id);
    }

    bail!(
        "{PIM_CLIENT_ID_ENV} is not set and no tenant-specific PIM app is configured; run `cloud_terrastodon az pim setup`"
    )
}
