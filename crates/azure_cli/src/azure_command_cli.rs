use super::app_service::AzureAppServiceArgs;
use super::application_gateway::AzureApplicationGatewayArgs;
use super::audit_cli::AzureAuditArgs;
use super::cognitive_services::AzureCognitiveServicesArgs;
use super::container_instance::AzureContainerInstanceArgs;
use super::find_cli::AzureFindArgs;
use super::network_interface::AzureNetworkInterfaceArgs;
use super::pim::AzurePimArgs;
use super::policy::AzurePolicyArgs;
use super::private_endpoint::AzurePrivateEndpointArgs;
use super::public_ip::AzurePublicIpArgs;
use super::resource::AzureResourceArgs;
use super::resource_group::AzureResourceGroupArgs;
use super::role::AzureRoleArgs;
use super::storage::AzureStorageArgs;
use super::subscription::AzureSubscriptionArgs;
use super::tag::AzureTagArgs;
use super::tenant::AzureTenantArgs;
use super::virtual_network::AzureVirtualNetworkArgs;
use super::vm::AzureVmArgs;
use crate::entra::AzureEntraArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure_devops_cli::AzureDevOpsArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Arguments for Azure-specific operations.
#[derive(Facet, Debug, Clone)]
pub struct AzureArgs {
    #[facet(figue::subcommand)]
    pub command: AzureCommand,
}

impl AzureArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

/// Azure-specific commands.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureCommand {
    /// Manage Azure App Services.
    #[facet(figue::alias = "app")]
    AppService(AzureAppServiceArgs),
    /// Audit Azure resources for configuration issues.
    Audit(AzureAuditArgs),
    /// Manage Azure application gateways.
    #[facet(figue::alias = "agw")]
    ApplicationGateway(AzureApplicationGatewayArgs),
    /// Manage Azure Cognitive Services accounts and deployments.
    CognitiveServices(AzureCognitiveServicesArgs),
    /// Manage Azure Container Instances.
    #[facet(figue::alias = "aci")]
    ContainerInstance(AzureContainerInstanceArgs),
    /// Find resources where resource JSON contains the given text.
    Find(AzureFindArgs),
    /// Manage Azure network interfaces.
    #[facet(figue::alias = "nic")]
    NetworkInterface(AzureNetworkInterfaceArgs),
    /// List virtual networks and inspect their subnets.
    #[facet(figue::alias = "vnet")]
    VirtualNetwork(AzureVirtualNetworkArgs),
    /// Manage Azure resource groups.
    #[facet(figue::alias = "rg", figue::alias = "group")]
    ResourceGroup(AzureResourceGroupArgs),
    /// Manage Azure policy resources.
    Policy(AzurePolicyArgs),
    /// Manage Azure private endpoints.
    #[facet(figue::alias = "pe")]
    PrivateEndpoint(AzurePrivateEndpointArgs),
    /// Manage Azure public IP addresses.
    PublicIp(AzurePublicIpArgs),
    /// Manage Azure resource tags.
    Tag(AzureTagArgs),
    /// Manage Azure resources.
    #[facet(figue::alias = "res")]
    Resource(AzureResourceArgs),
    /// Manage Azure role-based access control.
    Role(AzureRoleArgs),
    /// Manage Azure Privileged Identity Management operations.
    Pim(AzurePimArgs),
    /// Entra (Azure AD) commands.
    #[facet(figue::alias = "ad")]
    Entra(AzureEntraArgs),
    /// VM-related commands (images, publishers, sizes, etc.)
    Vm(AzureVmArgs),
    /// Manage subscriptions across tracked tenants.
    #[facet(figue::alias = "sub")]
    Subscription(AzureSubscriptionArgs),
    /// Inspect Azure Storage accounts and services.
    Storage(AzureStorageArgs),
    /// Manage tracked tenants for later login flows.
    Tenant(AzureTenantArgs),
    /// Azure DevOps-specific commands.
    #[facet(figue::alias = "devops")]
    DevOps(AzureDevOpsArgs),
}

impl AzureCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        // Box child futures so large commands do not enlarge every dispatcher stack frame.
        match self {
            AzureCommand::DevOps(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Tenant(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::AppService(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Audit(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::ApplicationGateway(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::CognitiveServices(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::ContainerInstance(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Find(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::NetworkInterface(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::VirtualNetwork(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::ResourceGroup(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Policy(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::PrivateEndpoint(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::PublicIp(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Tag(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Resource(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Role(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Pim(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Entra(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Vm(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Subscription(args) => Box::pin(args.invoke(executor)).await,
            AzureCommand::Storage(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
