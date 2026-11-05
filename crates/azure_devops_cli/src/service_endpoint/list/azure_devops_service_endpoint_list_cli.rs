use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_service_endpoints;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps service endpoints in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointListArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsServiceEndpointListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let endpoints = fetch_all_azure_devops_service_endpoints(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            cancellation,
        )
        .await?;
        facet_json::to_writer_std_pretty(stdout(), &endpoints)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}
