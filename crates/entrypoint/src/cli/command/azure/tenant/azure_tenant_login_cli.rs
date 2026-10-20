use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::resolve_tenant_auth_context;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AuthSource;
use cloud_terrastodon_credentials::login_browser_session;
use eyre::Result;
use eyre::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TenantLoginMode {
    Browser,
    AzureCli,
}

fn cli_reauthentication_denied(value: Option<&str>) -> bool {
    value.is_some_and(|value| value.eq_ignore_ascii_case("DENY"))
}

fn tenant_login_mode(source: AuthSource) -> Result<TenantLoginMode> {
    match source {
        AuthSource::None => bail!("tenant login is unavailable while authentication is disabled"),
        AuthSource::Auto | AuthSource::Browser => Ok(TenantLoginMode::Browser),
        AuthSource::AzureCli => Ok(TenantLoginMode::AzureCli),
        AuthSource::WorkloadIdentity => bail!(
            "tenant login is not applicable to workload identity; use the pipeline service connection directly"
        ),
        AuthSource::PersonalAccessToken => bail!(
            "tenant login is not applicable to an Azure DevOps personal access token; choose browser or azure-cli authentication"
        ),
    }
}

/// Arguments for logging in to an Azure tenant. Browser PKCE is the default;
/// Azure CLI remains available with `--auth-source azure-cli`.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantLoginArgs {
    /// Tracked tenant id or alias to log in to.
    #[facet(figue::positional)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureTenantLoginArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        eyre::ensure!(
            auth_context.source() != Some(AuthSource::None),
            "authentication is disabled by auth source none; cannot resolve a tenant for login"
        );
        let tenant_id = self.tenant.resolve(cancellation).await?;
        let auth_context =
            resolve_tenant_auth_context(auth_context, tenant_id, cancellation).await?;
        login_resolved_tenant(tenant_id, &auth_context, cancellation).await
    }
}

/// Dispatch after tracked-tenant resolution, so local policy tests need not
/// read or create entries in the user's tenant store.
async fn login_resolved_tenant(
    tenant_id: cloud_terrastodon_azure::AzureTenantId,
    auth_context: &AuthContext,
    cancellation: &CancellationScope,
) -> Result<()> {
    let requested_source = auth_context.requested_source().ok_or_else(|| {
        eyre::eyre!("the placeholder authentication context cannot be used for tenant login")
    })?;
    match tenant_login_mode(requested_source)? {
        TenantLoginMode::Browser => {
            if auth_context.is_headless() {
                bail!(
                    "browser tenant login requires an interactive terminal; use workload identity in pipelines"
                );
            }
            let client_id =
                cloud_terrastodon_credentials::pim_client_id(&tenant_id, cancellation).await?;
            login_browser_session(tenant_id, client_id, cancellation).await?;
            Ok(())
        }
        TenantLoginMode::AzureCli => {
            // This legacy guard controls implicit/compatibility CLI
            // reauthentication. An explicit browser login must remain
            // usable even when the environment disables CLI reauth.
            if cli_reauthentication_denied(
                std::env::var("CLOUD_TERRASTODON_REAUTH").ok().as_deref(),
            ) {
                bail!(
                    "Reauthentication is disabled by the CLOUD_TERRASTODON_REAUTH environment variable. Please refresh your credentials and try again."
                )
            }
            let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
            cmd.args(["login", "--tenant", &tenant_id.to_string()]);
            cmd.should_announce(true);
            cmd.run_raw(cancellation).await?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn disabled_authentication_rejects_login_before_default_tenant_lookup() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth = AuthContext::resolve(AuthSource::None)?;
                // Prevent external command admission even if the auth-policy
                // guard regresses and reaches default-tenant lookup.
                cancellation.request_cancel("external lookup is forbidden in this test");
                let error = AzureTenantLoginArgs {
                    tenant: AzureTenantArgument::Default,
                }
                .invoke(&auth, &cancellation)
                .await
                .expect_err("disabled authentication must reject tenant login locally");
                assert!(error.to_string().contains("auth source none"));
                Ok(())
            })
            .await
    }

    #[test]
    fn auto_and_browser_tenant_login_use_pkce() {
        assert_eq!(
            tenant_login_mode(AuthSource::Auto).unwrap(),
            TenantLoginMode::Browser
        );
        assert_eq!(
            tenant_login_mode(AuthSource::Browser).unwrap(),
            TenantLoginMode::Browser
        );
    }

    #[test]
    fn non_interactive_sources_do_not_fall_back_to_azure_cli_login() {
        assert_eq!(
            tenant_login_mode(AuthSource::AzureCli).unwrap(),
            TenantLoginMode::AzureCli
        );
        assert!(tenant_login_mode(AuthSource::None).is_err());
        assert!(tenant_login_mode(AuthSource::WorkloadIdentity).is_err());
        assert!(tenant_login_mode(AuthSource::PersonalAccessToken).is_err());
    }

    #[test]
    fn reauthentication_guard_only_matches_deny_value() {
        assert!(cli_reauthentication_denied(Some("deny")));
        assert!(cli_reauthentication_denied(Some("DENY")));
        assert!(!cli_reauthentication_denied(Some("allow")));
        assert!(!cli_reauthentication_denied(None));
    }

    #[tokio::test]
    async fn auto_tenant_login_rejects_headless_browser_dispatch_before_network() {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                // No environment, credential cache, or tracked-tenant prerequisite.
                let mut auth_context = AuthContext::explicit(AuthSource::Auto);
                if let AuthContext::Resolved { headless, .. } = &mut auth_context {
                    *headless = true;
                }
                let tenant = "11111111-1111-1111-1111-111111111111".parse().unwrap();
                let error = login_resolved_tenant(tenant, &auth_context, &cancellation)
                    .await
                    .expect_err("headless browser login should fail before opening a browser");
                assert!(
                    error.to_string().contains("interactive terminal"),
                    "unexpected headless tenant-login error: {error}"
                );

                Ok::<_, eyre::Report>(())
            })
            .await
            .unwrap();
    }
}
