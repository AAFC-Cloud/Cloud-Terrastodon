use crate::AzureActivityLogEventLevel;
use crate::AzureActivityLogLocalizableString;
use crate::AzureResourceGroupName;
use crate::ScopeImpl;
use crate::SubscriptionId;
use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;

/// A projection of Azure Monitor 2015-04-01 [`EventData`](https://learn.microsoft.com/en-us/rest/api/monitor/activity-logs/list?view=rest-monitor-2015-04-01#eventdata).
/// The request projects event identity, time, operation, status, severity, category, and scope.
#[derive(Debug, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureActivityLogEvent {
    pub category: Option<AzureActivityLogLocalizableString>,
    pub event_data_id: String,
    pub event_timestamp: Timestamp,
    pub operation_name: AzureActivityLogLocalizableString,
    pub status: Option<AzureActivityLogLocalizableString>,
    pub resource_id: Option<ScopeImpl>,
    pub level: Option<AzureActivityLogEventLevel>,
    pub resource_group_name: Option<AzureResourceGroupName>,
    pub subscription_id: Option<SubscriptionId>,
}

cloud_terrastodon_registry::register_thing!(AzureActivityLogEvent);
cloud_terrastodon_registry::register_arbitrary!(AzureActivityLogEvent);
