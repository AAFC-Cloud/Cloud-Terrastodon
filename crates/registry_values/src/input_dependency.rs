use facet::Shape;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InputDependency {
    pub field_name: &'static str,
    pub shape: &'static Shape,
}
