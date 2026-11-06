use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraUserArgument;
use cloud_terrastodon_azure::fetch_entra_user;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Show a single Entra (Azure AD) user.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraUserShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// User object id or user principal name.
    #[facet(figue::positional)]
    pub user: EntraUserArgument<'static>,
}

impl AzureEntraUserShowArgs {
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
        info!(needle = %self.user, tenant_id = %tenant_auth_context.tenant_id, "Fetching user");

        let user = fetch_entra_user(
            self.user,
            tenant_auth_context.tenant_id,
            tenant_auth_context.auth_source()?,
            executor,
        )
        .await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &user)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
