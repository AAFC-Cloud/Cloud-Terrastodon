use crate::prelude::QueryBuilder;
use anyhow::Result;
use azure_types::prelude::ResourceGroup;
use indoc::indoc;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_resource_groups() -> Result<Vec<ResourceGroup>> {
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

    let rgs = QueryBuilder::new(
        query,
        command::prelude::CacheBehaviour::Some {
            path: PathBuf::from("resource_groups"),
            valid_for: Duration::from_hours(8),
        },
    )
    .collect_all::<ResourceGroup>()
    .await?;

    Ok(rgs)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let result = fetch_all_resource_groups().await?;
        assert!(result.len() > 0);
        println!("Found {} resource groups:", result.len());
        for rg in result {
            assert!(!rg.name.is_empty());
            println!(" - {} (sub={})", rg.name, rg.subscription_id);
        }
        Ok(())
    }
}
