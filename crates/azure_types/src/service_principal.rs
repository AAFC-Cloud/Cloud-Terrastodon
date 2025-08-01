use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ServicePrincipalId(Uuid);
impl UuidWrapper for ServicePrincipalId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(ServicePrincipalId);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServicePrincipal {
    #[serde(rename = "accountEnabled")]
    pub account_enabled: bool,
    #[serde(rename = "addIns")]
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
    pub app_roles: Vec<Value>,
    #[serde(rename = "applicationTemplateId")]
    pub application_template_id: Option<Uuid>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: DateTime<Utc>,
    #[serde(rename = "deletedDateTime")]
    pub deleted_date_time: Option<Value>,
    pub description: Option<String>,
    #[serde(rename = "disabledByMicrosoftStatus")]
    pub disabled_by_microsoft_status: Option<Value>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub homepage: Option<String>,
    pub id: ServicePrincipalId,
    pub info: Option<Value>,
    #[serde(rename = "keyCredentials")]
    pub key_credentials: Vec<Value>,
    #[serde(rename = "loginUrl")]
    pub login_url: Option<Value>,
    #[serde(rename = "logoutUrl")]
    pub logout_url: Option<String>,
    pub notes: Option<String>,
    #[serde(rename = "notificationEmailAddresses")]
    pub notification_email_addresses: Vec<Value>,
    #[serde(rename = "oauth2PermissionScopes")]
    pub oauth2_permission_scopes: Vec<Value>,
    #[serde(rename = "passwordCredentials")]
    pub password_credentials: Vec<Value>,
    #[serde(rename = "preferredSingleSignOnMode")]
    pub preferred_single_sign_on_mode: Option<String>,
    #[serde(rename = "preferredTokenSigningKeyThumbprint")]
    pub preferred_token_signing_key_thumbprint: Option<String>,
    #[serde(rename = "replyUrls")]
    pub reply_urls: Vec<String>,
    #[serde(rename = "resourceSpecificApplicationPermissions")]
    pub resource_specific_application_permissions: Vec<Value>,
    #[serde(rename = "samlSingleSignOnSettings")]
    pub saml_single_sign_on_settings: Option<Value>,
    #[serde(rename = "servicePrincipalNames")]
    pub service_principal_names: Vec<String>,
    #[serde(rename = "servicePrincipalType")]
    pub service_principal_type: String,
    #[serde(rename = "signInAudience")]
    pub sign_in_audience: Option<String>,
    pub tags: Vec<Value>,
    #[serde(rename = "tokenEncryptionKeyId")]
    pub token_encryption_key_id: Option<Value>,
    #[serde(rename = "verifiedPublisher")]
    pub verified_publisher: Value,
}
