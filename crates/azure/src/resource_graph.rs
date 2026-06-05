use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ResourceGraphQueryResponse;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_relative_location::RelativeLocation;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestResponseBody;
use cloud_terrastodon_rest::SerializableRestResponse;
use eyre::Context;
use eyre::Result;
#[cfg(debug_assertions)]
use eyre::bail;
use serde::Deserialize;
use serde::Serialize;
#[cfg(debug_assertions)]
use std::collections::HashSet;
use std::future::Future;
use std::panic::Location;
use std::path::PathBuf;
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

#[derive(Debug, Clone, Copy)]
struct ResourceGraphRateLimitState {
    blocked_until: Option<Instant>,
}

impl ResourceGraphRateLimitState {
    const fn new() -> Self {
        Self {
            blocked_until: None,
        }
    }
}

pub struct ResourceGraphHelper {
    query: String,
    cache_behaviour: Option<CacheKey>,
    tenant_id: AzureTenantId,
    skip: Option<(u64, String)>,
    index: usize,
    #[cfg(debug_assertions)]
    seen_skip_tokens: HashSet<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceGraphQueryRestOptions {
    #[serde(rename = "$skip")]
    skip: u64,
    #[serde(rename = "$top")]
    top: u64,
    #[serde(rename = "$skipToken")]
    skip_token: Option<String>,
    #[serde(rename = "authorizationScopeFilter")]
    authorization_scope_filter: ResourceGraphQueryRestScopeFilterOption,
    #[serde(rename = "resultFormat")]
    result_format: QueryRestResultFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceGraphQueryRestScopeFilterOption {
    AtScopeAboveAndBelow,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QueryRestResultFormat {
    #[serde(rename = "table")]
    Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceGraphQueryRestBody {
    query: String,
    options: ResourceGraphQueryRestOptions,
}

impl ResourceGraphHelper {
    pub fn new(
        tenant_id: AzureTenantId,
        query: impl Into<String>,
        cache_behaviour: Option<CacheKey>,
    ) -> Self {
        Self {
            query: query.into(),
            cache_behaviour,
            tenant_id,
            skip: None,
            index: 0,
            #[cfg(debug_assertions)]
            seen_skip_tokens: Default::default(),
        }
    }

    fn get_request(&self, body: String) -> Result<RestRequest> {
        let mut request = RestRequest::new(
            http::Method::POST,
            "https://management.azure.com/providers/Microsoft.ResourceGraph/resources?api-version=2022-10-01",
        )?
        .tenant(self.tenant_id)
        .body(body);
        request.cache_key = self.cache_behaviour.clone().or_else(|| {
            Some(CacheKey::new(PathBuf::from_iter([
                "az",
                "resource_graph",
                "query",
            ])))
        });
        Ok(request)
    }

    #[track_caller]
    pub fn fetch<T: FromCommandOutput>(
        &mut self,
    ) -> impl Future<Output = Result<Option<ResourceGraphQueryResponse<T>>>> + '_ {
        self.fetch_from(Location::caller())
    }

    async fn fetch_from<T: FromCommandOutput>(
        &mut self,
        caller: &'static Location<'static>,
    ) -> Result<Option<ResourceGraphQueryResponse<T>>> {
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
            let body = serde_json::to_string_pretty(&ResourceGraphQueryRestBody {
                query: self.query.to_string(),
                options: ResourceGraphQueryRestOptions {
                    skip,
                    top: RESOURCE_GRAPH_BATCH_SIZE,
                    skip_token,
                    authorization_scope_filter:
                        ResourceGraphQueryRestScopeFilterOption::AtScopeAboveAndBelow,
                    result_format: QueryRestResultFormat::Table,
                },
            })?;
            let mut request = self.get_request(body)?;

            // Set up caching
            if let Some(CacheKey {
                ref path,
                ref valid_for,
            }) = self.cache_behaviour
            {
                request.cache_key = Some(CacheKey {
                    path: path.join(self.index.to_string()),
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

            let results = receive_resource_graph_response(request).await?;

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
    pub fn collect_all<T: FromCommandOutput>(
        &mut self,
    ) -> impl Future<Output = Result<Vec<T>>> + '_ {
        self.collect_all_from(Location::caller())
    }

    async fn collect_all_from<T: FromCommandOutput>(
        &mut self,
        caller: &'static Location<'static>,
    ) -> Result<Vec<T>> {
        let result: Result<Vec<T>> = async {
            let mut all_data = Vec::new();
            while let Some(response) = self.fetch_from(caller).await? {
                all_data.extend(response.data);

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

async fn note_resource_graph_rate_limit(response: &SerializableRestResponse) {
    let should_block = response.status == http::StatusCode::TOO_MANY_REQUESTS.as_u16()
        || resource_graph_quota_remaining(response) == Some(0);
    let Some(delay) = resource_graph_retry_delay(response) else {
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
            remaining_quota = ?resource_graph_quota_remaining(response),
            reset_in = %humantime::format_duration(delay),
            status = response.status,
            "Updated shared Resource Graph rate limit window"
        );
    }
}

async fn receive_resource_graph_response<T: FromCommandOutput>(
    request: RestRequest,
) -> Result<ResourceGraphQueryResponse<T>> {
    let mut retries = 0usize;

    loop {
        let _recovery_guard = acquire_resource_graph_recovery_guard().await;
        wait_for_resource_graph_rate_limit_window().await;

        let cache_key = request.cache_key.clone();
        let response = request.clone().receive_raw().await?;
        note_resource_graph_rate_limit(&response).await;

        if response.ok {
            let parsed =
                serde_json::from_value(response.into_json_body()?).wrap_err_with(|| {
                    format!(
                        "Deserializing REST response into {}",
                        std::any::type_name::<ResourceGraphQueryResponse<T>>()
                    )
                })?;
            return Ok(parsed);
        }

        let is_throttled = response.status == http::StatusCode::TOO_MANY_REQUESTS.as_u16()
            || resource_graph_quota_remaining(&response) == Some(0);
        if is_throttled && retries < RESOURCE_GRAPH_MAX_THROTTLE_RETRIES {
            retries += 1;
            if let Some(cache_key) = cache_key {
                cache_key.invalidate().await?;
            }
            warn!(
                attempt = retries,
                max_attempts = RESOURCE_GRAPH_MAX_THROTTLE_RETRIES,
                reset_in = ?resource_graph_retry_delay(&response)
                    .map(|delay| humantime::format_duration(delay).to_string()),
                "Retrying throttled Resource Graph request"
            );
            continue;
        }

        eyre::bail!(
            "REST call failed with status {}: {}{}",
            response.status,
            response.reason_phrase.as_deref().unwrap_or("Unknown error"),
            format_rest_error_body(&response.body)
        );
    }
}

fn resource_graph_quota_remaining(response: &SerializableRestResponse) -> Option<u64> {
    response
        .header("x-ms-user-quota-remaining")
        .and_then(|value| value.parse::<u64>().ok())
}

fn resource_graph_retry_delay(response: &SerializableRestResponse) -> Option<Duration> {
    response
        .header("x-ms-user-quota-resets-after")
        .and_then(parse_hms_duration)
        .or_else(|| response.header("retry-after").and_then(parse_retry_after))
}

fn parse_hms_duration(value: &str) -> Option<Duration> {
    let mut parts = value.split(':');
    let hours = parts.next()?.parse::<u64>().ok()?;
    let minutes = parts.next()?.parse::<u64>().ok()?;
    let seconds = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Duration::from_secs(
        hours * 60 * 60 + minutes * 60 + seconds,
    ))
}

fn parse_retry_after(value: &str) -> Option<Duration> {
    value.parse::<u64>().ok().map(Duration::from_secs)
}

fn format_rest_error_body(body: &RestResponseBody) -> String {
    match body {
        RestResponseBody::Json(value) => serde_json::to_string(value)
            .map(|json| format!("\nBody: {json}"))
            .unwrap_or_default(),
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
    use serde::Deserialize;
    use std::path::PathBuf;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let query = r#"
resourcecontainers
| project name
"#;
        #[derive(Deserialize)]
        struct Row {
            name: String,
        }
        let data = ResourceGraphHelper::new(
            crate::get_test_tenant_id().await?,
            query,
            Some(CacheKey::new(PathBuf::from_iter([
                "az",
                "resource_graph",
                "resource-container-names",
            ]))),
        )
        .collect_all::<Row>()
        .await?;
        assert!(data.len() > 10);
        assert!(data.iter().all(|row| !row.name.is_empty()));
        Ok(())
    }

    #[test]
    fn parses_quota_reset_duration() {
        assert_eq!(parse_hms_duration("00:00:05"), Some(Duration::from_secs(5)));
        assert_eq!(
            parse_hms_duration("01:02:03"),
            Some(Duration::from_secs(3_723))
        );
        assert_eq!(parse_hms_duration("5"), None);
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
        assert_eq!(resource_graph_quota_remaining(&response), Some(0));
        assert_eq!(
            resource_graph_retry_delay(&response),
            Some(Duration::from_secs(5))
        );
    }
}
