use crate::AuthContext;
use crate::AuthSource;
use crate::AzureBearerToken;
use crate::AzureRestResource;
use crate::workload_identity::exchange_workload_identity_assertion;
use chrono::Local;
use chrono::TimeDelta;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::FromCommandOutput;
use reqwest::Client;
use std::collections::HashMap;
use std::io::IsTerminal;
use std::sync::OnceLock;
use tokio::sync::Mutex;

type WifTokenCache =
    HashMap<(AzureTenantId, AzureRestResource), AzureAccessToken<AzureBearerToken>>;

const TOKEN_REFRESH_BUFFER_SECONDS: i64 = 120;

static WIF_TOKEN_CACHE: OnceLock<Mutex<WifTokenCache>> = OnceLock::new();

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

/// Fetches a resource-specific Entra bearer token using the configured
/// authentication source. Azure CLI remains available only as a compatibility
/// source; workload identity never shells out to it.
pub async fn fetch_azure_bearer_access_token(
    auth_context: &AuthContext,
    tenant: Option<AzureTenantId>,
    resource: AzureRestResource,
) -> eyre::Result<AzureAccessToken<AzureBearerToken>> {
    match auth_context.source() {
        AuthSource::WorkloadIdentity => {
            let config = auth_context.workload_identity().ok_or_else(|| {
                eyre::eyre!(
                    "workload identity authentication requires AZURE_CLIENT_ID/AZURE_TENANT_ID/AZURE_FEDERATED_TOKEN or servicePrincipalId/tenantId/idToken"
                )
            })?;
            let tenant = tenant.unwrap_or(config.tenant_id);
            eyre::ensure!(
                tenant == config.tenant_id,
                "requested tenant does not match the workload identity tenant"
            );
            let cache = WIF_TOKEN_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
            let mut cache = cache.lock().await;
            if let Some(token) = cache.get(&(config.tenant_id, resource))
                && is_token_fresh(token, Local::now())
            {
                return Ok(token.clone());
            }
            let endpoint = config.token_endpoint()?;
            let (token, expires_in) =
                exchange_workload_identity_assertion(&Client::new(), endpoint, config, resource)
                    .await?;
            let token = AzureAccessToken {
                access_token: AzureBearerToken::new(token),
                expires_on: Local::now() + TimeDelta::seconds(expires_in as i64),
                subscription: None,
                tenant: config.tenant_id,
                token_type: cloud_terrastodon_azure_types::TokenType::Bearer,
            };
            cache.insert((config.tenant_id, resource), token.clone());
            Ok(token)
        }
        AuthSource::AzureCli => {
            eyre::ensure!(
                auth_context.allows_interactive_reauthentication(),
                "Azure CLI authentication is disabled for the selected/headless authentication context; provide workload identity credentials or choose a non-interactive source"
            );
            let token = fetch_azure_access_token::<String>(tenant, resource).await?;
            Ok(AzureAccessToken {
                access_token: AzureBearerToken::new(token.access_token),
                expires_on: token.expires_on,
                subscription: token.subscription,
                tenant: token.tenant,
                token_type: token.token_type,
            })
        }
        AuthSource::Browser => eyre::bail!(
            "browser authentication is reserved for delegated PIM; use workload-identity or azure-cli for ARM, Graph, and Azure DevOps REST requests"
        ),
        AuthSource::PersonalAccessToken => eyre::bail!(
            "personal access tokens are only valid for Azure DevOps compatibility requests"
        ),
        AuthSource::Auto => eyre::bail!(
            "authentication source was not resolved; construct an AuthContext at the CLI entrypoint"
        ),
    }
}

fn is_token_fresh(
    token: &AzureAccessToken<AzureBearerToken>,
    now: chrono::DateTime<Local>,
) -> bool {
    token.expires_on > now + TimeDelta::seconds(TOKEN_REFRESH_BUFFER_SECONDS)
}

pub fn is_headless_auth_context() -> bool {
    std::env::var_os("CI").is_some()
        || std::env::var_os("TF_BUILD").is_some()
        || std::env::var_os("BUILD_BUILDID").is_some()
        || !std::io::stdin().is_terminal()
}

/// This method is kinda brittle, may fail with an error like:
/// ```txt
/// ERROR: (pii). Status: Response_Status.Status_InteractionRequired, Error code: 3399614467, Tag: 558133256
/// Please explicitly log in with:
/// az login --scope 499b84ac-1321-427f-aa17-267ca6975798/.default
/// ```
pub async fn fetch_azure_devops_personal_access_token()
-> eyre::Result<AzureAccessToken<AzureBearerToken>> {
    // https://www.dylanberry.com/2021/02/21/how-to-get-a-pat-personal-access-token-for-azure-devops-from-the-az-cli/
    // https://learn.microsoft.com/en-us/rest/api/azure/devops/tokens/?view=azure-devops-rest-7.1&tabs=powershell
    // https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow
    let token = fetch_azure_access_token::<String>(None, AzureRestResource::AzureDevOps).await?;
    Ok(AzureAccessToken {
        access_token: AzureBearerToken::new(token.access_token),
        expires_on: token.expires_on,
        subscription: token.subscription,
        tenant: token.tenant,
        token_type: token.token_type,
    })
}

#[cfg(test)]
mod test {
    use crate::AuthContext;
    use crate::AuthSource;
    use crate::AzureBearerToken;
    use crate::azure_access_token::TOKEN_REFRESH_BUFFER_SECONDS;
    use crate::azure_access_token::fetch_azure_devops_personal_access_token;
    use crate::create_azure_devops_rest_client;
    use chrono::Local;
    use chrono::TimeDelta;
    use cloud_terrastodon_azure_types::AzureAccessToken;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_azure_types::TokenType;
    use facet_json::RawJson;

    #[test]
    fn explicit_context_is_used_for_dispatch() -> eyre::Result<()> {
        let context = AuthContext::explicit(AuthSource::WorkloadIdentity);
        assert_eq!(context.source(), AuthSource::WorkloadIdentity);
        Ok(())
    }

    #[test]
    fn token_refresh_buffer_is_deterministic() -> eyre::Result<()> {
        let now = Local::now();
        let tenant = "22222222-2222-2222-2222-222222222222".parse::<AzureTenantId>()?;
        let token = AzureAccessToken {
            access_token: AzureBearerToken::new("access-token"),
            expires_on: now + TimeDelta::seconds(TOKEN_REFRESH_BUFFER_SECONDS + 1),
            subscription: None,
            tenant,
            token_type: TokenType::Bearer,
        };
        assert!(super::is_token_fresh(&token, now));
        assert!(!super::is_token_fresh(
            &AzureAccessToken {
                expires_on: now + TimeDelta::seconds(TOKEN_REFRESH_BUFFER_SECONDS),
                ..token
            },
            now
        ));
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires a signed-in Azure CLI and live Azure DevOps access"]
    pub async fn it_works_profiles() -> eyre::Result<()> {
        // this endpoint only works with the `az account get-access-token`, not with a raw PAT
        let url = "https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=6.0";

        let token = fetch_azure_devops_personal_access_token().await?;
        let client = create_azure_devops_rest_client(&token).await?;

        let resp = client.get(url).send().await?;
        let status = resp.status();

        let content = resp.text().await?;

        assert_eq!(200, status.as_u16(), "{:?}", status.canonical_reason());
        let parsed = facet_json::from_str::<RawJson<'static>>(&content)?;
        assert!(parsed.as_str().trim_start().starts_with('{'));
        Ok(())
    }

    #[tokio::test]
    #[ignore] // todo: remove, deprecate in favour of get_azure_devops_personal_access_token_from_credential_manager
    pub async fn it_works_projects() -> eyre::Result<()> {
        // this endpoint only works with the `az account get-access-token`, not with a raw PAT
        let url = "https://dev.azure.com/aafc/_apis/projects?api-version=7.1";

        let token = fetch_azure_devops_personal_access_token().await?;
        let client = create_azure_devops_rest_client(&token).await?;

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
        let parsed = facet_json::from_str::<RawJson<'static>>(&content)?;
        assert!(parsed.as_str().trim_start().starts_with('{'));
        Ok(())
    }
}
