use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cli_types::HttpMethodCli;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestResponseBody;
use cloud_terrastodon_rest::read_optional_body;
use eyre::Result;
use facet::Facet;

// TODO: deprecate in favour of top-level rest command
/// Arguments for issuing raw Azure DevOps REST calls.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsRestArgs {
    /// Organization scope for account selection.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,

    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// The HTTP method to use for the REST call.
    #[facet(figue::named)]
    pub method: HttpMethodCli,

    /// The Azure DevOps REST API URL to call (e.g., https://dev.azure.com/{organization}/_apis/projects).
    #[facet(figue::named)]
    pub url: String,

    /// Optional request body for POST/PUT requests. If begins with '@', reads from file.
    #[facet(figue::named)]
    pub body: Option<String>,
}

impl AzureDevOpsRestArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let mut request =
            RestRequest::new(self.method.0, &self.url)?.authentication(auth_selection.into());
        request.body = read_optional_body(self.body).await?;
        let response = request.receive_raw(executor).await?;
        let result = response.error_for_status().map_err(Into::into);
        Ok(
            CliOutput::facet_with_text(response, |response, writer, _scope| {
                let body = match &response.body {
                    RestResponseBody::Json(body) => body.as_str(),
                    RestResponseBody::Text(body) => body.as_str(),
                };
                writer.write_all(body.as_bytes())?;
                Ok(())
            })
            .with_result(result),
        )
    }
}
