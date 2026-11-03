use crate::AuthenticationProvider;
use crate::CloudTerrastodonExecutor;
use crate::ExecutionFuture;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_cache::AuthPartition;
use eyre::Result;
use eyre::ensure;

/// Anonymous-only execution without a runtime credential dependency or discovery.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnonymousAuthenticationProvider;

impl AuthenticationProvider for AnonymousAuthenticationProvider {
    fn admit(&self, selection: &AuthenticationSelection) -> Result<AuthPartition> {
        ensure!(
            matches!(selection, AuthenticationSelection::Anonymous),
            "this provider permits only anonymous authentication"
        );
        AuthPartition::new("anonymous")
    }

    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        _resource: AzureCredentialResource,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            self.admit(selection)?;
            Ok(Authorization::Anonymous)
        })
    }
}
