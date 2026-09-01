use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;

enum NextLink {
    Uninitialized,
    Some(String),
    StopIteration,
}
pub struct MicrosoftGraphHelper<'a> {
    url: String,
    cache_key: Option<CacheKey>,
    auth_context: &'a AzureTenantAuthContext,
}
impl<'a> MicrosoftGraphHelper<'a> {
    pub fn new(
        url: impl ToString,
        cache_key: Option<CacheKey>,
        auth_context: &'a AzureTenantAuthContext,
    ) -> Self {
        MicrosoftGraphHelper {
            url: url.to_string(),
            cache_key,
            auth_context,
        }
    }

    fn get_request(&self, url: &str) -> Result<RestRequest> {
        let mut request = RestRequest::new(http::Method::GET, url)?;
        request.tenant = Some(self.auth_context.tenant_id);
        request = request.auth_context(&self.auth_context.auth_context);
        request.cache_key = self.cache_key.clone();
        Ok(request)
    }

    pub async fn fetch_one<T: FromCommandOutput>(self) -> Result<T> {
        self.get_request(&self.url)?.receive::<T>().await
    }

    /// This doesn't handle 'singleton' responses like https://graph.microsoft.com/v1.0/me
    pub async fn fetch_all<T: FromCommandOutput>(&self) -> Result<Vec<T>> {
        match self.fetch_all_once().await {
            Ok(results) => Ok(results),
            Err((error, true)) => {
                let Some(cache_key) = self.cache_key.as_ref() else {
                    return Err(error);
                };

                tracing::debug!(
                    %error,
                    path = %cache_key.path.display(),
                    "Invalidating paginated Microsoft Graph cache before retry"
                );
                cache_key.invalidate().await?;
                self.fetch_all_once().await.map_err(|(retry_error, _)| {
                    retry_error
                        .wrap_err("cached retry after invalidating Graph continuation failed")
                })
            }
            Err((error, _)) => Err(error),
        }
    }

    async fn fetch_all_once<T: FromCommandOutput>(
        &self,
    ) -> std::result::Result<Vec<T>, (eyre::Report, bool)> {
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

            let mut request = self
                .get_request(url)
                .map_err(|error| (error, request_index > 0))?;
            if let Some(ref cache_key) = self.cache_key {
                request.cache_key = Some(CacheKey {
                    path: cache_key.path.join(request_index.to_string()),
                    valid_for: cache_key.valid_for,
                });
            }

            let mut response = request
                .receive::<MicrosoftGraphResponse<T>>()
                .await
                .map_err(|error| (error, request_index > 0))?;
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

#[derive(Debug, facet::Facet)]
pub struct MicrosoftGraphResponse<T> {
    #[facet(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    pub value: Vec<T>,
}
