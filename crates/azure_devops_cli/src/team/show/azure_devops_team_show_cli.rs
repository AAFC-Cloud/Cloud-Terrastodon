use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::io::stdout;

/// Show Azure DevOps team details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTeamShowArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

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
        let teams = fetch_azure_devops_teams_for_project(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            executor,
        )
        .await?;
        if let Some(team) = teams
            .into_iter()
            .find(|t| t.name == self.team || t.id.to_string() == self.team)
        {
            facet_json::to_writer_std_pretty(stdout(), &team)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            Ok(())
        } else {
            bail!("No team found matching '{}'.", self.team);
        }
    }
}
