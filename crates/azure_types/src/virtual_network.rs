use crate::ResourceGroupId;
use crate::VirtualNetworkId;
use crate::VirtualNetworkName;
use crate::VirtualNetworkProperties;
use arbitrary::Arbitrary;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct VirtualNetwork {
    pub id: VirtualNetworkId,
    pub name: VirtualNetworkName,
    pub location: String,
    #[facet(default, opaque, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    pub properties: VirtualNetworkProperties,
}

impl VirtualNetwork {
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}

cloud_terrastodon_registry::register_thing!(VirtualNetwork);
cloud_terrastodon_registry::register_arbitrary!(VirtualNetwork);
cloud_terrastodon_registry::register_arbitrary!(Vec<VirtualNetwork>);
