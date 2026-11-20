use crate::AzureActivityLogEvent;
use facet::Facet;
use std::sync::Arc;
use url::Url;

/// A page from Azure Monitor 2015-04-01 [`EventDataCollection`](https://learn.microsoft.com/en-us/rest/api/monitor/activity-logs/list?view=rest-monitor-2015-04-01#eventdatacollection).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureActivityLogPage {
    pub value: Vec<Arc<AzureActivityLogEvent>>,
    pub next_link: Option<Url>,
}

cloud_terrastodon_registry::register_thing!(AzureActivityLogPage);
