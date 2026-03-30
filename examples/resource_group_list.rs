// Prints each Azure Resource Group name using the re-exported Azure module.

use cloud_terrastodon::azure::fetch_all_resource_groups;
use cloud_terrastodon::azure::get_default_tenant_id;
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    // Fetch info
    let tenant_id = get_default_tenant_id().await?;
    let resource_groups = fetch_all_resource_groups(tenant_id).await?;

    // Print each resource group with its subscription name
    for resource_group in resource_groups {
        println!(
            "{subscription_name} - {resource_group_name} ({full_id})",
            subscription_name = resource_group.subscription_name,
            resource_group_name = resource_group.name,
            full_id = resource_group.id
        );
    }
    Ok(())
}
