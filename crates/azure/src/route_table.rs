use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RouteTable;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct RouteTableListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_route_tables(tenant_id: AzureTenantId) -> RouteTableListRequest {
    RouteTableListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for RouteTableListRequest {
    type Output = Vec<RouteTable>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "route_tables",
            self.tenant_id.to_string().as_str(),
        ]))
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

        let route_tables = ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
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
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_route_tables(get_test_tenant_id().await?).await?;
        assert!(!result.is_empty());
        for route_table in result {
            assert!(!route_table.name.is_empty());
        }
        Ok(())
    }
}
