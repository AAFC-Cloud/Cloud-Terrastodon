use arbitrary::Arbitrary;

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointOwner {
    Library,
}
