use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
mod azure_entra_service_principal_app_role_assignments_cli;
mod azure_entra_service_principal_app_role_catalogue_cli;
pub mod azure_entra_service_principal_browse_cli;
pub mod azure_entra_service_principal_cli;
mod azure_entra_service_principal_delegated_grants_cli;
pub mod azure_entra_service_principal_list_cli;
pub mod azure_entra_service_principal_show_cli;

pub use azure_entra_service_principal_app_role_assignments_cli::AzureEntraServicePrincipalAppRoleAssignmentsArgs;
pub use azure_entra_service_principal_app_role_catalogue_cli::AzureEntraServicePrincipalAppRoleCatalogueArgs;
pub use azure_entra_service_principal_browse_cli::AzureEntraSpBrowseArgs;
pub use azure_entra_service_principal_cli::AzureEntraSpCommand;
pub use azure_entra_service_principal_delegated_grants_cli::AzureEntraServicePrincipalDelegatedGrantsArgs;
pub use azure_entra_service_principal_list_cli::AzureEntraSpListArgs;
pub use azure_entra_service_principal_show_cli::AzureEntraSpShowArgs;
use eyre::Result;

/// Entra service principal subcommands.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraServicePrincipalArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraSpCommand,
}

impl AzureEntraServicePrincipalArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
