use crate::KeyVaultAccessPolicy;
use crate::tenant_id::AzureTenantId;
use facet_json::RawJson;
use std::convert::Infallible;

/// Selected Key Vault properties as returned by Azure. Some complex nested collections remain
/// as raw JSON until modeled (see TODOs).
#[derive(Debug, PartialEq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct KeyVaultProperties {
    pub network_acls: Option<NetworkAcls>,
    pub provisioning_state: ProvisioningState,
    pub enabled_for_deployment: bool,
    pub tenant_id: AzureTenantId,
    pub public_network_access: PublicNetworkAccess,
    pub enabled_for_disk_encryption: Option<bool>,
    pub enable_soft_delete: Option<bool>,
    pub enable_rbac_authorization: Option<bool>,
    pub vault_uri: String, // URL string
    pub access_policies: Vec<KeyVaultAccessPolicy>,
    pub soft_delete_retention_in_days: Option<u32>,
    pub enabled_for_template_deployment: Option<bool>,
    pub enable_purge_protection: Option<bool>,
    /// Private endpoint connections (raw). TODO: Vec<PrivateEndpointConnection>
    pub private_endpoint_connections: Option<RawJson<'static>>,
    pub sku: KeyVaultSku,
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
pub struct KeyVaultSku {
    pub name: String,
    pub family: String,
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct NetworkAcls {
    pub virtual_network_rules: RawJson<'static>, // TODO: Vec<VirtualNetworkRule>
    pub ip_rules: RawJson<'static>,              // TODO: Vec<IpRule>
    pub default_action: RawJson<'static>,        // TODO: enum DefaultAction (Allow / Deny)
    pub bypass: RawJson<'static>,                // TODO: enum Bypass (AzureServices / None)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum ProvisioningState {
    Succeeded,
    Failed,
    Canceled,
    Creating,
    Updating,
    Deleting,
    Unknown,
}

impl TryFrom<String> for ProvisioningState {
    type Error = Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "Succeeded" => Self::Succeeded,
            "Failed" => Self::Failed,
            "Canceled" => Self::Canceled,
            "Creating" => Self::Creating,
            "Updating" => Self::Updating,
            "Deleting" => Self::Deleting,
            _ => Self::Unknown,
        })
    }
}

impl From<&ProvisioningState> for String {
    fn from(value: &ProvisioningState) -> Self {
        match value {
            ProvisioningState::Succeeded => "Succeeded",
            ProvisioningState::Failed => "Failed",
            ProvisioningState::Canceled => "Canceled",
            ProvisioningState::Creating => "Creating",
            ProvisioningState::Updating => "Updating",
            ProvisioningState::Deleting => "Deleting",
            ProvisioningState::Unknown => "Unknown",
        }
        .to_owned()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum PublicNetworkAccess {
    Enabled,
    Disabled,
    Unknown,
}

impl TryFrom<String> for PublicNetworkAccess {
    type Error = Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "Enabled" => Self::Enabled,
            "Disabled" => Self::Disabled,
            _ => Self::Unknown,
        })
    }
}

impl From<&PublicNetworkAccess> for String {
    fn from(value: &PublicNetworkAccess) -> Self {
        match value {
            PublicNetworkAccess::Enabled => "Enabled",
            PublicNetworkAccess::Disabled => "Disabled",
            PublicNetworkAccess::Unknown => "Unknown",
        }
        .to_owned()
    }
}
