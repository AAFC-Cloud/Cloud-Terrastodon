use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct AzureDevOpsAccountId(Uuid);

impl std::fmt::Display for AzureDevOpsAccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for AzureDevOpsAccountId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevOpsAccountId {
    pub fn new(uuid: Uuid) -> AzureDevOpsAccountId {
        AzureDevOpsAccountId(uuid)
    }
}

impl FromStr for AzureDevOpsAccountId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsAccountId::new(uuid))
    }
}
