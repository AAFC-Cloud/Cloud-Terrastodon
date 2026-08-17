use super::breadcrumb::Breadcrumb;

#[derive(Clone, Debug, Default, Eq, PartialEq, facet::Facet)]
#[repr(C)]
pub(crate) struct Breadcrumbs {
    /// Ordered, composable projection/filter operations. This is the query
    /// program itself, never an evaluated result collection.
    operations: Vec<Breadcrumb>,
}

impl Breadcrumbs {
    pub(crate) fn new(operations: Vec<Breadcrumb>) -> Self {
        Self { operations }
    }

    pub(crate) fn operations(&self) -> &[Breadcrumb] {
        &self.operations
    }

    pub(crate) fn into_operations(self) -> Vec<Breadcrumb> {
        self.operations
    }

    pub(crate) fn push(&mut self, operation: Breadcrumb) {
        self.operations.push(operation);
    }

    pub(crate) fn remove(&mut self, index: usize) -> Breadcrumb {
        self.operations.remove(index)
    }

    pub(crate) fn replace(&mut self, index: usize, operation: Breadcrumb) -> Option<Breadcrumb> {
        let existing = self.operations.get_mut(index)?;
        Some(std::mem::replace(existing, operation))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

cloud_terrastodon_registry::register_thing!(Breadcrumbs);
