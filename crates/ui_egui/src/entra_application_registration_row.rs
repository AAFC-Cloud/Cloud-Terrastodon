use crate::entra_application_credential_status::EntraApplicationCredentialStatus;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::EntraApplicationObjectId;
use cloud_terrastodon_azure::EntraApplicationRegistration;
use facet::Facet;

/// Sortable display values alongside the original typed application identity.
#[derive(Facet)]
pub(crate) struct EntraApplicationRegistrationRow {
    pub display_name: String,
    pub id: EntraApplicationObjectId,
    pub client_id: EntraApplicationClientId,
    pub created_on: Option<String>,
    pub credential_status: String,
}

impl EntraApplicationRegistrationRow {
    pub fn new(application: &EntraApplicationRegistration, now: DateTime<Utc>) -> Self {
        Self {
            display_name: application.display_name.clone(),
            id: application.id,
            client_id: application.app_id,
            created_on: application
                .created_date_time
                .map(|date| date.format("%Y-%m-%d").to_string()),
            credential_status: EntraApplicationCredentialStatus::for_application(application, now)
                .label()
                .to_owned(),
        }
    }
}
