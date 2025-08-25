use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TenantId(Uuid);

impl TenantId {
    pub fn new(uuid: Uuid) -> Self {
        TenantId(uuid)
    }
}
impl Deref for TenantId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}

impl FromStr for TenantId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TenantId(uuid::Uuid::parse_str(s)?))
    }
}

impl Serialize for TenantId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for TenantId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}
