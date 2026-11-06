use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) principals.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraPrincipalListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraPrincipalListArgs {
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
        info!(tenant_id = %tenant_auth_context.tenant_id, "Fetching Entra principals");
        let principals = fetch_all_principals(&tenant_auth_context, executor).await?;
        let mut principals = principals.values().collect::<Vec<_>>();
        principals.sort_unstable_by_key(|principal| principal.id().to_string());

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &principals)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
