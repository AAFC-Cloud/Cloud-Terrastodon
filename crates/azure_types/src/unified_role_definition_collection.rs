use crate::prelude::UnifiedRoleDefinition;
use crate::prelude::UnifiedRoleDefinitionId;
use std::collections::HashMap;
use std::ops::Deref;

/// A collection of Entra role definitions, indexed by their template ID.
///
/// Not to be confused with Azure RBAC role definitions.
#[derive(Debug, Clone)]
pub struct UnifiedRoleDefinitionCollection {
    lookup: HashMap<UnifiedRoleDefinitionId, UnifiedRoleDefinition>,
    canonicalized: bool,
}
impl Deref for UnifiedRoleDefinitionCollection {
    type Target = HashMap<UnifiedRoleDefinitionId, UnifiedRoleDefinition>;
    fn deref(&self) -> &Self::Target {
        &self.lookup
    }
}
impl UnifiedRoleDefinitionCollection {
    pub fn new(roles: impl IntoIterator<Item = UnifiedRoleDefinition>) -> Self {
        let lookup = roles.into_iter().map(|r| (r.template_id, r)).collect();
        Self {
            lookup,
            canonicalized: false,
        }
    }

    /// Ensures that all role definitions in the collection have their inherited permissions.
    pub fn canonicalize(&mut self) {
        if self.canonicalized {
            return;
        }
        let lookup = self.lookup.clone();
        for role in self.lookup.values_mut() {
            role.canonicalize(&lookup);
        }
    }
}
impl IntoIterator for UnifiedRoleDefinitionCollection {
    type Item = UnifiedRoleDefinition;
    type IntoIter =
        std::collections::hash_map::IntoValues<UnifiedRoleDefinitionId, UnifiedRoleDefinition>;
    fn into_iter(self) -> Self::IntoIter {
        self.lookup.into_values()
    }
}
