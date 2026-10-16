use crate::cli::azure_devops::work_item::copy::AzureDevOpsWorkItemCopyArgs;
use crate::cli::azure_devops::work_item::create::AzureDevOpsWorkItemCreateArgs;
use crate::cli::azure_devops::work_item::field::AzureDevOpsWorkItemFieldArgs;
use crate::cli::azure_devops::work_item::list::AzureDevOpsWorkItemListArgs;
use crate::cli::azure_devops::work_item::query::AzureDevOpsWorkItemQueryArgs;
use crate::cli::azure_devops::work_item::relation::AzureDevOpsWorkItemRelationArgs;
use crate::cli::azure_devops::work_item::show::AzureDevOpsWorkItemShowArgs;
use crate::cli::azure_devops::work_item::r#type::AzureDevOpsWorkItemTypeArgs;
use crate::cli::azure_devops::work_item::update::AzureDevOpsWorkItemUpdateArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Work items, saved queries, fields, and relations.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemCommand,
}

#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemCommand {
    /// Fetch an explicit set of work item IDs.
    List(AzureDevOpsWorkItemListArgs),
    /// Show one work item's fields and relations.
    Show(AzureDevOpsWorkItemShowArgs),
    /// Create a work item with typed fields and optional relations.
    Create(AzureDevOpsWorkItemCreateArgs),
    /// Apply a JSON Patch document to one work item.
    Update(AzureDevOpsWorkItemUpdateArgs),
    /// Copy writable fields and optionally all descendant work items.
    Copy(AzureDevOpsWorkItemCopyArgs),
    /// Read and write field values, or inspect field definitions.
    Field(AzureDevOpsWorkItemFieldArgs),
    /// Read and write links, or inspect relation types.
    Relation(AzureDevOpsWorkItemRelationArgs),
    /// Inspect project work item types and their field constraints.
    Type(AzureDevOpsWorkItemTypeArgs),
    /// List, inspect, execute, and create saved work item queries.
    Query(AzureDevOpsWorkItemQueryArgs),
}

impl AzureDevOpsWorkItemArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemCommand::List(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemCommand::Show(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemCommand::Create(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemCommand::Update(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemCommand::Copy(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemCommand::Field(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemCommand::Relation(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemCommand::Type(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemCommand::Query(args) => args.invoke(auth, cancellation).await,
        }
    }
}
