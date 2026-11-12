use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupListByMemberRequest;
use cloud_terrastodon_azure::EntraGroupListRequest;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::sync::Arc;
use tracing::info;

/// List Entra (Azure AD) groups.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Only list groups containing this principal, including nested group membership.
    #[facet(figue::named)]
    pub for_member: Option<EntraPrincipalId>,
}

impl AzureEntraGroupListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let groups = match self.for_member {
            Some(principal_id) => {
                info!(tenant_id = %authentication.tenant_id, %principal_id, "Fetching Entra groups for principal");
                Arc::new(
                    EntraGroupListByMemberRequest {
                        principal_id: (principal_id),
                        authentication,
                    }
                    .execute(executor)
                    .await?,
                )
            }
            None => {
                info!(tenant_id = %authentication.tenant_id, "Fetching Entra groups");
                EntraGroupListRequest { authentication }
                    .execute(executor)
                    .await?
            }
        };

        Ok(CliOutput::facet(groups))
    }
}
