use crate::SubnetId;
use crate::SubnetName;
use crate::SubnetProperties;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct Subnet {
    pub id: SubnetId,
    pub name: SubnetName,
    pub properties: SubnetProperties,
}
