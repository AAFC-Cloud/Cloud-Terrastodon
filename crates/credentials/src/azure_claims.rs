use chrono::DateTime;
use chrono::Local;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::PrincipalId;
use cloud_terrastodon_azure_types::uuid::Uuid;

#[derive(Debug, facet::Facet, Clone)]
pub struct AzureClaims {
    #[facet(rename = "aud")]
    pub audience: String,
    #[facet(rename = "iss")]
    pub issuer: String,
    #[facet(rename = "iat")]
    #[facet(proxy = cloud_terrastodon_azure_types::LocalDateTimeEpochSecondsProxy)]
    pub issued_at: DateTime<Local>,
    #[facet(rename = "nbf")]
    #[facet(proxy = cloud_terrastodon_azure_types::LocalDateTimeEpochSecondsProxy)]
    pub not_before: DateTime<Local>,
    #[facet(rename = "exp")]
    #[facet(proxy = cloud_terrastodon_azure_types::LocalDateTimeEpochSecondsProxy)]
    pub expires: DateTime<Local>,
    #[facet(rename = "acr")]
    pub authentication_context_class: String,
    #[facet(rename = "acrs")]
    pub authentication_context_classes: Vec<String>,
    #[facet(rename = "aio")]
    pub aio: String,
    #[facet(rename = "amr")]
    pub authentication_methods: Vec<String>,
    #[facet(rename = "appid")]
    pub app_id: EntraApplicationClientId,
    #[facet(rename = "appidacr")]
    pub app_id_acr: String,
    #[facet(rename = "deviceid")]
    pub device_id: Option<Uuid>,
    #[facet(rename = "family_name")]
    pub family_name: String,
    #[facet(rename = "given_name")]
    pub given_name: String,
    #[facet(rename = "groups")]
    pub groups: Vec<EntraGroupId>,
    #[facet(rename = "idtyp")]
    pub identity_type: String,
    #[facet(rename = "ipaddr")]
    pub ip_address: String,
    #[facet(rename = "name")]
    pub name: String,
    #[facet(rename = "oid")]
    pub object_id: PrincipalId,
    #[facet(rename = "puid")]
    pub puid: String,
    #[facet(rename = "pwd_url")]
    pub password_change_url: Option<String>,
    #[facet(rename = "rh")]
    pub refresh_token_hash: String,
    #[facet(rename = "scp")]
    pub scopes: String,
    #[facet(rename = "sid")]
    pub session_id: Uuid,
    #[facet(rename = "sub")]
    pub subject: String,
    #[facet(rename = "tid")]
    pub tenant_id: AzureTenantId,
    #[facet(rename = "unique_name")]
    pub unique_name: String,
    #[facet(rename = "upn")]
    pub user_principal_name: Option<String>,
    #[facet(rename = "uti")]
    pub uti: String,
    #[facet(rename = "ver")]
    pub version: String,
    #[facet(rename = "wids")]
    pub windows_integrated_device_ids: Vec<Uuid>,
    #[facet(rename = "xms_ftd")]
    pub xms_ftd: String,
    #[facet(rename = "xms_idrel")]
    pub xms_idrel: String,
    #[facet(rename = "xms_tcdt")]
    pub xms_tcdt: i64,
}
