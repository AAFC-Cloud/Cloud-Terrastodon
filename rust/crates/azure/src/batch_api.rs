use cloud_terrastodon_core_azure_types::prelude::uuid::Uuid;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::bail;
use eyre::Result;
use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HttpMethod {
    PATCH,
    POST,
    GET,
}
impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HttpMethod::PATCH => "PATCH",
            HttpMethod::POST => "POST",
            HttpMethod::GET => "GET",
        })
    }
}

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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchRequestEntry<T> {
    #[serde(rename = "httpMethod")]
    pub http_method: HttpMethod,
    pub name: Uuid,
    pub url: String,
    pub content: Option<T>,
}
impl BatchRequestEntry<()> {
    pub fn new_get(url: String) -> Self {
        BatchRequestEntry {
            http_method: HttpMethod::GET,
            name: Uuid::new_v4(),
            url,
            content: None,
        }
    }
}
impl<T> BatchRequestEntry<T> {
    pub fn new(http_method: HttpMethod, url: String, content: Option<T>) -> Self {
        BatchRequestEntry {
            http_method,
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

pub async fn invoke_batch_request<REQ, RESP>(
    request: &BatchRequest<REQ>,
) -> Result<BatchResponse<RESP>>
where
    REQ: Serialize + Clone,
    RESP: DeserializeOwned,
{
    // create the base command
    let url = "https://management.azure.com/batch?api-version=2020-06-01";
    let mut cmd_base = CommandBuilder::new(CommandKind::AzureCLI);
    cmd_base.args(["rest", "--method", "POST", "--url", url, "--body"]);

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
        cmd.file_arg(
            "body.json",
            serde_json::to_string_pretty(&BatchRequest {
                requests: chunk.iter().cloned().collect_vec(),
            })?,
        );

        info!("Performing batch request {} of {}", i + 1, num_chunks);
        let response = cmd.run_with_validator(validator).await?;
        rtn.responses.extend(response.responses);
    }
    assert_eq!(request.requests.len(), rtn.responses.len());
    for (a, b) in request.requests.iter().zip(rtn.responses.iter()) {
        assert_eq!(a.name, b.name);
    }
    Ok(rtn)
}
impl<T> BatchRequest<T>
where
    T: Serialize + Clone,
{
    pub async fn invoke<RESP>(&self) -> eyre::Result<BatchResponse<RESP>>
    where
        RESP: DeserializeOwned,
    {
        invoke_batch_request(self).await
    }
}
