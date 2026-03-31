use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_application_gateways;
use cloud_terrastodon_azure::fetch_application_gateway_backend_health;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing backend health for a single Azure application gateway.
#[derive(Args, Debug, Clone)]
pub struct AzureApplicationGatewayShowBackendHealthArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Application gateway resource id or resource name.
    pub application_gateway: String,
}

impl AzureApplicationGatewayShowBackendHealthArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.application_gateway, %tenant_id, "Resolving Azure application gateway for backend health");
        let application_gateways = fetch_all_application_gateways(tenant_id).await?;

        let needle = self.application_gateway.trim();
        let mut matches = application_gateways
            .into_iter()
            .filter(|application_gateway| {
                application_gateway.id.expanded_form() == needle
                    || application_gateway.name.eq_ignore_ascii_case(needle)
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No application gateway found matching '{}'.", needle),
            1 => {
                let application_gateway = matches.remove(0);
                let backend_health =
                    fetch_application_gateway_backend_health(tenant_id, application_gateway.id)
                        .await?;
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &backend_health)?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|application_gateway| application_gateway.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|application_gateway| application_gateway.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple application gateways matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                )
            }
        }
    }
}
