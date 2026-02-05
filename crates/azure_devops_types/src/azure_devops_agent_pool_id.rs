use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct AzureDevOpsAgentPoolId(usize);

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
