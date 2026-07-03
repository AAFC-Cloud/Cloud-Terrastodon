use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Copy, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsAgentPoolId(usize);

impl AzureDevOpsAgentPoolId {
    pub fn new(id: usize) -> AzureDevOpsAgentPoolId {
        AzureDevOpsAgentPoolId(id)
    }
}

impl core::fmt::Display for AzureDevOpsAgentPoolId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for AzureDevOpsAgentPoolId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AzureDevOpsAgentPoolId::new(s.parse()?))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolId);

