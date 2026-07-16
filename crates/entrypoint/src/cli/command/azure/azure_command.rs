use super::app_service::AzureAppServiceArgs;
use super::application_gateway::AzureApplicationGatewayArgs;
use super::audit::AzureAuditArgs;
use super::cognitive_services::AzureCognitiveServicesArgs;
use super::container_instance::AzureContainerInstanceArgs;
use super::find::AzureFindArgs;
use super::network_interface::AzureNetworkInterfaceArgs;
use super::pim::AzurePimArgs;
use super::policy::AzurePolicyArgs;
use super::private_endpoint::AzurePrivateEndpointArgs;
use super::public_ip::AzurePublicIpArgs;
use super::resource::AzureResourceArgs;
use super::resource_group::AzureResourceGroupArgs;
use super::role::AzureRoleArgs;
use super::subscription::AzureSubscriptionArgs;
use super::tag::AzureTagArgs;
use super::tenant::AzureTenantArgs;
use super::vm::AzureVmArgs;
use crate::cli::azure::entra::AzureEntraArgs;
use crate::cli::azure_devops::AzureDevOpsArgs;
use eyre::Result;

/// Azure-specific commands.
#[derive(facet::Facet, Debug, Clone)]
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
    /// Manage subscriptions within the tenant.
    #[facet(figue::alias = "sub")]
    Subscription(AzureSubscriptionArgs),
    /// Manage tracked tenants for later login flows.
    Tenant(AzureTenantArgs),
    /// Azure DevOps-specific commands.
    #[facet(figue::alias = "devops")]
    DevOps(AzureDevOpsArgs),
}

impl AzureCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureCommand::AppService(args) => {
                args.invoke().await?;
            }
            AzureCommand::Audit(args) => {
                args.invoke().await?;
            }
            AzureCommand::ApplicationGateway(args) => {
                args.invoke().await?;
            }
            AzureCommand::CognitiveServices(args) => {
                args.invoke().await?;
            }
            AzureCommand::ContainerInstance(args) => {
                args.invoke().await?;
            }
            AzureCommand::Find(args) => {
                args.invoke().await?;
            }
            AzureCommand::NetworkInterface(args) => {
                args.invoke().await?;
            }
            AzureCommand::ResourceGroup(args) => {
                args.invoke().await?;
            }
            AzureCommand::Policy(args) => {
                args.invoke().await?;
            }
            AzureCommand::PrivateEndpoint(args) => {
                args.invoke().await?;
            }
            AzureCommand::PublicIp(args) => {
                args.invoke().await?;
            }
            AzureCommand::Tag(args) => {
                args.invoke().await?;
            }
            AzureCommand::Resource(args) => {
                args.invoke().await?;
            }
            AzureCommand::Role(args) => {
                args.invoke().await?;
            }
            AzureCommand::Pim(args) => {
                args.invoke().await?;
            }
            AzureCommand::Entra(args) => {
                args.invoke().await?;
            }
            AzureCommand::Vm(args) => {
                args.invoke().await?;
            }
            AzureCommand::Subscription(args) => {
                args.invoke().await?;
            }
            AzureCommand::Tenant(args) => {
                args.invoke().await?;
            }
            AzureCommand::DevOps(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
