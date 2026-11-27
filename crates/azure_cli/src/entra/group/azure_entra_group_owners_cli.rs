use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraGroupOwnerListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List the direct owners of an Entra group.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupOwnersArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Entra group object ID.
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    /// Fetch fresh data from Microsoft Graph.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraGroupOwnersArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let owners = EntraGroupOwnerListRequest {
            authentication,
            group_id: self.group_id,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;

        Ok(CliOutput::facet(owners))
    }
}
