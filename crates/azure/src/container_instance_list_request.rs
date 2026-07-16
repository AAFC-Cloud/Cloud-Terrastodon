use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureContainerInstanceResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct ContainerInstanceListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_container_instances(tenant_id: AzureTenantId) -> ContainerInstanceListRequest {
    ContainerInstanceListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ContainerInstanceListRequest {
    type Output = Vec<AzureContainerInstanceResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "container_instances",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.tenant_id, "Fetching Azure container instances");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.containerinstance/containergroups"
        | project
            id,
            tenantId,
            name,
            location,
            tags,
            identity,
            properties
        "#}
        .to_owned();

        let container_instances =
            ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
                .collect_all::<AzureContainerInstanceResource>()
                .await?;
        info!(
            count = container_instances.len(),
            "Fetched Azure container instances"
        );
        Ok(container_instances)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ContainerInstanceListRequest);
cloud_terrastodon_registry::register_thing!(ContainerInstanceListRequest);
cloud_terrastodon_registry::register_arbitrary!(ContainerInstanceListRequest);
cloud_terrastodon_registry::register_into_future!(
    ContainerInstanceListRequest => Vec<AzureContainerInstanceResource>
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_container_instances(get_test_tenant_id().await?).await?;
        for container_instance in &result {
            assert!(!container_instance.name.is_empty());
        }
        Ok(())
    }
}
