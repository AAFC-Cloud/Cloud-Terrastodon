use crate::EntraServicePrincipalId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet_json::RawJson;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraServicePrincipal {
    pub account_enabled: bool,
    #[arbitrary(default)]
    pub add_ins: Vec<RawJson<'static>>,
    pub alternative_names: Vec<String>,
    pub app_description: Option<String>,
    pub app_display_name: Option<String>,
    pub app_id: Uuid,
    pub app_owner_organization_id: Option<Uuid>,
    pub app_role_assignment_required: bool,
    #[arbitrary(default)]
    pub app_roles: Vec<RawJson<'static>>,
    pub application_template_id: Option<Uuid>,
    pub created_date_time: DateTime<Utc>,
    #[arbitrary(default)]
    pub deleted_date_time: Option<RawJson<'static>>,
    pub description: Option<String>,
    #[arbitrary(default)]
    pub disabled_by_microsoft_status: Option<RawJson<'static>>,
    pub display_name: String,
    pub homepage: Option<String>,
    pub id: EntraServicePrincipalId,
    #[arbitrary(default)]
    pub info: Option<RawJson<'static>>,
    pub key_credentials: Vec<ServicePrincipalKeyCredential>,
    #[arbitrary(default)]
    pub login_url: Option<RawJson<'static>>,
    pub logout_url: Option<String>,
    pub notes: Option<String>,
    #[arbitrary(default)]
    pub notification_email_addresses: Vec<RawJson<'static>>,
    #[arbitrary(default)]
    pub oauth2_permission_scopes: Vec<RawJson<'static>>,
    pub password_credentials: Vec<ServicePrincipalPasswordCredential>,
    pub preferred_single_sign_on_mode: Option<String>,
    pub preferred_token_signing_key_thumbprint: Option<String>,
    pub reply_urls: Vec<String>,
    #[arbitrary(default)]
    pub resource_specific_application_permissions: Vec<RawJson<'static>>,
    #[arbitrary(default)]
    pub saml_single_sign_on_settings: Option<RawJson<'static>>,
    pub service_principal_names: Vec<String>,
    pub service_principal_type: String,
    pub sign_in_audience: Option<String>,
    #[arbitrary(default)]
    pub tags: Vec<RawJson<'static>>,
    #[arbitrary(default)]
    pub token_encryption_key_id: Option<RawJson<'static>>,
    #[arbitrary(default)]
    pub verified_publisher: Option<RawJson<'static>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ServicePrincipalPasswordCredential {
    #[arbitrary(default)]
    pub custom_key_identifier: Option<RawJson<'static>>,
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
