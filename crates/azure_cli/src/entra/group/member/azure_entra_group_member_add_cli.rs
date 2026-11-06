use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_azure::add_group_member;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use tracing::info;

/// Add a member to an Entra (Azure AD) group.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraGroupMemberAddArgs {
    /// Tenant ID or tracked tenant alias for authentication.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
    /// Entra group id (GUID).
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    /// Principal id to add to the group (user/service-principal/group GUID).
    #[facet(figue::named, proxy = String)]
    pub member_id: EntraPrincipalId,
}

impl AzureEntraGroupMemberAddArgs {
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
        add_group_member(
            self.group_id,
            self.member_id,
            &tenant_auth_context,
            executor,
        )
        .await?;
        info!(group_id=%self.group_id, member_id=%self.member_id, "Added member to group");
        Ok(())
    }
}
