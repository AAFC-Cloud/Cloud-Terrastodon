use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraApplicationObjectId;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Read an application registration by its directory object ID.
/// [Microsoft Graph Get application](https://learn.microsoft.com/en-us/graph/api/application-get?view=graph-rest-1.0)
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraApplicationRegistrationGetRequest {
    /// Directory object ID used by the `/applications/{applicationObjectId}` route.
    pub application_object_id: EntraApplicationObjectId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl EntraApplicationRegistrationGetRequest {
    fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/applications/{}",
            self.application_object_id
        )
    }
}

impl InvalidatesOwnCacheKey for EntraApplicationRegistrationGetRequest {}

impl HasCacheKey for EntraApplicationRegistrationGetRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            self.authentication.tenant_id.to_string().as_str(),
            self.application_object_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraApplicationRegistrationGetRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<EntraApplicationRegistration>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.authentication.tenant_id,
                application_object_id = %self.application_object_id,
                "Fetching application registration by object ID"
            );
            MicrosoftGraphHelper::new(self.url(), Some(self.cache_key()?), self.authentication)
                .fetch_one(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraApplicationRegistrationGetRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationRegistrationGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraApplicationRegistrationGetRequest => Arc<EntraApplicationRegistration>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraApplicationRegistrationGetRequest,
    name = "application-registration-get",
    cached
);
