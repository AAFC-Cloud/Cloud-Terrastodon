use crate::prelude::Principal;
use crate::prelude::PrincipalId;
use std::collections::HashMap;
use std::ops::Deref;

pub struct PrincipalCollection(HashMap<PrincipalId, Principal>);

impl PrincipalCollection {
    pub fn new(principals: impl IntoIterator<Item = Principal>) -> Self {
        let lookup = principals
            .into_iter()
            .map(|p| (p.id().clone(), p))
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
