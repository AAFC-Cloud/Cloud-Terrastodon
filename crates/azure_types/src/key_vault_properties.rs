use crate::tenants::TenantId;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Selected Key Vault properties as returned by Azure. Some complex nested collections remain
/// loosely typed (`Value`) until modeled (see TODOs).
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct KeyVaultProperties {
    pub network_acls: Option<NetworkAcls>,
    pub provisioning_state: ProvisioningState,
    pub enabled_for_deployment: bool,
    pub tenant_id: TenantId,
    pub public_network_access: PublicNetworkAccess,
    pub enabled_for_disk_encryption: Option<bool>,
    pub enable_soft_delete: Option<bool>,
    pub enable_rbac_authorization: Option<bool>,
    pub vault_uri: String, // URL string
    /// Access policies (raw). TODO: Vec<AccessPolicy>
    pub access_policies: Value,
    pub soft_delete_retention_in_days: Option<u32>,
    pub enabled_for_template_deployment: Option<bool>,
    pub enable_purge_protection: Option<bool>,
    /// Private endpoint connections (raw). TODO: Vec<PrivateEndpointConnection>
    pub private_endpoint_connections: Option<Value>,
    pub sku: KeyVaultSku,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct KeyVaultSku {
    pub name: String,
    pub family: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAcls {
    pub virtual_network_rules: Value, // TODO: Vec<VirtualNetworkRule>
    pub ip_rules: Value,              // TODO: Vec<IpRule>
    pub default_action: Value,        // TODO: enum DefaultAction (Allow / Deny)
    pub bypass: Value,                // TODO: enum Bypass (AzureServices / None)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum ProvisioningState {
    Succeeded,
    Failed,
    Canceled,
    Creating,
    Updating,
    Deleting,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum PublicNetworkAccess {
    Enabled,
    Disabled,
    #[serde(other)]
    Unknown,
}
