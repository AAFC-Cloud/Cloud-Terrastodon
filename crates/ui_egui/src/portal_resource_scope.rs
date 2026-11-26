use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::ResourceId;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_iconography_azure::icon_for_resource_type;
use facet::Facet;
use std::borrow::Cow;

/// Azure scopes with a dedicated detail workspace in the desktop app.
#[derive(Clone, Debug, Eq, Facet, PartialEq, Hash)]
#[repr(C)]
pub enum PortalResourceScope {
    ResourceGroup(AzureResourceGroupId),
    Subscription(SubscriptionId),
    ManagementGroup(ManagementGroupId),
    StorageAccount(AzureStorageAccountId),
    Resource(Box<ScopeImpl>),
}

impl PortalResourceScope {
    pub fn arm_scope(&self) -> ScopeImpl {
        match self {
            Self::ResourceGroup(id) => ScopeImpl::ResourceGroup(id.clone()),
            Self::Subscription(id) => ScopeImpl::Subscription(*id),
            Self::ManagementGroup(id) => ScopeImpl::ManagementGroup(id.clone()),
            Self::StorageAccount(id) => ScopeImpl::StorageAccount(id.clone()),
            Self::Resource(id) => id.as_ref().clone(),
        }
    }

    pub fn overview(&self) -> PortalTabTarget {
        match self {
            Self::ResourceGroup(id) => PortalTabTarget::ResourceGroup(id.clone()),
            Self::Subscription(id) => PortalTabTarget::Subscription(*id),
            Self::ManagementGroup(id) => PortalTabTarget::ManagementGroup(id.clone()),
            Self::StorageAccount(id) => PortalTabTarget::StorageAccount(id.clone()),
            Self::Resource(id) => PortalTabTarget::Resource(id.clone()),
        }
    }

    pub fn inventory(&self) -> PortalPage {
        match self {
            Self::ResourceGroup(_) => PortalPage::ResourceGroups,
            Self::Subscription(_) => PortalPage::Subscriptions,
            Self::ManagementGroup(_) => PortalPage::ManagementGroups,
            Self::StorageAccount(_) => PortalPage::StorageAccounts,
            Self::Resource(_) => PortalPage::Resources,
        }
    }

    pub(crate) fn inventory_target(&self, app: &MyApp) -> PortalTabTarget {
        if let Some(kind) = self.inventory_resource_kind(app) {
            return PortalTabTarget::ResourcesOfType(kind.clone());
        }
        PortalTabTarget::Page(self.inventory())
    }

    pub(crate) fn label(&self, app: &MyApp) -> Cow<'_, str> {
        match self {
            Self::ResourceGroup(_) => Cow::Borrowed("Resource group"),
            Self::Subscription(_) => Cow::Borrowed("Subscription"),
            Self::ManagementGroup(_) => Cow::Borrowed("Management group"),
            Self::StorageAccount(_) => Cow::Borrowed("Storage account"),
            Self::Resource(_) => {
                self.inventory_resource_kind(app)
                    .map_or(Cow::Borrowed("Resource"), |kind| {
                        icon_for_resource_type(kind)
                            .map_or_else(|| Cow::Owned(kind.to_string()), |icon| icon.name())
                    })
            }
        }
    }

    fn inventory_resource_kind<'a>(&'a self, app: &'a MyApp) -> Option<&'a ResourceType> {
        let Self::Resource(scope) = self else {
            return None;
        };
        network_security_group_id(scope)
            .map(|id| &id.resource_type)
            .or_else(|| app.inventory_resource(scope).map(|resource| &resource.kind))
    }

    pub(crate) fn output_target(&self) -> PortalResourceOutputTarget {
        match self {
            Self::StorageAccount(id) => PortalResourceOutputTarget::StorageAccount(id.clone()),
            scope => PortalResourceOutputTarget::Scope(scope.clone()),
        }
    }
}

pub(crate) fn network_security_group_id(scope: &ScopeImpl) -> Option<&ResourceId> {
    let ScopeImpl::Resource(id) = scope else {
        return None;
    };
    (id.resource_type == ResourceType::MICROSOFT_DOT_NETWORK_SLASH_NETWORKSECURITYGROUPS)
        .then_some(id)
}
