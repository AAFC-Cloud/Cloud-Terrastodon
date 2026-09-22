use crate::cli::azure_devops::work_item::field::definition::AzureDevOpsWorkItemFieldDefinitionArgs;
use crate::cli::azure_devops::work_item::field::list::AzureDevOpsWorkItemFieldListArgs;
use crate::cli::azure_devops::work_item::field::remove::AzureDevOpsWorkItemFieldRemoveArgs;
use crate::cli::azure_devops::work_item::field::set::AzureDevOpsWorkItemFieldSetArgs;
use crate::cli::azure_devops::work_item::field::show::AzureDevOpsWorkItemFieldShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Read and write work item fields.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemFieldArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemFieldCommand,
}

#[derive(Debug, Clone, facet::Facet)]
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
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemFieldCommand::List(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemFieldCommand::Show(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemFieldCommand::Set(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemFieldCommand::Remove(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemFieldCommand::Definition(args) => args.invoke(auth).await,
        }
    }
}
