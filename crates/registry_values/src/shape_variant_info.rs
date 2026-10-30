use crate::ShapeFieldInfo;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShapeVariantInfo {
    pub variant_name: &'static str,
    pub payload_shape_name: Option<String>,
    pub payload_fields: Vec<ShapeFieldInfo>,
    pub is_default: bool,
}
