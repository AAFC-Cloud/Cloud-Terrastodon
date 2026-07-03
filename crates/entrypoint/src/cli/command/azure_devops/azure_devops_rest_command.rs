use crate::cli::scalar_args::HttpMethodCli;
use cloud_terrastodon_rest::RestOutputFormat;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::read_optional_body;
use eyre::Result;

// TODO: deprecate in favour of top-level rest command
/// Arguments for issuing raw Azure DevOps REST calls.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsRestArgs {
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
    pub async fn invoke(self) -> Result<()> {
        let mut request = RestRequest::new(self.method.0, &self.url)?;
        request.body = read_optional_body(self.body).await?;
        let response = request.receive_raw().await?;
        response.write(RestOutputFormat::Text, std::io::stdout())
    }
}

#[cfg(test)]
mod test {
    use cloud_terrastodon_azure_devops::get_default_organization_url;
    use cloud_terrastodon_rest::RestRequest;
    use http::Method;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let url = format!("{}/_apis/projects?api-version=7.1", org_url);
        let response = RestRequest::new(Method::GET, url.as_str())?
            .receive_raw()
            .await?;
        println!("{}", facet_json::to_string_pretty(&response)?);
        Ok(())
    }
}

