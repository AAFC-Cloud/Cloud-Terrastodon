use crate::AppId;
use crate::EntraApplicationRegistrationId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet_json::RawJson;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraApplicationRegistration {
    #[arbitrary(default)]
    pub add_ins: Vec<RawJson<'static>>,
    #[arbitrary(default)]
    pub api: Option<RawJson<'static>>,
    pub app_id: AppId,
    pub application_template_id: Option<Uuid>,
    #[arbitrary(default)]
    pub app_roles: Vec<RawJson<'static>>,
    pub created_date_time: Option<DateTime<Utc>>,
    pub default_redirect_uri: Option<String>,
    pub description: Option<String>,
    pub disabled_by_microsoft_status: Option<String>,
    pub display_name: String,
    pub group_membership_claims: Option<String>,
    pub id: EntraApplicationRegistrationId,
    pub identifier_uris: Vec<String>,
    #[arbitrary(default)]
    pub info: Option<RawJson<'static>>,
    pub is_device_only_auth_supported: Option<bool>,
    pub is_fallback_public_client: Option<bool>,
    #[arbitrary(default)]
    pub key_credentials: Vec<RawJson<'static>>,
    pub notes: Option<String>,
    #[arbitrary(default)]
    pub optional_claims: Option<RawJson<'static>>,
    #[arbitrary(default)]
    pub parental_control_settings: Option<RawJson<'static>>,
    #[arbitrary(default)]
    pub password_credentials: Vec<RawJson<'static>>,
    #[arbitrary(default)]
    pub public_client: Option<RawJson<'static>>,
    pub publisher_domain: Option<String>,
    #[arbitrary(default)]
    pub request_signature_verification: Option<RawJson<'static>>,
    #[arbitrary(default)]
    pub required_resource_access: Vec<RawJson<'static>>,
    pub service_management_reference: Option<String>,
    #[arbitrary(default)]
    pub service_principal_lock_configuration: Option<RawJson<'static>>,
    pub sign_in_audience: Option<String>,
    #[arbitrary(default)]
    pub spa: Option<RawJson<'static>>,
    pub tags: Vec<String>,
    pub token_encryption_key_id: Option<Uuid>,
    pub unique_name: Option<String>,
    #[arbitrary(default)]
    pub verified_publisher: Option<RawJson<'static>>,
    #[arbitrary(default)]
    pub web: Option<RawJson<'static>>,
    #[arbitrary(default)]
    #[facet(flatten)]
    pub additional_properties: HashMap<String, RawJson<'static>>,
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
