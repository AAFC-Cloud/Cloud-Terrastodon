use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use cloud_terrastodon_azure::ScopeImpl;

/// The assignment scopes selectable relative to the resource being inspected.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum ScopeRoleAssignmentScopeFilter {
    #[default]
    All,
    Direct,
    ParentResource,
    ResourceGroup,
    Subscription,
    ManagementGroup,
    Tenant,
}

impl ScopeRoleAssignmentScopeFilter {
    pub const ALL: [Self; 7] = [
        Self::All,
        Self::Direct,
        Self::ParentResource,
        Self::ResourceGroup,
        Self::Subscription,
        Self::ManagementGroup,
        Self::Tenant,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All scopes",
            Self::Direct => "This scope",
            Self::ParentResource => "Parent resource",
            Self::ResourceGroup => "Resource group",
            Self::Subscription => "Subscription",
            Self::ManagementGroup => "Management group",
            Self::Tenant => "Tenant root",
        }
    }

    pub fn of(row: &ScopeRoleAssignmentRow) -> Self {
        if row.inheritance == "Direct" {
            return Self::Direct;
        }
        match &row.scope {
            ScopeImpl::ManagementGroup(_) => Self::ManagementGroup,
            ScopeImpl::Subscription(_) => Self::Subscription,
            ScopeImpl::ResourceGroup(_) => Self::ResourceGroup,
            ScopeImpl::Unknown(id) if id.as_str() == "/" => Self::Tenant,
            _ => Self::ParentResource,
        }
    }
}
