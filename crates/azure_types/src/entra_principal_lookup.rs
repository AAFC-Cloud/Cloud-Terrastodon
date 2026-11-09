use crate::EntraGroupId;
use crate::EntraPrincipalId;
use crate::EntraServicePrincipalObjectId;
use crate::EntraUserId;
use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

/// Select the Microsoft Graph entity route for a principal lookup.
///
/// A known kind selects [users](https://learn.microsoft.com/graph/api/user-get?view=graph-rest-1.0),
/// [groups](https://learn.microsoft.com/graph/api/group-get?view=graph-rest-1.0), or
/// [service principals](https://learn.microsoft.com/graph/api/serviceprincipal-get?view=graph-rest-1.0).
/// `Unknown` probes those routes. Unlike [`EntraPrincipalId`]'s UUID identity
/// representation, this selector serializes its kind so replay preserves the
/// requested route and cache identity.
#[derive(Debug, Clone, Copy, Arbitrary, Facet)]
#[repr(u8)]
pub enum EntraPrincipalLookup {
    User(EntraUserId),
    Group(EntraGroupId),
    ServicePrincipal(EntraServicePrincipalObjectId),
    Unknown(Uuid),
}

impl EntraPrincipalLookup {
    pub fn object_id(&self) -> &Uuid {
        match self {
            Self::User(id) => id.as_ref(),
            Self::Group(id) => id.as_ref(),
            Self::ServicePrincipal(id) => id.as_ref(),
            Self::Unknown(id) => id,
        }
    }
}

impl From<EntraPrincipalId> for EntraPrincipalLookup {
    fn from(id: EntraPrincipalId) -> Self {
        match id {
            EntraPrincipalId::UserId(id) => Self::User(id),
            EntraPrincipalId::GroupId(id) => Self::Group(id),
            EntraPrincipalId::ServicePrincipalId(id) => Self::ServicePrincipal(id),
            EntraPrincipalId::Unknown(id) => Self::Unknown(id),
        }
    }
}

impl From<EntraUserId> for EntraPrincipalLookup {
    fn from(id: EntraUserId) -> Self {
        Self::User(id)
    }
}

impl From<EntraGroupId> for EntraPrincipalLookup {
    fn from(id: EntraGroupId) -> Self {
        Self::Group(id)
    }
}

impl From<EntraServicePrincipalObjectId> for EntraPrincipalLookup {
    fn from(id: EntraServicePrincipalObjectId) -> Self {
        Self::ServicePrincipal(id)
    }
}

cloud_terrastodon_registry::register_thing!(EntraPrincipalLookup);
cloud_terrastodon_registry::register_arbitrary!(EntraPrincipalLookup);
cloud_terrastodon_registry::register_from!(EntraPrincipalId => EntraPrincipalLookup);
cloud_terrastodon_registry::register_from!(EntraUserId => EntraPrincipalLookup);
cloud_terrastodon_registry::register_from!(EntraGroupId => EntraPrincipalLookup);
cloud_terrastodon_registry::register_from!(EntraServicePrincipalObjectId => EntraPrincipalLookup);
