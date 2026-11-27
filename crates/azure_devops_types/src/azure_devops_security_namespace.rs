use crate::AzureDevOpsSecurityAction;
use facet::Facet;
use uuid::Uuid;

/// A permission namespace from [Security Namespaces Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/security-namespaces/query?view=azure-devops-rest-7.1#securitynamespacedescription).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsSecurityNamespace {
    pub namespace_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub actions: Vec<AzureDevOpsSecurityAction>,
    pub dataspace_category: String,
    pub element_length: i32,
    pub extension_type: Option<String>,
    pub is_remotable: bool,
    pub read_permission: i32,
    pub separator_value: String,
    pub structure_value: i32,
    /// Bits reserved by the system store. Some namespace responses omit this mask.
    pub system_bit_mask: Option<i32>,
    pub use_token_translator: bool,
    pub write_permission: i32,
}

impl AzureDevOpsSecurityNamespace {
    /// Render known action names and retain any unmapped bits for inspection.
    pub fn action_names(&self, mask: i32) -> Vec<String> {
        let mut remaining = u32::from_ne_bytes(mask.to_ne_bytes());
        let mut names = Vec::new();
        for action in &self.actions {
            let bit = u32::from_ne_bytes(action.bit.to_ne_bytes());
            if remaining & bit != 0 {
                names.push(action.display_name.clone());
                remaining &= !bit;
            }
        }
        if remaining != 0 {
            names.push(format!("Unknown bits 0x{remaining:08x}"));
        }
        names
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsSecurityNamespace);
