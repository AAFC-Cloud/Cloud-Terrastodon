use crate::team::list::AzureDevOpsTeamListArgs;
use crate::team::show::AzureDevOpsTeamShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Azure DevOps team-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTeamArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTeamCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsTeamCommand {
    /// List Azure DevOps teams in the project.
    List(AzureDevOpsTeamListArgs),
    /// Show details for a single Azure DevOps team.
    Show(AzureDevOpsTeamShowArgs),
}

impl AzureDevOpsTeamArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsTeamCommand::List(args) => args.invoke(auth_context, executor).await?,
            AzureDevOpsTeamCommand::Show(args) => args.invoke(auth_context, executor).await?,
        }

        Ok(())
    }
}
