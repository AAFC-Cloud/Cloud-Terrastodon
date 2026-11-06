use crate::BatchResponseEntry;
use facet::Facet;

#[derive(Debug, Facet)]
pub struct BatchResponse<T> {
    pub responses: Vec<BatchResponseEntry<T>>,
}
