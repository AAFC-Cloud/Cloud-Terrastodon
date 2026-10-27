/// One observed field or lifecycle property of an object, independent of its renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcalidrawObjectField {
    pub name: String,
    pub type_name: Option<String>,
    pub value: String,
}
