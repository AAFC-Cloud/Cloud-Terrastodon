use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Hash, facet::Facet)]
#[facet(json::proxy = String)]
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

impl From<&AzureDevOpsAccountId> for String {
    fn from(value: &AzureDevOpsAccountId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for AzureDevOpsAccountId {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for AzureDevOpsAccountId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsAccountId::new(uuid))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAccountId);
