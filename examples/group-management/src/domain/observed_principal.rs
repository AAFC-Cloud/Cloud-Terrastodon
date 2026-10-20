use cloud_terrastodon::azure_types::EntraDirectoryObject;
use cloud_terrastodon::azure_types::EntraDirectoryObjectType;
use cloud_terrastodon::azure_types::EntraDirectoryObjectUser;
use cloud_terrastodon::azure_types::EntraPrincipalId;
use cloud_terrastodon::azure_types::EntraUser;
use cloud_terrastodon::azure_types::UserPrincipalName;

/// Identity and readable properties observed for one directory object.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ObservedPrincipal {
    pub id: EntraPrincipalId,
    pub kind: EntraDirectoryObjectType,
    pub display_name: Option<String>,
    pub user_principal_name: Option<UserPrincipalName>,
}

impl From<EntraDirectoryObjectUser> for ObservedPrincipal {
    fn from(user: EntraDirectoryObjectUser) -> Self {
        Self {
            id: user.id.into(),
            kind: EntraDirectoryObjectType::User,
            display_name: user.display_name,
            user_principal_name: user.user_principal_name,
        }
    }
}

impl From<EntraUser> for ObservedPrincipal {
    fn from(user: EntraUser) -> Self {
        Self {
            id: user.id.into(),
            kind: EntraDirectoryObjectType::User,
            display_name: Some(user.display_name),
            user_principal_name: Some(user.user_principal_name),
        }
    }
}

impl From<EntraDirectoryObject> for ObservedPrincipal {
    fn from(object: EntraDirectoryObject) -> Self {
        match object {
            EntraDirectoryObject::User(user) => user.into(),
            EntraDirectoryObject::Group(group) => Self {
                id: group.id.into(),
                kind: EntraDirectoryObjectType::Group,
                display_name: group.display_name.map(String::from),
                user_principal_name: None,
            },
            EntraDirectoryObject::ServicePrincipal(service_principal) => Self {
                id: service_principal.id.into(),
                kind: EntraDirectoryObjectType::ServicePrincipal,
                display_name: service_principal.display_name,
                user_principal_name: None,
            },
            EntraDirectoryObject::Device(device) => Self {
                id: EntraPrincipalId::new(*device.id.as_ref()),
                kind: EntraDirectoryObjectType::Device,
                display_name: device.display_name,
                user_principal_name: None,
            },
            EntraDirectoryObject::OrganizationalContact(contact) => Self {
                id: EntraPrincipalId::new(*contact.id.as_ref()),
                kind: EntraDirectoryObjectType::OrganizationalContact,
                display_name: contact.display_name,
                user_principal_name: None,
            },
        }
    }
}
