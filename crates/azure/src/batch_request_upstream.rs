use crate::BatchRequestEntry;
use facet::Facet;

/// The batch's HTTP body, borrowing entries without authentication or execution policy.
#[derive(Debug, Facet)]
pub(crate) struct BatchRequestUpstream<'a, T> {
    pub requests: &'a [BatchRequestEntry<T>],
}
