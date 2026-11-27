use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupAuditLogListRequest;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List Entra directory audit events targeting this group.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupAuditLogArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Entra group object ID.
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    /// Fetch fresh data from Microsoft Graph.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraGroupAuditLogArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let events = EntraGroupAuditLogListRequest {
            authentication,
            group_id: self.group_id,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;

        Ok(CliOutput::facet(events))
    }
}
