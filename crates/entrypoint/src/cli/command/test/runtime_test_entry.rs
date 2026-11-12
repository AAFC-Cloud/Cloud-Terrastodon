use cloud_terrastodon_registry::Effect;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry_execution::RuntimeTestRegistration;
use facet::Facet;
use facet::Type;
use facet::UserType;

/// Public metadata for one operator verification case.
#[derive(Debug, Facet)]
pub struct RuntimeTestEntry {
    pub name: String,
    pub request: String,
    pub effects: Vec<Effect>,
    pub input_fields: Vec<String>,
}

impl From<&RuntimeTestRegistration> for RuntimeTestEntry {
    fn from(registration: &RuntimeTestRegistration) -> Self {
        Self {
            name: registration.name.to_owned(),
            request: describe_shape(registration.request.input_shape),
            effects: registration.request.effects.to_vec(),
            input_fields: match registration.request.input_shape.ty {
                Type::User(UserType::Struct(record)) => record
                    .fields
                    .iter()
                    .map(|field| field.name.to_owned())
                    .collect(),
                _ => Vec::new(),
            },
        }
    }
}
