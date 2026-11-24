use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use facet::Facet;

/// Identity that opens the principal's directory page or its managed resource.
#[derive(Facet)]
#[repr(C)]
pub(crate) enum ScopeRoleAssignmentPrincipalLink {
    User(EntraUserId),
    Group(EntraGroupId),
    ManagedResource(ScopeImpl),
}

impl ScopeRoleAssignmentPrincipalLink {
    pub fn identifier(&self) -> String {
        match self {
            Self::User(id) => id.to_string(),
            Self::Group(id) => id.to_string(),
            Self::ManagedResource(scope) => scope.expanded_form(),
        }
    }

    pub fn navigation_target(&self) -> PortalTabTarget {
        match self {
            Self::User(id) => PortalTabTarget::EntraUser(*id),
            Self::Group(id) => PortalTabTarget::EntraGroup(*id),
            Self::ManagedResource(scope) => PortalTabTarget::from_arm_scope(scope),
        }
    }
}
