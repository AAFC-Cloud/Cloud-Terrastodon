use std::fmt;

use facet::Shape;

use super::field_binding::FieldBinding;
use super::field_binding_snapshot::FieldBindingSnapshot;

/// UI-neutral metadata for one field of an incomplete reflected value.
///
/// The static Shape reference is registry metadata, not an arena value. It is
/// safe to retain in picker state and lets the UI ask the generic candidate
/// service for compatible values without looking back into ValueBuilder.
#[derive(Clone)]
pub(crate) struct BuilderFieldSnapshot {
    index: usize,
    name: String,
    shape: &'static Shape,
    shape_name: String,
    has_default: bool,
    binding: FieldBindingSnapshot,
}

impl BuilderFieldSnapshot {
    pub(crate) fn new(
        index: usize,
        name: impl Into<String>,
        shape: &'static Shape,
        has_default: bool,
        binding: &FieldBinding,
    ) -> Self {
        Self {
            index,
            name: name.into(),
            shape,
            shape_name: cloud_terrastodon_registry::describe_shape(shape).to_owned(),
            has_default,
            binding: FieldBindingSnapshot::from(binding),
        }
    }

    pub(crate) const fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) const fn shape(&self) -> &'static Shape {
        self.shape
    }

    pub(crate) fn shape_name(&self) -> &str {
        &self.shape_name
    }

    pub(crate) const fn has_default(&self) -> bool {
        self.has_default
    }

    pub(crate) const fn binding(&self) -> &FieldBindingSnapshot {
        &self.binding
    }
}

impl fmt::Debug for BuilderFieldSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BuilderFieldSnapshot")
            .field("index", &self.index)
            .field("name", &self.name)
            .field("shape", &self.shape_name)
            .field("has_default", &self.has_default)
            .field("binding", &self.binding)
            .finish()
    }
}
