use arbitrary::Arbitrary;
use facet::Facet;

/// [Azure RBAC principal type](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01#principaltype).
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum RoleAssignmentPrincipalType {
    User,
    Group,
    ServicePrincipal,
    ForeignGroup,
    Device,
    AgentUser,
    AgentServicePrincipal,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for RoleAssignmentPrincipalType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "User" => Self::User,
            "Group" => Self::Group,
            "ServicePrincipal" => Self::ServicePrincipal,
            "ForeignGroup" => Self::ForeignGroup,
            "Device" => Self::Device,
            "AgentUser" => Self::AgentUser,
            "AgentServicePrincipal" => Self::AgentServicePrincipal,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&RoleAssignmentPrincipalType> for String {
    fn from(value: &RoleAssignmentPrincipalType) -> Self {
        match value {
            RoleAssignmentPrincipalType::User => "User",
            RoleAssignmentPrincipalType::Group => "Group",
            RoleAssignmentPrincipalType::ServicePrincipal => "ServicePrincipal",
            RoleAssignmentPrincipalType::ForeignGroup => "ForeignGroup",
            RoleAssignmentPrincipalType::Device => "Device",
            RoleAssignmentPrincipalType::AgentUser => "AgentUser",
            RoleAssignmentPrincipalType::AgentServicePrincipal => "AgentServicePrincipal",
            RoleAssignmentPrincipalType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(RoleAssignmentPrincipalType);
cloud_terrastodon_registry::register_arbitrary!(RoleAssignmentPrincipalType);
