use arbitrary::Arbitrary;
use eyre::Result;
use std::str::FromStr;
use url::Url;

/// The resource link advertised by a build definition's agent queue.
///
/// See `url` in Microsoft's [AgentPoolQueue 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#agentpoolqueue).
/// The Build schema provides no route or project-scope guarantee for this link.
/// Preserve its parsed URL instead of assuming it follows the different
/// Distributed Task Queues API's route or inferring a project from the definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsAgentPoolQueueUrl(pub Url);

impl AzureDevOpsAgentPoolQueueUrl {
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        Ok(Self(Url::parse(value.as_ref())?))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}

impl FromStr for AzureDevOpsAgentPoolQueueUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsAgentPoolQueueUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsAgentPoolQueueUrl> for String {
    fn from(value: &AzureDevOpsAgentPoolQueueUrl) -> Self {
        value.as_str().to_owned()
    }
}

impl From<Url> for AzureDevOpsAgentPoolQueueUrl {
    fn from(value: Url) -> Self {
        Self(value)
    }
}

impl From<AzureDevOpsAgentPoolQueueUrl> for Url {
    fn from(value: AzureDevOpsAgentPoolQueueUrl) -> Self {
        value.into_url()
    }
}

impl AsRef<str> for AzureDevOpsAgentPoolQueueUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AzureDevOpsAgentPoolQueueUrl {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPoolQueueUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let identifier = u32::arbitrary(u)?;
        format!("https://queues.invalid/resource/{identifier}")
            .parse()
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolQueueUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolQueueUrl);
