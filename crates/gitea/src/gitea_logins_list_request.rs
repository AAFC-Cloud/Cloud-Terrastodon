use crate::GiteaLogin;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use std::time::Duration;

#[must_use = "This is a future request, you must .await it"]
pub struct GiteaLoginsListRequest;

pub fn list_gitea_logins() -> GiteaLoginsListRequest {
    GiteaLoginsListRequest
}

#[async_trait]
impl CacheableCommand for GiteaLoginsListRequest {
    type Output = Vec<GiteaLogin>;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter(["tea", "logins", "list"]),
            valid_for: Duration::from_secs(5),
        }
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::Gitea);
        cmd.args(["logins", "list", "--output", "json"]);
        cmd.cache(self.cache_key());
        cmd.run().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaLoginsListRequest);
