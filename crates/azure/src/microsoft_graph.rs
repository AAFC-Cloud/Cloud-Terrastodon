use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use serde::Deserialize;

enum NextLink {
    Uninitialized,
    Some(String),
    StopIteration,
}
pub struct MicrosoftGraphHelper {
    url: String,
    cache_key: Option<CacheKey>,
    tenant_id: AzureTenantId,
}
impl MicrosoftGraphHelper {
    pub fn new(tenant_id: AzureTenantId, url: impl ToString, cache_key: Option<CacheKey>) -> Self {
        MicrosoftGraphHelper {
            url: url.to_string(),
            cache_key,
            tenant_id,
        }
    }

    fn get_request(&self, url: &str) -> Result<RestRequest> {
        let mut request = RestRequest::new(http::Method::GET, url)?;
        request.tenant = Some(self.tenant_id);
        request.cache_key = self.cache_key.clone();
        Ok(request)
    }

    pub async fn fetch_one<T: FromCommandOutput>(self) -> Result<T> {
        self.get_request(&self.url)?.receive::<T>().await
    }

    /// This doesn't handle 'singleton' responses like https://graph.microsoft.com/v1.0/me
    pub async fn fetch_all<T: FromCommandOutput>(&self) -> Result<Vec<T>> {
        let mut results = Vec::new();
        let mut next_link = NextLink::Uninitialized;
        let mut request_index = 0;
        loop {
            // Determine URL, considering pagination
            let url = match &next_link {
                NextLink::Uninitialized => &self.url,
                NextLink::Some(x) => x,
                NextLink::StopIteration => break,
            };

            let mut request = self.get_request(url)?;
            if let Some(ref cache_key) = self.cache_key {
                request.cache_key = Some(CacheKey {
                    path: cache_key.path.join(request_index.to_string()),
                    valid_for: cache_key.valid_for,
                });
            }

            let mut response = request.receive::<MicrosoftGraphResponse<T>>().await?;
            request_index += 1;

            // Update next link for pagination
            next_link = match response.next_link {
                Some(url) => NextLink::Some(url),
                None => NextLink::StopIteration,
            };

            // Update results
            results.append(&mut response.value);
        }
        Ok(results)
    }
}

#[derive(Debug, Deserialize)]
pub struct MicrosoftGraphResponse<T> {
    // #[serde(rename = "@odata.context")]
    // context: String,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    pub value: Vec<T>,
}
