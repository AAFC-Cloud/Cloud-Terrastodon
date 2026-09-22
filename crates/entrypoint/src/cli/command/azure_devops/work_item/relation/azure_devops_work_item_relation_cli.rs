use crate::cli::azure_devops::work_item::relation::create::AzureDevOpsWorkItemRelationCreateArgs;
use crate::cli::azure_devops::work_item::relation::list::AzureDevOpsWorkItemRelationListArgs;
use crate::cli::azure_devops::work_item::relation::remove::AzureDevOpsWorkItemRelationRemoveArgs;
use crate::cli::azure_devops::work_item::relation::show::AzureDevOpsWorkItemRelationShowArgs;
use crate::cli::azure_devops::work_item::relation::r#type::AzureDevOpsWorkItemRelationTypeArgs;
use crate::cli::azure_devops::work_item::relation::update::AzureDevOpsWorkItemRelationUpdateArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Read and write work item relations.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemRelationCommand,
}

#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemRelationCommand {
    /// List links without fetching their targets.
    List(AzureDevOpsWorkItemRelationListArgs),
    /// Select one link by relation type and target.
    Show(AzureDevOpsWorkItemRelationShowArgs),
    /// Add a link to an existing work item.
    Create(AzureDevOpsWorkItemRelationCreateArgs),
    /// Change a selected link's writable attributes.
    Update(AzureDevOpsWorkItemRelationUpdateArgs),
    /// Remove a selected link using a revision-checked patch.
    Remove(AzureDevOpsWorkItemRelationRemoveArgs),
    /// Inspect relation type definitions and directions.
    Type(AzureDevOpsWorkItemRelationTypeArgs),
}

impl AzureDevOpsWorkItemRelationArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemRelationCommand::List(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemRelationCommand::Show(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemRelationCommand::Create(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemRelationCommand::Update(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemRelationCommand::Remove(args) => args.invoke(auth).await,
            AzureDevOpsWorkItemRelationCommand::Type(args) => args.invoke(auth).await,
        }
    }
}
