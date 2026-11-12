use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsTeamsForProjectRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

/// Show Azure DevOps team details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTeamShowArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Team id (UUID) or team name.
    #[facet(figue::named)]
    pub team: String,
}

impl AzureDevOpsTeamShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let teams = AzureDevOpsTeamsForProjectRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
            project: (self.project),
        }
        .execute(executor)
        .await?;
        if let Some(team) = teams
            .into_iter()
            .find(|t| t.name == self.team || t.id.to_string() == self.team)
        {
            Ok(CliOutput::facet(team))
        } else {
            bail!("No team found matching '{}'.", self.team);
        }
    }
}
