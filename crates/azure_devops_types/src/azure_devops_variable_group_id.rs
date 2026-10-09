use arbitrary::Arbitrary;
use std::num::ParseIntError;
use std::str::FromStr;

/// The numeric identifier of an Azure DevOps variable group.
///
/// Microsoft documentation: [VariableGroup.id](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#variablegroup).
/// The API uses a signed 32-bit integer; no speculative range restriction is imposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsVariableGroupId(pub i32);

impl From<i32> for AzureDevOpsVariableGroupId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl FromStr for AzureDevOpsVariableGroupId {
    type Err = ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}

impl std::fmt::Display for AzureDevOpsVariableGroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsVariableGroupId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsVariableGroupId);
