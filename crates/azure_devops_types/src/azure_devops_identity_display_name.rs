use arbitrary::Arbitrary;
use std::convert::Infallible;
use std::str::FromStr;

/// The non-unique display name of an Azure DevOps identity.
///
/// See `displayName` in Microsoft's [IdentityRef 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#identityref).
/// The name comes from the identity's source provider. Preserve its spelling
/// without imposing project-name or build-definition naming restrictions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsIdentityDisplayName(String);

impl AzureDevOpsIdentityDisplayName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AzureDevOpsIdentityDisplayName {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(value))
    }
}

impl From<String> for AzureDevOpsIdentityDisplayName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for AzureDevOpsIdentityDisplayName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<AzureDevOpsIdentityDisplayName> for String {
    fn from(value: AzureDevOpsIdentityDisplayName) -> Self {
        value.0
    }
}

impl AsRef<str> for AzureDevOpsIdentityDisplayName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AzureDevOpsIdentityDisplayName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityDisplayName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsIdentityDisplayName);
