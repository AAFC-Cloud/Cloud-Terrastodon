use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::ensure;
use facet::Facet;

/// Resolve CLI authentication preferences to an explicit request identity.
///
/// Entra accounts are scoped to tenants, and PAT accounts to organizations.
/// An explicit account can supply its tenant when the tenant flag is omitted.
#[derive(Debug, Clone, Default, Facet)]
pub struct AzureDevOpsAuthenticationArgs {
    /// Restrict Entra authentication to this tenant ID or tracked alias.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Account ID or alias. Use --tenant or an account ID to disambiguate shared aliases.
    /// Omission uses the explicitly configured default for the selected tenant or organization.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,

    /// Override the account's preferred authentication source for this invocation.
    /// Auto uses that preference; unavailable mechanisms never fall back silently.
    #[facet(figue::named, default)]
    pub auth_source: AuthSource,
}

impl AzureDevOpsAuthenticationArgs {
    /// Resolve non-secret account metadata without loading credentials or
    /// changing the executor. The request retains the resolved identity.
    pub async fn resolve(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<AzureDevOpsAuthSelection> {
        ensure!(
            self.auth_source != AuthSource::None,
            "authentication is disabled by auth source none"
        );
        let tenant_id = match &self.tenant {
            Some(tenant) => {
                ensure!(
                    self.auth_source != AuthSource::PersonalAccessToken,
                    "PAT authentication cannot be combined with --tenant"
                );
                Some(tenant.resolve(executor.cancellation()).await?)
            }
            None => {
                ensure!(
                    self.account.is_some()
                        || matches!(
                            self.auth_source,
                            AuthSource::Auto | AuthSource::PersonalAccessToken
                        ),
                    "bearer authentication requires an explicit tenant or account"
                );
                None
            }
        };
        let account = self.account.as_ref().map(ToString::to_string);
        executor.resolve_azure_devops_authentication(
            organization,
            tenant_id,
            account.as_deref(),
            self.auth_source,
        )
    }
}
