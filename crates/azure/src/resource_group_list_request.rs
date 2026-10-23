use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::ResourceGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct ResourceGroupListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_resource_groups<'a>(
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> ResourceGroupListRequest<'a> {
    ResourceGroupListRequest {
        cancellation: cancellation.clone(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for ResourceGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for ResourceGroupListRequest<'a> {
    type Output = Vec<ResourceGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "resource_groups",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
    async fn run(self) -> Result<Self::Output> {
        ResourceGraphHelper::new(
            indoc! {r#"
                resourcecontainers
                | where type =~ "microsoft.resources/subscriptions/resourcegroups"
                | join kind=leftouter (
                    resourcecontainers
                    | where type =~ "Microsoft.Resources/subscriptions"
                    | project subscriptionId,subscription_name=name
                ) on $left.subscriptionId == $right.subscriptionId
                | project
                    id,
                    tenant_id=tenantId,
                    location,
                    managed_by=managedBy,
                    name,
                    properties,
                    tags,
                    subscription_name
            "#},
            Some(self.cache_key()),
            self.auth_context.as_ref(),
            &self.cancellation,
        )
        .collect_all::<ResourceGroup>()
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ResourceGroupListRequest<'a>, 'a);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_user_input::PickerTui;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let result = fetch_all_resource_groups(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &cancellation,
                )
                .await?;
                assert!(!result.is_empty());
                for rg in result {
                    assert!(!rg.name.is_empty());
                }
                Ok(())
            })
            .await
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn invalidation() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                fetch_all_resource_groups(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &cancellation,
                )
                .cache_key()
                .invalidate()
                .await?;
                Ok(())
            })
            .await
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn pick() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let chosen = PickerTui::<_>::new()
                    .pick_many_reloadable(|invalidate| {
                        let cancellation = cancellation.clone();
                        async move {
                            let tenant_id = get_test_tenant_id(&cancellation).await?;
                            if invalidate {
                                fetch_all_resource_groups(
                                    &AuthContext::explicit_azure_cli()
                                        .bind_to_azure_tenant(tenant_id)?,
                                    &cancellation,
                                )
                                .cache_key()
                                .invalidate()
                                .await?;
                            }
                            fetch_all_resource_groups(
                                &AuthContext::explicit_azure_cli()
                                    .bind_to_azure_tenant(tenant_id)?,
                                &cancellation,
                            )
                            .await
                        }
                    })
                    .await?;
                assert!(!chosen.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ResourceGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ResourceGroupListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ResourceGroupListRequest<'static> => Vec<ResourceGroup>, effects = [Read]);
