use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct AzureDevOpsUserId(Uuid);
impl std::fmt::Display for AzureDevOpsUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Deref for AzureDevOpsUserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevOpsUserId {
    pub fn new(uuid: Uuid) -> AzureDevOpsUserId {
        AzureDevOpsUserId(uuid)
    }
}
impl FromStr for AzureDevOpsUserId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsUserId::new(uuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn it_works() {
        let id = Uuid::new_v4().to_string();
        let _project_id = id.parse::<AzureDevOpsUserId>().unwrap();
    }
}
