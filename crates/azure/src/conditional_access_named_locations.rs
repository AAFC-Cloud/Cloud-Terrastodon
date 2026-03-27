use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ConditionalAccessNamedLocation;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct ConditionalAccessNamedLocationListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_conditional_access_named_locations(
    tenant_id: AzureTenantId,
) -> ConditionalAccessNamedLocationListRequest {
    ConditionalAccessNamedLocationListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ConditionalAccessNamedLocationListRequest {
    type Output = Vec<ConditionalAccessNamedLocation>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "conditional_access_named_locations",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let query = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/beta/identity/conditionalAccess/namedLocations",
            Some(self.cache_key()),
        );

        let found = query.fetch_all().await?;
        Ok(found)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ConditionalAccessNamedLocationListRequest);

#[cfg(test)]
mod test {
    use crate::fetch_all_conditional_access_named_locations;
    use crate::get_test_tenant_id;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let found =
            fetch_all_conditional_access_named_locations(get_test_tenant_id().await?).await?;
        assert!(!found.is_empty());
        Ok(())
    }
}
