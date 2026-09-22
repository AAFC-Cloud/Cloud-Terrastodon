use crate::cli::azure_devops::work_item::r#type::field::AzureDevOpsWorkItemTypeFieldArgs;
use crate::cli::azure_devops::work_item::r#type::list::AzureDevOpsWorkItemTypeListArgs;
use crate::cli::azure_devops::work_item::r#type::show::AzureDevOpsWorkItemTypeShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Inspect project work item types.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemTypeArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemTypeCommand,
}

#[derive(Debug, Clone, facet::Facet)]
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
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemTypeCommand::List(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemTypeCommand::Show(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemTypeCommand::Field(args) => args.invoke(auth).await,
        }
    }
}
