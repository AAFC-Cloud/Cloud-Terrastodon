use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_azure_types::prelude::TenantId;
use cloud_terrastodon_command::prelude::CommandBuilder;
use cloud_terrastodon_command::prelude::CommandKind;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::tls::Version;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AccessToken {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "expiresOn")]
    pub expires_on_string: String,
    #[serde(rename = "expires_on")]
    pub expires_on_ticks: u64,
    pub subscription: SubscriptionId,
    pub tenant: TenantId,
    #[serde(rename = "tokenType")]
    pub token_type: TokenType,
}
impl AccessToken {
    pub fn get_bearer_string(&self) -> String {
        let to_encode = format!(":{}", self.access_token);
        let encoded = BASE64_STANDARD.encode(to_encode);
        format!("Basic {encoded}")
    }
}

#[derive(Debug, Deserialize)]
pub enum TokenType {
    Bearer,
}

/// This method is pretty brittle, will often fail with an error like:
/// ```txt
/// ERROR: (pii). Status: Response_Status.Status_InteractionRequired, Error code: 3399614467, Tag: 558133256
/// Please explicitly log in with:
/// az login --scope 499b84ac-1321-427f-aa17-267ca6975798/.default
/// ```
pub async fn get_token() -> eyre::Result<AccessToken> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    // https://www.dylanberry.com/2021/02/21/how-to-get-a-pat-personal-access-token-for-azure-devops-from-the-az-cli/
    // https://learn.microsoft.com/en-us/rest/api/azure/devops/tokens/?view=azure-devops-rest-7.1&tabs=powershell
    // https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow
    let azure_devops_resource_id = "499b84ac-1321-427f-aa17-267ca6975798";
    cmd.args([
        "account",
        "get-access-token",
        "--resource",
        azure_devops_resource_id,
    ]);
    let rtn = cmd.run().await?;
    Ok(rtn)
}

/// This method is brittle and should probably be avoided, see `get_token`
#[deprecated = "use az cli through CommandBuilder instead"]
pub async fn create_azure_devops_rest_client() -> eyre::Result<Client> {
    let mut bearer = HeaderValue::from_str(&get_token().await?.get_bearer_string())?;
    bearer.set_sensitive(true);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, bearer);
    headers.insert("X-VSS-ForceMsaaPassThrough", HeaderValue::from_str("true")?);
    let client: Client = ClientBuilder::new()
        .default_headers(headers)
        .min_tls_version(Version::TLS_1_2)
        .build()?;
    Ok(client)
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use crate::create_client::create_azure_devops_rest_client;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let url = "https://app.vssps.visualstudio.com/_apis/accounts?api-version=7.1";
        let client = create_azure_devops_rest_client().await?;
        let rtn: serde_json::Value = client.get(url).send().await?.json().await?;
        println!(
            "(as long as this returns json instead of html, it's good. it might complain about 'necessary parameters' or whatever.)"
        );
        println!("{:#?}", rtn);

        Ok(())
    }
}
