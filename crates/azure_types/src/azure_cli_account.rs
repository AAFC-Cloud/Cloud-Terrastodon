use crate::ArbitraryJson;
use crate::AzureCliAccountUser;
use crate::AzureTenantId;
use facet::Facet;
use uuid::Uuid;

/// A subscription or tenant-level profile entry returned by [`az account list`](https://learn.microsoft.com/cli/azure/account#az-account-list).
#[derive(Debug, Eq, PartialEq, Facet)]
pub struct AzureCliAccount {
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
    pub user: AzureCliAccountUser,
}
