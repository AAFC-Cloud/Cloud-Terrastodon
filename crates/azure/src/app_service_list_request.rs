use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureAppServiceResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct AppServiceListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_app_services(tenant_id: AzureTenantId) -> AppServiceListRequest {
    AppServiceListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for AppServiceListRequest {
    type Output = Vec<AzureAppServiceResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "app_services",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.tenant_id, "Fetching app services");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.web/sites"
        | project
            id,
            tenantId,
            name,
            kind,
            location,
            tags,
            properties
        "#}
        .to_owned();

        let app_services = ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
            .collect_all::<AzureAppServiceResource>()
            .await?;
        info!(count = app_services.len(), "Fetched app services");
        Ok(app_services)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AppServiceListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_app_services(get_test_tenant_id().await?).await?;
        for app_service in &result {
            assert!(!app_service.name.is_empty());
        }
        Ok(())
    }
}
