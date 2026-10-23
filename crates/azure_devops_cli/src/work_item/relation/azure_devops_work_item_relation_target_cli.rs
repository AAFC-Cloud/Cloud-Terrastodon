use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemUrl;
use eyre::Result;
use std::borrow::Cow;
use std::str::FromStr;

/// A relation target supplied to the CLI as either a work-item ID or a URL.
#[derive(Debug, Clone, facet::Facet)]
#[facet(proxy = String)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemRelationTarget {
    Id(AzureDevOpsWorkItemId),
    Url(AzureDevOpsWorkItemRelationUrl),
}

impl AzureDevOpsWorkItemRelationTarget {
    pub fn into_relation_url(
        self,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> AzureDevOpsWorkItemRelationUrl {
        match self {
            Self::Id(id) => AzureDevOpsWorkItemRelationUrl::from(AzureDevOpsWorkItemUrl::new(
                Cow::Owned(organization.clone()),
                id,
            )),
            Self::Url(url) => url,
        }
    }
}

impl FromStr for AzureDevOpsWorkItemRelationTarget {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = value.parse() {
            return Ok(Self::Id(id));
        }
        Ok(Self::Url(value.parse()?))
    }
}

impl TryFrom<String> for AzureDevOpsWorkItemRelationTarget {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsWorkItemRelationTarget> for String {
    fn from(value: &AzureDevOpsWorkItemRelationTarget) -> Self {
        match value {
            AzureDevOpsWorkItemRelationTarget::Id(id) => id.to_string(),
            AzureDevOpsWorkItemRelationTarget::Url(url) => url.to_string(),
        }
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemRelationTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from(self).fmt(f)
    }
}
