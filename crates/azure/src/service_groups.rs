use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::ServiceGroup;
use cloud_terrastodon_command::CacheKey;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_all_service_groups() -> Result<Vec<ServiceGroup>> {
    debug!("Fetching service groups");
    let query = indoc! {r#"
        resourcecontainers
        | where type =~ "microsoft.management/servicegroups"
        | project
            id,
            name,
            properties
    "#}
    .to_owned();

    let service_groups = ResourceGraphHelper::new(
        query,
        Some(CacheKey {
            path: PathBuf::from_iter(["az", "resource_graph", "service_groups"]),
            valid_for: Duration::MAX,
        }),
    )
    .collect_all::<ServiceGroup>()
    .await?;
    debug!("Found {} service groups", service_groups.len());
    Ok(service_groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let result = fetch_all_service_groups().await?;
        for sg in &result {
            println!("service group {}", sg.name);
        }
        Ok(())
    }
}
