use clap::Args;
use clap::ValueEnum;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::SubscriptionIdExt;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use cloud_terrastodon_rest::SerializableRestResponse;
use cloud_terrastodon_rest::infer_tenant_id_for_request;
use cloud_terrastodon_rest::read_optional_body;
use cloud_terrastodon_rest::read_optional_headers;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use http::Method;
use reqwest::Url;

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
    pub async fn invoke(self) -> Result<SerializableRestResponse> {
        let url = Url::parse(&self.url).with_context(|| format!("parsing URL '{}'", self.url))?;
        let service = RestService::infer(&url).wrap_err_with(|| {
            format!("unsupported REST host '{}'", url.host_str().unwrap_or(""))
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
        let mut request = RestRequest::new(self.method, url.as_str())?;
        request.service = service;
        request.body = body;
        request.headers = headers;
        request.tenant = tenant;
        request.receive_raw().await
    }

    pub async fn invoke_and_print(self) -> Result<()> {
        let output_format = match self.output_format {
            RestOutputFormat::Text => cloud_terrastodon_rest::RestOutputFormat::Text,
            RestOutputFormat::Json => cloud_terrastodon_rest::RestOutputFormat::Json,
        };
        let response = self.invoke().await?;
        response.write(output_format, std::io::stdout())
    }
}

#[cfg(test)]
mod test {
    use super::RestService;
    use cloud_terrastodon_rest::RestResponseBody;
    use cloud_terrastodon_rest::parse_response_body;
    use cloud_terrastodon_rest::serialize_headers;
    use facet_json::RawJson;
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
            RestResponseBody::Json(RawJson::from_owned("{\"hello\":\"world\"}".to_string()))
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
