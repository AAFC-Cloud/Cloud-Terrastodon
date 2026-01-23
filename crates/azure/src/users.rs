use cloud_terrastodon_azure_types::prelude::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct UserListRequest;

pub fn fetch_all_users() -> UserListRequest {
    UserListRequest
}

#[async_trait]
impl CacheableCommand for UserListRequest {
    type Output = Vec<EntraUser>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter(["az", "ad", "user", "list"]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching users");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["ad", "user", "list", "--output", "json"]);
        cmd.cache(self.cache_key());
        let users: Vec<EntraUser> = cmd.run().await?;
        debug!("Found {} users", users.len());
        Ok(users)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(UserListRequest);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_users().await?;
        println!("Found {} users:", result.len());
        for group in result {
            println!("- {} ({})", group.display_name, group.id);
        }
        Ok(())
    }
}
