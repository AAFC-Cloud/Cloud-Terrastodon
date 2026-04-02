use crate::AppId;
use crate::EntraApplicationRegistrationId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct EntraApplicationRegistration {
    #[serde(rename = "addIns")]
    #[arbitrary(default)]
    pub add_ins: Vec<Value>,
    #[arbitrary(default)]
    pub api: Option<Value>,
    #[serde(rename = "appId")]
    pub app_id: AppId,
    #[serde(rename = "applicationTemplateId")]
    pub application_template_id: Option<Uuid>,
    #[serde(rename = "appRoles")]
    #[arbitrary(default)]
    pub app_roles: Vec<Value>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<DateTime<Utc>>,
    #[serde(rename = "defaultRedirectUri")]
    pub default_redirect_uri: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "disabledByMicrosoftStatus")]
    pub disabled_by_microsoft_status: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "groupMembershipClaims")]
    pub group_membership_claims: Option<String>,
    pub id: EntraApplicationRegistrationId,
    #[serde(rename = "identifierUris")]
    pub identifier_uris: Vec<String>,
    #[arbitrary(default)]
    pub info: Option<Value>,
    #[serde(rename = "isDeviceOnlyAuthSupported")]
    pub is_device_only_auth_supported: Option<bool>,
    #[serde(rename = "isFallbackPublicClient")]
    pub is_fallback_public_client: Option<bool>,
    #[serde(rename = "keyCredentials")]
    #[arbitrary(default)]
    pub key_credentials: Vec<Value>,
    pub notes: Option<String>,
    #[serde(rename = "optionalClaims")]
    #[arbitrary(default)]
    pub optional_claims: Option<Value>,
    #[serde(rename = "parentalControlSettings")]
    #[arbitrary(default)]
    pub parental_control_settings: Option<Value>,
    #[serde(rename = "passwordCredentials")]
    #[arbitrary(default)]
    pub password_credentials: Vec<Value>,
    #[serde(rename = "publicClient")]
    #[arbitrary(default)]
    pub public_client: Option<Value>,
    #[serde(rename = "publisherDomain")]
    pub publisher_domain: Option<String>,
    #[serde(rename = "requestSignatureVerification")]
    #[arbitrary(default)]
    pub request_signature_verification: Option<Value>,
    #[serde(rename = "requiredResourceAccess")]
    #[arbitrary(default)]
    pub required_resource_access: Vec<Value>,
    #[serde(rename = "serviceManagementReference")]
    pub service_management_reference: Option<String>,
    #[serde(rename = "servicePrincipalLockConfiguration")]
    #[arbitrary(default)]
    pub service_principal_lock_configuration: Option<Value>,
    #[serde(rename = "signInAudience")]
    pub sign_in_audience: Option<String>,
    #[arbitrary(default)]
    pub spa: Option<Value>,
    pub tags: Vec<String>,
    #[serde(rename = "tokenEncryptionKeyId")]
    pub token_encryption_key_id: Option<Uuid>,
    #[serde(rename = "uniqueName")]
    pub unique_name: Option<String>,
    #[serde(rename = "verifiedPublisher")]
    #[arbitrary(default)]
    pub verified_publisher: Option<Value>,
    #[arbitrary(default)]
    pub web: Option<Value>,
    #[serde(flatten)]
    #[arbitrary(default)]
    pub additional_properties: HashMap<String, Value>,
}

impl std::fmt::Display for EntraApplicationRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id.to_string().as_str())?;
        f.write_str(" - ")?;
        f.write_str(&self.display_name)?;
        Ok(())
    }
}
