use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Facet)]
pub struct ApplicationRegistrationGetRequest<'a> {
    pub application_id: EntraApplicationClientId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ApplicationRegistrationGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            application_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_application_registration<'a>(
    application_id: EntraApplicationClientId,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ApplicationRegistrationGetRequest<'a>> {
    executor.execute(ApplicationRegistrationGetRequest {
        application_id,
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl ApplicationRegistrationGetRequest<'_> {
    fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/applications/{}",
            self.application_id
        )
    }
}

impl InvalidatesOwnCacheKey for ApplicationRegistrationGetRequest<'_> {}

impl HasCacheKey for ApplicationRegistrationGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            self.auth_context.tenant_id.to_string().as_str(),
            self.application_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ApplicationRegistrationGetRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<EntraApplicationRegistration>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.auth_context.tenant_id,
                application_registration_id = %self.application_id,
                "Fetching application registration by object id"
            );
            MicrosoftGraphHelper::new(
                self.url(),
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .fetch_one(&executor)
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(ApplicationRegistrationGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRegistrationGetRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ApplicationRegistrationGetRequest<'static> => Arc<EntraApplicationRegistration>, effects = [Read]);
