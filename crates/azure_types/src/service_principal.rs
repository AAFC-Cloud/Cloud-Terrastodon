use crate::EntraServicePrincipalId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct EntraServicePrincipal {
    #[serde(rename = "accountEnabled")]
    pub account_enabled: bool,
    #[serde(rename = "addIns")]
    #[arbitrary(default)]
    pub add_ins: Vec<Value>,
    #[serde(rename = "alternativeNames")]
    pub alternative_names: Vec<String>,
    #[serde(rename = "appDescription")]
    pub app_description: Option<String>,
    #[serde(rename = "appDisplayName")]
    pub app_display_name: Option<String>,
    #[serde(rename = "appId")]
    pub app_id: Uuid,
    #[serde(rename = "appOwnerOrganizationId")]
    pub app_owner_organization_id: Option<Uuid>,
    #[serde(rename = "appRoleAssignmentRequired")]
    pub app_role_assignment_required: bool,
    #[serde(rename = "appRoles")]
    #[arbitrary(default)]
    pub app_roles: Vec<Value>,
    #[serde(rename = "applicationTemplateId")]
    pub application_template_id: Option<Uuid>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: DateTime<Utc>,
    #[serde(rename = "deletedDateTime")]
    #[arbitrary(default)]
    pub deleted_date_time: Option<Value>,
    pub description: Option<String>,
    #[serde(rename = "disabledByMicrosoftStatus")]
    #[arbitrary(default)]
    pub disabled_by_microsoft_status: Option<Value>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub homepage: Option<String>,
    pub id: EntraServicePrincipalId,
    #[arbitrary(default)]
    pub info: Option<Value>,
    #[serde(rename = "keyCredentials")]
    pub key_credentials: Vec<ServicePrincipalKeyCredential>,
    #[serde(rename = "loginUrl")]
    #[arbitrary(default)]
    pub login_url: Option<Value>,
    #[serde(rename = "logoutUrl")]
    pub logout_url: Option<String>,
    pub notes: Option<String>,
    #[serde(rename = "notificationEmailAddresses")]
    #[arbitrary(default)]
    pub notification_email_addresses: Vec<Value>,
    #[serde(rename = "oauth2PermissionScopes")]
    #[arbitrary(default)]
    pub oauth2_permission_scopes: Vec<Value>,
    #[serde(rename = "passwordCredentials")]
    pub password_credentials: Vec<ServicePrincipalPasswordCredential>,
    #[serde(rename = "preferredSingleSignOnMode")]
    pub preferred_single_sign_on_mode: Option<String>,
    #[serde(rename = "preferredTokenSigningKeyThumbprint")]
    pub preferred_token_signing_key_thumbprint: Option<String>,
    #[serde(rename = "replyUrls")]
    pub reply_urls: Vec<String>,
    #[serde(rename = "resourceSpecificApplicationPermissions")]
    #[arbitrary(default)]
    pub resource_specific_application_permissions: Vec<Value>,
    #[serde(rename = "samlSingleSignOnSettings")]
    #[arbitrary(default)]
    pub saml_single_sign_on_settings: Option<Value>,
    #[serde(rename = "servicePrincipalNames")]
    pub service_principal_names: Vec<String>,
    #[serde(rename = "servicePrincipalType")]
    pub service_principal_type: String,
    #[serde(rename = "signInAudience")]
    pub sign_in_audience: Option<String>,
    #[arbitrary(default)]
    pub tags: Vec<Value>,
    #[serde(rename = "tokenEncryptionKeyId")]
    #[arbitrary(default)]
    pub token_encryption_key_id: Option<Value>,
    #[serde(rename = "verifiedPublisher")]
    #[arbitrary(default)]
    pub verified_publisher: Value,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct ServicePrincipalPasswordCredential {
    #[serde(rename = "customKeyIdentifier")]
    #[arbitrary(default)]
    pub custom_key_identifier: Option<Value>,
    #[serde(rename = "endDateTime")]
    pub end_date_time: DateTime<Utc>,
    #[serde(rename = "keyId")]
    pub key_id: Uuid,
    #[serde(rename = "startDateTime")]
    pub start_date_time: DateTime<Utc>,
    #[serde(rename = "secretText")]
    pub secret_text: Option<String>,
    pub hint: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct ServicePrincipalKeyCredential {
    #[serde(rename = "customKeyIdentifier")]
    pub custom_key_identifier: Option<String>,
    #[serde(rename = "endDateTime")]
    pub end_date_time: DateTime<Utc>,
    #[serde(rename = "keyId")]
    pub key_id: Uuid,
    #[serde(rename = "startDateTime")]
    pub start_date_time: DateTime<Utc>,
    #[serde(rename = "type")]
    pub kind: String,
    pub usage: String,
    pub key: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "hasExtendedValue")]
    pub has_extended_value: Option<bool>,
}
