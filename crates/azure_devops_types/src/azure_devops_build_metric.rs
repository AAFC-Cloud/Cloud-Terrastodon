use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;

/// A build metric's name, value and time scope.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildmetric).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildMetric {
    pub name: String,
    pub int_value: i32,
    pub date: Option<DateTime<Utc>>,
    pub scope: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildMetric);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildMetric);
