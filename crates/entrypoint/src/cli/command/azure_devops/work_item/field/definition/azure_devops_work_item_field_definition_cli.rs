use crate::cli::azure_devops::work_item::field::definition::list::AzureDevOpsWorkItemFieldDefinitionListArgs;
use crate::cli::azure_devops::work_item::field::definition::show::AzureDevOpsWorkItemFieldDefinitionShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Inspect field definitions.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemFieldDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemFieldDefinitionCommand,
}

#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemFieldDefinitionCommand {
    /// List definitions.
    List(AzureDevOpsWorkItemFieldDefinitionListArgs),
    /// Show a definition by name.
    Show(AzureDevOpsWorkItemFieldDefinitionShowArgs),
}

impl AzureDevOpsWorkItemFieldDefinitionArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemFieldDefinitionCommand::List(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemFieldDefinitionCommand::Show(args) => args.invoke(auth).await,
        }
    }
}
