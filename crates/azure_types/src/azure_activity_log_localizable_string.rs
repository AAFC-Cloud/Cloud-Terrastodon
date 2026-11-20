use arbitrary::Arbitrary;
use facet::Facet;

/// An invariant operation or status label and its localized display text.
/// See Azure Monitor 2015-04-01 [`LocalizableString`](https://learn.microsoft.com/en-us/rest/api/monitor/activity-logs/list?view=rest-monitor-2015-04-01#localizablestring).
#[derive(Debug, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureActivityLogLocalizableString {
    pub value: String,
    pub localized_value: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureActivityLogLocalizableString);
cloud_terrastodon_registry::register_arbitrary!(AzureActivityLogLocalizableString);
