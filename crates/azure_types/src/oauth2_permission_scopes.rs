use arbitrary::Arbitrary;
use uuid::Uuid;

#[derive(Debug, Hash, Eq, PartialEq, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum OAuth2PermissionScopeKind {
    Admin,
    User,
}
#[derive(Debug, Hash, Eq, PartialEq, Arbitrary, facet::Facet)]
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

cloud_terrastodon_registry::register_thing!(OAuth2PermissionScopeKind);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionScopeKind);
cloud_terrastodon_registry::register_thing!(OAuth2PermissionScope);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionScope);
cloud_terrastodon_registry::register_arbitrary!(Vec<OAuth2PermissionScope>);
