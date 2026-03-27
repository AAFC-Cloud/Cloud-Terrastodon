use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::MicrosoftGraphOrganization;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use eyre::ensure;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct AzureTenantDetailsRequest(pub AzureTenantId);

pub fn fetch_azure_tenant_details(tenant_id: AzureTenantId) -> AzureTenantDetailsRequest {
    AzureTenantDetailsRequest(tenant_id)
}

#[async_trait]
impl CacheableCommand for AzureTenantDetailsRequest {
    type Output = MicrosoftGraphOrganization;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "organization",
            self.0.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let url = "https://graph.microsoft.com/v1.0/organization";
        let resp = MicrosoftGraphHelper::new(self.0, url, Some(self.cache_key()))
            .fetch_all::<MicrosoftGraphOrganization>()
            .await?;
        ensure!(
            resp.len() == 1,
            "Expected exactly one organization in response, got {}",
            resp.len()
        );
        Ok(resp.into_iter().next().unwrap())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureTenantDetailsRequest);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_azure_tenant_details;
    use crate::prelude::list_tracked_tenants;
    use eyre::ensure;
    use std::collections::HashSet;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenants = list_tracked_tenants().await?;
        ensure!(!tenants.is_empty(), "Expected at least one tracked tenant");
        let mut seen = HashSet::new();
        for tenant_id in tenants {
            let details = fetch_azure_tenant_details(tenant_id).await?;
            let unique = seen.insert(details.entity.id.clone());
            ensure!(unique, "Duplicate tenant ID found: {}", details.entity.id);
        }
        Ok(())
    }
}
