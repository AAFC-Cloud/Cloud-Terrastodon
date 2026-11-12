use super::AzurePolicyDefinitionBrowseArgs;
use super::AzurePolicyDefinitionListArgs;
use super::AzurePolicyDefinitionShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for managing Azure policy definitions.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePolicyDefinitionCommand {
    /// List all Azure policy definitions accessible to the account.
    List(AzurePolicyDefinitionListArgs),
    /// Browse Azure policy definitions in an interactive manner.
    Browse(AzurePolicyDefinitionBrowseArgs),
    /// Show a single Azure policy definition by id, name, or display name.
    Show(AzurePolicyDefinitionShowArgs),
}

impl AzurePolicyDefinitionCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzurePolicyDefinitionCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzurePolicyDefinitionCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzurePolicyDefinitionCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
