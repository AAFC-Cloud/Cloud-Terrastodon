use arbitrary::Arbitrary;
use std::str::FromStr;
use uuid::Uuid;

/// The UUID identifying a secure file used by a build process.
///
/// Microsoft's [SecureFileReference.id wire model](https://github.com/microsoft/azure-devops-go-api/blob/dev/azuredevops/build/models.go#L1684)
/// declares a UUID, unlike a build repository's provider-specific string ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsSecureFileId(pub Uuid);

impl From<Uuid> for AzureDevOpsSecureFileId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for AzureDevOpsSecureFileId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}

impl TryFrom<String> for AzureDevOpsSecureFileId {
    type Error = uuid::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsSecureFileId> for String {
    fn from(value: &AzureDevOpsSecureFileId) -> Self {
        value.0.to_string()
    }
}

impl std::fmt::Display for AzureDevOpsSecureFileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsSecureFileId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsSecureFileId);
