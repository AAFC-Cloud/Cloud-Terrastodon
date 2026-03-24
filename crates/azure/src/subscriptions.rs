use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::Subscription;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
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
    use crate::prelude::get_default_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let tenant_id = get_default_tenant_id().await?;
        let result = fetch_all_subscriptions(tenant_id).await?;
        println!("Found {} subscriptions:", result.len());
        for sub in result {
            println!(
                "- {} ({}) under {}",
                sub.name,
                sub.id,
                sub.management_group_ancestors_chain.first().unwrap().name
            );
        }
        Ok(())
    }

    #[tokio::test]
    pub async fn get_active() -> eyre::Result<()> {
        println!("{}", get_active_subscription_id().await?);
        Ok(())
    }
}
