use cloud_terrastodon_azure_types::prelude::EntraGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct GroupListRequest;

pub fn fetch_all_groups() -> GroupListRequest {
    GroupListRequest
}

#[async_trait]
impl CacheableCommand for GroupListRequest {
    type Output = Vec<EntraGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter(["az", "ad", "group", "list"]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching Azure AD groups");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["ad", "group", "list", "--output", "json"]);
        cmd.cache(self.cache_key());
        let rtn: Vec<EntraGroup> = cmd.run().await?;
        debug!("Found {} groups", rtn.len());
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GroupListRequest);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_groups() -> Result<()> {
        let result = fetch_all_groups().await?;
        println!("Found {} groups:", result.len());
        for group in result {
            println!("- {} ({})", group.display_name, group.id);
        }
        Ok(())
    }
}
