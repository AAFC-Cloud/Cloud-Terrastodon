use crate::cli::scalar_args::HttpMethodCli;
use arbitrary::Arbitrary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::SubscriptionIdExt;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_rest::RequestHeaders;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestResponseBody;
use cloud_terrastodon_rest::RestService;
use cloud_terrastodon_rest::SerializableRestResponse;
use cloud_terrastodon_rest::infer_tenant_id_for_request;
use cloud_terrastodon_rest::read_optional_body;
use cloud_terrastodon_rest::read_optional_headers;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use reqwest::Url;

/// Arguments for issuing raw REST calls with Cloud Terrastodon's auth helpers.
#[derive(facet::Facet, Debug, Clone)]
pub struct RestArgs {
    /// The HTTP method to use for the REST call.
    #[facet(figue::named)]
    pub method: HttpMethodCli,

    /// The REST API URL to call.
    #[facet(figue::named)]
    pub url: String,

    /// Optional request body for POST/PUT requests. If begins with '@', reads from file.
    #[facet(figue::named)]
    pub body: Option<String>,

    /// Optional request headers as a JSON object. If begins with '@', reads from file.
    #[facet(figue::named)]
    pub headers: Option<String>,

    /// Optional request header in `name: value` form. May be specified multiple times.
    #[facet(figue::named, default)]
    pub header: Vec<String>,

    /// Optional tracked tenant id or alias to use when acquiring Azure access tokens.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl<'a> Arbitrary<'a> for RestArgs {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut host = String::arbitrary(u)?
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
            .collect::<String>();
        if host.is_empty() {
            host.push_str("example");
        }
        Ok(Self {
            method: HttpMethodCli::arbitrary(u)?,
            url: format!("https://{host}.example.com/api"),
            body: Option::<String>::arbitrary(u)?,
            headers: Option::<String>::arbitrary(u)?,
            header: Vec::<String>::arbitrary(u)?,
            tenant: Option::<AzureTenantArgument<'static>>::arbitrary(u)?,
        })
    }
}
impl RestArgs {
    /// Return the response with body-only text and shared JSON/Facet rendering.
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let response = self.fetch_response(auth_context).await?;
        Ok(response_output(response))
    }

    async fn fetch_response(self, auth_context: &AuthContext) -> Result<SerializableRestResponse> {
        let url = Url::parse(&self.url).with_context(|| format!("parsing URL '{}'", self.url))?;
        let service = RestService::infer(&url).wrap_err_with(|| {
            format!("unsupported REST host '{}'", url.host_str().unwrap_or(""))
        })?;
        let tenant_inference_url = url.clone();
        let requested_tenant = match self.tenant {
            Some(tenant) => Some(tenant.resolve().await?),
            None => None,
        };
        let tenant = match requested_tenant.or(auth_context.tenant_id()) {
            Some(tenant) => Some(tenant),
            None => {
                infer_tenant_id_for_request(service, &url, |subscription_id| async move {
                    subscription_id
                        .resolve_tenant_id(auth_context)
                        .await
                        .with_context(|| {
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
        let header = RequestHeaders::from_header_lines(self.header.iter().map(String::as_str))?;
        let headers = match (headers, header) {
            (Some(headers), Some(header)) => Some(headers.merge(header)),
            (Some(headers), None) | (None, Some(headers)) => Some(headers),
            (None, None) => None,
        };
        let mut request =
            RestRequest::from_method_and_url(self.method.0, url)?.azure_auth_context(auth_context);
        request.service = service;
        request.body = body;
        request.headers = headers;
        request.tenant = tenant;
        request.receive_raw().await
    }
}

fn response_output(response: SerializableRestResponse) -> CliOutput {
    let status_result = if response.ok {
        Ok(())
    } else {
        Err(eyre::eyre!(
            "REST call failed with status {}: {}",
            response.status,
            response.reason_phrase.as_deref().unwrap_or("Unknown error")
        ))
    };
    CliOutput::facet_with_text(response, render_response_body).with_result(status_result)
}

fn render_response_body(response: &SerializableRestResponse) -> Result<String> {
    Ok(match &response.body {
        RestResponseBody::Json(body) => body.as_str().to_owned(),
        RestResponseBody::Text(body) => body.clone(),
    })
}

#[cfg(test)]
mod test {
    use super::RestArgs;
    use super::RestService;
    use super::SerializableRestResponse;
    use super::response_output;
    use cloud_terrastodon_app::GlobalArgs;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_rest::RestResponseBody;
    use cloud_terrastodon_rest::RestResponseHeaders;
    use cloud_terrastodon_rest::parse_response_body;
    use facet_json::RawJson;
    use http::StatusCode;
    use reqwest::Url;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

    #[derive(facet::Facet, Debug)]
    struct ParseArgs {
        #[facet(flatten)]
        global_args: GlobalArgs,

        #[facet(flatten)]
        args: RestArgs,
    }

    #[test]
    fn facet_pretty_renders_the_full_response() {
        let response = SerializableRestResponse::new(
            StatusCode::OK,
            &HeaderMap::new(),
            "synthetic response body".to_owned(),
        );
        let mut output = Vec::new();
        response_output(response)
            .write_to(Some(OutputFormat::FacetPretty), false, &mut output)
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("status"));
        assert!(output.contains("200"));
        assert!(output.contains("synthetic response body"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn failed_responses_are_printed_before_returning_http_error_in_all_formats() {
        for format in [
            OutputFormat::Text,
            OutputFormat::Json,
            OutputFormat::FacetPretty,
        ] {
            let response = SerializableRestResponse::new(
                StatusCode::FORBIDDEN,
                &HeaderMap::new(),
                "synthetic access denied".to_owned(),
            );
            let mut output = Vec::new();
            let error = response_output(response)
                .write_to(Some(format), false, &mut output)
                .expect_err("a failed REST status must remain a command failure");
            let output = String::from_utf8(output).unwrap();
            assert!(output.contains("synthetic access denied"));
            assert!(error.to_string().contains("403"));
        }
    }

    #[test]
    fn custom_text_keeps_only_the_body_and_redirected_default_keeps_full_json() {
        let response = SerializableRestResponse::new(
            StatusCode::OK,
            &HeaderMap::new(),
            "synthetic response body".to_owned(),
        );
        let mut text = Vec::new();
        response_output(response.clone())
            .write_to(Some(OutputFormat::Text), false, &mut text)
            .unwrap();
        assert_eq!(text, b"synthetic response body\n");

        let mut json = Vec::new();
        response_output(response)
            .write_to(None, false, &mut json)
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(json["status"], 200);
        assert_eq!(json["ok"], true);
        assert!(json.get("headers").is_some());
    }

    #[test]
    fn parses_shared_output_format_with_rest_arguments() {
        let parsed: ParseArgs = figue::from_slice(&[
            "--method",
            "GET",
            "--url",
            "https://graph.microsoft.com/v1.0/users",
            "--output-format",
            "json",
        ])
        .unwrap();

        assert_eq!(parsed.global_args.output_format, Some(OutputFormat::Json));
    }

    #[test]
    fn parses_repeated_header_options() {
        let parsed: ParseArgs = figue::from_slice(&[
            "--method",
            "GET",
            "--url",
            "https://graph.microsoft.com/v1.0/users",
            "--header",
            "ConsistencyLevel: eventual",
            "--header",
            "Accept: application/json",
        ])
        .unwrap();

        assert_eq!(
            parsed.args.header,
            vec![
                "ConsistencyLevel: eventual".to_owned(),
                "Accept: application/json".to_owned()
            ]
        );
    }

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

        let serialized = RestResponseHeaders::from(&headers);
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

cloud_terrastodon_registry::register_thing!(RestArgs);
cloud_terrastodon_registry::register_arbitrary!(RestArgs);
