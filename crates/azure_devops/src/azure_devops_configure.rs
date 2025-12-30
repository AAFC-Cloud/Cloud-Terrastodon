use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;

pub fn get_azure_devops_configuration_list_command() -> CommandBuilder {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "configure", "--list"]);
    cmd.use_cache_behaviour(Some(CacheKey::new(PathBuf::from_iter([
        "az", "devops", "config", "list",
    ]))));
    cmd
}
