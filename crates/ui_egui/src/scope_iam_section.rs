/// Read-only inventories returned by Azure Authorization for a scope.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum ScopeIamSection {
    #[default]
    RoleAssignments,
    Roles,
    DenyAssignments,
}

impl ScopeIamSection {
    pub const ALL: [Self; 3] = [Self::RoleAssignments, Self::Roles, Self::DenyAssignments];

    pub fn label(self) -> &'static str {
        match self {
            Self::RoleAssignments => "Role assignments",
            Self::Roles => "Roles",
            Self::DenyAssignments => "Deny assignments",
        }
    }
}
