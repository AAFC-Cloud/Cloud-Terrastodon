use arbitrary::Arbitrary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_cli_types::HttpMethodCli;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_rest::RequestHeaders;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestResponseBody;
use cloud_terrastodon_rest::RestService;
use cloud_terrastodon_rest::SerializableRestResponse;
use cloud_terrastodon_rest::read_optional_body;
use cloud_terrastodon_rest::read_optional_headers;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use facet::Facet;
use reqwest::Url;
use std::io::Write;

/// Arguments for issuing raw REST calls with Cloud Terrastodon's auth helpers.
#[derive(Facet, Debug, Clone)]
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

    /// Explicit organization scope for Azure DevOps REST authentication.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Named Cloud Terrastodon account; omit for its deliberately configured default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
    /// Credential mechanism; auto uses the selected account preference.
    #[facet(figue::named, default)]
    pub auth_source: AuthSource,
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
            org: Option::<AzureDevOpsOrganizationUrl>::arbitrary(u)?,
            account: Option::<CloudTerrastodonAccountArgument>::arbitrary(u)?,
            auth_source: AuthSource::arbitrary(u)?,
        })
    }
}
impl RestArgs {
    /// Return the response with body-only text and shared JSON/Facet rendering.
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let response = self.fetch_response(executor).await?;
        Ok(response_output(response))
    }

    async fn fetch_response(
        self,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<SerializableRestResponse> {
        let url = Url::parse(&self.url).with_context(|| format!("parsing URL '{}'", self.url))?;
        let service = RestService::infer(&url).wrap_err_with(|| {
            format!("unsupported REST host '{}'", url.host_str().unwrap_or(""))
        })?;
        let mut request = RestRequest::from_method_and_url(self.method.0, url.clone())?;
        if service != RestService::AzureRetailPrices {
            eyre::ensure!(
                self.auth_source != AuthSource::None,
                "authentication is disabled by auth source none"
            );
            let account = self.account.as_ref().map(ToString::to_string);
            if service == RestService::AzureDevOps {
                eyre::ensure!(
                    self.tenant.is_none() || self.auth_source != AuthSource::PersonalAccessToken,
                    "PAT authentication cannot be combined with --tenant"
                );
                eyre::ensure!(
                    self.tenant.is_some()
                        || matches!(
                            self.auth_source,
                            AuthSource::Auto | AuthSource::PersonalAccessToken
                        ),
                    "bearer authentication requires an explicit tenant"
                );
                let org = self
                    .org
                    .as_ref()
                    .wrap_err("Azure DevOps REST requires --org")?;
                let tenant_id = match &self.tenant {
                    Some(tenant) => Some(tenant.resolve(executor.cancellation()).await?),
                    None => None,
                };
                request = request.authentication(
                    executor
                        .resolve_azure_devops_authentication(
                            org,
                            tenant_id,
                            account.as_deref(),
                            self.auth_source,
                        )?
                        .into(),
                );
            } else {
                eyre::ensure!(
                    self.auth_source != AuthSource::PersonalAccessToken,
                    "Azure authentication does not support personal access tokens"
                );
                let tenant = self.tenant.as_ref().wrap_err(
                    "authenticated REST requires --tenant (ID, alias, or configured default)",
                )?;
                request = request.authentication(
                    executor
                        .resolve_azure_authentication(
                            tenant.resolve(executor.cancellation()).await?,
                            account.as_deref(),
                            self.auth_source,
                        )?
                        .into(),
                );
            }
        }
        let body = read_optional_body(self.body).await?;
        let headers = read_optional_headers(self.headers).await?;
        let header = RequestHeaders::from_header_lines(self.header.iter().map(String::as_str))?;
        let headers = match (headers, header) {
            (Some(headers), Some(header)) => Some(headers.merge(header)),
            (Some(headers), None) | (None, Some(headers)) => Some(headers),
            (None, None) => None,
        };
        request.body = body;
        request.headers = headers;
        request.receive_raw(executor).await
    }
}

fn response_output(response: SerializableRestResponse) -> CliOutput {
    let status_result = if response.ok {
        Ok(())
    } else {
        Err(eyre::eyre!(
            "REST call failed with status {}: {}",
            response.status.as_u16(),
            response.reason_phrase.as_deref().unwrap_or("Unknown error")
        ))
    };
    CliOutput::facet_with_text(response, write_response_body).with_result(status_result)
}

fn write_response_body(
    response: &SerializableRestResponse,
    writer: &mut dyn Write,
    _cancellation: &CancellationScope,
) -> Result<()> {
    let body = match &response.body {
        RestResponseBody::Json(body) => body.as_str(),
        RestResponseBody::Text(body) => body.as_str(),
    };
    writer
        .write_all(body.as_bytes())
        .wrap_err("failed to write REST response body")
}

#[cfg(test)]
#[allow(
    clippy::disallowed_types,
    reason = "These tests construct the REST transport's raw response-body representation."
)]
mod test {
    use super::RestArgs;
    use super::RestService;
    use super::SerializableRestResponse;
    use super::response_output;
    use cloud_terrastodon_app::GlobalArgs;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_execution::CloudTerrastodonExecutor;
    use cloud_terrastodon_execution::UnreachableAuthenticationProvider;
    use cloud_terrastodon_rest::RestResponseBody;
    use cloud_terrastodon_rest::RestResponseHeaders;
    use cloud_terrastodon_rest::parse_response_body;
    use facet::Facet;
    use facet_json::RawJson;
    use http::StatusCode;
    use reqwest::Url;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;
    use std::sync::Arc;

    #[derive(Facet, Debug)]
    struct ParseArgs {
        #[facet(flatten)]
        global_args: GlobalArgs,

        #[facet(flatten)]
        args: RestArgs,
    }

    #[tokio::test]
    async fn disabled_authentication_rejects_before_tenant_or_body_file_resolution() {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(UnreachableAuthenticationProvider),
                    cancellation,
                );
                let args = RestArgs {
                    method: cloud_terrastodon_cli_types::HttpMethodCli(http::Method::GET),
                    url: "https://graph.microsoft.com/v1.0/users".to_owned(),
                    body: Some("@must-not-read-body.json".to_owned()),
                    org: None,
                    account: None,
                    auth_source: AuthSource::None,
                    headers: None,
                    header: Vec::new(),
                    tenant: Some("must-not-resolve-tenant".parse()?),
                };
                let error = args
                    .fetch_response(&executor)
                    .await
                    .expect_err("disabled authentication must precede local or service lookups");
                assert!(error.to_string().contains("auth source none"));
                eyre::Ok(())
            })
            .await
            .unwrap();
    }

    #[test]
    fn facet_pretty_renders_the_full_response() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let response = SerializableRestResponse::new(
                    StatusCode::OK,
                    &HeaderMap::new(),
                    "synthetic response body".to_owned(),
                );
                let mut output = Vec::new();
                response_output(response)
                    .write_to(
                        Some(OutputFormat::FacetPretty),
                        false,
                        &mut output,
                        &cancellation,
                    )
                    .unwrap();
                let output = String::from_utf8(output).unwrap();
                assert!(output.contains("status"));
                assert!(output.contains("200"));
                assert!(output.contains("synthetic response body"));
                assert!(output.ends_with('\n'));
                eyre::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn failed_responses_are_printed_before_returning_http_error_in_all_formats() {
        CancellationToken::new()
            .with_scope(|cancellation| {
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
                        .write_to(Some(format), false, &mut output, &cancellation)
                        .expect_err("a failed REST status must remain a command failure");
                    let output = String::from_utf8(output).unwrap();
                    assert!(output.contains("synthetic access denied"));
                    assert!(error.to_string().contains("403"));
                }
                eyre::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn custom_text_keeps_only_the_body_and_redirected_default_keeps_full_json() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let response = SerializableRestResponse::new(
                    StatusCode::OK,
                    &HeaderMap::new(),
                    "synthetic response body".to_owned(),
                );
                let mut text = Vec::new();
                response_output(response.clone())
                    .write_to(Some(OutputFormat::Text), false, &mut text, &cancellation)
                    .unwrap();
                assert_eq!(text, b"synthetic response body\n");

                let mut json = Vec::new();
                response_output(response)
                    .write_to(None, false, &mut json, &cancellation)
                    .unwrap();
                let json: serde_json::Value = serde_json::from_slice(&json).unwrap();
                assert_eq!(json["status"], 200);
                assert_eq!(json["ok"], true);
                assert!(json.get("headers").is_some());
                eyre::Ok(())
            })
            .unwrap();
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
        .into_result()
        .expect("synthetic CLI arguments should parse")
        .get();

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
        .into_result()
        .expect("synthetic CLI arguments should parse")
        .get();

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
