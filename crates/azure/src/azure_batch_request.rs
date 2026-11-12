use crate::AzureBatchRequestEntry;
use crate::BatchResponse;
use crate::batch_request_upstream::BatchRequestUpstream;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_relative_location::RelativeLocation;
use cloud_terrastodon_rest::RestRequest;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use http::Method;
use std::future::Future;
use std::panic::Location;
use tracing::debug;
#[derive(Debug, Clone)]
pub struct AzureBatchRequest<T> {
    /// Allow interruption only when the caller guarantees every entry is read-only.
    /// False keeps generic batches running through completion.
    pub cancellation_safe: bool,

    pub requests: Vec<AzureBatchRequestEntry<T>>,
    pub authentication: AzureBearerAuthenticationSelection,
}
impl<T> AzureBatchRequest<T> {
    pub fn new(authentication: AzureBearerAuthenticationSelection) -> Self {
        Self {
            cancellation_safe: false,
            requests: Vec::new(),
            authentication,
        }
    }
    /// Allow cancellation to drop a batch whose entries the caller guarantees are read-only.
    pub fn cancellation_safe(mut self) -> Self {
        self.cancellation_safe = true;
        self
    }
}

#[track_caller]
pub fn invoke_batch_request<'a, REQ, RESP>(
    request: &'a AzureBatchRequest<REQ>,
    executor: &'a CloudTerrastodonExecutor,
) -> impl Future<Output = Result<BatchResponse<RESP>>> + 'a
where
    REQ: Facet<'a>,
    RESP: FromCommandOutput,
{
    invoke_batch_request_from(request, Location::caller(), executor)
}

async fn invoke_batch_request_from<'a, REQ, RESP>(
    request: &'a AzureBatchRequest<REQ>,
    caller: &'static Location<'static>,
    executor: &CloudTerrastodonExecutor,
) -> Result<BatchResponse<RESP>>
where
    REQ: Facet<'a>,
    RESP: FromCommandOutput,
{
    let result: Result<BatchResponse<RESP>> = async {
        if request.requests.is_empty() {
            return Ok(BatchResponse {
                responses: Vec::new(),
            });
        }

        let url = "https://management.azure.com/batch?api-version=2020-06-01";

        // create the results holder
        let mut rtn = BatchResponse {
            responses: Vec::new(),
        };

        // batch the batch requests into size=20 chunks
        let chunks = request.requests.chunks(20);
        let num_chunks = chunks.len();
        for (i, chunk) in chunks.enumerate() {
            executor.cancellation().bail_if_cancelled()?;
            let rest_request = RestRequest::new(Method::POST, url)?
                .authentication(request.authentication.into())
                .body(
                    facet_json::to_string_pretty(&BatchRequestUpstream { requests: chunk })
                        .map_err(|error| eyre::eyre!("{error:?}"))?,
                );

            let rest_request = if request.cancellation_safe {
                rest_request.cancellation_safe()
            } else {
                rest_request
            };

            debug!(
                batch_index = i,
                total_batches = num_chunks,
                "Performing batch request"
            );
            let response = rest_request
                .receive_with_validator(
                    |response: BatchResponse<RESP>| {
                        eyre::ensure!(
                            chunk.len() == response.responses.len(),
                            "ARM batch response count differs from its requests"
                        );
                        for (request, response) in chunk.iter().zip(&response.responses) {
                            eyre::ensure!(
                                request.name == response.name,
                                "ARM batch response order differs from its requests"
                            );
                        }
                        let failures = response
                            .responses
                            .iter()
                            .filter(|resp| resp.http_status_code != 200)
                            .count();
                        if failures > 0 {
                            bail!("There were {} requests with non-200 status codes", failures);
                        }
                        Ok(response)
                    },
                    executor,
                )
                .await?;
            rtn.responses.extend(response.responses);
        }
        Ok(rtn)
    }
    .await;

    result.wrap_err(format!(
        "invoke_batch_request failed, called from {}",
        RelativeLocation::from(caller)
    ))
}
impl<T> AzureBatchRequest<T> {
    #[track_caller]
    pub fn invoke<'exec, RESP>(
        &'exec self,
        executor: &'exec CloudTerrastodonExecutor,
    ) -> impl Future<Output = eyre::Result<BatchResponse<RESP>>> + 'exec
    where
        T: Facet<'exec>,
        RESP: FromCommandOutput,
    {
        self.invoke_from(Location::caller(), executor)
    }

    async fn invoke_from<'exec, RESP>(
        &'exec self,
        caller: &'static Location<'static>,
        executor: &CloudTerrastodonExecutor,
    ) -> eyre::Result<BatchResponse<RESP>>
    where
        T: Facet<'exec>,
        RESP: FromCommandOutput,
    {
        invoke_batch_request_from(self, caller, executor).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn batch_request_stores_the_tenant_binding() -> Result<()> {
        CancellationToken::new().with_scope(|_cancellation| {
            let tenant_id = "22222222-2222-2222-2222-222222222222".parse()?;
            let authentication = AzureBearerAuthenticationSelection {
                tenant_id,
                ..test_authentication()
            };
            let mut request = AzureBatchRequest::new(authentication);
            request.requests.push(AzureBatchRequestEntry::new_get(
                "/subscriptions/example/providers/Microsoft.Resources/tags/default".to_owned(),
            ));

            assert_eq!(request.authentication.tenant_id, tenant_id);
            Ok(())
        })
    }
}
