use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
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

#[must_use = "This is a future request, you must .execute it"]
#[derive(Facet)]
pub struct EntraApplicationRegistrationListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraApplicationRegistrationListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for EntraApplicationRegistrationListRequest {}

impl HasCacheKey for EntraApplicationRegistrationListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraApplicationRegistrationListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraApplicationRegistration>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(tenant_id = %self.authentication.tenant_id, "Fetching application registrations");
            let applications: Vec<Arc<EntraApplicationRegistration>> = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/v1.0/applications",
                Some(self.cache_key()?),
                self.authentication,
            )
            .fetch_all(&executor)
            .await?;
            debug!(
                tenant_id = %self.authentication.tenant_id,
                count = applications.len(),
                "Found application registrations"
            );
            Ok(applications)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraApplicationRegistrationListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationRegistrationListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraApplicationRegistrationListRequest => Vec<Arc<EntraApplicationRegistration>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraApplicationRegistrationListRequest,
    name = "application-registration-list",
    cached
);
