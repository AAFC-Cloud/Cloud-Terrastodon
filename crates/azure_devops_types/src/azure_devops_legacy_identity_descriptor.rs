use arbitrary::Arbitrary;
use facet::Facet;
use std::fmt;

/// A legacy security identity descriptor, distinct from a Graph subject descriptor.
/// See [Read Identities (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/ims/identities/read-identities?view=azure-devops-rest-7.1#examples).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, Facet)]
#[facet(transparent)]
pub struct AzureDevOpsLegacyIdentityDescriptor(pub String);

impl fmt::Display for AzureDevOpsLegacyIdentityDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsLegacyIdentityDescriptor);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsLegacyIdentityDescriptor);
