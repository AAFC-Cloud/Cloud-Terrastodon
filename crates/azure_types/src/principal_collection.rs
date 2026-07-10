use crate::EntraGroupId;
use crate::Principal;
use crate::PrincipalId;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug, Clone, arbitrary::Arbitrary, facet::Facet)]
pub struct PrincipalCollection(pub HashMap<PrincipalId, Principal>);

impl PrincipalCollection {
    pub fn new(principals: impl IntoIterator<Item = Principal>) -> Self {
        let lookup = principals
            .into_iter()
            .map(|p| (p.id(), p))
            .collect::<HashMap<_, _>>();
        Self(lookup)
    }

    pub fn get(&self, id: &PrincipalId) -> Option<&Principal> {
        self.0.get(id)
    }
}
impl Deref for PrincipalCollection {
    type Target = HashMap<PrincipalId, Principal>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for PrincipalCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

cloud_terrastodon_registry::register_arbitrary!(std::collections::HashMap<EntraGroupId, Vec<Principal>>);
