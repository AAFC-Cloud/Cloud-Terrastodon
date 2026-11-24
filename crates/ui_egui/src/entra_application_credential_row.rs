use crate::entra_application_credential_metadata::EntraApplicationCredentialMetadata;
use crate::entra_application_credential_status::EntraApplicationCredentialStatus;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure::ArbitraryJson;
use facet::Facet;

/// Visible credential metadata with no credential material or hints.
#[derive(Facet)]
pub(crate) struct EntraApplicationCredentialRow {
    pub index: usize,
    pub name: Option<String>,
    pub status: &'static str,
    pub starts: Option<String>,
    pub expires: Option<String>,
}

impl EntraApplicationCredentialRow {
    pub fn new(
        index: usize,
        credential: &ArbitraryJson,
        now: DateTime<Utc>,
        demo_mode: bool,
    ) -> Self {
        let metadata = EntraApplicationCredentialMetadata::read(credential);
        Self {
            index,
            name: metadata
                .as_ref()
                .and_then(|entry| entry.display_name.as_ref())
                .filter(|name| !demo_mode && !name.is_empty())
                .cloned(),
            status: metadata
                .as_ref()
                .map_or(EntraApplicationCredentialStatus::Unknown, |entry| {
                    entry.status(now)
                })
                .label(),
            starts: metadata
                .as_ref()
                .and_then(|entry| entry.start_date_time)
                .map(credential_timestamp),
            expires: metadata
                .as_ref()
                .and_then(|entry| entry.end_date_time)
                .map(credential_timestamp),
        }
    }
}

fn credential_timestamp(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M UTC").to_string()
}
