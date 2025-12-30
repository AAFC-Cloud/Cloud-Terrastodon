use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::ConditionalAccessNamedLocation;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct ConditionalAccessNamedLocationListRequest;

pub fn fetch_all_conditional_access_named_locations() -> ConditionalAccessNamedLocationListRequest {
    ConditionalAccessNamedLocationListRequest
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
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let query = MicrosoftGraphHelper::new(
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
    use crate::prelude::fetch_all_conditional_access_named_locations;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let found = fetch_all_conditional_access_named_locations().await?;
        println!("Found {} entries", found.len());
        println!("{:#?}", found);
        Ok(())
    }
}
