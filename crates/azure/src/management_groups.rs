use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::ManagementGroup;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use eyre::bail;
use indoc::indoc;
use std::path::PathBuf;
use std::time::Duration;
use tracing::error;
use tracing::info;

pub async fn fetch_root_management_group() -> Result<ManagementGroup> {
    info!("Fetching root management group");
    let found = fetch_all_management_groups()
        .await?
        .into_iter()
        .find(|mg| mg.name() == mg.tenant_id.to_string());
    match found {
        Some(management_group) => {
            info!("Found root management group");
            Ok(management_group)
        }
        None => {
            let msg = "Failed to find a management group with name matching the tenant ID";
            error!(msg);
            bail!(msg);
        }
    }
}

pub async fn fetch_all_management_groups() -> Result<Vec<ManagementGroup>> {
    info!("Fetching management groups");
    let query = indoc! {r#"
        resourcecontainers
        | where type =~ "Microsoft.Management/managementGroups"
        | project 
            tenant_id=tenantId,
            id,
            display_name=properties.displayName,
            management_group_ancestors_chain=properties.details.managementGroupAncestorsChain
    "#};

    let management_groups = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from_iter(["az", "resource_graph", "management_groups"]),
            valid_for: Duration::from_hours(8),
        },
    )
    .collect_all::<ManagementGroup>()
    .await?;
    info!("Found {} management groups", management_groups.len());
    Ok(management_groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_management_groups().await?;
        println!("Found {} management groups:", result.len());
        for mg in result {
            println!("{:?}", mg);
        }
        Ok(())
    }
}
