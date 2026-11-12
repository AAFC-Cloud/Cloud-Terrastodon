use crate::AzureManagementGroupListRequest;
use crate::cost_management_response::CostManagementResponse;
use crate::fetch_root_management_group;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::CostManagementQueryDefinition;
use cloud_terrastodon_azure_types::CostManagementQueryResult;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestResponseBody;
use cloud_terrastodon_rest::SerializableRestResponse;
use eyre::Context;
use facet::Facet;
use http::StatusCode;
use std::borrow::Cow;
use std::time::Duration;
use tracing::warn;

const COST_MANAGEMENT_RETRY_BUFFER: Duration = Duration::from_secs(1);
const COST_MANAGEMENT_MAX_THROTTLE_RETRIES: usize = 3;

/// Query costs under the selected tenant's root management group.
/// [Cost Management query](https://learn.microsoft.com/en-us/rest/api/cost-management/query/usage?view=rest-cost-management-2021-10-01).
#[derive(Debug, Facet)]
pub struct AzureCostQueryRequest<'a> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub query: Cow<'a, CostManagementQueryDefinition>,
}

impl CacheInvalidationKeys for AzureCostQueryRequest<'_> {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        // The cost POST is not reused. Root-group lookup is the only cached
        // dependency and owns the scope that --clean must refresh.
        AzureManagementGroupListRequest {
            authentication: self.authentication,
        }
        .cache_keys_to_invalidate()
    }
}

impl ExecutableRequest for AzureCostQueryRequest<'_> {
    type Output = CostManagementQueryResult;
    type Error = eyre::Report;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let root = fetch_root_management_group(&self.authentication, &executor).await?;
            let url = format!(
                "https://management.azure.com/providers/Microsoft.Management/managementGroups/{}/providers/Microsoft.CostManagement/query?api-version=2021-10-01",
                root.tenant_id
            );
            let request = RestRequest::new(http::Method::POST, url.as_str())?
                .cancellation_safe()
                .authentication(self.authentication.into())
                .body(
                    facet_json::to_string_pretty(self.query.as_ref())
                        .map_err(|error| eyre::eyre!("{error:?}"))?,
                );
            receive_cost_management_response(request, &executor).await
        })
    }
}
async fn receive_cost_management_response(
    request: RestRequest,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<CostManagementQueryResult> {
    let cancellation = executor.cancellation();
    let mut retries = 0usize;

    loop {
        cancellation.bail_if_cancelled()?;
        let cache_key = request.cache_key.clone();
        let outcome = request
            .clone()
            .receive_raw_with_decoder(
                |response| {
                    if response.ok {
                        let parsed = facet_json::from_str::<CostManagementQueryResult>(
                            response.into_json_body()?.as_str(),
                        )
                        .map_err(|error| eyre::eyre!("{error:?}"))
                        .wrap_err_with(|| {
                            format!(
                                "Deserializing REST response into {}",
                                std::any::type_name::<CostManagementQueryResult>()
                            )
                        })?;
                        Ok(CostManagementResponse::Parsed(parsed))
                    } else {
                        Ok(CostManagementResponse::Raw(response))
                    }
                },
                executor,
            )
            .await?;

        let response = match outcome {
            CostManagementResponse::Parsed(parsed) => return Ok(parsed),
            CostManagementResponse::Raw(response) => response,
        };

        if is_cost_management_throttled(&response) && retries < COST_MANAGEMENT_MAX_THROTTLE_RETRIES
        {
            retries += 1;
            if let Some(cache_key) = cache_key {
                executor
                    .cache()?
                    .invalidate(&cache_key.scope, executor.cancellation())
                    .await?;
            }
            let delay = response
                .headers
                .retry_after()
                .unwrap_or(COST_MANAGEMENT_RETRY_BUFFER)
                + COST_MANAGEMENT_RETRY_BUFFER;
            warn!(
                attempt = retries,
                max_attempts = COST_MANAGEMENT_MAX_THROTTLE_RETRIES,
                reset_in = %humantime::format_duration(delay),
                "Retrying throttled Cost Management request"
            );
            cancellation
                .cancellable_scope(|_| async {
                    tokio::time::sleep(delay).await;
                    eyre::Ok(())
                })
                .await?;
            continue;
        }

        eyre::bail!(
            "REST call failed with status {}: {}{}",
            response.status.as_u16(),
            response.reason_phrase.as_deref().unwrap_or("Unknown error"),
            format_rest_error_body(&response.body)
        );
    }
}

fn is_cost_management_throttled(response: &SerializableRestResponse) -> bool {
    response.status == StatusCode::TOO_MANY_REQUESTS || response.headers.retry_after().is_some()
}

fn format_rest_error_body(body: &RestResponseBody) -> String {
    match body {
        RestResponseBody::Json(value) => format!("\nBody: {}", value.as_str()),
        RestResponseBody::Text(text) if text.trim().is_empty() => String::new(),
        RestResponseBody::Text(text) => format!("\nBody: {}", text.trim()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;
    use http::HeaderValue;
    use http::StatusCode;

    #[test]
    fn identifies_cost_management_throttle_responses() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-ms-ratelimit-microsoft.costmanagement-clienttype-retry-after",
            HeaderValue::from_static("2"),
        );

        let throttled_by_status = SerializableRestResponse::new(
            StatusCode::TOO_MANY_REQUESTS,
            &HeaderMap::new(),
            "{}".to_string(),
        );
        let throttled_by_header = SerializableRestResponse::new(
            StatusCode::SERVICE_UNAVAILABLE,
            &headers,
            "{}".to_string(),
        );
        let not_throttled = SerializableRestResponse::new(
            StatusCode::BAD_REQUEST,
            &HeaderMap::new(),
            "{}".to_string(),
        );

        assert!(is_cost_management_throttled(&throttled_by_status));
        assert!(is_cost_management_throttled(&throttled_by_header));
        assert!(!is_cost_management_throttled(&not_throttled));
    }
}

cloud_terrastodon_registry::register_thing!(AzureCostQueryRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureCostQueryRequest<'static> => CostManagementQueryResult, effects = [Read]);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureCostQueryRequest<'static>,
    name = "cost-query",
    cached
);
