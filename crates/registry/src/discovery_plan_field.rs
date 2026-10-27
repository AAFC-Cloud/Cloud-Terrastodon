use facet::Facet;
use facet::Shape;
use facet::Type;
use facet::UserType;

/// A declared field on a shape shown in a structural discovery diagram.
///
/// Describes the schema only; planning never inspects or invents field values.
#[derive(Debug, Clone, Facet)]
pub struct DiscoveryPlanField {
    pub name: String,
    pub type_name: String,
}

impl DiscoveryPlanField {
    pub(crate) fn for_shape(shape: &'static Shape) -> Vec<Self> {
        match shape.ty {
            Type::User(UserType::Struct(record)) => record
                .fields
                .iter()
                .map(|field| Self {
                    name: field.effective_name().to_owned(),
                    type_name: crate::describe_shape(field.shape()),
                })
                .collect(),
            Type::User(UserType::Enum(enumeration)) => enumeration
                .variants
                .iter()
                .flat_map(|variant| {
                    variant.data.fields.iter().map(move |field| Self {
                        name: format!("{}.{}", variant.name, field.effective_name()),
                        type_name: crate::describe_shape(field.shape()),
                    })
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}
