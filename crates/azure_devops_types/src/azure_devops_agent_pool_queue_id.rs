use arbitrary::Arbitrary;
use std::num::ParseIntError;
use std::str::FromStr;

/// The identifier of an Azure DevOps build queue.
///
/// See `id` in Microsoft's [AgentPoolQueue 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#agentpoolqueue).
/// The documented wire value is a signed 32-bit integer; no additional range
/// restriction is imposed. Queue and agent-pool identifiers are distinct values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsAgentPoolQueueId(pub i32);

impl AzureDevOpsAgentPoolQueueId {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for AzureDevOpsAgentPoolQueueId {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl From<AzureDevOpsAgentPoolQueueId> for i32 {
    fn from(value: AzureDevOpsAgentPoolQueueId) -> Self {
        value.0
    }
}

impl FromStr for AzureDevOpsAgentPoolQueueId {
    type Err = ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self::new)
    }
}

impl std::fmt::Display for AzureDevOpsAgentPoolQueueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolQueueId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolQueueId);
