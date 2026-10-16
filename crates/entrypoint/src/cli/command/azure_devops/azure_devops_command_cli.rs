use super::audit_cli::AzureDevOpsAuditArgs;
use super::azure_devops_rest_command_cli::AzureDevOpsRestArgs;
use crate::cli::azure_devops::agent::AzureDevOpsAgentArgs;
use crate::cli::azure_devops::build::AzureDevOpsBuildArgs;
use crate::cli::azure_devops::group::AzureDevOpsGroupArgs;
use crate::cli::azure_devops::license_entitlement::AzureDevOpsLicenseEntitlementArgs;
use crate::cli::azure_devops::project::AzureDevOpsProjectArgs;
use crate::cli::azure_devops::repo::AzureDevOpsRepoArgs;
use crate::cli::azure_devops::service_endpoint::AzureDevOpsServiceEndpointArgs;
use crate::cli::azure_devops::team::AzureDevOpsTeamArgs;
use crate::cli::azure_devops::test::AzureDevOpsTestArgs;
use crate::cli::azure_devops::work_item::AzureDevOpsWorkItemArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps-specific commands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsCommand {
    /// Build runs, pipeline definitions, and definition folders.
    Build(AzureDevOpsBuildArgs),
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
    /// Work items, queries, fields, and relations.
    WorkItem(AzureDevOpsWorkItemArgs),
    /// Test-related commands (e.g. test plan subcommands).
    Test(AzureDevOpsTestArgs),
}

impl AzureDevOpsCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<CliOutput> {
        // Box child futures so large commands do not enlarge every dispatcher stack frame.
        match self {
            AzureDevOpsCommand::Build(args) => {
                return Box::pin(args.invoke(auth_context, cancellation)).await;
            }
            AzureDevOpsCommand::Project(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
            AzureDevOpsCommand::Audit(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
            AzureDevOpsCommand::Rest(args) => Box::pin(args.invoke(cancellation)).await?,
            AzureDevOpsCommand::Group(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
            AzureDevOpsCommand::Team(args) => Box::pin(args.invoke(cancellation)).await?,
            AzureDevOpsCommand::Repo(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
            AzureDevOpsCommand::ServiceEndpoint(args) => {
                Box::pin(args.invoke(cancellation)).await?
            }
            AzureDevOpsCommand::LicenseEntitlement(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
            AzureDevOpsCommand::Agent(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
            AzureDevOpsCommand::WorkItem(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
            AzureDevOpsCommand::Test(args) => {
                Box::pin(args.invoke(auth_context, cancellation)).await?
            }
        }

        Ok(CliOutput::none())
    }
}
