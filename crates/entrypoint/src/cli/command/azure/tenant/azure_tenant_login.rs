use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::resolve_tenant_auth_context;
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let auth_context = resolve_tenant_auth_context(auth_context, tenant_id).await?;
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
                let client_id = cloud_terrastodon_credentials::pim_client_id(&tenant_id).await?;
                login_browser_session(tenant_id, client_id).await?;
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
                cmd.run_raw().await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let auth_context = AuthContext::resolve(AuthSource::Auto)
            .expect("resolving automatic authentication should be local");
        if !auth_context.is_headless() {
            return;
        }
        let args = AzureTenantLoginArgs {
            tenant: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
        };
        let error = args
            .invoke(&auth_context)
            .await
            .expect_err("headless browser login should fail before opening a browser");
        assert!(
            error.to_string().contains("interactive terminal"),
            "unexpected headless tenant-login error: {error}"
        );
    }
}
