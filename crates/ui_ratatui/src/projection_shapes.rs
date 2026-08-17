use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::map_value_shape;
use cloud_terrastodon_registry::sequence_element_shape;
use facet::Shape;
use facet::Type;
use facet::UserType;
use std::collections::BTreeSet;

fn proxied_container_shape(mut shape: &'static Shape) -> &'static Shape {
    loop {
        if let Some(proxy) = shape.effective_proxy(None)
            && !proxy.shape.is_shape(shape)
        {
            shape = proxy.shape;
            continue;
        }
        return shape;
    }
}

/// Shape names reachable below a reflected value without visiting instances.
///
/// Query pruning uses this static shape graph only; it never materializes
/// projection addresses or inspects a collection's elements.
pub(crate) fn projection_shape_names(shape: &'static Shape) -> BTreeSet<String> {
    fn visit_children(
        shape: &'static Shape,
        labels: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) {
        let shape = proxied_container_shape(shape);
        if let Some(element_shape) = sequence_element_shape(shape) {
            visit(element_shape, labels, visited);
            return;
        }
        if let Some(value_shape) = map_value_shape(shape) {
            visit(value_shape, labels, visited);
            return;
        }
        if shape.is_transparent()
            && let Some(inner) = shape.inner
            && !inner.is_shape(shape)
        {
            visit_children(inner, labels, visited);
            return;
        }
        match shape.ty {
            Type::User(UserType::Struct(struct_type)) => {
                for field in struct_type.fields {
                    if !field.should_skip_serializing_unconditional() {
                        visit(
                            field
                                .effective_proxy(None)
                                .map(|proxy| proxy.shape)
                                .unwrap_or_else(|| field.shape()),
                            labels,
                            visited,
                        );
                    }
                }
            }
            Type::User(UserType::Enum(enum_type)) => {
                for variant in enum_type.variants {
                    for field in variant.data.fields {
                        if !field.should_skip_serializing_unconditional() {
                            visit(
                                field
                                    .effective_proxy(None)
                                    .map(|proxy| proxy.shape)
                                    .unwrap_or_else(|| field.shape()),
                                labels,
                                visited,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn visit(shape: &'static Shape, labels: &mut BTreeSet<String>, visited: &mut BTreeSet<String>) {
        let label = describe_shape(shape);
        labels.insert(label.clone());
        if visited.insert(label) {
            visit_children(shape, labels, visited);
        }
    }

    let mut labels = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit(shape, &mut labels, &mut visited);
    labels
}
