use crate::ManagementGroupId;
use arbitrary::Arbitrary;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct ManagementGroupAncestorsChain(Vec<ManagementGroupAncestorsChainEntry>);
impl Deref for ManagementGroupAncestorsChain {
    type Target = Vec<ManagementGroupAncestorsChainEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ManagementGroupAncestorsChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct ManagementGroupAncestorsChainEntry {
    pub name: String,
    #[facet(rename = "displayName")]
    pub display_name: String,
}
impl ManagementGroupAncestorsChainEntry {
    pub fn id(&self) -> ManagementGroupId {
        ManagementGroupId::from_name(&self.name)
    }
}

cloud_terrastodon_registry::register_thing!(ManagementGroupAncestorsChain);
cloud_terrastodon_registry::register_arbitrary!(ManagementGroupAncestorsChain);
