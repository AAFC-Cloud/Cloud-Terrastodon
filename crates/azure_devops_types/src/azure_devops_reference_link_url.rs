use arbitrary::Arbitrary;
use eyre::Result;
use std::str::FromStr;
use url::Url;

/// The target URL of an Azure DevOps REST reference link.
///
/// See Microsoft's [ReferenceLinks 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#referencelinks)
/// and the `href` property in the official [ReferenceLink interface](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/common/VSSInterfaces.ts).
/// Reference links can identify different resource kinds. Preserve the parsed
/// URL without guessing an organization, resource route, or additional scheme,
/// host, path, query, or fragment restrictions.
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsReferenceLinkUrl(Url);

impl AzureDevOpsReferenceLinkUrl {
    /// Parse the service-provided URL using `url::Url`'s URL semantics.
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

impl FromStr for AzureDevOpsReferenceLinkUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsReferenceLinkUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for AzureDevOpsReferenceLinkUrl {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<Url> for AzureDevOpsReferenceLinkUrl {
    fn from(value: Url) -> Self {
        Self(value)
    }
}

impl AsRef<str> for AzureDevOpsReferenceLinkUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Url> for AzureDevOpsReferenceLinkUrl {
    fn as_ref(&self) -> &Url {
        self.as_url()
    }
}

impl From<&AzureDevOpsReferenceLinkUrl> for String {
    fn from(value: &AzureDevOpsReferenceLinkUrl) -> Self {
        value.as_str().to_owned()
    }
}

impl From<AzureDevOpsReferenceLinkUrl> for Url {
    fn from(value: AzureDevOpsReferenceLinkUrl) -> Self {
        value.into_url()
    }
}

impl std::fmt::Display for AzureDevOpsReferenceLinkUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsReferenceLinkUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let identifier = u64::arbitrary(u)?;
        format!("https://references.invalid/resource/{identifier}")
            .parse()
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsReferenceLinkUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsReferenceLinkUrl);
