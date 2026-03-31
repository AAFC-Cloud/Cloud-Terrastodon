use clap::Args;
use clap::ValueEnum;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::SubscriptionIdExt;
use cloud_terrastodon_credentials::RestResponseBody;
use cloud_terrastodon_credentials::RestService;
use cloud_terrastodon_credentials::SerializableRestResponse;
use cloud_terrastodon_credentials::execute_rest_request;
use cloud_terrastodon_credentials::infer_tenant_id_for_request;
use cloud_terrastodon_credentials::read_optional_body;
use cloud_terrastodon_credentials::read_optional_headers;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use http::Method;
use reqwest::Response;
use reqwest::Url;
use std::time::Instant;
use tracing::debug;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestOutputFormat {
    Text,
    Json,
}

/// Arguments for issuing raw REST calls with Cloud Terrastodon's auth helpers.
#[derive(Args, Debug, Clone)]
pub struct RestArgs {
    /// The HTTP method to use for the REST call.
    #[arg(long)]
    pub method: Method,

    /// The REST API URL to call.
    #[arg(long)]
    pub url: String,

    /// Optional request body for POST/PUT requests. If begins with '@', reads from file.
    #[arg(long)]
    pub body: Option<String>,

    /// Optional request headers as a JSON object. If begins with '@', reads from file.
    #[arg(long)]
    pub headers: Option<String>,

    /// Optional tracked tenant id or alias to use when acquiring Azure access tokens.
    #[arg(long)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Output format. `text` prints the response body only; `json` includes status and headers.
    #[arg(long, default_value = "text")]
    pub output_format: RestOutputFormat,
}

impl RestArgs {
    pub async fn invoke(self) -> Result<Response> {
        let url = Url::parse(&self.url).with_context(|| format!("parsing URL '{}'", self.url))?;
        let service = RestService::infer(&url).ok_or_else(|| {
            eyre::eyre!("unsupported REST host '{}'", url.host_str().unwrap_or(""))
        })?;
        let tenant_inference_url = url.clone();
        let tenant = match self.tenant {
            Some(tenant) => Some(tenant.resolve().await?),
            None => {
                infer_tenant_id_for_request(service, &url, |subscription_id| async move {
                    subscription_id.resolve_tenant_id().await.with_context(|| {
                        format!(
                            "Failed to infer tracked tenant for subscription '{}' from '{}'. If the Azure CLI default tenant is intended, specify '--tenant default'.",
                            subscription_id, tenant_inference_url
                        )
                    })
                })
                .await?
            }
        };
        let body = read_optional_body(self.body).await?;
        let headers = read_optional_headers(self.headers).await?;

        let start = Instant::now();
        let response =
            execute_rest_request(service, self.method, url, body, headers, tenant).await?;
        let elapsed = start.elapsed();
        debug!(
            elapsed_ms = elapsed.as_millis(),
            "REST call completed in {}",
            humantime::format_duration(elapsed)
        );

        Ok(response)
    }

    pub async fn invoke_and_print(self) -> Result<()> {
        let output_format = self.output_format;
        let response = self.invoke().await?;
        print_response(response, output_format).await
    }
}

async fn print_response(response: Response, output_format: RestOutputFormat) -> Result<()> {
    let start = Instant::now();
    let serialized_response = SerializableRestResponse::from_response(response).await?;
    let elapsed = start.elapsed();
    debug!(
        elapsed_ms = elapsed.as_millis(),
        "Response prettifying completed in {}",
        humantime::format_duration(elapsed)
    );

    match output_format {
        RestOutputFormat::Text => match &serialized_response.body {
            RestResponseBody::Json(value) => println!("{}", serde_json::to_string_pretty(value)?),
            RestResponseBody::Text(content) => {
                debug!("Response is not valid JSON, printing raw content");
                println!("{}", content);
            }
        },
        RestOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&serialized_response)?);
        }
    }

    if !serialized_response.ok {
        bail!(
            "REST call failed with status {}: {}",
            serialized_response.status,
            serialized_response
                .reason_phrase
                .as_deref()
                .unwrap_or("Unknown error")
        );
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::RestResponseBody;
    use super::RestService;
    use cloud_terrastodon_credentials::parse_response_body;
    use cloud_terrastodon_credentials::serialize_headers;
    use reqwest::Url;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

    #[test]
    fn infers_microsoft_graph() {
        let url = Url::parse("https://graph.microsoft.com/v1.0/organization").unwrap();
        assert_eq!(RestService::infer(&url), Some(RestService::MicrosoftGraph));
    }

    #[test]
    fn infers_azure_resource_manager() {
        let url = Url::parse("https://management.azure.com/subscriptions?api-version=2020-01-01")
            .unwrap();
        assert_eq!(
            RestService::infer(&url),
            Some(RestService::AzureResourceManager)
        );
    }

    #[test]
    fn infers_azure_devops_hosts() {
        for host in [
            "https://dev.azure.com/example/_apis/projects?api-version=7.1",
            "https://vssps.dev.azure.com/example/_apis/graph/users?api-version=7.1-preview.1",
            "https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=6.0",
        ] {
            let url = Url::parse(host).unwrap();
            assert_eq!(RestService::infer(&url), Some(RestService::AzureDevOps));
        }
    }

    #[test]
    fn rejects_unknown_hosts() {
        let url = Url::parse("https://example.com/api").unwrap();
        assert_eq!(RestService::infer(&url), None);
    }

    #[test]
    fn parses_json_response_body() {
        let body = parse_response_body("{\"hello\":\"world\"}".to_string());
        assert_eq!(
            body,
            RestResponseBody::Json(serde_json::json!({"hello": "world"}))
        );
    }

    #[test]
    fn preserves_text_response_body() {
        let body = parse_response_body("not json".to_string());
        assert_eq!(body, RestResponseBody::Text("not json".to_string()));
    }

    #[test]
    fn serializes_repeated_headers() {
        let mut headers = HeaderMap::new();
        headers.append("x-test", HeaderValue::from_static("a"));
        headers.append("x-test", HeaderValue::from_static("b"));
        headers.append("content-type", HeaderValue::from_static("application/json"));

        let serialized = serialize_headers(&headers);
        assert_eq!(
            serialized.get("x-test").unwrap(),
            &vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            serialized.get("content-type").unwrap(),
            &vec!["application/json".to_string()]
        );
    }
}
