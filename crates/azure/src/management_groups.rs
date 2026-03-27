use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::ManagementGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use eyre::bail;
use indoc::indoc;
use std::path::PathBuf;
use tracing::error;
use tracing::info;

pub async fn fetch_root_management_group(tenant_id: AzureTenantId) -> Result<ManagementGroup> {
    info!("Fetching root management group");
    let found = fetch_all_management_groups(tenant_id)
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

#[must_use = "This is a future request, you must .await it"]
pub struct ManagementGroupListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_management_groups(tenant_id: AzureTenantId) -> ManagementGroupListRequest {
    ManagementGroupListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ManagementGroupListRequest {
    type Output = Vec<ManagementGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "management_groups",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
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

        let management_groups =
            ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
                .collect_all::<ManagementGroup>()
                .await?;
        info!("Found {} management groups", management_groups.len());
        Ok(management_groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ManagementGroupListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_management_groups(get_test_tenant_id().await?).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}
