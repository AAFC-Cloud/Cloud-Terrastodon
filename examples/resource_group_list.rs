// Prints each Azure Resource Group name using the re-exported Azure module.
// Run with: cargo run --example resource_group_list --features azure

use cloud_terrastodon::azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_entrypoint::tracing::Level;
use cloud_terrastodon_entrypoint::tracing::init_tracing;
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing(Level::DEBUG)?;
    color_eyre::install().ok();

    let rgs = fetch_all_resource_groups().await?;
    for rg in rgs {
        println!("{}", rg.id.expanded_form());
    }
    Ok(())
}
