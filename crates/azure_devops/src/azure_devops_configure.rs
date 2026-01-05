use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::bstr::ByteSlice;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct AzureDevOpsCliConfigRequest;

pub fn get_azure_devops_cli_config() -> AzureDevOpsCliConfigRequest {
    AzureDevOpsCliConfigRequest
}

#[async_trait]
impl CacheableCommand for AzureDevOpsCliConfigRequest {
    type Output = String;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter(["az", "devops", "config", "list"]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["devops", "configure", "--list"]);
        cmd.cache(self.cache_key());
        let rtn = cmd.run_raw().await?;
        let stdout = rtn.stdout.to_str()?.to_string();
        Ok(stdout)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsCliConfigRequest);

#[cfg(test)]
mod test {
    use crate::prelude::get_azure_devops_cli_config;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let config = get_azure_devops_cli_config().await?;
        println!("Config:\n{config}");
        Ok(())
    }
}
