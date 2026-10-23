use crate::work_item::query::create::AzureDevOpsWorkItemQueryCreateArgs;
use crate::work_item::query::invoke::AzureDevOpsWorkItemQueryInvokeArgs;
use crate::work_item::query::list::AzureDevOpsWorkItemQueryListArgs;
use crate::work_item::query::show::AzureDevOpsWorkItemQueryShowArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// List, inspect, execute, and create work item queries.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemQueryArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemQueryCommand,
}

#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemQueryCommand {
    /// List saved queries and folders in a project.
    List(AzureDevOpsWorkItemQueryListArgs),
    /// Show a query definition, including its WIQL.
    Show(AzureDevOpsWorkItemQueryShowArgs),
    /// Execute a saved query ID or supplied WIQL; does not create a query.
    Invoke(AzureDevOpsWorkItemQueryInvokeArgs),
    /// Create a saved query in an existing folder.
    Create(AzureDevOpsWorkItemQueryCreateArgs),
}

impl AzureDevOpsWorkItemQueryArgs {
    pub async fn invoke(self, auth: &AuthContext, cancellation: &CancellationScope) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemQueryCommand::List(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemQueryCommand::Show(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemQueryCommand::Invoke(args) => args.invoke(auth, cancellation).await,
            AzureDevOpsWorkItemQueryCommand::Create(args) => args.invoke(auth, cancellation).await,
        }
    }
}
