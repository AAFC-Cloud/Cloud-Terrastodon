use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::fetch_group_members;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List the members of an Entra (Azure AD) group.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraGroupMemberListArgs {
    /// Entra group id (GUID).
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraGroupMemberListArgs {
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
        info!(tenant_id = %tenant_auth_context.tenant_id, group_id = %self.group_id, "Fetching Entra group members");
        let members = fetch_group_members(self.group_id, &tenant_auth_context, executor).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &members)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
