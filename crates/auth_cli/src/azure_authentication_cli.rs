use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::ensure;
use facet::Facet;

/// Select and validate the account, tenant, and credential mechanism for Azure commands.
/// Resolution produces serializable request metadata; execution acquires credentials.
#[derive(Debug, Clone, Default, Facet, Arbitrary)]
pub struct AzureAuthenticationArgs {
    /// Tenant ID or tracked alias; omission uses the configured tenant default.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Account ID or alias within the selected tenant; omission uses its configured default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,

    /// Override the account's preferred authentication source; auto uses that preference.
    #[facet(figue::named, default)]
    pub auth_source: AuthSource,
}

impl AzureAuthenticationArgs {
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
        let tenant_id = self.tenant.resolve(executor.cancellation()).await?;
        let account = self.account.as_ref().map(ToString::to_string);
        executor.resolve_azure_authentication(tenant_id, account.as_deref(), self.auth_source)
    }
}
