use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphBatchResponse;
use crate::microsoft_graph_batch_request_entry_wire::MicrosoftGraphBatchRequestEntryWire;
use crate::microsoft_graph_batch_request_wire::MicrosoftGraphBatchRequestWire;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::sync::Arc;

const MAX_BATCH_REQUESTS: usize = 20;

/// Send up to twenty operations per [Microsoft Graph JSON batch](https://learn.microsoft.com/graph/json-batching).
pub struct MicrosoftGraphBatchRequest<REQ> {
    /// Allow interruption only when the caller guarantees every entry is read-only.
    /// False keeps generic batches running through completion.
    pub cancellation_safe: bool,

    /// The requests to be made in the batch
    pub requests: Vec<MicrosoftGraphBatchRequestEntry<REQ>>,
    /// The IDs of the requests, in the order the requests were added
    pub ids: Vec<String>,
    /// The key to use for caching the batch request
    pub cache_key: Option<CacheKey>,
    pub authentication: AzureBearerAuthenticationSelection,
}
impl<REQ> MicrosoftGraphBatchRequest<REQ> {
    pub fn new(authentication: AzureBearerAuthenticationSelection) -> Self {
        MicrosoftGraphBatchRequest {
            cancellation_safe: false,
            requests: Vec::new(),
            ids: Vec::new(),
            cache_key: None,
            authentication,
        }
    }
    /// Allow cancellation to drop a batch whose entries the caller guarantees are read-only.
    pub fn cancellation_safe(mut self) -> Self {
        self.cancellation_safe = true;
        self
    }

    pub fn add(&mut self, entry: impl Into<MicrosoftGraphBatchRequestEntry<REQ>>) {
        let entry = entry.into();
        self.ids.push(entry.id.clone());
        self.requests.push(entry);
    }
    pub fn add_all<T: Into<MicrosoftGraphBatchRequestEntry<REQ>>>(
        &mut self,
        entries: impl IntoIterator<Item = T>,
    ) {
        for entry in entries {
            self.add(entry);
        }
    }
    pub fn cache(&mut self, cache_key: CacheKey) -> &mut Self {
        self.cache_key = Some(cache_key);
        self
    }
    pub fn use_cache(&mut self, cache_key: Option<CacheKey>) -> &mut Self {
        self.cache_key = cache_key;
        self
    }
    pub async fn send<RESP: FromCommandOutput + Sync>(
        self,
        executor: &CloudTerrastodonExecutor,
    ) -> eyre::Result<MicrosoftGraphBatchResponse<RESP>>
    where
        REQ: Facet<'static>,
    {
        let Self {
            cancellation_safe,
            requests,
            ids,
            cache_key,
            authentication,
        } = self;
        if requests.is_empty() {
            return Ok(MicrosoftGraphBatchResponse {
                responses: Vec::new(),
            });
        }

        let use_base_cache_key = requests.len() <= MAX_BATCH_REQUESTS;
        let mut responses = Vec::with_capacity(requests.len());
        let mut requests = requests.into_iter();
        let mut ids = ids.into_iter();
        let mut batch_index = 0;
        loop {
            executor.cancellation().bail_if_cancelled()?;
            let mut batch_requests = Vec::with_capacity(MAX_BATCH_REQUESTS);
            let mut batch_ids = Vec::with_capacity(MAX_BATCH_REQUESTS);
            for _ in 0..MAX_BATCH_REQUESTS {
                let Some(request) = requests.next() else {
                    break;
                };
                let Some(id) = ids.next() else {
                    return Err(eyre::eyre!(
                        "Microsoft Graph batch request IDs are out of sync with requests"
                    ));
                };
                batch_ids.push(id);
                batch_requests.push(request);
            }
            if batch_requests.is_empty() {
                if ids.next().is_some() {
                    return Err(eyre::eyre!(
                        "Microsoft Graph batch request IDs are out of sync with requests"
                    ));
                }
                break;
            }

            let request_cache_key = cache_key
                .as_ref()
                .map(|cache_key| {
                    if use_base_cache_key {
                        Ok(cache_key.clone())
                    } else {
                        cache_key.join(batch_index.to_string())
                    }
                })
                .transpose()?;
            let body = MicrosoftGraphBatchRequestWire {
                requests: batch_requests
                    .into_iter()
                    .map(|request| MicrosoftGraphBatchRequestEntryWire {
                        id: request.id,
                        method: request.method.as_str().to_string(),
                        url: request.url,
                        headers: request.headers,
                        body: request.body,
                    })
                    .collect(),
            };
            let mut request =
                RestRequest::new(Method::POST, "https://graph.microsoft.com/v1.0/$batch")?
                    .authentication(authentication.into())
                    .body(
                        facet_json::to_string_pretty(&body)
                            .map_err(|error| eyre::eyre!("{error:?}"))?,
                    );
            if cancellation_safe {
                request = request.cancellation_safe();
            }
            request.cache_key = request_cache_key;
            let response = if cancellation_safe && request.cache_key.is_some() {
                request
                    .receive_with_validator_cached::<MicrosoftGraphBatchResponse<RESP>, _>(
                        |response| response.validate_request_ids(&batch_ids),
                        executor,
                        CacheOperationOwner::of::<MicrosoftGraphBatchRequest<()>>(),
                    )
                    .await?
            } else {
                request
                    .receive_with_validator(
                        |response: MicrosoftGraphBatchResponse<RESP>| {
                            response.validate_request_ids(&batch_ids)?;
                            Ok(response)
                        },
                        executor,
                    )
                    .await
                    .map(Arc::new)?
            };
            // Reorder each response chunk to match the order of its requests.
            let mut batch_responses = response.responses.clone();
            batch_responses.sort_by_key(|response| {
                batch_ids
                    .iter()
                    .position(|id| id == &response.id)
                    .unwrap_or(usize::MAX)
            });
            responses.extend(batch_responses);
            batch_index += 1;
        }

        Ok(MicrosoftGraphBatchResponse { responses })
    }
}
