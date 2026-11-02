use crate::Thing;

#[derive(Clone)]
pub struct KnownShapeInfo {
    pub thing: &'static Thing,
    pub label: String,
}
