use crate::http_method_proxy::HttpMethodProxy;
use cloud_terrastodon_azure_types::uuid::Uuid;
use facet::Facet;
use http::Method;

#[derive(Debug, Clone, Facet)]
pub struct AzureBatchRequestEntry<T> {
    #[facet(opaque, rename = "httpMethod", proxy = HttpMethodProxy)]
    pub http_method: Method,
    pub name: Uuid,
    pub url: String,
    pub content: Option<T>,
}

impl AzureBatchRequestEntry<()> {
    pub fn new_get(url: String) -> Self {
        AzureBatchRequestEntry {
            http_method: Method::GET,
            name: Uuid::new_v4(),
            url,
            content: None,
        }
    }
}
impl<T> AzureBatchRequestEntry<T> {
    pub fn new(http_method: Method, url: String, content: Option<T>) -> Self {
        AzureBatchRequestEntry {
            http_method,
            name: Uuid::new_v4(),
            url,
            content,
        }
    }
}
