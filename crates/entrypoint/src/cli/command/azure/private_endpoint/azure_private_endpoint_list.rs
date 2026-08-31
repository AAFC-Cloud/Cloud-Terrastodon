use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_private_endpoints;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure private endpoints.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePrivateEndpointListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePrivateEndpointListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Azure private endpoints");
        let private_endpoints = fetch_all_private_endpoints(tenant_id, auth_context).await?;
        info!(
            count = private_endpoints.len(),
            "Fetched Azure private endpoints"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &private_endpoints)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
