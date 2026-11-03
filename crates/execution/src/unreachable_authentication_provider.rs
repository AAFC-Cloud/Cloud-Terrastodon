use crate::AuthenticationProvider;
use crate::CloudTerrastodonExecutor;
use crate::ExecutionFuture;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_cache::AuthPartition;
use eyre::Result;

/// Test guard for execution that must never consult authentication.
///
/// Available to downstream tests through the `test-support` feature. Both
/// admission and authorization panic immediately, including for anonymous
/// selections. Authorization panics when called, before returning a future,
/// so an ignored result or unpolled future cannot hide an unexpected call.
/// Use [`crate::AnonymousAuthenticationProvider`] when anonymous admission is
/// expected instead.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnreachableAuthenticationProvider;

impl AuthenticationProvider for UnreachableAuthenticationProvider {
    fn admit(&self, _selection: &AuthenticationSelection) -> Result<AuthPartition> {
        panic!("authentication admission must not be reached in this test")
    }

    fn authorization<'a>(
        &'a self,
        _selection: &'a AuthenticationSelection,
        _resource: AzureCredentialResource,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        panic!("authorization must not be reached in this test")
    }
}
