use crate::MicrosoftGraphBatchRequestEntry;
use std::collections::HashMap;

#[derive(Debug, facet::Facet)]
pub(crate) struct MicrosoftGraphBatchRequestEntryWire<T> {
    pub(crate) id: String,
    pub(crate) method: String,
    pub(crate) url: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Option<T>,
}

impl<T> From<MicrosoftGraphBatchRequestEntry<T>> for MicrosoftGraphBatchRequestEntryWire<T> {
    fn from(value: MicrosoftGraphBatchRequestEntry<T>) -> Self {
        Self {
            id: value.id,
            method: value.method.as_str().to_string(),
            url: value.url,
            headers: value.headers,
            body: value.body,
        }
    }
}
