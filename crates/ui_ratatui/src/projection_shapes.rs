use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::map_value_shape as registry_map_value_shape;
use cloud_terrastodon_registry::sequence_element_shape as registry_sequence_element_shape;
use cloud_terrastodon_registry::shape_field_shape as registry_shape_field_shape;
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

pub(crate) fn projection_field_shape(
    shape: &'static Shape,
    field_name: &str,
) -> Option<&'static Shape> {
    let shape = proxied_container_shape(shape);
    if shape.is_transparent()
        && let Some(inner) = shape.inner
        && !inner.is_shape(shape)
    {
        return projection_field_shape(inner, field_name);
    }
    match shape.ty {
        Type::User(UserType::Struct(struct_type)) => struct_type
            .fields
            .iter()
            .find(|field| {
                !field.should_skip_serializing_unconditional()
                    && field.effective_name() == field_name
            })
            .map(|field| {
                field
                    .effective_proxy(None)
                    .map(|proxy| proxy.shape)
                    .unwrap_or_else(|| field.shape())
            }),
        _ => registry_shape_field_shape(shape, field_name),
    }
}

pub(crate) fn projection_sequence_element_shape(shape: &'static Shape) -> Option<&'static Shape> {
    registry_sequence_element_shape(proxied_container_shape(shape))
}

pub(crate) fn projection_map_value_shape(shape: &'static Shape) -> Option<&'static Shape> {
    registry_map_value_shape(proxied_container_shape(shape))
}

pub(crate) fn projection_shape_names(shape: &'static Shape) -> BTreeSet<String> {
    fn visit_children(
        shape: &'static Shape,
        labels: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) {
        let shape = proxied_container_shape(shape);
        if let Some(element_shape) = registry_sequence_element_shape(shape) {
            visit(element_shape, labels, visited);
            return;
        }
        if let Some(value_shape) = registry_map_value_shape(shape) {
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
                    if field.should_skip_serializing_unconditional() {
                        continue;
                    }
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
            Type::User(UserType::Enum(enum_type)) => {
                for variant in enum_type.variants {
                    for field in variant.data.fields {
                        if field.should_skip_serializing_unconditional() {
                            continue;
                        }
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
            _ => {}
        }
    }

    fn visit(shape: &'static Shape, labels: &mut BTreeSet<String>, visited: &mut BTreeSet<String>) {
        let label = describe_shape(shape);
        labels.insert(label.clone());
        if !visited.insert(label) {
            return;
        }

        visit_children(shape, labels, visited);
    }

    let mut labels = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit(shape, &mut labels, &mut visited);
    labels
}

pub(crate) fn projection_fields(shape: &'static Shape) -> BTreeSet<(String, String)> {
    fn visit(
        shape: &'static Shape,
        fields: &mut BTreeSet<(String, String)>,
        visited: &mut BTreeSet<String>,
    ) {
        let logical_label = describe_shape(shape);
        if !visited.insert(logical_label) {
            return;
        }
        let shape = proxied_container_shape(shape);
        if let Some(element_shape) = registry_sequence_element_shape(shape) {
            visit(element_shape, fields, visited);
            return;
        }
        if let Some(value_shape) = registry_map_value_shape(shape) {
            visit(value_shape, fields, visited);
            return;
        }
        if shape.is_transparent()
            && let Some(inner) = shape.inner
            && !inner.is_shape(shape)
        {
            visit(inner, fields, visited);
            return;
        }
        match shape.ty {
            Type::User(UserType::Struct(struct_type)) => {
                for field in struct_type.fields {
                    if field.should_skip_serializing_unconditional() {
                        continue;
                    }
                    let field_shape = field
                        .effective_proxy(None)
                        .map(|proxy| proxy.shape)
                        .unwrap_or_else(|| field.shape());
                    fields.insert((
                        describe_shape(field_shape),
                        field.effective_name().to_string(),
                    ));
                    visit(field_shape, fields, visited);
                }
            }
            Type::User(UserType::Enum(enum_type)) => {
                for variant in enum_type.variants {
                    for field in variant.data.fields {
                        if field.should_skip_serializing_unconditional() {
                            continue;
                        }
                        let field_shape = field
                            .effective_proxy(None)
                            .map(|proxy| proxy.shape)
                            .unwrap_or_else(|| field.shape());
                        fields.insert((
                            describe_shape(field_shape),
                            field.effective_name().to_string(),
                        ));
                        visit(field_shape, fields, visited);
                    }
                }
            }
            _ => {}
        }
    }

    let mut fields = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit(shape, &mut fields, &mut visited);
    fields
}
