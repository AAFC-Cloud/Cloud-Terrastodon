use crate::portal_page::PortalPage;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_azure::Scope;
use facet::Facet;

/// A resource whose typed details can be rendered in the desktop output viewer.
#[derive(Clone, Debug, Eq, Facet, PartialEq)]
#[repr(C)]
pub enum PortalResourceOutputTarget {
    Scope(PortalResourceScope),
    StorageAccount(AzureStorageAccountId),
}

impl PortalResourceOutputTarget {
    pub fn overview(&self) -> PortalTabTarget {
        match self {
            Self::Scope(scope) => scope.overview(),
            Self::StorageAccount(id) => PortalTabTarget::StorageAccount(id.clone()),
        }
    }

    pub fn inventory(&self) -> PortalPage {
        match self {
            Self::Scope(scope) => scope.inventory(),
            Self::StorageAccount(_) => PortalPage::StorageAccounts,
        }
    }

    pub fn resource_id(&self) -> String {
        match self {
            Self::Scope(scope) => scope.arm_scope().expanded_form(),
            Self::StorageAccount(id) => id.expanded_form(),
        }
    }
}
