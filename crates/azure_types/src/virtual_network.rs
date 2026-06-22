use crate::ResourceGroupId;
use crate::VirtualNetworkId;
use crate::VirtualNetworkName;
use crate::VirtualNetworkProperties;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct VirtualNetwork {
    pub id: VirtualNetworkId,
    pub name: VirtualNetworkName,
    pub location: String,
    #[facet(default, opaque, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    pub properties: VirtualNetworkProperties,
}

impl VirtualNetwork {
    // Helper to get the ResourceGroupId from the VirtualNetworkId
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}
