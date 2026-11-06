use crate::http_method_proxy::HttpMethodProxy;
use cloud_terrastodon_azure_types::uuid::Uuid;
use http::Method;

#[derive(Debug, Clone, facet::Facet)]
pub struct BatchRequestEntry<T> {
    #[facet(opaque, rename = "httpMethod", proxy = HttpMethodProxy)]
    pub http_method: Method,
    pub name: Uuid,
    pub url: String,
    pub content: Option<T>,
}

impl BatchRequestEntry<()> {
    pub fn new_get(url: String) -> Self {
        BatchRequestEntry {
            http_method: Method::GET,
            name: Uuid::new_v4(),
            url,
            content: None,
        }
    }
}
impl<T> BatchRequestEntry<T> {
    pub fn new(http_method: Method, url: String, content: Option<T>) -> Self {
        BatchRequestEntry {
            http_method,
            name: Uuid::new_v4(),
            url,
            content,
        }
    }
}
