use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::RouteTable;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;
use cloud_terrastodon_command::async_trait;

#[must_use = "This is a future request, you must .await it"]
pub struct RouteTableListRequest;

pub fn fetch_all_route_tables() -> RouteTableListRequest {
    RouteTableListRequest
}

#[async_trait]
impl CacheableCommand for RouteTableListRequest {
    type Output = Vec<RouteTable>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter(["az", "resource_graph", "route_tables"]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!("Fetching route tables");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/routetables"
        | project
            id,
            name,
            location,
            resource_group_name=resourceGroup,
            subscription_id=subscriptionId,
            tags,
            properties
    "#}
        .to_owned();

        let route_tables = ResourceGraphHelper::new(
            query,
            Some(self.cache_key()),
        )
        .collect_all::<RouteTable>()
        .await?;
        info!("Found {} route tables", route_tables.len());
        Ok(route_tables)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(RouteTableListRequest);

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_route_tables().await?;
        assert!(!result.is_empty());
        println!("Found {} route tables:", result.len());
        for route_table in result {
            assert!(!route_table.name.is_empty());
            println!(" - {:#?}", route_table);
        }
        Ok(())
    }
}
