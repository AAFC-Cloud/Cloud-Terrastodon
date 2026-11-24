use crate::AzureResourceGroupId;
use crate::ManagementGroupId;
use crate::SUBSCRIPTION_ID_PREFIX;
use crate::Scope;
use crate::ScopeImpl;
use crate::SubscriptionId;
use crate::strip_prefix_get_slug_and_leading_slashed_remains;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use facet::Facet;

/// Query scopes for [Azure Monitor activity logs](https://learn.microsoft.com/en-us/azure/azure-monitor/fundamentals/activity-log#activity-log-access-scenarios).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Facet)]
#[repr(C)]
pub enum AzureActivityLogScope {
    Subscription(SubscriptionId),
    ResourceGroup(AzureResourceGroupId),
    ManagementGroup(ManagementGroupId),
    /// Query a single ARM resource using its entire ID, including any child path.
    Resource(Box<ScopeImpl>),
}

impl<'a> Arbitrary<'a> for AzureActivityLogScope {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match input.int_in_range(0..=3)? {
            0 => Self::Subscription(input.arbitrary()?),
            1 => Self::ResourceGroup(input.arbitrary()?),
            2 => Self::ManagementGroup(input.arbitrary()?),
            _ => Self::Resource(Box::new(ScopeImpl::StorageAccount(input.arbitrary()?))),
        })
    }
}

impl AzureActivityLogScope {
    /// Return the subscription containing this scope, when one is present.
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            Self::Subscription(id) => Some(*id),
            Self::ResourceGroup(id) => Some(id.subscription_id),
            Self::ManagementGroup(_) => None,
            Self::Resource(id) => {
                let expanded = id.expanded_form();
                let (subscription, _) = strip_prefix_get_slug_and_leading_slashed_remains(
                    &expanded,
                    SUBSCRIPTION_ID_PREFIX,
                )
                .ok()?;
                subscription.parse().ok()
            }
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureActivityLogScope);
cloud_terrastodon_registry::register_arbitrary!(AzureActivityLogScope);
