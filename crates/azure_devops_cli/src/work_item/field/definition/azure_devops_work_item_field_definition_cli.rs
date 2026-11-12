use crate::work_item::field::definition::list::AzureDevOpsWorkItemFieldDefinitionListArgs;
use crate::work_item::field::definition::show::AzureDevOpsWorkItemFieldDefinitionShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Inspect field definitions.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemFieldDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemFieldDefinitionCommand,
}

#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemFieldDefinitionCommand {
    /// List definitions.
    List(AzureDevOpsWorkItemFieldDefinitionListArgs),
    /// Show a definition by name.
    Show(AzureDevOpsWorkItemFieldDefinitionShowArgs),
}

impl AzureDevOpsWorkItemFieldDefinitionArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsWorkItemFieldDefinitionCommand::List(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsWorkItemFieldDefinitionCommand::Show(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
