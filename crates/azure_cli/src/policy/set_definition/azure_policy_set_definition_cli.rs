use super::AzurePolicySetDefinitionBrowseArgs;
use super::AzurePolicySetDefinitionListArgs;
use super::AzurePolicySetDefinitionShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for managing Azure policy set definitions.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePolicySetDefinitionCommand {
    /// List all Azure policy set definitions accessible to the account.
    List(AzurePolicySetDefinitionListArgs),
    /// Browse Azure policy set definitions in an interactive manner.
    Browse(AzurePolicySetDefinitionBrowseArgs),
    /// Show a single Azure policy set definition by id, name, or display name.
    Show(AzurePolicySetDefinitionShowArgs),
}

impl AzurePolicySetDefinitionCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzurePolicySetDefinitionCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzurePolicySetDefinitionCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzurePolicySetDefinitionCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
