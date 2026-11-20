use crate::AzureResourceGroupId;
use crate::SubscriptionId;
use arbitrary::Arbitrary;
use facet::Facet;

/// The subscription or resource-group filters supported by the desktop
/// [Azure Monitor Activity Logs operation](https://learn.microsoft.com/en-us/rest/api/monitor/activity-logs/list?view=rest-monitor-2015-04-01#uri-parameters).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[repr(C)]
pub enum AzureActivityLogScope {
    Subscription(SubscriptionId),
    ResourceGroup(AzureResourceGroupId),
}

impl AzureActivityLogScope {
    pub fn subscription_id(&self) -> SubscriptionId {
        match self {
            Self::Subscription(id) => *id,
            Self::ResourceGroup(id) => id.subscription_id,
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureActivityLogScope);
cloud_terrastodon_registry::register_arbitrary!(AzureActivityLogScope);
