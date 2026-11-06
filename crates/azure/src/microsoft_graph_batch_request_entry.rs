use http::Method;
use std::collections::HashMap;

/// One operation in a [Microsoft Graph JSON batch](https://learn.microsoft.com/graph/json-batching#creating-a-batch-request).
#[derive(Debug)]
pub struct MicrosoftGraphBatchRequestEntry<T> {
    pub id: String,
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    /// None if this is a GET request
    pub body: Option<T>,
}

impl<T> MicrosoftGraphBatchRequestEntry<T> {
    pub fn new(
        id: String,
        method: Method,
        url: String,
        headers: HashMap<String, String>,
        body: Option<T>,
    ) -> Self {
        MicrosoftGraphBatchRequestEntry {
            id,
            method,
            url: Self::prepare_url(url),
            headers,
            body,
        }
    }

    /// We want the ID to be consistent such that caching change detection doesn't get unnecessarily invalidated.
    pub fn new_get(id: String, url: String) -> Self {
        MicrosoftGraphBatchRequestEntry {
            id,
            method: Method::GET,
            url: Self::prepare_url(url),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn prepare_url(url: String) -> String {
        url.trim_start_matches("https://graph.microsoft.com/v1.0")
            .to_string()
    }
}
