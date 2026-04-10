use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use http::Method;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::panic::Location;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct BatchRequest<T> {
    pub requests: Vec<BatchRequestEntry<T>>,
}
impl<T> BatchRequest<T>
where
    T: Default,
{
    pub fn new() -> Self {
        BatchRequest::<T>::default()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct BatchRequestUpstream<T> {
    requests: Vec<BatchRequestEntryUpstream<T>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchRequestEntry<T> {
    #[serde(
        rename = "httpMethod",
        deserialize_with = "cloud_terrastodon_azure_types::serde_helpers::deserialize_using_from_str",
        serialize_with = "cloud_terrastodon_azure_types::serde_helpers::serialize_using_asref_str"
    )]
    pub http_method: Method,
    pub tenant_id: AzureTenantId,
    pub name: Uuid,
    pub url: String,
    pub content: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BatchRequestEntryUpstream<T> {
    #[serde(
        rename = "httpMethod",
        deserialize_with = "cloud_terrastodon_azure_types::serde_helpers::deserialize_using_from_str",
        serialize_with = "cloud_terrastodon_azure_types::serde_helpers::serialize_using_asref_str"
    )]
    pub http_method: Method,
    pub name: Uuid,
    pub url: String,
    pub content: Option<T>,
}

impl<T: Clone> From<&BatchRequestEntry<T>> for BatchRequestEntryUpstream<T> {
    fn from(value: &BatchRequestEntry<T>) -> Self {
        BatchRequestEntryUpstream {
            http_method: value.http_method.clone(),
            name: value.name,
            url: value.url.clone(),
            content: value.content.clone(),
        }
    }
}

impl BatchRequestEntry<()> {
    pub fn new_get(tenant_id: AzureTenantId, url: String) -> Self {
        BatchRequestEntry {
            http_method: Method::GET,
            tenant_id,
            name: Uuid::new_v4(),
            url,
            content: None,
        }
    }
}
impl<T> BatchRequestEntry<T> {
    pub fn new(
        tenant_id: AzureTenantId,
        http_method: Method,
        url: String,
        content: Option<T>,
    ) -> Self {
        BatchRequestEntry {
            http_method,
            tenant_id,
            name: Uuid::new_v4(),
            url,
            content,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchResponse<T> {
    pub responses: Vec<BatchResponseEntry<T>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchResponseEntry<T> {
    pub name: Uuid,
    #[serde(rename = "httpStatusCode")]
    pub http_status_code: u16,
    pub headers: HashMap<String, String>,
    pub content: T,
    #[serde(rename = "contentLength")]
    pub content_length: u64,
}

#[track_caller]
pub fn invoke_batch_request<REQ, RESP>(
    request: &BatchRequest<REQ>,
) -> impl Future<Output = Result<BatchResponse<RESP>>> + '_
where
    REQ: Serialize + Clone,
    RESP: FromCommandOutput,
{
    invoke_batch_request_from(request, Location::caller())
}

async fn invoke_batch_request_from<REQ, RESP>(
    request: &BatchRequest<REQ>,
    caller: &'static Location<'static>,
) -> Result<BatchResponse<RESP>>
where
    REQ: Serialize + Clone,
    RESP: FromCommandOutput,
{
    let result: Result<BatchResponse<RESP>> = async {
        if request.requests.is_empty() {
            return Ok(BatchResponse {
                responses: Vec::new(),
            });
        }

        let tenant_id = request.requests[0].tenant_id;
        ensure!(
            request
                .requests
                .iter()
                .all(|entry| entry.tenant_id == tenant_id),
            "Batch request entries must all use the same tenant ID"
        );

        // create the base command
        let url = "https://management.azure.com/batch?api-version=2020-06-01";
        let mut cmd_base = CommandBuilder::new(CommandKind::CloudTerrastodon);
        cmd_base.args(["rest", "--method", "POST", "--url", url]);
        let tenant_id = tenant_id.to_string();
        cmd_base.args(["--tenant", tenant_id.as_str()]);
        cmd_base.arg("--body");

        // create the status=200 validator
        let validator = |response: BatchResponse<RESP>| {
            let failures = response
                .responses
                .iter()
                .filter(|resp| resp.http_status_code != 200)
                .count();
            if failures > 0 {
                bail!("There were {} requests with non-200 status codes", failures)
            }
            Ok(response)
        };

        // create the results holder
        let mut rtn = BatchResponse {
            responses: Vec::new(),
        };

        // batch the batch requests into size=20 chunks
        let chunks = request.requests.chunks(20);
        let num_chunks = chunks.len();
        for (i, chunk) in chunks.enumerate() {
            let mut cmd = cmd_base.clone();
            cmd.azure_file_arg(
                "body.json",
                serde_json::to_string_pretty(&BatchRequestUpstream {
                    requests: chunk
                        .iter()
                        .map(BatchRequestEntryUpstream::from)
                        .collect_vec(),
                })?,
            );

            debug!(
                batch_index = i,
                total_batches = num_chunks,
                "Performing batch request"
            );
            let response = cmd.run_with_validator(validator).await?;
            rtn.responses.extend(response.responses);
        }
        assert_eq!(request.requests.len(), rtn.responses.len());
        for (a, b) in request.requests.iter().zip(rtn.responses.iter()) {
            assert_eq!(a.name, b.name);
        }

        Ok(rtn)
    }
    .await;

    result.wrap_err(format!(
        "invoke_batch_request failed, called from {}",
        RelativeLocation::from(caller)
    ))
}
impl<T> BatchRequest<T>
where
    T: Serialize + Clone,
{
    #[track_caller]
    pub fn invoke<RESP>(&self) -> impl Future<Output = eyre::Result<BatchResponse<RESP>>> + '_
    where
        RESP: FromCommandOutput,
    {
        self.invoke_from(Location::caller())
    }

    async fn invoke_from<RESP>(
        &self,
        caller: &'static Location<'static>,
    ) -> eyre::Result<BatchResponse<RESP>>
    where
        RESP: FromCommandOutput,
    {
        invoke_batch_request_from(self, caller).await
    }
}
