use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_entra_group_cli;
pub mod azure_entra_group_list_cli;
pub mod azure_entra_group_show_cli;
mod azure_entra_group_show_lookup;
mod azure_entra_group_show_lookup_result;
pub mod member;

pub use azure_entra_group_cli::AzureEntraGroupCommand;
pub use azure_entra_group_list_cli::AzureEntraGroupListArgs;
pub use azure_entra_group_show_cli::AzureEntraGroupShowArgs;
use cloud_terrastodon_app::CliOutput;
use eyre::Result;
pub use member::AzureEntraGroupMemberArgs;

/// Entra group subcommands
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraGroupCommand,
}

impl AzureEntraGroupArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
