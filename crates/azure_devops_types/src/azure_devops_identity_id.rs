use arbitrary::Arbitrary;
use std::convert::Infallible;
use std::str::FromStr;

/// The opaque ID of an Azure DevOps identity, which may represent a person,
/// group, or process rather than only a user.
///
/// See `id` in Microsoft's [IdentityRef 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#identityref).
/// The schema specifies a string without further format constraints. Preserve
/// it verbatim rather than assuming that every identity ID is a user UUID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsIdentityId(String);

impl AzureDevOpsIdentityId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AzureDevOpsIdentityId {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(value))
    }
}

impl From<String> for AzureDevOpsIdentityId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for AzureDevOpsIdentityId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<AzureDevOpsIdentityId> for String {
    fn from(value: AzureDevOpsIdentityId) -> Self {
        value.0
    }
}

impl AsRef<str> for AzureDevOpsIdentityId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AzureDevOpsIdentityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsIdentityId);
