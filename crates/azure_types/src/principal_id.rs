use crate::EntraGroupId;
use crate::EntraServicePrincipalId;
use crate::EntraUserId;
use std::hash::Hash;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum PrincipalId {
    UserId(EntraUserId),
    GroupId(EntraGroupId),
    ServicePrincipalId(EntraServicePrincipalId),
    Unknown(Uuid),
}
crate::impl_facet_string_proxy!(PrincipalId, value => value.to_string());

impl PrincipalId {
    pub fn new(uuid: impl Into<Uuid>) -> Self {
        Self::Unknown(uuid.into())
    }
}
impl AsRef<Uuid> for PrincipalId {
    fn as_ref(&self) -> &Uuid {
        match self {
            PrincipalId::UserId(inner) => inner.as_ref(),
            PrincipalId::GroupId(inner) => inner.as_ref(),
            PrincipalId::ServicePrincipalId(inner) => inner.as_ref(),
            PrincipalId::Unknown(inner) => inner,
        }
    }
}

impl std::fmt::Display for PrincipalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref().to_string().as_str())
    }
}

impl std::str::FromStr for PrincipalId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Uuid::parse_str(s)?))
    }
}

impl std::ops::Deref for PrincipalId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Hash for PrincipalId {
    fn hash<H: std::hash::Hasher>(self: &PrincipalId, state: &mut H) {
        let id: &Uuid = self;
        id.hash(state);
    }
}

impl From<Uuid> for PrincipalId {
    fn from(value: Uuid) -> Self {
        Self::Unknown(value)
    }
}
impl From<EntraUserId> for PrincipalId {
    fn from(value: EntraUserId) -> Self {
        Self::UserId(value)
    }
}
impl From<EntraGroupId> for PrincipalId {
    fn from(value: EntraGroupId) -> Self {
        Self::GroupId(value)
    }
}
impl From<EntraServicePrincipalId> for PrincipalId {
    fn from(value: EntraServicePrincipalId) -> Self {
        Self::ServicePrincipalId(value)
    }
}
impl From<&Uuid> for PrincipalId {
    fn from(value: &Uuid) -> Self {
        Self::Unknown(*value)
    }
}
impl From<&EntraUserId> for PrincipalId {
    fn from(value: &EntraUserId) -> Self {
        Self::UserId(*value)
    }
}
impl From<&EntraGroupId> for PrincipalId {
    fn from(value: &EntraGroupId) -> Self {
        Self::GroupId(*value)
    }
}
impl From<&EntraServicePrincipalId> for PrincipalId {
    fn from(value: &EntraServicePrincipalId) -> Self {
        Self::ServicePrincipalId(*value)
    }
}

// Because the internal uuids should be unique between categories, it's fine to compare between unknown/group/user/etc
impl Eq for PrincipalId {}
impl<T: AsRef<Uuid>> PartialEq<T> for PrincipalId {
    fn eq(&self, other: &T) -> bool {
        let left: &Uuid = self;
        let right: &Uuid = other.as_ref();
        left == right
    }
}

cloud_terrastodon_registry::register_thing!(PrincipalId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let id = facet_json::from_str::<PrincipalId>("\"00000000-0000-0000-0000-000000000000\"")?;
        assert_eq!(id, Uuid::nil());
        let reparsed = facet_json::from_str::<PrincipalId>(&facet_json::to_string(&id)?)?;
        assert_eq!(id, reparsed);
        Ok(())
    }
}
