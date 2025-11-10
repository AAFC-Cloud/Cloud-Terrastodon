use chrono::DateTime;
use chrono::Local;
use cloud_terrastodon_azure_types::prelude::AppId;
use cloud_terrastodon_azure_types::prelude::GroupId;
use cloud_terrastodon_azure_types::prelude::PrincipalId;
use cloud_terrastodon_azure_types::prelude::TenantId;
use cloud_terrastodon_azure_types::prelude::uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct AzureClaims {
    #[serde(rename = "aud")]
    pub audience: String,
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "iat")]
    #[serde(
        deserialize_with = "cloud_terrastodon_azure_types::serde_helpers::deserialize_local_date_time_from_epoch"
    )]
    pub issued_at: DateTime<Local>,
    #[serde(rename = "nbf")]
    #[serde(
        deserialize_with = "cloud_terrastodon_azure_types::serde_helpers::deserialize_local_date_time_from_epoch"
    )]
    pub not_before: DateTime<Local>,
    #[serde(rename = "exp")]
    #[serde(
        deserialize_with = "cloud_terrastodon_azure_types::serde_helpers::deserialize_local_date_time_from_epoch"
    )]
    pub expires: DateTime<Local>,
    #[serde(rename = "acr")]
    pub authentication_context_class: String,
    #[serde(rename = "acrs")]
    pub authentication_context_classes: Vec<String>,
    #[serde(rename = "aio")]
    pub aio: String,
    #[serde(rename = "amr")]
    pub authentication_methods: Vec<String>,
    #[serde(rename = "appid")]
    pub app_id: AppId,
    #[serde(rename = "appidacr")]
    pub app_id_acr: String,
    #[serde(rename = "deviceid")]
    pub device_id: Option<Uuid>,
    #[serde(rename = "family_name")]
    pub family_name: String,
    #[serde(rename = "given_name")]
    pub given_name: String,
    #[serde(rename = "groups")]
    pub groups: Vec<GroupId>,
    #[serde(rename = "idtyp")]
    pub identity_type: String,
    #[serde(rename = "ipaddr")]
    pub ip_address: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "oid")]
    pub object_id: PrincipalId,
    #[serde(rename = "puid")]
    pub puid: String,
    #[serde(rename = "pwd_url")]
    pub password_change_url: Option<String>,
    #[serde(rename = "rh")]
    pub refresh_token_hash: String,
    #[serde(rename = "scp")]
    pub scopes: String,
    #[serde(rename = "sid")]
    pub session_id: Uuid,
    #[serde(rename = "sub")]
    pub subject: String,
    #[serde(rename = "tid")]
    pub tenant_id: TenantId,
    #[serde(rename = "unique_name")]
    pub unique_name: String,
    #[serde(rename = "upn")]
    pub user_principal_name: Option<String>,
    #[serde(rename = "uti")]
    pub uti: String,
    #[serde(rename = "ver")]
    pub version: String,
    #[serde(rename = "wids")]
    pub windows_integrated_device_ids: Vec<Uuid>,
    #[serde(rename = "xms_ftd")]
    pub xms_ftd: String,
    #[serde(rename = "xms_idrel")]
    pub xms_idrel: String,
    #[serde(rename = "xms_tcdt")]
    pub xms_tcdt: i64,
}
