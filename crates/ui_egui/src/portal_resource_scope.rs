use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;

/// Azure scopes with a dedicated detail workspace in the desktop app.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PortalResourceScope {
    ResourceGroup(AzureResourceGroupId),
    Subscription(SubscriptionId),
}

impl PortalResourceScope {
    pub fn arm_scope(&self) -> ScopeImpl {
        match self {
            Self::ResourceGroup(id) => ScopeImpl::ResourceGroup(id.clone()),
            Self::Subscription(id) => ScopeImpl::Subscription(*id),
        }
    }

    pub fn overview(&self) -> PortalTabTarget {
        match self {
            Self::ResourceGroup(id) => PortalTabTarget::ResourceGroup(id.clone()),
            Self::Subscription(id) => PortalTabTarget::Subscription(*id),
        }
    }

    pub fn inventory(&self) -> PortalPage {
        match self {
            Self::ResourceGroup(_) => PortalPage::ResourceGroups,
            Self::Subscription(_) => PortalPage::Subscriptions,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::ResourceGroup(_) => "Resource group",
            Self::Subscription(_) => "Subscription",
        }
    }
}
