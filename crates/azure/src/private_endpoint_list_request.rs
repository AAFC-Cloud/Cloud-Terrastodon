use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzurePrivateEndpointResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct PrivateEndpointListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_private_endpoints(tenant_id: AzureTenantId) -> PrivateEndpointListRequest {
    PrivateEndpointListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for PrivateEndpointListRequest {
    type Output = Vec<AzurePrivateEndpointResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "private_endpoints",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.tenant_id, "Fetching private endpoints");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/privateendpoints"
        | project
            id,
            tenantId,
            name,
            location,
            tags,
            properties
        "#}
        .to_owned();

        let private_endpoints =
            ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
                .collect_all::<AzurePrivateEndpointResource>()
                .await?;
        info!(count = private_endpoints.len(), "Fetched private endpoints");
        Ok(private_endpoints)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PrivateEndpointListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_private_endpoints(get_test_tenant_id().await?).await?;
        for private_endpoint in &result {
            assert!(!private_endpoint.name.is_empty());
        }
        Ok(())
    }
}
