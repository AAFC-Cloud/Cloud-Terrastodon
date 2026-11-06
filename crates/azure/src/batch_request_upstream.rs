use crate::BatchRequestEntry;

/// The batch's HTTP body, borrowing entries without authentication or execution policy.
#[derive(Debug, facet::Facet)]
pub(crate) struct BatchRequestUpstream<'a, T> {
    pub requests: &'a [BatchRequestEntry<T>],
}
