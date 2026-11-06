use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_service_principals;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) service principals.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraSpListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraSpListArgs {
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
        info!("Fetching service principals");
        let sps = fetch_all_service_principals(
            tenant_auth_context.tenant_id,
            tenant_auth_context.auth_source()?,
            executor,
        )
        .await?;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for sp in sps.iter() {
            writeln!(
                out,
                "{} {:64} {} {}",
                sp.id,
                sp.display_name,
                sp.app_id,
                sp.service_principal_names.join(",")
            )?;
        }
        Ok(())
    }
}
