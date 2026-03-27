use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::ServiceGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct ServiceGroupListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_service_groups(tenant_id: AzureTenantId) -> ServiceGroupListRequest {
    ServiceGroupListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ServiceGroupListRequest {
    type Output = Vec<ServiceGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "service_groups",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
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

        let service_groups =
            ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
                .collect_all::<ServiceGroup>()
                .await?;
        debug!("Found {} service groups", service_groups.len());
        Ok(service_groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ServiceGroupListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let result = fetch_all_service_groups(get_test_tenant_id().await?).await?;
        assert!(!result.is_empty());
        assert!(result.iter().all(|sg| !sg.name.is_empty()));
        Ok(())
    }
}
