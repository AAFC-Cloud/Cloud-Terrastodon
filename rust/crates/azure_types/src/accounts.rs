use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::tenants::TenantId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "cloudName")]
    cloud_name: String,
    #[serde(rename = "homeTenantId")]
    home_tenant_id: TenantId,
    id: Uuid,
    #[serde(rename = "isDefault")]
    is_default: bool,
    #[serde(rename = "managedByTenants")]
    managed_by_tenants: String,
    name: String,
    state: String,
    #[serde(rename = "tenantId")]
    tenant_id: String,
    user: AccountUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantIdHolder {
    #[serde(rename = "tenantId")]
    pub tenant_id: TenantId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountUser {
    name: String,
    // #[serde(rename="type")]
    // kind: UserKind,
}