use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureAppServiceResource;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AppServiceListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_app_services<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AppServiceListRequest<'a>> {
    executor.execute(AppServiceListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for AppServiceListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for AppServiceListRequest<'a> {}

impl<'a> HasCacheKey for AppServiceListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "app_services",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for AppServiceListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureAppServiceResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!(%self.auth_context.tenant_id, "Fetching app services");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.web/sites"
        | project
            id,
            tenantId,
            name,
            kind,
            location,
            tags,
            properties
        "#}
            .to_owned();

            let app_services = ResourceGraphHelper::new(
                query,
                Some(self.cache_key()?),
                self.auth_context.tenant_id,
                self.auth_context.auth_source()?,
            )
            .collect_all::<AzureAppServiceResource>(&executor)
            .await?;
            info!(count = app_services.len(), "Fetched app services");
            Ok(app_services)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_registry::RuntimeValue;
    use eyre::Result;

    #[test]
    fn reflected_app_service_list_clone_retains_its_owned_tenant() -> Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        let original = RuntimeValue::from_box(Box::new(AppServiceListRequest {
            auth_context: Cow::Owned(AzureTenantAuthContext {
                auth_context: AuthContext::Unresolved,
                tenant_id,
            }),
        }))?;
        let cloned = original
            .try_clone()?
            .into_box::<AppServiceListRequest<'static>>()?
            .downcast::<AppServiceListRequest<'static>>()
            .expect("reflected cloning preserves the request type");
        drop(original);
        assert_eq!(cloned.auth_context.tenant_id, tenant_id);
        assert!(matches!(
            cloned.auth_context.as_ref().auth_context,
            AuthContext::Unresolved
        ));
        assert!(matches!(cloned.auth_context, Cow::Owned(_)));
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let result = fetch_all_app_services(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
                )
                .await?;
                for app_service in &result {
                    assert!(!app_service.name.is_empty());
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(AppServiceListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AppServiceListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AppServiceListRequest<'static> => Vec<Arc<AzureAppServiceResource>>, effects = [Read]);
