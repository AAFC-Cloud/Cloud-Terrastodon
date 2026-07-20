use crate::ArbitraryJson;
use crate::EntraApplicationClientId;
use crate::entra_application_object_id::EntraApplicationObjectId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraApplicationRegistration {
    pub add_ins: Vec<ArbitraryJson>,
    pub api: Option<ArbitraryJson>,
    pub app_id: EntraApplicationClientId,
    pub application_template_id: Option<Uuid>,
    pub app_roles: Vec<ArbitraryJson>,
    pub created_date_time: Option<DateTime<Utc>>,
    pub default_redirect_uri: Option<String>,
    pub description: Option<String>,
    pub disabled_by_microsoft_status: Option<String>,
    pub display_name: String,
    pub group_membership_claims: Option<String>,
    pub id: EntraApplicationObjectId,
    pub identifier_uris: Vec<String>,
    pub info: Option<ArbitraryJson>,
    pub is_device_only_auth_supported: Option<bool>,
    pub is_fallback_public_client: Option<bool>,
    pub key_credentials: Vec<ArbitraryJson>,
    pub notes: Option<String>,
    pub optional_claims: Option<ArbitraryJson>,
    pub parental_control_settings: Option<ArbitraryJson>,
    pub password_credentials: Vec<ArbitraryJson>,
    pub public_client: Option<ArbitraryJson>,
    pub publisher_domain: Option<String>,
    pub request_signature_verification: Option<ArbitraryJson>,
    pub required_resource_access: Vec<ArbitraryJson>,
    pub service_management_reference: Option<String>,
    pub service_principal_lock_configuration: Option<ArbitraryJson>,
    pub sign_in_audience: Option<String>,
    pub spa: Option<ArbitraryJson>,
    pub tags: Vec<String>,
    pub token_encryption_key_id: Option<Uuid>,
    pub unique_name: Option<String>,
    pub verified_publisher: Option<ArbitraryJson>,
    pub web: Option<ArbitraryJson>,
    #[facet(flatten)]
    pub additional_properties: HashMap<String, ArbitraryJson>,
}

impl std::fmt::Display for EntraApplicationRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id.to_string().as_str())?;
        f.write_str(" - ")?;
        f.write_str(&self.display_name)?;
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(EntraApplicationRegistration);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationRegistration);
cloud_terrastodon_registry::register_arbitrary!(Vec<EntraApplicationRegistration>);
