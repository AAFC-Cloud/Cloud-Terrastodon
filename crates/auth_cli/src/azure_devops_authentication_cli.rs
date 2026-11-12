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
/// Tenant-scoped Entra accounts and organization-scoped PAT accounts have
/// separate defaults. Resolving one never searches the other namespace.
#[derive(Debug, Clone, Default, Facet)]
pub struct AzureDevOpsAuthenticationArgs {
    /// Tenant ID or tracked alias for Entra authentication; omit for PAT authentication.
    /// Use `--tenant default` to select the configured Cloud Terrastodon tenant.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Account ID or alias within the selected tenant or organization.
    /// Omission uses that scope's explicitly configured account default.
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
                    matches!(
                        self.auth_source,
                        AuthSource::Auto | AuthSource::PersonalAccessToken
                    ),
                    "bearer authentication requires an explicit tenant; use --tenant default to select the configured tenant"
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
