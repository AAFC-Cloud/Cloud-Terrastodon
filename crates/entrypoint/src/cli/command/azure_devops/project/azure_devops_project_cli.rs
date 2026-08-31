use crate::cli::azure_devops::project::dump::AzureDevOpsProjectDumpArgs;
use crate::cli::azure_devops::project::list::AzureDevOpsProjectListArgs;
use crate::cli::azure_devops::project::member::AzureDevOpsProjectMemberArgs;
use crate::cli::azure_devops::project::show::AzureDevOpsProjectShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps project-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsProjectCommand,
}

#[derive(facet::Facet, Debug, Clone)]
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsProjectCommand::List(args) => args.invoke(auth_context).await?,
            AzureDevOpsProjectCommand::Show(args) => args.invoke(auth_context).await?,
            AzureDevOpsProjectCommand::Dump(args) => args.invoke(auth_context).await?,
            AzureDevOpsProjectCommand::Member(args) => args.invoke(auth_context).await?,
        }

        Ok(())
    }
}
