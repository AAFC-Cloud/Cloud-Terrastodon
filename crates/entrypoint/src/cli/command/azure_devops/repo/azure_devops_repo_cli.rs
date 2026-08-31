use crate::cli::azure_devops::repo::list::AzureDevOpsRepoListArgs;
use crate::cli::azure_devops::repo::show::AzureDevOpsRepoShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps repository-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsRepoArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsRepoCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsRepoCommand {
    /// List Azure DevOps repos in the project.
    List(AzureDevOpsRepoListArgs),
    /// Show details for a single Azure DevOps repo.
    Show(AzureDevOpsRepoShowArgs),
}

impl AzureDevOpsRepoArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsRepoCommand::List(args) => args.invoke(auth_context).await?,
            AzureDevOpsRepoCommand::Show(args) => args.invoke(auth_context).await?,
        }

        Ok(())
    }
}
