use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::EntraServicePrincipal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct ServicePrincipalListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_service_principals(tenant_id: AzureTenantId) -> ServicePrincipalListRequest {
    ServicePrincipalListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ServicePrincipalListRequest {
    type Output = Vec<EntraServicePrincipal>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principals",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching service principals");
        let query = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/v1.0/servicePrincipals",
            Some(self.cache_key()),
        );
        let entries: Vec<EntraServicePrincipal> = query.fetch_all().await?;
        debug!("Found {} service principals", entries.len());
        Ok(entries)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ServicePrincipalListRequest);

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_service_principals;
    use crate::prelude::get_test_tenant_id;
    use cloud_terrastodon_azure_types::prelude::EntraServicePrincipal;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let found: Vec<EntraServicePrincipal> =
            fetch_all_service_principals(get_test_tenant_id().await?).await?;
        assert!(found.len() > 10);
        Ok(())
    }
}
