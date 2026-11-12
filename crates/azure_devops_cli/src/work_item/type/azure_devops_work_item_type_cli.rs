use crate::work_item::r#type::field::AzureDevOpsWorkItemTypeFieldArgs;
use crate::work_item::r#type::list::AzureDevOpsWorkItemTypeListArgs;
use crate::work_item::r#type::show::AzureDevOpsWorkItemTypeShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Inspect project work item types.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemTypeArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemTypeCommand,
}

#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemTypeCommand {
    /// List definitions.
    List(AzureDevOpsWorkItemTypeListArgs),
    /// Show a definition by name.
    Show(AzureDevOpsWorkItemTypeShowArgs),
    /// Inspect fields and constraints for a work item type.
    Field(AzureDevOpsWorkItemTypeFieldArgs),
}

impl AzureDevOpsWorkItemTypeArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsWorkItemTypeCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsWorkItemTypeCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsWorkItemTypeCommand::Field(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
