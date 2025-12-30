use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::ServicePrincipal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct ServicePrincipalListRequest;

pub fn fetch_all_service_principals() -> ServicePrincipalListRequest {
    ServicePrincipalListRequest
}

#[async_trait]
impl CacheableCommand for ServicePrincipalListRequest {
    type Output = Vec<ServicePrincipal>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principals",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching service principals");
        let query = MicrosoftGraphHelper::new(
            "https://graph.microsoft.com/v1.0/servicePrincipals",
            Some(self.cache_key()),
        );
        let entries: Vec<ServicePrincipal> = query.fetch_all().await?;
        debug!("Found {} service principals", entries.len());
        Ok(entries)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ServicePrincipalListRequest);

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_service_principals;
    use cloud_terrastodon_azure_types::prelude::ServicePrincipal;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let found: Vec<ServicePrincipal> = fetch_all_service_principals().await?;
        println!("Found {} service principals", found.len());
        assert!(found.len() > 10);
        Ok(())
    }
}
