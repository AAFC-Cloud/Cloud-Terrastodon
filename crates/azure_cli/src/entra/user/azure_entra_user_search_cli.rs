use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::search_entra_users;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
use std::io::Write;
use tracing::info;

/// Search Entra (Azure AD) users by name, email address, or user principal name.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserSearchArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Search text. Matching is performed against the beginning of user name fields.
    #[facet(figue::positional)]
    pub search_term: String,
}

impl AzureEntraUserSearchArgs {
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
        info!(tenant_id = %tenant_auth_context.tenant_id, search_term = %self.search_term, "Searching Entra users");
        let users = search_entra_users(
            self.search_term,
            tenant_auth_context.tenant_id,
            tenant_auth_context.auth_source()?,
            executor,
        )
        .await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &users)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
