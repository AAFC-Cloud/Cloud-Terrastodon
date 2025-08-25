use arbitrary::Arbitrary;
use eyre::Context;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;
use uuid::Uuid;

pub const SUBSCRIPTION_ID_PREFIX: &str = "/subscriptions/";

#[derive(Debug, Eq, PartialEq, Clone, Copy, Arbitrary, Hash)]
pub struct AzureDevOpsServiceEndpointId(Uuid);
impl AzureDevOpsServiceEndpointId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }
    pub fn try_new<T>(uuid: T) -> eyre::Result<Self>
    where
        T: TryInto<Uuid>,
        T::Error: Into<eyre::Error>,
    {
        let uuid = uuid
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert to Uuid")?;
        Ok(Self(uuid))
    }
}
impl std::fmt::Display for AzureDevOpsServiceEndpointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.hyphenated()))
    }
}
impl Deref for AzureDevOpsServiceEndpointId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for AzureDevOpsServiceEndpointId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Uuid> for AzureDevOpsServiceEndpointId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}
impl serde::Serialize for AzureDevOpsServiceEndpointId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for AzureDevOpsServiceEndpointId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| serde::de::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

impl FromStr for AzureDevOpsServiceEndpointId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix(SUBSCRIPTION_ID_PREFIX).unwrap_or(s);
        let id: eyre::Result<Uuid, _> = s.parse();
        let id = id.wrap_err_with(|| format!("Parsing subscription id from {s:?}"))?;
        Ok(Self(id))
    }
}
impl TryFrom<&str> for AzureDevOpsServiceEndpointId {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

#[cfg(test)]
mod test {
    use super::AzureDevOpsServiceEndpointId;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        // a random guid
        let _id = AzureDevOpsServiceEndpointId::try_new("ba53fb6a-867e-413b-8c91-53fb5ff77d70")?;
        Ok(())
    }
}
