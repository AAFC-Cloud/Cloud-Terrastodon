use crate::prelude::EntraGroupId;
use crate::prelude::EntraServicePrincipalId;
use crate::prelude::EntraUserId;
use std::hash::Hash;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum PrincipalId {
    UserId(EntraUserId),
    GroupId(EntraGroupId),
    ServicePrincipalId(EntraServicePrincipalId),
    Unknown(Uuid),
}

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

impl serde::Serialize for PrincipalId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref().to_string().as_str())
    }
}

impl<'de> serde::Deserialize<'de> for PrincipalId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        use std::str::FromStr;
        let uuid = Uuid::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(Self::new(uuid))
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
