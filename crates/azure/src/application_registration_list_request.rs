use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
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
pub struct ApplicationRegistrationListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ApplicationRegistrationListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_application_registrations<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ApplicationRegistrationListRequest<'a>> {
    executor.execute(ApplicationRegistrationListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for ApplicationRegistrationListRequest<'_> {}

impl HasCacheKey for ApplicationRegistrationListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ApplicationRegistrationListRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraApplicationRegistration>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(tenant_id = %self.auth_context.tenant_id, "Fetching application registrations");
            let applications: Vec<Arc<EntraApplicationRegistration>> = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/v1.0/applications",
                Some(self.cache_key()?),
                self.auth_context.tenant_id,
                self.auth_context.auth_source()?,
            )
            .fetch_all(&executor)
            .await?;
            debug!(
                tenant_id = %self.auth_context.tenant_id,
                count = applications.len(),
                "Found application registrations"
            );
            Ok(applications)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::Result;

    #[tokio::test]
    async fn list_application_registrations() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let auth_context =
                    AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?;
                let result = fetch_all_application_registrations(&auth_context, &executor).await?;
                assert!(!result.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ApplicationRegistrationListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRegistrationListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ApplicationRegistrationListRequest<'static> => Vec<Arc<EntraApplicationRegistration>>, effects = [Read]);
