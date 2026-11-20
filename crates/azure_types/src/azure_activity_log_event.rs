use crate::AzureActivityLogLocalizableString;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// A projection of Azure Monitor 2015-04-01 [`EventData`](https://learn.microsoft.com/en-us/rest/api/monitor/activity-logs/list?view=rest-monitor-2015-04-01#eventdata).
/// The request selects event identity, timestamp, operation, status, and resource.
#[derive(Debug, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureActivityLogEvent {
    /// Azure can return this event label even when the request does not select it.
    pub category: Option<AzureActivityLogLocalizableString>,
    pub event_data_id: String,
    pub event_timestamp: DateTime<Utc>,
    pub operation_name: AzureActivityLogLocalizableString,
    pub status: Option<AzureActivityLogLocalizableString>,
    pub resource_id: Option<ScopeImpl>,
}

cloud_terrastodon_registry::register_thing!(AzureActivityLogEvent);
cloud_terrastodon_registry::register_arbitrary!(AzureActivityLogEvent);
