use crate::EntraDirectoryObjectType;
use crate::EntraGroupId;
use crate::EntraServicePrincipalObjectId;
use crate::EntraUserId;
use crate::PrincipalId;
use crate::PrincipalKind;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use facet_json::RawJson;
use std::collections::HashMap;

/// The subset of an Entra user returned by `directoryObjects/getByIds` that is
/// useful to callers resolving object IDs.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraDirectoryObjectUser {
    pub display_name: Option<String>,
    pub id: EntraUserId,
    pub mail: Option<String>,
    pub user_principal_name: Option<String>,
}

/// The subset of an Entra group returned by `directoryObjects/getByIds` that is
/// useful to callers resolving object IDs.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraDirectoryObjectGroup {
    pub display_name: Option<String>,
    pub id: EntraGroupId,
}

/// The subset of an Entra service principal returned by
/// `directoryObjects/getByIds` that is useful to callers resolving object IDs.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraDirectoryObjectServicePrincipal {
    pub display_name: Option<String>,
    pub id: EntraServicePrincipalObjectId,
}

/// A typed Entra directory object returned by Microsoft Graph.
///
/// The `@odata.type` discriminator is consumed while deserializing the raw
/// Graph response. The variants intentionally contain only the fields needed
/// for object-ID resolution because `getByIds` does not return full resource
/// representations.
#[derive(Debug, Clone, Eq, PartialEq, facet::Facet)]
#[facet(proxy = RawJson<'static>)]
#[repr(C)]
pub enum EntraDirectoryObject {
    User(EntraDirectoryObjectUser),
    Group(EntraDirectoryObjectGroup),
    ServicePrincipal(EntraDirectoryObjectServicePrincipal),
}

impl<'a> Arbitrary<'a> for EntraDirectoryObject {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0..=2)? {
            0 => Self::User(EntraDirectoryObjectUser::arbitrary(u)?),
            1 => Self::Group(EntraDirectoryObjectGroup::arbitrary(u)?),
            _ => Self::ServicePrincipal(EntraDirectoryObjectServicePrincipal::arbitrary(u)?),
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
    pub fn id(&self) -> PrincipalId {
        match self {
            Self::User(user) => PrincipalId::UserId(user.id),
            Self::Group(group) => PrincipalId::GroupId(group.id),
            Self::ServicePrincipal(service_principal) => {
                PrincipalId::ServicePrincipalId(service_principal.id)
            }
        }
    }

    pub fn kind(&self) -> PrincipalKind {
        self.object_type().principal_kind()
    }

    pub const fn object_type(&self) -> EntraDirectoryObjectType {
        match self {
            Self::User(..) => EntraDirectoryObjectType::User,
            Self::Group(..) => EntraDirectoryObjectType::Group,
            Self::ServicePrincipal(..) => EntraDirectoryObjectType::ServicePrincipal,
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
        }
    }
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObject);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObject);
cloud_terrastodon_registry::register_arbitrary!(Vec<EntraDirectoryObject>);

#[cfg(test)]
mod tests {
    use super::*;

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
