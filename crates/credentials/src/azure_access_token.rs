use crate::AzureDevOpsPersonalAccessToken;
use cloud_terrastodon_azure_types::prelude::AccessToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

pub const AZURE_DEVOPS_RESOURCE_ID: &str = "499b84ac-1321-427f-aa17-267ca6975798";

/// This method is kinda brittle, may fail with an error like:
/// ```txt
/// ERROR: (pii). Status: Response_Status.Status_InteractionRequired, Error code: 3399614467, Tag: 558133256
/// Please explicitly log in with:
/// az login --scope 499b84ac-1321-427f-aa17-267ca6975798/.default
/// ```
pub async fn fetch_azure_devops_personal_access_token()
-> eyre::Result<AccessToken<AzureDevOpsPersonalAccessToken>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    // https://www.dylanberry.com/2021/02/21/how-to-get-a-pat-personal-access-token-for-azure-devops-from-the-az-cli/
    // https://learn.microsoft.com/en-us/rest/api/azure/devops/tokens/?view=azure-devops-rest-7.1&tabs=powershell
    // https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow
    cmd.args([
        "account",
        "get-access-token",
        "--resource",
        AZURE_DEVOPS_RESOURCE_ID,
    ]);
    let rtn = cmd.run().await?;
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use crate::azure_access_token::fetch_azure_devops_personal_access_token;
    use crate::create_azure_devops_rest_client;
    use serde_json::Value;

    #[tokio::test]
    pub async fn it_works_profiles() -> eyre::Result<()> {
        // this endpoint only works with the `az account get-access-token`, not with a raw PAT
        let url = "https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=6.0";

        let pat = fetch_azure_devops_personal_access_token()
            .await?
            .access_token;
        let client = create_azure_devops_rest_client(&pat).await?;

        let resp = client.get(url).send().await?;
        let status = resp.status();

        let content = resp.text().await?;
        println!("{:#?}", content);

        assert_eq!(200, status.as_u16(), "{:?}", status.canonical_reason());
        serde_json::from_str::<Value>(&content)?;
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works_projects() -> eyre::Result<()> {
        // this endpoint only works with the `az account get-access-token`, not with a raw PAT
        let url = "https://dev.azure.com/aafc/_apis/projects?api-version=7.1";

        let pat = fetch_azure_devops_personal_access_token()
            .await?
            .access_token;
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
