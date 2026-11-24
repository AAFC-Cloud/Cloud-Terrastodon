use cloud_terrastodon_azure::ManagementGroupAncestorsChainEntry;
use cloud_terrastodon_azure::Subscription;
use std::borrow::Cow;

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

impl<'a> ManagementGroupTreeRow<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::Group { group, .. } => &group.display_name,
            Self::Subscription { subscription, .. } => subscription.name.as_str(),
        }
    }

    pub fn identifier(&self) -> Cow<'a, str> {
        match self {
            Self::Group { group, .. } => Cow::Borrowed(&group.name),
            Self::Subscription { subscription, .. } => Cow::Owned(subscription.id.to_string()),
        }
    }

    pub fn parent_name(&self) -> &'a str {
        match self {
            Self::Group { .. } => "",
            Self::Subscription { subscription, .. } => subscription
                .management_group_ancestors_chain
                .first()
                .map_or("", |parent| parent.display_name.as_str()),
        }
    }
}
