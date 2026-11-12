use crate::AuthBearerExt;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::tls::Version;

pub async fn create_azure_devops_rest_client(token: &impl AuthBearerExt) -> eyre::Result<Client> {
    let client: Client = ClientBuilder::new()
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, token.as_authorization_header_value()?);
            headers
        })
        .min_tls_version(Version::TLS_1_2)
        .build()?;
    Ok(client)
}
