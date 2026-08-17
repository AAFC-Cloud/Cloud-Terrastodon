use super::builder_field_snapshot::BuilderFieldSnapshot;
use super::value_builder::ValueBuilder;
use facet::Shape;
use facet::Type;
use facet::UserType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BuilderKindSnapshot {
    ShapeUnset,
    Scalar {
        value_is_set: bool,
    },
    Struct,
    Enum {
        selected_variant: Option<usize>,
        selected_variant_name: Option<String>,
    },
}

/// Bounded, non-owning construction metadata for one Building root.
#[derive(Clone, Debug)]
pub(crate) struct BuilderSnapshot {
    shape: Option<&'static Shape>,
    shape_name: Option<String>,
    kind: BuilderKindSnapshot,
    fields: Vec<BuilderFieldSnapshot>,
    fields_complete: bool,
}

impl BuilderSnapshot {
    pub(crate) fn shape_unset() -> Self {
        Self {
            shape: None,
            shape_name: None,
            kind: BuilderKindSnapshot::ShapeUnset,
            fields: Vec::new(),
            fields_complete: true,
        }
    }

    pub(crate) fn observe(builder: &ValueBuilder, max_fields: usize) -> Self {
        let shape = builder.shape();
        let field_count = builder.field_count();
        let fields = (0..field_count.min(max_fields))
            .map(|index| {
                BuilderFieldSnapshot::new(
                    index,
                    builder
                        .field_name(index)
                        .expect("field count came from this builder"),
                    builder
                        .field_shape(index)
                        .expect("field count came from this builder"),
                    builder
                        .field_has_default(index)
                        .expect("field count came from this builder"),
                    builder
                        .field_binding(index)
                        .expect("field count came from this builder"),
                )
            })
            .collect();
        let kind = match shape.ty {
            Type::User(UserType::Struct(_)) => BuilderKindSnapshot::Struct,
            Type::User(UserType::Enum(object)) => {
                let selected_variant = builder.selected_variant();
                let selected_variant_name = selected_variant.and_then(|index| {
                    object
                        .variants
                        .get(index)
                        .map(|variant| variant.name.to_owned())
                });
                BuilderKindSnapshot::Enum {
                    selected_variant,
                    selected_variant_name,
                }
            }
            _ => BuilderKindSnapshot::Scalar {
                value_is_set: builder.scalar_is_set(),
            },
        };
        Self {
            shape: Some(shape),
            shape_name: Some(cloud_terrastodon_registry::describe_shape(shape).to_owned()),
            kind,
            fields,
            fields_complete: field_count <= max_fields,
        }
    }

    pub(crate) const fn shape(&self) -> Option<&'static Shape> {
        self.shape
    }

    pub(crate) fn shape_name(&self) -> Option<&str> {
        self.shape_name.as_deref()
    }

    pub(crate) const fn kind(&self) -> &BuilderKindSnapshot {
        &self.kind
    }

    pub(crate) fn fields(&self) -> &[BuilderFieldSnapshot] {
        &self.fields
    }

    pub(crate) const fn fields_complete(&self) -> bool {
        self.fields_complete
    }
}
