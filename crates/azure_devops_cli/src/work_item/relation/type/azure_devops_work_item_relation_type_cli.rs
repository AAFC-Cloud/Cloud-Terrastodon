use crate::work_item::relation::r#type::list::AzureDevOpsWorkItemRelationTypeListArgs;
use crate::work_item::relation::r#type::show::AzureDevOpsWorkItemRelationTypeShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Inspect relation type definitions.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemRelationTypeArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemRelationTypeCommand,
}

#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemRelationTypeCommand {
    /// List definitions.
    List(AzureDevOpsWorkItemRelationTypeListArgs),
    /// Show a definition by name.
    Show(AzureDevOpsWorkItemRelationTypeShowArgs),
}

impl AzureDevOpsWorkItemRelationTypeArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemRelationTypeCommand::List(args) => args.invoke(auth, executor).await,
            AzureDevOpsWorkItemRelationTypeCommand::Show(args) => args.invoke(auth, executor).await,
        }
    }
}
