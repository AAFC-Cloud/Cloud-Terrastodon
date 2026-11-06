use crate::UnifiedRoleDefinition;
use crate::UnifiedRoleDefinitionId;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

/// A collection of Entra role definitions, indexed by their template ID.
///
/// Not to be confused with Azure RBAC role definitions.
#[derive(Debug, Clone, arbitrary::Arbitrary, facet::Facet)]
pub struct UnifiedRoleDefinitionCollection {
    lookup: HashMap<UnifiedRoleDefinitionId, Arc<UnifiedRoleDefinition>>,
    canonicalized: bool,
}
impl Deref for UnifiedRoleDefinitionCollection {
    type Target = HashMap<UnifiedRoleDefinitionId, Arc<UnifiedRoleDefinition>>;
    fn deref(&self) -> &Self::Target {
        &self.lookup
    }
}
impl UnifiedRoleDefinitionCollection {
    pub fn new(roles: impl IntoIterator<Item = Arc<UnifiedRoleDefinition>>) -> Self {
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
            if role.inherits_permissions_from.is_empty() {
                continue;
            }
            // Expanded permissions belong to this collection, not the cached source.
            Arc::make_mut(role).canonicalize(&lookup);
        }
    }
}
impl IntoIterator for UnifiedRoleDefinitionCollection {
    type Item = Arc<UnifiedRoleDefinition>;
    type IntoIter =
        std::collections::hash_map::IntoValues<UnifiedRoleDefinitionId, Arc<UnifiedRoleDefinition>>;
    fn into_iter(self) -> Self::IntoIter {
        self.lookup.into_values()
    }
}
