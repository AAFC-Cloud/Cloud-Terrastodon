use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, facet::Facet)]
#[facet(json::proxy = String)]
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

impl From<&AzureDevOpsUserId> for String {
    fn from(value: &AzureDevOpsUserId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for AzureDevOpsUserId {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
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
