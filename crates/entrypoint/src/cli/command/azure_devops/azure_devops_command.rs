use super::audit::AzureDevOpsAuditArgs;
use super::azure_devops_rest_command::AzureDevOpsRestArgs;
use crate::cli::azure_devops::agent::AzureDevOpsAgentArgs;
use crate::cli::azure_devops::group::AzureDevOpsGroupArgs;
use crate::cli::azure_devops::license_entitlement::AzureDevOpsLicenseEntitlementArgs;
use crate::cli::azure_devops::project::AzureDevOpsProjectArgs;
use crate::cli::azure_devops::repo::AzureDevOpsRepoArgs;
use crate::cli::azure_devops::service_endpoint::AzureDevOpsServiceEndpointArgs;
use crate::cli::azure_devops::team::AzureDevOpsTeamArgs;
use crate::cli::azure_devops::test::AzureDevOpsTestArgs;
use crate::cli::azure_devops::work_item_query::AzureDevOpsWorkItemQueryArgs;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps-specific commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsCommand {
    /// Audit Azure DevOps resources for configuration issues.
    Audit(AzureDevOpsAuditArgs),
    /// Issue raw Azure DevOps REST requests.
    Rest(AzureDevOpsRestArgs),
    /// Project-level operations (list, show, ...)
    Project(AzureDevOpsProjectArgs),
    /// Group-related operations.
    Group(AzureDevOpsGroupArgs),
    /// Team-related operations.
    Team(AzureDevOpsTeamArgs),
    /// Repository-related operations.
    Repo(AzureDevOpsRepoArgs),
    /// Service endpoint-related operations.
    ServiceEndpoint(AzureDevOpsServiceEndpointArgs),
    /// License entitlement-related operations.
    LicenseEntitlement(AzureDevOpsLicenseEntitlementArgs),
    /// Agent-related operations (e.g. package list).
    Agent(AzureDevOpsAgentArgs),
    /// Work item query operations.
    Query(AzureDevOpsWorkItemQueryArgs),
    /// Test-related commands (e.g. test plan subcommands).
    Test(AzureDevOpsTestArgs),
}

impl AzureDevOpsCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureDevOpsCommand::Audit(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Rest(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Project(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Group(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Team(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Repo(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::ServiceEndpoint(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::LicenseEntitlement(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Agent(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Query(args) => {
                args.invoke().await?;
            }
            AzureDevOpsCommand::Test(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
