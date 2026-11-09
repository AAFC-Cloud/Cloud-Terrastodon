use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::MicrosoftGraphResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::CollectionProgress;
use cloud_terrastodon_rest::RequestHeaders;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
/// Fetch a resource or all pages of a [Microsoft Graph collection](https://learn.microsoft.com/graph/paging).
pub struct MicrosoftGraphHelper {
    pub url: String,
    pub cache_key: Option<CacheKey>,
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
    pub headers: Option<RequestHeaders>,
}
impl MicrosoftGraphHelper {
    pub fn new(
        url: impl ToString,
        cache_key: Option<CacheKey>,
        tenant_id: AzureTenantId,
        auth_source: AzureBearerAuthSource,
    ) -> Self {
        MicrosoftGraphHelper {
            url: url.to_string(),
            cache_key,
            tenant_id,
            auth_source,
            headers: None,
        }
    }

    pub fn add_headers(mut self, headers: RequestHeaders) -> Self {
        self.headers = Some(match self.headers.take() {
            Some(existing) => existing.merge(headers),
            None => headers,
        });
        self
    }

    pub fn with_consistency_level_eventual(self) -> Result<Self> {
        let consistency_level = RequestHeaders::from_header("ConsistencyLevel", "eventual")?;
        Ok(self.add_headers(consistency_level))
    }

    fn get_request(&self, url: &str) -> Result<RestRequest> {
        let mut request = RestRequest::new(http::Method::GET, url)?.authentication(
            AuthenticationSelection::AzureBearer {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            },
        );
        request.cache_key = self.cache_key.clone();
        request.headers = self.headers.clone();
        Ok(request)
    }

    /// Share the decoded resource with the cache and other callers.
    pub async fn fetch_one<T: FromCommandOutput + Sync>(
        self,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Arc<T>> {
        let request = self.get_request(&self.url)?;
        if request.cache_key.is_some() {
            request
                .receive_cached::<T>(executor, CacheOperationOwner::of::<MicrosoftGraphHelper>())
                .await
        } else {
            request.receive::<T>(executor).await.map(Arc::new)
        }
    }

    /// Share decoded items across callers while assembling each paginated inventory.
    /// This doesn't handle 'singleton' responses like https://graph.microsoft.com/v1.0/me
    pub async fn fetch_all<T: FromCommandOutput + Sync>(
        &self,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Vec<Arc<T>>> {
        match self.fetch_all_once(executor).await {
            Ok(results) => Ok(results),
            Err((error, true)) => {
                let Some(cache_key) = self.cache_key.as_ref() else {
                    return Err(error);
                };

                tracing::debug!(
                    %error,
                    scope = ?cache_key.scope,
                    "Invalidating paginated Microsoft Graph cache before retry"
                );
                // Invalidation also advances any active aggregate's generation.
                // A successful retry is usable by this caller but is not published
                // under that obsolete generation. The next invocation can assemble
                // a reusable aggregate from the fresh pages without another fetch.
                executor.cancellation().bail_if_cancelled()?;
                executor
                    .cache()?
                    .invalidate(&cache_key.scope, executor.cancellation())
                    .await?;
                self.fetch_all_once(executor)
                    .await
                    .map_err(|(retry_error, _)| {
                        retry_error
                            .wrap_err("cached retry after invalidating Graph continuation failed")
                    })
            }
            Err((error, _)) => Err(error),
        }
    }

    async fn fetch_all_once<T: FromCommandOutput + Sync>(
        &self,
        executor: &CloudTerrastodonExecutor,
    ) -> std::result::Result<Vec<Arc<T>>, (eyre::Report, bool)> {
        let mut results = Vec::new();
        let mut next_link = Some(self.url.clone());
        let mut request_index = 0;
        let mut total = None;
        let started_at = Instant::now();
        loop {
            executor
                .cancellation()
                .bail_if_cancelled()
                .map_err(|error| (eyre::Report::from(error), false))?;
            let Some(url) = next_link.take() else {
                break;
            };

            let mut request = self
                .get_request(&url)
                .map_err(|error| (error, request_index > 0))?;
            if let Some(ref cache_key) = self.cache_key {
                request.cache_key = Some(
                    cache_key
                        .join(request_index.to_string())
                        .map_err(|error| (error, false))?,
                );
            }

            let response = if request.cache_key.is_some() {
                request
                    .receive_cached::<MicrosoftGraphResponse<Arc<T>>>(
                        executor,
                        CacheOperationOwner::of::<MicrosoftGraphHelper>(),
                    )
                    .await
            } else {
                request
                    .receive::<MicrosoftGraphResponse<Arc<T>>>(executor)
                    .await
                    .map(Arc::new)
            }
            .map_err(|error| (error, request_index > 0))?;
            request_index += 1;

            let received_this_page = response.value.len();
            total = response.count.or(total);

            // Update next link for pagination
            next_link = response.next_link.clone();

            // Update results
            results.extend(response.value.iter().cloned());

            let time_remaining = total
                .and_then(|total| total.checked_sub(results.len()))
                .filter(|remaining| *remaining > 0 && !results.is_empty())
                .map(|remaining| {
                    started_at
                        .elapsed()
                        .mul_f64(remaining as f64 / results.len() as f64)
                })
                .unwrap_or(Duration::ZERO);
            executor.report_progress(CollectionProgress {
                total,
                accumulated: results.len(),
                received_this_page,
                time_remaining,
            });
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_types::uuid::Uuid;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn consistency_level_eventual_adds_the_expected_header() -> Result<()> {
        CancellationToken::new().with_scope(|_cancellation| {
            let helper = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/v1.0/users",
                None,
                AzureTenantId::new(Uuid::nil()),
                AzureBearerAuthSource::AzureCli,
            )
            .add_headers(
                RequestHeaders::from_header_lines(["X-Test: value"])
                    .expect("the test header is valid")
                    .expect("the test header is always present"),
            )
            .with_consistency_level_eventual()
            .expect("valid consistency header");

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
        })
    }
}
