use crate::project::dump::AzureDevOpsProjectDumpArgs;
use crate::project::list::AzureDevOpsProjectListArgs;
use crate::project::member::AzureDevOpsProjectMemberArgs;
use crate::project::show::AzureDevOpsProjectShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps project-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsProjectArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsProjectCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsProjectCommand {
    /// List Azure DevOps projects in the organization.
    List(AzureDevOpsProjectListArgs),
    /// Show details for a single Azure DevOps project by id or name.
    Show(AzureDevOpsProjectShowArgs),
    /// Dump details for a single Azure DevOps project by id or name.
    Dump(AzureDevOpsProjectDumpArgs),
    /// Project member operations.
    Member(AzureDevOpsProjectMemberArgs),
}

impl AzureDevOpsProjectArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsProjectCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsProjectCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsProjectCommand::Dump(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsProjectCommand::Member(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
