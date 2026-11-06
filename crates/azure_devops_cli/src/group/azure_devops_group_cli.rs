use crate::group::list::AzureDevOpsGroupListArgs;
use crate::group::show::AzureDevOpsGroupShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Azure DevOps group-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsGroupCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsGroupCommand {
    /// List Azure DevOps groups in the project.
    List(AzureDevOpsGroupListArgs),
    /// Show details for a single Azure DevOps group.
    Show(AzureDevOpsGroupShowArgs),
}

impl AzureDevOpsGroupArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsGroupCommand::List(args) => args.invoke(auth_context, executor).await?,
            AzureDevOpsGroupCommand::Show(args) => args.invoke(auth_context, executor).await?,
        }

        Ok(())
    }
}
