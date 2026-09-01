use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_relative_location::RelativeLocation;
use cloud_terrastodon_rest::RestRequest;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use http::Method;
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashMap;
use std::future::Future;
use std::panic::Location;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct BatchRequest<'a, T> {
    pub requests: Vec<BatchRequestEntry<T>>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}
impl<'a, T> BatchRequest<'a, T> {
    pub fn new(auth_context: &'a AzureTenantAuthContext) -> Self {
        Self {
            requests: Vec::new(),
            auth_context: Cow::Borrowed(auth_context),
        }
    }
}

#[derive(Debug, Default, Clone, facet::Facet)]
struct BatchRequestUpstream<T> {
    requests: Vec<BatchRequestEntryUpstream<T>>,
}

#[derive(Debug, Clone)]
pub struct BatchRequestEntry<T> {
    pub http_method: Method,
    pub name: Uuid,
    pub url: String,
    pub content: Option<T>,
}

#[derive(Debug, Clone, facet::Facet)]
struct BatchRequestEntryUpstream<T> {
    #[facet(opaque, rename = "httpMethod", proxy = HttpMethodProxy)]
    pub http_method: Method,
    pub name: Uuid,
    pub url: String,
    pub content: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
struct HttpMethodProxy(String);

impl TryFrom<HttpMethodProxy> for Method {
    type Error = http::method::InvalidMethod;

    fn try_from(value: HttpMethodProxy) -> Result<Self, Self::Error> {
        Method::from_bytes(value.0.as_bytes())
    }
}

impl From<&Method> for HttpMethodProxy {
    fn from(value: &Method) -> Self {
        Self(value.as_str().to_string())
    }
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
    pub fn new_get(url: String) -> Self {
        BatchRequestEntry {
            http_method: Method::GET,
            name: Uuid::new_v4(),
            url,
            content: None,
        }
    }
}
impl<T> BatchRequestEntry<T> {
    pub fn new(http_method: Method, url: String, content: Option<T>) -> Self {
        BatchRequestEntry {
            http_method,
            name: Uuid::new_v4(),
            url,
            content,
        }
    }
}

#[derive(Debug, facet::Facet)]
pub struct BatchResponse<T> {
    pub responses: Vec<BatchResponseEntry<T>>,
}
#[derive(Debug, facet::Facet)]
pub struct BatchResponseEntry<T> {
    pub name: Uuid,
    #[facet(rename = "httpStatusCode")]
    pub http_status_code: u16,
    pub headers: HashMap<String, String>,
    pub content: T,
    #[facet(rename = "contentLength")]
    pub content_length: u64,
}

#[track_caller]
pub fn invoke_batch_request<'a, REQ, RESP>(
    request: &'a BatchRequest<'_, REQ>,
) -> impl Future<Output = Result<BatchResponse<RESP>>> + 'a
where
    REQ: Facet<'static> + Clone,
    RESP: FromCommandOutput,
{
    invoke_batch_request_from(request, Location::caller())
}

async fn invoke_batch_request_from<REQ, RESP>(
    request: &BatchRequest<'_, REQ>,
    caller: &'static Location<'static>,
) -> Result<BatchResponse<RESP>>
where
    REQ: Facet<'static> + Clone,
    RESP: FromCommandOutput,
{
    let result: Result<BatchResponse<RESP>> = async {
        if request.requests.is_empty() {
            return Ok(BatchResponse {
                responses: Vec::new(),
            });
        }

        let auth_context = request.auth_context.as_ref();

        let url = "https://management.azure.com/batch?api-version=2020-06-01";

        // create the results holder
        let mut rtn = BatchResponse {
            responses: Vec::new(),
        };

        // batch the batch requests into size=20 chunks
        let chunks = request.requests.chunks(20);
        let num_chunks = chunks.len();
        for (i, chunk) in chunks.enumerate() {
            let request = RestRequest::new(Method::POST, url)?
                .auth_context(&auth_context.auth_context)
                .tenant(auth_context.tenant_id)
                .body(
                    facet_json::to_string_pretty(&BatchRequestUpstream {
                        requests: chunk
                            .iter()
                            .map(BatchRequestEntryUpstream::from)
                            .collect_vec(),
                    })
                    .map_err(|error| eyre::eyre!("{error:?}"))?,
                );

            debug!(
                batch_index = i,
                total_batches = num_chunks,
                "Performing batch request"
            );
            let response = request
                .receive_with_validator(|response: BatchResponse<RESP>| {
                    let failures = response
                        .responses
                        .iter()
                        .filter(|resp| resp.http_status_code != 200)
                        .count();
                    if failures > 0 {
                        bail!("There were {} requests with non-200 status codes", failures);
                    }
                    Ok(response)
                })
                .await?;
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
impl<T> BatchRequest<'_, T>
where
    T: Facet<'static> + Clone,
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

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_credentials::AuthSource;

    #[test]
    fn batch_request_stores_the_tenant_binding() -> Result<()> {
        let tenant_id = "22222222-2222-2222-2222-222222222222".parse()?;
        let auth_context =
            AuthContext::explicit(AuthSource::AzureCli).bind_to_azure_tenant(tenant_id)?;
        let mut request = BatchRequest::new(&auth_context);
        request.requests.push(BatchRequestEntry::new_get(
            "/subscriptions/example/providers/Microsoft.Resources/tags/default".to_owned(),
        ));

        assert_eq!(request.auth_context.tenant_id, tenant_id);
        Ok(())
    }
}
