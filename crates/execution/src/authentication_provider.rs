use crate::CloudTerrastodonExecutor;
use crate::ExecutionFuture;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_cache::AuthPartition;
use eyre::Result;

/// An already-configured credential session injected by application startup.
///
/// Admission uses owned policy and identity only: no token acquisition,
/// configuration discovery, or ambient account resolution. Clones of a runtime
/// provider must preserve the admitted identity; independent sessions must not
/// silently share its partition. Credential bytes never form cache identities.
pub trait AuthenticationProvider: Send + Sync {
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
}
