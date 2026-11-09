use crate::QueryRestResultFormat;
use crate::ResourceGraphQueryRestBody;
use crate::ResourceGraphQueryRestOptions;
use crate::ResourceGraphQueryRestScopeFilterOption;
use crate::resource_graph_rate_limit_state::ResourceGraphRateLimitState;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RawResourceGraphQueryResponse;
use cloud_terrastodon_azure_types::ResourceGraphEntryDeserializeError;
use cloud_terrastodon_azure_types::ResourceGraphQueryResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_relative_location::RelativeLocation;
use cloud_terrastodon_rest::RestHttpError;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestResponseHeaders;
use eyre::Context;
use eyre::Result;
#[cfg(debug_assertions)]
use eyre::bail;
use http::StatusCode;
use std::collections::BTreeMap;
#[cfg(debug_assertions)]
use std::collections::HashSet;
use std::future::Future;
use std::panic::Location;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use tracing::debug;
use tracing::warn;

const RESOURCE_GRAPH_BATCH_SIZE: u64 = 1_000;
const RESOURCE_GRAPH_RETRY_BUFFER: Duration = Duration::from_secs(1);
const RESOURCE_GRAPH_MAX_THROTTLE_RETRIES: usize = 3;

static RESOURCE_GRAPH_RATE_LIMIT_STATE: LazyLock<Mutex<ResourceGraphRateLimitState>> =
    LazyLock::new(|| Mutex::new(ResourceGraphRateLimitState::new()));
static RESOURCE_GRAPH_RECOVERY_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Fetch paginated [Resource Graph 2022-10-01 query results](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01).
pub struct ResourceGraphHelper {
    query: String,
    cache_behaviour: Option<CacheKey>,
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    skip: Option<(u64, String)>,
    index: usize,
    #[cfg(debug_assertions)]
    seen_skip_tokens: HashSet<String>,
}
impl ResourceGraphHelper {
    pub fn new(
        query: impl Into<String>,
        cache_behaviour: Option<CacheKey>,
        tenant_id: AzureTenantId,
        auth_source: AzureBearerAuthSource,
    ) -> Self {
        Self {
            query: query.into(),
            cache_behaviour,
            tenant_id,
            auth_source,
            skip: None,
            index: 0,
            #[cfg(debug_assertions)]
            seen_skip_tokens: Default::default(),
        }
    }

    fn get_request(&self, body: String) -> Result<RestRequest> {
        let mut request = RestRequest::new(http::Method::POST, "https://management.azure.com/providers/Microsoft.ResourceGraph/resources?api-version=2022-10-01")?.cancellation_safe()
        .authentication(AuthenticationSelection::AzureBearer {
            tenant_id: self.tenant_id,
            auth_source: self.auth_source,
        })
        .body(body);
        request.cache_key = self.cache_behaviour.clone();
        Ok(request)
    }

    #[track_caller]
    pub fn fetch<'exec, T: FromCommandOutput + Sync>(
        &'exec mut self,
        executor: &'exec CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<Option<Arc<ResourceGraphQueryResponse<Arc<T>>>>>> + 'exec {
        let caller = Location::caller();
        async move { self.fetch_from(caller, executor).await }
    }

    async fn fetch_from<T: FromCommandOutput + Sync>(
        &mut self,
        caller: &'static Location<'static>,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Option<Arc<ResourceGraphQueryResponse<Arc<T>>>>> {
        async {
            #[cfg(debug_assertions)]
            if let Some((_, token)) = &self.skip
                && !self.seen_skip_tokens.insert(token.to_owned())
            {
                bail!("Saw the same skip token twice, infinite loop detected");
            }

            // Previously tried using `az graph query` but hit issues with scopes.
            // We use the REST endpoint so we can pass authorizationScopeFilter.
            let (skip, skip_token) = match &self.skip {
                Some((skip, token)) => (*skip, Some(token.to_owned())),
                None => (0u64, None),
            };
            let body = facet_json::to_string_pretty(&ResourceGraphQueryRestBody {
                query: self.query.to_string(),
                options: ResourceGraphQueryRestOptions {
                    skip,
                    top: RESOURCE_GRAPH_BATCH_SIZE,
                    skip_token,
                    authorization_scope_filter:
                        ResourceGraphQueryRestScopeFilterOption::AtScopeAboveAndBelow,
                    result_format: QueryRestResultFormat::Table,
                },
            })
            .map_err(|error| eyre::eyre!("{error:?}"))?;
            let mut request = self.get_request(body)?;

            // Set up caching
            if let Some(CacheKey {
                ref scope,
                ref valid_for,
            }) = self.cache_behaviour
            {
                request.cache_key = Some(CacheKey {
                    scope: scope.join(self.index.to_string())?,
                    valid_for: *valid_for,
                });
            }

            debug!(
                batch_index=self.index,
                batch_size=RESOURCE_GRAPH_BATCH_SIZE,
                skip,
                ?self.tenant_id,
                ?self.cache_behaviour,
                "Fetching resource graph batch",
            );

            let results = receive_resource_graph_response(request, executor).await?;

            // Increment index for the next potential query
            self.index += 1;

            // Update skip token
            if let Some(skip_token) = &results.skip_token {
                self.skip
                    .replace((skip + results.count, skip_token.to_owned()));
            } else {
                self.skip.clone_from(&None);
            }

            // // Transform results
            // let results: QueryResponse<T> = results.try_into()?;

            eyre::Ok(Some(results))
        }
        .await
        .wrap_err(format!(
            "ResourceGraphHelper::fetch failed, called from {}",
            RelativeLocation::from(caller)
        ))
    }

    #[track_caller]
    pub fn collect_all<'exec, T: FromCommandOutput + Sync>(
        &'exec mut self,
        executor: &'exec CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<Vec<Arc<T>>>> + 'exec {
        self.collect_all_from(Location::caller(), executor)
    }

    async fn collect_all_from<T: FromCommandOutput + Sync>(
        &mut self,
        caller: &'static Location<'static>,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Vec<Arc<T>>> {
        let result: Result<Vec<Arc<T>>> = async {
            let mut all_data = Vec::new();
            while let Some(response) = self.fetch_from(caller, executor).await? {
                all_data.extend(response.data.iter().cloned());

                if self.skip.is_none() {
                    break;
                }
            }

            debug!(
                total_items=all_data.len(),
                ?self.tenant_id,
                ?self.cache_behaviour,
                "Completed fetching all resource graph data",
            );

            Ok(all_data)
        }
        .await;

        result.wrap_err(format!(
            "ResourceGraphHelper::collect_all failed, called from {}",
            RelativeLocation::from(caller)
        ))
    }
}

async fn acquire_resource_graph_recovery_guard() -> Option<MutexGuard<'static, ()>> {
    let should_serialize = {
        let state = RESOURCE_GRAPH_RATE_LIMIT_STATE.lock().await;
        state.blocked_until.is_some()
    };

    if should_serialize {
        Some(RESOURCE_GRAPH_RECOVERY_LOCK.lock().await)
    } else {
        None
    }
}

async fn wait_for_resource_graph_rate_limit_window() {
    loop {
        let delay = {
            let mut state = RESOURCE_GRAPH_RATE_LIMIT_STATE.lock().await;
            match state.blocked_until {
                Some(blocked_until) => match blocked_until.checked_duration_since(Instant::now()) {
                    Some(delay) => Some(delay),
                    None => {
                        state.blocked_until = None;
                        None
                    }
                },
                None => None,
            }
        };

        match delay {
            Some(delay) if !delay.is_zero() => {
                warn!(
                    reset_in = %humantime::format_duration(delay),
                    "Resource Graph quota exhausted, waiting before next request"
                );
                tokio::time::sleep(delay).await;
            }
            _ => return,
        }
    }
}

async fn note_resource_graph_rate_limit(status: StatusCode, headers: &RestResponseHeaders) {
    let should_block = status == StatusCode::TOO_MANY_REQUESTS
        || headers.resource_graph_quota_remaining() == Some(0);
    let Some(delay) = headers.retry_after() else {
        return;
    };

    if !should_block {
        return;
    }

    let blocked_until = Instant::now() + delay + RESOURCE_GRAPH_RETRY_BUFFER;
    let mut state = RESOURCE_GRAPH_RATE_LIMIT_STATE.lock().await;
    let should_update = match state.blocked_until {
        Some(existing) => existing < blocked_until,
        None => true,
    };
    if should_update {
        state.blocked_until = Some(blocked_until);
        warn!(
            remaining_quota = ?headers.resource_graph_quota_remaining(),
            reset_in = %humantime::format_duration(delay),
            status = status.as_u16(),
            "Updated shared Resource Graph rate limit window"
        );
    }
}

async fn receive_resource_graph_response<T: FromCommandOutput + Sync>(
    request: RestRequest,
    executor: &CloudTerrastodonExecutor,
) -> Result<Arc<ResourceGraphQueryResponse<Arc<T>>>> {
    let mut retries = 0usize;
    let request = request.failure_extra_files(resource_graph_failure_extra_files);
    loop {
        executor.cancellation().bail_if_cancelled()?;
        let _recovery_guard = executor
            .cancellation()
            .cancellable_scope(|_| async {
                let guard = acquire_resource_graph_recovery_guard().await;
                wait_for_resource_graph_rate_limit_window().await;
                eyre::Ok(guard)
            })
            .await?;
        // Table conversion is part of canonical decoding: cache typed rows once,
        // retain decode-failure artifacts, and share records across page callers.
        let outcome = if request.cache_key.is_some() {
            request.clone().receive_converted_cached::<RawResourceGraphQueryResponse, ResourceGraphQueryResponse<Arc<T>>>(
                executor, CacheOperationOwner::of::<ResourceGraphHelper>(),
            ).await
        } else {
            request.clone().receive_converted::<RawResourceGraphQueryResponse, ResourceGraphQueryResponse<Arc<T>>>(
                executor,
            ).await.map(Arc::new)
        };
        let error = match outcome {
            Ok(parsed) => return Ok(parsed),
            Err(error) => error,
        };
        let Some(response) = error.downcast_ref::<RestHttpError>() else {
            return Err(error);
        };
        note_resource_graph_rate_limit(response.status, &response.headers).await;
        let is_throttled = response.status == StatusCode::TOO_MANY_REQUESTS
            || response.headers.resource_graph_quota_remaining() == Some(0);
        if !is_throttled || retries >= RESOURCE_GRAPH_MAX_THROTTLE_RETRIES {
            return Err(error);
        }
        retries += 1;
        warn!(
            attempt = retries,
            max_attempts = RESOURCE_GRAPH_MAX_THROTTLE_RETRIES,
            "Retrying throttled Resource Graph request"
        );
    }
}

fn resource_graph_failure_extra_files(error: &eyre::Report) -> BTreeMap<PathBuf, bstr::BString> {
    let mut files = BTreeMap::new();
    for cause in error.chain() {
        let Some(error) = cause.downcast_ref::<ResourceGraphEntryDeserializeError>() else {
            continue;
        };
        files.insert(
            PathBuf::from(format!("entry.{}.json", error.index())),
            error.entry_json().into(),
        );
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::CredentialAuthenticationProvider;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_rest::SerializableRestResponse;
    use facet::Facet;
    use http::HeaderMap;
    use http::HeaderValue;
    use http::StatusCode;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    #[ignore = "requires explicitly configured test tenant and live Azure Resource Graph access"]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let query = r#"
    resourcecontainers
    | project name
    "#;
                #[derive(Facet)]
                struct Row {
                    name: String,
                }
                let auth = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth.bind_to_azure_tenant(crate::get_test_tenant_id(&cancellation).await?)?;
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(CredentialAuthenticationProvider::new(auth)),
                    cancellation,
                );
                let data = ResourceGraphHelper::new(
                    query,
                    Some(CacheKey::from_relative_path(PathBuf::from_iter([
                        "az",
                        "resource_graph",
                        "resource-container-names",
                    ]))?),
                    auth_context.tenant_id,
                    auth_context.auth_source()?,
                )
                .collect_all::<Row>(&executor)
                .await?;
                assert!(data.len() > 10);
                assert!(data.iter().all(|row| !row.name.is_empty()));
                Ok(())
            })
            .await
    }

    #[test]
    fn reads_resource_graph_quota_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ms-user-quota-remaining", HeaderValue::from_static("0"));
        headers.insert(
            "x-ms-user-quota-resets-after",
            HeaderValue::from_static("00:00:05"),
        );

        let response = SerializableRestResponse::new(
            StatusCode::TOO_MANY_REQUESTS,
            &headers,
            "{}".to_string(),
        );
        assert_eq!(response.headers.resource_graph_quota_remaining(), Some(0));
        assert_eq!(response.headers.retry_after(), Some(Duration::from_secs(5)));
    }

    #[test]
    fn writes_only_failed_resource_graph_entry_extra_file() -> Result<()> {
        let body = r#"{
            "count": 2,
            "data": {
                "columns": [
                    {"name": "name", "type": "string"},
                    {"name": "properties", "type": "dynamic"}
                ],
                "rows": [
                    ["first", {"description": 1}],
                    ["second", {"description": "two"}]
                ]
            },
            "resultTruncated": "false",
            "totalRecords": 2
        }"#;
        #[derive(Debug, Facet)]
        struct Row {
            name: String,
            properties: TestProperties,
        }

        #[derive(Debug, Facet)]
        struct TestProperties {
            description: u64,
        }

        let raw: cloud_terrastodon_azure_types::RawResourceGraphQueryResponse =
            facet_json::from_str(body)?;
        let result: Result<ResourceGraphQueryResponse<Row>> = raw.try_into();
        let error = result.expect_err("second entry should fail");

        let files = resource_graph_failure_extra_files(&error);

        assert_eq!(files.len(), 1);
        assert_eq!(
            files
                .get(&PathBuf::from("entry.1.json"))
                .map(|value| value.as_ref()),
            Some(
                &br#"{
  "name": "second",
  "properties": {
    "description": "two"
  }
}"#[..],
            )
        );
        Ok(())
    }
}
