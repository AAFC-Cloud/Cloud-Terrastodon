use arbitrary::Arbitrary;
use std::convert::Infallible;
use std::ops::Deref;
use std::str::FromStr;

/// The name of an Azure DevOps build queue.
///
/// See `name` in Microsoft's [AgentPoolQueue 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#agentpoolqueue).
/// Preserve the service's exact Unicode spelling without applying agent-pool
/// naming restrictions to this distinct property.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsAgentPoolQueueName(pub String);

impl AzureDevOpsAgentPoolQueueName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AzureDevOpsAgentPoolQueueName {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(value))
    }
}

impl From<String> for AzureDevOpsAgentPoolQueueName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for AzureDevOpsAgentPoolQueueName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<AzureDevOpsAgentPoolQueueName> for String {
    fn from(value: AzureDevOpsAgentPoolQueueName) -> Self {
        value.0
    }
}

impl Deref for AzureDevOpsAgentPoolQueueName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsAgentPoolQueueName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AzureDevOpsAgentPoolQueueName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolQueueName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolQueueName);
