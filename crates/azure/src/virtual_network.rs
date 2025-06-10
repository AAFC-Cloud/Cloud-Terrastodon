use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::VirtualNetwork;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn fetch_all_virtual_networks() -> Result<Vec<VirtualNetwork>> {
    info!("Fetching virtual networks");    let query = indoc! {r#"
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

    let virtual_networks = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from("virtual_networks"),
            valid_for: Duration::from_hours(8),
        },
    )
    .collect_all::<VirtualNetwork>()
    .await?;
    info!("Found {} virtual networks", virtual_networks.len());
    Ok(virtual_networks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let result = fetch_all_virtual_networks().await?;
        assert!(!result.is_empty());
        println!("Found {} virtual networks:", result.len());
        for vnet in result {
            assert!(!vnet.name.is_empty());
            println!(" - {} {:?}", vnet.name, vnet.properties.address_space);
        }
        Ok(())
    }
}
