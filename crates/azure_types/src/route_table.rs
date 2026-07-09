use crate::ResourceGroupId;
use crate::RouteTableId;
use crate::RouteTableProperties;
use arbitrary::Arbitrary;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct RouteTable {
    pub id: RouteTableId,
    pub name: String, // This is the name from Azure, distinct from RouteTableName in ID
    pub location: String,
    #[facet(default, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    pub properties: RouteTableProperties,
}

impl RouteTable {
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}

cloud_terrastodon_registry::register_thing!(RouteTable);
cloud_terrastodon_registry::register_arbitrary!(RouteTable);
cloud_terrastodon_registry::register_arbitrary!(Vec<RouteTable>);
