use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::VirtualNetwork;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

pub struct VirtualNetworkListRequest;

pub fn fetch_all_virtual_networks() -> VirtualNetworkListRequest {
    VirtualNetworkListRequest
}

#[async_trait]
impl CacheableCommand for VirtualNetworkListRequest {
    type Output = Vec<VirtualNetwork>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "virtual_networks",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        info!("Fetching virtual networks");
        let query = indoc! {r#"
            Resources
            | where type == "microsoft.network/virtualnetworks"
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

        let virtual_networks = ResourceGraphHelper::new(query, Some(self.cache_key()))
            .collect_all::<VirtualNetwork>()
            .await?;
        info!("Found {} virtual networks", virtual_networks.len());
        Ok(virtual_networks)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(VirtualNetworkListRequest);

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_virtual_networks().await?;
        assert!(!result.is_empty());
        println!("Found {} virtual networks:", result.len());
        for vnet in result {
            assert!(!vnet.name.is_empty());
            println!("{:#?}", vnet);
        }
        Ok(())
    }
}
