use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::ResourceGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct ResourceGroupListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_resource_groups(tenant_id: AzureTenantId) -> ResourceGroupListRequest {
    ResourceGroupListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ResourceGroupListRequest {
    type Output = Vec<ResourceGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "resource_groups",
            self.tenant_id.to_string().as_str(),
        ]))
    }
    async fn run(self) -> Result<Self::Output> {
        ResourceGraphHelper::new(
            indoc! {r#"
                    resourcecontainers
                    | where type =~ "microsoft.resources/subscriptions/resourcegroups"
                    | project
                        id,
                        location,
                        managed_by=managedBy,
                        name,
                        properties,
                        tags,
                        subscription_id=subscriptionId
                "#},
            Some(self.cache_key()),
        )
        .tenant_id(self.tenant_id)
        .collect_all::<ResourceGroup>()
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ResourceGroupListRequest);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::prelude::get_test_tenant_id;
    use cloud_terrastodon_azure_types::prelude::Scope;
    use cloud_terrastodon_user_input::PickerTui;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let result = fetch_all_resource_groups(tenant_id).await?;
        assert!(!result.is_empty());
        println!("Found {} resource groups:", result.len());
        for rg in result {
            assert!(!rg.name.is_empty());
            println!(" - {} (sub={})", rg.name, rg.subscription_id);
        }
        Ok(())
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn invalidation() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        fetch_all_resource_groups(tenant_id)
            .cache_key()
            .invalidate()
            .await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn pick() -> Result<()> {
        let chosen = PickerTui::new()
            .pick_many_reloadable(async |invalidate| {
                let tenant_id = get_test_tenant_id().await?;
                if invalidate {
                    fetch_all_resource_groups(tenant_id)
                        .cache_key()
                        .invalidate()
                        .await?;
                }
                fetch_all_resource_groups(tenant_id).await
            })
            .await?;
        for rg in chosen {
            println!("{}", rg.id.expanded_form());
        }
        Ok(())
    }
}
