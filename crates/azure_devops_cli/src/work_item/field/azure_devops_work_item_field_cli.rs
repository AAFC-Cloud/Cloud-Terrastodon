use crate::work_item::field::definition::AzureDevOpsWorkItemFieldDefinitionArgs;
use crate::work_item::field::list::AzureDevOpsWorkItemFieldListArgs;
use crate::work_item::field::remove::AzureDevOpsWorkItemFieldRemoveArgs;
use crate::work_item::field::set::AzureDevOpsWorkItemFieldSetArgs;
use crate::work_item::field::show::AzureDevOpsWorkItemFieldShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Read and write work item fields.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemFieldArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemFieldCommand,
}

#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemFieldCommand {
    /// List the selected item's field values.
    List(AzureDevOpsWorkItemFieldListArgs),
    /// Show one field value by reference name.
    Show(AzureDevOpsWorkItemFieldShowArgs),
    /// Set a field value using JSON, or @file containing JSON.
    Set(AzureDevOpsWorkItemFieldSetArgs),
    /// Remove a field value, subject to the work item rules.
    Remove(AzureDevOpsWorkItemFieldRemoveArgs),
    /// Inspect field definitions, including data types and read-only flags.
    Definition(AzureDevOpsWorkItemFieldDefinitionArgs),
}

impl AzureDevOpsWorkItemFieldArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsWorkItemFieldCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsWorkItemFieldCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsWorkItemFieldCommand::Set(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsWorkItemFieldCommand::Remove(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsWorkItemFieldCommand::Definition(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
