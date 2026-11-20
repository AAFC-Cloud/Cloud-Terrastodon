use cloud_terrastodon_azure::ManagementGroupAncestorsChainEntry;
use cloud_terrastodon_azure::Subscription;

/// One visible row in the management group workspace.
pub(crate) enum ManagementGroupTreeRow<'a> {
    Group {
        group: &'a ManagementGroupAncestorsChainEntry,
        depth: usize,
        expanded: Option<bool>,
    },
    Subscription {
        subscription: &'a Subscription,
        depth: usize,
    },
}
