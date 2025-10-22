use crate::prelude::Principal;
use crate::prelude::PrincipalId;
use std::collections::HashMap;
use std::ops::Deref;

pub struct PrincipalCollection(HashMap<PrincipalId, Principal>);

impl PrincipalCollection {
    pub fn new(principals: impl IntoIterator<Item = Principal>) -> Self {
        let lookup = principals
            .into_iter()
            .map(|p| (p.id(), p))
            .collect::<HashMap<_, _>>();
        Self(lookup)
    }
}
impl Deref for PrincipalCollection {
    type Target = HashMap<PrincipalId, Principal>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Debug for PrincipalCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrincipalCollection")
            .field("principal_count", &self.0.len())
            .finish()
    }
}