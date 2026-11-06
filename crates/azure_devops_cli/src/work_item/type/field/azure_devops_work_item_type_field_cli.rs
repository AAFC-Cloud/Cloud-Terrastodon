use crate::work_item::r#type::field::list::AzureDevOpsWorkItemTypeFieldListArgs;
use crate::work_item::r#type::field::show::AzureDevOpsWorkItemTypeFieldShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Inspect the fields of a work item type.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemTypeFieldArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemTypeFieldCommand,
}

#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemTypeFieldCommand {
    /// List fields and constraints for a work item type.
    List(AzureDevOpsWorkItemTypeFieldListArgs),
    /// Show one field and its constraints for a work item type.
    Show(AzureDevOpsWorkItemTypeFieldShowArgs),
}

impl AzureDevOpsWorkItemTypeFieldArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemTypeFieldCommand::List(args) => args.invoke(auth, executor).await,
            AzureDevOpsWorkItemTypeFieldCommand::Show(args) => args.invoke(auth, executor).await,
        }
    }
}
