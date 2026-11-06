use crate::AzureDevOpsWorkItemIdentityAvatar;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;
use std::collections::BTreeMap;

/// Hypermedia links attached to an Azure DevOps work item identity.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
pub struct AzureDevOpsWorkItemIdentityLinks {
    pub avatar: Option<AzureDevOpsWorkItemIdentityAvatar>,
    #[facet(flatten, default)]
    pub additional: BTreeMap<String, ArbitraryJson>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemIdentityLinks);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemIdentityLinks);
