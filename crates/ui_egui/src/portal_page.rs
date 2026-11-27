use cloud_terrastodon_azure::ResourceType;
use facet::Facet;

/// A destination in the GUI's service navigation. Resource destinations share
/// the inventory view while keeping their service filter explicit.
#[derive(Clone, Copy, Debug, Eq, Facet, PartialEq)]
#[repr(u8)]
pub enum PortalPage {
    Home,
    Resources,
    Subscriptions,
    ResourceGroups,
    ManagementGroups,
    StorageAccounts,
    ContainerRegistries,
    KeyVaults,
    Entra,
    EntraUsers,
    EntraGroups,
    // The former global Projects page restores as the organization list.
    #[facet(alias = "AzureDevOpsProjects")]
    AzureDevOpsOrganizations,
    Accounts,
    AzureTenants,
    StartingPoints,
    SelectedItems,
    WorkDirs,
    FileDrop,
    Logs,
    About,
    History,
    Tags,
    EntraAppRegistrations,
    BackgroundWork,
    Auditing,
    AzureDevOpsAudit,
    StorageAccountAudit,
    Pim,
    PimEntraRoles,
    PimAzureResources,
    EntraMyUser,
}

impl PortalPage {
    pub fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Resources => "All resources",
            Self::Subscriptions => "Subscriptions",
            Self::ResourceGroups => "Resource groups",
            Self::ManagementGroups => "Management groups",
            Self::StorageAccounts => "Storage accounts",
            Self::ContainerRegistries => "Container registries",
            Self::KeyVaults => "Key vaults",
            Self::Entra => "Microsoft Entra",
            Self::EntraUsers => "Users",
            Self::EntraGroups => "Groups",
            Self::EntraAppRegistrations => "App registrations",
            Self::EntraMyUser => "My User",
            Self::AzureDevOpsOrganizations => "DevOps organizations",
            Self::Accounts => "Accounts",
            Self::AzureTenants => "Azure tenants",
            Self::StartingPoints => "Starting points",
            Self::SelectedItems => "Selected items",
            Self::WorkDirs => "Work directories",
            Self::FileDrop => "Import files",
            Self::Logs => "Logs",
            Self::BackgroundWork => "Background work",
            Self::About => "About",
            Self::History => "History",
            Self::Tags => "Tags",
            Self::Auditing => "Auditing",
            Self::AzureDevOpsAudit => "Azure DevOps audit",
            Self::StorageAccountAudit => "Storage account audit",
            Self::Pim => "Privileged Identity Management",
            Self::PimEntraRoles => "Microsoft Entra roles",
            Self::PimAzureResources => "Azure resources",
        }
    }

    pub(crate) fn is_entra(self) -> bool {
        matches!(
            self,
            Self::Entra
                | Self::EntraUsers
                | Self::EntraGroups
                | Self::EntraAppRegistrations
                | Self::EntraMyUser
        )
    }

    pub(crate) fn is_about_section(self) -> bool {
        matches!(
            self,
            Self::About | Self::Logs | Self::BackgroundWork | Self::Accounts | Self::AzureTenants
        )
    }

    pub(crate) fn is_audit_section(self) -> bool {
        matches!(
            self,
            Self::Auditing | Self::AzureDevOpsAudit | Self::StorageAccountAudit
        )
    }

    pub(crate) fn is_pim_section(self) -> bool {
        matches!(
            self,
            Self::Pim | Self::PimEntraRoles | Self::PimAzureResources
        )
    }

    pub fn resource_type(self) -> Option<ResourceType> {
        match self {
            Self::Subscriptions => Some(ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS),
            Self::ResourceGroups => {
                Some(ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS_SLASH_RESOURCEGROUPS)
            }
            Self::ManagementGroups => {
                Some(ResourceType::MICROSOFT_DOT_MANAGEMENT_SLASH_MANAGEMENTGROUPS)
            }
            Self::StorageAccounts => {
                Some(ResourceType::MICROSOFT_DOT_STORAGE_SLASH_STORAGEACCOUNTS)
            }
            Self::ContainerRegistries => {
                Some(ResourceType::MICROSOFT_DOT_CONTAINERREGISTRY_SLASH_REGISTRIES)
            }
            Self::KeyVaults => Some(ResourceType::MICROSOFT_DOT_KEYVAULT_SLASH_VAULTS),
            _ => None,
        }
    }
}
