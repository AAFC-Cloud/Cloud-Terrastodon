use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_cli_types::HttpMethodCli;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_rest::RestOutputFormat;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::read_optional_body;
use eyre::Result;
use facet::Facet;

// TODO: deprecate in favour of top-level rest command
/// Arguments for issuing raw Azure DevOps REST calls.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsRestArgs {
    /// Tenant id or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let auth_context = self
            .tenant
            .bind_auth_context(auth_context, executor.cancellation())
            .await?;
        let executor = &auth_context.bind_executor(executor)?;
        let mut request =
            RestRequest::new(self.method.0, &self.url)?.azure_devops_auth_context(&auth_context)?;
        request.body = read_optional_body(self.body).await?;
        let response = request.receive_raw(executor).await?;
        response.write(RestOutputFormat::Text, std::io::stdout())
    }
}

#[cfg(test)]
mod test {
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_auth::CredentialAuthenticationProvider;
    use cloud_terrastodon_azure_devops::get_default_organization_url;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_execution::CloudTerrastodonExecutor;
    use cloud_terrastodon_rest::RestRequest;
    use http::Method;
    use std::sync::Arc;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let org_url = get_default_organization_url(&cancellation).await?;
                let tenant = std::env::var("CT_TEST_AZDO_TENANT_ID")?.parse()?;
                let auth_context = AuthContext::explicit_azure_cli();
                let selected = AzureDevOpsAuthContext::for_tenant(&auth_context, tenant)?;
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(CredentialAuthenticationProvider::new(auth_context)),
                    cancellation.clone(),
                );
                let url = format!("{}/_apis/projects?api-version=7.1", org_url);
                let response = RestRequest::new(Method::GET, url.as_str())?
                    .azure_devops_auth_context(&selected)?
                    .receive_raw(&executor)
                    .await?;
                assert!(
                    response.ok,
                    "Unexpected HTTP status {}",
                    response.status.as_u16()
                );
                Ok(())
            })
            .await
    }
}
