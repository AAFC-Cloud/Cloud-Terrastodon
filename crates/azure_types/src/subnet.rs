use crate::SubnetId;
use crate::SubnetName;
use crate::SubnetProperties;
use arbitrary::Arbitrary;
use facet::Facet;

#[derive(Debug, PartialEq, Clone, Arbitrary, Facet)]
pub struct Subnet {
    pub id: SubnetId,
    pub name: SubnetName,
    pub properties: SubnetProperties,
}
