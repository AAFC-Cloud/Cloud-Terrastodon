use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Copy)]
pub struct AzureDevOpsAgentPoolId(usize);

impl AzureDevOpsAgentPoolId {
    pub fn new(id: usize) -> AzureDevOpsAgentPoolId {
        AzureDevOpsAgentPoolId(id)
    }
}

impl core::fmt::Display for AzureDevOpsAgentPoolId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for AzureDevOpsAgentPoolId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AzureDevOpsAgentPoolId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = usize::deserialize(deserializer)?;
        Ok(Self(s))
    }
}

impl FromStr for AzureDevOpsAgentPoolId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AzureDevOpsAgentPoolId::new(s.parse()?))
    }
}
