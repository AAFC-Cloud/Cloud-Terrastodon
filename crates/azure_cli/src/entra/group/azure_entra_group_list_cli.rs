use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_azure::fetch_all_groups;
use cloud_terrastodon_azure::fetch_entra_groups_for_member;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
use std::io::Write;
use std::sync::Arc;
use tracing::info;

/// List Entra (Azure AD) groups.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Only list groups containing this principal, including nested group membership.
    #[facet(figue::named)]
    pub for_member: Option<EntraPrincipalId>,
}

impl AzureEntraGroupListArgs {
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
        let groups = match self.for_member {
            Some(principal_id) => {
                info!(tenant_id = %tenant_auth_context.tenant_id, %principal_id, "Fetching Entra groups for principal");
                Arc::new(
                    fetch_entra_groups_for_member(principal_id, &tenant_auth_context, executor)
                        .await?,
                )
            }
            None => {
                info!(tenant_id = %tenant_auth_context.tenant_id, "Fetching Entra groups");
                fetch_all_groups(
                    tenant_auth_context.tenant_id,
                    tenant_auth_context.auth_source()?,
                    executor,
                )
                .await?
            }
        };

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &groups)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
