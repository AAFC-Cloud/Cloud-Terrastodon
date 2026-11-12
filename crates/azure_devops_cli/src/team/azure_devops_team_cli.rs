use crate::team::list::AzureDevOpsTeamListArgs;
use crate::team::show::AzureDevOpsTeamShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps team-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTeamArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTeamCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsTeamCommand {
    /// List Azure DevOps teams in the project.
    List(AzureDevOpsTeamListArgs),
    /// Show details for a single Azure DevOps team.
    Show(AzureDevOpsTeamShowArgs),
}

impl AzureDevOpsTeamArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsTeamCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsTeamCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
