use arbitrary::Arbitrary;
use std::convert::Infallible;
use std::str::FromStr;

/// The legacy unique name of an Azure DevOps identity.
///
/// See `uniqueName` in Microsoft's [IdentityRef 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#identityref).
/// Preserves provider spelling without assuming an email address or UPN format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsIdentityUniqueName(String);

impl AzureDevOpsIdentityUniqueName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AzureDevOpsIdentityUniqueName {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(value))
    }
}

impl From<String> for AzureDevOpsIdentityUniqueName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for AzureDevOpsIdentityUniqueName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<AzureDevOpsIdentityUniqueName> for String {
    fn from(value: AzureDevOpsIdentityUniqueName) -> Self {
        value.0
    }
}

impl AsRef<str> for AzureDevOpsIdentityUniqueName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AzureDevOpsIdentityUniqueName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityUniqueName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsIdentityUniqueName);
