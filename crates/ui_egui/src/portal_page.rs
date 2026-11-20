use cloud_terrastodon_azure::ResourceType;

/// A destination in the GUI's service navigation. Resource destinations share
/// the inventory view while keeping their service filter explicit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortalPage {
    Home,
    Resources,
    Subscriptions,
    ResourceGroups,
    ManagementGroups,
    StorageAccounts,
    ContainerRegistries,
    KeyVaults,
    AzureDevOpsOrganizations,
    AzureDevOpsProjects,
    Accounts,
    AzureTenants,
    StartingPoints,
    SelectedItems,
    WorkDirs,
    FileDrop,
    Logs,
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
            Self::AzureDevOpsOrganizations => "DevOps organizations",
            Self::AzureDevOpsProjects => "DevOps projects",
            Self::Accounts => "Accounts",
            Self::AzureTenants => "Azure tenants",
            Self::StartingPoints => "Starting points",
            Self::SelectedItems => "Selected items",
            Self::WorkDirs => "Work directories",
            Self::FileDrop => "Import files",
            Self::Logs => "Logs",
        }
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
