use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;
use std::time::Duration;

pub fn get_azure_devops_configuration_command() -> CommandBuilder {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "configure", "--list"]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "devops", "configure", "--list"]),
        valid_for: Duration::from_hours(8),
    });
    cmd
}
