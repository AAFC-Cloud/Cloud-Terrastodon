use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::ResourceGroup;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_all_resource_groups() -> Result<Vec<ResourceGroup>> {
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
