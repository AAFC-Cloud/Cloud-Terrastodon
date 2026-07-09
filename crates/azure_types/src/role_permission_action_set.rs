use crate::RolePermissionAction;
use arbitrary::Arbitrary;
use indexmap::IndexSet;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default, PartialEq, Eq, Clone, facet::Facet, Arbitrary)]
#[facet(transparent)]
pub struct RolePermissionActionSet(IndexSet<RolePermissionAction>);

impl Deref for RolePermissionActionSet {
    type Target = IndexSet<RolePermissionAction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RolePermissionActionSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::hash::Hash for RolePermissionActionSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Sort before hashing so logical equality does not depend on insertion order.
        let mut sorted_actions: Vec<_> = self.0.iter().collect();
        sorted_actions.sort();
        for action in sorted_actions {
            action.hash(state);
        }
    }
}

impl FromIterator<RolePermissionAction> for RolePermissionActionSet {
    fn from_iter<T: IntoIterator<Item = RolePermissionAction>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for RolePermissionActionSet {
    type Item = RolePermissionAction;
    type IntoIter = indexmap::set::IntoIter<RolePermissionAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a RolePermissionActionSet {
    type Item = &'a RolePermissionAction;
    type IntoIter = indexmap::set::Iter<'a, RolePermissionAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut RolePermissionActionSet {
    type Item = &'a RolePermissionAction;
    type IntoIter = indexmap::set::Iter<'a, RolePermissionAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
