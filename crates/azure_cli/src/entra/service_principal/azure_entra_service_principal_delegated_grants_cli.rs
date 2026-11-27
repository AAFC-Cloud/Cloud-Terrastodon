use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraServicePrincipalOAuth2PermissionGrantListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List delegated permissions granted to an Entra service principal.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraServicePrincipalDelegatedGrantsArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Client service principal object ID.
    #[facet(figue::named, proxy = String)]
    pub service_principal_id: EntraServicePrincipalObjectId,

    /// Show grants for this user and grants available to every user in the tenant.
    #[facet(figue::named)]
    pub user_id: Option<EntraUserId>,

    /// Fetch fresh data from Microsoft Graph.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraServicePrincipalDelegatedGrantsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let mut grants = EntraServicePrincipalOAuth2PermissionGrantListRequest {
            authentication,
            service_principal_id: self.service_principal_id,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        if let Some(user_id) = self.user_id {
            grants.retain(|grant| grant.applies_to_user(user_id));
        }

        Ok(CliOutput::facet(grants))
    }
}
