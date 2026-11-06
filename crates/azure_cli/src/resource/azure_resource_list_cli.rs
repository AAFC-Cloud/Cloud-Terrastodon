use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure resources.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureResourceListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureResourceListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        info!("Fetching all Azure resources");
        let resources = fetch_all_resources(&tenant_auth_context, executor).await?;
        info!(count = resources.len(), "Fetched Azure resources");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &resources)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
