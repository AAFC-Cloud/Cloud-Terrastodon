use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::ensure;
use facet::Facet;

/// Select and validate the account, tenant, and credential mechanism for Azure commands.
/// Resolution produces serializable request metadata; execution acquires credentials.
#[derive(Debug, Clone, Default, Facet, Arbitrary)]
pub struct AzureAuthenticationArgs {
    /// Tenant ID or tracked alias.
    /// Omit to use the selected account's tenant, or the configured default if no account is given.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Account ID or alias. Use --tenant to disambiguate aliases shared across tenants.
    /// Omit to use the configured account default for the selected tenant.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,

    /// Override the account's preferred authentication source; auto uses that preference.
    #[facet(figue::named, default)]
    pub auth_source: AuthSource,
}

impl AzureAuthenticationArgs {
    /// Resolve the explicit tenant, named account's tenant, or configured default.
    /// This works for commands that only need the tenant and do not authenticate.
    pub async fn resolve_tenant_id(
        &self,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<AzureTenantId> {
        if self.tenant.is_none()
            && let Some(account) = &self.account
        {
            return Ok(executor
                .resolve_azure_account_authentication(account, AuthSource::Auto)?
                .tenant_id);
        }
        self.tenant.resolve(executor.cancellation()).await
    }

    pub async fn resolve(
        &self,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<AzureBearerAuthenticationSelection> {
        ensure!(
            self.auth_source != AuthSource::None,
            "authentication is disabled by auth source none"
        );
        ensure!(
            self.auth_source != AuthSource::PersonalAccessToken,
            "Azure authentication does not support personal access tokens"
        );
        if self.tenant.is_none()
            && let Some(account) = &self.account
        {
            return executor.resolve_azure_account_authentication(account, self.auth_source);
        }
        let tenant_id = self.tenant.resolve(executor.cancellation()).await?;
        let account = self.account.as_ref().map(ToString::to_string);
        executor.resolve_azure_authentication(tenant_id, account.as_deref(), self.auth_source)
    }
}
