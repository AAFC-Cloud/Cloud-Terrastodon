#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShapeFieldInfo {
    pub field_name: &'static str,
    pub field_shape_name: String,
    pub has_default: bool,
    pub default_value_label: Option<String>,
}
