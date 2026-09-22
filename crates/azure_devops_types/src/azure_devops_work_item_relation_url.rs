use crate::AzureDevOpsOrganizationUrl;
use crate::AzureDevOpsWorkItemId;
use crate::AzureDevOpsWorkItemUrl;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::str::FromStr;
use url::Url;

/// A relation target URL. Work-item targets are structured, while Azure DevOps
/// relation responses may also contain URLs for attachments or other resources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(proxy = String)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemRelationUrl {
    WorkItem(AzureDevOpsWorkItemUrl<'static>),
    Other(Url),
}

impl AzureDevOpsWorkItemRelationUrl {
    pub fn work_item(&self) -> Option<&AzureDevOpsWorkItemUrl<'static>> {
        match self {
            Self::WorkItem(url) => Some(url),
            Self::Other(_) => None,
        }
    }

    pub fn into_work_item(self) -> Result<AzureDevOpsWorkItemUrl<'static>> {
        match self {
            Self::WorkItem(url) => Ok(url),
            Self::Other(_) => eyre::bail!("Relation URL does not target a work item"),
        }
    }

    /// Returns the target work item ID after verifying its organization.
    pub fn work_item_id_for_organization(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> Result<AzureDevOpsWorkItemId> {
        self.work_item()
            .ok_or_else(|| eyre::eyre!("Relation URL does not target a work item"))?
            .work_item_id_for_organization(organization)
    }

    pub fn into_url(self) -> Result<Url> {
        match self {
            Self::WorkItem(url) => url.into_url(),
            Self::Other(url) => Ok(url),
        }
    }
}

impl From<AzureDevOpsWorkItemUrl<'static>> for AzureDevOpsWorkItemRelationUrl {
    fn from(value: AzureDevOpsWorkItemUrl<'static>) -> Self {
        Self::WorkItem(value)
    }
}

impl TryFrom<AzureDevOpsWorkItemRelationUrl> for Url {
    type Error = eyre::Error;

    fn try_from(value: AzureDevOpsWorkItemRelationUrl) -> Result<Self, Self::Error> {
        value.into_url()
    }
}

impl TryFrom<&AzureDevOpsWorkItemRelationUrl> for Url {
    type Error = eyre::Error;

    fn try_from(value: &AzureDevOpsWorkItemRelationUrl) -> Result<Self, Self::Error> {
        value.clone().into_url()
    }
}

impl From<&AzureDevOpsWorkItemRelationUrl> for String {
    fn from(value: &AzureDevOpsWorkItemRelationUrl) -> Self {
        match value {
            AzureDevOpsWorkItemRelationUrl::WorkItem(url) => String::from(url),
            AzureDevOpsWorkItemRelationUrl::Other(url) => url.to_string(),
        }
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemRelationUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(String::from(self).as_str())
    }
}

impl FromStr for AzureDevOpsWorkItemRelationUrl {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(url) = value.parse::<AzureDevOpsWorkItemUrl<'static>>() {
            return Ok(Self::WorkItem(url));
        }
        let url = Url::parse(value)?;
        ensure!(
            matches!(url.scheme(), "https" | "http" | "vstfs"),
            "Unsupported relation URL scheme"
        );
        Ok(Self::Other(url))
    }
}

impl TryFrom<String> for AzureDevOpsWorkItemRelationUrl {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemRelationUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::WorkItem(AzureDevOpsWorkItemUrl::arbitrary(u)?))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemRelationUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemRelationUrl);
