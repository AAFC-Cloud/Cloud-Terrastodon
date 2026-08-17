/// Stable identity for one named field selected by a projection breadcrumb.
///
/// The owner shape is part of the identity because unrelated reflected
/// structs may use the same field name with different meanings.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, facet::Facet)]
#[repr(C)]
pub(crate) struct ProjectedField {
    owner_shape: String,
    field_name: String,
}

impl ProjectedField {
    pub(crate) fn new(owner_shape: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            owner_shape: owner_shape.into(),
            field_name: field_name.into(),
        }
    }

    pub(crate) fn owner_shape(&self) -> &str {
        &self.owner_shape
    }

    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    pub(crate) fn label(&self) -> String {
        format!("{}.{}", self.owner_shape, self.field_name)
    }
}
