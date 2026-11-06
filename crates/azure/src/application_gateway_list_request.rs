use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureApplicationGatewayResource;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct ApplicationGatewayListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_application_gateways<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ApplicationGatewayListRequest<'a>> {
    executor.execute(ApplicationGatewayListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for ApplicationGatewayListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for ApplicationGatewayListRequest<'a> {}

impl<'a> HasCacheKey for ApplicationGatewayListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "application_gateways",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for ApplicationGatewayListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureApplicationGatewayResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(%self.auth_context.tenant_id, "Fetching application gateways");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/applicationgateways"
        | project
            id,
            tenantId,
            name,
            location,
            tags,
            identity,
            properties
        "#}
            .to_owned();

            let application_gateways = ResourceGraphHelper::new(
                query,
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .collect_all::<AzureApplicationGatewayResource>(&executor)
            .await?;
            debug!(
                count = application_gateways.len(),
                "Fetched application gateways"
            );
            Ok(application_gateways)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let result = fetch_all_application_gateways(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
                )
                .await?;
                for application_gateway in &result {
                    assert!(!application_gateway.name.is_empty());
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ApplicationGatewayListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ApplicationGatewayListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ApplicationGatewayListRequest<'static> => Vec<Arc<AzureApplicationGatewayResource>>, effects = [Read]);
