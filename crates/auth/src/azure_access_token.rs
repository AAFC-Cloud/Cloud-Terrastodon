use crate::AuthContext;
use crate::AzureBearerToken;
use crate::CancellationScope;
use crate::browser_access_token::fetch_browser_access_token;
use crate::pim_client_id;
use crate::workload_identity::exchange_workload_identity_assertion;
use chrono::Local;
use chrono::TimeDelta;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use reqwest::Client;
use std::io::IsTerminal;

const TOKEN_REFRESH_BUFFER_SECONDS: i64 = 120;

/// Acquire a token through this invocation's isolated, explicitly tenant-bound CLI session.
pub async fn fetch_azure_access_token(
    auth_context: &AuthContext,
    tenant: AzureTenantId,
    resource: AzureCredentialResource,
    cancellation: &CancellationScope,
) -> eyre::Result<AzureAccessToken<AzureBearerToken>> {
    auth_context.admit(&AuthenticationSelection::AzureBearer {
        auth_source: AzureBearerAuthSource::AzureCli,
        tenant_id: tenant,
    })?;
    auth_context
        .azure_cli_sessions()
        .ok_or_else(|| eyre::eyre!("Azure CLI session is not configured"))?
        .fetch_bearer_access_token(tenant, resource, cancellation)
        .await
}

/// Acquire a bearer token for an explicit source and tenant, preserving the
/// invocation's admitted credential identity across audiences and refreshes.
pub async fn fetch_azure_bearer_access_token(
    auth_context: &AuthContext,
    tenant: AzureTenantId,
    resource: AzureCredentialResource,
    cancellation: &CancellationScope,
) -> eyre::Result<AzureAccessToken<AzureBearerToken>> {
    let source = match auth_context.source() {
        None => {
            eyre::bail!("the placeholder authentication context cannot acquire access tokens");
        }
        Some(AuthSource::None) => {
            eyre::bail!("authentication is disabled by auth source none");
        }
        Some(source) => AzureBearerAuthSource::try_from(source)?,
    };
    auth_context.admit(&AuthenticationSelection::AzureBearer {
        auth_source: source,
        tenant_id: tenant,
    })?;
    cancellation
        .cancellable_scope(|cancellation| async move {
            match source {
                AzureBearerAuthSource::WorkloadIdentity => {
                    let config = auth_context.workload_identity().ok_or_else(|| {
                        eyre::eyre!("workload identity credentials are not configured")
                    })?;
                    let mut cache = auth_context
                        .workload_identity_token_cache()
                        .ok_or_else(|| {
                            eyre::eyre!("workload identity token cache is not configured")
                        })?
                        .lock()
                        .await;
                    if let Some(token) = cache.get(&(tenant, resource))
                        && is_token_fresh(token, Local::now())
                    {
                        return Ok(token.clone());
                    }
                    let (token, expires_in) = exchange_workload_identity_assertion(
                        &Client::new(),
                        config.token_endpoint()?,
                        config,
                        resource,
                        &cancellation,
                    )
                    .await?;
                    let lifetime = i64::try_from(expires_in)
                        .ok()
                        .and_then(TimeDelta::try_seconds)
                        .ok_or_else(|| {
                            eyre::eyre!("workload identity token lifetime is out of range")
                        })?;
                    let expires_on =
                        Local::now().checked_add_signed(lifetime).ok_or_else(|| {
                            eyre::eyre!("workload identity token expiry is out of range")
                        })?;
                    let token = AzureAccessToken {
                        access_token: AzureBearerToken::new(token),
                        expires_on,
                        subscription: None,
                        tenant,
                        token_type: cloud_terrastodon_azure_types::TokenType::Bearer,
                    };
                    cache.insert((tenant, resource), token.clone());
                    Ok(token)
                }
                AzureBearerAuthSource::AzureCli => {
                    fetch_azure_access_token(auth_context, tenant, resource, &cancellation).await
                }
                AzureBearerAuthSource::Browser => {
                    let client_id = if let Some(session) = auth_context.browser_session() {
                        session.client_id
                    } else {
                        pim_client_id(&tenant, &cancellation).await?
                    };
                    fetch_browser_access_token(
                        auth_context
                            .browser_token_cache()
                            .ok_or_else(|| eyre::eyre!("browser token cache is not configured"))?,
                        tenant,
                        client_id,
                        resource,
                        auth_context.allows_interactive_browser_login(),
                        &cancellation,
                    )
                    .await
                }
            }
        })
        .await
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

/// Acquire Azure DevOps bearer authorization with an explicit tenant and source.
/// This operation does not acquire a personal access token.
pub async fn fetch_azure_devops_bearer_access_token(
    auth_context: &AuthContext,
    tenant: AzureTenantId,
    cancellation: &CancellationScope,
) -> eyre::Result<AzureAccessToken<AzureBearerToken>> {
    fetch_azure_bearer_access_token(
        auth_context,
        tenant,
        AzureCredentialResource::AzureDevOps,
        cancellation,
    )
    .await
}

#[cfg(test)]
#[allow(
    clippy::disallowed_types,
    reason = "These tests inspect raw JSON returned by the authentication transport."
)]
mod test {
    use crate::AuthContext;
    use crate::AzureBearerToken;
    use crate::azure_access_token::TOKEN_REFRESH_BUFFER_SECONDS;
    use crate::azure_access_token::fetch_azure_devops_bearer_access_token;
    use crate::create_azure_devops_rest_client;
    use chrono::Local;
    use chrono::TimeDelta;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_azure_types::AzureAccessToken;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_azure_types::TokenType;
    use cloud_terrastodon_command::CancellationToken;
    use facet_json::RawJson;

    #[test]
    fn explicit_context_is_used_for_dispatch() -> eyre::Result<()> {
        let context = AuthContext::explicit(AuthSource::WorkloadIdentity);
        assert_eq!(context.source(), Some(AuthSource::WorkloadIdentity));
        Ok(())
    }

    #[tokio::test]
    async fn headless_cli_auth_fails_before_spawning_azure_cli() {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let context = AuthContext::explicit_headless(AuthSource::AzureCli);
                let tenant = "11111111-1111-1111-1111-111111111111".parse().unwrap();
                let error = context
                    .admit(
                        &cloud_terrastodon_auth_types::AuthenticationSelection::AzureBearer {
                            auth_source:
                                cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                            tenant_id: tenant,
                        },
                    )
                    .expect_err("headless admission must fail before any reusable result lookup");
                assert!(error.to_string().contains("headless"));
                let error = super::fetch_azure_bearer_access_token(
                    &context,
                    tenant,
                    cloud_terrastodon_auth_types::AzureCredentialResource::AzureDevOps,
                    &cancellation,
                )
                .await
                .expect_err("headless CLI authentication should fail locally");
                assert!(error.to_string().contains("headless"));
                Ok::<_, eyre::Error>(())
            })
            .await
            .expect("test execution scope remains valid");
    }

    #[tokio::test]
    async fn placeholder_context_fails_before_token_acquisition() {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let error = super::fetch_azure_bearer_access_token(
                    &AuthContext::Unresolved,
                    "11111111-1111-1111-1111-111111111111".parse().unwrap(),
                    cloud_terrastodon_auth_types::AzureCredentialResource::AzureDevOps,
                    &cancellation,
                )
                .await
                .expect_err("placeholder authentication must not execute");
                assert!(error.to_string().contains("placeholder"));
                Ok::<_, eyre::Error>(())
            })
            .await
            .expect("test execution scope remains valid");
    }

    #[tokio::test]
    async fn disabled_authentication_fails_before_token_acquisition() {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let context = AuthContext::resolve(AuthSource::None)?;
                let error = super::fetch_azure_bearer_access_token(
                    &context,
                    "11111111-1111-1111-1111-111111111111".parse().unwrap(),
                    cloud_terrastodon_auth_types::AzureCredentialResource::MicrosoftGraph,
                    &cancellation,
                )
                .await
                .expect_err("disabled authentication must not acquire a token");
                assert!(error.to_string().contains("auth source none"));
                Ok::<_, eyre::Error>(())
            })
            .await
            .expect("test execution scope remains valid");
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
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                // this endpoint only works with the `az account get-access-token`, not with a raw PAT
                let url =
                    "https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=6.0";

                let token = fetch_azure_devops_bearer_access_token(
                    &AuthContext::explicit_azure_cli(),
                    "11111111-1111-1111-1111-111111111111".parse()?,
                    &cancellation,
                )
                .await?;
                let client = create_azure_devops_rest_client(&token).await?;

                let resp = client.get(url).send().await?;
                let status = resp.status();

                let content = resp.text().await?;

                assert_eq!(200, status.as_u16(), "{:?}", status.canonical_reason());
                let parsed = facet_json::from_str::<RawJson<'static>>(&content)?;
                assert!(parsed.as_str().trim_start().starts_with('{'));
                Ok(())
            })
            .await
    }

    #[tokio::test]
    #[ignore] // todo: remove, deprecate in favour of get_azure_devops_personal_access_token_from_credential_manager
    pub async fn it_works_projects() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                // this endpoint only works with the `az account get-access-token`, not with a raw PAT
                let url = "https://dev.azure.com/aafc/_apis/projects?api-version=7.1";

                let token = fetch_azure_devops_bearer_access_token(
                    &AuthContext::explicit_azure_cli(),
                    "11111111-1111-1111-1111-111111111111".parse()?,
                    &cancellation,
                )
                .await?;
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
            })
            .await
    }
}
