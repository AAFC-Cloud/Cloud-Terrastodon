use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::MicrosoftGraphOrganization;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::ensure;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Facet)]
pub struct AzureTenantDetailsRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureTenantDetailsRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_azure_tenant_details<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureTenantDetailsRequest<'a>> {
    executor.execute(AzureTenantDetailsRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for AzureTenantDetailsRequest<'_> {}

impl HasCacheKey for AzureTenantDetailsRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "organization",
            self.auth_context.tenant_id.to_string().as_ref(),
        ]))
    }
}

impl ExecutableRequest for AzureTenantDetailsRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<MicrosoftGraphOrganization>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = "https://graph.microsoft.com/v1.0/organization";
            let resp = MicrosoftGraphHelper::new(
                url,
                Some(self.cache_key()?),
                self.auth_context.tenant_id,
                self.auth_context.auth_source()?,
            )
            .fetch_all::<MicrosoftGraphOrganization>(&executor)
            .await?;
            ensure!(
                resp.len() == 1,
                "Expected exactly one organization in response, got {}",
                resp.len()
            );
            resp.into_iter()
                .next()
                .ok_or_else(|| eyre::eyre!("Organization response is empty"))
        })
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_azure_tenant_details;
    use crate::list_tracked_tenants;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::ensure;
    use std::collections::HashSet;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenants = list_tracked_tenants().await?;
                ensure!(!tenants.is_empty(), "Expected at least one tracked tenant");
                let mut seen = HashSet::new();
                let auth_context = AuthContext::explicit_azure_cli();
                for tenant_id in tenants {
                    let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
                    let details =
                        fetch_azure_tenant_details(&tenant_auth_context, &executor).await?;
                    let unique = seen.insert(details.entity.id);
                    ensure!(unique, "Duplicate tenant ID found: {}", details.entity.id);
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(AzureTenantDetailsRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureTenantDetailsRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureTenantDetailsRequest<'static> => Arc<MicrosoftGraphOrganization>, effects = [Read]);
