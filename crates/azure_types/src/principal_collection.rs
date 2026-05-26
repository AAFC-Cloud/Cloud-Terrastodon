use crate::Principal;
use crate::PrincipalId;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

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
impl std::fmt::Debug for PrincipalCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrincipalCollection")
            .field("principal_count", &self.0.len())
            .finish()
    }
}
