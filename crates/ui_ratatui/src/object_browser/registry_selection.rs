use cloud_terrastodon_registry::RegisteredFunction;
use cloud_terrastodon_registry::RegisteredShape;
use facet::Facet;
use facet::Shape;

/// Typed registry identities selected for a discovery request field.
pub(super) enum RegistrySelection {
    Shapes(Vec<RegisteredShape>),
    Functions(Vec<RegisteredFunction>),
}

impl RegistrySelection {
    pub(super) fn supports(shape: &'static Shape) -> bool {
        shape.is_shape(Vec::<RegisteredShape>::SHAPE)
            || shape.is_shape(Vec::<RegisteredFunction>::SHAPE)
    }

    pub(super) fn len(&self) -> usize {
        match self {
            Self::Shapes(shapes) => shapes.len(),
            Self::Functions(functions) => functions.len(),
        }
    }

    pub(super) fn description(&self) -> &'static str {
        match self {
            Self::Shapes(_) => "shapes",
            Self::Functions(_) => "functions",
        }
    }
}
