use arbitrary::Arbitrary;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointOwner {
    Library,
}
