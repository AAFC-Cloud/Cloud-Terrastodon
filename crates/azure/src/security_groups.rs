use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::EntraGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct SecurityGroupListRequest;

pub fn fetch_all_security_groups() -> SecurityGroupListRequest {
    SecurityGroupListRequest
}

#[async_trait]
impl CacheableCommand for SecurityGroupListRequest {
    type Output = Vec<EntraGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter(["ms", "graph", "GET", "security_groups"]),
            valid_for: Duration::from_hours(2),
        }
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching security groups");
        let query = MicrosoftGraphHelper::new(
            "https://graph.microsoft.com/v1.0/groups?$select=id,displayName,description,securityEnabled,isAssignableToRole&$filter=securityEnabled eq true",
            Some(self.cache_key()),
        );
        let groups: Vec<EntraGroup> = query.fetch_all().await?;
        debug!("Found {} security groups", groups.len());
        Ok(groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(SecurityGroupListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::fetch_all_security_groups;
    use cloud_terrastodon_azure_types::prelude::EntraGroup;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let groups: Vec<EntraGroup> = fetch_all_security_groups().await?;
        println!("Found {} groups", groups.len());
        assert!(groups.len() > 1);
        Ok(())
    }
}
