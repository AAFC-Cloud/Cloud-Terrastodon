use crate::EntraGroupId;
use crate::EntraServicePrincipalObjectId;
use crate::EntraUserId;
use arbitrary::Arbitrary;
use std::hash::Hash;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum EntraPrincipalId {
    UserId(EntraUserId),
    GroupId(EntraGroupId),
    ServicePrincipalId(EntraServicePrincipalObjectId),
    Unknown(Uuid),
}
crate::impl_facet_string_proxy!(EntraPrincipalId, value => value.to_string());

impl EntraPrincipalId {
    pub fn new(uuid: impl Into<Uuid>) -> Self {
        Self::Unknown(uuid.into())
    }
}
impl AsRef<Uuid> for EntraPrincipalId {
    fn as_ref(&self) -> &Uuid {
        match self {
            EntraPrincipalId::UserId(inner) => inner.as_ref(),
            EntraPrincipalId::GroupId(inner) => inner.as_ref(),
            EntraPrincipalId::ServicePrincipalId(inner) => inner.as_ref(),
            EntraPrincipalId::Unknown(inner) => inner,
        }
    }
}

impl std::fmt::Display for EntraPrincipalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref().to_string().as_str())
    }
}

impl std::str::FromStr for EntraPrincipalId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Uuid::parse_str(s)?))
    }
}

impl std::ops::Deref for EntraPrincipalId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Hash for EntraPrincipalId {
    fn hash<H: std::hash::Hasher>(self: &EntraPrincipalId, state: &mut H) {
        let id: &Uuid = self;
        id.hash(state);
    }
}

impl From<Uuid> for EntraPrincipalId {
    fn from(value: Uuid) -> Self {
        Self::Unknown(value)
    }
}
impl From<EntraUserId> for EntraPrincipalId {
    fn from(value: EntraUserId) -> Self {
        Self::UserId(value)
    }
}
impl From<EntraGroupId> for EntraPrincipalId {
    fn from(value: EntraGroupId) -> Self {
        Self::GroupId(value)
    }
}
impl From<EntraServicePrincipalObjectId> for EntraPrincipalId {
    fn from(value: EntraServicePrincipalObjectId) -> Self {
        Self::ServicePrincipalId(value)
    }
}
impl From<&Uuid> for EntraPrincipalId {
    fn from(value: &Uuid) -> Self {
        Self::Unknown(*value)
    }
}
impl From<&EntraUserId> for EntraPrincipalId {
    fn from(value: &EntraUserId) -> Self {
        Self::UserId(*value)
    }
}
impl From<&EntraGroupId> for EntraPrincipalId {
    fn from(value: &EntraGroupId) -> Self {
        Self::GroupId(*value)
    }
}
impl From<&EntraServicePrincipalObjectId> for EntraPrincipalId {
    fn from(value: &EntraServicePrincipalObjectId) -> Self {
        Self::ServicePrincipalId(*value)
    }
}

// Because the internal uuids should be unique between categories, it's fine to compare between unknown/group/user/etc
impl Eq for EntraPrincipalId {}
impl<T: AsRef<Uuid>> PartialEq<T> for EntraPrincipalId {
    fn eq(&self, other: &T) -> bool {
        let left: &Uuid = self;
        let right: &Uuid = other.as_ref();
        left == right
    }
}

cloud_terrastodon_registry::register_thing!(EntraPrincipalId);
cloud_terrastodon_registry::register_arbitrary!(EntraPrincipalId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let id =
            facet_json::from_str::<EntraPrincipalId>("\"00000000-0000-0000-0000-000000000000\"")?;
        assert_eq!(id, Uuid::nil());
        let reparsed = facet_json::from_str::<EntraPrincipalId>(&facet_json::to_string(&id)?)?;
        assert_eq!(id, reparsed);
        Ok(())
    }
}
