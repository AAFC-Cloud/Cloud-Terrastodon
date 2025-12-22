use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::ResourceGroup;
use cloud_terrastodon_command::{CacheBehaviour, HasCacheKey};
use eyre::Result;
use indoc::indoc;
use std::borrow::Cow;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct ResourceGroupsRequest;

impl HasCacheKey for ResourceGroupsRequest {
    fn cache_key<'a>(&'a self) -> Cow<'a, PathBuf> {
        Cow::Owned(PathBuf::from_iter([
            "az",
            "resource_graph",
            "resource_groups",
        ]))
    }
}
pub fn fetch_all_resource_groups() -> ResourceGroupsRequest {
    ResourceGroupsRequest
}
impl IntoFuture for ResourceGroupsRequest {
    type Output = Result<Vec<ResourceGroup>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            debug!("Fetching resource groups");
            let query = indoc! {r#"
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
            "#}
            .to_owned();

            let resource_groups = ResourceGraphHelper::new(
                query,
                CacheBehaviour::Some {
                    path: PathBuf::from("resource_groups"),
                    valid_for: Duration::from_hours(8),
                },
            )
            .collect_all::<ResourceGroup>()
            .await?;
            debug!("Found {} resource groups", resource_groups.len());
            Ok(resource_groups)
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
}
