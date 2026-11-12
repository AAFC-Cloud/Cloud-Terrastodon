use crate::group::list::AzureDevOpsGroupListArgs;
use crate::group::show::AzureDevOpsGroupShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps group-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsGroupCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsGroupCommand {
    /// List Azure DevOps groups in the project.
    List(AzureDevOpsGroupListArgs),
    /// Show details for a single Azure DevOps group.
    Show(AzureDevOpsGroupShowArgs),
}

impl AzureDevOpsGroupArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsGroupCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsGroupCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
