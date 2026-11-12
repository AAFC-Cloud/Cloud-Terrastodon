use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraGroupMembersListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// List the members of an Entra (Azure AD) group.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupMemberListArgs {
    /// Entra group id (GUID).
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraGroupMemberListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, group_id = %self.group_id, "Fetching Entra group members");
        let members = EntraGroupMembersListRequest {
            group_id: (self.group_id),
            authentication,
        }
        .execute(executor)
        .await?;

        Ok(CliOutput::facet(members))
    }
}
