use arbitrary::Arbitrary;
use std::str::FromStr;
use uuid::Uuid;

/// The UUID identifying an optional build behavior.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildoptiondefinitionreference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildOptionId(pub Uuid);

impl From<Uuid> for AzureDevOpsBuildOptionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for AzureDevOpsBuildOptionId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}

impl TryFrom<String> for AzureDevOpsBuildOptionId {
    type Error = uuid::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildOptionId> for String {
    fn from(value: &AzureDevOpsBuildOptionId) -> Self {
        value.0.to_string()
    }
}

impl std::fmt::Display for AzureDevOpsBuildOptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildOptionId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildOptionId);
