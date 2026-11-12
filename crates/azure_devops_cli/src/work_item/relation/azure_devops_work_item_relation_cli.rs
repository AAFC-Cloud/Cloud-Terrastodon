use crate::work_item::relation::create::AzureDevOpsWorkItemRelationCreateArgs;
use crate::work_item::relation::list::AzureDevOpsWorkItemRelationListArgs;
use crate::work_item::relation::remove::AzureDevOpsWorkItemRelationRemoveArgs;
use crate::work_item::relation::show::AzureDevOpsWorkItemRelationShowArgs;
use crate::work_item::relation::r#type::AzureDevOpsWorkItemRelationTypeArgs;
use crate::work_item::relation::update::AzureDevOpsWorkItemRelationUpdateArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Read and write work item relations.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemRelationArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemRelationCommand,
}

#[derive(Debug, Clone, Facet)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsWorkItemRelationCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsWorkItemRelationCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsWorkItemRelationCommand::Create(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsWorkItemRelationCommand::Update(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsWorkItemRelationCommand::Remove(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureDevOpsWorkItemRelationCommand::Type(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
