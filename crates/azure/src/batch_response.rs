use crate::BatchResponseEntry;

#[derive(Debug, facet::Facet)]
pub struct BatchResponse<T> {
    pub responses: Vec<BatchResponseEntry<T>>,
}
