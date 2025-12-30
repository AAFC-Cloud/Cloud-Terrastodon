use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;
use std::time::Duration;

pub fn get_azure_devops_configuration_list_command() -> CommandBuilder {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "configure", "--list"]);
    cmd.use_cache_behaviour(Some(CacheKey {
        path: PathBuf::from_iter(["az", "devops", "configure", "--list"]),
        valid_for: Duration::MAX,
    }));
    cmd
}
