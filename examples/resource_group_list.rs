// Prints each Azure Resource Group name using the re-exported Azure module.

use cloud_terrastodon::azure::fetch_all_resource_groups;
use cloud_terrastodon::azure::fetch_all_subscriptions;
use cloud_terrastodon::azure::get_default_tenant_id;
use color_eyre::eyre::Result;
use std::collections::HashMap;
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(error) = color_eyre::install() {
        eprintln!("Failed to install color_eyre: {error:?}");
    }

    // Fetch info in parallel
    let tenant_id = get_default_tenant_id().await?;
    let (resource_groups, subscriptions) = try_join!(
        fetch_all_resource_groups(tenant_id),
        fetch_all_subscriptions(tenant_id)
    )?;

    // Create lookup table for subscription names
    let subscriptions = subscriptions
        .into_iter()
        .map(|s| (s.id, s))
        .collect::<HashMap<_, _>>();

    // Print each resource group with its subscription name
    for resource_group in resource_groups {
        let subscription_name = subscriptions
            .get(&resource_group.subscription_id)
            .map_or("<unknown subscription>", |s| s.name.as_str());
        println!(
            "{subscription_name} - {resource_group_name} ({full_id})",
            resource_group_name = resource_group.name,
            full_id = resource_group.id
        );
    }
    Ok(())
}
