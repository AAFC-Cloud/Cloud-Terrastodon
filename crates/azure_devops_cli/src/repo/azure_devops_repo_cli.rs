use crate::repo::list::AzureDevOpsRepoListArgs;
use crate::repo::show::AzureDevOpsRepoShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsRepoCommand::List(args) => args.invoke(auth_context, executor).await?,
            AzureDevOpsRepoCommand::Show(args) => args.invoke(auth_context, executor).await?,
        }

        Ok(())
    }
}
