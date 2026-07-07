use crate::SubnetId;
use crate::SubnetName;
use crate::SubnetProperties;
use arbitrary::Arbitrary;

#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct Subnet {
    pub id: SubnetId,
    pub name: SubnetName,
    pub properties: SubnetProperties,
}
