use cloud_terrastodon_azure_types::uuid::Uuid;
use facet::Facet;
use std::collections::HashMap;

#[derive(Debug, Facet)]
pub struct BatchResponseEntry<T> {
    pub name: Uuid,
    #[facet(rename = "httpStatusCode")]
    pub http_status_code: u16,
    pub headers: HashMap<String, String>,
    pub content: T,
    #[facet(rename = "contentLength")]
    pub content_length: u64,
}
