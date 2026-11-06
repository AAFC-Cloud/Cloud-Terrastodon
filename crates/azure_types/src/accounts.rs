use crate::ArbitraryJson;
use crate::tenant_id::AzureTenantId;
use facet::Facet;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Facet)]
pub struct Account {
    #[facet(rename = "cloudName")]
    pub cloud_name: String,
    #[facet(rename = "homeTenantId")]
    pub home_tenant_id: Option<AzureTenantId>,
    pub id: Uuid,
    #[facet(rename = "isDefault")]
    pub is_default: bool,
    #[facet(rename = "managedByTenants")]
    pub managed_by_tenants: Option<Vec<ArbitraryJson>>,
    pub name: String,
    pub state: String,
    #[facet(rename = "tenantId")]
    pub tenant_id: AzureTenantId,
    pub user: AccountUser,
}

#[derive(Debug, Eq, PartialEq, Facet)]
pub struct TenantIdHolder {
    #[facet(rename = "tenantId")]
    pub tenant_id: AzureTenantId,
}

#[derive(Debug, Eq, PartialEq, Facet)]
pub struct AccountUser {
    pub name: String,
    #[facet(rename = "type")]
    pub r#type: AccountUserType,
}

#[derive(Debug, Eq, PartialEq, Facet)]
#[repr(C)]
pub enum AccountUserType {
    #[facet(rename = "user")]
    User,
    #[facet(rename = "servicePrincipal")]
    ServicePrincipal,
    #[facet(other)]
    Other(String),
}
