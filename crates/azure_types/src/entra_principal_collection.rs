use crate::EntraGroupId;
use crate::EntraPrincipal;
use crate::EntraPrincipalId;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug, Clone, Facet)]
pub struct EntraPrincipalCollection(pub HashMap<EntraPrincipalId, EntraPrincipal>);

impl<'a> Arbitrary<'a> for EntraPrincipalCollection {
    fn arbitrary(unstructured: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Keys identify their records, including when inventory discovery uses
        // the shared collection output. Independent map-key generation breaks
        // the same relationship maintained by normal inventory assembly.
        Ok(Self::new(Vec::<EntraPrincipal>::arbitrary(unstructured)?))
    }
}

impl EntraPrincipalCollection {
    pub fn new(principals: impl IntoIterator<Item = EntraPrincipal>) -> Self {
        let lookup = principals
            .into_iter()
            .map(|p| (p.id(), p))
            .collect::<HashMap<_, _>>();
        Self(lookup)
    }

    pub fn get(&self, id: &EntraPrincipalId) -> Option<&EntraPrincipal> {
        self.0.get(id)
    }
}
impl Deref for EntraPrincipalCollection {
    type Target = HashMap<EntraPrincipalId, EntraPrincipal>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for EntraPrincipalCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

cloud_terrastodon_registry::register_arbitrary!(std::collections::HashMap<EntraGroupId, Vec<EntraPrincipal>>);

cloud_terrastodon_registry::register_arbitrary!(EntraPrincipalCollection);
