use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryInvokeRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQuerySelector;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::bail;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemQueryInvokeArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Saved query UUID. Supply either this or --wiql.
    #[facet(figue::positional)]
    pub id: Option<String>,
    /// WIQL text, or @file containing WIQL.
    #[facet(figue::named)]
    pub wiql: Option<String>,
}

impl AzureDevOpsWorkItemQueryInvokeArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let query = match (self.id, self.wiql) {
            (Some(id), None) => AzureDevOpsWorkItemQuerySelector::Id(id.parse()?),
            (None, Some(wiql)) => AzureDevOpsWorkItemQuerySelector::Wiql(
                cloud_terrastodon_rest::read_optional_body(Some(wiql))
                    .await?
                    .ok_or_else(|| eyre::eyre!("Missing text input"))?,
            ),
            _ => bail!("Specify exactly one saved query ID or --wiql"),
        };
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let result = AzureDevOpsWorkItemQueryInvokeRequest {
            org_url: Cow::Owned(org_url),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            query,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &result)?;
        Ok(())
    }
}
