use crate::CloudTerrastodonExecutor;
use crate::ExecutionFuture;
use crate::GiteaCommandSession;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_auth_types::ContainerRegistryBearerChallenge;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::WorkCache;
use eyre::Result;
use std::sync::Arc;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use url::Url;

/// Application-owned authentication policy and credential sessions.
///
/// Selector resolution reads account metadata. Admission may load the explicitly
/// selected local credential session, but never acquires tokens or selects an
/// ambient account. Clones of a runtime
/// provider must preserve the admitted identity; independent sessions must not
/// silently share its partition. Credential bytes never form cache identities.
///
/// Applications may implement and inject this trait, but its selections,
/// audiences, and the built-in account catalogue currently describe the supported
/// Azure/Entra, Azure DevOps, and Gitea services. This is not an open service
/// registration API; adding another account service requires extending those
/// contracts, not disguising its credentials as an existing selection.
pub trait AuthenticationProvider: Send + Sync {
    /// Close admission and drain retained work before credentials are removed.
    /// A source selects one mechanism; omission retires the entire account.
    fn retire_account<'a>(
        &'a self,
        _account_id: CloudTerrastodonAccountId,
        _source: Option<CloudTerrastodonCredentialSource>,
        _cache: &'a WorkCache,
        _cancellation: &'a CancellationScope,
    ) -> ExecutionFuture<'a, ()> {
        Box::pin(async {
            eyre::bail!("this authentication provider cannot retire accounts");
        })
    }

    /// Resolve human selectors to reviewable metadata without loading credentials.
    /// Providers injected by tests or restricted applications may reject discovery.
    fn resolve_azure_authentication(
        &self,
        _tenant_id: AzureTenantId,
        _account: Option<&str>,
        _source: AuthSource,
    ) -> Result<AzureBearerAuthenticationSelection> {
        eyre::bail!("this authentication provider does not resolve account selectors");
    }

    /// Resolve the explicit organization/tenant namespace before SDK execution.
    fn resolve_azure_devops_authentication(
        &self,
        _organization: &AzureDevOpsOrganizationUrl,
        _tenant_id: Option<AzureTenantId>,
        _account: Option<&str>,
        _source: AuthSource,
    ) -> Result<AzureDevOpsAuthSelection> {
        eyre::bail!("this authentication provider does not resolve account selectors");
    }

    /// Select one configured account belonging to the requested Gitea instance.
    fn resolve_gitea_authentication(
        &self,
        _instance: &str,
        _account: Option<&str>,
    ) -> Result<CloudTerrastodonAccountId> {
        eyre::bail!("this authentication provider does not resolve Gitea accounts");
    }
    /// Cancellation signals for this execution's retained account leases. They
    /// cancel optional work while linked deferrals retain required completion.
    fn retirement_tokens(&self) -> Vec<CancellationToken> {
        Vec::new()
    }

    /// Verify a request destination against the selected account's service scope
    /// before cached results or credentials are used. Pure Entra accounts are
    /// tenant-scoped; providers owning service credentials enforce their instance.
    fn validate_target(&self, selection: &AuthenticationSelection, _target: &Url) -> Result<()> {
        self.admit(selection)?;
        Ok(())
    }
    /// Retain an admitted credential binding through the complete SDK request,
    /// including required response/artifact handling and affected-read invalidation.
    /// Providers without revocable account state need no additional lease.
    fn lease(
        &self,
        selection: &AuthenticationSelection,
    ) -> Result<Option<Arc<dyn AuthenticationProvider>>> {
        self.admit(selection)?;
        Ok(None)
    }

    /// Resolve an already admitted Gitea account's exact tea profile. The selected
    /// instance must match the account binding; ambient tea defaults are irrelevant.
    fn gitea_session(
        &self,
        _account_id: CloudTerrastodonAccountId,
        _instance: &str,
    ) -> Result<Arc<GiteaCommandSession>> {
        eyre::bail!("this authentication provider does not admit Gitea accounts");
    }
    /// Admit the explicit selection before any shared result can be reused.
    fn admit(&self, selection: &AuthenticationSelection) -> Result<AuthPartition>;

    /// Acquire sensitive authorization only when execution actually needs it.
    ///
    /// Use this executor's bounded scope for credential infrastructure. Keep
    /// tokens and credential responses out of the general work/artifact cache.
    /// Implementations must not recursively resolve authorization through it.
    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        resource: AzureCredentialResource,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization>;

    /// Exchange a validated registry challenge using this provider's admitted
    /// bearer identity. Credential forms and responses never enter work artifacts.
    /// Providers without registry exchange support reject this capability.
    fn container_registry_authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        _challenge: &'a ContainerRegistryBearerChallenge,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            self.admit(selection)?;
            eyre::bail!(
                "this authentication provider does not support Container Registry exchange"
            );
        })
    }
}
