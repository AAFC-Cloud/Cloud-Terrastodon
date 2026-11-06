use crate::MicrosoftGraphBatchResponseEntryError;

/// A Graph batch item is either its successful resource body or an error envelope.
/// [JSON batch responses](https://learn.microsoft.com/graph/json-batching#processing-the-json-batch-response).
#[derive(Debug, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum MicrosoftGraphBatchResponseEntryBody<T> {
    Error {
        error: MicrosoftGraphBatchResponseEntryError,
    },
    Success(T),
}
