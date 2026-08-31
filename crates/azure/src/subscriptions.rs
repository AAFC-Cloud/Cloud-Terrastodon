use crate::ResourceGraphHelper;
use crate::list_tracked_tenants;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::Subscription;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::bail;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct SubscriptionListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_subscriptions<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> SubscriptionListRequest<'a> {
    SubscriptionListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for SubscriptionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[expect(async_fn_in_trait)]
pub trait SubscriptionIdExt {
    async fn resolve_tenant_id(&self, auth_context: &AuthContext) -> Result<AzureTenantId>;
}

impl SubscriptionIdExt for SubscriptionId {
    async fn resolve_tenant_id(&self, auth_context: &AuthContext) -> Result<AzureTenantId> {
        let tracked_tenants = list_tracked_tenants().await?;
        for tenant_id in tracked_tenants.iter().copied() {
            let Some(subscription) = fetch_all_subscriptions(tenant_id, auth_context)
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
impl<'a> CacheableCommand for SubscriptionListRequest<'a> {
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

        let subscriptions = ResourceGraphHelper::new(
            self.tenant_id,
            query,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
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

cloud_terrastodon_command::impl_cacheable_into_future!(SubscriptionListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let auth_context = AuthContext::default();
        let result = fetch_all_subscriptions(tenant_id, &auth_context).await?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn resolves_tenant_for_subscription_id() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let auth_context = AuthContext::default();
        let subscription_id = fetch_all_subscriptions(tenant_id, &auth_context)
            .await?
            .first()
            .unwrap()
            .id;
        let resolved = subscription_id.resolve_tenant_id(&auth_context).await?;
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

cloud_terrastodon_registry::register_thing!(SubscriptionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(SubscriptionListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(SubscriptionListRequest<'static> => Vec<Subscription>);
