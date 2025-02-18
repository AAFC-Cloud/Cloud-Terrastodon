use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum OAuth2PermissionScopeKind {
    Admin,
    User
}
#[derive(Serialize, Deserialize, Debug)]
pub struct OAuth2PermissionScope {
    #[serde(rename = "adminConsentDescription")]
    pub admin_consent_description: String,
    #[serde(rename = "adminConsentDisplayName")]
    pub admin_consent_display_name: String,
    #[serde(rename = "id")]
    pub id: Uuid,
    #[serde(rename = "isEnabled")]
    pub is_enabled: bool,
    #[serde(rename = "type")]
    pub kind: OAuth2PermissionScopeKind,
    #[serde(rename = "userConsentDescription")]
    pub user_consent_description: String,
    #[serde(rename = "userConsentDisplayName")]
    pub user_consent_display_name: String,
    #[serde(rename = "value")]
    pub value: String,
}