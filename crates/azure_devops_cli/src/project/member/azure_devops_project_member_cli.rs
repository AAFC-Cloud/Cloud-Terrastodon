use crate::project::member::list::AzureDevOpsProjectMemberListArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps project member-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsProjectMemberArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsProjectMemberCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsProjectMemberCommand {
    /// List users that are transitively members of the project.
    List(AzureDevOpsProjectMemberListArgs),
}

impl AzureDevOpsProjectMemberArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsProjectMemberCommand::List(args) => {
                args.invoke(auth_context, executor).await?
            }
        }

        Ok(())
    }
}
