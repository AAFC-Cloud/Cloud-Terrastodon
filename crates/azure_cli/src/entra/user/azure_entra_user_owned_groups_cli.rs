use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupListByOwnerRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List groups owned by an Entra user.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserOwnedGroupsArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Entra user object ID.
    #[facet(figue::named, proxy = String)]
    pub user_id: EntraUserId,

    /// Fetch fresh data from Microsoft Graph.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraUserOwnedGroupsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let groups = EntraGroupListByOwnerRequest {
            authentication,
            user_id: self.user_id,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;

        Ok(CliOutput::facet(groups))
    }
}
