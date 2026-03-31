use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureApplicationGatewayResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct ApplicationGatewayListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_application_gateways(tenant_id: AzureTenantId) -> ApplicationGatewayListRequest {
    ApplicationGatewayListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ApplicationGatewayListRequest {
    type Output = Vec<AzureApplicationGatewayResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "application_gateways",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.tenant_id, "Fetching application gateways");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/applicationgateways"
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

        let application_gateways =
            ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
                .collect_all::<AzureApplicationGatewayResource>()
                .await?;
        info!(
            count = application_gateways.len(),
            "Fetched application gateways"
        );
        Ok(application_gateways)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationGatewayListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_application_gateways(get_test_tenant_id().await?).await?;
        for application_gateway in &result {
            assert!(!application_gateway.name.is_empty());
        }
        Ok(())
    }
}
