use arbitrary::Arbitrary;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Arbitrary, Hash, PartialOrd, Ord)]
pub struct AzureTenantId(Uuid);

impl AzureTenantId {
    pub fn new(uuid: Uuid) -> Self {
        AzureTenantId(uuid)
    }
}
impl Deref for AzureTenantId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for AzureTenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hyphenated().fmt(f)
    }
}

impl FromStr for AzureTenantId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AzureTenantId(uuid::Uuid::parse_str(s)?))
    }
}

impl Serialize for AzureTenantId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.hyphenated().to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AzureTenantId {
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
