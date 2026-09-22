use arbitrary::Arbitrary;

/// The avatar link returned in an Azure DevOps work item identity's `_links`
/// object.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
pub struct AzureDevOpsWorkItemIdentityAvatar {
    pub href: String,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemIdentityAvatar);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemIdentityAvatar);
