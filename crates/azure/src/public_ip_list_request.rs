use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzurePublicIpResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct PublicIpListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_public_ips(tenant_id: AzureTenantId) -> PublicIpListRequest {
    PublicIpListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for PublicIpListRequest {
    type Output = Vec<AzurePublicIpResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "public_ips",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.tenant_id, "Fetching public IP addresses");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/publicipaddresses"
        | project
            id,
            tenantId,
            name,
            sku,
            location,
            tags,
            properties
        "#}
        .to_owned();

        let public_ips = ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
            .collect_all::<AzurePublicIpResource>()
            .await?;
        info!(count = public_ips.len(), "Fetched public IP addresses");
        Ok(public_ips)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PublicIpListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_public_ips(get_test_tenant_id().await?).await?;
        for public_ip in &result {
            assert!(!public_ip.name.is_empty());
        }
        Ok(())
    }
}
