use crate::prelude::ResourceGraphHelper;
use crate::prelude::list_tracked_tenants;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::Subscription;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use eyre::bail;
use indoc::indoc;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct SubscriptionListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_subscriptions(tenant_id: AzureTenantId) -> SubscriptionListRequest {
    SubscriptionListRequest { tenant_id }
}

#[expect(async_fn_in_trait)]
pub trait SubscriptionIdExt {
    async fn resolve_tenant_id(&self) -> Result<AzureTenantId>;
}

impl SubscriptionIdExt for SubscriptionId {
    async fn resolve_tenant_id(&self) -> Result<AzureTenantId> {
        let tracked_tenants = list_tracked_tenants().await?;
        for tenant_id in tracked_tenants.iter().copied() {
            let Some(subscription) = fetch_all_subscriptions(tenant_id)
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
        )
    }
}

#[async_trait]
impl CacheableCommand for SubscriptionListRequest {
    type Output = Vec<Subscription>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "subscriptions",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
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

        let subscriptions = ResourceGraphHelper::new(query, Some(self.cache_key()))
            .tenant_id(self.tenant_id)
            .collect_all::<Subscription>()
            .await?;
        debug!("Found {} subscriptions", subscriptions.len());
        Ok(subscriptions)
    }
}

pub async fn get_active_subscription_id() -> Result<SubscriptionId> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "account",
        "list",
        "--query",
        "[?isDefault].id",
        "--output",
        "json",
    ]);
    let rtn = cmd.run::<[SubscriptionId; 1]>().await?[0];
    Ok(rtn)
}

cloud_terrastodon_command::impl_cacheable_into_future!(SubscriptionListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let result = fetch_all_subscriptions(tenant_id).await?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn resolves_tenant_for_subscription_id() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let subscription_id = fetch_all_subscriptions(tenant_id)
            .await?
            .first()
            .unwrap()
            .id;
        let resolved = subscription_id.resolve_tenant_id().await?;
        assert_eq!(resolved, tenant_id);
        Ok(())
    }

    #[tokio::test]
    pub async fn get_active() -> eyre::Result<()> {
        let active = get_active_subscription_id().await?;
        let active_text = active.to_string();
        assert!(!active_text.is_empty());
        Ok(())
    }
}
