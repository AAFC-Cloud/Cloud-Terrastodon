use crate::RegisteredShape;
use crate::describe_shape;
use std::collections::HashMap;

/// Resolves names within an explicitly supplied catalog of shape identities.
///
/// Qualified and short names are indexed once. Lookups take expected constant
/// time in the number of shapes, plus the cost of hashing the supplied name.
/// Construction only inspects metadata; it does not consult the linked registry
/// or invoke any registered operation.
#[derive(Debug)]
pub struct RegisteredShapeIndex {
    // A name identifies either one shape or an ambiguity (None). Keeping the
    // index immutable prevents later insertions from hiding an alias collision.
    by_name: HashMap<String, Option<RegisteredShape>>,
}

impl RegisteredShapeIndex {
    pub fn new(shapes: impl IntoIterator<Item = RegisteredShape>) -> Self {
        let mut by_name = HashMap::new();
        for shape in shapes {
            for name in [shape.to_string(), describe_shape(shape.shape())] {
                by_name
                    .entry(name)
                    .and_modify(|existing| {
                        if *existing != Some(shape) {
                            *existing = None;
                        }
                    })
                    .or_insert(Some(shape));
            }
        }
        Self { by_name }
    }

    /// Reject unknown and ambiguous names rather than choosing by link order.
    pub fn resolve(&self, name: &str) -> eyre::Result<RegisteredShape> {
        match self.by_name.get(name) {
            Some(Some(shape)) => Ok(*shape),
            Some(None) => Err(eyre::eyre!(
                "Shape name {name:?} is ambiguous; use its module-qualified name"
            )),
            None => Err(eyre::eyre!("No registered shape matches {name:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use facet::Facet;

    mod first {
        #[derive(facet::Facet)]
        pub struct Item;
    }

    mod second {
        #[derive(facet::Facet)]
        pub struct Item;
    }

    #[test]
    fn duplicate_names_require_a_module_and_resolution_preserves_identity() {
        let first = RegisteredShape::from_shape(first::Item::SHAPE);
        let second = RegisteredShape::from_shape(second::Item::SHAPE);
        for shapes in [[first, second, first], [second, first, second]] {
            let index = RegisteredShapeIndex::new(shapes);
            assert!(
                index
                    .resolve("Item")
                    .unwrap_err()
                    .to_string()
                    .contains("ambiguous")
            );
            assert_eq!(index.resolve(&first.to_string()).unwrap(), first);
            assert_eq!(index.resolve(&second.to_string()).unwrap(), second);
            assert!(
                index
                    .resolve("Missing")
                    .unwrap_err()
                    .to_string()
                    .contains("No registered shape")
            );
        }
    }

    #[test]
    fn repeated_shapes_and_identical_aliases_do_not_create_ambiguity() {
        let item = RegisteredShape::from_shape(first::Item::SHAPE);
        let primitive = RegisteredShape::from_shape(usize::SHAPE);
        let index = RegisteredShapeIndex::new([item, item, primitive, primitive]);
        assert_eq!(index.resolve("Item").unwrap(), item);
        assert_eq!(index.resolve("usize").unwrap(), primitive);
        assert_eq!(index.resolve(&primitive.to_string()).unwrap(), primitive);
    }

    #[test]
    fn colliding_qualified_names_remain_ambiguous() {
        // The displayed container name does not qualify its element type, so
        // even a module-qualified alias can identify multiple distinct shapes.
        let first = RegisteredShape::from_shape(Vec::<first::Item>::SHAPE);
        let second = RegisteredShape::from_shape(Vec::<second::Item>::SHAPE);
        assert_ne!(first, second);
        assert_eq!(first.to_string(), second.to_string());
        let index = RegisteredShapeIndex::new([first, second, first]);
        for name in [first.to_string(), describe_shape(first.shape())] {
            assert!(
                index
                    .resolve(&name)
                    .unwrap_err()
                    .to_string()
                    .contains("ambiguous")
            );
        }
    }

    #[test]
    fn explicit_indexes_only_resolve_their_own_catalog() {
        let first = RegisteredShape::from_shape(first::Item::SHAPE);
        let second = RegisteredShape::from_shape(second::Item::SHAPE);
        let first_index = RegisteredShapeIndex::new([first]);
        let second_index = RegisteredShapeIndex::new([second]);
        assert_eq!(first_index.resolve("Item").unwrap(), first);
        assert_eq!(second_index.resolve("Item").unwrap(), second);
        assert!(first_index.resolve(&second.to_string()).is_err());
        assert!(second_index.resolve(&first.to_string()).is_err());
        assert!(RegisteredShapeIndex::new([]).resolve("Item").is_err());
    }
}
