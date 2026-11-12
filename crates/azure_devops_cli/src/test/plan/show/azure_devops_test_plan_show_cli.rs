use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsTestPlanListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

/// Show Azure DevOps test plan details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestPlanShowArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id or name.
    #[facet(figue::named)]
    pub plan: String,
}

impl AzureDevOpsTestPlanShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let plans = AzureDevOpsTestPlanListRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project),
            auth_selection,
        }
        .execute(executor)
        .await?;
        if let Some(plan) = plans
            .into_iter()
            .find(|p| p.name == self.plan || p.id.to_string() == self.plan)
        {
            Ok(CliOutput::facet(plan))
        } else {
            bail!("No test plan found matching '{}'.", self.plan);
        }
    }
}
