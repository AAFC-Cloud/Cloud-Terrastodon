use crate::entra_application_credential_status::EntraApplicationCredentialStatus;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure::ArbitraryJson;
use facet::Facet;

/// The readable fields shared by Graph [key credentials](https://learn.microsoft.com/en-us/graph/api/resources/keycredential?view=graph-rest-1.0)
/// and [password credentials](https://learn.microsoft.com/en-us/graph/api/resources/passwordcredential?view=graph-rest-1.0).
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub(crate) struct EntraApplicationCredentialMetadata {
    pub display_name: Option<String>,
    pub start_date_time: Option<DateTime<Utc>>,
    pub end_date_time: Option<DateTime<Utc>>,
}

impl EntraApplicationCredentialMetadata {
    pub fn read(credential: &ArbitraryJson) -> Option<Self> {
        // Project only metadata from the SDK's credential object. This keeps
        // credential material and hints out of the GUI even when Graph returns them.
        facet_json::from_str(credential.as_ref()).ok()
    }

    pub fn status(&self, now: DateTime<Utc>) -> EntraApplicationCredentialStatus {
        use EntraApplicationCredentialStatus as Status;
        match (self.start_date_time, self.end_date_time) {
            (Some(start), Some(end)) if start >= end => Status::Unknown,
            (_, Some(end)) if end <= now => Status::Expired,
            (Some(start), Some(_)) if start > now => Status::NotYetValid,
            (Some(_), Some(_)) => Status::Current,
            _ => Status::Unknown,
        }
    }
}
