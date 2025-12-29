use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::ResourceGroup;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

#[must_use = "This is a future request, you must .await it"]
pub struct ResourceGroupsRequest;

pub fn fetch_all_resource_groups() -> ResourceGroupsRequest {
    ResourceGroupsRequest
}

#[async_trait]
impl CacheableCommand for ResourceGroupsRequest {
    type Output = Vec<ResourceGroup>;

    fn cache_key<'a>(&'a self) -> Cow<'a, PathBuf> {
        Cow::Owned(PathBuf::from_iter([
            "az",
            "resource_graph",
            "resource_groups",
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
                CacheBehaviour::Some {
                    path: PathBuf::from_iter(["az", "resource_graph", "resource_groups"]),
                    valid_for: Duration::MAX,
                },
            )
            .collect_all::<ResourceGroup>()
            .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ResourceGroupsRequest);

#[cfg(test)]
mod tests {

    use super::*;
    use cloud_terrastodon_command::InvalidatableCache;
    use cloud_terrastodon_user_input::PickerTui;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let result = fetch_all_resource_groups().await?;
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
        fetch_all_resource_groups().invalidate_cache().await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn pick() -> Result<()> {
        let chosen = PickerTui::new()
            .pick_many_reloadable(async |invalidate| {
                if invalidate {
                    fetch_all_resource_groups().invalidate_cache().await?;
                }
                fetch_all_resource_groups().await
            })
            .await?;
        for rg in chosen {
            println!("{}", rg.id.expanded_form());
        }
        Ok(())
    }
}
