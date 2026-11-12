use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraGroupMemberAddRequest;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Add a member to an Entra (Azure AD) group.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupMemberAddArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Entra group id (GUID).
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    /// Principal id to add to the group (user/service-principal/group GUID).
    #[facet(figue::named, proxy = String)]
    pub member_id: EntraPrincipalId,
}

impl AzureEntraGroupMemberAddArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        EntraGroupMemberAddRequest {
            authentication,
            group_id: (self.group_id),
            member_id: (self.member_id),
        }
        .execute(executor)
        .await?;
        info!(group_id=%self.group_id, member_id=%self.member_id, "Added member to group");
        Ok(CliOutput::none())
    }
}
