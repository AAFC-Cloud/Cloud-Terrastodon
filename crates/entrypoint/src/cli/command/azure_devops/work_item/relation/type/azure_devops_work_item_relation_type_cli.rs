use crate::cli::azure_devops::work_item::relation::r#type::list::AzureDevOpsWorkItemRelationTypeListArgs;
use crate::cli::azure_devops::work_item::relation::r#type::show::AzureDevOpsWorkItemRelationTypeShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Inspect relation type definitions.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationTypeArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemRelationTypeCommand,
}

#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemRelationTypeCommand {
    /// List definitions.
    List(AzureDevOpsWorkItemRelationTypeListArgs),
    /// Show a definition by name.
    Show(AzureDevOpsWorkItemRelationTypeShowArgs),
}

impl AzureDevOpsWorkItemRelationTypeArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemRelationTypeCommand::List(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemRelationTypeCommand::Show(args) => args.invoke(auth).await,
        }
    }
}
