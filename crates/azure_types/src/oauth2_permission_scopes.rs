use uuid::Uuid;

#[derive(Debug, Hash, Eq, PartialEq, facet::Facet)]
#[repr(C)]
pub enum OAuth2PermissionScopeKind {
    Admin,
    User,
}
#[derive(Debug, Hash, Eq, PartialEq, facet::Facet)]
pub struct OAuth2PermissionScope {
    #[facet(rename = "adminConsentDescription")]
    pub admin_consent_description: String,
    #[facet(rename = "adminConsentDisplayName")]
    pub admin_consent_display_name: String,
    #[facet(rename = "id")]
    pub id: Uuid,
    #[facet(rename = "isEnabled")]
    pub is_enabled: bool,
    #[facet(rename = "type")]
    pub kind: OAuth2PermissionScopeKind,
    #[facet(rename = "userConsentDescription")]
    pub user_consent_description: String,
    #[facet(rename = "userConsentDisplayName")]
    pub user_consent_display_name: String,
    #[facet(rename = "value")]
    pub value: String,
}
