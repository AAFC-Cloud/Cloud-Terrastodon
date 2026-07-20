use crate::ArbitraryJson;
use crate::EntraApplicationClientId;
use crate::EntraServicePrincipalObjectId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraServicePrincipal {
    pub account_enabled: bool,
    pub add_ins: Vec<ArbitraryJson>,
    pub alternative_names: Vec<String>,
    pub app_description: Option<String>,
    pub app_display_name: Option<String>,
    pub app_id: EntraApplicationClientId,
    pub app_owner_organization_id: Option<Uuid>,
    pub app_role_assignment_required: bool,
    pub app_roles: Vec<ArbitraryJson>,
    pub application_template_id: Option<Uuid>,
    pub created_date_time: DateTime<Utc>,
    pub deleted_date_time: Option<ArbitraryJson>,
    pub description: Option<String>,
    pub disabled_by_microsoft_status: Option<ArbitraryJson>,
    pub display_name: String,
    pub homepage: Option<String>,
    pub id: EntraServicePrincipalObjectId,
    pub info: Option<ArbitraryJson>,
    pub key_credentials: Vec<ServicePrincipalKeyCredential>,
    pub login_url: Option<ArbitraryJson>,
    pub logout_url: Option<String>,
    pub notes: Option<String>,
    pub notification_email_addresses: Vec<ArbitraryJson>,
    pub oauth2_permission_scopes: Vec<ArbitraryJson>,
    pub password_credentials: Vec<ServicePrincipalPasswordCredential>,
    pub preferred_single_sign_on_mode: Option<String>,
    pub preferred_token_signing_key_thumbprint: Option<String>,
    pub reply_urls: Vec<String>,
    pub resource_specific_application_permissions: Vec<ArbitraryJson>,
    pub saml_single_sign_on_settings: Option<ArbitraryJson>,
    pub service_principal_names: Vec<String>,
    pub service_principal_type: String,
    pub sign_in_audience: Option<String>,
    pub tags: Vec<ArbitraryJson>,
    pub token_encryption_key_id: Option<ArbitraryJson>,
    pub verified_publisher: Option<ArbitraryJson>,
}

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ServicePrincipalPasswordCredential {
    pub custom_key_identifier: Option<ArbitraryJson>,
    pub end_date_time: DateTime<Utc>,
    pub key_id: Uuid,
    pub start_date_time: DateTime<Utc>,
    pub secret_text: Option<String>,
    pub hint: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ServicePrincipalKeyCredential {
    pub custom_key_identifier: Option<String>,
    pub end_date_time: DateTime<Utc>,
    pub key_id: Uuid,
    pub start_date_time: DateTime<Utc>,
    #[facet(rename = "type")]
    pub kind: String,
    pub usage: String,
    pub key: Option<String>,
    pub display_name: Option<String>,
    pub has_extended_value: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(EntraServicePrincipal);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipal);
cloud_terrastodon_registry::register_arbitrary!(Vec<EntraServicePrincipal>);
