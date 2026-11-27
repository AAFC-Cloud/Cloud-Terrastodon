use super::AzureEntraServicePrincipalAppRoleAssignmentsArgs;
use super::AzureEntraServicePrincipalAppRoleCatalogueArgs;
use super::AzureEntraServicePrincipalDelegatedGrantsArgs;
use super::AzureEntraSpBrowseArgs;
use super::AzureEntraSpListArgs;
use super::AzureEntraSpShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Service principal-related Entra (Azure AD) commands.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraSpCommand {
    /// List service principals.
    List(AzureEntraSpListArgs),
    /// Show a service principal by object id, app id, display name, or SPN.
    Show(AzureEntraSpShowArgs),
    /// Browse service principals interactively.
    Browse(AzureEntraSpBrowseArgs),
    /// List app-only permissions granted to a service principal.
    AppRoleAssignments(AzureEntraServicePrincipalAppRoleAssignmentsArgs),
    /// Show the application roles exposed by a service principal.
    AppRoleCatalogue(AzureEntraServicePrincipalAppRoleCatalogueArgs),
    /// List OAuth2 delegated permissions granted to a service principal.
    DelegatedGrants(AzureEntraServicePrincipalDelegatedGrantsArgs),
}

impl AzureEntraSpCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraSpCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraSpCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraSpCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraSpCommand::AppRoleAssignments(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraSpCommand::AppRoleCatalogue(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraSpCommand::DelegatedGrants(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
