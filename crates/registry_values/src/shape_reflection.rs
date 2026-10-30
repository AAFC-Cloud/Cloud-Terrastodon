use crate::InputDependency;
use crate::ShapeFieldInfo;
use crate::ShapeVariantInfo;
use crate::Thing;
use facet::Def;
use facet::Shape;
use facet::Type;
use facet::UserType;
use facet_reflect::Partial;

pub fn input_dependencies(shape: &'static Shape) -> Vec<InputDependency> {
    match shape.ty {
        Type::User(UserType::Struct(struct_type)) => struct_type
            .fields
            .iter()
            .filter(|field| !field.should_skip_deserializing())
            .map(|field| InputDependency {
                field_name: field.name,
                shape: field.shape(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub fn shape_fields_for_thing(thing: &Thing) -> Vec<ShapeFieldInfo> {
    shape_fields(thing.shape)
}

pub fn shape_variants_for_thing(thing: &Thing) -> Vec<ShapeVariantInfo> {
    shape_variants(thing.shape)
}

pub fn list_element_shape(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::List(list) => Some(list.t()),
        _ => None,
    }
}

pub fn sequence_element_shape(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::List(list) => Some(list.t()),
        Def::Set(set) => Some(set.t()),
        _ => indirect_inner_shape(shape).and_then(sequence_element_shape),
    }
}

pub fn map_value_shape(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::Map(map) => Some(map.v()),
        _ => indirect_inner_shape(shape).and_then(map_value_shape),
    }
}

pub fn shape_field_shape(shape: &'static Shape, field_name: &str) -> Option<&'static Shape> {
    match shape.ty {
        Type::User(UserType::Struct(struct_type)) => struct_type
            .fields
            .iter()
            .find(|field| field.effective_name() == field_name)
            .map(|field| field.proxy_shape().unwrap_or_else(|| field.shape())),
        _ => indirect_inner_shape(shape).and_then(|inner| shape_field_shape(inner, field_name)),
    }
}

fn indirect_inner_shape(shape: &'static Shape) -> Option<&'static Shape> {
    transparent_inner_shape(shape).or_else(|| optional_inner_shape(shape))
}

fn transparent_inner_shape(shape: &'static Shape) -> Option<&'static Shape> {
    if !shape.is_transparent() {
        return None;
    }

    match shape.ty {
        Type::User(UserType::Struct(struct_type)) if struct_type.fields.len() == 1 => {
            Some(struct_type.fields[0].shape())
        }
        _ => None,
    }
}

fn optional_inner_shape(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::Option(option) => Some(option.t()),
        _ => None,
    }
}

pub fn describe_shape(shape: &'static Shape) -> String {
    if let Some(element_shape) = list_element_shape(shape) {
        return format!("List<{}>", describe_shape(element_shape));
    }
    shape.type_name().to_string()
}

fn shape_fields(shape: &'static Shape) -> Vec<ShapeFieldInfo> {
    match shape.ty {
        Type::User(UserType::Struct(struct_type)) => struct_type
            .fields
            .iter()
            .filter(|field| !field.should_skip_deserializing())
            .map(shape_field_info)
            .collect(),
        _ => Vec::new(),
    }
}

fn shape_variants(shape: &'static Shape) -> Vec<ShapeVariantInfo> {
    match shape.ty {
        Type::User(UserType::Enum(enum_type)) => {
            let default_variant_index = default_variant_index_for_shape(shape);
            enum_type
                .variants
                .iter()
                .enumerate()
                .map(|(variant_index, variant)| ShapeVariantInfo {
                    variant_name: variant.effective_name(),
                    payload_shape_name: variant_payload_shape_name(variant),
                    payload_fields: variant.data.fields.iter().map(shape_field_info).collect(),
                    is_default: default_variant_index == Some(variant_index),
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn default_variant_index_for_shape(shape: &'static Shape) -> Option<usize> {
    let partial = unsafe { Partial::alloc_shape_owned(shape).ok()? };
    let value = partial.set_default().ok()?.build().ok()?;
    value.peek().into_enum().ok()?.variant_index().ok()
}

fn shape_field_info(field: &facet::Field) -> ShapeFieldInfo {
    ShapeFieldInfo {
        field_name: field.effective_name(),
        field_shape_name: describe_shape(field.shape()),
        has_default: field.has_default(),
        default_value_label: default_value_label(field),
    }
}

fn variant_payload_shape_name(variant: &facet::Variant) -> Option<String> {
    match variant.data.fields {
        [] => None,
        [field] => Some(describe_shape(field.shape())),
        fields => Some(
            fields
                .iter()
                .map(|field| describe_shape(field.shape()))
                .collect::<Vec<_>>()
                .join(", "),
        ),
    }
}

fn default_value_label(field: &facet::Field) -> Option<String> {
    if !field.has_default() {
        return None;
    }
    let field_shape = field.shape();
    match field_shape.ty {
        Type::User(UserType::Enum(enum_type))
            if enum_type
                .variants
                .iter()
                .any(|variant| variant.effective_name() == "Default") =>
        {
            Some(format!("{}::Default", describe_shape(field_shape)))
        }
        _ => Some("<default>".to_string()),
    }
}
