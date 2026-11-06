use crate::ResourceGraphHelper;
use crate::list_tracked_tenants;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_auth::read_azure_cli_local_accounts;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::Subscription;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::Result;
use eyre::bail;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct SubscriptionListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_subscriptions<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, SubscriptionListRequest<'a>> {
    executor.execute(SubscriptionListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for SubscriptionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[expect(async_fn_in_trait)]
pub trait SubscriptionIdExt {
    async fn resolve_tenant_id(
        &self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<AzureTenantId>;
}

impl SubscriptionIdExt for SubscriptionId {
    async fn resolve_tenant_id(
        &self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<AzureTenantId> {
        let tracked_tenants = list_tracked_tenants().await?;
        for tenant_id in tracked_tenants.iter().copied() {
            let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
            let Some(subscription) = fetch_all_subscriptions(&tenant_auth_context, executor)
                .await?
                .into_iter()
                .find(|subscription| subscription.id == *self)
            else {
                continue;
            };
            return Ok(subscription.tenant_id);
        }

        bail!(
            "Failed to resolve tracked tenant for subscription '{}' across {} tracked tenants.",
            self,
            tracked_tenants.len()
        );
    }
}

impl<'a> InvalidatesOwnCacheKey for SubscriptionListRequest<'a> {}

impl<'a> HasCacheKey for SubscriptionListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "subscriptions",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for SubscriptionListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<Subscription>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching subscriptions");
            let query = indoc! {r#"
        resourcecontainers
        | where type =~ "Microsoft.Resources/subscriptions"
        | project 
            name,
            id,
            tenant_id=tenantId,
            management_group_ancestors_chain=properties.managementGroupAncestorsChain,
            tags=tags
    "#};

            let subscriptions = ResourceGraphHelper::new(
                query,
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .collect_all::<Subscription>(&executor)
            .await?;
            debug!("Found {} subscriptions", subscriptions.len());
            Ok(subscriptions)
        })
    }
}

pub async fn get_active_subscription_id(
    cancellation: &CancellationScope,
) -> Result<SubscriptionId> {
    let mut accounts = read_azure_cli_local_accounts(cancellation)
        .await?
        .into_iter()
        .filter(|account| account.is_default);
    let account = accounts
        .next()
        .ok_or_else(|| eyre::eyre!("Azure CLI has no selected local subscription"))?;
    eyre::ensure!(
        accounts.next().is_none(),
        "Azure CLI has multiple selected local subscriptions"
    );
    Ok(account.id.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let result = fetch_all_subscriptions(
                    &auth_context.bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .await?;
                assert!(!result.is_empty());
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn resolves_tenant_for_subscription_id() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let subscription_id = fetch_all_subscriptions(
                    &auth_context.bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .await?
                .first()
                .unwrap()
                .id;
                let resolved = subscription_id
                    .resolve_tenant_id(&auth_context, &executor)
                    .await?;
                assert_eq!(resolved, tenant_id);
                Ok(())
            })
            .await
    }

    #[tokio::test]
    pub async fn get_active() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let active = get_active_subscription_id(&cancellation).await?;
                let active_text = active.to_string();
                assert!(!active_text.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(SubscriptionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(SubscriptionListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(SubscriptionListRequest<'static> => Vec<Arc<Subscription>>, effects = [Read]);
