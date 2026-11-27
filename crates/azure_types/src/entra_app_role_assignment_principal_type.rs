use arbitrary::Arbitrary;
use facet::Facet;

/// Principal categories reported by a Graph [appRoleAssignment](https://learn.microsoft.com/graph/api/resources/approleassignment?view=graph-rest-1.0#properties).
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum EntraAppRoleAssignmentPrincipalType {
    User,
    Group,
    ServicePrincipal,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for EntraAppRoleAssignmentPrincipalType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "User" => Self::User,
            "Group" => Self::Group,
            "ServicePrincipal" => Self::ServicePrincipal,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&EntraAppRoleAssignmentPrincipalType> for String {
    fn from(value: &EntraAppRoleAssignmentPrincipalType) -> Self {
        match value {
            EntraAppRoleAssignmentPrincipalType::User => "User",
            EntraAppRoleAssignmentPrincipalType::Group => "Group",
            EntraAppRoleAssignmentPrincipalType::ServicePrincipal => "ServicePrincipal",
            EntraAppRoleAssignmentPrincipalType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(EntraAppRoleAssignmentPrincipalType);
cloud_terrastodon_registry::register_arbitrary!(EntraAppRoleAssignmentPrincipalType);
