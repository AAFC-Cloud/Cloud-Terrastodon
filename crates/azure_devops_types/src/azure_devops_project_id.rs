use arbitrary::Arbitrary;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct AzureDevOpsProjectId(Uuid);
impl std::fmt::Display for AzureDevOpsProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Deref for AzureDevOpsProjectId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevOpsProjectId {
    pub fn new(uuid: Uuid) -> AzureDevOpsProjectId {
        AzureDevOpsProjectId(uuid)
    }
}

impl From<&AzureDevOpsProjectId> for String {
    fn from(value: &AzureDevOpsProjectId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for AzureDevOpsProjectId {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for AzureDevOpsProjectId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsProjectId::new(uuid))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectId);

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn it_works() {
        let id = Uuid::new_v4().to_string();
        let _project_id = id.parse::<AzureDevOpsProjectId>().unwrap();
    }
}

