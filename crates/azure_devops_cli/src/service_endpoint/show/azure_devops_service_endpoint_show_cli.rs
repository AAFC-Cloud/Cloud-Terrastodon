use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsServiceEndpointsListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

/// Show Azure DevOps service endpoint details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointShowArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Service endpoint id or service endpoint name.
    #[facet(figue::named)]
    pub endpoint: String,
}

impl AzureDevOpsServiceEndpointShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let endpoints = AzureDevOpsServiceEndpointsListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
            project: (self.project),
        }
        .execute(executor)
        .await?;
        if let Some(ep) = endpoints
            .iter()
            .find(|e| e.name.to_string() == self.endpoint || e.id.to_string() == self.endpoint)
        {
            Ok(CliOutput::facet(ep.clone()))
        } else {
            bail!("No service endpoint found matching '{}'.", self.endpoint);
        }
    }
}
