use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureAppServiceResource;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AppServiceListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_app_services<'a>(
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> AppServiceListRequest<'a> {
    AppServiceListRequest {
        cancellation: cancellation.clone(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for AppServiceListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for AppServiceListRequest<'a> {
    type Output = Vec<AzureAppServiceResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "app_services",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
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
            Some(self.cache_key()),
            self.auth_context.as_ref(),
            &self.cancellation,
        )
        .collect_all::<AzureAppServiceResource>()
        .await?;
        info!(count = app_services.len(), "Fetched app services");
        Ok(app_services)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AppServiceListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_registry::RuntimeValue;

    #[test]
    fn reflected_app_service_list_clone_retains_its_owned_tenant() -> Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        let original = RuntimeValue::from_box(Box::new(AppServiceListRequest {
            cancellation: CancellationScope::unbound(),
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
                let result = fetch_all_app_services(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &cancellation,
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
cloud_terrastodon_registry::register_into_future!(AppServiceListRequest<'static> => Vec<AzureAppServiceResource>, effects = [Read]);
