use crate::Function;
use crate::RegisteredShapeIndex;
use crate::Thing;
use crate::describe_shape;
use facet::Def;
use facet::Facet;
use facet::Shape;
use facet::Type;
use facet::UserType;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

/// A typed reference to a Facet shape available to discovery planning.
///
/// This is type metadata, never a fabricated instance of the referenced type.
/// Its string representation includes the defining module when available;
/// parsing also accepts an unambiguous short display name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = String)]
pub struct RegisteredShape(#[facet(opaque)] &'static Shape);

impl RegisteredShape {
    pub const fn from_shape(shape: &'static Shape) -> Self {
        Self(shape)
    }

    pub const fn shape(self) -> &'static Shape {
        self.0
    }

    /// Collect the supplied registrations and their reflected descendants.
    /// Includes collection elements and request fields without invoking any
    /// registration, constructor, credential provider, or arbitrary generator.
    pub fn all(
        functions: impl IntoIterator<Item = &'static Function>,
        things: impl IntoIterator<Item = &'static Thing>,
    ) -> Vec<Self> {
        let mut pending = things
            .into_iter()
            .map(|thing| thing.shape)
            .collect::<Vec<_>>();
        for function in functions {
            pending.extend([function.input_shape, function.output_shape]);
        }
        let mut seen = HashSet::new();
        while let Some(shape) = pending.pop() {
            if !seen.insert(shape) {
                continue;
            }
            match shape.def {
                Def::List(list) => pending.push(list.t()),
                Def::Set(set) => pending.push(set.t()),
                Def::Array(array) => pending.push(array.t()),
                Def::Slice(slice) => pending.push(slice.t()),
                Def::NdArray(array) => pending.push(array.t()),
                Def::Option(optional) => pending.push(optional.t()),
                Def::Pointer(pointer) => pending.extend(pointer.pointee()),
                Def::Map(map) => pending.extend([map.k(), map.v()]),
                Def::Result(result) => pending.extend([result.t(), result.e()]),
                _ => match shape.ty {
                    Type::User(UserType::Struct(record)) => {
                        pending.extend(record.fields.iter().map(|field| field.shape()));
                    }
                    Type::User(UserType::Enum(enumeration)) => {
                        pending.extend(enumeration.variants.iter().flat_map(|variant| {
                            variant.data.fields.iter().map(|field| field.shape())
                        }));
                    }
                    _ => {}
                },
            }
        }
        let mut shapes = seen.into_iter().map(Self).collect::<Vec<_>>();
        shapes.sort_by_cached_key(ToString::to_string);
        shapes
    }
}

impl fmt::Display for RegisteredShape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(module) = self.0.module_path {
            write!(formatter, "{module}::")?;
        }
        formatter.write_str(&describe_shape(self.0))
    }
}

impl FromStr for RegisteredShape {
    type Err = eyre::Report;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        // The user-facing parsing boundary opts into the linked catalog. The
        // planning algorithm and explicit resolution API remain catalog-driven.
        // Link-time registrations are immutable, so collect and index them only
        // once rather than traversing every shape for each parsed target name.
        static LINKED_SHAPES: LazyLock<RegisteredShapeIndex> = LazyLock::new(|| {
            RegisteredShapeIndex::new(RegisteredShape::all(
                crate::known_functions(),
                crate::known_things(),
            ))
        });
        LINKED_SHAPES.resolve(name)
    }
}

impl TryFrom<String> for RegisteredShape {
    type Error = eyre::Report;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        name.parse()
    }
}

impl From<&RegisteredShape> for String {
    fn from(shape: &RegisteredShape) -> Self {
        shape.to_string()
    }
}

crate::register_thing!(RegisteredShape);
crate::register_thing!(Vec<RegisteredShape>);

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Facet)]
    struct NestedOutput {
        items: Vec<Option<first::Item>>,
    }

    crate::register_thing!(NestedOutput);

    mod first {
        #[derive(facet::Facet)]
        pub struct Item;
    }

    #[test]
    fn catalog_exposes_nested_targets_without_requiring_each_child_to_register() {
        let output = crate::known_thing_for_shape(NestedOutput::SHAPE).unwrap();
        let index = RegisteredShapeIndex::new(RegisteredShape::all([], [output]));
        assert_eq!(index.resolve("Item").unwrap().shape(), first::Item::SHAPE);
        let value = crate::RuntimeValue::from_text(
            RegisteredShape::SHAPE,
            &RegisteredShape::from_shape(first::Item::SHAPE).to_string(),
        )
        .unwrap();
        let value = value
            .into_box::<RegisteredShape>()
            .unwrap()
            .downcast::<RegisteredShape>()
            .unwrap();
        assert_eq!(value.shape(), first::Item::SHAPE);
    }
}
