use crate::SerializableRestResponse;

/// The accepted body and the response whose metadata accompanies that body.
/// Shared execution stores an `Arc<T>` body so callers can retain it without
/// cloning either the decoded value or its original response.
pub(crate) struct DecodedRestResponse<T> {
    pub(crate) value: T,
    pub(crate) response: SerializableRestResponse,
}
