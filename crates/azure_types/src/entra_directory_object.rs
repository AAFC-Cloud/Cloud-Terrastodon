use crate::EntraDirectoryObjectDevice;
use crate::EntraDirectoryObjectGroup;
use crate::EntraDirectoryObjectOrganizationalContact;
use crate::EntraDirectoryObjectServicePrincipal;
use crate::EntraDirectoryObjectType;
use crate::EntraDirectoryObjectUser;
use crate::EntraPrincipalId;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use facet_json::RawJson;
use std::collections::HashMap;

/// A shallow typed Microsoft Graph v1.0 [directory object](https://learn.microsoft.com/en-us/graph/api/resources/directoryobject?view=graph-rest-1.0).
///
/// The `@odata.type` discriminator is consumed while deserializing the raw
/// Graph response. Variants intentionally project only identity properties;
/// unrestricted Graph responses can contain additional fields. Limited-information
/// objects retain their IDs and types without manufacturing names. Other returned
/// object types cause an explicit decoding error, rather than being discarded and
/// mistaken for absent objects.
#[derive(Debug, Clone, Eq, PartialEq, facet::Facet)]
#[facet(proxy = RawJson<'static>)]
#[repr(C)]
pub enum EntraDirectoryObject {
    User(EntraDirectoryObjectUser),
    Group(EntraDirectoryObjectGroup),
    ServicePrincipal(EntraDirectoryObjectServicePrincipal),
    Device(EntraDirectoryObjectDevice),
    OrganizationalContact(EntraDirectoryObjectOrganizationalContact),
}

impl<'a> Arbitrary<'a> for EntraDirectoryObject {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0..=4)? {
            0 => Self::User(EntraDirectoryObjectUser::arbitrary(u)?),
            1 => Self::Group(EntraDirectoryObjectGroup::arbitrary(u)?),
            2 => Self::ServicePrincipal(EntraDirectoryObjectServicePrincipal::arbitrary(u)?),
            3 => Self::Device(EntraDirectoryObjectDevice::arbitrary(u)?),
            _ => Self::OrganizationalContact(EntraDirectoryObjectOrganizationalContact::arbitrary(
                u,
            )?),
        })
    }
}

impl TryFrom<RawJson<'static>> for EntraDirectoryObject {
    type Error = eyre::Error;

    fn try_from(value: RawJson<'static>) -> Result<Self, Self::Error> {
        let object = facet_json::from_str::<HashMap<String, RawJson<'static>>>(value.as_str())?;
        let odata_type = object
            .get("@odata.type")
            .ok_or_else(|| eyre::eyre!("Entra directory object missing @odata.type"))?;
        let odata_type = facet_json::from_str::<String>(odata_type.as_str())?;

        let object_type = EntraDirectoryObjectType::try_from_odata_type(&odata_type)?;
        Ok(match object_type {
            EntraDirectoryObjectType::User => Self::User(facet_json::from_str(value.as_str())?),
            EntraDirectoryObjectType::Group => Self::Group(facet_json::from_str(value.as_str())?),
            EntraDirectoryObjectType::ServicePrincipal => {
                Self::ServicePrincipal(facet_json::from_str(value.as_str())?)
            }
            EntraDirectoryObjectType::Device => Self::Device(facet_json::from_str(value.as_str())?),
            EntraDirectoryObjectType::OrganizationalContact => {
                Self::OrganizationalContact(facet_json::from_str(value.as_str())?)
            }
        })
    }
}

impl TryFrom<&EntraDirectoryObject> for RawJson<'static> {
    type Error = eyre::Error;

    fn try_from(value: &EntraDirectoryObject) -> Result<Self, Self::Error> {
        let inner_json = match value {
            EntraDirectoryObject::User(user) => facet_json::to_string(user)?,
            EntraDirectoryObject::Group(group) => facet_json::to_string(group)?,
            EntraDirectoryObject::ServicePrincipal(service_principal) => {
                facet_json::to_string(service_principal)?
            }
            EntraDirectoryObject::Device(device) => facet_json::to_string(device)?,
            EntraDirectoryObject::OrganizationalContact(contact) => facet_json::to_string(contact)?,
        };
        let mut object = facet_json::from_str::<HashMap<String, RawJson<'static>>>(&inner_json)?;
        object.insert(
            "@odata.type".to_string(),
            RawJson::from_owned(facet_json::to_string(value.object_type().odata_type())?),
        );
        Ok(RawJson::from_owned(facet_json::to_string(&object)?))
    }
}

impl EntraDirectoryObject {
    /// Return the directory identity through the existing UUID-based ID bridge.
    ///
    /// Devices and organizational contacts use `EntraPrincipalId::Unknown` because the
    /// full [`crate::EntraPrincipal`] model represents only users, groups, and service
    /// principals. In particular, a contact is not a security principal; carrying
    /// its UUID here does not imply that it can be assigned roles or ownership.
    pub fn id(&self) -> EntraPrincipalId {
        match self {
            Self::User(user) => EntraPrincipalId::UserId(user.id),
            Self::Group(group) => EntraPrincipalId::GroupId(group.id),
            Self::ServicePrincipal(service_principal) => {
                EntraPrincipalId::ServicePrincipalId(service_principal.id)
            }
            Self::Device(device) => EntraPrincipalId::Unknown(*device.id.as_ref()),
            Self::OrganizationalContact(contact) => EntraPrincipalId::Unknown(*contact.id.as_ref()),
        }
    }

    pub const fn kind(&self) -> EntraDirectoryObjectType {
        self.object_type()
    }

    pub const fn object_type(&self) -> EntraDirectoryObjectType {
        match self {
            Self::User(..) => EntraDirectoryObjectType::User,
            Self::Group(..) => EntraDirectoryObjectType::Group,
            Self::ServicePrincipal(..) => EntraDirectoryObjectType::ServicePrincipal,
            Self::Device(..) => EntraDirectoryObjectType::Device,
            Self::OrganizationalContact(..) => EntraDirectoryObjectType::OrganizationalContact,
        }
    }

    /// Return the name used when displaying this object to a user.
    pub fn display_name(&self) -> Option<&str> {
        match self {
            Self::User(user) => user
                .user_principal_name
                .as_deref()
                .or(user.mail.as_deref())
                .or(user.display_name.as_deref()),
            Self::Group(group) => group.display_name.as_deref(),
            Self::ServicePrincipal(service_principal) => service_principal.display_name.as_deref(),
            Self::Device(device) => device.display_name.as_deref(),
            Self::OrganizationalContact(contact) => {
                contact.display_name.as_deref().or(contact.mail.as_deref())
            }
        }
    }
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObject);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObject);
cloud_terrastodon_registry::register_arbitrary!(Vec<EntraDirectoryObject>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntraDeviceObjectId;
    use crate::EntraOrganizationalContactId;
    use uuid::Uuid;

    #[test]
    fn limited_information_objects_preserve_identity_without_inventing_names() {
        let device_id = Uuid::from_u128(1);
        let contact_id = Uuid::from_u128(2);
        let objects = [
            (
                EntraDirectoryObject::Device(EntraDirectoryObjectDevice {
                    id: EntraDeviceObjectId::new(device_id),
                    display_name: None,
                }),
                device_id,
                EntraDirectoryObjectType::Device,
            ),
            (
                EntraDirectoryObject::OrganizationalContact(
                    EntraDirectoryObjectOrganizationalContact {
                        id: EntraOrganizationalContactId::new(contact_id),
                        display_name: None,
                        mail: None,
                    },
                ),
                contact_id,
                EntraDirectoryObjectType::OrganizationalContact,
            ),
        ];

        for (object, id, object_type) in objects {
            assert_eq!(object.id(), id);
            assert_eq!(object.kind(), object_type);
            assert!(object.display_name().is_none());
            assert!(object.kind().principal_kind().is_none());
        }
    }

    #[test]
    fn an_unmodeled_returned_object_is_an_error_instead_of_an_absent_result() -> eyre::Result<()> {
        // Exercise the custom discriminator boundary, rather than manufacturing
        // an empty lookup result for a returned but unsupported directory type.
        let fields = HashMap::from([
            (
                "@odata.type",
                RawJson::from_owned(facet_json::to_string("#microsoft.graph.directoryRole")?),
            ),
            (
                "id",
                RawJson::from_owned(facet_json::to_string(&Uuid::from_u128(3))?),
            ),
        ]);
        let error =
            EntraDirectoryObject::try_from(RawJson::from_owned(facet_json::to_string(&fields)?))
                .expect_err("an unsupported returned object must fail the lookup");

        assert!(error.to_string().contains(
            "Unsupported Entra directory object @odata.type: #microsoft.graph.directoryRole"
        ));
        Ok(())
    }

    #[test]
    fn deserializes_typed_objects_from_odata_type() -> eyre::Result<()> {
        let user = facet_json::from_str::<EntraDirectoryObject>(
            r##"{
                "@odata.type": "#microsoft.graph.user",
                "id": "11111111-1111-1111-1111-111111111111",
                "displayName": "Example User",
                "mail": "example.mail@example.com",
                "userPrincipalName": "example.user@example.com"
            }"##,
        )?;
        assert!(matches!(user, EntraDirectoryObject::User(..)));
        assert_eq!(user.display_name(), Some("example.user@example.com"));

        let group = facet_json::from_str::<EntraDirectoryObject>(
            r##"{
                "@odata.type": "#microsoft.graph.group",
                "id": "22222222-2222-2222-2222-222222222222",
                "displayName": "Example Group"
            }"##,
        )?;
        assert!(matches!(group, EntraDirectoryObject::Group(..)));
        assert_eq!(group.display_name(), Some("Example Group"));

        let service_principal = facet_json::from_str::<EntraDirectoryObject>(
            r##"{
                "@odata.type": "#microsoft.graph.servicePrincipal",
                "id": "33333333-3333-3333-3333-333333333333",
                "displayName": "Example Service Principal"
            }"##,
        )?;
        assert!(matches!(
            service_principal,
            EntraDirectoryObject::ServicePrincipal(..)
        ));
        assert_eq!(
            service_principal.display_name(),
            Some("Example Service Principal")
        );

        Ok(())
    }

    #[test]
    fn falls_back_to_user_mail_when_upn_is_missing() -> eyre::Result<()> {
        let user = facet_json::from_str::<EntraDirectoryObject>(
            r##"{
                "@odata.type": "#microsoft.graph.user",
                "id": "11111111-1111-1111-1111-111111111111",
                "displayName": "Example User",
                "mail": "example.mail@example.com"
            }"##,
        )?;

        assert_eq!(user.display_name(), Some("example.mail@example.com"));
        Ok(())
    }

    #[test]
    fn falls_back_to_user_display_name_when_upn_and_mail_are_missing() -> eyre::Result<()> {
        let user = facet_json::from_str::<EntraDirectoryObject>(
            r##"{
                "@odata.type": "#microsoft.graph.user",
                "id": "11111111-1111-1111-1111-111111111111",
                "displayName": "Example User"
            }"##,
        )?;

        assert_eq!(user.display_name(), Some("Example User"));
        Ok(())
    }
}
