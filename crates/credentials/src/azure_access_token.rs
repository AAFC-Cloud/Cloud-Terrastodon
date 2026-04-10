use crate::AzureDevOpsPersonalAccessToken;
use crate::AzureRestResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::FromCommandOutput;

pub async fn fetch_azure_access_token<T: FromCommandOutput>(
    tenant: Option<AzureTenantId>,
    resource: AzureRestResource,
) -> eyre::Result<AzureAccessToken<T>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["account", "get-access-token", "--output", "json"]);
    if let Some(tenant) = tenant {
        let tenant = tenant.to_string();
        cmd.args(["--tenant", tenant.as_str()]);
    }
    resource.apply_access_token_args(&mut cmd);
    cmd.run::<AzureAccessToken<T>>().await
}

/// This method is kinda brittle, may fail with an error like:
/// ```txt
/// ERROR: (pii). Status: Response_Status.Status_InteractionRequired, Error code: 3399614467, Tag: 558133256
/// Please explicitly log in with:
/// az login --scope 499b84ac-1321-427f-aa17-267ca6975798/.default
/// ```
pub async fn fetch_azure_devops_personal_access_token()
-> eyre::Result<AzureAccessToken<AzureDevOpsPersonalAccessToken>> {
    // https://www.dylanberry.com/2021/02/21/how-to-get-a-pat-personal-access-token-for-azure-devops-from-the-az-cli/
    // https://learn.microsoft.com/en-us/rest/api/azure/devops/tokens/?view=azure-devops-rest-7.1&tabs=powershell
    // https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow
    fetch_azure_access_token(None, AzureRestResource::AzureDevOps).await
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

        assert_eq!(200, status.as_u16(), "{:?}", status.canonical_reason());
        let parsed = serde_json::from_str::<Value>(&content)?;
        assert!(parsed.is_object());
        Ok(())
    }

    #[tokio::test]
    #[ignore] // todo: remove, deprecate in favour of get_azure_devops_personal_access_token_from_credential_manager
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

        assert_eq!(
            200,
            status.as_u16(),
            "{} - {:?}",
            status.as_u16(),
            status.canonical_reason()
        );
        let parsed = serde_json::from_str::<Value>(&content)?;
        assert!(parsed.is_object());
        Ok(())
    }
}
