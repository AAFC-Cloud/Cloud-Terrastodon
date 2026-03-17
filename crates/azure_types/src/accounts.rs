use crate::tenant_id::AzureTenantId;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Account {
    #[serde(rename = "cloudName")]
    pub cloud_name: String,
    #[serde(rename = "homeTenantId")]
    pub home_tenant_id: AzureTenantId,
    pub id: Uuid,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
    #[serde(rename = "managedByTenants")]
    pub managed_by_tenants: Vec<Value>,
    pub name: String,
    pub state: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: AzureTenantId,
    pub user: AccountUser,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TenantIdHolder {
    #[serde(rename = "tenantId")]
    pub tenant_id: AzureTenantId,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountUser {
    pub name: String,
    // #[serde(rename="type")]
    // kind: UserKind,
}
