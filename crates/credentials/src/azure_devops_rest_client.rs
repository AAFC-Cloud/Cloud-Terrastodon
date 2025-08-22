use crate::AzureDevOpsPersonalAccessToken;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::tls::Version;

pub async fn create_azure_devops_rest_client(
    pat: &AzureDevOpsPersonalAccessToken,
) -> eyre::Result<Client> {
    let client: Client = ClientBuilder::new()
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, pat.into_authorization_header_value());
            headers
        })
        .min_tls_version(Version::TLS_1_2)
        .build()?;
    Ok(client)
}

#[cfg(test)]
mod test {
    use crate::create_azure_devops_rest_client;
    use crate::get_azure_devops_pat;
    use serde_json::Value;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let url = "https://dev.azure.com/aafc/_apis/projects?api-version=7.1";

        let pat = get_azure_devops_pat().await?;
        let client = create_azure_devops_rest_client(&pat).await?;

        let resp = client.get(url).send().await?;
        let status = resp.status();

        let content = resp.text().await?;
        println!("{:#?}", content);

        assert_eq!(200, status.as_u16(), "{:?}", status.canonical_reason());
        serde_json::from_str::<Value>(&content)?;
        Ok(())
    }
}
