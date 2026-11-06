use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_service_endpoints;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::io::stdout;

/// Show Azure DevOps service endpoint details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointShowArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;
        let endpoints = fetch_all_azure_devops_service_endpoints(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            executor,
        )
        .await?;
        if let Some(ep) = endpoints
            .iter()
            .find(|e| e.name.to_string() == self.endpoint || e.id.to_string() == self.endpoint)
        {
            facet_json::to_writer_std_pretty(stdout(), &ep)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            Ok(())
        } else {
            bail!("No service endpoint found matching '{}'.", self.endpoint);
        }
    }
}
