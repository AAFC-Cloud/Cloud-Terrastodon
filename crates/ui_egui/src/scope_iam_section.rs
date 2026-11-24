/// Read-only inventories returned by Azure Authorization for a scope.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum ScopeIamSection {
    CheckAccess,
    #[default]
    RoleAssignments,
    Roles,
    DenyAssignments,
}

impl ScopeIamSection {
    pub const ALL: [Self; 4] = [
        Self::CheckAccess,
        Self::RoleAssignments,
        Self::Roles,
        Self::DenyAssignments,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::CheckAccess => "Check access",
            Self::RoleAssignments => "Role assignments",
            Self::Roles => "Roles",
            Self::DenyAssignments => "Deny assignments",
        }
    }
}
