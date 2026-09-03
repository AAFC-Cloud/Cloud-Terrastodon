use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_rest::RequestHeaders;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MicrosoftGraphProgress {
    pub total: Option<usize>,
    pub accumulated: usize,
    pub received_this_page: usize,
    pub time_remaining: Duration,
}

pub type MicrosoftGraphProgressHook = Box<dyn Fn(MicrosoftGraphProgress) + Send + Sync + 'static>;

enum NextLink {
    Uninitialized,
    Some(String),
    StopIteration,
}
pub struct MicrosoftGraphHelper<'a> {
    pub url: String,
    pub cache_key: Option<CacheKey>,
    pub auth_context: &'a AzureTenantAuthContext,
    pub headers: Option<RequestHeaders>,
    pub progress_hook: Option<MicrosoftGraphProgressHook>,
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
            headers: None,
            progress_hook: None,
        }
    }

    pub fn add_headers(mut self, headers: RequestHeaders) -> Self {
        self.headers = Some(match self.headers.take() {
            Some(existing) => existing.merge(headers),
            None => headers,
        });
        self
    }

    pub fn with_consistency_level_eventual(self) -> Self {
        let consistency_level = RequestHeaders::from_header_lines(["ConsistencyLevel: eventual"])
            .expect("the ConsistencyLevel header is valid")
            .expect("the ConsistencyLevel header is always present");
        self.add_headers(consistency_level)
    }

    fn get_request(&self, url: &str) -> Result<RestRequest> {
        let mut request = RestRequest::new(http::Method::GET, url)?;
        request.tenant = Some(self.auth_context.tenant_id);
        request = request.auth_context(&self.auth_context.auth_context);
        request.cache_key = self.cache_key.clone();
        request.headers = self.headers.clone();
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
        let mut total = None;
        let started_at = Instant::now();
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

            let received_this_page = response.value.len();
            total = response.count.or(total);

            // Update next link for pagination
            next_link = match response.next_link {
                Some(url) => NextLink::Some(url),
                None => NextLink::StopIteration,
            };

            // Update results
            results.append(&mut response.value);

            if let Some(progress_hook) = self.progress_hook.as_ref() {
                let time_remaining = total
                    .and_then(|total| total.checked_sub(results.len()))
                    .filter(|remaining| *remaining > 0 && !results.is_empty())
                    .map(|remaining| {
                        started_at
                            .elapsed()
                            .mul_f64(remaining as f64 / results.len() as f64)
                    })
                    .unwrap_or(Duration::ZERO);
                progress_hook(MicrosoftGraphProgress {
                    total,
                    accumulated: results.len(),
                    received_this_page,
                    time_remaining,
                });
            }
        }
        Ok(results)
    }
}

#[derive(Debug, facet::Facet)]
pub struct MicrosoftGraphResponse<T> {
    #[facet(rename = "@odata.count")]
    pub count: Option<usize>,
    #[facet(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    pub value: Vec<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_credentials::AzureTenantAuthContext;

    #[test]
    fn consistency_level_eventual_adds_the_expected_header() -> Result<()> {
        let auth_context = AzureTenantAuthContext::default();
        let helper = MicrosoftGraphHelper::new(
            "https://graph.microsoft.com/v1.0/users",
            None,
            &auth_context,
        )
        .add_headers(
            RequestHeaders::from_header_lines(["X-Test: value"])
                .expect("the test header is valid")
                .expect("the test header is always present"),
        )
        .with_consistency_level_eventual();

        let headers = helper
            .headers
            .expect("the consistency level header should be configured")
            .to_header_map()?;
        assert_eq!(
            headers
                .get("ConsistencyLevel")
                .and_then(|value| value.to_str().ok()),
            Some("eventual")
        );
        assert_eq!(
            headers.get("X-Test").and_then(|value| value.to_str().ok()),
            Some("value")
        );
        Ok(())
    }
}
