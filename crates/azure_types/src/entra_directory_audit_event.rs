use crate::EntraDirectoryAuditResult;
use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;

/// Selected activity fields from the Microsoft Graph v1.0
/// [directoryAudit resource](https://learn.microsoft.com/en-us/graph/api/resources/directoryaudit?view=graph-rest-1.0#properties).
#[derive(Arbitrary, Clone, Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct EntraDirectoryAuditEvent {
    pub id: String,
    pub activity_date_time: Timestamp,
    pub activity_display_name: String,
    pub category: String,
    pub result: EntraDirectoryAuditResult,
    pub logged_by_service: String,
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryAuditEvent);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryAuditEvent);
