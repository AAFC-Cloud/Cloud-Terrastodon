use compact_str::CompactString;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComputeSkuName(CompactString);
impl ComputeSkuName {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        if value.is_empty() {
            Err(eyre::eyre!("Compute SKU name cannot be empty"))
        } else {
            Ok(Self(value))
        }
    }
}

impl Display for ComputeSkuName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl Deref for ComputeSkuName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl FromStr for ComputeSkuName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ComputeSkuName::try_new(s)
    }
}
impl TryFrom<CompactString> for ComputeSkuName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<ComputeSkuName> for CompactString {
    fn from(value: ComputeSkuName) -> Self {
        value.0
    }
}


impl Serialize for ComputeSkuName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ComputeSkuName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <CompactString as Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
