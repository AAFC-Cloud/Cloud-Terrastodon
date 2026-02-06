use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::ops::Deref;
use std::str::FromStr;

/// A valid name is less than 128 characters in length and does not contain the following characters: ',', '"', '/', '\', '[', ']', ':', '|', '<', '>', '+', '=', ';', '?', '*'.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AzureDevOpsAgentPoolName(String);

impl AzureDevOpsAgentPoolName {
    pub fn try_new(name: impl Into<String>) -> eyre::Result<Self> {
        let name = name.into();
        if name.len() > 128 {
            bail!("Agent pool name must be less than 128 characters");
        }
        let invalid_chars = [
            ',', '"', '/', '\\', '[', ']', ':', '|', '<', '>', '+', '=', ';', '?', '*',
        ];
        if let Some(pos) = name.chars().position(|c| invalid_chars.contains(&c)) {
            bail!(
                "Agent pool name contains invalid character '{}' at position {}",
                name.chars().nth(pos).unwrap(),
                pos
            );
        }
        Ok(AzureDevOpsAgentPoolName(name))
    }
}

impl core::fmt::Display for AzureDevOpsAgentPoolName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for AzureDevOpsAgentPoolName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AzureDevOpsAgentPoolName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

impl Deref for AzureDevOpsAgentPoolName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for AzureDevOpsAgentPoolName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for AzureDevOpsAgentPoolName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AzureDevOpsAgentPoolName::try_new(s)
    }
}
